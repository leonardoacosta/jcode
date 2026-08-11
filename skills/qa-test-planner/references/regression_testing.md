
# Regression Testing Guide

Suite structure, prioritization, and a report template for regression testing. Governed by this
skill's [SKILL.md](../SKILL.md) § "When a Manual Test Plan Is the Right Tool" — that section
decides whether a regression pass is worth running manually at all vs. relying on this repo's
automated e2e suite; this file is the artifact generator once that call is made.

---

## Suite Types

| Suite | Duration | When | Coverage |
|---|---|---|---|
| Smoke | 15-30 min | Daily, before detailed testing | Critical paths, core functionality, build stability |
| Targeted | 30-60 min | After specific changes | Modified feature area + integration points |
| Full | 2-4 hours | Before releases, weekly | All functional cases, integration, UI, data integrity, security |
| Sanity | 10-15 min | After hotfix | Quick validation |

**Example smoke suite:**
```
SMOKE-001: User can login
SMOKE-002: User can navigate to main features
SMOKE-003: Critical API endpoints respond
SMOKE-004: Database connectivity works
SMOKE-005: User can complete primary action
SMOKE-006: User can logout
```

---

## Building a Suite: Priority + Grouping

**Prioritize:** P0 (business-critical, security, data integrity, revenue) -> P1 (major features,
common flows, integration points) -> P2 (minor features, edge cases, UI polish).

**Group by feature area**, e.g.:
```
Authentication & Authorization
├─ Login/Logout
├─ Password reset
├─ Session management
└─ Permissions
```

**Execution order:** smoke first (stop on failure) -> P0 -> P1/P2 -> exploratory.

**Pass/fail:** PASS = all P0 pass + 90%+ P1 pass + no open criticals. FAIL = any P0 fails, a
critical bug surfaces, or data loss occurs. CONDITIONAL PASS = P1 failures with a documented
workaround and fix plan in place.

---

## Regression Test Execution Report

```markdown
# Regression Test Report: Release 2.5.0

**Date:** 2024-01-15
**Build:** v2.5.0-rc1
**Tester:** QA Team
**Environment:** Staging

## Summary

| Suite | Total | Pass | Fail | Blocked | Pass Rate |
|-------|-------|------|------|---------|-----------|
| Smoke | 10 | 10 | 0 | 0 | 100% |
| P0 Critical | 25 | 23 | 2 | 0 | 92% |
| P1 High | 50 | 47 | 2 | 1 | 94% |
| P2 Medium | 40 | 38 | 1 | 1 | 95% |
| **TOTAL** | **125** | **118** | **5** | **2** | **94%** |

## Critical Failures (P0)

### BUG-234: Payment processing fails for Visa
- **Test:** TC-PAY-001
- **Impact:** High - Blocks 40% of transactions
- **Status:** In Progress
- **ETA:** 2024-01-16

## Recommendation

**Status:** CONDITIONAL GO
- Fix BUG-234 (payment) before release
- Retest after fixes
- Final regression run before production deployment
```

---

## Maintenance

Review monthly: remove obsolete tests, update changed functionality, add new critical paths. After
each release: refresh test data, fix broken tests, add regression coverage for bugs found.
Automate stable/repetitive tests (smoke, API, data validation); keep exploratory, usability, and
visual-design validation manual.
