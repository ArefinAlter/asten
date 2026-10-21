# Changelog

All notable changes to Asten are recorded here. The format follows
[Keep a Changelog](https://keepachangelog.com/en/1.1.0/). Version numbers
follow [Semantic Versioning](https://semver.org/spec/v2.0.0.html); while the
major version is 0, a minor version may change behaviour.

## [Unreleased]

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

[Unreleased]: https://github.com/ArefinAlter/asten/compare/v0.1.0...HEAD
[0.1.0]: https://github.com/ArefinAlter/asten/releases/tag/v0.1.0
