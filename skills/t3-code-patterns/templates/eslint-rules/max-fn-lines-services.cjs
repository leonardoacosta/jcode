"use strict";

const MAX_LINES = 60;

/**
 * max-fn-lines-services
 *
 * Enforces a 60-LOC cap on function bodies in the service layer
 * (packages/api/src/services/**), excluding __contracts__, __tests__,
 * and __fixtures__ directories.
 *
 * Rationale: Phase A service migration targets small, focused functions.
 * The three god-services (vendor-invoice-explorer 2385L, panels 1224L,
 * badge-addon 1154L) contain functions well over 100 lines — this rule
 * flags them at `warn` level so new code stays focused while cleanup
 * proceeds.
 *
 * Implementation notes:
 * - Measures BODY line range (block-statement loc), not the whole
 *   function node, so long parameter destructuring lists do not
 *   inflate the count.
 * - ArrowFunctionExpression with a bare-expression body (e.g. `(x) => x`)
 *   has no `body.loc`; the early return handles that case.
 * - Anonymous/inline callbacks longer than 60 lines are still reported —
 *   intentional; they should also be split.
 */

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "Service-layer functions must be 60 lines or fewer",
      category: "acme/service-layer",
      recommended: true,
    },
    schema: [],
    messages: {
      tooLong:
        "Service-layer function '{{name}}' is {{lines}} lines (max {{max}}). Split into smaller focused functions.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const filename = context.getFilename();
    if (!/packages\/api\/src\/services\//.test(filename)) return {};
    if (/__(contracts|tests|fixtures)__/.test(filename)) return {};

    /** @param {any} node */
    function check(node) {
      const body = node.body;
      if (!body || !body.loc) return;
      const lines = body.loc.end.line - body.loc.start.line;
      if (lines > MAX_LINES) {
        const name =
          (node.id && node.id.name) ||
          (node.key && node.key.name) ||
          (node.parent &&
            node.parent.type === "VariableDeclarator" &&
            node.parent.id &&
            node.parent.id.name) ||
          (node.parent &&
            node.parent.type === "Property" &&
            node.parent.key &&
            node.parent.key.name) ||
          (node.parent &&
            node.parent.type === "MethodDefinition" &&
            node.parent.key &&
            node.parent.key.name) ||
          "<anonymous>";
        context.report({
          node,
          messageId: "tooLong",
          data: { name, lines, max: MAX_LINES },
        });
      }
    }

    return {
      FunctionDeclaration: check,
      FunctionExpression: check,
      ArrowFunctionExpression: check,
      MethodDefinition: check,
    };
  },
};
