"use strict";

/**
 * Rule: acme/require-output-schema
 *
 * Flags any tRPC procedure (chain on `publicProcedure`,
 * `protectedProcedure`, `tenantProtectedProcedure`, `protectedUserProcedure`,
 * `tenantSeriesProtectedProcedure`, `eventAwareProcedure`) that ends in
 * `.query(...)`, `.mutation(...)`, or `.subscription(...)` without an
 * `.output(...)` call earlier in the chain.
 *
 * Spec: openspec/changes/backfill-trpc-output-schemas §5.1.
 *
 * Severity: warn (per §5.2). Documented exemption:
 *   // eslint-disable-next-line acme/require-output-schema -- <slug>: <reason>
 *
 * Detection strategy:
 *   - Walk every CallExpression whose callee is a MemberExpression where
 *     the property is "query", "mutation", or "subscription".
 *   - Traverse the chain root-ward; if we ever see `.output(...)`, pass.
 *   - When we hit the root, check whether the root identifier is one of
 *     the recognized procedure builders. If yes and no `.output(...)` was
 *     seen, report.
 */

const PROCEDURE_BUILDERS = new Set([
  "publicProcedure",
  "protectedProcedure",
  "tenantProtectedProcedure",
  "protectedUserProcedure",
  "tenantSeriesProtectedProcedure",
  "eventAwareProcedure",
]);

const TERMINAL_METHODS = new Set(["query", "mutation", "subscription"]);

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Require .output(<zod schema>) on every tRPC procedure to enable runtime payload parsing",
      category: "acme/trpc",
      recommended: true,
    },
    schema: [],
    messages: {
      missingOutput:
        "tRPC procedure '{{builder}}' is missing .output(<schema>). Use .output(output(<schema>)) to enable runtime parsing — see openspec/changes/backfill-trpc-output-schemas.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    return {
      /** @param {any} node */
      CallExpression(node) {
        const callee = node.callee;
        if (
          !callee ||
          callee.type !== "MemberExpression" ||
          !callee.property ||
          callee.property.type !== "Identifier" ||
          !TERMINAL_METHODS.has(callee.property.name)
        ) {
          return;
        }
        // Walk the chain root-ward
        let cursor = callee.object;
        let hasOutput = false;
        while (
          cursor &&
          cursor.type === "CallExpression" &&
          cursor.callee &&
          cursor.callee.type === "MemberExpression"
        ) {
          const propName =
            cursor.callee.property && cursor.callee.property.type === "Identifier"
              ? cursor.callee.property.name
              : null;
          if (propName === "output") {
            hasOutput = true;
            break;
          }
          cursor = cursor.callee.object;
        }
        // Root cursor should be an Identifier matching a procedure builder
        if (
          !cursor ||
          cursor.type !== "Identifier" ||
          !PROCEDURE_BUILDERS.has(cursor.name)
        ) {
          return;
        }
        if (!hasOutput) {
          context.report({
            node,
            messageId: "missingOutput",
            data: { builder: cursor.name },
          });
        }
      },
    };
  },
};
