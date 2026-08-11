#!/usr/bin/env node
import { spawnSync } from "node:child_process";
import { createRequire } from "node:module";

const require = createRequire(import.meta.url);
const cli = require.resolve("@playwright/test/cli");
const args = process.argv.slice(2);
const forwarded = args[0] === "--" ? args.slice(1) : args;
const result = spawnSync(process.execPath, [cli, "test", ...forwarded], { stdio: "inherit" });
process.exit(result.status ?? 1);
