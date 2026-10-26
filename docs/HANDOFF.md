# Asten — handoff

Updated 26 October 2026 (release 0.0.1b). Asten is a Verne Ltd. product.

Local-first writing desk with a canon engine. The manuscript stays in SQLite on
the machine. Asten may notice a contradiction and explain it. It does not write
the prose. No account. No sync. Generative models only under the AI
policy, optional and off by default (D27).

## Repos

- Product: https://github.com/ArefinAlter/asten
- Research ingest: https://github.com/ArefinAlter/asten-research-ingest

Local folders are still `B:\margin` and `B:\margin-research-ingest`. GitHub
redirects the old `margin` names.

`docs/PROJECT_HANDOFF.md` is the September planning archive.
`docs/ENGINE_DESIGN.md` is the engine design the October work implements.
`docs/BRAND.md` is the brand; §12 covers the desk. `docs/SECURITY.md` is the
threat model and the auth modes. This file is the current stage.

## Run

From `B:\margin`, after `pnpm install`:

- `pnpm dev:api` — API on `127.0.0.1:4317`. With no paired client it prints a
  six-digit pairing code to stderr.
- `pnpm dev` — desk on `localhost:1420`. Type the code once; the browser keeps
  a token.
- `pnpm dev:desktop` — Tauri shell. It embeds the API and pairs itself.
- `pnpm www` — public site on `localhost:4320`.

Env vars: `ASTEN_BIND`, `ASTEN_DATABASE_PATH`, `ASTEN_AUTH`
(`pairing` default, `owner`, `off` on loopback only), `ASTEN_ALLOWED_ORIGINS`,
`ASTEN_SERVE_DESK`, `ASTEN_COOKIE_SECURE`. See `.env.example`.

Desktop data is `asten.db` in the OS app-data directory. API dev data is
`data/asten.db`.

Research ingest is a separate Python tool in the other repo. It is not the
writing app.

## Shipped

### Engine (Rust, `crates/`)

- `asten-ir`, `asten-time`, `asten-blocks`, `asten-diagnostics`, `asten-logic`,
  `asten-perceive`, `asten-ls`, `asten-engine`, `asten-auth`. See
  `docs/architecture.md` for what each owns.
- Dynodoc block model: stable block ids, rich-text patches, hash-chained event
  log per document, snapshots with Merkle roots, anchors that follow edits,
  blame, three-way merge for drafts, verify.
- Story IR with gradual canon (settled, proposed, deprecated, rejected) and
  epistemic contexts (narration, dialogue, belief, memory, dream, vision, frame,
  hypothetical). Lies and dreams do not count against the world.
- Analyses and codes: `CANON-E013/E014/E015` attribute and quantity clashes,
  `E101/E102/E103` lifecycle, `E201/E202/E203` two clocks and temporal
  network, `E301` routes, `E401` knowledge, `E501/E502/E503` relations,
  `E601` author invariants; `NAME-W001–W004` name hygiene; `PLOT-I001` the
  setup ledger. `E302`, `E602` and `PLOT-I002` are in the catalogue but no rule
  raises them yet. Severity by pairing (settled vs settled is an error,
  prose vs settled a warning, a merely possible clash information; proposed vs
  proposed shows only at the Thorough setting), confidence
  thresholds, single-fault priors, explanation chains (①②③) and ranked
  repairs the author owns. Suppressions carry an evidence fingerprint and go
  stale when the evidence changes. Baselines for "new since".
- Perception without models: Unicode folding, Aho-Corasick mentions with
  suffix tolerance, cue-based attribute observations (hair, eyes, lifecycle,
  material, location, age, height, relations), negation, dialogue attribution,
  inline `[[entity.predicate=value]]`, name candidates across scripts.
  Language tiers reported per project.
- Performance: a 250k-word project re-perceives one block per edit, under two
  seconds end to end in debug.

### Application and API

- One `Application` behind one HTTP transport. Per-project engine, broadcast
  stream, SSE with diagnostics deltas and heartbeats.
- TypeScript contracts generated from the Rust types with `ts-rs` and checked
  for drift against the handwritten desk contracts on every build.
- Story language server routes: diagnostics, fact card, mentions, where
  settled, rename preview/apply, scene summary, cast search, proposals, apply
  repair, bisect, catalog, invariants, routes, baselines, settings.
- History: event log, verify, document at a sequence, blame, drafts and merge
  preview with semantic conflict marks.
- Auth: pairing codes that grant a token kind (desk or guest), bearer tokens
  hashed at rest, single-use stream tickets for SSE, optional owner
  passphrase (argon2id, set once by the store), per-client and per-route rate
  limiting with trusted-proxy support, CORS allowlist with strict wildcards,
  same-origin-only session cookies, security headers. Guests cannot manage
  pairing. Default mode is pairing. The API refuses `off` beyond loopback.
- Reviewed 12 October: three independent review passes (API security, engine
  correctness, desk and add-ins). Fixed: wildcard origin matching, query
  tokens in logs, owner-setup race, pairing atomicity, limiter keying and
  growth, Myers diff memory (bounded edit distance), fictional-calendar year
  loops and zero-day months, duplicate block ids, fractional-key growth,
  fuzzy reattach cost, number overflow, editor autosave caret jump and
  lost-save states, Settings keystroke saves, `.docx` import size limit,
  insecure-context UUIDs, 401 handling.
