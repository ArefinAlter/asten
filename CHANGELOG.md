# Changelog

All notable changes to Asten are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Version numbers
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html); while the
major version is 0, a minor version may change behaviour.

## [Unreleased]

## [0.0.1-beta] - 2026-10-27

Tagged `v0.0.1b`, the second public pre-release (pre-alpha). The desktop app is
now complete standalone software for writers, with an optional assistant that
uses a model of the writer's choosing.

### Added

- The assistant (D30): `crates/asten-assistant`, wired into the desktop shell
  by `ai_*` commands. Providers: Anthropic Messages; the OpenAI Chat
  Completions family (OpenAI, Mistral, DeepSeek, Qwen, Kimi, GLM, Ollama and
  any OpenAI-compatible endpoint); Google Gemini. Streaming, tool calling
  (with a JSON fallback for models without it), retries with backoff,
  cancellation, size limits, no redirects. Keys live in the operating
  system's credential store or an encrypted file (Argon2id, ChaCha20-Poly1305)
  and never reach the page, logs, errors, exports or the ledger. Consent per
  provider, tied to the host and a notice version; offline mode; spend
  ledger and caps (zero means no call); the exact request can be read first
  (the builder is the sender's own, with the key as a placeholder); manuscript
  wording is excluded unless the writer attaches a passage. The tool table is
  closed (ten tools that read or prepare, five closed by rule); a request for
  prose is refused before any provider is contacted, the system prompt and the
  tool gate say it again, and a reply that reads like new narrative is
  withheld.
- Importers that read structure (D32): Markdown, plain text, HTML and Word
  into a block model; Word tracked changes become marks and comments keep
  their ranges; long files with chapter headings arrive as a manuscript with
  a document per chapter (`split`: `auto`, `chapters`, `none`). Compile writes
  the same formatting back: Word with real styles, lists and revisions, HTML
  with its emphasis.
- A project file (`.asten`, D31): a project as one JSON file to move to
  another computer, opened as a new project with fresh ids; project rename and
  delete.
- Library backups and restore (D31): on demand and at launch (the newest seven
  kept), a copy saved anywhere, and a validated restore that swaps at the next
  launch and keeps the library it replaced. Native save dialogs for files the
  writer names (`save_file`).
- Asten's own mark in every application icon (the Palette on the day page
  colour, drawn by `apps/www/scripts/app-icons.mjs`): desktop, installer and
  uninstaller (with a header and a side picture for the wizard), web app,
  favicon, Word add-in and Android. They had been the brown "M" placeholder.
- Acceptance scripts under `tools/acceptance/` that drive the real desktop
  window with a scratch profile (see `docs/ACCEPTANCE-0.0.1b.md`).

### Changed

- D6 is amended by D30: keys, provider calls, the library and file saving are
  Tauri commands; the story stays on the HTTP API.
- One underline extension in the editor (the editor registered it twice).

### Known limits

- The assistant's providers are verified against mock servers that speak each
  vendor's wire format, not against live services; a live call needs your key.
- Eye-colour clashes between brown and green are not raised: the colour
  lattice treats hazel as a neighbour of both (D15 still means no pronoun
  resolution).
- The hosted pre-alpha stays on 0.0.1a and has no assistant.

## [0.0.1-alpha] - 2026-10-25

Tagged `v0.0.1a`, the first public pre-release (pre-alpha).

### Added

- The hosted pre-alpha (`services/asten-prealpha`, D29): one real Asten API
  per anonymous visitor, each on its own SQLite file, deleted 24 hours after
  the first visit, with quotas; a `/svc/v1` store for the workbench state the
  API does not keep (preferences, assistant threads, console history, layout;
  never provider keys); a strict origin rule for the visitor cookie. The
  desk's `--mode hosted` build talks to it and creates the sample novel
  through the API on the first visit. Deployment files in `deploy/pre-alpha/`.
