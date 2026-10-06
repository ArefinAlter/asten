# Asten — handoff

6 October 2026. Asten is a Verne Ltd. product.

Local-first writing desk. The manuscript stays in SQLite on the machine. Asten may notice a contradiction. It does not write the prose. No account. No sync.

## Repos

- Product: https://github.com/ArefinAlter/asten
- Research ingest: https://github.com/ArefinAlter/asten-research-ingest

Local folders are still `B:\margin` and `B:\margin-research-ingest`. Windows has them open, so they were not renamed. GitHub already redirects the old `margin` names.

`docs/PROJECT_HANDOFF.md` is the September planning archive. This file is the current stage.

## Run

From `B:\margin`, after `pnpm install`:

- `pnpm dev:api` — API on `127.0.0.1:4317`
- `pnpm dev` — desk on `localhost:1420`
- `pnpm www` — public site on `localhost:4320`
- `pnpm dev:desktop` — Tauri shell

Desktop data is `asten.db` in the OS app-data directory. API dev data is `data/asten.db`. Env vars are `ASTEN_BIND` and `ASTEN_DATABASE_PATH`.

Research ingest is a separate Python tool (`asten-ingest`) in the other repo. It is not the writing app. Reindex after the rename: the Qdrant collection is `asten_chunks`.

## Shipped

- Binder, Tiptap editor, comments, find next/previous, letter page view, track-change marks, snapshots.
- Cast, timeline, optional world (wall, atlas, dossier). A scene can start with no world.
- Canon facts. Diagnostics: CANON-E013 attribute clash, CANON-E101 life after a settled death, CANON-E201 reading order against story time. Suppressions. Optional diction packs.
- Compile and import: markdown, plain, fountain, HTML, docx. PDF is print-from-HTML.
- Local API and a Tauri desktop shell. Run from source. No signed installer.
- Word task pane and Docs sidebar. Sideload only. They call `127.0.0.1:4317`.
- Public site: product pages, legal record, sitemap, robots, `llms.txt`, FAQ schema. Footer credit: Designed and built by Verne Ltd. Live origin in code is `https://asten.vercel.app`.

## Still to do

- Close Cursor, stop Qdrant, and rename the two local folders.
- Signed desktop installer. The download page says to run from source.
- Publish the add-ins only if they go to the Office or Workspace stores. Today they are sideload guests. The Word pane must be served at `https://localhost:3000`.
- Reindex the research collection `asten_chunks`. Search is empty until that runs.
- Set `site.origin` in `apps/www/src/seo/catalog.ts` when a real domain replaces `asten.vercel.app`.
- Legal pages name Verne and point at the GitHub repo. They do not invent a postal address, company number, or governing law. Add those only when Verne Ltd. supplies them.
- Align older “a Verne product” lines with “Verne Ltd.” where the legal identity should say the company name.

## Left out on purpose

- Adaptable writing-style workspace template. Not started. Still out of scope until asked.
- InDesign, a pagination engine, and a PDF crate. Page view is a CSS letter sheet.
- Full Word-style review. Track changes are insertion and deletion marks.
- Find paints one match at a time. It does not highlight every hit.
- A cloud model. If one is added, the privacy page and the UI say so first.
- CSS `margin` properties were not renamed. The product name does not touch them.

## CI

On `fb373b8`, desktop passed. Rust failed clippy in `asten-diagnostics` and `asten-application`. Frontend failed Prettier in `@asten/contracts` and `@asten/web`. The fix for those checks is in the tree with this note.
