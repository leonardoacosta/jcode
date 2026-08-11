
# File Naming Conventions & Token Allowlists

Static tables — compare the candidate name/class directly, no script needed.

## File Naming Conventions

| Component Type | File Name | Example |
|----------------|-----------|---------|
| Page | `page.tsx` | `app/users/page.tsx` |
| List | `{Entity}List.tsx` | `UserList.tsx` |
| Form | `{Entity}Form.tsx` | `UserForm.tsx` |
| Dialog | `{Action}{Entity}Dialog.tsx` | `CreateUserDialog.tsx` |
| Card | `{Entity}Card.tsx` | `UserCard.tsx` |

## Allowed Theme Tokens

| Category | Allowed | Forbidden |
|----------|---------|-----------|
| Background | `bg-background`, `bg-primary`, `bg-secondary`, `bg-muted`, `bg-accent`, `bg-destructive`, `bg-card`, `bg-popover` | `bg-red-600`, `bg-slate-100` |
| Text | `text-foreground`, `text-primary`, `text-secondary`, `text-muted-foreground`, `text-accent-foreground`, `text-destructive` | `text-gray-500`, `text-blue-600` |
| Border | `border-border`, `border-input`, `border-primary`, `border-destructive` | `border-gray-200`, `border-red-500` |
| Ring | `ring-ring`, `ring-primary` | `ring-blue-500` |

Whitelist a deliberate exception with `@theme-exception` comment (see `references/audit-scripts.md`
§ Design Token Compliance for the detection grep).

## Tailwind Plugin (Optional)

Flat-config `eslint-plugin-tailwindcss` enforcement mirrors the same allowlist above:

```javascript
// eslint.config.js
{
  plugins: ["tailwindcss"],
  rules: {
    "tailwindcss/no-custom-classname": ["warn", {
      whitelist: [
        "bg-(background|primary|secondary|muted|accent|destructive|card|popover)",
        "text-(foreground|primary|secondary|muted-foreground|accent-foreground|destructive)",
        "border-(border|input|primary|destructive)",
      ]
    }]
  }
}
```
