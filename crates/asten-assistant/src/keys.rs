//! Where provider keys live.
//!
//! Keys go to the operating system's credential store (Windows Credential
//! Manager, the Secret Service on Linux, the Keychain on macOS) from this
//! process; the page never holds one after the writer has typed it. Where no
//! credential store exists (a headless Linux session, for instance) they go to
//! an encrypted file guarded by a passphrase the writer chooses: Argon2id turns
//! the passphrase into a key, ChaCha20-Poly1305 seals the file.
//!
//! Nothing here implements `Serialize` for a key, and nothing logs one.

use std::{
    collections::BTreeMap,
    path::{Path, PathBuf},
    sync::{Arc, Mutex},
};

use argon2::{Algorithm, Argon2, Params, Version};
use base64::{Engine as _, engine::general_purpose::STANDARD as B64};
use chacha20poly1305::{
    ChaCha20Poly1305, Key, Nonce,
    aead::{Aead, KeyInit},
};
use rand::RngCore;
use serde::{Deserialize, Serialize};
use zeroize::{Zeroize, Zeroizing};

use crate::{
    error::{AssistantError, Result},
    secret::Secret,
};

/// Which store holds the keys.
#[derive(Clone, Copy, Debug, Eq, PartialEq, Serialize)]
#[serde(rename_all = "snake_case")]
pub enum Backend {
    /// The operating system's credential store.
    Os,
    /// An encrypted file guarded by a passphrase.
    File,
    /// Memory only (tests, or when nothing else works and the writer agreed).
    Memory,
}

/// What the interface needs to know about the store.
#[derive(Clone, Debug, Serialize)]
pub struct VaultState {
    /// Which backend is in use.
    pub backend: Backend,
    /// The file store is locked: a passphrase is needed.
    pub locked: bool,
    /// The file store has no file yet: the first passphrase creates it.
    pub needs_setup: bool,
    /// Why the credential store was not used, when it was not.
    pub note: Option<String>,
}

/// A place keys can be kept.
pub trait KeyStore: Send + Sync {
    /// Which backend this is.
    fn backend(&self) -> Backend;
    /// Whether the store can be used right now.
    fn state(&self) -> VaultState;
    /// Unlock with a passphrase (file store only).
    fn unlock(&self, _passphrase: &Secret) -> Result<()> {
        Ok(())
    }
    /// Store a key.
    fn set(&self, provider: &str, key: &Secret) -> Result<()>;
    /// Read a key.
    fn get(&self, provider: &str) -> Result<Option<Secret>>;
    /// Remove a key (no error when there is none).
    fn delete(&self, provider: &str) -> Result<()>;
}

// ───────────────────────── the operating system's store ─────────────────────────

/// The credential store of the operating system.
pub struct OsKeyStore {
    service: String,
}

impl OsKeyStore {
    /// Use `service` as the service name (`Asten` in the app).
    #[must_use]
    pub fn new(service: &str) -> Self {
        Self {
            service: service.to_owned(),
        }
    }

    fn entry(&self, provider: &str) -> Result<keyring::Entry> {
        keyring::Entry::new(&self.service, &format!("provider:{provider}"))
            .map_err(|error| AssistantError::Keys(format!("credential store: {error}")))
    }

    /// Whether the store works: write, read back and remove a probe value.
    pub fn probe(&self) -> Result<()> {
        if let Err(error) = keyring::Entry::store_status() {
            return Err(AssistantError::Keys(format!(
                "no credential store on this system ({error})"
            )));
        }
        let entry = self.entry("probe")?;
        let value = Secret::new(format!("probe-{}", rand::random::<u32>()));
        entry
            .set_password(value.expose())
            .map_err(|error| AssistantError::Keys(format!("credential store: {error}")))?;
        let read = entry
            .get_password()
            .map_err(|error| AssistantError::Keys(format!("credential store: {error}")))?;
        let _ = entry.delete_credential();
        if read == value.expose() {
            Ok(())
        } else {
            Err(AssistantError::Keys(
                "credential store returned a different value".into(),
            ))
        }
    }
}

impl KeyStore for OsKeyStore {
    fn backend(&self) -> Backend {
        Backend::Os
    }

