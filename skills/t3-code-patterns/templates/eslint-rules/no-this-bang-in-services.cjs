"use strict";

/**
 * Rule: acme/no-this-bang-in-services
 *
 * Flags `this.x!` non-null assertions in the service layer
 * (packages/api/src/services/**). The existing BaseService pattern stores
 * tenancy/user fields as optional (`this.eventSeriesId?: string`), forcing
 * 161 subclasses to use `this.eventSeriesId!` repeatedly (~548 occurrences
 * total). Those `!` assertions lie to the type system — they assert
 * non-nullness without runtime proof.
 *
 * The new ServiceCtx pattern (see phase-a-service-contract-foundation
 * Decision 1) makes tenancy fields non-null by construction. Instead of
 * storing scope on `this`, services accept a narrowly-typed `ServiceCtx`
 * parameter whose tenancy fields are required (not optional). That
 * structural guarantee removes the need for `!` at the call site.
 *
 * Level: warn (non-blocking). Phase A uses this as a migration signal,
 * not a gate. Refactoring to accept ServiceCtx is a genuine code change,
 * not a mechanical transform, so this rule is NOT auto-fixable.
 *
 * Scope: files matching `packages/api/src/services/**`. Other packages
 * and the router/lib/infra layers are unaffected. Contract boundaries
 * (__contracts__, __tests__, __fixtures__) are excluded.
 *
 * Detection scope: narrowly `this.x!`. AST match:
 *   TSNonNullExpression
 *     > expression: MemberExpression
 *         > object: ThisExpression
 *
 * Out of scope (intentionally NOT flagged):
 *   - `variable!.method()`         — unrelated variable non-null assertions
 *   - `this.foo?.bar!`             — optional chain then bang (different pattern)
 *   - `(this.foo as Bar)!`         — explicit cast then bang
 *
 * Escape hatch: standard `// eslint-disable-next-line acme/no-this-bang-in-services`
 * per-site suppression.
 */
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow 'this.x!' non-null assertions in service-layer code",
      category: "acme/service-layer",
      recommended: true,
    },
    schema: [],
    messages: {
      noThisBang:
        "Avoid 'this.{{name}}!' — the new ServiceCtx pattern makes tenancy fields non-null by construction. Accept ServiceCtx as a parameter instead of storing on 'this'.",
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

    // Scope: only service-layer files under packages/api/src/services/.
    if (!/packages\/api\/src\/services\//.test(normalized)) return {};

    // Exclude contract boundaries, test files, and fixtures.
    if (/__(contracts|tests|fixtures)__/.test(normalized)) return {};

    return {
      /** @param {any} node */
      TSNonNullExpression(node) {
        const inner = node.expression;
        if (
          inner &&
          inner.type === "MemberExpression" &&
          inner.object &&
          inner.object.type === "ThisExpression"
        ) {
          const name =
            inner.property && inner.property.type === "Identifier"
              ? inner.property.name
              : "<computed>";
          context.report({
            node,
            messageId: "noThisBang",
            data: { name },
          });
        }
      },
    };
  },
};
