# Asten

Asten is a local-first writing environment with a canon engine. The name is
another name for Thoth, the Egyptian god of writing. The thesis is unchanged:
let the machine be the editor, not the writer.

Asten is a Verne product. The desk is a manuscript editor with a binder,
comments, find, page view, and track changes. The engine reads the page, keeps a
gradual canon of what is settled and what is only proposed, and weighs the two
on the Scales: attribute clashes, life after death, two clocks, routes,
knowledge, relations, author invariants. Every mark explains itself and offers
repairs the author owns. Word and Docs remain guests. No account, no sync. Asten never writes the
prose; any AI feature follows the AI policy (D27).

## Try the pre-alpha

Release [0.0.1c](https://github.com/ArefinAlter/asten/releases/tag/v0.0.1c)
(pre-alpha) has unsigned desktop builds for Windows x64 and Linux x64. The
desktop app runs the engine on your machine, can open a sample novel, imports
Word, Markdown, HTML and plain text, moves a project as one `.asten` file,
backs up its library, and has an optional bring-your-own-key assistant (off by
default; it explains, searches and proposes, never writes, and shows you the
exact request before it is sent). The
same workbench runs at <https://asten.103.191.50.32.sslip.io/> with the real
engine in a private sandbox per visitor, deleted 24 hours after the first
visit; it is for trying Asten, not for manuscripts. How it is hosted:
`deploy/pre-alpha/README.md` and `docs/SECURITY.md`.

## Platforms

Published today: release 0.0.1c for Windows x64 and Linux x64 (unsigned), and
the hosted pre-alpha in any modern browser (it still runs 0.0.1a and has no
assistant). Configured but not yet published:
Windows ARM64, macOS 11+ (one universal build), Linux aarch64, and the Android
and iOS shells. A browser can also talk to the local API. See
`docs/PLATFORMS.md` for the support matrix, build recipes and signing.

## Prerequisites

- Rust 1.85 or newer (edition 2024; CI uses stable)
- Node.js 22 or newer
- pnpm 11
- Windows desktop development: WebView2 and the Tauri prerequisites

## Run

```powershell
pnpm install

# Terminal 1: local API
pnpm dev:api

# Terminal 2: browser desk
pnpm dev

# Marketing site
pnpm www

# Desktop shell (embeds the API, pairs itself)
pnpm dev:desktop
```

The API listens on `127.0.0.1:4317`; the desk on `localhost:1420`; the site on
`localhost:4320`. When no client is paired yet the API prints a six-digit
pairing code. Type it once on the desk's pair screen; the browser keeps a
token. More codes come from the desk's **Guests** panel.

Desktop data is stored in the operating system's application-data directory
as `asten.db`. API development data defaults to `data/asten.db`.

### Environment

| Variable                | Default            | Meaning                                                        |
| ----------------------- | ------------------ | -------------------------------------------------------------- |
| `ASTEN_BIND`            | `127.0.0.1:4317`   | Listen address                                                 |
| `ASTEN_DATABASE_PATH`   | `data/asten.db`    | SQLite file                                                    |
| `ASTEN_AUTH`            | `pairing`          | `pairing`, `owner`, or `off` (refused beyond loopback)         |
| `ASTEN_ALLOWED_ORIGINS` | built-in list      | Extra browser origins, comma-separated                         |
| `ASTEN_SERVE_DESK`      | unset              | Serve a built desk (`apps/web/dist`) from the API              |
| `ASTEN_COOKIE_SECURE`   | `false`            | Mark the session cookie `Secure` behind TLS                    |
| `ASTEN_TRUSTED_PROXIES` | unset              | Proxies whose `X-Forwarded-For` is trusted, addresses or CIDRs |

See `docs/SECURITY.md` for the threat model.

## Workspace

```text
apps/
  web/                    The desk: shared React frontend
  desktop/src-tauri/      Tauri shell; embeds the API
  www/                    Public site, legal record, crawl files
  addin-word/             Word task pane (guest; pairs with a code)
  addin-docs/             Docs sidebar (guest; pairs with a code)
crates/
  asten-ir/               Story IR: anchors, entities, facts, events, contexts
  asten-time/             Clocks, calendars, time parsing, temporal network
  asten-blocks/           Block model: patches, hash chain, merge, anchors, blame
  asten-diagnostics/      Codes, severities, diagnosis, suppressions, baselines
  asten-logic/            The analyses
  asten-perceive/         Language-agnostic perception, no models
  asten-ls/               Story language server protocol
  asten-engine/           Incremental engine and queries
  asten-auth/             Tokens, pairing codes, passphrases, rate limiting
  asten-domain/           Product types
  asten-application/      Use cases and the repository boundary
  asten-storage/          SQLite adapter and migrations
packages/
  contracts/              TypeScript API contracts, generated from Rust and drift-checked
  brand/                  Ink and paper tokens
services/
  asten-api/              Axum HTTP transport and SSE
  asten-prealpha/         Hosts one API per visitor for the hosted pre-alpha (D29)
deploy/
  pre-alpha/              Compose, nginx and README for the hosted pre-alpha
  release/                Local Linux release build (Docker)
docs/
  ENGINE_DESIGN.md        The engine design
  architecture.md         Crate layering and transport
  DECISIONS.md            Why things are the way they are
  ROADMAP.md              Shipped, in review, next, later, never
  PLATFORMS.md            Support matrix, build recipes, signing
  PARITY.md               What each client can do
  SECURITY.md             Threat model and auth modes
  API.md                  Calling the HTTP API: pairing, tokens, the stream ticket, every route
  BRAND.md                Brand guidelines
  HANDOFF.md              Current stage
```

## Quality checks

```powershell
pnpm check          # tsc + cargo check
pnpm lint           # oxlint + clippy -D warnings
pnpm test           # vitest + cargo test
pnpm format:check   # prettier + rustfmt
pnpm contracts:gen  # regenerate packages/contracts/src/generated from the Rust types
pnpm e2e            # Playwright: builds the current desk and asten-api, starts the API on 127.0.0.1:4391 with a throwaway database, pairs, runs apps/web/e2e
                    # first time only: pnpm --filter @asten/web exec playwright install chromium
```

The TypeScript contracts are generated from the Rust types with `ts-rs`
(`packages/contracts/src/generated`, committed). The handwritten contracts in
`packages/contracts/src/index.ts` are checked against them by `src/drift.ts`,
so a field that changes on one side fails the type-check until both agree.

## Licence

Asten is released under the MIT License; see [`LICENSE`](LICENSE). The
names Asten and Verne and their logos are not covered by it.

The engine design draws on the MIT-licensed `dynodoc-engine` reference. No
code from the AGPL `dynodoc` application is used.

0.0.1c: The optional assistant supports installed subscription CLIs alongside API keys. Canon checks remain rule based, proposed facts require confirmation, and the assistant never writes prose. The native assistant completed a real bounded Claude subscription request. See [installed agent connections](docs/INSTALLED-AGENTS.md) and [acceptance](docs/ACCEPTANCE-0.0.1c.md).
