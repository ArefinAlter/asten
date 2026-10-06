# Asten

Asten is a local-first writing environment. The name is another name for Thoth,
the Egyptian god of writing. The thesis is unchanged: let AI be the editor, not
the writer.

The desk is a manuscript editor with a binder, comments, find, page view, and
track changes. The compiler notices contradictions in world, character, and
time. Word and Docs remain guests.

## Prerequisites

- Rust 1.85 or newer
- Node.js 22 or newer
- pnpm 11
- Windows desktop development: WebView2 and the Tauri prerequisites

## Run

```powershell
pnpm install

# Terminal 1: web API
pnpm dev:api

# Terminal 2: browser UI
pnpm dev

# Marketing site
pnpm www

# Desktop shell
pnpm dev:desktop
```

The API listens on `127.0.0.1:4317`; the desk listens on `localhost:1420`;
the foyer listens on `localhost:4320`. Desktop data is stored in the operating
system's application-data directory as `asten.db`. API development data defaults
to `data/asten.db`.

## Workspace

```text
apps/
  web/                    Shared React frontend
  desktop/src-tauri/      Tauri transport and native shell
  www/                    Marketing foyer
  addin-word/             Word task pane (local API)
  addin-docs/             Docs add-on notes
crates/
  asten-domain/          Transport-independent data types
  asten-application/     Use cases and repository boundary
  asten-storage/         SQLite adapter and migrations
  asten-diagnostics/     Rule engine
packages/
  contracts/              Shared TypeScript API contracts
  brand/                  Ink and paper tokens
services/
  asten-api/             Axum HTTP transport
```

## Quality checks

```powershell
pnpm check
pnpm lint
pnpm test
pnpm format:check
```
