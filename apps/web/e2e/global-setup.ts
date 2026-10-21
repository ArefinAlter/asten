import { spawn, spawnSync, type ChildProcess } from "node:child_process";
import { existsSync, mkdtempSync, rmSync } from "node:fs";
import { tmpdir } from "node:os";
import path from "node:path";
import { fileURLToPath } from "node:url";
import { E2E_ORIGIN, E2E_PORT } from "./desk";

const here = path.dirname(fileURLToPath(import.meta.url));
const webRoot = path.resolve(here, "..");
const repoRoot = path.resolve(webRoot, "../..");
const dist = path.join(webRoot, "dist");
const targetDir = process.env.CARGO_TARGET_DIR ?? path.join(repoRoot, "target");
const binary = path.join(
  targetDir,
  "debug",
  process.platform === "win32" ? "asten-api.exe" : "asten-api",
);

const START_TIMEOUT_MS = 30_000;
const CODE_LINE = /Asten pairing code: (\d{6})/;

/** Six digits in a row are a pairing code until proven otherwise. */
function redact(text: string): string {
  return text.replace(/\b\d{6}\b/g, "······");
}

function run(command: string, args: string[], env?: NodeJS.ProcessEnv) {
  const result = spawnSync(command, args, {
    cwd: repoRoot,
    stdio: "inherit",
    shell: process.platform === "win32",
    env: { ...process.env, ...env },
  });
  if (result.status !== 0) {
    throw new Error(`${command} ${args.join(" ")} failed (${result.status})`);
  }
}

async function healthy(): Promise<boolean> {
  try {
    const response = await fetch(`${E2E_ORIGIN}/health`);
    return response.ok;
  } catch {
    return false;
  }
}

const sleep = (ms: number) => new Promise((resolve) => setTimeout(resolve, ms));

export default async function globalSetup(): Promise<() => Promise<void>> {
  if (!existsSync(path.join(dist, "index.html"))) {
    console.log("e2e: building the desk");
    run("pnpm", ["--filter", "@asten/web", "build"]);
  }
  if (!existsSync(binary)) {
    console.log("e2e: building asten-api");
    run("cargo", ["build", "-p", "asten-api"], { CARGO_INCREMENTAL: "0" });
  }
  if (await healthy()) {
    throw new Error(
      `Something already answers on ${E2E_ORIGIN}. Stop it before running the e2e suite.`,
    );
  }

  const dataDir = mkdtempSync(path.join(tmpdir(), "asten-e2e-"));
  const child: ChildProcess = spawn(binary, [], {
    cwd: repoRoot,
    env: {
      ...process.env,
      ASTEN_DATABASE_PATH: path.join(dataDir, "asten.db"),
      ASTEN_AUTH: "pairing",
      ASTEN_BIND: `127.0.0.1:${E2E_PORT}`,
      ASTEN_SERVE_DESK: dist,
    },
    stdio: ["ignore", "pipe", "pipe"],
    windowsHide: true,
  });

  // The API prints its bootstrap code to stderr when no client is paired.
  // Read it, use it, and keep it out of every log line.
  let stderr = "";
  let code: string | undefined;
  let exited: number | null | undefined;
  child.stderr?.setEncoding("utf8");
  child.stderr?.on("data", (chunk: string) => {
    stderr = (stderr + chunk).slice(-65_536);
    code ??= CODE_LINE.exec(stderr)?.[1];
  });
  child.stdout?.resume();
  child.on("exit", (status) => {
    exited = status;
  });

  const stop = async () => {
    if (exited === undefined) {
      const gone = new Promise<void>((resolve) =>
        child.once("exit", () => resolve()),
      );
      child.kill();
      await Promise.race([gone, sleep(5_000)]);
    }
    for (let attempt = 0; attempt < 5; attempt += 1) {
      try {
        rmSync(dataDir, { recursive: true, force: true });
        break;
      } catch {
        await sleep(200);
      }
    }
  };

  const deadline = Date.now() + START_TIMEOUT_MS;
  while (!(code && (await healthy()))) {
    if (exited !== undefined) {
      await stop();
      throw new Error(
        `asten-api exited with ${exited} before it was ready:\n${redact(stderr)}`,
      );
    }
    if (Date.now() > deadline) {
      await stop();
      throw new Error(
        `asten-api did not come up on ${E2E_ORIGIN} within ${START_TIMEOUT_MS} ms:\n${redact(stderr)}`,
      );
    }
    await sleep(100);
  }

  const response = await fetch(`${E2E_ORIGIN}/api/v1/auth/pair`, {
    method: "POST",
    headers: { "content-type": "application/json", accept: "application/json" },
    body: JSON.stringify({ code, label: "e2e setup" }),
  });
  if (!response.ok) {
    await stop();
    throw new Error(
      `Pairing the e2e setup with the API failed (${response.status}).`,
    );
  }
  const paired = (await response.json()) as { token: string };
  process.env.ASTEN_E2E_TOKEN = paired.token;
  process.env.ASTEN_E2E_ORIGIN = E2E_ORIGIN;
  code = undefined;

  return stop;
}
