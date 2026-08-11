---
name: vercel-react-best-practices
description: React and Next.js performance optimization from Vercel Engineering. Use when diagnosing slow first paint, laggy interactions, excessive re-renders, large JS bundles, or stale data. Triggers on performance review, bundle analysis, waterfall elimination, React.memo decisions, dynamic imports, Suspense streaming, or any "why is this page slow?" investigation.
source: ~/.agents/skills@2026-07-13
license: MIT
metadata:
  author: vercel
  version: "2.0.0"
user-invocable: false
---


# Vercel React Best Practices

Performance optimization guide for React and Next.js applications, maintained by Vercel. Contains 45 rules across 7 categories, prioritized by impact.

## Performance Triage

Before optimizing, **measure first**: React DevTools Profiler to identify the bottleneck, then apply the specific rule. NEVER optimize speculatively -- premature optimization wastes time and adds complexity.

```
Page feels slow?
├── First paint slow (white screen)?
│   ├── Large JS bundle? → Read rules/bundle-*.md
│   │   Start with: bundle-barrel-imports, bundle-dynamic-imports
│   └── Sequential server fetches (waterfall)? → Read rules/async-*.md
│       Start with: async-parallel, async-suspense-boundaries
├── Interactions feel laggy (clicks, typing)?
│   ├── Component re-renders too often? → Read rules/rerender-*.md
│   │   Start with: rerender-derived-state-no-effect, rerender-memo
│   └── Heavy DOM updates (lists, animations)? → Read rules/rendering-*.md
│       Start with: rendering-content-visibility, rendering-hoist-jsx
├── Data appears stale?
│   └── Caching issue → use nextjs-app-router skill instead (not this skill)
└── Not sure where the problem is?
    └── Profile first: React DevTools Profiler for renders, Chrome DevTools Performance for JS
```

## Rule Categories by Priority

| Priority | Category | Impact | Prefix |
|----------|----------|--------|--------|
| 1 | Eliminating Waterfalls | CRITICAL | `async-` |
| 2 | Bundle Size Optimization | CRITICAL | `bundle-` |
| 3 | Server-Side Performance | HIGH | `server-` |
| 4 | Client-Side Data Fetching | MEDIUM-HIGH | `client-` |
| 5 | Re-render Optimization | MEDIUM | `rerender-` |
| 6 | Rendering Performance | MEDIUM | `rendering-` |
| 7 | Advanced Patterns | LOW | `advanced-` |

## Top 5 Rules (Always Apply)

These 5 rules prevent the most common performance problems. Apply them by default without profiling.

### 1. async-parallel -- Promise.all for independent fetches

**WHY:** Sequential awaits create a waterfall -- total time = sum of all fetch times. Parallel = max of fetch times.

```typescript
// ❌ Waterfall: 200ms + 300ms = 500ms
const user = await getUser(id);
const orders = await getOrders(id);

// ✅ Parallel: max(200ms, 300ms) = 300ms
const [user, orders] = await Promise.all([getUser(id), getOrders(id)]);
```

### 2. bundle-barrel-imports -- Import directly, never from barrel files

**WHY:** Barrel files (`index.ts` re-exports) defeat tree-shaking -- bundler pulls the entire module graph.

```typescript
// ❌ Pulls entire utils package into client bundle
import { formatDate } from "@/utils";

// ✅ Only pulls formatDate and its dependencies
import { formatDate } from "@/utils/date";
```

### 3. rerender-derived-state-no-effect -- Derive during render

**WHY:** `useState` + `useEffect` for derived values causes an extra render with stale intermediate state.

```typescript
// ❌ Two renders: first with stale filteredItems, second with correct
const [filteredItems, setFiltered] = useState(items);
useEffect(() => setFiltered(items.filter(i => i.active)), [items]);

// ✅ One render, always consistent
const filteredItems = items.filter(i => i.active);
// Expensive? const filteredItems = useMemo(() => items.filter(i => i.active), [items]);
```

### 4. server-cache-react -- React.cache() for per-request dedup

**WHY:** Multiple Server Components in one render tree may fetch the same data. Without cache(), each triggers a separate DB/API call.

```typescript
import { cache } from "react";
export const getUser = cache(async (id: string) => {
  return db.query.user.findFirst({ where: eq(user.id, id) });
});
// Called 3 times in one request? Only 1 DB query executes.
```

### 5. async-suspense-boundaries -- Stream slow content

**WHY:** Without Suspense, the entire page waits for the slowest query. With it, fast content renders immediately.

```tsx
// ❌ Page blocked on slow orders query
export default async function Page() {
  const orders = await getOrders(); // 2 seconds
  return <><Header /><OrdersList orders={orders} /></>;
}

// ✅ Header renders instantly, orders stream in
export default function Page() {
  return (
    <>
      <Header />
      <Suspense fallback={<OrdersSkeleton />}>
        <OrdersList /> {/* async Server Component */}
      </Suspense>
    </>
  );
}
```

## Performance Anti-Patterns

NEVER:
- **React.memo on every component** -- adds comparison overhead on every render. Only memo components that: (a) receive complex object props, (b) re-render frequently from parent state changes, (c) are expensive to render (>5ms in profiler)
- **Optimize without measuring** -- you will optimize the wrong thing. Profile first, then apply the specific rule
- **Split bundles too aggressively** -- every dynamic import adds a network round-trip. Only split above 50KB or for routes the user may never visit
- **Cache everything** -- stale data bugs are harder to debug than slow queries. Start with no caching, add targeted caching where profiling shows repeated expensive operations
- **useCallback/useMemo everywhere** -- the memoization itself costs memory and comparison time. Only use when passing callbacks to memoized children or for genuinely expensive computations (>1ms)

## Loading Rules

Read individual rule files when the triage tree points you there:

```bash
# Example: triage identified waterfall → load async rules
Read ~/.agents/skills/vercel-react-best-practices/rules/async-parallel.md
```

**MANDATORY for performance reviews**: Read the Top 5 rules above (already inline).

**Load on demand** (match to triage result):
- `rules/async-*.md` -- waterfall elimination (5 rules)
- `rules/bundle-*.md` -- bundle size (5 rules)
- `rules/server-*.md` -- server-side perf (7 rules)
- `rules/rerender-*.md` -- re-render optimization (12 rules)
- `rules/rendering-*.md` -- DOM rendering perf (9 rules)
- `rules/advanced-*.md` -- advanced patterns (3 rules)

**Do NOT load** all rule files at once -- pick the category matching your triage result.
