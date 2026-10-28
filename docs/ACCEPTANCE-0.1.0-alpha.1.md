# Windows alpha 0.1.0-alpha.1 acceptance

This increment packages the merged draggable-panel fixes, installed-agent executable discovery and the official Grok Build ACP connector. Asten's assistant remains disabled until the writer enables it and grants provider consent. It never writes manuscript prose.

## Scope

Windows x64 NSIS installer, portable desktop ZIP and API/desk ZIP. Windows executables are not Authenticode signed. The installer has a separate Tauri updater signature. This is a GitHub prerelease and a manual download: the existing updater manifest remains on 0.0.1c. Linux retains its separately labeled 0.0.1c assets; this increment makes no new Linux binary/runtime claim.

## Recorded checks

- Native Windows frontend TypeScript build and all 52 Vitest assertions pass. The first parallel test launch hit worker-start timeouts; one-worker execution passed all seven files.
- Production desktop and sample frontends build. The checked-in panel keyboard regression passes against the sample frontend.
- Website TypeScript, production client/server and prerender builds pass. Windows alpha links are separate from prior Linux assets.
- The generated API document was refreshed using its export test; only the previously stale version value changed.

- Native Windows workspace tests pass: 336 assertions across 48 suites; five existing helper/manual tests are ignored. The run excludes desktop unit tests, while strict Clippy covers the complete workspace and all targets, including the desktop shell. Clippy passes with warnings denied.
- The first native workspace pass exposed timestamp collisions between parallel Windows gateway tests. An atomic per-process suffix now isolates each scratch SQLite sandbox, and the full workspace rerun passes.
- The packaged API starts against a fresh scratch database, serves its bundled desk, refuses unpaired requests, exchanges a fresh pairing code and creates/reads a project.
- Real WebView2 acceptance opens a scratch manuscript and checks physical window sizes from 1920 pixels down to 640, fitted keyboard bounds, pointer-drag Escape cleanup and absence of root overflow or page errors. Visual review confirmed the editor and panels; it also identified a stale prerelease badge, corrected before the final embedded-frontend rebuild.

The final rebuilt packages passed fresh-profile WebView2 and API acceptance. The compact assistant panel closes and reopens, leaving the full editor width available. No user profile or manuscripts were used. Final GUI receipt: `D:/verne-build/0.1/asten-evidence/native-gui/fresh-ac8938b41e/asten-results.json`; API receipt: `D:/verne-build/0.1/asten-evidence/packaged-api.json`. Native tests, strict lint, frontend and website logs are in the same evidence directory.

## Subscription boundary

Claude Code, Codex, Cursor and Antigravity run through their installed official CLI processes and existing sign-ins. Grok Build uses its own official login in a dedicated profile; see [setup and protocol evidence](acceptance/grok-subscription.md). Executable detection does not prove account authentication or entitlement. New Grok protocol fixtures cover the successful ACP exchange and refusal of API-only authentication, permissions, native tools, malformed/truncated/oversized replies and authentication-store separation. Live Grok inference has not been exercised. No browser cookies or provider credential contents are read or copied by Asten.

The prior 0.0.1c acceptance recorded a bounded native Claude subscription request. This release does not claim a new live request against every provider account. All connections keep the existing offline/consent and closed product-tool boundaries; the deterministic canon engine is unchanged.


## Published asset checksums

These are the final locally validated Windows assets, mirrored in the release's `SHA256SUMS.txt`.

| Asset | Bytes | SHA-256 |
|---|---:|---|
| `asten-api_0.1.0-alpha.1_windows_x64.zip` | 7,236,536 | `3e5f96b2c1dd6fa2fe4dfeef44f10675fc1b6fea6f69980b641ce9b31a0e4ff5` |
| `Asten_0.1.0-alpha.1_windows_x64_portable.zip` | 12,364,797 | `a6058d072f0ff48a619b8cb12bc335fcd935c131c9f7192cae670276bf4ef503` |
| `Asten_0.1.0-alpha.1_x64-setup.exe` | 8,979,906 | `ebdf541033808ec376a103907f7108e045481ca16f627f09761ea0684c32676a` |
| `Asten_0.1.0-alpha.1_x64-setup.exe.sig` | 456 | `784f92bbe89fc5f7f9cfce99693a85612752192e6bbadf1bab1399015ca5ac82` |
