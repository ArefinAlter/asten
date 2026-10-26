// First launch on a scratch profile: window opens, the embedded API answers,
// the data lands in the scratch folder, and a second launch is turned away.
import { readdirSync } from "node:fs";
import { join } from "node:path";
import { apiClient, args, deskSession, launchApp, logTo, sleep } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-smoke";
const out = args.out ?? "D:/verne-build/asten/evidence";
const log = logTo(join(out, "smoke.log"));

const app = await launchApp({ exe, profile });
try {
  const { page } = app;
  await page.waitForSelector("body", { timeout: 20000 });
  await sleep(2500);
  const session = await deskSession(page);
  log({ origin: session.origin, version: session.version, database: session.database_path });
  const health = await fetch(`${session.origin}/health`);
  log({ health: health.status });
  const api = apiClient(session.origin, session.token);
  log({ projects: (await api.get("/projects")).length });
  log({ files: readdirSync(app.data) });
  await page.screenshot({ path: join(out, "smoke-first-launch.png") });
  log({ title: await page.title(), url: page.url() });
} finally {
  await app.close();
}
