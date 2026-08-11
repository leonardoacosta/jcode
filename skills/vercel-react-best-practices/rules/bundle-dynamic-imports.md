---
title: Dynamic Imports for Heavy Components
impact: CRITICAL
impactDescription: directly affects TTI and LCP
tags: bundle, dynamic-import, code-splitting, next-dynamic
---


## Dynamic Imports for Heavy Components

Use `next/dynamic` to lazy-load large components not needed on initial render.

**Incorrect (Monaco bundles with main chunk ~300KB):**

```tsx
import { MonacoEditor } from './monaco-editor'

function CodePanel({ code }: { code: string }) {
  return <MonacoEditor value={code} />
}
```

**Correct (Monaco loads on demand):**

```tsx
import dynamic from 'next/dynamic'

const MonacoEditor = dynamic(
  () => import('./monaco-editor').then(m => m.MonacoEditor),
  { ssr: false }
)

function CodePanel({ code }: { code: string }) {
  return <MonacoEditor value={code} />
}
```

> **Turbopack update (Next.js 16.2+):** destructured dynamic imports
> (`const { X } = await import('./lib')`) are now tree-shaken the same as static imports. If you
> were reaching for `next/dynamic` purely to dodge a barrel file's tree-shaking cost, that
> workaround is obsolete — see `bundle-barrel-imports.md`. The lazy-loading use case above (keep a
> heavy component out of the initial render) is a separate concern and still applies.
