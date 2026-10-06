# Asten for Word

Asten is a Verne product. This task pane is a guest. It lists local projects,
shows diagnostics, and files the Word selection into the project on this
machine. Canon is not uploaded to Verne.

## Sideload

1. Start the API: `pnpm dev:api`.
2. Serve this folder at `https://localhost:3000`. Word will not load the pane
   from plain HTTP. Use a local certificate you already trust.
3. In Word: Insert → Add-ins → Upload My Add-in, and choose `manifest.xml`.
4. The pane calls `http://127.0.0.1:4317`.

`icon-32.png` and `icon-64.png` sit beside the manifest.
