
# UI/UX Pro Max - Design Intelligence

> Formerly a standalone skill (`ui-ux-pro-max`), demoted to a `frontend-design` reference
> (`skill-classification-and-trial-lifecycle`, 2026-07-18) — by its own description this is a
> database (161 color palettes, 57 font pairings, 99 UX guidelines, 25 chart types across 10
> stacks) wearing a Skill's clothes, not decision-shaping content. Original vendored source:
> `~/.agents/skills@2026-07-13`. See sibling `data/`, `references/`, and `scripts/` in this
> directory for the searchable rule set.

161 rules across 10 categories, searchable via Python CLI. Rules extracted to `references/` for on-demand loading.

## When to Apply

**Must Use:**
- Designing new pages (Landing, Dashboard, Admin, SaaS, Mobile)
- Creating or refactoring UI components
- Choosing color schemes, typography, spacing, or layout systems
- Reviewing UI for UX, accessibility, or visual consistency
- Implementing navigation, animations, or responsive behavior

**Recommended:**
- UI looks unprofessional but reason is unclear
- Pre-launch quality optimization
- Cross-platform design alignment (Web / iOS / Android)
- Building design systems or reusable component libraries

**Skip:**
- Pure backend logic, API/DB design, non-UI performance, infra/DevOps, non-visual scripts

**Decision criteria**: If the task changes how a feature **looks, feels, moves, or is interacted with**, use this skill.

## UX Triage

```
UI problem?
├── Accessibility issue (contrast, focus, screen reader)?
│   └── Read references/rules-accessibility.md
├── Touch/interaction problem (targets, feedback, gestures)?
│   └── Read references/rules-interaction.md
├── Performance (jank, layout shift, slow load)?
│   └── Read references/rules-performance.md
├── Style inconsistency (colors, effects, platform mismatch)?
│   └── Read references/rules-style.md
├── Layout breaks (responsive, spacing, safe areas)?
│   └── Read references/rules-layout.md
├── Typography/color issues (readability, contrast, dark mode)?
│   └── Read references/rules-typography-color.md
├── Animation feels wrong (timing, easing, blocking)?
│   └── Read references/rules-animation.md
├── Form UX (validation, errors, labels)?
│   └── Read references/rules-forms.md
├── Navigation confusion (back behavior, hierarchy, deep links)?
│   └── Read references/rules-navigation.md
└── Charts/data viz (legends, tooltips, accessibility)?
    └── Read references/rules-charts.md
```

## Top 10 Rules (Always Check)

| Rule | Summary |
|------|---------|
| `color-contrast` | 4.5:1 minimum for normal text |
| `touch-target-size` | Min 44x44pt (Apple) / 48x48dp (Material) |
| `image-optimization` | WebP/AVIF, responsive images, lazy load |
| `no-emoji-icons` | Use SVG icons, not emojis |
| `mobile-first` | Design mobile-first, scale up |
| `input-labels` | Visible label per input, not placeholder-only |
| `duration-timing` | 150-300ms micro-interactions, <=400ms complex |
| `back-behavior` | Back navigation must be predictable, preserve state |
| `reduced-motion` | Respect prefers-reduced-motion |
| `content-jumping` | Reserve space for async content (CLS < 0.1) |

## Rule Categories by Priority

| Priority | Category | Impact | Domain | Key Checks | Anti-Patterns |
|----------|----------|--------|--------|------------|---------------|
| 1 | Accessibility | CRITICAL | `ux` | Contrast 4.5:1, Alt text, Keyboard nav, Aria-labels | Removing focus rings, Icon-only buttons without labels |
| 2 | Touch & Interaction | CRITICAL | `ux` | Min size 44x44px, 8px+ spacing, Loading feedback | Reliance on hover only, Instant state changes (0ms) |
| 3 | Performance | HIGH | `ux` | WebP/AVIF, Lazy loading, Reserve space (CLS < 0.1) | Layout thrashing, Cumulative Layout Shift |
| 4 | Style Selection | HIGH | `style`, `product` | Match product type, Consistency, SVG icons | Mixing flat & skeuomorphic, Emoji as icons |
| 5 | Layout & Responsive | HIGH | `ux` | Mobile-first, Viewport meta, No horizontal scroll | Horizontal scroll, Fixed px widths, Disable zoom |
| 6 | Typography & Color | MEDIUM | `typography`, `color` | Base 16px, Line-height 1.5, Semantic tokens | Text < 12px, Gray-on-gray, Raw hex |
| 7 | Animation | MEDIUM | `ux` | Duration 150-300ms, Motion meaning, Continuity | Decorative-only, Animating width/height, No reduced-motion |
| 8 | Forms & Feedback | MEDIUM | `ux` | Visible labels, Error near field, Progressive disclosure | Placeholder-only, Errors only at top |
| 9 | Navigation Patterns | HIGH | `ux` | Predictable back, Bottom nav <=5, Deep linking | Overloaded nav, Broken back, No deep links |
| 10 | Charts & Data | LOW | `chart` | Legends, Tooltips, Accessible colors | Color-only meaning |

