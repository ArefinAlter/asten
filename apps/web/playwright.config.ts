import { defineConfig } from "@playwright/test";
import { E2E_ORIGIN } from "./e2e/desk";

/**
 * End-to-end tests drive the built desk served by a real `asten-api` on
 * 127.0.0.1:4391 with a throwaway database. The global setup refreshes the builds,
 * starts the API, pairs with it once, and tears it all down.
 *
 * Traces and screenshots are off by default: both would capture the pairing
 * code and the bearer token. Opt in for one run with `--trace on`.
 */
export default defineConfig({
  testDir: "./e2e",
  testMatch: /.*\.e2e\.ts$/,
  globalSetup: "./e2e/global-setup.ts",
  // One API, one database: the tests run one at a time and never share state
  // beyond the token the setup minted.
  fullyParallel: false,
  workers: 1,
  retries: process.env.CI ? 1 : 0,
  reporter: [["list"]],
  timeout: 60_000,
  expect: { timeout: 20_000 },
  outputDir: "./test-results",
  use: {
    baseURL: E2E_ORIGIN,
    browserName: "chromium",
    viewport: { width: 1366, height: 900 },
    serviceWorkers: "block",
    trace: "off",
    screenshot: "off",
    video: "off",
  },
});
