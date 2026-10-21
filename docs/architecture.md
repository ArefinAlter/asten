# Architecture

## Constraints

1. The desktop application must work completely offline.
2. Desktop and web use one frontend and one application layer.
3. Domain, engine and application code cannot depend on Tauri, Axum, SQLite,
   React, or any future model runtime.
4. User documents remain usable without the diagnostic engine.
5. Research-dependent concepts enter through schemas, rules, and method packs;
   they are not hard-coded into infrastructure.
6. No generative language model is in the loop. Perception is deterministic
   today; trained models, when they come, are local classifiers and taggers
   (see `docs/ENGINE_DESIGN.md` §11).

## Dependency direction

```text
React desk (browser tab or Tauri webview)
    │  HTTP /api/v1  +  Server-Sent Events /stream
    ▼
asten-api (axum)  ◄── embedded on 127.0.0.1 by the Tauri shell
    │
    ▼
asten-application ─────────────────────────┐
    │                │                     │
    ▼                ▼                     ▼
asten-domain   Repository trait       asten-engine
                     ▲                 │        │
                     │                 ▼        ▼
               asten-storage     asten-perceive  asten-logic
                  SQLite               │        │
                                       └────┬───┘
                                            ▼
                      asten-ir · asten-time · asten-blocks · asten-diagnostics · asten-ls
```

Crates, bottom up:

| Crate               | Owns                                                                                                      |
| ------------------- | --------------------------------------------------------------------------------------------------------- |
| `asten-ir`          | The Story IR: anchors, entities, predicates with value lattices, facts, observations, events, scenes, contexts, provenance, settings |
| `asten-time`        | Clocks (day count, Gregorian, fictional calendars), time-expression parsing, and a simple temporal network solver |
| `asten-blocks`      | The Dynodoc block model: canonical JSON fingerprints, fractional positions, rich-text patches, the hash-chained event log, materialisation, three-way merge, anchor remapping, blame |
| `asten-diagnostics` | Diagnostic records, severities and tiers, the code catalogue, single-fault diagnosis, suppressions and baselines |
| `asten-logic`       | The analyses: event calculus with must/may clipping, lifecycle typestate, two clocks, name hygiene, relations, knowledge, routes, author invariants |
| `asten-perceive`    | Language-agnostic perception: folding, Aho-Corasick mention matching, cue-based attribute observations, inline markup, name candidates |
| `asten-ls`          | The story-language-server protocol types: snapshots and deltas, fact cards, mentions, rename previews, repairs, stream events |
| `asten-engine`      | Ties perception and logic together with a fingerprint-keyed cache; one edit re-perceives one block |
| `asten-auth`        | Tokens, pairing codes, argon2id passphrases, a rate limiter |
| `asten-domain`      | Transport-independent product types (projects, documents, facts, settings, tokens) and their mapping to the IR |
| `asten-application` | Use cases, the `Repository` trait, per-project engines and broadcast streams, repairs, bisect, drafts, auth flows |
| `asten-storage`     | SQLite adapter and migrations                                                                             |
| `asten-api`         | Axum transport, middleware, SSE                                                                           |
| `asten-desktop`     | Tauri shell that embeds the API and hands the webview a desk token                                        |

## Data flow for one edit

1. The editor commits a document with its ProseMirror JSON and the head
   sequence it last saw.
2. The application diffs the new content against the materialised block state,
   appends block events to the hash chain (preferring rich-text patches so
   anchors can follow), stores a snapshot every 500 events, and remaps every
   anchor in the document.
3. A refresh is scheduled. The engine re-perceives only blocks whose
   fingerprint changed, rebuilds the IR, runs the analyses, and diffs the
   diagnostics against the previous snapshot.
4. The delta is broadcast on the project's SSE stream. The desk updates the
   inline marks and the Scales panel without a refetch.

## Data ownership

SQLite is the transactional store. Document bodies are stored as ProseMirror
JSON with a stable block id on every top-level node. Beside the body, each
document keeps its block event log (sequence, actor, time, content hash, chain
hash), periodic snapshots with a Merkle root, and its scene placement.

Plain-file project bundles and CRDT synchronisation belong behind additional
repository implementations. They must not leak filesystem or CRDT types into
the domain crate.

## Clients

Three doors share the one transport: the Tauri shell for Windows, macOS, Linux
and (from the same crate) Android and iOS; the browser desk, which is the same
React app with a responsive shell and can be installed on a phone; and the
Word and Docs guests. `docs/PLATFORMS.md` is the support matrix and
`docs/PARITY.md` the capability matrix. Decisions D6, D12, D13 and D14 in
`docs/DECISIONS.md` explain the shape.

## Transport parity

There is one transport. The browser calls `/api/v1` on the API it was served
from or paired with. The desktop shell starts the same API on a loopback port,
issues a fresh desk token, and exposes exactly two Tauri commands:
`desk_session` (origin, token, database path, version) and `start_pairing`.
The React client therefore has one code path for both, and a feature that works
in the browser works on the desktop by construction.

The TypeScript contracts in `packages/contracts` come from the Rust types:
every serde-derived type in the contract crates also derives `ts_rs::TS`
behind the `ts` feature, `pnpm contracts:gen` exports them to
`src/generated/<crate>/`, and `src/drift.ts` asserts that each handwritten
type the desk uses is mutually assignable with its generated twin. CI
regenerates and fails on a diff. An OpenAPI document is still to come.

## HTTP middleware

- authentication (bearer or session cookie; a one-minute single-use ticket
  for the SSE stream), three modes and two token kinds with different
  authority, see `docs/SECURITY.md`;
- rate limiting per client and per route, with `ASTEN_TRUSTED_PROXIES` for
  deployments behind a reverse proxy;
- an explicit CORS origin allowlist with private-network approval only for
  allowed origins;
- security headers on every response and `Cache-Control: no-store`;
- request IDs and propagation, structured tracing, panic containment,
  compression, a 30-second timeout and a 10 MiB body limit;
- optional static serving of the built desk (`ASTEN_SERVE_DESK`);
- graceful shutdown.

## Decisions taken since the scaffold

- Editor schema: the Dynodoc block model, carried into Rust as `asten-blocks`.
- Canon ontology: the Story IR in `asten-ir`, gradual canon with four statuses
  and nine epistemic contexts.
- Diagnostic catalogue: `CANON-E0xx..E6xx`, `NAME-W00x`, `PLOT-I00x`, owned by
  method packs, with must/may tiers and single-fault priors.
- Incrementality: fingerprint-keyed perception cache in the engine rather than
  a salsa database. Analyses are whole-project and fast enough at 250k words.

## Still deferred

- collaboration CRDT and multi-author merge beyond drafts;
- trained local perception models (Tier A) and the training pipeline;
- a plugin runtime for third-party packs;
- map and drawing renderers;
- an OpenAPI document for the routes (the types are already generated);
- store listings for the mobile shells and the guests;
- product licensing and distribution terms.
