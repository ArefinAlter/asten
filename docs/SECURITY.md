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
- The cookie is also _honoured_ only on same-origin requests: when an `Origin`
  header is present it must name the host the request reached (the `Host`
  header, or the HTTP/2 authority), or the request is answered `401` as if it
  carried nothing. `SameSite=Strict` keeps other sites out, but not other
  ports of the same host or sibling subdomains, and a form posted from one of
  those would otherwise ride the cookie into routes that take no body, such as
  a snapshot restore. A bearer header is never attached by a browser on its
  own, so it works from any origin the CORS allowlist admits.
- A client label (the name shown in the Guests panel) is at most eighty
  characters; a longer one is refused before the code is even checked.
- Single use is enforced by the store: two racing redemptions cannot both
  succeed. Wrong guesses are counted atomically.

### Tokens

- Format `ast_` + base64url of 32 random bytes. Shown to the client exactly once.
- Only the SHA-256 hash is stored. Verification is constant-time.
- Each token has a kind (`desk`, `guest`, `owner`), a label, a creation time and
  a last-used time. The Guests panel lists and revokes them. Revocation is
  immediate for every request that follows, including the redemption of a
  stream ticket minted before it; a change stream that was already open ends
  when its connection does.
- The desktop shell revokes its previous desk tokens at start-up and issues a
  fresh one for the webview.
- Tokens travel in the `Authorization: Bearer` header or the session cookie.
  A bearer value must be the `ast_` prefix and base64url characters and
  nothing else, or it is not consulted at all. The scheme is case-insensitive.
  A cookie jar split over several `Cookie` fields, as HTTP/2 allows, is read
  whole. Tokens never appear in a URL.
- The Server-Sent Events stream cannot send headers from a browser, so an
  authenticated client first asks `POST …/stream-ticket` for a ticket: random,
  hashed at rest, bound to one project, valid for sixty seconds, spent on first
  use, and refused if the token that asked for it has been revoked since. Only
  that ticket goes in the query string, so a reverse proxy's access log holds
  nothing worth stealing.
- Request tracing spans record the method and path only, never the query
  string.
- `GET /desk` tells a paired client the API version and host. The database
  path is included only for `desk` and owner `session` tokens.
- The browser desk registers a service worker for installability on phones. It
  caches only the shell's static files and never a response under `/api/`, so
  the browser cache holds no manuscript, canon or token.
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
and the API keys clients by `X-Forwarded-For` instead: the rightmost hop that
is not itself a trusted proxy, which is the one the proxy appended. The
leftmost hop is whatever the client chose to send and is ignored, so a client
cannot give itself a fresh bucket per request. IPv4 clients of a dual-stack
listener arrive as `::ffff:a.b.c.d` and are keyed, and matched against the
proxy list, as the IPv4 addresses they are rather than as one shared /64.
Never trust a proxy the client can bypass.

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
  `Permissions-Policy` and a `Content-Security-Policy`. Each response carries
  an `x-request-id`. With `ASTEN_COOKIE_SECURE=true` every response also
  carries `Strict-Transport-Security: max-age=31536000`.
- **API responses** (`/api/…`, `/health`) are `Cache-Control: no-store`, so
  manuscripts never land in a disk cache, and sit under
  `default-src 'none'; frame-ancestors 'none'`: they are data, never a page.
- **The served desk** (`ASTEN_SERVE_DESK`) gets a policy fitted to it. Scripts
  come from its own origin plus the one inline bootstrap in `index.html` that
  applies the saved theme before paint, which is allowed by its SHA-256 hash,
  computed from the served file at start-up, never by `'unsafe-inline'`.
  Styles come from its origin and inline, because the editor injects a
  `<style>` element at start-up. Fonts come from its origin and `data:`;
  images from its origin, `data:` and `blob:`; connections go to its origin
  only; there are no frames, objects, base URIs or foreign form targets, and
  `frame-ancestors 'none'`. The hashed files under `/assets/` are
  content-addressed, so they are `public, max-age=31536000, immutable`; a
  missing asset is a `404`, never the shell page, so nothing wrong is ever
  cached under an asset's name. The shell itself (`index.html`, `sw.js`, the
  manifest, the icons) is `no-cache`: kept, but revalidated on every load, so
  a new build reaches the browser at once.
