# Asten for Google Docs

Asten is a Verne product. This sidebar is a guest. It reads the selection in
the open Doc and files that prose to the Asten API on `127.0.0.1:4317`.
Canon is not copied to Google or to Verne.

## Pairing

The API answers only to paired clients. The first time the sidebar opens it
asks for a six-digit code. Get one from the Asten desk under **Guests → Show a
code**, or read the code the API prints when it starts with no paired client.
The sidebar exchanges the code for a bearer token once. Because the sidebar's
iframe origin changes between loads, the token is kept in the script's user
properties (private to your Google account and this script), not in the
browser. **Forget this pairing** revokes it; so does the desk's Guests panel.

## Sideload

1. Start the local API: `pnpm dev:api` in the Asten repository, or open the
   desktop app.
2. In the Google Doc, open Extensions → Apps Script.
3. Replace the script with `Code.gs`. Add `Sidebar.html` and `appsscript.json`.
4. Reload the Doc. Use the Asten menu → Open the guest pane.

Chrome may block a Google-hosted sidebar from calling loopback until the API
answers `Access-Control-Allow-Private-Network: true`. The Asten API sends that
header for `https://*.googleusercontent.com` origins. If the pane still cannot
see the API, file the scene from the desk instead. The desk remains the source
of truth.
