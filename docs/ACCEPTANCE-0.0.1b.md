# Acceptance of release 0.0.1b

What was run, on what, with what result, and what was not. Written on 27
October 2026 for the release `v0.0.1b` (`0.0.1-beta`). The scripts are in
`tools/acceptance/`; the logs of the final run are in
`docs/acceptance/0.0.1b/`; a few screenshots are in `docs/images/`.

## How it was run

- **Build under test:** the release installer, `Asten_0.0.1-beta_x64-setup.exe`
  (the file published as an asset), installed silently into a scratch folder
  on Windows 11 Pro (x64, 16 logical cores, 32 GB), and the installed
  `asten-desktop.exe` driven as a person would drive it. The first-run, settings
  and project-file scenarios also ran on the release `asten-desktop.exe` before
  packaging, with the same results.
- **Isolation:** every run starts the app with `ASTEN_DATA_DIR`,
  `WEBVIEW2_USER_DATA_FOLDER`, `APPDATA` and `LOCALAPPDATA` pointed at a scratch
  profile, so the owner's real `%APPDATA%\com.asten.app` is never read or
  written. `ui.mjs` checks that folder is byte-for-byte unchanged.
- **How the window is driven:** WebView2 is started with a remote debugging
  port and Playwright attaches over CDP (`tools/acceptance/lib.mjs`). The
  session token comes from the app's own `desk_session` command. Native Windows
  dialogs (Save As, "Restore the library") are answered through UI Automation
  and the keyboard (`dialog.ps1`), because the page cannot reach them. Windows
  Credential Manager is the real one, under a throwaway service name that the
  script removes.
- **Real versus stand-in:** the app, the engine, the SQLite file, the editor,
  the dialogs, the credential store, the installer, the updater and the Linux
  packages are real. Providers are mock servers on loopback that speak each
  vendor's wire format. Office.js is a stand-in (no Word was available). The
  public-domain novel is Project Gutenberg #1342 (127,360 words, 63 chapters);
  it stays in the scratch folder and is not committed.

## The nine scenarios

| # | Scenario | Result | Evidence |
| --- | --- | --- | --- |
| 1 | Fresh install, sample novel, new project, autosave, reopen | Passed | `ui.log` (26 of 26); `docs/images/asten-0.0.1b-welcome.png`, `asten-0.0.1b-sample-novel.png` |
| 2 | A real long manuscript and a Word file; typing, engine, numbers | Passed, with a limit on one very long scene | `documents.log` (73 of 73), `perf.log`, `typing.log`, `typing-whole-book.log` |
| 3 | The canon workflow end to end | Passed | `canon.log` (24 of 24) |
| 4 | Compile and export, import of six formats | Passed | `documents.log` |
| 5 | Word and Google Docs guests over loopback | Passed, with limits | `guests.log` (23 of 23) |
| 6 | The bring-your-own-key assistant | Passed against mock providers; live providers not run | `assistant.log` (25 of 25), `cargo test -p asten-assistant` |
| 7 | The updater, older build to this release | Passed | `updater.log` (9 of 9), `updater-bad-signature.log` (7 of 7) |
| 8 | Linux: AppImage and .deb | Passed, with limits | `linux-clean-install-first-run.log`, `linux-clean-install.log`; `docs/images/asten-0.0.1b-linux-appimage.png` |
| 9 | Settings, keybindings, themes, storage, project file, backup and restore | Passed | `ui.log`, `restore.log` (8 of 8) |

### 1. First run, sample, new project, autosave, reopen

A fresh profile opens on the welcome page with no project. **Open the sample
novel** creates "Salt and Lantern" (5 scenes, 9 cast entries; the engine finds
three marks). File, New, Novel makes a second project. Typing a line at the end
of a sample scene is saved without any button (the status line reads "saved")
and is in the stored scene. After closing and starting again both projects and
the typed line are there and the desk opens where it was left. The library is
one file in the scratch folder.

