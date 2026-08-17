"use strict";

/**
 * Rule: local/no-process-env
 *
 * Flags direct `process.env.X` member access outside the sanctioned env boundary.
 * In T3 monorepos every direct read bypasses the build-enforced `@t3-oss/env`
 * createEnv schema — an unvalidated, untyped access that fails at runtime instead
 * of build time (the ~780 bypassed reads found in the 2026-07-06 fleet survey are
 * exactly this failure class). The canonical fix is to add the var to the owning
 * `env.ts` schema and read it via the validated `env` object:
 *
 *   // ❌ unvalidated, untyped, runtime-only
 *   const url = process.env.POSTGRES_URL;
 *
 *   // ✅ validated at build time via createEnv
 *   import { env } from "~/env";
 *   const url = env.POSTGRES_URL;
 *
 * Exempt files (where the boundary legitimately touches process.env):
 *   - `env.ts` / `env.mjs` / files under an `env/` dir (the schema definitions)
 *   - `next.config.*` (build-time config, evaluated before the schema exists)
 *
 * Per-repo allowlist: pass an options object `{ allow: ["NODE_ENV", "CI"] }` for
 * vars legitimately read raw (build-time-only, NODE_ENV guards). Document each
 * entry in the repo's env allowlist. See `t3-code-patterns` skill § Env Validation.
 *
 * Detection: a `MemberExpression` whose `object` is the `process.env`
 * `MemberExpression`. Both dotted (`process.env.X`) and computed
 * (`process.env[expr]`) accesses are flagged; a computed non-literal reports as
 * `<computed>`.
 */
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow direct process.env.X reads outside env.ts / next.config — use the validated env object from @t3-oss/env",
      category: "env",
      recommended: true,
    },
    schema: [
      {
        type: "object",
        properties: {
          allow: { type: "array", items: { type: "string" } },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      noProcessEnv:
        "Direct `process.env.{{name}}` bypasses the validated env schema. Add it to env.ts and read via the `env` object (or allowlist it if a legitimate raw read).",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const filename = (
      context.filename ||
      (context.getFilename && context.getFilename()) ||
      ""
    ).replace(/\\/g, "/");

    // Exempt the sanctioned boundary files: env schema definitions + next.config.
    if (
      /(^|\/)env\.(ts|mts|cts|js|mjs|cjs)$/.test(filename) ||
      /(^|\/)env\/[^/]+\.(ts|mts|cts|js|mjs|cjs)$/.test(filename) ||
      /(^|\/)next\.config\.[a-z]+$/.test(filename)
    ) {
      return {};
    }

    const opts = context.options[0] || {};
    const allow = new Set(Array.isArray(opts.allow) ? opts.allow : []);

    return {
      /** @param {any} node */
      MemberExpression(node) {
        // Match the OUTER access: <process.env>.<NAME>
        const obj = node.object;
        if (
          !obj ||
          obj.type !== "MemberExpression" ||
          !obj.object ||
          obj.object.type !== "Identifier" ||
          obj.object.name !== "process" ||
          !obj.property ||
          obj.property.type !== "Identifier" ||
          obj.property.name !== "env"
        ) {
          return;
        }

        // Resolve the accessed var name for the message + allowlist check.
        let name = "<computed>";
        if (node.property) {
          if (node.property.type === "Identifier" && !node.computed) {
            name = node.property.name;
          } else if (
            node.property.type === "Literal" &&
            typeof node.property.value === "string"
          ) {
            name = node.property.value;
          }
        }
        if (allow.has(name)) return;
        context.report({ node, messageId: "noProcessEnv", data: { name } });
      },
    };
  },
};
