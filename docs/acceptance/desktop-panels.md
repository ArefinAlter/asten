# Desktop panel resizing

Preferred sidebar, assistant and bottom-panel sizes now fit the measured workbench. Side panels preserve at least 320 pixels for the center at desktop widths; the bottom panel preserves usable space above it. Automatically shrinking the window does not overwrite preferred sizes, so they return when space allows. A user drag starts from actual rendered sizes.

Resize grips use pointer capture and clean up on release, cancellation, capture loss, Escape, focus loss, window resize and unmount. Keyboard arrows, Home and End use the same available-space bounds. Bottom-panel close/maximize controls remain outside the scrolling tab list. Compact drawers leave space for status and navigation.

Three pure-layout regression tests passed for constrained widths/heights, restoration of preferences, hidden peers and corrupted values. Windows Edge checks exercised all three grips with release, pointer cancellation, blur, capture loss and Escape, plus window resize during a drag.

Validation on 28 October 2026: frontend type checks, existing frontend tests and a production asset build passed. Fresh Windows Edge profiles passed an automated interaction run at 1366×768, 1024×600, 960×500, 800×600, 640×480, 390×500, 1280×400 and 1920×1080, including a return to the wide layout. Root scrolling stayed inside the window. Thread details could be opened, closed from within the pane, and reopened at narrow widths. Screenshots were inspected. These browser checks exercise the shared desktop CSS; installed Linux acceptance is not inferred from them.

The additional Windows Edge route matrix passed 6 workspace views at 1024×600, 800×600, 640×480 and 1280×400. The locked Windows release executable passed real WebView2 acceptance in a fresh scratch profile: the embedded API created a project and document, five physical window resizes retained a usable workspace without root overflow, and a native separator drag ended cleanly on Escape. Its binary hash and screenshots are retained with the Windows evidence. The existing 0.0.1c downloads are unchanged by this source patch.

A compact-window follow-up also covers keyboard resizing with large saved peer widths. The regression reproduced a sidebar shrinking from 282 to 232 pixels when asked to grow; the correction grows it to 283 pixels while retaining 320 pixels for the center. Both side panels start keyboard edits from their rendered peer sizes, and accessibility bounds use that same fitted geometry. The checked-in browser regression resets large preferences independently for both grips. The rebuilt Windows app passed the same constrained keyboard path, physical resizing and Escape cleanup in a newly named scratch profile.

Repeat the browser regression from the repository root:

```sh
pnpm --filter @asten/web exec vite build --mode sample --outDir dist-sample
node scripts/check-panel-keys.mjs
```

The script uses a fresh browser profile and a loopback-only server. `ASTEN_BROWSER_CHANNEL` selects an installed browser; optional `ASTEN_PANEL_ASSETS`, `ASTEN_PLAYWRIGHT_PACKAGE` and `ASTEN_PANEL_EVIDENCE` inputs support isolated build/test environments.
