/**
 * REFERENCE TEMPLATE — t3-testing-patterns
 *
 * The validated wrapper described in references/e2e-command-target-lifecycle.md is authoritative.
 * It must create a short-lived private run-capability file after trusted production-denial checks.
 * This config refuses a naked URL and validates that capability again as defense in depth.
 */
import { timingSafeEqual } from "node:crypto";
import { closeSync, constants, fstatSync, openSync, readFileSync } from "node:fs";
import { defineConfig, devices } from "@playwright/test";

const isTrue = (value: string | undefined) => /^(?:1|true)$/i.test(value?.trim() ?? "");
const isCI = isTrue(process.env.CI);
const rawBaseUrl = process.env.E2E_BASE_URL;
const capabilityPath = process.env.E2E_RUN_CAPABILITY_FILE;
const capabilityToken = process.env.E2E_RUN_CAPABILITY_TOKEN;
const requestedMode = process.env.E2E_RUN_MODE;
const requestedOperation = process.env.E2E_RUN_OPERATION;

if (!rawBaseUrl || !capabilityPath || !capabilityToken || !requestedMode || !requestedOperation) {
  throw new Error(
    "E2E_BASE_URL, E2E_RUN_MODE, E2E_RUN_OPERATION, E2E_RUN_CAPABILITY_FILE, and E2E_RUN_CAPABILITY_TOKEN are required from the validated runner; a naked URL is not accepted.",
  );
}

let baseUrl: URL;
try {
  baseUrl = new URL(rawBaseUrl);
} catch {
  throw new Error("E2E_BASE_URL must be a valid HTTP(S) URL.");
}
if (baseUrl.protocol !== "http:" && baseUrl.protocol !== "https:") {
  throw new Error("E2E_BASE_URL must use HTTP(S).");
}
if (baseUrl.username || baseUrl.password || baseUrl.search || baseUrl.hash) {
  throw new Error("E2E_BASE_URL must not contain credentials, query parameters, or fragments.");
}

const host = baseUrl.hostname.replace(/^\[|\]$/g, "").replace(/\.$/, "").toLowerCase();
const isIpv4Loopback = /^127(?:\.\d{1,3}){3}$/.test(host);
const isMappedDottedLoopback = /^::ffff:127(?:\.\d{1,3}){3}$/.test(host);
const isMappedHexLoopback = /^::ffff:7f[0-9a-f]{2}:[0-9a-f]{1,4}$/.test(host);
const isLoopback =
  host === "localhost" ||
  host === "::1" ||
  isIpv4Loopback ||
  isMappedDottedLoopback ||
  isMappedHexLoopback;
const targetClass = isLoopback ? "loopback" : "deployed";

let capabilityFd: number;
try {
  capabilityFd = openSync(capabilityPath, constants.O_RDONLY | constants.O_NOFOLLOW);
} catch {
  throw new Error("E2E run capability must be a readable regular non-symlink file.");
}
const stat = fstatSync(capabilityFd);
if (!stat.isFile()) {
  closeSync(capabilityFd);
  throw new Error("E2E run capability must be a regular file.");
}
if (typeof process.getuid !== "function" || stat.uid !== process.getuid() || (stat.mode & 0o077) !== 0) {
  closeSync(capabilityFd);
  throw new Error("E2E run capability must be owned by the current user with no group/other permissions.");
}

interface RunCapability {
  version: 1;
  token: string;
  mode: "canonical" | "local" | "ci" | "collection";
  operation: "execute" | "list";
  baseUrl: string;
  targetClass: "loopback" | "deployed";
  deploymentIdentity: string;
  runIdentity: string;
  databaseIdentity: string;
  serviceIdentity: string;
  nonProduction: true;
  issuedAt: string;
  expiresAt: string;
  browserCapabilities?: {
    webkit?: { supported: boolean; omissionEvidenceId?: string };
  };
}

let capability: RunCapability;
try {
  capability = JSON.parse(readFileSync(capabilityFd, "utf8")) as RunCapability;
} catch {
  throw new Error("E2E run capability must contain valid JSON.");
} finally {
  closeSync(capabilityFd);
}

const decodeToken = (value: string) => {
  if (!/^[A-Za-z0-9_-]{43}$/.test(value)) return Buffer.alloc(0);
  return Buffer.from(value, "base64url");
};
const suppliedToken = decodeToken(capabilityToken);
const recordedToken = decodeToken(capability.token);
if (
  suppliedToken.length !== 32 ||
  recordedToken.length !== 32 ||
  !timingSafeEqual(suppliedToken, recordedToken)
) {
  throw new Error("E2E run capability token is invalid.");
}

