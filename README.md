# Asten

Asten is a local-first writing environment with a canon engine. The name is
another name for Thoth, the Egyptian god of writing. The thesis is unchanged:
let the machine be the editor, not the writer.

Asten is a Verne product. The desk is a manuscript editor with a binder,
comments, find, page view, and track changes. The engine reads the page, keeps a
gradual canon of what is settled and what is only proposed, and weighs the two
on the Scales: attribute clashes, life after death, two clocks, routes,
knowledge, relations, author invariants. Every mark explains itself and offers
repairs the author owns. Word and Docs remain guests. No account, no sync, no
generative model.

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
  contracts/              Shared TypeScript API contracts
  brand/                  Ink and paper tokens
services/
  asten-api/              Axum HTTP transport and SSE
docs/
  ENGINE_DESIGN.md        The engine design
  architecture.md         Crate layering and transport
  SECURITY.md             Threat model and auth modes
  BRAND.md                Brand guidelines
  HANDOFF.md              Current stage
```

## Quality checks

```powershell
pnpm check          # tsc + cargo check
pnpm lint           # oxlint + clippy -D warnings
pnpm test           # vitest + cargo test
pnpm format:check   # prettier + rustfmt
```

## Licence note

The engine design draws on the MIT-licensed `dynodoc-engine` reference. No
code from the AGPL `dynodoc` application is used.