### 2. A long manuscript and a Word file

- The novel imports split into 63 chapter documents; the Word file (made with
  python-docx: headings, italics, comments, tracked insertions and deletions,
  a list) imports with the structure intact, comments with their ranges and
  revisions as marks.
- Numbers on the installed build, 127,360 words in 2,448 paragraphs:

| What | Result |
| --- | --- |
| Import, split into chapters | 4.3 s in the final run (1.4 s to 5.1 s across five runs) |
| First full check (cold) | 0.96 s wall; the engine itself 42 ms (perceive 12, analyse 29); 249 marks |
| Full check again (warm, cached) | median 75 ms (69 to 84) |
| Save one edited paragraph | median 7 ms (3 to 94) |
| Check after that edit (incremental) | median 83 ms (78 to 222) |
| Typing in a 1,719-word chapter: key to the key's pixels on screen | 474 keystrokes, median 5.1 ms, p95 8.5 ms, worst 16.9 ms |
| A sentence that contradicts a settled fact: mark on the server / in the window | 1.3 s / 1.3 s after the last key |
| The whole novel as one 127,360-word scene: page opens | 2.3 s |
| Typing in that scene | median 33.0 ms, p95 42.6 ms, worst 137.8 ms |
| The same contradiction in that scene: mark on the server / in the window | 1.7 s / 2.6 s |
| Library file after all of this | 7.5 MB |

The numbers are from the installed build on a machine with nothing else heavy
running (an earlier run, while other builds were going, was slower but passed
the same limits). A whole novel in one scene is slower to type in than a
chapter (about 33 ms against 5 ms per keystroke) but stays under the 50 ms the
script allows; splitting at chapters, which the importer does, is the intended
way to work.

### 3. The canon workflow

Facts settled, a cast, events with story and reading order, a world board
entry, routes; a mark appears when a sentence contradicts a settled fact and
clears when the sentence is fixed; a repair the writer presses ("Add an event")
is applied and reported; suppress with a reason, lift the suppression; a
baseline freezes today's marks and a later mark is flagged new; a named
snapshot, a draft and its three-way merge preview and merge, history with a
verified hash chain, blame by actor, a snapshot restored as a new entry,
bisect to the version that raised a mark; and the Source Control, knowledge,
timeline and world views show what the API holds.

### 4. Compile, export, import

Every format was compiled from the novel and from the Word file and checked by
script: Markdown, plain text, HTML (tags balance, UTF-8 and language declared,
emphasis kept), Fountain (title page), DOCX (opens with python-docx, headings
and italics present, tracked changes are real Word revisions, core parts
present), EPUB 3 (mimetype first and stored, container and package present,
every manifest item exists, spine in the manifest, navigation document and
NCX, title, identifier, language and modified date, every XML part well
formed), and Final Draft (well-formed XML, root `FinalDraft`, typed
paragraphs). The word counts of the exports match the manuscript (127k to
128k words each). Markdown, plain text, HTML and Word exports import again
(64, 63 and 63 chapters), and Fountain and Final Draft files import.

### 5. Word and Google Docs guests

A pairing code minted by the app pairs a guest from the Origin a Word task
pane (`https://localhost:3000`) and an Apps Script sidebar send; the code works
once; the CORS and private-network preflights allow those origins and refuse a
stranger's; a guest lists projects, files a selection as a scene, reads the
Scales, opens the change stream with a one-minute ticket, cannot mint a code or
list tokens, can revoke itself and is refused afterwards. The real
`apps/addin-word/taskpane.html` was served over HTTPS on port 3000 and driven in
Chromium with a stub for Office.js: it paired with the loopback API, followed
the stream, filed a selection as a scene, and forgetting the pairing revoked
the token.

