"use strict";

/**
 * Rule: acme/no-ctx-db-query
 *
 * Flags any `MemberExpression` chain matching `ctx.db.query.*`. In T3 Turbo
 * monorepos this pattern causes TypeScript recursion / serialization errors
 * (the inferred type graph closes over the full router on every query). The
 * canonical replacement is importing `db` directly:
 *
 *   // ❌ ctx.db.query causes TS recursion
 *   const rows = await ctx.db.query.safetyReports.findMany({ ... });
 *
 *   // ✅ import db directly
 *   import { db } from "@{ws}/db/client";
 *   const rows = await db.query.safetyReports.findMany({ ... });
 *
 * Reference violation site: acme `packages/api/src/router/admin/safety-reports/triage.ts:59`
 * (the 2026-05-17 fleet audit flagged a cluster of these). See also
 * `t3-code-patterns` skill § Database / Import Pattern and § TypeScript
 * Serialization Limits.
 *
 * Detection: a `MemberExpression` whose `object` is itself a `MemberExpression`
 * `ctx.db` and whose property is `query`. We do NOT require the trailing
 * `.findMany`/`.findFirst` call — the `.query` access alone is the smell.
 *
 * Scope: every file (not service-only). The recursion error surfaces in any
 * package that imports the router types, so router/lib/service code all
 * matter. Escape hatch: standard
 * `// eslint-disable-next-line acme/no-ctx-db-query` per-site suppression with
 * a documented reason.
 */
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow ctx.db.query.* — causes TypeScript recursion; import db directly from @{ws}/db/client",
      category: "acme/typescript",
      recommended: true,
    },
    schema: [],
    messages: {
      noCtxDbQuery:
        "`ctx.db.query` causes TypeScript recursion. Import `db` directly from `@{ws}/db/client` instead.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    return {
      /** @param {any} node */
      MemberExpression(node) {
        // We want the outer access: <something>.query where <something> is ctx.db
        if (
          !node.property ||
          node.property.type !== "Identifier" ||
          node.property.name !== "query"
        ) {
          return;
        }
        const obj = node.object;
        if (
          !obj ||
          obj.type !== "MemberExpression" ||
          !obj.property ||
          obj.property.type !== "Identifier" ||
          obj.property.name !== "db"
        ) {
          return;
        }
        const root = obj.object;
        if (!root || root.type !== "Identifier" || root.name !== "ctx") {
          return;
        }
        context.report({ node, messageId: "noCtxDbQuery" });
      },
    };
  },
};
