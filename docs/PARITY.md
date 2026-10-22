# Client parity

One desk, several doors. Every client talks to the same local API, so a feature
exists once and is reachable from every door that can show it. This page says
what each door can do, why a gap exists where it does, and how it would close.

## The doors

| Door                       | What it is                                                                            | Transport                              |
| -------------------------- | ------------------------------------------------------------------------------------- | -------------------------------------- |
| Desktop shell              | Tauri app on Windows, macOS, Linux; embeds the API and the desk                       | HTTP to the embedded API, desk token   |
| Browser desk (computer)    | The same React desk opened in a browser against `pnpm dev:api` or a hosted API        | HTTP, pairing code → token or cookie   |
| Browser desk (phone/tablet)| The same desk in the one-column layout; installation requires phone-trusted HTTPS     | HTTPS to the desk and API on the home machine |
| Android / iOS shell        | Tauri mobile app; embeds the API inside the app sandbox                               | HTTP to the embedded API, desk token   |
| Word task pane             | Office add-in, sideloaded                                                              | HTTP, pairs once as a guest            |
| Google Docs sidebar        | Apps Script sidebar                                                                   | HTTP, pairs once as a guest            |

For phone installation, use the same HTTPS origin for the static desk and API
and trust its certificate on the phone. See [Phone setup](PHONE_SETUP.md).
Plain HTTP over the LAN is browser-only and cannot register the service worker.
Native Android/iOS columns describe the intended shared-shell capabilities;
their SDK builds and device testing remain pending, not certified by this matrix.

## Capability matrix

✓ available · ◐ partial · ✗ not available · — not applicable

| Capability                                   | Desktop | Browser (computer) | Browser (phone) | Android / iOS | Word pane | Docs sidebar |
| -------------------------------------------- | :-----: | :----------------: | :-------------: | :-----------: | :-------: | :----------: |
| Write in the manuscript editor               | ✓       | ✓                  | ✓               | ✓             | ✗ (Word is the editor) | ✗ (Docs is the editor) |
| Inline marks and margin cards                | ✓       | ✓                  | ◐ inline only¹  | ◐ inline only¹ | ✗        | ✗            |
| The Scales: explanation chain, repairs, suppress | ✓   | ✓                  | ✓ (bottom sheet) | ✓ (bottom sheet) | ◐ list only² | ◐ list only² |
| Binder: create, rename, move, exclude, delete | ✓      | ✓                  | ✓ (drawer)      | ✓ (drawer)    | ✗         | ✗            |
| Cast, Canon, Timeline, World, History views  | ✓       | ✓                  | ✓               | ✓             | ✗         | ✗            |
| Settings: strictness, narrator, clock        | ✓       | ✓                  | ✓               | ✓             | ✗         | ✗            |
| File a passage into a project                | —       | —                  | —               | —             | ✓         | ✓            |
| Import markdown, plain, Fountain, HTML, docx | ✓       | ✓                  | ◐ file picker³  | ◐ file picker³ | —        | —            |
| Compile to markdown, plain, Fountain, HTML, docx | ✓   | ✓                  | ✓ (download)    | ◐ share sheet⁴ | —        | —            |
| Guests panel: mint codes, revoke tokens      | ✓       | ✓ (desk token)     | ✓ (desk token)  | ◐ no reachable guests⁵ | ✗ (guest) | ✗ (guest) |
| Owner passphrase                             | ✓       | ✓                  | ✓               | ◐ not needed⁵ | ✗         | ✗            |
| Where the database file lives                | ✓       | ✓ (`/desk`, desk token) | ✓          | ✓ (sandbox path) | ✗      | ✗            |
| Live change stream                           | ✓       | ✓                  | ✓               | ✓             | ✓ (ticket)⁶ | ✓ (ticket)⁶ |
| Dark and light theme                         | ✓       | ✓                  | ✓               | ✓             | ◐ follows Office | ◐ follows Docs |
| Keyboard shortcuts (Ctrl/⌘ F, J)             | ✓       | ✓                  | —               | —             | —         | —            |
| Works with no network at all                 | ✓       | ✓ (same machine)   | ✗ needs the LAN | ✓             | ✗         | ✗            |

¹ Margin cards need a second column; on a narrow screen the inline underline
and the Scales carry the same information.
² The guests list marks with their codes and messages. Repairs, suppression
with a reason and bisect stay in the desk, because they change the project and
a guest token may only read and write the story.
³ Phones have no drag-and-drop; the Compile view's file input still works.
⁴ Mobile WebViews cannot save a download to disk; the share sheet is the
platform's way out and is on the list below.
⁵ A phone's embedded API is not reachable from another device, so there is
nothing to pair with it. The owner passphrase exists for hosting the API beyond
the machine, which a phone does not do.
⁶ The panes open the same `EventSource` the desk does, each connection paid
for with a single-use stream ticket, and re-fetch the Scales on every revision.
If the stream drops they reconnect with backoff on a fresh ticket; the Refresh
button stays as the fallback.

## Why the gaps are where they are

- **Guests do not edit.** Word and Docs own the text while it is open there.
  The pane shows the Scales for a chosen project and files a selection as a
  new scene. Making the pane a second editor would create two sources of truth
  for the same words.
- **Guests do not manage pairing.** A guest token can read and write the story
  and revoke only itself (see `docs/SECURITY.md`). Minting codes and setting
  the passphrase belong to desk and owner sessions.
- **Mobile is the whole desk, not a companion.** The Android and iOS shells
  embed the API like the desktop does, so every view exists there. What
  differs is the layout and the platform's file handling.

## Parity by construction

Three decisions keep the doors aligned:

1. **One transport.** The desktop and mobile shells embed the HTTP API and the
   webview uses the same client as the browser. There are two Tauri commands
   in total (`desk_session`, `start_pairing`); everything else is a route.
2. **One contract.** The TypeScript types are generated from the Rust types
   and drift-checked (`pnpm contracts:gen`, `packages/contracts/src/drift.ts`).
3. **One desk.** Phone, tablet and desktop layouts are the same components with
   a responsive shell, not separate apps.

## Closing the remaining gaps

| Gap                                   | How it closes                                                                                         |
| ------------------------------------- | ----------------------------------------------------------------------------------------------------- |
| Compile download on mobile            | Tauri `dialog` + `fs` plugins to save, or the share sheet via the `share` community plugin             |
| Guest panes: repairs                  | Only if the pane can show the explanation chain; otherwise it stays a window onto the Scales           |
| Phone browser needs the LAN           | By design: the manuscript stays on the machine that owns it. The mobile shell is the offline answer    |
| Office and Workspace store listing    | Separate review processes; the panes are sideloaded until then                                         |