What works and what needs HTTPS: the API listens on `http://127.0.0.1:4317`
(or another loopback port if 4317 is taken). A task pane served from
`https://localhost:3000` may call it (Chromium and Edge treat loopback as
secure), so Word on Windows works; the pane itself must be served over HTTPS
(`apps/addin-word/README.md`). Word for Mac and Safari block an https page
calling http loopback and were not tested. The Docs sidebar's origin and the
preflight were tested, not Apps Script itself.

### 6. The assistant

`assistant.mjs` drives the real window against a mock Anthropic server on
loopback, with the real Windows credential store. It shows that: providers are
off by default; a key typed once is stored in Credential Manager, the page
never gets it back and finds it in no file the app or its web view wrote, and
forgetting it removes it; consent is per host and the consent preview names the
destination and shows no key; the exact request can be read first (host, the
tools offered, no manuscript text, never the key); the answer streams into the
panel after a tool round (two requests); the key travels only in the provider's
authentication header; no manuscript sentence is sent; only the tools the
access level allows are offered (eight in read-only mode; the closed ones are
not); the tool's card, the request's digest and the tool call appear in the
log; the spend ledger records both calls with tokens and cost and survives a
restart; a request to write prose never reaches the provider; offline mode
blocks a cloud provider; a cap of zero means no call; and settings, consent and
the key's presence survive a restart.

`cargo test -p asten-assistant` (75 tests) covers, beyond that, the wire
formats with mock servers for the Anthropic Messages API, the OpenAI Chat Completions family
(OpenAI, Mistral, DeepSeek, Qwen, Kimi, GLM, Ollama and any compatible
address) and Gemini: streaming, tool calling and the JSON fallback, retries
and `retry-after`, cancellation, redirects refused, size and idle limits, keys
never in output (canary strings searched in every output), consent tied to the
host, and the four layers against prose, including mock providers that try to
comply with a request for prose (by text, or by calling a closed or invented
tool) and are stopped. **No live provider was called**: that needs
the owner's key for each vendor.

### 7. The updater