## How to Use This Skill

| Scenario | Trigger Examples | Start From |
|----------|-----------------|------------|
| **New project / page** | "Build a landing page", "Build a dashboard" | Step 1 then Step 2 (design system) |
| **New component** | "Create a pricing card", "Add a modal" | Step 3 (domain search: style, ux) |
| **Choose style / color / font** | "What style fits a fintech app?" | Step 2 (design system) |
| **Review existing UI** | "Review this page for UX issues" | Top 10 Rules + Triage tree |
| **Fix a UI bug** | "Button hover is broken", "Layout shifts" | Triage tree to reference file |
| **Improve / optimize** | "Make this faster", "Improve mobile UX" | Step 3 (domain search: ux, react) |

### Step 1: Analyze User Requirements

Extract: product type, target audience, style keywords, stack.

### Step 2: Generate Design System (REQUIRED)

```bash
python3 skills/ui-ux-pro-max/scripts/search.py "<product_type> <industry> <keywords>" --design-system [-p "Project Name"]
```

Returns: pattern, style, colors, typography, effects, anti-patterns.

**Persist for cross-session use:** add `--persist` (creates `design-system/MASTER.md`).
**Page-specific override:** add `--page "dashboard"` (creates `design-system/pages/dashboard.md`).

Hierarchical retrieval: check `design-system/pages/[name].md` first, fall back to `MASTER.md`.

### Step 3: Supplement with Detailed Searches

```bash
python3 skills/ui-ux-pro-max/scripts/search.py "<keyword>" --domain <domain> [-n <max_results>]
```

| Need | Domain | Example |
|------|--------|---------|
| Product type patterns | `product` | `--domain product "entertainment social"` |
| More style options | `style` | `--domain style "glassmorphism dark"` |
| Color palettes | `color` | `--domain color "entertainment vibrant"` |
| Font pairings | `typography` | `--domain typography "playful modern"` |
| Chart recommendations | `chart` | `--domain chart "real-time dashboard"` |
| UX best practices | `ux` | `--domain ux "animation accessibility"` |
| Individual Google Fonts | `google-fonts` | `--domain google-fonts "sans serif popular variable"` |
| Landing structure | `landing` | `--domain landing "hero social-proof"` |
| React Native perf | `react` | `--domain react "rerender memo list"` |
| App interface a11y | `web` | `--domain web "accessibilityLabel touch safe-areas"` |
| AI prompt / CSS keywords | `prompt` | `--domain prompt "minimalism"` |

### Step 4: Stack Guidelines

```bash
python3 skills/ui-ux-pro-max/scripts/search.py "<keyword>" --stack react-native
```

## Output Formats

```bash
# ASCII box (default)
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system

# Markdown
python3 skills/ui-ux-pro-max/scripts/search.py "fintech crypto" --design-system -f markdown
```

## Tips

- Use **multi-dimensional keywords**: `"entertainment social vibrant content-dense"` not just `"app"`
- Try different keywords for the same need: `"playful neon"` / `"vibrant dark"` / `"content-first minimal"`
- Use `--design-system` first, then `--domain` to deep-dive specific dimensions
- Always add `--stack react-native` for implementation guidance

## Common Sticking Points

| Problem | What to Do |
|---------|------------|
| Can't decide on style/color | Re-run `--design-system` with different keywords |
| Dark mode contrast issues | `color-dark-mode` + `color-accessible-pairs` (rules-typography-color.md) |
| Animations feel unnatural | `spring-physics` + `easing` + `exit-faster-than-enter` (rules-animation.md) |
| Form UX is poor | `inline-validation` + `error-clarity` + `focus-management` (rules-forms.md) |
| Navigation feels confusing | `nav-hierarchy` + `bottom-nav-limit` + `back-behavior` (rules-navigation.md) |
| Layout breaks on small screens | `mobile-first` + `breakpoint-consistency` (rules-layout.md) |
| Performance / jank | `virtualize-lists` + `main-thread-budget` + `debounce-throttle` (rules-performance.md) |

## Loading Rules

**MANDATORY**: Top 10 rules above (already inline).

**Load by triage result** -- read the specific reference file matching your issue:
- `references/rules-accessibility.md` -- 14 rules
- `references/rules-interaction.md` -- 17 rules
- `references/rules-performance.md` -- 19 rules
- `references/rules-style.md` -- 13 rules
- `references/rules-layout.md` -- 16 rules
- `references/rules-typography-color.md` -- 15 rules
- `references/rules-animation.md` -- 24 rules
- `references/rules-forms.md` -- 31 rules
- `references/rules-navigation.md` -- 26 rules
- `references/rules-charts.md` -- 30 rules

**Before delivery**: Read `references/professional-ui-checklist.md`

**Do NOT** load all reference files at once -- pick the category matching your triage.
