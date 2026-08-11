#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const cli = require.resolve("@playwright/test/cli");
const args = process.argv.slice(2);
const forwarded = args[0] === "--" ? args.slice(1) : args;
const env = { ...process.env };
if (!env.JCODE_COMMAND_CENTER_BASE_URL && !env.JCODE_COMMAND_CENTER_FIXTURE_MODE) {
  env.JCODE_COMMAND_CENTER_FIXTURE_MODE = "1";
}
const result = spawnSync(process.execPath, [cli, "test", ...forwarded], { stdio: "inherit", env });
process.exit(result.status ?? 1);