`updater.mjs` builds the scenario for real. A local build of the same code with
version 0.0.1-alpha, whose update address is a loopback server (the public key
is the release key), is installed silently into a scratch folder and started.
The script serves a manifest naming the release installer and its real
signature. In Settings, Continuity checks, This desk: nothing is requested
until **Check for updates** is pressed; it finds 0.0.1-beta; **Install and
relaunch** downloads, verifies and runs the installer; the installed program
becomes 0.0.1-beta; the app relaunches by itself; the project made before the
update is still there; the uninstaller removes the program. With a signature
that belongs to a different file the update is refused ("The signature
verification failed") and the installed program is unchanged.

`apps/www/public/updates/latest.json` carries the real signatures of the
installer and the AppImage and the GitHub download addresses of 0.0.1b. Not
tested: an update from the published 0.0.1a (its address is the production
site, which only serves 0.0.1b after this release is merged), and the AppImage
updating itself.

### 8. Linux

Built in a private Ubuntu 22.04 distribution under WSL 2 on the release
machine (the VPS was not used; no password was available): the AppImage and the
.deb (signed for the updater with the release key passed through the
environment) and the `asten-api` archive.

- The AppImage ran under `xvfb-run` (X11 forced; Wayland would open on the
  Windows desktop through WSLg): the embedded API answers `/health` with
  `{"status":"ok","service":"asten-api","version":"0.0.1-beta"}`, the window is
  titled "Asten" at 1440x900, and the screenshot shows the welcome page
  (`docs/images/asten-0.0.1b-linux-appimage.png`).
- The .deb was installed with `apt` on a second, clean Ubuntu 22.04 (95 packages
  pulled in to satisfy `libwebkit2gtk-4.1-0` and `libgtk-3-0`), the desktop entry
  validates (`desktop-file-validate`), the hicolor icons (32, 128, 256@2) are the
  new mark, `ldd` finds no missing library, the app starts and answers
  `/health`, and `apt remove` leaves no file behind
  (`linux-clean-install-first-run.log` is the install with dependencies pulled in; `linux-clean-install.log` the run after).

Limits: no physical machine or GPU was available. Under a virtual display the
window needs `WEBKIT_DISABLE_COMPOSITING_MODE=1 WEBKIT_DISABLE_DMABUF_RENDERER=1`
to paint; on real hardware WebKitGTK normally does not. The AppImage needs FUSE
2 (`libfuse2`) or `--appimage-extract-and-run`. Linux was not driven with
Playwright (WebKitGTK has no CDP), so scenarios 1 to 6 and 9 were run on
Windows only; the Linux checks above cover launch, the embedded API, rendering
and packaging.

### 9. Settings, storage, project file, backup and restore

The theme (Dark), the accent and the density are changed and are the same after
a restart; a keybinding is recorded (F7 to Ctrl+Alt+J) and kept; the AI switch
is off by default and says what off means; Storage shows the library file and
**Back up now** writes a consistent copy; **Save a project file** writes a
`.asten` file through the real Windows Save As dialog (valid, `asten-project`
version 1, with the manuscript); **Open a project file** creates a new project
from it; a project is renamed and deleted (by typing its name) from the Open
page. `restore.mjs` makes a backup, adds a project, then restores the backup
from Settings: Escape in the native confirmation changes nothing; confirming
restarts the app, the later project is gone, the app reports the restore, and
the library it replaced is kept as `before-restore-…db`.

## Windows installer, shortcuts and icons

The installer installs for the user into the chosen folder, creates the Start
menu and desktop shortcuts and an uninstall entry (Verne Ltd., version
0.0.1-beta), and the uninstaller removes the folder, the shortcuts and the
registry entry. The program, the portable build, the installer and the
uninstaller all carry the Asten mark (`docs/images/asten-icons-installed.png`),
and the running window's taskbar button shows it
(`docs/images/asten-taskbar-button.png`). The installer's wizard has a header
and a side picture drawn from the mark.

## Gates

`cargo fmt --check` clean; `cargo clippy --workspace --all-targets -- -D warnings`
clean; `cargo test --workspace --exclude asten-desktop` 324 passed, 0 failed, and
`cargo test -p asten-desktop` 7 passed; web `tsc -b`, vitest (49 tests) and
`prettier --check .` clean; `pnpm contracts:gen` leaves no drift. The release
binaries were built from the commit before the last two clippy clean-ups (an
`if` collapsed in the Markdown importer, a `match` in a test); neither changes
behaviour.

## What was not done, and why

- **Live providers.** Needs a key per vendor from the owner.
- **Word itself, Apps Script, Word for Mac.** No Word or Google account here.
- **The hosted pre-alpha.** It stays on 0.0.1a: rebuilding its image needs the
  VPS password (`OGUN_VPS_PW`), which was not provided. The hosted favicon
  therefore is still the old one until that is done.
- **Linux on hardware, macOS, Windows ARM64.** No such machines.
- **Code signing.** The builds are unsigned (SmartScreen warns).
- **Web sign-in.** Not applicable: Asten has no accounts by design.
- **Eye colours.** A brown fact against "green eyes" raises no mark (hazel
  sits between the two in the colour lattice); blue, grey and black against
  brown do. Hair colours behave the same way.

## Running it again

```powershell
# installed build, scratch profile (the scripts never touch %APPDATA%\com.asten.app)
$exe = "D:/verne-build/asten/install-final/asten-desktop.exe"
foreach ($s in "documents","canon","guests","assistant","ui","restore","perf","typing") {
  node tools/acceptance/$s.mjs --exe $exe
}
node tools/acceptance/typing.mjs --exe $exe --whole-book
node tools/acceptance/updater.mjs --old <older installer> --new <release installer>
```

Needs Node 22, Playwright, Python with `python-docx`, the Gutenberg text under
`scratch/gutenberg/`, and Windows PowerShell for the dialogs.