const allowedModes = new Set(["canonical", "local", "ci", "collection"]);
const issuedAt = Date.parse(capability.issuedAt);
const expiresAt = Date.parse(capability.expiresAt);
const now = Date.now();
const identities = [
  capability.deploymentIdentity,
  capability.runIdentity,
  capability.databaseIdentity,
  capability.serviceIdentity,
];
if (
  capability.version !== 1 ||
  !allowedModes.has(capability.mode) ||
  capability.mode !== requestedMode ||
  capability.operation !== requestedOperation ||
  !new Set(["execute", "list"]).has(capability.operation) ||
  capability.baseUrl !== baseUrl.toString() ||
  capability.targetClass !== targetClass ||
  capability.nonProduction !== true ||
  identities.some((value) => typeof value !== "string" || value.trim().length < 8) ||
  !Number.isFinite(issuedAt) ||
  !Number.isFinite(expiresAt) ||
  issuedAt > now + 30_000 ||
  expiresAt <= now ||
  expiresAt - issuedAt > 60 * 60 * 1000
) {
  throw new Error(
    "E2E run capability is expired or does not bind the requested mode, operation, URL, target, non-production deployment, run, database, and service identities.",
  );
}
const isListInvocation = process.argv.includes("--list");
if (targetClass === "deployed" && baseUrl.protocol !== "https:") {
  throw new Error("Every non-loopback E2E target must use HTTPS.");
}
if (
  (capability.mode === "canonical" && (targetClass !== "deployed" || baseUrl.protocol !== "https:")) ||
  (capability.mode === "local" && targetClass !== "loopback") ||
  (capability.mode === "ci" && (targetClass !== "loopback" || !isCI)) ||
  (isCI && capability.mode !== "ci" && capability.mode !== "collection")
) {
  throw new Error("E2E mode/target matrix rejected this canonical, local, or CI invocation.");
}
if (
  (capability.mode === "collection" && (capability.operation !== "list" || !isListInvocation)) ||
  (capability.mode !== "collection" && capability.operation !== "execute")
) {
  throw new Error("Collection mode requires an explicit --list no-test-body operation; execution modes cannot use collection operation authority.");
}

const webkit = capability.browserCapabilities?.webkit;
const webkitUnsupported = webkit?.supported === false;
if (webkitUnsupported && !webkit.omissionEvidenceId?.trim()) {
  throw new Error("Validated WebKit omission requires a retained omission evidence ID.");
}

// Diagnostics use stderr so stdout remains machine-readable. Identity values and capability data
// stay private; bypass credentials belong in protected headers/cookies, never the base URL.
console.error(`[e2e] targetClass=${targetClass} mode=${capability.mode} operation=${capability.operation} protocol=${baseUrl.protocol}`);
if (webkitUnsupported) {
  console.error("[e2e] projectOmitted=critical-webkit capability=validated-unsupported evidence=recorded");
}

const setupProject = {
  name: "setup",
  testMatch: /.*\.setup\.ts/,
  fullyParallel: false,
  retries: 0,
  workers: 1,
};
const chromiumProject = {
  // Exact, unfiltered completeness project. Do not add grep/grepInvert/path filtering.
  name: "chromium",
  use: { ...devices["Desktop Chrome"] },
};
const executionProjects = [
  setupProject,
  { ...chromiumProject, dependencies: ["setup"] },
  {
    name: "critical-firefox",
    grep: /@critical/,
    grepInvert: /@quarantine/,
    use: { ...devices["Desktop Firefox"] },
    dependencies: ["setup"],
  },
  ...(!webkitUnsupported
    ? [
        {
          name: "critical-webkit",
          grep: /@critical/,
          grepInvert: /@quarantine/,
          use: { ...devices["Desktop Safari"] },
          dependencies: ["setup"],
        },
      ]
    : []),
  {
    name: "critical-mobile",
    grep: /@critical/,
    grepInvert: /@quarantine/,
    use: { ...devices["Pixel 5"] },
    dependencies: ["setup"],
  },
];

export default defineConfig({
  testDir: "./tests",
  fullyParallel: false,
  forbidOnly: isCI,
  retries: isCI ? 1 : 0,
  workers: 1,
  reporter: isCI
    ? [
        ["blob", { outputDir: "test-results/blob-report" }],
        ["junit", { outputFile: "test-results/junit.xml" }],
        ["json", { outputFile: "test-results/results.json" }],
        ["github"],
        ["html", { outputFolder: "test-results/html-report", open: "never" }],
      ]
    : [["list"], ["html", { outputFolder: "test-results/html-report", open: "never" }]],

  use: {
    baseURL: baseUrl.toString(),
    trace: "on-first-retry",
    screenshot: "only-on-failure",
    video: "retain-on-failure",
  },

  // Collection mode exposes only exact Chromium, with no setup dependency graph or webServer.
  projects: capability.mode === "collection" ? [chromiumProject] : executionProjects,
});

/**
 * Native shard examples (after serial setup once, shards run with --no-deps):
 *   pnpm test:e2e -- --project=chromium --shard=1/4 --no-deps
 *   pnpm test:e2e -- --project=chromium --shard=2/4 --no-deps
 *
 * CI inventories expected/validated-omitted projects, classifies retry passes, scans artifacts,
 * and merges safe reports under always(). Storage-state and capability files are never uploaded.
 */