    fn state(&self) -> VaultState {
        VaultState {
            backend: Backend::Os,
            locked: false,
            needs_setup: false,
            note: None,
        }
    }

    fn set(&self, provider: &str, key: &Secret) -> Result<()> {
        self.entry(provider)?
            .set_password(key.expose())
            .map_err(|error| AssistantError::Keys(format!("credential store: {error}")))
    }

    fn get(&self, provider: &str) -> Result<Option<Secret>> {
        match self.entry(provider)?.get_password() {
            Ok(value) => Ok(Some(Secret::new(value))),
            Err(keyring::Error::NoEntry) => Ok(None),
            Err(error) => Err(AssistantError::Keys(format!("credential store: {error}"))),
        }
    }

    fn delete(&self, provider: &str) -> Result<()> {
        match self.entry(provider)?.delete_credential() {
            Ok(()) | Err(keyring::Error::NoEntry) => Ok(()),
            Err(error) => Err(AssistantError::Keys(format!("credential store: {error}"))),
        }
    }
}

// ───────────────────────── the encrypted file ─────────────────────────

#[derive(Serialize, Deserialize)]
struct Sealed {
    v: u32,
    kdf: String,
    m_kib: u32,
    t: u32,
    p: u32,
    salt: String,
    nonce: String,
    ct: String,
}

/// An encrypted file of keys guarded by a passphrase.
pub struct FileKeyStore {
    path: PathBuf,
    note: Option<String>,
    unlocked: Mutex<Option<Unlocked>>,
    /// Argon2 memory cost in KiB; lowered in tests.
    m_kib: u32,
}

struct Unlocked {
    passphrase: Secret,
    keys: BTreeMap<String, String>,
}

impl Drop for Unlocked {
    fn drop(&mut self) {
        for value in self.keys.values_mut() {
            value.zeroize();
        }
    }
}

impl FileKeyStore {
    /// Keep the keys in `path`. `note` says why the file is used.
    #[must_use]
    pub fn new(path: &Path, note: Option<String>) -> Self {
        Self {
            path: path.to_owned(),
            note,
            unlocked: Mutex::new(None),
            m_kib: 19_456,
        }
    }

    /// A cheaper key derivation for tests.
    #[must_use]
    pub fn with_cost(mut self, m_kib: u32) -> Self {
        self.m_kib = m_kib;
        self
    }

    fn derive(
        passphrase: &Secret,
        salt: &[u8],
        m_kib: u32,
        t: u32,
        p: u32,
    ) -> Result<Zeroizing<[u8; 32]>> {
        let params = Params::new(m_kib, t, p, Some(32))
            .map_err(|error| AssistantError::Keys(format!("key derivation: {error}")))?;
        let argon = Argon2::new(Algorithm::Argon2id, Version::V0x13, params);
        let mut out = Zeroizing::new([0u8; 32]);
        argon
            .hash_password_into(passphrase.expose().as_bytes(), salt, out.as_mut())
            .map_err(|error| AssistantError::Keys(format!("key derivation: {error}")))?;
        Ok(out)
    }

    fn read_file(&self, passphrase: &Secret) -> Result<BTreeMap<String, String>> {
        let text = std::fs::read_to_string(&self.path)?;
        let sealed: Sealed = serde_json::from_str(&text)
            .map_err(|_| AssistantError::Keys("the key file is damaged".into()))?;
        if sealed.v != 1 || sealed.kdf != "argon2id" {
            return Err(AssistantError::Keys(
                "the key file is from a newer version".into(),
            ));
        }
        let salt = B64
            .decode(&sealed.salt)
            .map_err(|_| AssistantError::Keys("the key file is damaged".into()))?;
        let nonce = B64
            .decode(&sealed.nonce)
            .map_err(|_| AssistantError::Keys("the key file is damaged".into()))?;
        let ct = B64
            .decode(&sealed.ct)
            .map_err(|_| AssistantError::Keys("the key file is damaged".into()))?;
        if nonce.len() != 12 {
            return Err(AssistantError::Keys("the key file is damaged".into()));
        }
        let key = Self::derive(passphrase, &salt, sealed.m_kib, sealed.t, sealed.p)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
        let plain = cipher
            .decrypt(Nonce::from_slice(&nonce), ct.as_ref())
            .map_err(|_| AssistantError::Keys("wrong passphrase".into()))?;
        let plain = Zeroizing::new(plain);
        serde_json::from_slice(&plain)
            .map_err(|_| AssistantError::Keys("the key file is damaged".into()))
    }