- Compile: markdown, plain, fountain, HTML, docx, EPUB 3, Final Draft (.fdx), all in binder order; import of the same except EPUB, with one screenplay classifier behind Fountain and FDX.

### Desktop

- Tauri shell embeds the API on `127.0.0.1:4317` (ephemeral fallback), revokes
  stale desk tokens, issues a fresh one, and exposes `desk_session` and
  `start_pairing`. The webview uses the same HTTP client as the browser.

### Desk (React, `apps/web`)

- Brand §12: ink and papyrus tokens, Fraunces/Literata/Plex Mono, day and
  night, cartouches, glyph rail.
- Manuscript editor on Tiptap with stable block ids, inline underline marks
  and margin cards, entity tokens with fact-card popovers, scene header with
  story-time placement, track-change marks, page view, find, blame.
- The Scales panel: filters, explanation chain, repairs, suppress with reason,
  bisect, balanced empty state.
- Binder, Overview, Cast (facts settle/reject/retire, aliases, mentions,
  rename preview, merge), Canon (facts, proposals, invariants, routes,
  baselines), Timeline (two registers), History (log, verify, peek, drafts,
  merge preview), Guests (pairing codes, tokens, owner passphrase), Settings
  (strictness, narrator, clock and calendar, parent age, language tiers),
  Compile, Packs. Pair screen for browsers.
- Hash routing `#/p/{project}/{view}/{doc}`.

### Guests and site

- Word task pane and Docs sidebar pair with a code, keep a bearer token, show
  the Scales, file the selection, and can forget the pairing. Sideload only.
- Public site (`apps/www`): features, FAQ, glossary, audience pages, roadmap
  and reference refreshed on 12 October to match what shipped.

## Merged on 21 to 25 October (stacked pull requests)

1. Cross-platform builds: Windows x64 and ARM64, macOS universal, Linux
   x86_64 and aarch64; CI on three operating systems; mobile-ready crate;
   `docs/PLATFORMS.md`.
2. Responsive desk: one column on phones and tablets, drawer binder, sheet
   inspector, bottom activity bar; Android 8 and iOS 15 floors.
3. Client parity: `GET /desk`, "This desk" in Settings, installable web app
   for phones, `docs/PARITY.md`.
4. `docs/DECISIONS.md` and `docs/ROADMAP.md`, with the site roadmap and FAQ
   brought into line.

`docs/ROADMAP.md` is the engineering roadmap; the list below is the short
version.

## Still to do

- Signed installers for Windows, macOS and Linux. The release workflow targets
  the configured desktop platforms on a `v*` tag; installer verification is
  still pending; signing turns on when the Windows and Apple
  secrets exist (see `docs/PLATFORMS.md`). Release 0.0.1b (Windows x64 and Linux
  x64) is published unsigned; the download page links it. The hosted
  pre-alpha still runs 0.0.1a (rebuilding its image needs the VPS password).
- Live-provider checks of the assistant: it is tested against mock servers
  that speak each wire format; run it once with a real key per vendor before
  calling a provider supported.
- Mobile shells generated, built and tested on devices (`tauri android init`,
  `tauri ios init` on machines with the SDKs); hardware testing of the ARM
  builds; an ARM runner for the Linux aarch64 release job.
- Tier A perception models (M1–M5 in the design) and the training pipeline.
  Everything today is Tier B/C.
- Docker: the compose file runs the API in pairing mode behind nginx with the
  stream unbuffered and unlogged. The images have not been built; the Docker
  daemon was not running in this session.
- Publish the add-ins only if they go to the Office or Workspace stores. The
  Word pane must be served at `https://localhost:3000`.
- Reindex the research collection `asten_chunks`.
- Set `site.origin` in `apps/www/src/site.ts` when a real domain replaces
  `asten-writing.vercel.app`.
- Legal identity: Verne Ltd. everywhere (D28). Counsel review before launch.
- Rotate the Vercel token that an earlier session printed, if not done.

## Where things are written down

- `docs/DECISIONS.md`: the decisions and why, as a record that is appended to.
- `docs/ROADMAP.md`: shipped, next, later, never.
- `docs/PLATFORMS.md`: support matrix, build recipes, signing.
- `docs/PARITY.md`: what each client can do and why.
- `docs/PHONE_SETUP.md`: trusted HTTPS, pairing and phone acceptance checks.
- `docs/SECURITY.md`: threat model and authentication.
- `docs/architecture.md`: crates and transport. `docs/ENGINE_DESIGN.md`: the
  engine design.

## Left out on purpose

- A generative model in the judging logic. The engine is deterministic and
  explainable; optional generative features follow the AI policy (D27).
- Adaptable writing-style workspace template. Not started.
- InDesign, a pagination engine, and a PDF crate. Page view is a CSS letter
  sheet.
- Full Word-style review. Track changes are insertion and deletion marks.
- Cloud sync, accounts, telemetry.
- Pronoun resolution. Perception attributes an observation to a name in the
  same sentence; "Her blue eyes" in a sentence without a name is not weighed.

## CI

Three jobs. Frontend formats, type-checks, lints, tests and builds the desk on
Linux. Rust runs clippy and tests for the whole workspace except the Tauri
shell. Desktop builds the desk and checks the shell on Windows.
