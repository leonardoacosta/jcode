
# AST-Selector ESLint Configs by Package

CI-enforced rules — each selector compiles into the package's flat `eslint.config.js` and fails
the build on every PR. Use these for anything a bad actor could silently reintroduce.

## Base (All Projects)

```javascript
{
  rules: {
    "no-restricted-syntax": [
      "error",
      {
        selector: "CallExpression[callee.object.name='console'][callee.property.name='error']",
        message: "Use Sentry.captureException() or logError() instead"
      },
      {
        selector: "Program > :matches(ExpressionStatement, BlockStatement) > :matches(Line, Block):has([value=/(TODO|FIXME|XXX|HACK)/i])",
        message: "Remove TODO/FIXME comments. Create tasks or implement fully."
      }
    ],
    "@typescript-eslint/no-explicit-any": "error",
    "@typescript-eslint/no-unsafe-assignment": "error",
    "no-console": ["warn", { allow: ["warn", "error", "info"] }],
  }
}
```

## packages/db

```javascript
{
  rules: {
    "no-restricted-syntax": [
      "error",
      {
        selector: "TaggedTemplateExpression[tag.name='sql']",
        message: "Wrap raw SQL in db.transaction() for safety"
      }
    ]
  }
}
```

## packages/api

```javascript
{
  rules: {
    "no-restricted-syntax": [
      "error",
      {
        selector: "MemberExpression[object.name='ctx'][property.name='db']",
        message: "Import db directly: import { db } from '@{workspace}/db/client'"
      }
    ],
    "no-restricted-imports": [
      "error",
      {
        patterns: [
          {
            group: ["**/db/client"],
            importNames: ["db"],
            message: "In routers, db should be imported at top of file, not in procedures"
          }
        ]
      }
    ]
  }
}
```

## apps/nextjs

```javascript
{
  rules: {
    "no-restricted-syntax": [
      "error",
      {
        selector: "CallExpression[callee.property.name='useQuery'][callee.object.property.name!='queryOptions']",
        message: "Use useQuery(trpc.x.queryOptions(...)) not trpc.x.useQuery()"
      },
      {
        selector: "CallExpression[callee.property.name='useMutation'][callee.object.property.name!='mutationOptions']",
        message: "Use useMutation(trpc.x.mutationOptions(...)) not trpc.x.useMutation()"
      },
      {
        selector: "JSXAttribute[name.name='style']",
        message: "Use Tailwind classes instead of inline styles"
      },
      {
        selector: "CallExpression[callee.name='useAtom'] > Identifier[name=/.*Query.*|.*Data.*|.*Response.*/]",
        message: "Server data belongs in React Query, not Jotai. Use queryOptions with select."
      },
      {
        selector: "FunctionDeclaration:has(CallExpression[callee.name='useQuery']):has(CallExpression[callee.name='useQuery'] ~ CallExpression[callee.name='useQuery']):has(CallExpression[callee.name='useMutation'])",
        message: "Components with 3+ queries must not also have mutations. Split into display + action components per STATE.md."
      }
    ],
    "no-restricted-imports": [
      "error",
      {
        patterns: [
          { group: ["@radix-ui/*"], message: "Import from @{workspace}/ui instead" },
          { group: ["class-variance-authority"], message: "CVA variants belong in @{workspace}/ui" },
          { group: ["**/types/*"], importNames: ["*DTO", "*Response", "*Request"], message: "Use RouterOutputs from @{workspace}/api for DTOs" },
          { group: ["lucide-react"], message: "Import icons from @{workspace}/ui/icons instead" }
        ]
      }
    ]
  }
}
```

## apps/expo

```javascript
{
  rules: {
    "no-restricted-syntax": [
      "error",
      { selector: "JSXAttribute[name.name='width'][value.type='Literal']", message: "Use responsive values, not hardcoded dimensions" },
      { selector: "JSXAttribute[name.name='height'][value.type='Literal']", message: "Use responsive values, not hardcoded dimensions" },
      { selector: "MemberExpression[object.name='window']", message: "window is not available in React Native" },
      { selector: "MemberExpression[object.name='document']", message: "document is not available in React Native" }
    ],
    "no-restricted-imports": [
      "error",
      { patterns: [{ group: ["react-dom", "next/*"], message: "Web-only package not available in React Native" }] }
    ]
  }
}
```

## packages/e2e

```javascript
{
  rules: {
    "no-restricted-syntax": [
      "error",
      { selector: "CallExpression[callee.property.name='waitForTimeout']", message: "Use waitForSelector or expect assertions instead" },
      { selector: "Literal[value=/^\\.[a-z-]+$/]", message: "Avoid CSS class selectors. Use data-testid or role" },
      { selector: "Literal[value=/^#[a-z-]+$/]", message: "Avoid ID selectors. Use data-testid or role" },
      { selector: "Literal[value=/:nth-child/]", message: "nth-child selectors are fragile. Use data-testid" }
    ]
  }
}
```

## oxlint parity

`oxlint` reads the same `no-restricted-syntax` / `no-restricted-imports` selector shape as ESLint's
flat config — the blocks above port directly when a package migrates to oxlint; no rewrite needed
beyond confirming oxlint's AST selector dialect supports the specific matcher used (most of the
above are plain `CallExpression`/`MemberExpression`/`JSXAttribute` matches, which oxlint supports).