    fn write_file(&self, passphrase: &Secret, keys: &BTreeMap<String, String>) -> Result<()> {
        let mut salt = [0u8; 16];
        let mut nonce = [0u8; 12];
        rand::rng().fill_bytes(&mut salt);
        rand::rng().fill_bytes(&mut nonce);
        let key = Self::derive(passphrase, &salt, self.m_kib, 2, 1)?;
        let cipher = ChaCha20Poly1305::new(Key::from_slice(key.as_ref()));
        let plain = Zeroizing::new(
            serde_json::to_vec(keys).map_err(|error| AssistantError::Keys(error.to_string()))?,
        );
        let ct = cipher
            .encrypt(Nonce::from_slice(&nonce), plain.as_ref())
            .map_err(|_| AssistantError::Keys("could not seal the key file".into()))?;
        let sealed = Sealed {
            v: 1,
            kdf: "argon2id".into(),
            m_kib: self.m_kib,
            t: 2,
            p: 1,
            salt: B64.encode(salt),
            nonce: B64.encode(nonce),
            ct: B64.encode(ct),
        };
        if let Some(dir) = self.path.parent() {
            std::fs::create_dir_all(dir)?;
        }
        let temp = self.path.with_extension("tmp");
        std::fs::write(
            &temp,
            serde_json::to_string(&sealed)
                .map_err(|error| AssistantError::Keys(error.to_string()))?,
        )?;
        #[cfg(unix)]
        {
            use std::os::unix::fs::PermissionsExt;
            let _ = std::fs::set_permissions(&temp, std::fs::Permissions::from_mode(0o600));
        }
        std::fs::rename(&temp, &self.path)?;
        Ok(())
    }

    fn guard(&self) -> std::sync::MutexGuard<'_, Option<Unlocked>> {
        self.unlocked
            .lock()
            .unwrap_or_else(|poisoned| poisoned.into_inner())
    }
}

impl KeyStore for FileKeyStore {
    fn backend(&self) -> Backend {
        Backend::File
    }

    fn state(&self) -> VaultState {
        VaultState {
            backend: Backend::File,
            locked: self.guard().is_none(),
            needs_setup: !self.path.exists(),
            note: self.note.clone(),
        }
    }

    fn unlock(&self, passphrase: &Secret) -> Result<()> {
        if passphrase.expose().chars().count() < 8 {
            return Err(AssistantError::Keys(
                "Use a passphrase of at least eight characters".into(),
            ));
        }
        let keys = if self.path.exists() {
            self.read_file(passphrase)?
        } else {
            BTreeMap::new()
        };
        *self.guard() = Some(Unlocked {
            passphrase: passphrase.clone(),
            keys,
        });
        Ok(())
    }

    fn set(&self, provider: &str, key: &Secret) -> Result<()> {
        let mut guard = self.guard();
        let unlocked = guard.as_mut().ok_or_else(|| {
            AssistantError::Keys("The key file is locked: enter its passphrase".into())
        })?;
        unlocked
            .keys
            .insert(provider.to_owned(), key.expose().to_owned());
        let result = self.write_file(&unlocked.passphrase, &unlocked.keys);
        if result.is_err() {
            unlocked.keys.remove(provider);
        }
        result
    }

    fn get(&self, provider: &str) -> Result<Option<Secret>> {
        let guard = self.guard();
        let unlocked = guard.as_ref().ok_or_else(|| {
            AssistantError::Keys("The key file is locked: enter its passphrase".into())
        })?;
        Ok(unlocked.keys.get(provider).map(|v| Secret::new(v.clone())))
    }

