"use strict";

//
// Rule: acme/no-restricted-imports
//
// Flags `@<workspace>/db` (and any subpath like `@acme/db/client`) imports from
// `apps/nextjs/src/`-rooted files EXCEPT route handlers under
// `apps/nextjs/src/app/api/.../route.ts`.
//
// The 2026-05-17 fleet audit identified anti-pattern A4: Next.js Server
// Components and client code were importing the Drizzle DB client directly,
// bypassing the tRPC service layer. This breaks the architectural boundary:
//   apps/nextjs -> packages/api (tRPC services) -> packages/db (Drizzle)
//
// Direct app -> db imports introduce schema coupling, leak DB internals into
// UI code, and make it impossible to swap the data layer without touching
// every consumer.
//
// Allowed exception: App Router route handlers (file path ending in /route.ts
// under app/api/) are the only allowed direct-db boundary, used for webhook
// receivers and other endpoints that need raw DB access without tRPC overhead.
//
// Scope: This rule applies to apps/nextjs/src/-rooted files (Server Components,
// client components, server actions). Source files under packages/api/,
// packages/db/, etc. are NOT affected — the boundary rule only protects the
// app layer.
//
// Detection: ImportDeclaration where:
//   - the file path matches apps/nextjs/src/ AND
//   - the file path does NOT match apps/nextjs/src/app/api/.../route.ts AND
//   - the import source matches /^@\w+\/db(\/|$)/
//
// Escape hatch: `// eslint-disable-next-line acme/no-restricted-imports` per-site
// with a documented reason. Use only for genuinely-unavoidable direct DB
// access in the app layer (e.g., a one-off migration script that lives in
// apps/nextjs by accident — fix the location, not the suppression).
//

const DB_PACKAGE_REGEX = /^@\w+\/db(\/|$)/;
const APP_SRC_REGEX = /apps\/nextjs\/src\//;
const ROUTE_HANDLER_REGEX = /apps\/nextjs\/src\/app\/api\/.*\/route\.ts$/;

module.exports = {
  meta: {
    type: "problem",
    docs: {
      description:
        "Disallow direct @{ws}/db imports from apps/nextjs/src/** (use tRPC service layer); only app/api/**/route.ts is exempt",
      category: "acme/architecture",
      recommended: true,
    },
    schema: [],
    messages: {
      noDirectDb:
        "Direct import of `{{source}}` from apps/nextjs is forbidden. Use the tRPC service layer (`@{ws}/api`) — only `app/api/**/route.ts` may import the DB directly.",
    },
  },

  /** @param {import('eslint').Rule.RuleContext} context */
  create(context) {
    const filename =
      typeof context.getFilename === "function"
        ? context.getFilename()
        : context.filename || "";
    const normalized = filename.replace(/\\/g, "/");

    // Only enforce inside apps/nextjs/src/**
    if (!APP_SRC_REGEX.test(normalized)) return {};

    // Skip the explicit exception: route handlers
    if (ROUTE_HANDLER_REGEX.test(normalized)) return {};

    return {
      /** @param {any} node */
      ImportDeclaration(node) {
        const source = node.source && node.source.value;
        if (typeof source !== "string") return;
        if (!DB_PACKAGE_REGEX.test(source)) return;
        context.report({
          node,
          messageId: "noDirectDb",
          data: { source },
        });
      },
    };
  },
};
