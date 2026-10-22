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
- Public site in the brand, honest about the above.

## In review (stacked pull requests)

1. **Cross-platform builds** (PR 1): Windows x64 and ARM64, macOS universal,
   Linux x86_64 and aarch64; CI on three operating systems; mobile-ready crate.
2. **Responsive desk** (PR 2): one column on phones and tablets, drawer binder,
   sheet inspector, bottom activity bar, touch targets; Android 8 and iOS 15
   floors.
3. **Client parity** (PR 3): `GET /desk`, "This desk" in Settings, installable
   web app for phones, the parity matrix.
4. **Decisions and roadmap** (PR 4): this page, `docs/DECISIONS.md`, site
   roadmap and FAQ updates.

## Next

- **Signed installers.** Add the Windows and Apple signing secrets; notarise
  the macOS build; publish the first tagged release. Download page then stops
  saying "run from source".
- **Mobile shells built and tested.** Run `tauri android init` and
  `tauri ios init` on machines with the SDKs, commit the generated projects,
  test the one-column desk on a phone and a tablet, and add the share-sheet
  path for compile output.
- **Device testing for ARM.** Windows on ARM and Linux aarch64 have configured
  release jobs; first verify their artifacts, then run them on hardware and record the result in `docs/PLATFORMS.md`.
- **Linux aarch64 runner.** Enable an ARM runner for the private repository so
  the release job stops being optional.
- **OpenAPI document.** Describe the routes formally now that the types are
  generated; publish it for tinkerers.
- **Guest panes live.** `EventSource` with a stream ticket in the Word pane and
  the Docs sidebar, replacing the refresh button.
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