    fn delete(&self, provider: &str) -> Result<()> {
        let mut guard = self.guard();
        let unlocked = guard.as_mut().ok_or_else(|| {
            AssistantError::Keys("The key file is locked: enter its passphrase".into())
        })?;
        if unlocked.keys.remove(provider).is_some() {
            self.write_file(&unlocked.passphrase, &unlocked.keys)?;
        }
        Ok(())
    }
}

// ───────────────────────── memory (tests, last resort) ─────────────────────────

/// Keys in memory only.
#[derive(Default)]
pub struct MemoryKeyStore {
    keys: Mutex<BTreeMap<String, Secret>>,
}

impl KeyStore for MemoryKeyStore {
    fn backend(&self) -> Backend {
        Backend::Memory
    }

    fn state(&self) -> VaultState {
        VaultState {
            backend: Backend::Memory,
            locked: false,
            needs_setup: false,
            note: Some("Keys are held in memory and are gone when Asten closes".into()),
        }
    }

    fn set(&self, provider: &str, key: &Secret) -> Result<()> {
        self.keys
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .insert(provider.to_owned(), key.clone());
        Ok(())
    }

    fn get(&self, provider: &str) -> Result<Option<Secret>> {
        Ok(self
            .keys
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .get(provider)
            .cloned())
    }

    fn delete(&self, provider: &str) -> Result<()> {
        self.keys
            .lock()
            .unwrap_or_else(|p| p.into_inner())
            .remove(provider);
        Ok(())
    }
}

/// Choose the store: the operating system's when it works, else the encrypted
/// file. `ASTEN_KEY_STORE=file` forces the file (used by the tests and by
/// writers who prefer it).
#[must_use]
pub fn detect(service: &str, file: &Path) -> Arc<dyn KeyStore> {
    let forced = std::env::var("ASTEN_KEY_STORE").unwrap_or_default();
    if forced == "memory" {
        return Arc::new(MemoryKeyStore::default());
    }
    if forced != "file" {
        let os = OsKeyStore::new(service);
        match os.probe() {
            Ok(()) => return Arc::new(os),
            Err(error) => {
                tracing::info!(%error, "no usable credential store; keys will use an encrypted file");
                return Arc::new(FileKeyStore::new(file, Some(error.to_string())));
            }
        }
    }
    Arc::new(FileKeyStore::new(
        file,
        Some("Chosen by ASTEN_KEY_STORE".into()),
    ))
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn the_file_store_seals_and_reopens() {
        let dir = tempfile::tempdir().unwrap();
        let path = dir.path().join("keys.enc");
        let store = FileKeyStore::new(&path, None).with_cost(64);
        assert!(store.state().needs_setup);
        assert!(store.set("anthropic", &Secret::new("k".into())).is_err());
        store
            .unlock(&Secret::new("correct horse battery".into()))
            .unwrap();
        store
            .set(
                "anthropic",
                &Secret::new("ASTEN-CANARY-file-key-0001".into()),
            )
            .unwrap();
        let on_disk = std::fs::read_to_string(&path).unwrap();
        assert!(!on_disk.contains("CANARY"), "the key must be sealed");

        let again = FileKeyStore::new(&path, None).with_cost(64);
        assert!(again.state().locked);
        assert!(
            again
                .unlock(&Secret::new("wrong passphrase".into()))
                .is_err()
        );
        again
            .unlock(&Secret::new("correct horse battery".into()))
            .unwrap();
        assert_eq!(
            again.get("anthropic").unwrap().unwrap().expose(),
            "ASTEN-CANARY-file-key-0001"
        );
        again.delete("anthropic").unwrap();
        assert!(again.get("anthropic").unwrap().is_none());
    }

    #[test]
    fn a_short_passphrase_is_refused() {
        let dir = tempfile::tempdir().unwrap();
        let store = FileKeyStore::new(&dir.path().join("k"), None).with_cost(64);
        assert!(store.unlock(&Secret::new("short".into())).is_err());
    }

    #[test]
    fn memory_store_works() {
        let store = MemoryKeyStore::default();
        store.set("a", &Secret::new("value-123456".into())).unwrap();
        assert_eq!(store.get("a").unwrap().unwrap().last4(), "3456");
        store.delete("a").unwrap();
        assert!(store.get("a").unwrap().is_none());
    }
}
