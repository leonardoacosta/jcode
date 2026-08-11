"use strict";

/**
 * Rule: acme/no-nested-template-in-sql
 *
 * Flags nested template literals inside `sql\`...\`` tagged-template
 * expressions, e.g. `sql\`... LIKE ${\`%${input}%\`} ...\``. This pattern is
 * the canonical Cluster D' unsafe interpolation from
 * `openspec/changes/purge-unsafe-sql-template-interp/`: the inner template
 * is evaluated as a JS string and concatenated into the SQL text *before*
 * Drizzle gets a chance to bind it as a parameter, opening an injection
 * channel.
 *
 * Safe rewrite: use `sql.placeholder('name')` (bind in `.execute({ name })`),
 * or pass the wildcards as a JS string and let Drizzle bind:
 *   const pattern = `%${input}%`;
 *   sql`... LIKE ${pattern}`;
 *
 * Level: error in packages/api, packages/db, packages/auth, packages/e2e.
 *
 * Scope: AST match TaggedTemplateExpression where:
 *   - tag is Identifier 'sql' OR PropertyAccess ending in '.sql'
 *   - template is TemplateLiteral with one or more expressions of type
 *     TemplateLiteral.
 */
module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow nested template literals inside sql\\`...\\` tagged templates",
      category: "acme/sql-safety",
      recommended: true,
    },
    schema: [],
    messages: {
      nestedTemplate:
        "Nested template literal inside sql\\`\\` is UNSAFE — the inner string is concatenated into the SQL text before parameter binding. Use sql.placeholder() or bind a pre-built string instead.",
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
        for (const expr of quasi.expressions || []) {
          if (
            expr &&
            (expr.type === "TemplateLiteral" ||
              expr.type === "TaggedTemplateExpression")
          ) {
            context.report({
              node: expr,
              messageId: "nestedTemplate",
            });
          }
        }
      },
    };
  },
};