- **Desktop CSP.** The Tauri webview allows connections to `127.0.0.1` and
  `localhost` only, and no objects, base URIs or foreign form targets.

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

1. Put TLS in front of it and set `ASTEN_COOKIE_SECURE=true`. That also turns
   on HSTS for a year, so do it on a host name that will keep its TLS.
2. Use `ASTEN_AUTH=owner` once a passphrase is set, or stay in `pairing` and
   mint codes from a paired desk.
3. Add your desk's origin to `ASTEN_ALLOWED_ORIGINS` if it is not served by
   the API itself (`ASTEN_SERVE_DESK`).
4. Keep the host port bound to an address only you can reach.
5. Name the reverse proxy in `ASTEN_TRUSTED_PROXIES` so rate limits apply per
   client rather than to the proxy as a whole.

## Hosted pre-alpha (visitor sandboxes)

The public pre-alpha at `https://asten.103.191.50.32.sslip.io` is not a mode of
the API; it is a host for it, `services/asten-prealpha`, built the way the
desktop shell hosts the API in-process (D6, D29). Nothing in the API's rules
changes.

- **One API per visitor.** The first request without a live `asten_visitor`
  cookie creates a visitor. Each visitor gets a separate SQLite file and a
  separate `asten_api::router` over it, in `pairing` mode. No two visitors
  share a database, a token table or a change stream.
- **The gateway is the paired client.** When it opens a sandbox it revokes
  that sandbox's earlier `desk` tokens and issues a fresh one, exactly as the
  desktop shell does at launch, and presents it as a bearer header on the
  visitor's requests. The token never reaches the browser; the visitor holds
  only the cookie. If the desk signs out (which revokes that token), the
  gateway issues a fresh one and retries once.
- **The visitor cookie** is 32 random bytes, `HttpOnly`, `Secure`,
  `SameSite=Strict`, `Path=/`, `Max-Age` 24 hours. Only its SHA-256 is stored.
  It is removed from the request before the API sees it.
- **Origin rule.** As for the session cookie (D24), the visitor cookie counts
  only on same-origin requests: a request whose `Origin` is not the host it
  reached, or that the browser labels `Sec-Fetch-Site: same-site` or
  `cross-site`, is answered `403` before any sandbox is touched. Sibling
  `sslip.io` names are "same-site" to a browser, so `SameSite` alone is not
  enough.
- **Expiry and quotas.** A visitor and everything they made are deleted 24
  hours after the first visit; there are no backups. New visitors are capped
  per client address per hour and in total; a sandbox stops accepting writes
  at 48 MB; the state store caps value size, total size and key count.
- **What the gateway stores beside the sandboxes** (`/svc/v1/state`): the
  desk's own preferences, assistant threads, console history and layout.
  Provider keys are never sent to it. The hosted pre-alpha and the browser
  desk call no provider and keep a typed key in the page's memory only
  (`apps/web/src/workbench/secrets.ts`); the desktop app keeps keys in the shell
  (see "The assistant").
- **No egress.** The gateway container sits on a Docker network with no route
  out; only a forwarding nginx container is reachable, on loopback, behind the
  host's TLS nginx with request-rate and body-size limits. Compile, import and
  every check run inside the sandbox's API; nothing a visitor types makes the
  server fetch anything.
- **Not for manuscripts.** The hosted copy is for trying the workbench. The
  site and the About page say so: real work belongs in the desktop app, where
  the file stays on the writer's machine.

The gateway's own tests (`services/asten-prealpha/tests/gateway.rs`) cover
isolation between visitors, the origin rule, the quotas, the state limits,
sign-out recovery and expiry.

## The assistant (desktop)

Release 0.0.1b (D30). Everything below happens in the desktop shell's Rust
process (`crates/asten-assistant`, the `ai_*` commands); the webview has no
permission to reach a provider and its content-security policy still allows
only loopback.

