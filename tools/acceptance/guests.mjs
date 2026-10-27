// Acceptance: the Word and Google Docs guests against the desktop app's
// loopback API.
//
//   node tools/acceptance/guests.mjs
//
// What is checked for real: a pairing code minted by the app, redeemed with the
// Origin a Word task pane and an Apps Script sidebar send; the CORS and private
// network preflights for each origin (and a stranger's); what a guest token may
// and may not do; and the actual Word task pane (apps/addin-word/taskpane.html)
// served from https://localhost:3000 and driven in Chromium with a stub for
// Office.js, because no Word host exists on this machine. What cannot be checked
// here is said in the output.
import { execFileSync } from "node:child_process";
import { existsSync, mkdirSync, readFileSync } from "node:fs";
import { createServer } from "node:https";
import { request } from "node:http";
import { join } from "node:path";
import { args, chromium, deskSession, launchApp, logTo, sleep, until } from "./lib.mjs";

const exe = args.exe ?? "D:/verne-build/asten/cargo-target/release/asten-desktop.exe";
const profile = args.profile ?? "D:/verne-build/asten/scratch/profile-guests";
const out = args.out ?? "D:/verne-build/asten/evidence";
const log = logTo(join(out, "guests.log"));
const results = [];
const check = (name, ok, detail = "") => {
  results.push({ name, ok: Boolean(ok), detail });
  log({ check: name, ok: Boolean(ok), detail: typeof detail === "string" ? detail : JSON.stringify(detail) });
};

function raw(method, url, { headers = {}, body, headersOnly = false } = {}) {
  return new Promise((resolve, reject) => {
    const target = new URL(url);
    const req = request(
      { host: target.hostname, port: target.port, path: target.pathname + target.search, method, headers },
      (res) => {
        if (headersOnly) {
          // A stream never ends: take the status and headers, then hang up.
          resolve({ status: res.statusCode, headers: res.headers, text: "", json: undefined });
          res.destroy();
          return;
        }
        const chunks = [];
        res.on("data", (c) => chunks.push(c));
        res.on("end", () => {
          const text = Buffer.concat(chunks).toString("utf8");
          let json;
          try { json = JSON.parse(text); } catch {}
          resolve({ status: res.statusCode, headers: res.headers, text, json });
        });
      },
    );
    req.on("error", reject);
    if (body) req.write(typeof body === "string" ? body : JSON.stringify(body));
    req.end();
  });
}

const WORD = "https://localhost:3000";
const DOCS = "https://n-abcdefg123-0lu-script.googleusercontent.com";
const STRANGER = "https://evil.example";

