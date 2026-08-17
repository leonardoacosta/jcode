
# Figma Design Validation with MCP

Design-vs-implementation validation via Figma MCP. Governed by this skill's
[SKILL.md](../SKILL.md) § "When a Manual Test Plan Is the Right Tool" — a Figma comparison is
inherently manual-judgment work (Rule 11's visual-regression baseline catches pixel drift between
CI runs, not "does this match design intent"), so it stays a manual template even in a fleet that
already runs automated visual regression.

---

## Prerequisites

Figma MCP server configured, API token set, access to the target design file confirmed.

---

## Workflow

1. **Get design specs via Figma MCP** — query dimensions, colors, typography, spacing, border
   radius, and interactive states for the target component/page.
2. **Inspect the implementation** — browser DevTools: computed styles, dimensions, color picker,
   typography panel, interactive-state testing.
3. **Document discrepancies** in a test case or bug report using the templates below.

**Example query set:**
```
"Get complete specifications for the [component] from Figma at [URL]"
"Extract spacing values for the card component from Figma"
"Get typography specifications for all heading levels from Figma design system"
"List all color tokens used in the navigation component"
"What are the defined breakpoints in this Figma design?"
```

## What to Compare

Layout & spacing (dimensions, padding, margin, grid alignment, breakpoints) — Typography (family,
size, weight, line-height, letter-spacing, color) — Colors (background, text, border, shadow,
gradient, opacity) — Component states (default, hover, active/pressed, focus, disabled, loading,
error).

---

## Test Case Template

```markdown
## TC-UI-XXX: [Component] Visual Validation

**Figma Design:** [URL to specific component]

### Desktop (1920x1080)

**Layout:**
- [ ] Width: XXXpx
- [ ] Height: XXXpx
- [ ] Padding: XXpx XXpx XXpx XXpx
- [ ] Margin: XXpx

**Typography:**
- [ ] Font: [Family] [Weight]
- [ ] Size: XXpx
- [ ] Line-height: XXpx
- [ ] Color: #XXXXXX

**Colors:**
- [ ] Background: #XXXXXX
- [ ] Border: Xpx solid #XXXXXX
- [ ] Shadow: XXpx XXpx XXpx rgba(X,X,X,X)

**Interactive States:**
- [ ] Hover: [changes]
- [ ] Active: [changes]
- [ ] Focus: [changes]
- [ ] Disabled: [changes]

### Tablet (768px)
- [ ] [Responsive changes]

### Mobile (375px)
- [ ] [Responsive changes]

### Status
- [ ] PASS - All match
- [ ] FAIL - Discrepancies found
- [ ] BLOCKED - Design incomplete
```

---

## Bug Report for UI Discrepancies

```markdown
# BUG-XXX: [Component] doesn't match Figma design

**Severity:** Medium (UI)
**Type:** Visual

## Design vs Implementation

**Figma Design:** [URL]

**Expected (from Figma):**
- Button background: #0066FF
- Font weight: 600 (Semi-bold)
- Padding: 12px 24px

**Actual (in implementation):**
- Button background: #0052CC ❌
- Font weight: 400 (Regular) ❌
- Padding: 12px 24px ✓

## Screenshots

- Figma design: [attach]
- Current implementation: [attach]
- Side-by-side comparison: [attach]

## Impact

Users see inconsistent branding. Button appears less prominent than designed.
```

---

## Automation Boundary

Visual-regression tools (Percy, Chromatic, BackstopJS) catch pixel drift between builds — they do
not validate design *intent*. Design-token-vs-CSS-variable diffing can automate part of the color
check but not layout/typography judgment calls. Neither replaces this manual comparison.

---

## Quick Reference

| Element | What to Check | Tool |
|---------|---------------|------|
| Colors | Hex values exact | Browser color picker |
| Spacing | Padding/margin px | DevTools computed styles |
| Typography | Font, size, weight | DevTools font panel |
| Layout | Width, height, position | DevTools box model |
| States | Hover, active, focus | Manual interaction |
| Responsive | Breakpoint behavior | DevTools device mode |
