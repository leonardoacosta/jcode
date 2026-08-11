"use strict";

/**
 * Rule: acme/no-bare-identifier-in-sql-template
 *
 * Heuristic Cluster-E detector: flags `sql\`...FROM ${identifier}...\``
 * (and JOIN/INTO/UPDATE/TABLE/ALTER/REFERENCES variants) where the
 * interpolated expression is a bare Identifier appearing in
 * SQL-identifier-position. The risk is that `identifier` resolves to a
 * runtime string (e.g. a function parameter `tableName: string`), in
 * which case Drizzle splices it into the SQL text as-is.
 *
 * The rule is heuristic by design: we cannot tell from syntax alone
 * whether the identifier resolves to a Drizzle `PgTable` object (safe)
 * or a `string` (unsafe). The escape hatch is the canonical
 * `sql.identifier(name)` wrapper (or this codebase's `sqlIdentifier()`
 * helper) which produces a quoted-identifier SQL chunk. Suppressing
 * the rule with `// eslint-disable-next-line acme/no-bare-identifier-in-sql-template`
 * is acceptable when the identifier is provably a `PgTable`.
 *
 * Detection:
 *   - tag is sql / x.sql
 *   - the TemplateLiteral quasi immediately preceding a TemplateSpan
 *     ends (ignoring whitespace) with one of the keywords:
 *     FROM, JOIN, INTO, UPDATE, TABLE, ALTER, REFERENCES
 *   - the corresponding expression is an Identifier (not a
 *     PropertyAccess, CallExpression, BinaryExpression, etc.)
 *
 * Level: error in packages/api, packages/db, packages/auth, packages/e2e.
 */
const KEYWORD_RX = /\b(FROM|JOIN|INTO|UPDATE|TABLE|ALTER|REFERENCES|EXISTS)\s*$/i;

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow bare string identifiers in SQL identifier-position inside sql\\`\\` templates",
      category: "acme/sql-safety",
      recommended: true,
    },
    schema: [],
    messages: {
      bareIdentifier:
        "Bare identifier '{{name}}' in SQL identifier-position is unsafe if '{{name}}' is a runtime string. Use sql.identifier({{name}}) or the local sqlIdentifier() helper. If '{{name}}' is provably a Drizzle PgTable, suppress with an eslint-disable comment.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    /** @param {any} tag */
    function isSqlTag(tag) {
      if (!tag) return false;
      if (tag.type === "Identifier" && tag.name === "sql") return true;
      if (
        tag.type === "MemberExpression" &&
        tag.property &&
        tag.property.type === "Identifier" &&
        tag.property.name === "sql"
      )
        return true;
      return false;
    }

    return {
      /** @param {any} node */
      TaggedTemplateExpression(node) {
        if (!isSqlTag(node.tag)) return;
        const quasi = node.quasi;
        if (!quasi || quasi.type !== "TemplateLiteral") return;
        const quasis = quasi.quasis || [];
        const expressions = quasi.expressions || [];
        for (let i = 0; i < expressions.length; i++) {
          const expr = expressions[i];
          if (!expr || expr.type !== "Identifier") continue;
          const before = quasis[i];
          if (!before) continue;
          const rawText = before.value && (before.value.cooked || before.value.raw);
          if (typeof rawText !== "string") continue;
          if (!KEYWORD_RX.test(rawText)) continue;
          context.report({
            node: expr,
            messageId: "bareIdentifier",
            data: { name: expr.name },
          });
        }
      },
    };
  },
};
