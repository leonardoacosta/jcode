
# Codebase Audit Scripts

One-shot grep audits — run on demand during a codebase audit, not enforced by CI. Findings are
whitelisted per-instance with an inline exception comment; the violation set is expected to shrink
over time, not be re-litigated every commit.

## Design Token Compliance

```bash
# Detect arbitrary color classes (should use theme tokens)
grep -rn "className=" apps/nextjs/src --include="*.tsx" | \
  grep -oE "(bg|text|border|ring|fill|stroke)-(red|blue|green|yellow|orange|purple|pink|gray|slate|zinc|neutral|stone|amber|lime|emerald|teal|cyan|sky|indigo|violet|fuchsia|rose)-[0-9]+" | \
  sort | uniq -c | sort -rn

# Find files with arbitrary colors
grep -rln "(bg|text|border)-(red|blue|green|yellow|purple|pink|gray)-[0-9]" apps/nextjs/src --include="*.tsx"
```

Allowed vs forbidden token classes: see `references/naming-and-tokens.md` § Allowed Theme Tokens.

Whitelist with `@theme-exception` comment.

---

## Component Sourcing Audit

```bash
# Find local primitives that duplicate @{workspace}/ui
find apps/nextjs/src -name "Button.tsx" -o -name "Card.tsx" -o -name "Dialog.tsx" -o -name "Input.tsx" -o -name "Modal.tsx" -o -name "Select.tsx"

# Count components in app vs ui package
echo "App components:" && find apps/nextjs/src/components -name "*.tsx" | wc -l
echo "UI package:" && find packages/ui/src -name "*.tsx" | wc -l

# Find imports not from @{workspace}/ui
grep -r "from ['\"]\.\./" apps/nextjs/src --include="*.tsx" | grep -E "Button|Card|Dialog|Input|Modal" | head -20
```

### Inline Primitive Detection

```bash
# Button-like: onClick + cursor-pointer on div/span
grep -rn "className=.*cursor-pointer" apps/nextjs/src --include="*.tsx" | grep -E "<(div|span).*onClick"

# Button-like: role="button" on non-button elements
grep -rn 'role="button"' apps/nextjs/src --include="*.tsx" | grep -v "<button"

# Card-like: rounded + shadow combo
grep -rn "className=.*rounded.*shadow\|className=.*shadow.*rounded" apps/nextjs/src --include="*.tsx"

# Input-like: contentEditable on non-input
grep -rn "contentEditable" apps/nextjs/src --include="*.tsx" | grep -v "<input\|<textarea"

# Link-like: programmatic navigation on clickable div
grep -rn "router.push\|navigate(" apps/nextjs/src --include="*.tsx" | grep "onClick"
```

Whitelist with `@ui-exception` comment.

---

## State Management Audit

```bash
# Find atoms that might hold server data
grep -rn "atom<.*\[\]>\|atomWithStorage.*\[\]" apps/nextjs/src/lib/atoms --include="*.ts"

# Find Context storing arrays (likely server data)
grep -rn "createContext<.*\[\]>" apps/nextjs/src --include="*.tsx"

# Find useAtom near API types
grep -rn "useAtom.*Data\|useAtom.*Response\|useAtom.*Entity" apps/nextjs/src --include="*.tsx"
```

### Missing Invalidation

```bash
# Find mutations without onSuccess
grep -rn "mutationOptions({" apps/nextjs/src --include="*.tsx" -A 5 | grep -v "onSuccess"

# Find mutateAsync without nearby invalidate
grep -rn "mutateAsync" apps/nextjs/src --include="*.tsx" | \
  xargs -I {} sh -c 'file=$(echo "{}" | cut -d: -f1); grep -L "invalidateQueries" "$file" 2>/dev/null'
```

### Parallel State Stores

```bash
# Find components with both useAtom and useQuery for same entity
grep -rln "useAtom.*schedule\|schedule.*useAtom" apps/nextjs/src --include="*.tsx" | \
  xargs grep -l "trpc.schedule"

# Find useState mirroring query data
grep -rn "useState.*data\)" apps/nextjs/src --include="*.tsx" | grep -v "isLoading\|error"
```

Whitelist with `@state-exception` comment.

---

## Component File Isolation

```bash
# Find files with multiple exported components
grep -rln "^export function\|^export const.*=.*function\|^export default function" apps/nextjs/src --include="*.tsx" | \
  xargs -I {} sh -c 'count=$(grep -c "^export function\|^export const.*=.*function\|^export default function" "{}"); [ "$count" -gt 1 ] && echo "{}: $count exports"'

# Find inline dialog definitions (Dialog not in *Dialog.tsx file)
grep -rln "<Dialog" apps/nextjs/src --include="*.tsx" | grep -v "Dialog.tsx$"

# Find function components defined inside other components
grep -rn "function [A-Z][a-zA-Z]*(" apps/nextjs/src --include="*.tsx" | \
  grep -v "^[^:]*:.*export\|^[^:]*:.*^function"
```

Whitelist with `@multi-component` comment.
