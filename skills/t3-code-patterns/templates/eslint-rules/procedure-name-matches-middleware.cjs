"use strict";

/**
 * Rule: acme/procedure-name-matches-middleware
 *
 * Flags `export const adminProcedure = ...` (or any procedure whose name
 * MATCHES a configured `procedurePatterns` regex) whose value's middleware
 * chain does NOT include a recognizable role/permission check. Auth-naming
 * bugs are critical: a procedure named `adminProcedure` that only calls
 * `requireTenantContext` and not `requireRole("admin")` looks safe but
 * permits every authenticated tenant user.
 *
 * The 2026-05-17 fleet audit caught one of these in portal
 * (`ssAdminProcedure` missing the admin-role middleware). See
 * `t3-code-patterns` skill § Tenant-Scoped Procedures and
 * `trpc-patterns` skill § Procedure Inventory File.
 *
 * Detection:
 *   1. Match `VariableDeclarator` whose `id` is an Identifier matching one
 *      of `options.procedurePatterns` (default: ["^admin", "Admin$"]).
 *   2. Walk the call chain on `init` collecting every `.use(<fn>)` argument.
 *   3. For each `.use()` callback, scan its source text for any substring
 *      from `options.roleCheckIdentifiers`
 *      (default: ["role", "Admin", "permission", "rbac"]).
 *   4. If none of the middleware bodies contain a match, report the
 *      declarator's `id`.
 *
 * The substring scan is intentionally coarse — it matches `requireRole`,
 * `assertAdmin`, `hasPermission`, `rbac.check`, etc. without enumerating
 * each helper name. If a project's helper names sit outside these
 * substrings, extend `roleCheckIdentifiers` via options instead of editing
 * this rule.
 *
 * Options:
 *   {
 *     procedurePatterns:    string[]  // regex strings; default ["^admin", "Admin$"]
 *     roleCheckIdentifiers: string[]  // substrings;    default ["role", "Admin", "permission", "rbac"]
 *   }
 *
 * Escape hatch: standard
 * `// eslint-disable-next-line acme/procedure-name-matches-middleware`
 * per-site suppression with a documented reason. Adding suppression
 * SHOULD trigger a review comment — the rule exists to catch auth bugs,
 * so silencing it without inspection is a smell.
 */

const DEFAULT_PROCEDURE_PATTERNS = ["^admin", "Admin$"];
const DEFAULT_ROLE_IDENTIFIERS = ["role", "Admin", "permission", "rbac"];

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Procedure names implying an authorization gate must have a matching role/permission middleware",
      category: "acme/trpc",
      recommended: true,
    },
    schema: [
      {
        type: "object",
        properties: {
          procedurePatterns: {
            type: "array",
            items: { type: "string" },
          },
          roleCheckIdentifiers: {
            type: "array",
            items: { type: "string" },
          },
        },
        additionalProperties: false,
      },
    ],
    messages: {
      missingCheck:
        "Procedure named `{{name}}` implies an authorization check, but its middleware chain does not include one. Either rename the procedure to match its actual behavior, or add the role check.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const options = context.options[0] || {};
    const patterns = (options.procedurePatterns || DEFAULT_PROCEDURE_PATTERNS).map(
      (p) => new RegExp(p),
    );
    const identifiers = options.roleCheckIdentifiers || DEFAULT_ROLE_IDENTIFIERS;
    const sourceCode =
      typeof context.getSourceCode === "function"
        ? context.getSourceCode()
        : context.sourceCode;

    function nameMatches(name) {
      return patterns.some((rx) => rx.test(name));
    }

    /** Collect every `.use(fn)` argument node found in the chain. */
    function collectUseArgs(init) {
      /** @type {any[]} */
      const args = [];
      let cursor = init;
      while (cursor && cursor.type === "CallExpression") {
        const callee = cursor.callee;
        if (
          callee &&
          callee.type === "MemberExpression" &&
          callee.property &&
          callee.property.type === "Identifier" &&
          callee.property.name === "use"
        ) {
          if (cursor.arguments && cursor.arguments.length > 0) {
            args.push(cursor.arguments[0]);
          }
        }
        // Walk root-ward through the call chain.
        if (callee && callee.type === "MemberExpression") {
          cursor = callee.object;
        } else {
          break;
        }
      }
      return args;
    }

    function bodyContainsIdentifier(node) {
      if (!node || !sourceCode) return false;
      let text;
      try {
        text = sourceCode.getText(node);
      } catch {
        return false;
      }
      if (typeof text !== "string") return false;
      // Substring match is case-insensitive so default identifiers like
      // "role" / "Admin" catch `requireRole`, `assertADMIN`, `roleCheck`,
      // etc. without requiring callers to enumerate every casing.
      const haystack = text.toLowerCase();
      return identifiers.some((id) => haystack.includes(id.toLowerCase()));
    }

    return {
      /** @param {any} node */
      VariableDeclarator(node) {
        if (!node.id || node.id.type !== "Identifier") return;
        const name = node.id.name;
        if (!nameMatches(name)) return;
        if (!node.init) return;

        const useArgs = collectUseArgs(node.init);
        // If there is no .use() call at all, definitely no role check.
        // Also flag when none of the .use() bodies contain a role identifier.
        const hasRoleCheck = useArgs.some(bodyContainsIdentifier);
        if (hasRoleCheck) return;

        context.report({
          node: node.id,
          messageId: "missingCheck",
          data: { name },
        });
      },
    };
  },
};
