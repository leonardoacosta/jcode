"use strict";

/**
 * Rule: acme/no-double-cast
 *
 * Flags `x as unknown as Y` (and any other `TSAsExpression > TSAsExpression`
 * chain). The double cast is type tunneling — it forces the compiler to
 * accept a conversion it would otherwise reject, hiding a real type-system
 * disagreement at a package boundary (DB row -> DTO, third-party SDK type ->
 * domain type, etc.). The fix is to define a proper union/intersection or
 * fix the schema, not to launder the cast.
 *
 * The 2026-05-17 fleet audit caught 16 occurrences of this pattern across
 * the T3 fleet, all clustered at package boundaries where someone wanted
 * to "make the type checker shut up" rather than reconcile the types.
 *
 * Scope: every file EXCEPT `<any>/__tests__/<any>`. Mocks and test doubles
 * legitimately need to cast partial fixtures to full types — the production
 * code path is where the smell matters.
 *
 * Detection: `TSAsExpression` whose `expression` is itself a `TSAsExpression`.
 * Both `(x as A) as B` and `x as A as B` produce the same nested shape.
 *
 * Escape hatch: standard `// eslint-disable-next-line acme/no-double-cast`
 * per-site suppression with a documented reason. Use rarely — the lint
 * exists precisely because the pattern is almost never justified outside
 * tests.
 */
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow double type assertions (`x as unknown as Y`) outside test files",
      category: "acme/typescript",
      recommended: true,
    },
    schema: [],
    messages: {
      noDoubleCast:
        "`as unknown as T` tunnels through the type system at a boundary. Define a proper union type or fix the schema mismatch instead.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const filename =
      typeof context.getFilename === "function"
        ? context.getFilename()
        : context.filename || "";
    const normalized = filename.replace(/\\/g, "/");

    // Tests legitimately cast partial fixtures; do not flag there.
    if (/\/__tests__\//.test(normalized)) return {};

    return {
      /** @param {any} node */
      TSAsExpression(node) {
        if (
          node.expression &&
          node.expression.type === "TSAsExpression"
        ) {
          context.report({ node, messageId: "noDoubleCast" });
        }
      },
    };
  },
};