const app = await launchApp({ exe, profile });
let tls;
try {
  const { page } = app;
  const session = await deskSession(page);
  const origin = session.origin;
  const mint = async (label) => page.evaluate((l) => window.__TAURI_INTERNALS__.invoke("start_pairing", { label: l }), label);
  const deskHeaders = { authorization: `Bearer ${session.token}`, "content-type": "application/json" };
  const project = (await raw("POST", `${origin}/api/v1/projects`, { headers: deskHeaders, body: { name: "Guest test", kind: "novel", description: null } })).json;
  log({ api: origin, defaultPortFree: origin.endsWith(":4317") });

  // ── preflights ──
  for (const [name, o, allowed] of [["Word task pane", WORD, true], ["Apps Script sidebar", DOCS, true], ["a stranger's page", STRANGER, false]]) {
    const r = await raw("OPTIONS", `${origin}/api/v1/projects`, {
      headers: { origin: o, "access-control-request-method": "GET", "access-control-request-headers": "authorization", "access-control-request-private-network": "true" },
    });
    const allow = r.headers["access-control-allow-origin"];
    const pna = r.headers["access-control-allow-private-network"];
    check(`preflight from ${name}: ${allowed ? "allowed" : "refused"}`, allowed ? allow === o && pna === "true" : !allow && !pna, `${r.status} allow-origin=${allow} private-network=${pna}`);
  }

  // ── pairing as Word and as Docs ──
  const pairAs = async (o, label) => {
    const code = await mint(label);
    const r = await raw("POST", `${origin}/api/v1/auth/pair`, { headers: { origin: o, "content-type": "application/json" }, body: { code: code.code, label } });
    return { code, r };
  };
  const word = await pairAs(WORD, "Word on the study machine");
  check("a code minted by the app pairs a Word guest", [200, 201].includes(word.r.status) && word.r.json.kind === "guest" && word.r.json.token.startsWith("ast_"), `${word.r.status} ${word.r.json?.kind}`);
  check("a cross-origin guest gets a bearer token and no cookie", word.r.json.cookie === false && !word.r.headers["set-cookie"]);
  const again = await raw("POST", `${origin}/api/v1/auth/pair`, { headers: { origin: WORD, "content-type": "application/json" }, body: { code: word.code.code, label: "again" } });
  check("a code works once", again.status === 401, String(again.status));
  const docs = await pairAs(DOCS, "Docs sidebar");
  check("the same for a Docs guest", [200, 201].includes(docs.r.status) && docs.r.json.kind === "guest", String(docs.r.status));

  // ── what a guest may and may not do ──
  const guest = { authorization: `Bearer ${word.r.json.token}`, origin: WORD, "content-type": "application/json" };
  const list = await raw("GET", `${origin}/api/v1/projects`, { headers: guest });
  check("a guest lists projects", list.status === 200 && list.json.some((p) => p.id === project.id));
  const filed = await raw("POST", `${origin}/api/v1/projects/${project.id}/import`, { headers: guest, body: { filename: "From Word", format: "plain", body: "Mira had black hair.", kind: "scene" } });
  check("a guest files a selection as a scene", filed.status === 201, String(filed.status));
  const diag = await raw("GET", `${origin}/api/v1/projects/${project.id}/story/diagnostics`, { headers: guest });
  check("a guest reads the Scales", diag.status === 200 && Array.isArray(diag.json.diagnostics));
  const codes = await raw("POST", `${origin}/api/v1/auth/pairing-codes`, { headers: guest, body: { label: "x", grants: "guest" } });
  check("a guest cannot mint a pairing code", codes.status === 403, String(codes.status));
  const tokens = await raw("GET", `${origin}/api/v1/auth/tokens`, { headers: guest });
  check("a guest cannot list tokens", tokens.status === 403, String(tokens.status));
  const ticket = await raw("POST", `${origin}/api/v1/projects/${project.id}/stream-ticket`, { headers: guest });
  check("a guest gets a one-minute stream ticket", [200, 201].includes(ticket.status) && typeof ticket.json.ticket === "string", String(ticket.status));
  const stream = await raw("GET", `${origin}/api/v1/projects/${project.id}/stream?ticket=${ticket.json.ticket}`, { headers: { origin: WORD }, headersOnly: true }).catch(() => ({ status: 0, headers: {} }));
  check("the change stream opens for a guest with the ticket", stream.status === 200 && /text\/event-stream/.test(String(stream.headers["content-type"])), `${stream.status} ${stream.headers["content-type"]}`);
  const stolen = await raw("GET", `${origin}/api/v1/projects`, { headers: { authorization: `Bearer ${word.r.json.token}`, origin: STRANGER } });
  check("the token from a stranger's origin is answered, but without CORS permission", !stolen.headers["access-control-allow-origin"], `${stolen.status}, allow-origin=${stolen.headers["access-control-allow-origin"]}`);
  const out1 = await raw("POST", `${origin}/api/v1/auth/logout`, { headers: guest });
  check("a guest can revoke itself", out1.status === 204 || out1.status === 200, String(out1.status));
  const after = await raw("GET", `${origin}/api/v1/projects`, { headers: guest });
  check("and is refused afterwards", after.status === 401, String(after.status));

  // ── the real task pane, over HTTPS on port 3000, with a stub for Office.js ──
  const certDir = join(profile, "cert");
  mkdirSync(certDir, { recursive: true });
  if (!existsSync(join(certDir, "cert.pem"))) {
    execFileSync("openssl", ["req", "-x509", "-newkey", "rsa:2048", "-nodes", "-keyout", join(certDir, "key.pem"), "-out", join(certDir, "cert.pem"), "-days", "2", "-subj", "/CN=localhost", "-addext", "subjectAltName=DNS:localhost,IP:127.0.0.1"], { stdio: "ignore" });
  }
  const pane = readFileSync("apps/addin-word/taskpane.html");
  tls = createServer({ key: readFileSync(join(certDir, "key.pem")), cert: readFileSync(join(certDir, "cert.pem")) }, (req, res) => {
    if (req.url?.startsWith("/taskpane.html")) {
      res.writeHead(200, { "content-type": "text/html" });
      res.end(pane);
    } else {
      res.writeHead(404);
      res.end();
    }
  });
  const listening = await new Promise((resolve) => {
    tls.on("error", (e) => resolve(e.code));
    tls.listen(3000, "127.0.0.1", () => resolve("ok"));
  });
  check("port 3000 was free for the task pane", listening === "ok", listening);
  if (listening === "ok") {
    const browser = await chromium.launch({ args: ["--ignore-certificate-errors"] });
    const context = await browser.newContext({ ignoreHTTPSErrors: true });
    const tab = await context.newPage();
    const consoleLines = [];
    tab.on("console", (m) => consoleLines.push(`${m.type()}: ${m.text()}`));
    // A stand-in for Office.js: a selection of prose and the ready callback.
    await tab.route("**/office.js", (route) =>
      route.fulfill({
        contentType: "application/javascript",
        body: `window.Office = { onReady(cb){ setTimeout(cb, 0); return Promise.resolve(); }, CoercionType:{Text:"text"}, AsyncResultStatus:{Succeeded:"succeeded"}, context:{document:{ getSelectedDataAsync(kind, cb){ cb({status:"succeeded", value:"Mira shook out her red hair and read the letter."}); } }} };`,
      }),
    );
    await tab.goto(`${WORD}/taskpane.html`);
    await tab.waitForSelector("#pairing:not([hidden])", { timeout: 15000 }).catch(() => undefined);
    const shown = await tab.locator("#pairing").isVisible();
    check("the task pane loads over HTTPS and offers pairing", shown, await tab.locator("#status").innerText());
    const paneCode = await mint("Word task pane");
    await tab.fill("#origin", origin);
    await tab.fill("#code", paneCode.code);
    await tab.click("#pair");
    await tab.waitForSelector("#desk:not([hidden])", { timeout: 15000 }).catch(() => undefined);
    const paired = await tab.locator("#desk").isVisible();
    check("an https page pairs with the http loopback API", paired, (await tab.locator("#status").innerText()) + " | " + consoleLines.slice(-3).join(" | "));
    if (paired) {
      await until(async () => (await tab.locator("#project option").count()) > 0, { label: "projects in the pane" });
      const marksShown = await until(async () => (await tab.locator("#problems li").count()) >= 0 && (await tab.locator("#out").innerText()).length >= 0, { label: "scales" });
      await sleep(1500);
      const live = await tab.locator("#live").innerText();
      log({ live, problems: await tab.locator("#problems li").count() });
      check("the pane follows the change stream (live)", /Live/.test(live), live);
      await tab.click("#file");
      await sleep(1500);
      const scenes = (await raw("GET", `${origin}/api/v1/projects/${project.id}/documents?fields=summary`, { headers: deskHeaders })).json;
      check("filing the selection from the pane creates a scene", scenes.filter((d) => d.title === "From Word").length >= 2, scenes.map((d) => d.title).join(", "));
      await tab.screenshot({ path: join(out, "guests-word-pane.png") });
      void marksShown;
      await tab.click("#forget");
      await sleep(500);
      const tokensAfter = (await raw("GET", `${origin}/api/v1/auth/tokens`, { headers: deskHeaders })).json;
      check("forgetting the pairing revokes the token on the desk", !tokensAfter.some((t) => t.label === "Word on this computer" && !t.revoked_at), JSON.stringify(tokensAfter.map((t) => [t.label, !!t.revoked_at])));
    }
    log({ consoleLines: consoleLines.slice(0, 12) });
    await browser.close();
  }
} finally {
  tls?.close();
  await app.close();
}
const failed = results.filter((r) => !r.ok);
log({ summary: `${results.length - failed.length} of ${results.length} checks passed`, failed: failed.map((f) => `${f.name}: ${f.detail}`) });
log({ notChecked: ["Word itself (no Word host here): the pane runs in Chromium with a stub for Office.js", "Google Docs itself: the sidebar's Origin and the preflight are checked, not Apps Script", "Safari-based hosts (Word for Mac): mixed-content handling of https pages calling http://127.0.0.1 is not tested"] });
process.exit(failed.length ? 1 : 0);
