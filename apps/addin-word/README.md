# Asten for Word

Asten is a Verne product. This task pane is a guest. It lists local projects,
shows the Scales for one of them, and files the Word selection into the project
on this machine. Canon is not uploaded to Verne.

## Pairing

The API answers only to paired clients. The first time the pane opens it asks
for a six-digit code. Get one from the Asten desk under **Guests → Show a
code**, or read the code the API prints when it starts with no paired client.
The pane exchanges the code for a bearer token once and keeps the token in the
pane's own storage. **Forget this pairing** at the bottom of the pane revokes
it; the desk's Guests panel can revoke it too.

## Sideload

1. Start the API: `pnpm dev:api` (or open the desktop app, which embeds it).
2. Serve this folder at `https://localhost:3000`. Word will not load the pane
   from plain HTTP. Use a local certificate you already trust.
3. In Word: Insert → Add-ins → Upload My Add-in, and choose `manifest.xml`.
4. The pane calls `http://127.0.0.1:4317` by default. Change the origin on the
   pairing screen if the API listens elsewhere.

`icon-32.png` and `icon-64.png` sit beside the manifest.
