# Issue Taxonomy

Reference for categorizing issues found during dogfooding. Read this at the start of a dogfood session to calibrate what to look for.

## Contents

- [Severity Levels](#severity-levels)
- [Categories](#categories)
- [Exploration Checklist](#exploration-checklist)

## Severity Levels

| Severity | Definition |
|----------|------------|
| **critical** | Blocks a core workflow, causes data loss, or crashes the app |
| **high** | Major feature broken or unusable, no workaround |
| **medium** | Feature works but with noticeable problems, workaround exists |
| **low** | Minor cosmetic or polish issue |

Severity follows **user impact**, not how technically alarming the symptom looks. A console exception with no observable consequence is not automatically high severity. A plain-looking validation defect that permits data loss may be critical.

Use the highest level whose full definition is satisfied. Do not inflate severity to make a report appear consequential.

## Finding Qualification

A reportable finding needs all three:

1. **Observable symptom** — a user-visible failure, accessibility barrier, misleading state, or action-linked console/network error.
2. **Defensible expectation** — supported by the UI's labels and patterns, an explicit requirement, established platform convention, or accessibility behavior.
3. **Repeatable evidence** — reproduced at least once after discovery from a known starting state.

Do not report:

- Personal taste without a usability or consistency consequence.
- Expected authorization failures, validation messages, or intentionally disabled controls.
- Third-party browser-extension noise or unrelated background requests.
- One transient failure that cannot be reproduced. Mention it only as an unconfirmed observation in the final summary if it materially affected testing.
- Multiple issues for the same root symptom on adjacent pages. Document one issue and list all confirmed affected locations.

## Categories

### Visual / UI

- Layout broken or misaligned elements
- Overlapping or clipped text
- Inconsistent spacing, padding, or margins
- Missing or broken icons/images
- Dark mode / light mode rendering issues
- Responsive layout problems (viewport sizes)
- Z-index stacking issues (elements hidden behind others)
- Font rendering issues (wrong font, size, weight)
- Color contrast problems
- Animation glitches or jank

### Functional

- Broken links (404, wrong destination)
- Buttons or controls that do nothing on click
- Form validation that rejects valid input or accepts invalid input
- Incorrect redirects
- Features that fail silently
- State not persisted when expected (lost on refresh, navigation)
- Race conditions (double-submit, stale data)
- Broken search or filtering
- Pagination issues
- File upload/download failures

### UX

- Confusing or unclear navigation
- Missing loading indicators or feedback after actions
- Slow or unresponsive interactions (>300ms perceived delay)
- Unclear error messages
- Missing confirmation for destructive actions
- Dead ends (no way to go back or proceed)
- Inconsistent patterns across similar features
- Missing keyboard shortcuts or focus management
- Unintuitive defaults
- Missing empty states or unhelpful empty states

### Content

- Typos or grammatical errors
- Outdated or incorrect text
- Placeholder or lorem ipsum content left in
- Truncated text without tooltip or expansion
- Missing or wrong labels
- Inconsistent terminology

### Performance

- Slow page loads (>3s)
- Janky scrolling or animations
- Large layout shifts (content jumping)
- Excessive network requests (check via console/network)
- Memory leaks (page slows over time)
- Unoptimized images (large file sizes)

### Console / Errors

- JavaScript exceptions in console
- Failed network requests (4xx, 5xx)
- Deprecation warnings
- CORS errors
- Mixed content warnings
- Unhandled promise rejections

### Accessibility

- Missing alt text on images
- Unlabeled form inputs
- Poor keyboard navigation (can't tab to elements)
- Focus traps
- Insufficient color contrast
- Missing ARIA attributes on dynamic content
- Screen reader incompatible patterns

## Exploration Checklist

Use this as a guide for what to test on each page/feature:

1. **Visual scan** -- Take an annotated screenshot. Look for layout, alignment, and rendering issues.
2. **Interactive elements** -- Click every button, link, and control. Do they work? Is there feedback?
3. **Forms** -- Fill and submit. Test empty submission, invalid input, and edge cases.
4. **Navigation** -- Follow all navigation paths. Check breadcrumbs, back button, deep links.
5. **States** -- Check empty states, loading states, error states, and full/overflow states.
6. **Console** -- Check for JS errors, failed requests, and warnings.
7. **Responsiveness** -- If relevant, test at different viewport sizes.
8. **Auth boundaries** -- Test what happens when not logged in, with different roles if applicable.
9. **Persistence** -- Refresh, revisit, and use back/forward navigation after meaningful state changes.
10. **Destructive boundaries** -- Verify warning, cancellation, and confirmation behavior without completing an unauthorized irreversible action.
11. **Evidence integrity** -- Confirm the URL, expected/actual result, and evidence files before counting a finding.
