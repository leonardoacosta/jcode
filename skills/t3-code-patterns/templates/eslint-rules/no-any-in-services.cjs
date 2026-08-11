"use strict";

/**
 * Rule: acme/no-any-in-services
 *
 * Discourages use of `any` in the service layer (packages/api/src/services/**).
 * Services should use `unknown` with narrowing, or define explicit types — `any`
 * silently disables TypeScript's safety net at the boundary where business logic
 * lives.
 *
 * Level: warn (non-blocking). Phase A of the service-layer migration uses this
 * as a signal, not a gate. Contracts boundaries (__contracts__, __tests__,
 * __fixtures__) are excluded because fixtures intentionally model untyped
 * external input.
 *
 * Scope: files matching `packages/api/src/services/**`. Other packages and the
 * router/lib/infra layers are unaffected.
 *
 * Detection: flags any `TSAnyKeyword` AST node. This covers:
 *   - explicit `: any` annotations
 *   - `as any` casts (the inner `TSAnyKeyword` is what users see highlighted)
 *   - generic parameters of type `any` (e.g. `Promise<any>`)
 *
 * Implicit `any` (missing annotations) is NOT covered — that requires typed
 * linting infra and is more appropriately handled by `noImplicitAny` in tsconfig.
 *
 * Escape hatch: standard `// eslint-disable-next-line acme/no-any-in-services`
 * per-site suppression. Do NOT parse custom comment decorators in this rule.
 */
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description: "Disallow 'any' in service-layer code",
      category: "acme/service-layer",
      recommended: true,
    },
    schema: [],
    messages: {
      noAny:
        "Avoid 'any' in service-layer code. Use 'unknown' with narrowing, or define an explicit type. Whitelist specific cases with // eslint-disable-next-line if truly necessary.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const filename = context.getFilename();

    // Scope: only service-layer files under packages/api/src/services/.
    // Use path separator `/` — Node on Windows normalizes via posix in ESLint.
    if (!/packages\/api\/src\/services\//.test(filename)) return {};

    // Exclude contract boundaries, test files, and fixtures.
    if (/__(contracts|tests|fixtures)__/.test(filename)) return {};

    return {
      /** @param {any} node */
      TSAnyKeyword(node) {
        context.report({ node, messageId: "noAny" });
      },
    };
  },
};
