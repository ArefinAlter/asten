# Architecture

## Constraints

1. The desktop application must work completely offline.
2. Desktop and web use one frontend and one application layer.
3. Domain and application code cannot depend on Tauri, Axum, SQLite, React, or
   any future AI provider.
4. User documents remain usable without the diagnostic engine.
5. Research-dependent concepts enter through schemas, rules, and method packs;
   they are not hard-coded into infrastructure.

## Dependency direction

```text
React UI ── HTTP ──► Axum API ──────┐
    │                               │
    └── Tauri invoke ─► Desktop ────┤
                                    ▼
                         asten-application
                           │              │
                           ▼              ▼
                    asten-domain   Repository trait
                                          ▲
                                          │
                                  asten-storage
                                      SQLite
```

`asten-diagnostics` consumes domain snapshots and exposes a rule registry. It
will later use incremental query infrastructure, but the transport and storage
layers do not need to know which rules are installed.

## Data ownership

SQLite is currently the transactional application store. Document bodies are
stored as versioned JSON (`prosemirror-json-v1`) rather than HTML. The schema
already tracks stable IDs, hierarchy, order, revisions, and timestamps.

Plain-file project bundles and CRDT synchronization belong behind additional
repository implementations. They should not leak filesystem or CRDT types into
the domain crate.

## Transport parity

The browser uses `/api/v1`; desktop uses Tauri commands. Both transports call
the same `Application` methods and return the same serialized contracts. The
frontend selects a transport at runtime.

The TypeScript contracts are handwritten during the scaffold phase. Before the
API grows, replace this with generated contracts from Rust/OpenAPI and enforce
generation in CI.

## HTTP middleware

The API baseline includes:

- request IDs and propagation;
- structured request tracing;
- panic containment;
- response compression;
- a 30-second timeout;
- a 10 MiB request-body limit;
- configurable bind address and database path;
- graceful shutdown.

Development CORS is permissive. Deployment configuration must replace it with
an explicit origin allowlist before public hosting.

## Deferred decisions

The following require research or measured prototypes and are intentionally not
selected by this scaffold:

- manuscript editor schema and pagination;
- canon ontology and fact model details;
- diagnostic rule catalogue and severities;
- collaboration CRDT;
- graph/query engine;
- plugin runtime;
- map and drawing renderers;
- local versus hosted extraction models.
- product licensing and distribution terms.