- Desktop release 0.0.1a for Windows x64 (NSIS installer, portable zip) and
  Linux x64 (AppImage, .deb), unsigned, plus the `asten-api` runtime for both.
  Built on the release machine (`deploy/release/linux.Dockerfile`,
  `scripts/release-assets.mjs`); `release-prealpha.yml` runs the same on
  demand.

### Changed

- The updater reads `latest.json` from the public site
  (`/updates/latest.json`), which a pre-release can update; GitHub's
  "latest release" link skips pre-releases.
- Windows bundles are NSIS only (MSI cannot carry a pre-release version).
- `release.yml` builds on full version tags only (`v1.2.3`).
- The desk's own state (preferences, threads, console, layout) goes through
  one store: the webview's storage on desktop and local builds, the hosting
  service on the hosted pre-alpha.
- The publisher in the installer metadata is Verne Ltd. (D28).
- The public site's call to action is "Try the pre-alpha"; `/app` redirects to
  the hosted pre-alpha.

### Added

- Desktop installers for Windows (x64 and ARM64), macOS (one universal build
  for Apple silicon and Intel) and Linux (x86_64, with an aarch64 job waiting
  on an ARM runner), built on a version tag and attached to a draft release.
- A one-column desk for phones and tablets: the activity bar along the bottom,
  the binder as a drawer, the inspector as a sheet, larger touch targets.
- A `/desk` endpoint that tells a client about the API it reached (version,
  host, whether it is embedded, how it authenticates, and for desk and owner
  sessions where the database lives), and a web-app manifest so the browser
  desk can be installed on a phone.
- The client parity matrix (`docs/PARITY.md`), the decisions record
  (`docs/DECISIONS.md`), the engineering roadmap, and the platform story on
  the public site.
- In the desktop shell: a copy of `asten.db` at every launch (SQLite
  `VACUUM INTO`; the newest seven kept; at most one copy every six hours)
  under app-data `backups/`; one log file a day under app-data `logs/` (the
  newest seven kept), with no token, ticket, pairing code or passphrase ever
  written; and a guard so a second launch raises the running window instead
  of starting a second copy.
- Settings → This desk shows the database, log and backup paths with copy
  buttons and, on desktop, a "Check for updates" button. The check runs only
  when pressed, reports "You have the latest" or the version available, and
  installs and relaunches after a second press. Release builds carry signed
  updater artifacts and a `latest.json`.
- The generated Android project (`apps/desktop/src-tauri/gen/android`), so
  the phone shell builds on any machine with the SDK and NDK.
- This changelog.

### Changed

- The shell library is named `asten_desktop_lib`, so it no longer collides
  with the binary on the platforms that build both.
- CI checks the Tauri shell on macOS weekly and on demand rather than on
  every push.

## [0.1.0] - 2026-10-12

The first early-access build. Asten is a local-first writing desk with a
deterministic canon engine: it keeps the manuscript on the writer's machine,
weighs each page against the facts the writer has settled, and never writes
prose.

### Added

- The engine crates: the story IR, clocks and calendars, the block model with
  its event history, diagnostics, the logic layer, and deterministic
  perception.
- One authenticated loopback API (`asten-api`) in front of the engine, with
  pairing codes and an owner passphrase, embedded in the Tauri desktop shell
  and runnable on its own for the browser desk.
- The web desk on the Asten brand: the manuscript editor with inline marks,
  the Scales, the binder, the cast, world and timeline views, compile to
  Markdown and `.docx`, and import.
- Guests for Microsoft Word (a task pane) and Google Docs (a sidebar) that
  pair with a desk by typing a code.
- TypeScript contracts generated from the Rust types, with a drift check.
- Windows installers bundled on version tags.
- The public site in the Egyptian brand with prerendered pages, the brand
  guidelines, and the design documents: engine design, security model,
  handoff, and deployment configuration.

[Unreleased]: https://github.com/ArefinAlter/asten/compare/v0.0.1a...HEAD
[0.0.1-alpha]: https://github.com/ArefinAlter/asten/releases/tag/v0.0.1a
[0.1.0]: https://github.com/ArefinAlter/asten/releases/tag/v0.1.0
