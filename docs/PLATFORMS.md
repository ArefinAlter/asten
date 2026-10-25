# Platforms

Asten is one desk served three ways: a desktop shell that embeds the local
API, a browser desk that talks to that API, and two guests (Word, Google Docs)
that pair with it. This page is the support matrix and the build recipe for
each platform and processor.

## Platform targets

These are configured build targets, not a list of published installers.
Release builds and hardware testing must be recorded before a target is called
verified. Adding a workflow does not demonstrate a successful build.

| Platform                   | Processor         | Desktop shell                 | Browser desk                     | Status                                     |
| -------------------------- | ----------------- | ----------------------------- | -------------------------------- | ------------------------------------------ |
| Windows 10 1809+ / 11      | x64               | NSIS and MSI installers       | Edge, Chrome, Firefox            | Release workflow configured; installer verification pending |
| Windows 11 on ARM          | ARM64             | NSIS installer                | Edge, Chrome                     | Cross-build configured; installer and hardware verification pending |
| macOS 11 Big Sur and later | Apple silicon, Intel | Universal `.app` and `.dmg` | Safari, Chrome, Firefox          | Universal build configured; installer verification pending |
| Linux (glibc 2.35+)        | x86_64            | `.deb`, `.rpm`, AppImage      | Firefox, Chromium                | Release workflow configured; installer verification pending |
| Linux                      | aarch64           | `.deb`, `.rpm`, AppImage      | Firefox, Chromium                | Release job present; needs an ARM runner   |
| Android 8+                 | arm64, x86_64     | Tauri app (see below)         | Chrome, as an installed web app  | Project committed; an unsigned arm64 APK built, with Gradle run by hand on Windows (see below) |
| iOS and iPadOS 15+         | arm64             | Tauri app (see below)         | Safari, as an installed web app  | Shell scaffold; project files generated on a Mac with Xcode |

The browser desk works anywhere a modern browser can reach the API. On a phone
or tablet it is the same desk in a one-column layout; see `docs/PARITY.md` for
what each client can and cannot do.

## What the desktop shell does

The shell opens `asten.db` in the operating system's application-data folder,
starts the API on `127.0.0.1:4317` (or any free port), pairs its own webview
with a desk token, and serves the built desk from the bundle. There is no
platform-specific code in the shell: paths come from Tauri's path resolver, the
API binds loopback the same way on every OS, and SQLite is bundled with the
binary. Processor differences do not reach the code; the Rust crates have no
SIMD or inline-assembly paths.

Per-platform packaging lives beside the main config:

- `apps/desktop/src-tauri/tauri.conf.json`: window, security policy, icons,
  the updater endpoint and public key.
- `tauri.windows.conf.json`: NSIS per-user install, WebView2 bootstrapper, MSI.
- `tauri.macos.conf.json`: minimum macOS 11, DMG layout.
- `tauri.linux.conf.json`: Debian and RPM dependencies, AppImage.

## What the desktop shell keeps for you

Everything lives under the operating system's application-data folder for
Asten (Windows `%APPDATA%\com.asten.app`, macOS
`~/Library/Application Support/com.asten.app`, Linux
`~/.local/share/com.asten.app`). Settings → This desk shows each path with a
copy button.

| Path       | What it holds                                                                                                                                                                                                                                                             |
| ---------- | ------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------------- |
| `asten.db` | The manuscript, the canon and the pairing records. One SQLite file.                                                                                                                                                                                                       |
| `backups/` | A copy of `asten.db` taken at launch, before anything opens the file, named `asten-YYYYMMDD-HHMMSS.db`. Made with SQLite's `VACUUM INTO`, so it is whole even after a bad shutdown. The newest seven are kept; a launch within six hours of the last copy makes none. |
| `logs/`    | One plain-text file a day (`asten.YYYY-MM-DD.log`), the newest seven kept. Request lines name the method and path only; tokens, stream tickets, pairing codes and passphrases never reach a log. `ASTEN_LOG` (in `RUST_LOG` syntax) changes the level.                 |

A second launch of Asten brings the running window forward instead of
starting a second copy, and so a second API server.

### Updates

The desk never checks for updates on its own. Settings → This desk → Check
for updates asks GitHub for `latest.json` from the newest published release,
reports "You have the latest" or "Version X available", and installs and
relaunches only after a second, explicit press. That is the only network call
the desktop app makes beyond loopback.

Updater artifacts are signed with a minisign key pair:

- The public key is in `apps/desktop/src-tauri/tauri.conf.json`
  (`plugins.updater.pubkey`).
- The private key lives outside every repository, on the build owner's
  machine at `B:\margin-keys\asten-updater.key`, with the matching `.pub`
  beside it. It was generated with an empty password. It must never be copied
  into a repository, a worktree, a log or a chat.
