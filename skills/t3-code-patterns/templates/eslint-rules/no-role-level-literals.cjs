"use strict";

/**
 * Rule: acme/no-role-level-literals
 *
 * Bans bare string-literal comparisons against `roleLevel` fields. The
 * canonical role-level vocabulary lives in
 * `packages/db/src/schemas/admin/role-vocabulary.ts` (re-exported from
 * `@acme/auth` as `ROLE_LEVEL.<NAME>`); using the named constants makes a
 * future rename surface as a missing-export error rather than a silently
 * stale string compare.
 *
 * Spec: openspec/changes/unify-rbac-and-add-departmental-scoping (API.10
 * / acme-i13xcs).
 *
 * Severity: warn (non-blocking). The first deployment cycle uses this as
 * a signal so existing call sites surface for migration; can elevate to
 * error in a follow-up once the catalog is clean.
 *
 * Detection patterns flagged:
 *   1. `<expr>.roleLevel === "<literal>"` (any direction)
 *   2. `["literal", "literal"].includes(<expr>.roleLevel)` — common
 *      `STAFF_TIERS`-style aggregates that should import the
 *      `STAFF_TIER_ROLE_LEVELS` constant.
 *
 * Patterns intentionally NOT flagged:
 *   - Comparisons against constants (`role.roleLevel === ROLE_LEVEL.STAFF`)
 *     resolve to MemberExpression, not Literal.
 *   - Switch cases on roleLevel — out of scope for v1; can extend if a
 *     real-world site arises.
 *
 * Escape hatch: standard `// eslint-disable-next-line acme/no-role-level-literals`
 * per-site suppression. Use sparingly and document the reason in the
 * disable comment.
 */
module.exports = {
  meta: {
    type: "suggestion",
    docs: {
      description: "Disallow bare string literals compared against roleLevel",
      category: "acme/rbac",
      recommended: true,
    },
    schema: [],
    messages: {
      noLiteralEquality:
        "Compare `roleLevel` against a named constant from `@acme/auth` (e.g. `ROLE_LEVEL.{{name}}`) instead of a bare string literal. Spec: unify-rbac-and-add-departmental-scoping.",
      noLiteralIncludes:
        "Use a named aggregate from `@acme/auth` (e.g. `STAFF_TIER_ROLE_LEVELS` / `ADMIN_TIER_ROLE_LEVELS`) instead of a bare string-literal `.includes()` against `roleLevel`. Spec: unify-rbac-and-add-departmental-scoping.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    /**
     * Convert a kebab-case canonical role-level to the SCREAMING_SNAKE_CASE
     * constant name exported from `@acme/auth`. e.g. `"event-admin"` →
     * `"EVENT_ADMIN"`. Returns the original string if it doesn't match
     * the kebab pattern (gracefully handles non-canonical literals).
     */
    function toConstantName(literal) {
      if (typeof literal !== "string") return "<NAME>";
      return literal.replace(/-/g, "_").toUpperCase();
    }

    /**
     * True when the AST node is a MemberExpression accessing `.roleLevel`.
     * Both `role.roleLevel` and `someExpr.roleLevel` qualify.
     */
    function isRoleLevelAccess(node) {
      return (
        node &&
        node.type === "MemberExpression" &&
        node.property &&
        node.property.type === "Identifier" &&
        node.property.name === "roleLevel"
      );
    }

    /**
     * True when the node is a string literal (handles both Literal and
     * TemplateLiteral with no expressions).
     */
    function isStringLiteral(node) {
      if (!node) return false;
      if (node.type === "Literal" && typeof node.value === "string") return true;
      if (
        node.type === "TemplateLiteral" &&
        node.expressions.length === 0 &&
        node.quasis.length === 1
      ) {
        return true;
      }
      return false;
    }

    function literalValue(node) {
      if (node.type === "Literal") return node.value;
      if (node.type === "TemplateLiteral") return node.quasis[0].value.cooked;
      return null;
    }

    return {
      // Pattern 1: <expr>.roleLevel === "literal" (or reversed)
      BinaryExpression(node) {
        if (node.operator !== "===" && node.operator !== "!==") return;

        if (isRoleLevelAccess(node.left) && isStringLiteral(node.right)) {
          context.report({
            node: node.right,
            messageId: "noLiteralEquality",
            data: { name: toConstantName(literalValue(node.right)) },
          });
        } else if (
          isRoleLevelAccess(node.right) &&
          isStringLiteral(node.left)
        ) {
          context.report({
            node: node.left,
            messageId: "noLiteralEquality",
            data: { name: toConstantName(literalValue(node.left)) },
          });
        }
      },

      // Pattern 2: ["literal", "literal"].includes(<expr>.roleLevel)
      CallExpression(node) {
        if (
          !node.callee ||
          node.callee.type !== "MemberExpression" ||
          !node.callee.property ||
          node.callee.property.type !== "Identifier" ||
          node.callee.property.name !== "includes"
        ) {
          return;
        }

        // Array literal of string literals as the receiver?
        const receiver = node.callee.object;
        if (!receiver || receiver.type !== "ArrayExpression") return;
        const allStringLiterals =
          receiver.elements.length > 0 &&
          receiver.elements.every((el) => el && isStringLiteral(el));
        if (!allStringLiterals) return;

        // First arg is `<expr>.roleLevel`?
        if (node.arguments.length === 0) return;
        if (!isRoleLevelAccess(node.arguments[0])) return;

        context.report({
          node: receiver,
          messageId: "noLiteralIncludes",
        });
      },
    };
  },
};
