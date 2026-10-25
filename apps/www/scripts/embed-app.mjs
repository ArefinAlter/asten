// Copies the workbench's hosted build (apps/web, `pnpm --filter @asten/web
// build:site`) into the site's output at /app/, where Vercel serves it. It is
// the pre-alpha release on sample data: no backend, no network, noindex.
// Without that build (a site-only build), it says so and leaves /app/ out.

import { cpSync, existsSync, readFileSync, rmSync } from "node:fs";
import { dirname, join, resolve } from "node:path";
import { fileURLToPath } from "node:url";

const here = dirname(fileURLToPath(import.meta.url));
const source = resolve(here, "../../web/dist-site");
const target = resolve(here, "../dist/app");

if (!existsSync(join(source, "index.html"))) {
  console.warn("embed-app: no workbench build at apps/web/dist-site; /app/ is left out.");
  process.exit(0);
}
const html = readFileSync(join(source, "index.html"), "utf8");
if (!html.includes('name="robots" content="noindex')) {
  console.error("embed-app: the workbench build is not marked noindex; build it with --mode site.");
  process.exit(1);
}
if (/<script>(?!\s*<\/script>)/.test(html)) {
  console.error("embed-app: the workbench build has an inline script, which the site's CSP refuses.");
  process.exit(1);
}
rmSync(target, { recursive: true, force: true });
cpSync(source, target, { recursive: true });
// The installable-app files belong to the local desk, not to this copy.
for (const file of ["manifest.webmanifest", "sw.js"]) rmSync(join(target, file), { force: true });
console.log("embed-app: workbench copied to dist/app/.");
