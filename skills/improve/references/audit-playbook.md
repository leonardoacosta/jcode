# Audit Playbook

Use this playbook to gather evidence, not to generate generic advice. Adapt depth to repository
size and the user's requested effort. A finding without a precise code or configuration location
is a lead, not a confirmed finding.

## 1. Correctness and bugs

- Inspect swallowed errors, incomplete error states, unawaited work, races, missing cleanup, and
  retry paths that are not idempotent.
- Trace nullable values, unchecked indexes, boundary conditions, timezone and locale assumptions,
  and state-machine branches that silently accept impossible states.
- Look for transactions missing around related writes, resource leaks, and clusters of type-system
  escape hatches.

## 2. Security

- Inspect input boundaries, authorization at the operation and object level, path handling,
  subprocess arguments, deserialization, redirects, request forwarding, and output escaping.
- Check secret storage, logging, rotation expectations, dependency advisories, and production
  security controls such as cookie flags, CORS, CSP, and rate limits.
- Describe defensive remediation and tests. Do not include exploit payloads or secret values.

## 3. Performance

- Look for repeated I/O or queries in loops, unbounded reads, missing pagination, duplicated work,
  synchronous blocking on hot paths, retained resources, and avoidable large bundles.
- Require evidence that the path is meaningful. Do not propose caching without an invalidation
  strategy or optimization without a measurable gate.

## 4. Test coverage

- Find important behavior with no coverage, especially permissions, money, migrations, parsing,
  destructive operations, retry logic, and negative paths.
- Inspect disabled, flaky, over-mocked, or assertion-free tests and gaps between documented test
  commands and CI.
- Prefer tests at the cheapest layer that proves the behavior. If the baseline is broken, route
  its repair ahead of risky changes.

## 5. Architecture and technical debt

- Look for divergent duplication, circular or inverted dependencies, high-fan-in junk drawers,
  dead code, oversized modules, and several competing patterns for the same responsibility.
- Distinguish intentional boundaries from accidental complexity by checking ADRs, specs, and
  recent convergence in the codebase.
- Avoid abstraction work unless multiple concrete call sites need the same change.

## 6. Dependencies and migrations

- Check supported runtime ranges, end-of-life frameworks, deprecated APIs, abandoned critical
  dependencies, duplicated libraries, and lockfile or manifest drift.
- Estimate blast radius and required migration sequencing. Minor version lag alone is not a
  high-value finding.

## 7. Developer experience and tooling

- Verify that documented setup, lint, typecheck, test, build, and release paths actually exist and
  agree with automation.
- Look for slow feedback loops, missing caches, unclear environment requirements, unhelpful errors,
  and repository guidance that leaves executors guessing about conventions or verification.

## 8. Documentation

- Prioritize public APIs without usable references, actively wrong setup or operational guidance,
  and consequential architectural decisions that cannot be reconstructed.
- Do not recommend documentation merely to increase coverage; name the concrete cost of absence.

## 9. Direction

Ground every product option in repository evidence:

- unfinished intent such as TODO clusters, stubs, or dormant flags;
- stated but undelivered behavior in product documents or public surfaces;
- asymmetric capabilities such as export without import;
- an adjacent capability made unusually cheap by existing architecture;
- repeated user workarounds that the product could absorb.

Treat direction as options for maintainers, not defects. Explain intended users, tradeoffs, coarse
effort, and why the evidence makes the option timely. Prefer a research or design-spike route when
the product decision is unsettled.

## Finding format

Return each audit finding in this shape:

```markdown
### [CATEGORY-NN] Short imperative title

- **Evidence**: `path/file.ts:123` — what the code does at this location.
- **Impact**: the concrete failure, cost, or user consequence.
- **Effort**: S | M | L, including tests.
- **Risk**: LOW | MED | HIGH, with the likely blast radius.
- **Confidence**: HIGH | MED | LOW, based on direct verification.
- **Fix sketch**: one to three sentences, or an investigation boundary for LOW confidence.
```

## Prioritization

Rank by impact divided by effort, discounted by confidence and change risk. Put prerequisites
first, then high-confidence security findings, then findings with clean verification stories.
“Not worth doing” is a valid conclusion when the evidence does not support the cost.