- GitHub Actions needs it as two repository secrets:
  `TAURI_SIGNING_PRIVATE_KEY` (the file's contents) and
  `TAURI_SIGNING_PRIVATE_KEY_PASSWORD` (empty, but the secret must exist).
  Without them a release build fails, because `bundle.createUpdaterArtifacts`
  is on.

The endpoint is `https://asten-writing.vercel.app/updates/latest.json`, a file
in the public site (`apps/www/public/updates/latest.json`). GitHub's
`releases/latest/download/…` link skips pre-releases, so it could not serve
the pre-alpha. Publishing a release means writing that file with the new
version, the asset URLs and the contents of their `.sig` files, and deploying
the site; until then the desk keeps reporting the version it has. On Linux
only the AppImage can update itself; `.deb` installs update through the
package manager. Windows ships NSIS only: MSI cannot carry a pre-release
version such as `0.0.1-alpha`.

## Building

All platforms:

```sh
pnpm install
pnpm --filter @asten/desktop build        # tauri build for the host platform
```

Cross-targets on the host that supports them:

```sh
# Windows ARM64 from an x64 Windows machine
rustup target add aarch64-pc-windows-msvc
pnpm --filter @asten/desktop exec tauri build --target aarch64-pc-windows-msvc --bundles nsis

# macOS universal binary (Apple silicon + Intel) from any Mac
rustup target add aarch64-apple-darwin x86_64-apple-darwin
pnpm --filter @asten/desktop exec tauri build --target universal-apple-darwin
```

Linux build dependencies (Debian and Ubuntu):

```sh
sudo apt-get install -y libwebkit2gtk-4.1-dev libayatana-appindicator3-dev \
  librsvg2-dev patchelf libxdo-dev libssl-dev build-essential
```

Fedora: `webkit2gtk4.1-devel libappindicator-gtk3-devel librsvg2-devel openssl-devel`.

### Pre-alpha releases

`v0.0.1a` (`0.0.1-alpha`) was built on the release machine, not in Actions:

```sh
# Windows x64, on Windows (updater key from outside the repository)
export TAURI_SIGNING_PRIVATE_KEY="$(cat B:/margin-keys/asten-updater.key)" TAURI_SIGNING_PRIVATE_KEY_PASSWORD=""
(cd apps/desktop/src-tauri && ../node_modules/.bin/tauri build --bundles nsis)
cargo build --locked --release -p asten-api
node scripts/release-assets.mjs --platform windows-x64 --out release-assets

# Linux x64 (what 0.0.1a used): a clean `git archive` of the commit in src/,
# built in an ubuntu:22.04 container capped at 4 CPUs. No signing key goes in;
# the AppImage is signed for the updater on the release machine afterwards.
docker run --rm --cpus 4 --memory 8g -v "$PWD/src":/src -v "$PWD/out":/out \
  -v "$PWD/linux-build.sh":/build.sh:ro -v asten-build-cache:/cache \
  ubuntu:22.04 nice -n 10 bash /build.sh
apps/desktop/node_modules/.bin/tauri signer sign \
  -f B:/margin-keys/asten-updater.key -p "" out/release/Asten_0.0.1-alpha_amd64.AppImage
# or the same in one BuildKit build, with the key as a build secret:
docker build -f deploy/release/linux.Dockerfile \
  --secret id=tauri_key,src=B:/margin-keys/asten-updater.key \
  --output type=local,dest=deploy/release/out/linux .
```

Then `sha256sum` the assets into `SHA256SUMS.txt`, create the pre-release with
`gh release create v0.0.1a --prerelease`, and update the site's
`updates/latest.json`. `.github/workflows/release-prealpha.yml` does the same
on manual dispatch when Actions minutes allow. The builds are unsigned.

### Release workflow

`.github/workflows/release.yml` runs on a full version tag (`v1.2.3`) or manual dispatch and is
configured to build the desktop targets above and attach successful artifacts
to a draft GitHub release. On manual dispatch, select an existing `v*` tag;
do not use a branch name as a release tag. The Linux
aarch64 job runs on `ubuntu-24.04-arm`, which private repositories must enable
under larger runners; until then that job is allowed to fail.

Signing is off by default. Add these repository secrets to turn it on:

| Secret                                                                              | Effect                                   |
| ----------------------------------------------------------------------------------- | ---------------------------------------- |
| `WINDOWS_CERTIFICATE`, `WINDOWS_CERTIFICATE_PASSWORD`                               | Authenticode-signs the Windows installers |
| `APPLE_CERTIFICATE`, `APPLE_CERTIFICATE_PASSWORD`, `APPLE_SIGNING_IDENTITY`         | Signs the macOS app                      |
| `APPLE_ID`, `APPLE_PASSWORD`, `APPLE_TEAM_ID`                                       | Notarises the macOS app                  |
| `TAURI_SIGNING_PRIVATE_KEY`, `TAURI_SIGNING_PRIVATE_KEY_PASSWORD`                   | Signs the updater artifacts (required)   |

Unsigned builds install; Windows shows a SmartScreen warning and macOS asks
the user to allow the app under Privacy and Security.

## Android and iOS

Tauri 2 builds the same shell for phones. The Rust crate exposes the mobile
entry point and the library types mobile needs (`cdylib` for Android,
`staticlib` for iOS). The updater and the single-instance guard are compiled
out on phones; the store handles updates there.

### Android

The generated project is committed at `apps/desktop/src-tauri/gen/android`
(Gradle 8.14, Android Gradle plugin 8.11, Kotlin 1.9, `compileSdk` 36,
`minSdk` 26). It holds no machine-specific paths. Tauri writes its
`tauri.*.gradle` helpers, `local.properties` and the build outputs at build
time, and the project's own `.gitignore` files keep them out of git.

```sh
# Needs: ANDROID_HOME with platform-tools, build-tools and a platform; an NDK
# (27.2.12479018 is known to work); JAVA_HOME pointing at a JDK 17 or newer
# (JDK 24 runs this Gradle); and the Rust Android targets, which android:init
# installs.
export ANDROID_HOME=/path/to/sdk NDK_HOME=$ANDROID_HOME/ndk/27.2.12479018
pnpm --filter @asten/desktop android:init     # regenerates gen/android when needed
pnpm --filter @asten/desktop android:dev
pnpm --filter @asten/desktop exec tauri android build --apk --target aarch64
```

Status on the Windows build machine, as of this writing: `android:init`
succeeds, and the whole workspace compiles for `aarch64-linux-android` with
NDK 27 (the 15 MB `libasten_desktop_lib.so` comes out of a release build in
about eleven minutes). The build then stops before Gradle runs, because the
Tauri CLI places the library in `jniLibs/` with a symbolic link and Windows
refuses to create one unless Developer Mode is on or the shell is elevated:

> Failed to create a symbolic link ... Creation symbolic link is not allowed
> for this system. For Windows 10 or newer: you should use developer mode.

Turning on Settings → System → For developers → Developer Mode (an
administrator action) removes the block, and the same command then continues
into Gradle. Linux and macOS have no such restriction.

Gradle was then exercised on its own, with the library copied into
`app/src/main/jniLibs/arm64-v8a/` by hand:

```sh
cd apps/desktop/src-tauri/gen/android
./gradlew assembleArm64Release -x rustBuildArm64Release --no-daemon
```

JDK 24 runs it (Kotlin warns that it falls back to a JVM 22 target), Gradle
8.14.3 and the Android Gradle plugin download, and `buildSrc` compiles.
Configuring `:app` first failed with a `java.util.concurrent.TimeoutException`
from Gradle's Kotlin DSL accessor generator. The cause was the disk, not the
project: `GRADLE_USER_HOME` had been pointed at a spinning drive to spare the
nearly full system SSD, the generator writes around two thousand small files
inside a bounded wait, and that drive managed about 35 such writes a second
against the SSD's 1,800. Either keep `GRADLE_USER_HOME` (by default
`~/.gradle`) on an SSD, or lengthen the wait:

