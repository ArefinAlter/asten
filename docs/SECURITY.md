# Security

Asten is local-first. The manuscript, the canon and every diagnostic live in one
SQLite file on the author's machine. There is no account, no sync and no cloud
model. This document describes what the API protects, how, and what it does not
try to protect.

## Threat model

Asten assumes:

- The machine belongs to the author. Code that already runs as the author's
  user (another program, a browser extension with native access) is out of
  scope: it could read `asten.db` directly.
- Web pages the author visits are **not** trusted. The API listens on
  loopback, and any page in any browser on the machine could otherwise call
  `http://127.0.0.1:4317` and read or change the manuscript. This is the main
  threat the authentication layer exists for.
- Other machines on the same network are not trusted. By default the API binds
  to loopback only.
- Verne is not a party. Nothing is sent to Verne. The public site sets no
  cookies and runs no analytics.

Asten does not defend against a hostile operating-system user, a compromised
Office or Google host, or a browser that leaks `localStorage` to other origins.

## Authentication modes

`ASTEN_AUTH` selects the mode. The desktop app always runs the embedded API in
pairing mode.

| Mode      | Who gets in                                           | Use                                    |
| --------- | ----------------------------------------------------- | -------------------------------------- |
| `pairing` | Clients that exchanged a one-time six-digit code       | Default. Desktop, dev, LAN for yourself |
| `owner`   | Paired clients, plus a browser that knows a passphrase | Hosting the API for yourself behind TLS |
| `off`     | Everyone                                               | Loopback only. Refused on other binds   |

The API refuses to start with `ASTEN_AUTH=off` on a non-loopback bind.

### Pairing codes

- Six decimal digits from a CSPRNG, shown once in the desk's **Guests** panel,
  or printed to the API's stderr at start-up when no client is paired yet.
- Stored as a salted SHA-256 hash with a ten-minute expiry.
- Single use. Five wrong attempts lock the code.
- A code grants a token of a stated kind. Codes minted under **Guests** grant
  `guest` tokens; the bootstrap code printed at first start, and codes minted
  with "another desk", grant `desk` tokens.
- Exchanging a code yields a bearer token. When the request is same-origin
  (the desk served by the API itself, or by a proxy in front of both) the
  response also sets an `asten_session` cookie (`HttpOnly`, `SameSite=Strict`,
  `Secure` when `ASTEN_COOKIE_SECURE=true`). A cross-origin desk gets no
  cookie, because browsers would send it to every other port on the host.
- Single use is enforced by the store: two racing redemptions cannot both
  succeed. Wrong guesses are counted atomically.

### Tokens

- Format `ast_` + base64url of 32 random bytes. Shown to the client exactly once.
- Only the SHA-256 hash is stored. Verification is constant-time.
- Each token has a kind (`desk`, `guest`, `owner`), a label, a creation time and
  a last-used time. The Guests panel lists and revokes them. Revocation is
  immediate.
- The desktop shell revokes its previous desk tokens at start-up and issues a
  fresh one for the webview.
- Tokens travel in the `Authorization: Bearer` header or the session cookie.
  Tokens never appear in a URL.
- The Server-Sent Events stream cannot send headers from a browser, so an
  authenticated client first asks `POST …/stream-ticket` for a ticket: random,
  hashed at rest, bound to one project, valid for sixty seconds, spent on first
  use. Only that ticket goes in the query string, so a reverse proxy's access
  log holds nothing worth stealing.
- Request tracing spans record the method and path only, never the query
  string.
- Token kinds carry authority. `desk` and owner `session` tokens may mint
  pairing codes, list and revoke tokens and set or change the passphrase.
  `guest` tokens (Word, Docs, a paired browser pane) may read and write the
  story and revoke only themselves. A guest asking for more gets `403`.

### Owner passphrase

- Optional. At least twelve characters. Hashed with argon2id.
- Setting it for the first time needs a `desk` token, except in `owner` mode
  with no passphrase yet, where the first caller sets it. "Once" is enforced by
  the store with a plain insert, so two racing callers cannot both win.
  Changing it asks for the current passphrase and a `desk` or `session` token.
  `POST /auth/login` is only answered in `owner` mode.
- A successful login issues an ordinary token of kind `owner`.

### Rate limiting

`/auth/pair`, `/auth/login`, `/auth/owner/setup` and `/auth/owner/passphrase`
pass two token buckets: one per client (ten tries, refilling ten a minute) and
one per route shared by everyone (sixty a minute), so a crowd of fresh
addresses cannot multiply the allowance. IPv6 clients are keyed by their /64.
The bucket table is bounded; when full, the least recently seen client is
forgotten first.

Behind a reverse proxy every client arrives from the proxy's address. Name the
proxy in `ASTEN_TRUSTED_PROXIES` (addresses or CIDR blocks, comma-separated)
and the API keys clients by the first `X-Forwarded-For` hop instead. Never
trust a proxy the client can bypass.

## Browser boundary

- **CORS allowlist.** Only the dev desk (`localhost:1420`), the API's own
  origin, the Tauri webview origins, `https://localhost:3000` for the Word
  task pane and `https://*.googleusercontent.com` for Apps Script sidebars are
  allowed by default. `ASTEN_ALLOWED_ORIGINS` adds more. Everything else gets
  no `Access-Control-Allow-Origin` and no private-network approval. Wildcards
  match one or more labels before the dot and nothing else, so
  `evil-googleusercontent.com` is not `*.googleusercontent.com`.
- **Private network access.** `Access-Control-Allow-Private-Network: true` is
  sent only to allowed origins, so Chrome lets the Google sidebar reach
  loopback and nothing else can.
- **Headers on every response.** `X-Content-Type-Options: nosniff`,
  `X-Frame-Options: DENY`, `Referrer-Policy: no-referrer`, a restrictive
  `Permissions-Policy`, and `Cache-Control: no-store` so manuscripts never land
  in a disk cache. Each response carries an `x-request-id`.
- **Desktop CSP.** The Tauri webview allows connections to `127.0.0.1` and
  `localhost` only.

## Transport hardening

- 30-second request timeout, 10 MiB body limit, panic containment, response
  compression, request-id propagation and structured tracing.
- Optimistic concurrency on document commits (`expected_head_seq`), so two
  guests cannot silently overwrite each other.
- Every document keeps a hash-chained event log; `GET …/verify` recomputes the
  chain and the Merkle root of the current block set.

## What is stored

Only `asten.db`:

- Projects, documents, block events and snapshots, facts, entities, events,
  settings, suppressions, drafts, baselines.
- Token hashes, pairing-code hashes and the argon2id owner hash. No plaintext
  secrets.

Nothing is written outside the database directory except the API log on
stderr, which never contains a token.

## Hosting for yourself

If you bind beyond loopback (`ASTEN_BIND=0.0.0.0:4317`, or the Docker compose
file):

1. Put TLS in front of it and set `ASTEN_COOKIE_SECURE=true`.
2. Use `ASTEN_AUTH=owner` once a passphrase is set, or stay in `pairing` and
   mint codes from a paired desk.
3. Add your desk's origin to `ASTEN_ALLOWED_ORIGINS` if it is not served by
   the API itself (`ASTEN_SERVE_DESK`).
4. Keep the host port bound to an address only you can reach.
5. Name the reverse proxy in `ASTEN_TRUSTED_PROXIES` so rate limits apply per
   client rather than to the proxy as a whole.

## Reporting

Report a vulnerability by opening a private security advisory on the GitHub
repository, or by writing to the address in the public site's `security.txt`.
Do not include a manuscript in the report.
