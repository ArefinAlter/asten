# Roadmap

The engineering roadmap. The public site's roadmap (`apps/www/src/content/roadmap.ts`)
is the reader-facing version of this page and must never promise more than it.
No dates. Items move down the page as they ship.

## Shipped

- Engine: Story IR, clocks and calendars, block model, seventeen canon rules in
  seven families, name hygiene, setup ledgers, repairs, suppressions with
  evidence fingerprints, baselines, bisect.
- Perception without models: names across scripts, plain attributes, dialogue,
  belief, memory, dream and frame contexts.
- Desk: editor with inline marks, the Scales, Binder, Cast, Canon, Timeline,
  World, History (verify, blame, drafts, merge), Guests, Settings, Compile.
- API with pairing-code authentication, token kinds, stream tickets, rate
  limits; desktop shell embedding it; Word and Docs guests that pair.
- TypeScript contracts generated from Rust and drift-checked.
- Compile to Markdown, plain text, Fountain, HTML, .docx, EPUB 3 and Final
  Draft (.fdx), in binder order; import of all but EPUB.
- OpenAPI 3.1 document generated from the handler annotations and the same
  Rust types, served at `/api/v1/openapi.json`, committed as
  `docs/openapi.json`, held to the router by a test; `docs/API.md` walks
  through pairing, calling and the stream ticket.
- Public site in the brand, honest about the above.

## In review (stacked pull requests, merge in order)

1. **Cross-platform builds** (#1): Windows x64 and ARM64, macOS universal,
   Linux x86_64 and aarch64; mobile-ready crate.
2. **Responsive desk** (#2): one column on phones and tablets.
3. **Client parity** (#3): `GET /desk`, "This desk", installable web app.
4. **Decisions and roadmap** (#4).
5. **Efficiency**: document summaries, SQLite pragmas and indexes, engine hot
   paths, lazy views (D20).
6. **Security**: served-desk CSP and caching, cookie origin rule, forwarded-hop
   keying, ticket revocation, dependency audit (D24).
7. **Guests live and end-to-end tests**: Word and Docs panes follow the stream;
   Playwright suite against the real API; the revision livelock fix (D18, D19).
8. **Algorithms**: property suites and the bugs they found (D23).
9. **Accessibility and onboarding**: dialogs, skip link, keyboard binder, live
   region, contrast audit, first-run steps (D26).
10. **Formats**: EPUB 3 export, Final Draft export and import, binder-order
    compile (D25).
11. **OpenAPI**: generated document held to the router, `docs/API.md` (D22).
12. **Desktop deployment**: signed user-initiated updates, single instance,
    file logs, launch backups, the Android project and an unsigned APK (D21).

## Next

- **Signed installers and updates.** Add the Windows and Apple signing
  secrets and the updater signing key (`TAURI_SIGNING_PRIVATE_KEY`); notarise
  the macOS build; publish the first tagged release. The download page then
  stops saying "run from source".
- **Mobile shells tested.** The Android project is committed and builds an
  unsigned APK; it needs a keystore, a device run, and the share-sheet path for
  compile output. The iOS project needs a Mac with Xcode for `tauri ios init`.
- **Device testing for ARM.** Windows on ARM and Linux aarch64 have configured
  release jobs; first verify their artifacts, then run them on hardware and record the result in `docs/PLATFORMS.md`.

- **Linux aarch64 runner.** Enable an ARM runner for the private repository so
  the release job stops being optional.
- **Docker images built and smoke-tested** on a machine with the daemon
  running; the compose file is ready.

## Later

- **Tier A perception models**: small local classifiers for mentions,
  attributes, time phrases and speaker attribution, trained through the logic,
  always proposing, never generating (D1). Pronoun resolution belongs here.
- **Store listings**: Microsoft AppSource and Google Workspace Marketplace for
  the guests; Google Play and the App Store for the shells, once signed and
  tested.
- **Collaboration that keeps offline work**: drafts and merge exist; a second
  author on a LAN or over a relay is a design question, not a feature yet.
- **Graph, maps with scale, boards**; a runtime for third-party packs.
- **Research reindex** of the `asten_chunks` collection in the ingest tool.

## Decisions pending the owner

- Legal identity: "Verne Ltd." in the footer versus the entity on
  vernesystems.com/legal/company. Counsel review before launch.
- A real domain in place of `asten-writing.vercel.app` (`apps/www/src/site.ts`).
- Rotate the Vercel token an earlier session printed, if not already done.

## Never

- Writing the author's prose: no generate, continue, rewrite or autocomplete.
- Training on manuscripts, by Verne or anyone Verne works with.
- Locking files behind a server or a subscription.
- Craft doctrine by default: style advice stays in optional packs.
