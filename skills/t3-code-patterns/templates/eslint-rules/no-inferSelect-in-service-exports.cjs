"use strict";

/**
 * no-inferSelect-in-service-exports
 *
 * Flags usage of `typeof <table>.$inferSelect` (and `<table>.$inferSelect`) in
 * service-layer files under `packages/api/src/services/`. Services should emit
 * explicit DTOs rather than leaking raw Drizzle row shapes (including internal
 * columns like `deletedAt`) through RouterOutputs to clients.
 *
 * Category: acme/service-layer
 * Level:    warn (non-blocking — spec: phase-a-service-contract-foundation task 2.10)
 *
 * Scope:
 *   - INCLUDES: packages/api/src/services/**\/*.ts
 *   - EXCLUDES: __tests__, __contracts__, __fixtures__ directories
 */

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow typeof table.$inferSelect in service-layer exports. Define explicit DTOs to prevent leaking internal columns (e.g. deletedAt) to clients.",
      category: "acme/service-layer",
      recommended: true,
    },
    schema: [],
    messages: {
      noInferSelect:
        "Avoid typeof <table>.$inferSelect in service-layer exports. Define an explicit DTO — prevents leaking internal columns like deletedAt to clients.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const filename =
      typeof context.getFilename === "function"
        ? context.getFilename()
        : context.filename || "";

    // Normalize path separators so Windows paths match too.
    const normalized = filename.replace(/\\/g, "/");

    // Only apply to service-layer files.
    if (!/packages\/api\/src\/services\//.test(normalized)) return {};

    // Skip tests, contracts, and fixtures subdirectories.
    if (/__(contracts|tests|fixtures)__/.test(normalized)) return {};

    /** @param {any} node */
    function reportIfInferSelect(node) {
      // Type position: `typeof tableName.$inferSelect`
      // AST: TSTypeQuery -> exprName: TSQualifiedName { right: { name: "$inferSelect" } }
      if (
        node.type === "TSTypeQuery" &&
        node.exprName &&
        node.exprName.type === "TSQualifiedName" &&
        node.exprName.right &&
        node.exprName.right.name === "$inferSelect"
      ) {
        context.report({ node, messageId: "noInferSelect" });
        return;
      }

      // Value position: `tableName.$inferSelect` as a MemberExpression
      // (rare in type-only code, but covers `typeof table["$inferSelect"]` style
      // and other edge cases)
      if (
        node.type === "MemberExpression" &&
        node.property &&
        ((node.property.type === "Identifier" &&
          node.property.name === "$inferSelect") ||
          (node.property.type === "Literal" &&
            node.property.value === "$inferSelect"))
      ) {
        context.report({ node, messageId: "noInferSelect" });
      }
    }

    return {
      TSTypeQuery: reportIfInferSelect,
      MemberExpression: reportIfInferSelect,
    };
  },
};
