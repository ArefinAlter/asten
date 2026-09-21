# Margin

Margin is a local-first writing IDE. The current repository is an architectural
scaffold: project and document persistence works, while research-dependent
editorial rules, knowledge schemas, and writing methods remain extension points.

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

# Or run the same UI in the desktop shell
pnpm dev:desktop
```

The API listens on `127.0.0.1:4317`; Vite listens on `localhost:1420`.
Desktop data is stored in the operating system's application-data directory.
API development data defaults to `data/margin.db`.

## Workspace

```text
apps/
  web/                    Shared React frontend
  desktop/src-tauri/      Tauri transport and native shell
crates/
  margin-domain/          Transport-independent data types
  margin-application/     Use cases and repository boundary
  margin-storage/         SQLite adapter and migrations
  margin-diagnostics/     Rule-engine extension boundary
packages/
  contracts/              Shared TypeScript API contracts
services/
  margin-api/             Axum HTTP transport and middleware
docs/
  architecture.md         Dependency rules and design decisions
  PROJECT_HANDOFF.md      Complete product and engineering handoff
```

## Quality checks

```powershell
pnpm check
pnpm lint
pnpm test
pnpm format:check
```

The `books/` research directory, databases, generated output, editor settings,
and secrets are excluded from Git.
