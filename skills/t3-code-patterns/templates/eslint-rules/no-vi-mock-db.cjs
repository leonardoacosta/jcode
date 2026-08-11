"use strict";

/**
 * Rule: acme/no-vi-mock-db
 *
 * Flags `vi.mock("@<workspace>/db", ...)` (and any subpath of the db package
 * like `@acme/db/client`, `@storefront/db/schema`). The canon Real-DB-Not-Mocks rule
 * (see `t3-testing-patterns` skill) requires a real test database for every
 * unit test that touches the data layer — mocking the DB client hides
 * schema/SQL bugs and produces tests that pass while production breaks.
 *
 * The 2026-05-17 fleet audit caught a small but persistent cluster of these
 * across the fleet, almost always in tests that were "easier to write" with
 * a mock. The fix is to point at the local Postgres test DB (`OO_TEST_POSTGRES_URL`
 * for acme, analogous env per project) and let the real schema validate the
 * query.
 *
 * Scope: every file EXCEPT `**\/__tests__/integration/**`. Integration tests
 * occasionally need to mock a remote DB or fault-inject, and the integration
 * dir is the explicit escape hatch. Standard unit-test files (under
 * `__tests__/` without `/integration/`) MUST hit the real DB.
 *
 * Detection: `CallExpression` where:
 *   - callee is `vi.mock` (MemberExpression: object.name === "vi",
 *     property.name === "mock"), and
 *   - first argument is a string Literal matching `/^@\w+\/db(\/|$)/`.
 *
 * Escape hatch: standard `// eslint-disable-next-line acme/no-vi-mock-db`
 * per-site suppression with a documented reason. Use only for genuinely
 * unmockable concerns (e.g. testing a teardown path that destroys the
 * connection).
 */

// Matches `@<ws>/db` exactly, or with a `/subpath`. Does NOT match
// `@<ws>/db-utils`, `@<ws>/database`, etc. — only the canonical db package.
const DB_PACKAGE_REGEX = /^@\w+\/db(\/|$)/;

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow vi.mock(@{ws}/db) — use a real test database (Real-DB-Not-Mocks canon)",
      category: "acme/testing",
      recommended: true,
    },
    schema: [],
    messages: {
      noMockDb:
        "Mocking the DB client is forbidden. Use a real test database (`OO_TEST_POSTGRES_URL` for acme; analogous env per project) per the canon Real-DB-Not-Mocks rule.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const filename =
      typeof context.getFilename === "function"
        ? context.getFilename()
        : context.filename || "";
    const normalized = filename.replace(/\\/g, "/");

    // Integration tests are the explicit escape hatch.
    if (/\/__tests__\/integration\//.test(normalized)) return {};

    return {
      /** @param {any} node */
      CallExpression(node) {
        const callee = node.callee;
        if (
          !callee ||
          callee.type !== "MemberExpression" ||
          !callee.object ||
          callee.object.type !== "Identifier" ||
          callee.object.name !== "vi" ||
          !callee.property ||
          callee.property.type !== "Identifier" ||
          callee.property.name !== "mock"
        ) {
          return;
        }
        const arg0 = node.arguments && node.arguments[0];
        if (!arg0) return;
        let value;
        if (arg0.type === "Literal" && typeof arg0.value === "string") {
          value = arg0.value;
        } else if (
          arg0.type === "TemplateLiteral" &&
          arg0.expressions.length === 0 &&
          arg0.quasis.length === 1
        ) {
          value = arg0.quasis[0].value.cooked;
        } else {
          return;
        }
        if (!DB_PACKAGE_REGEX.test(value)) return;
        context.report({ node, messageId: "noMockDb" });
      },
    };
  },
};
