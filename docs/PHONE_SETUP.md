# Install the browser desk on a phone

The phone needs the host computer awake and reachable. This serves the existing
desk and API from one HTTPS origin; it does not sync data to the phone or create
a native mobile build. Phone/device acceptance is still required before claiming
this setup verified on a specific browser.

Service workers require a secure context. `http://localhost` is a development
exception on the device opening it; `http://192.168.x.x:4317` on a different
computer is not. A manifest alone does not make that LAN address installable.

## 1. Build and run the desk on the host

From the repository root on a POSIX host, with Rust, pnpm and Caddy installed:

```sh
pnpm install --frozen-lockfile
pnpm --filter @asten/web build
ASTEN_BIND=127.0.0.1:4317 \
ASTEN_SERVE_DESK=apps/web/dist \
ASTEN_AUTH=pairing \
ASTEN_COOKIE_SECURE=true \
ASTEN_ALLOWED_ORIGINS=https://asten.home.arpa \
ASTEN_TRUSTED_PROXIES=127.0.0.1 \
cargo run --release -p asten-api
```

Keep the backend bound to loopback. Save the first-run pairing code printed in
the host terminal; after a desk is paired, mint subsequent codes in Guests.
Use a code for **another desk** when that client needs to administer pairing.

## 2. Resolve the host and proxy both the desk and API

Give the host a stable LAN address. Configure the LAN DNS/router so
`asten.home.arpa` resolves to that address on the phone and computer. Allow only
your trusted LAN to reach the host's HTTPS port. Keep port 4317 private.

Save this as a Caddyfile on the host:

```caddyfile
asten.home.arpa {
    tls internal
    reverse_proxy 127.0.0.1:4317
}
```

Run `caddy run --config Caddyfile` (or use the host's Caddy service). The proxy
forwards **all** paths, including `/`, `/assets/`, `/sw.js` and `/api/v1/`, to the
same API server. Do not put the desk on HTTPS while fetching its API over HTTP.

## 3. Trust the certificate authority on the phone

Caddy's `tls internal` uses a local CA. Export its **public root certificate**
from the Caddy data directory: `pki/authorities/local/root.crt`. For the standard
Linux service this is usually under
`/var/lib/caddy/.local/share/caddy/`; a user-run Caddy uses that user's data
directory. Confirm the directory for the Caddy process you actually started.

Transfer only `root.crt` privately to your own phone and install it as a trusted
CA using the phone's certificate settings. On iOS, also enable full trust for
the installed root in Certificate Trust Settings. Never transfer `root.key` or
disable TLS verification. Alternatively, use a hostname and certificate already
trusted by the phone, with matching DNS and proxy configuration.

## 4. Pair, install and verify

1. Open `https://asten.home.arpa` on the phone without a certificate warning.
2. Pair with the first-run code or a code minted by an existing desk.
3. Use the browser's Install/Add to Home Screen flow. Names and support vary by
   browser; opening in a tab remains available if installation is unsupported.
4. In browser developer tools (remote inspection if needed), verify
   `window.isSecureContext === true` and an active `/sw.js` registration.
5. Reload once while online so the worker controls and caches the shell. Reopen
   the installed app and confirm it can reach the same authenticated API.
6. Confirm Cache Storage contains only shell resources, with no `/api/` responses.
   Disconnecting the host still prevents manuscript reads/writes; a cached shell
   is not an offline copy of the database.

For authentication, origins and reverse-proxy boundaries, see
[Security](SECURITY.md#hosting-for-yourself).