- **Keys.** Typed once into `ai_set_key` and never returned. Stored in the
  operating system's credential store (Windows Credential Manager, the Secret
  Service, the Keychain) or, where none exists, in a file sealed with
  Argon2id and ChaCha20-Poly1305 that the writer unlocks with a passphrase.
  Held in zeroizing types with no `Serialize`; they never appear in logs,
  events, errors, exports, the ledger or the exact-request preview (which shows
  a placeholder). Tests plant `ZEUS-CANARY-` style markers and search every
  output.
- **Where requests go.** Only to the endpoint of a provider the writer enabled
  and consented to: HTTPS everywhere, plain HTTP only to a loopback address;
  userinfo, query strings and fragments in an endpoint are refused; redirects
  are not followed, so a key cannot follow one; size caps and idle timeouts
  apply to bodies and streams.
- **Consent.** Per provider, tied to the host and a notice version; a different
  host or a new notice asks again. The master switch is off by default;
  offline mode allows only loopback providers; a cap of zero means no call.
- **What is sent.** The writer's question and the thread, the fixed
  instructions, the tool definitions, and what the tools return. Manuscript
  wording is excluded unless the writer attaches a passage; wording quoted in
  a mark is withheld unless that mark is attached. The preview is produced by
  the same builder as the sender and carries the SHA-256 of the body it shows.
- **What it cannot do.** The tool table is closed: ten tools read or prepare
  (explain a mark, search, summarise a source, propose a fact for the writer
  to confirm) and five are closed by rule (anything that would edit the
  manuscript, confirm canon, change settings, run a command or decide an
  approval). An invented or closed tool name is refused and shown. A request
  for prose is refused before any provider is contacted (a classifier), the
  system prompt says it again, the tool gate says it a third time, and an
  output guard withholds a reply that reads like new narrative. Tests drive
  mock providers that try to comply.
- **Not covered.** What a provider does with text it receives is governed by
  that provider's terms. The adapters are tested against mock servers, not
  live services.

## Dependencies

Dependabot (`.github/dependabot.yml`) opens one grouped pull request a week
per ecosystem (Cargo, npm, GitHub Actions) for minor and patch updates. Major
versions of axum, Tauri, sqlx, React, Vite and Tiptap are held back there and
taken deliberately, migration notes first. `cargo audit` and
`pnpm audit --prod` were run for this pass; run both again before a release.

### Known advisories

What `cargo audit` reports that cannot be fixed from this repository, and why
each is accepted:

- **RUSTSEC-2023-0071** `rsa` 0.9 (Marvin timing side channel; no fixed
  version exists). Present in `Cargo.lock` only through `sqlx-mysql`, an
  optional driver that sqlx's macro crate lists and nothing here enables; it
  is never compiled into an Asten binary. Asten speaks SQLite only.
- **RUSTSEC-2025-0075, -0080, -0081, -0098, -0100** `unic-*` 0.9
  (unmaintained). Pulled in by `urlpattern` through `tauri-utils`, at build
  time only, for the desktop shell's configuration; it goes when Tauri drops
  it.
- **RUSTSEC-2024-0370** `proc-macro-error` and **RUSTSEC-2024-0429** `glib`
  0.18 (unmaintained; an unsound iterator). Linux-only GTK bindings beneath
  Tauri's window runtime; not compiled on Windows or macOS, and not reachable
  from the API.
- `yoke-derive` 0.8.3 is yanked. `yoke`, beneath the ICU crates that `url`
  uses, pins it, and no replacement release exists yet; it is a procedural
  macro that runs at build time only.

`zip` moved from 2.4 to 9.0 for RUSTSEC-2026-0367, -0368 and -0369. Asten
never extracts an archive to disk (it reads `word/document.xml` out of a
`.docx` in memory and writes one back), so the advisories did not apply, but
the fixed line is the one that receives updates. `pnpm audit --prod` reports
nothing.

## Reporting

Phone installation requires an HTTPS origin trusted by the phone. Use the
same reverse proxy for the desk and `/api/`; see [Phone setup](PHONE_SETUP.md).
The localhost secure-context exception does not apply to another computer's
plain-HTTP LAN address. Do not disable browser security checks to install.

Report a vulnerability by opening a private security advisory on the GitHub
repository, or by writing to the address in the public site's `security.txt`.
Do not include a manuscript in the report.
