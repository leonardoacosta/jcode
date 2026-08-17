#!/usr/bin/env node
"use strict";

/**
 * Test runner for the 4 new audit-driven ESLint rules.
 *
 * Loads each rule against its __fixtures__/<rule>/{invalid,valid}.ts file
 * via ESLint's Linter API and prints pass/fail per fixture.
 *
 * Resolves eslint + @typescript-eslint/parser via Node's own require.resolve()
 * starting from the current working directory, so it finds whatever T3
 * monorepo you run it from (both are standard T3 Turbo devDependencies) --
 * no hardcoded path to a specific machine's install.
 *
 * Exit code: 0 if every fixture matches its expected outcome; 1 otherwise.
 */

const path = require("path");
const fs = require("fs");

function resolveOrDie(pkg) {
  try {
    return require.resolve(pkg, { paths: [process.cwd(), __dirname] });
  } catch (e) {
    console.error(
      `[test-rules] Could not resolve "${pkg}" from ${process.cwd()} or ${__dirname}.\n` +
        `  Run this from within a T3 Turbo monorepo that has "${pkg}" installed.`,
    );
    process.exit(2);
  }
}

const tsParser = require(resolveOrDie("@typescript-eslint/parser"));
const { Linter } = require(resolveOrDie("eslint"));

// --- Load the rules under test ------------------------------------------
const RULES_DIR = __dirname;
const FIXTURES_DIR = path.join(RULES_DIR, "__fixtures__");

const rules = {
  "no-ctx-db-query": require(path.join(RULES_DIR, "no-ctx-db-query.cjs")),
  "no-double-cast": require(path.join(RULES_DIR, "no-double-cast.cjs")),
  "procedure-name-matches-middleware": require(
    path.join(RULES_DIR, "procedure-name-matches-middleware.cjs"),
  ),
  "no-vi-mock-db": require(path.join(RULES_DIR, "no-vi-mock-db.cjs")),
  "no-process-env": require(path.join(RULES_DIR, "no-process-env.cjs")),
};

// --- Expected fixture outcomes ------------------------------------------
// Each entry: { rule, file, expectErrorCount (>=N if N, ==0 if 0) }
const cases = [
  {
    rule: "no-ctx-db-query",
    file: "no-ctx-db-query/invalid.ts",
    expectAtLeast: 3,
  },
  { rule: "no-ctx-db-query", file: "no-ctx-db-query/valid.ts", expect: 0 },

  { rule: "no-double-cast", file: "no-double-cast/invalid.ts", expectAtLeast: 3 },
  { rule: "no-double-cast", file: "no-double-cast/valid.ts", expect: 0 },

  {
    rule: "procedure-name-matches-middleware",
    file: "procedure-name-matches-middleware/invalid.ts",
    expectAtLeast: 3,
  },
  {
    rule: "procedure-name-matches-middleware",
    file: "procedure-name-matches-middleware/valid.ts",
    expect: 0,
  },

  { rule: "no-vi-mock-db", file: "no-vi-mock-db/invalid.test.ts", expectAtLeast: 4 },
  { rule: "no-vi-mock-db", file: "no-vi-mock-db/valid.test.ts", expect: 0 },

  { rule: "no-process-env", file: "no-process-env/invalid.ts", expectAtLeast: 4 },
  { rule: "no-process-env", file: "no-process-env/valid.ts", expect: 0 },
  // allowlist option: POSTGRES_URL + NODE_ENV suppressed, STRIPE + computed remain.
  {
    rule: "no-process-env",
    file: "no-process-env/invalid.ts",
    options: { allow: ["POSTGRES_URL", "NODE_ENV"] },
    expect: 2,
  },
  // env.ts is the sanctioned boundary — process.env reads there are exempt.
  { rule: "no-process-env", file: "no-process-env/env.ts", expect: 0 },
];

// --- Run -----------------------------------------------------------------
const linter = new Linter();

function lintOnce({ rule, file, options }) {
  const ruleId = `local/${rule}`;
  const absPath = path.join(FIXTURES_DIR, file);
  const source = fs.readFileSync(absPath, "utf8");

  // Register the rule and the parser inline for this run.
  const config = {
    files: ["**/*.{ts,tsx,test.ts}"],
    languageOptions: {
      parser: tsParser,
      parserOptions: {
        ecmaVersion: 2022,
        sourceType: "module",
      },
    },
    plugins: {
      local: { rules: { [rule]: rules[rule] } },
    },
    rules: {
      [ruleId]: options ? ["error", options] : "error",
    },
  };

  return linter.verify(source, config, { filename: absPath });
}

let pass = 0;
let fail = 0;

for (const c of cases) {
  let messages;
  try {
    messages = lintOnce(c);
  } catch (e) {
    console.log(`FAIL  ${c.rule} :: ${c.file} — exception: ${e.message}`);
    fail++;
    continue;
  }
  const ruleMessages = messages.filter(
    (m) => m.ruleId === `local/${c.rule}` || m.ruleId === c.rule,
  );
  const count = ruleMessages.length;

  let ok;
  if (typeof c.expect === "number") {
    ok = count === c.expect;
  } else if (typeof c.expectAtLeast === "number") {
    ok = count >= c.expectAtLeast;
  } else {
    ok = true;
  }

  const expectStr =
    typeof c.expect === "number"
      ? `exactly ${c.expect}`
      : `>= ${c.expectAtLeast}`;
  if (ok) {
    pass++;
    console.log(`PASS  ${c.rule} :: ${c.file}  (${count} reports, expect ${expectStr})`);
  } else {
    fail++;
    console.log(`FAIL  ${c.rule} :: ${c.file}  (${count} reports, expect ${expectStr})`);
    for (const m of ruleMessages) {
      console.log(`        @${m.line}:${m.column} ${m.message}`);
    }
    // Also dump any parser/other errors that crept in.
    const otherErrors = messages.filter((m) => m.fatal);
    for (const m of otherErrors) {
      console.log(`        fatal @${m.line}:${m.column} ${m.message}`);
    }
  }
}

console.log("");
console.log(`Summary: ${pass} passed, ${fail} failed (of ${cases.length} fixtures)`);
process.exit(fail === 0 ? 0 : 1);
