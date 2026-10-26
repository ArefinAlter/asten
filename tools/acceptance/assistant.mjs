// Acceptance for the BYOK assistant, in the real desktop app.
//
// A mock Anthropic server on loopback stands in for the provider (no real key
// is used). The app runs on a scratch profile with the operating system's real
// credential store under a throwaway service name. Everything is driven through
// the window the way a writer would, and the checks read what the mock saw, the
// ledger, the credential store and every file the app wrote.
//
//   node tools/acceptance/assistant.mjs [--exe <asten-desktop.exe>]
import { execFileSync } from "node:child_process";
import { createServer } from "node:http";
import { readdirSync, readFileSync, statSync } from "node:fs";
import { join } from "node:path";
import { args, deskSession, launchApp, logTo, sleep, until } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-assistant";
const out = args.out ?? "D:/verne-build/asten/evidence";
const log = logTo(join(out, "assistant.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail });
};

const CANARY = `ASTEN-CANARY-${Math.random().toString(16).slice(2, 10)}-KEY`;
const SERVICE = `asten-acceptance-${Math.random().toString(16).slice(2, 8)}`;
const EXPLANATION =
  "CANON-E013 marks a clash about Mara Vell's eye colour: the canon settles grey, and a later scene reads brown. The rule offers repairs on the card, and keeping your sentence is always one of them.";

// ── the mock provider ──
const seen = [];
const server = createServer((req, res) => {
  const chunks = [];
  req.on("data", (c) => chunks.push(c));
  req.on("end", () => {
    const raw = Buffer.concat(chunks);
    let body = {};
    try {
      body = JSON.parse(raw.toString());
    } catch {}
    seen.push({ path: req.url, headers: req.headers, body, raw: raw.toString() });
    const last = body.messages?.at?.(-1);
    const gotResult = Array.isArray(last?.content) && last.content[0]?.type === "tool_result";
    res.writeHead(200, { "content-type": "text/event-stream" });
    const send = (event, data) => res.write(`event: ${event}\ndata: ${JSON.stringify(data)}\n\n`);
    send("message_start", {
      type: "message_start",
      message: { id: "m", role: "assistant", content: [], usage: { input_tokens: gotResult ? 900 : 700, output_tokens: 1 } },
    });
    if (!gotResult) {
      send("content_block_start", { type: "content_block_start", index: 0, content_block: { type: "tool_use", id: "toolu_acc", name: "marks__list", input: {} } });
      send("content_block_delta", { type: "content_block_delta", index: 0, delta: { type: "input_json_delta", partial_json: "{}" } });
      send("content_block_stop", { type: "content_block_stop", index: 0 });
      send("message_delta", { type: "message_delta", delta: { stop_reason: "tool_use" }, usage: { output_tokens: 30 } });
    } else {
      send("content_block_start", { type: "content_block_start", index: 0, content_block: { type: "text", text: "" } });
      for (const piece of EXPLANATION.match(/.{1,40}/g)) {
        send("content_block_delta", { type: "content_block_delta", index: 0, delta: { type: "text_delta", text: piece } });
      }
      send("content_block_stop", { type: "content_block_stop", index: 0 });
      send("message_delta", { type: "message_delta", delta: { stop_reason: "end_turn" }, usage: { output_tokens: 55 } });
    }
    send("message_stop", { type: "message_stop" });
    res.end();
  });
});
await new Promise((r) => server.listen(0, "127.0.0.1", r));
const base = `http://127.0.0.1:${server.address().port}`;

const credentialTargets = () => {
  try {
    return execFileSync("cmdkey", ["/list"], { encoding: "utf8" })
      .split(/\r?\n/)
      .filter((l) => l.includes(SERVICE));
  } catch {
    return [];
  }
};

const walk = (dir, files = []) => {
  for (const name of readdirSync(dir)) {
    const p = join(dir, name);
    let st;
    try {
      st = statSync(p);
    } catch {
      continue;
    }
    if (st.isDirectory()) walk(p, files);
    else files.push(p);
  }
  return files;
};

const open = async (fresh = true) =>
  launchApp({ exe, profile, fresh, env: { ASTEN_KEY_STORE: "", ASTEN_KEYRING_SERVICE: SERVICE } });

let app = await open();
try {
  let { page } = app;
  const invoke = (cmd, a) =>
    page.evaluate(([c, x]) => window.__TAURI_INTERNALS__.invoke(c, x), [cmd, a]);
  await deskSession(page);

  // 1. Open the sample novel through the window.
  await page.getByRole("button", { name: "Open the sample novel" }).click();
  await page.waitForSelector("text=Recently touched", { timeout: 30000 });
  await sleep(1500);
  check("the sample novel opens", true);

  // 2. Off by default: the first status says so, nothing can be sent.
  let status = await invoke("ai_status", { threadId: null });
  check("providers are off by default", status.enabled === false && status.providers.every((p) => !p.enabled && !p.consent));
  check("keys are kept in the OS credential store", status.vault.backend === "os", status.vault.backend);

  // 3. Settings → AI providers, through the palette.
  await page.keyboard.press("Control+k");
  await page.keyboard.type("Settings: AI providers");
  await sleep(300);
  await page.keyboard.press("Enter");
  await page.waitForSelector("text=Allow AI providers");
  await page.screenshot({ path: join(out, "assistant-1-providers.png") });

  // Turn the master switch on, point Anthropic at the mock, store a key, consent.
  await page.getByRole("switch", { name: "Allow AI providers" }).click();
  await until(async () => (await invoke("ai_status", { threadId: null })).enabled, { label: "master switch" });
  const urlBox = page.getByLabel("Anthropic base URL");
  await urlBox.fill(base);
  await urlBox.blur();
  await until(async () => (await invoke("ai_status", { threadId: null })).providers.find((p) => p.id === "anthropic").endpoint === base, { label: "endpoint saved" });
  await page.getByLabel("Anthropic key").fill(CANARY);
  await page.getByRole("button", { name: "Store key" }).first().click();
  await until(async () => (await invoke("ai_status", { threadId: null })).providers.find((p) => p.id === "anthropic").key_stored, { label: "key stored" });
  check("the page never gets the key back", !JSON.stringify(await invoke("ai_status", { threadId: null })).includes(CANARY));
  const dom = await page.content();
  check("the key is not in the page after storing it", !dom.includes(CANARY));
  const creds = credentialTargets();
  check("the credential store holds the key", creds.length >= 1, creds.join(" | "));
  await page.getByRole("switch", { name: "Use Anthropic" }).click();
  await page.waitForSelector("text=Use Anthropic?");
  await page.screenshot({ path: join(out, "assistant-2-consent.png") });
  await page.getByRole("button", { name: "Show the exact request" }).click();
  await page.waitForSelector("text=The exact request");
  const previewText = await page.locator(".preview-dialog").innerText();
  check("the consent preview shows the destination and no key", previewText.includes("127.0.0.1") && !previewText.includes(CANARY));
  await page.screenshot({ path: join(out, "assistant-3-preview-in-consent.png") });
  await page.locator(".preview-dialog .dialog-actions button", { hasText: "Close" }).click();
  await page.getByRole("button", { name: "I consent for Anthropic" }).click();
  await until(async () => (await invoke("ai_status", { threadId: null })).providers.find((p) => p.id === "anthropic").consent_valid, { label: "consent" });
  check("consent is recorded for the host", true);
  await page.screenshot({ path: join(out, "assistant-4-ready.png") });

  // 4. Back to the page; pick the provider; ask.
  await page.getByRole("button", { name: "Assistant", exact: true }).click().catch(() => undefined);
  await sleep(500);
  await page.getByLabel("Model", { exact: true }).selectOption({ label: "Anthropic · claude-haiku-5-5" });
  await page.getByLabel("Message the assistant").fill("Explain the open mark about Mara's eyes");
  await page.getByRole("button", { name: "Show what will be sent" }).click();
  await page.waitForSelector(".preview-dialog");
  const preview = await page.locator(".preview-dialog").innerText();
  check("the preview names the host, offers the tools and shows no manuscript text", preview.includes("127.0.0.1") && preview.includes("marks.list") && preview.includes("none"));
  check("the preview never shows the key", !preview.includes(CANARY));
  await page.screenshot({ path: join(out, "assistant-5-preview.png") });
  await page.locator(".preview-dialog").getByRole("button", { name: "Send", exact: true }).click();
  await page.waitForSelector(".msg-assistant .prose-reply", { timeout: 30000 });
  await until(async () => (await page.locator(".msg-assistant .prose-reply").last().innerText()).includes("CANON-E013"), { label: "answer" });
  await sleep(500);
  await page.screenshot({ path: join(out, "assistant-6-answer.png") });
  check("the answer streamed into the panel", true);
  check("two requests were made (tool round, then answer)", seen.length === 2, String(seen.length));
  check("the key went in the x-api-key header only", seen.every((s) => s.headers["x-api-key"] === CANARY) && seen.every((s) => !s.raw.includes(CANARY)));
  check("no manuscript sentence was sent", seen.every((s) => !s.raw.includes("wiped a circle clear") && !s.raw.includes("Harrowmouth: forty roofs")));
  check("closed tools were not offered", seen[0].body.tools.every((t) => !t.name.startsWith("manuscript") && t.name !== "facts__settle"), seen[0].body.tools.map((t) => t.name).join(","));
  const card = await page.locator(".agent-card").count();
  check("the tool's card appeared in the panel", card > 0, String(card));
  await page.locator(".tool-log button").first().click();
  const toolLog = await page.locator(".model-log").first().innerText();
  check("the log lists the request, its digest and the tool call", toolLog.includes("sha256") && toolLog.includes("marks.list"));
  await page.screenshot({ path: join(out, "assistant-7-log.png") });
  const ledger = await invoke("ai_ledger", { threadId: null });
  check("the ledger recorded both calls with tokens", ledger.totals.day_calls === 2 && ledger.totals.day_input === 1600 && ledger.totals.day_output === 85, JSON.stringify(ledger.totals));

  // 5. Prose requests are refused before any provider is contacted.
  const before = seen.length;
  for (const text of ["write the next paragraph", "Continue the story", "Rewrite this sentence"]) {
    await page.getByLabel("Message the assistant").fill(text);
    await page.getByRole("button", { name: "Send", exact: true }).click();
    await until(async () => (await page.locator(".msg-assistant .prose-reply").last().innerText()).includes("never write"), { label: `refusal of ${text}` });
  }
  check("prose requests never reached the provider", seen.length === before, `${seen.length - before} extra requests`);
  await page.screenshot({ path: join(out, "assistant-8-refusal.png") });

  // 6. Offline mode and caps stop a call.
  await page.getByRole("button", { name: "Offline mode" }).click();
  await until(async () => (await invoke("ai_status", { threadId: null })).offline, { label: "offline" });
  await page.getByLabel("Message the assistant").fill("Which marks are open?");
  await page.getByRole("button", { name: "Send", exact: true }).click();
  await until(async () => (await page.locator(".agent-error").count()) > 0, { label: "offline error" });
  const offlineText = await page.locator(".agent-error").last().innerText();
  check("offline mode blocks a cloud provider", offlineText.includes("Offline") && seen.length === before, offlineText);
  await page.screenshot({ path: join(out, "assistant-9-offline.png") });
  await page.getByRole("button", { name: "Offline mode" }).click();
  await invoke("ai_update", { patch: { caps: { daily_tokens: 0, thread_tokens: 60000, daily_usd: 1 } } });
  await page.getByLabel("Message the assistant").fill("Which marks are open?");
  await page.getByRole("button", { name: "Send", exact: true }).click();
  await until(async () => (await page.locator(".agent-error").count()) > 0 && (await page.locator(".agent-error").last().innerText()).includes("cap"), { label: "cap error" });
  check("a cap of zero means no call", seen.length === before);
  await invoke("ai_update", { patch: { caps: { daily_tokens: 200000, thread_tokens: 60000, daily_usd: 1 } } });

  // 7. Everything persists across a restart; nothing on disk holds the key.
  await app.close();
  app = await open(false);
  ({ page } = app);
  await deskSession(page);
  status = await page.evaluate(() => window.__TAURI_INTERNALS__.invoke("ai_status", { threadId: null }));
  const anthropic = status.providers.find((p) => p.id === "anthropic");
  check("settings, consent and the key's presence survive a restart", status.enabled && anthropic.consent_valid && anthropic.key_stored && anthropic.endpoint === base);
  check("the ledger survives a restart", status.totals.day_calls === 2, JSON.stringify(status.totals));
  const leaked = walk(profile).filter((f) => {
    try {
      return readFileSync(f).includes(CANARY);
    } catch {
      return false;
    }
  });
  check("the key is in no file the app or its web view wrote", leaked.length === 0, leaked.join(", "));

  // 8. Forget removes it from the credential store.
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke("ai_forget_key", { provider: "anthropic" }));
  await sleep(500);
  check("forgetting the key removes it from the credential store", credentialTargets().length === 0, credentialTargets().join(" | "));
  await page.evaluate(() => window.__TAURI_INTERNALS__.invoke("ai_withdraw_all"));
} finally {
  await app.close();
  server.close();
  for (const line of credentialTargets()) {
    const target = line.split("target=")[1]?.trim();
    if (target) {
      try {
        execFileSync("cmdkey", [`/delete:${target}`]);
      } catch {}
    }
  }
}
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => f.name) });
process.exit(failed.length ? 1 : 0);
