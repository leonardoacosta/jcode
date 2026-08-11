import { defineConfig } from "@playwright/test";

const suppliedBaseURL = process.env.JCODE_COMMAND_CENTER_BASE_URL;
const baseURL = suppliedBaseURL ?? "http://127.0.0.1:3000";
const fixtureMode = process.env.JCODE_COMMAND_CENTER_FIXTURE_MODE === "1";

export default defineConfig({
  testDir: "./e2e",
  grepInvert: fixtureMode ? undefined : /@fixture-only/,
  webServer: suppliedBaseURL
    ? undefined
    : {
        command: "pnpm dev",
        url: `${baseURL}/initiatives`,
        reuseExistingServer: !process.env.CI,
        timeout: 120_000,
      },
  use: { baseURL, trace: "on-first-retry" },
  projects: [
    { name: "repository-local", metadata: { fixtureMode, orcaUnavailable: false } },
    { name: "orca-unavailable", metadata: { fixtureMode, orcaUnavailable: true } },
  ],
});