```sh
JAVA_TOOL_OPTIONS=-Dorg.gradle.kotlin.dsl.internal.io.timeout=900000 \
  ./gradlew assembleArm64Release -x rustBuildArm64Release --no-daemon
```

With the wait at fifteen minutes the same spinning drive finished the build
in 21 minutes and wrote `app/build/outputs/apk/arm64/release/`
`app-arm64-release-unsigned.apk` (17 MB). It is unsigned, as the name says:
an installable release needs a keystore and a `signingConfigs` block in
`app/build.gradle.kts`, as Tauri's Android signing guide describes;
`keystore.properties` is already git-ignored for it.

### iOS

```sh
# iOS: a Mac with Xcode and an Apple developer team
pnpm --filter @asten/desktop ios:init         # writes apps/desktop/src-tauri/gen/apple
pnpm --filter @asten/desktop ios:dev
pnpm --filter @asten/desktop ios:build
```

The iOS project is not committed, because it can only be created on a Mac
with Xcode.

On a phone the embedded API still binds loopback inside the app's sandbox, the
database lives in the app's data directory, and the desk opens in its
one-column layout: the activity bar runs along the bottom, the binder is a
drawer from the left (the ≡ button), the inspector is a sheet from the right
(the eye button), and the Scales rise from the bottom. Below 1280px wide the
inspector already becomes a sheet; below 960px the whole desk stacks. Touch
targets grow to 40px when the pointer is a finger. Word and Docs guests cannot pair with a phone, because the
phone's API is not reachable from another device; see `docs/PARITY.md`.

### Installed web app instead of a store app

Writers who host the API on their own computer can open the browser desk from
a phone on the same network. Installation and the service worker require an
HTTPS origin with a certificate trusted by the phone. Plain HTTP to another
computer's LAN address supports browser access only; it is not the localhost
development exception. Follow [Phone setup](PHONE_SETUP.md) to proxy the desk
and API together and configure certificate trust before pairing and installing.

## Checks

CI is configured to check the Tauri shell on Windows and Ubuntu on every push, and on macOS
weekly and on demand (Actions → CI → Run workflow), because macOS minutes bill
at ten times the Linux rate (`.github/workflows/ci.yml`, job `desktop`). Release
builds target macOS as well. The Rust workspace and the desk are configured for
checks on Ubuntu. Jobs that never obtain a runner are not successful checks.
Hardware testing for Windows on ARM and Linux aarch64 remains pending;
compile checks alone do not verify an installer or hardware behavior.
