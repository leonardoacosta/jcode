# Proposal verification

Production code unchanged. All implementation and runtime checks in tasks.md are prospective.

Observed source: question overlay early return, existing composer allocation, server pending-question lock, existing overnight manifest and event structures. The earlier native-tool proposal still has unchecked tasks despite present implementation, so its completion status is not used as evidence.

Live Jev MCP layout evaluation attempted 2026-09-24. Result: HTTP 404. No recommendation probabilities were obtained. Composer recommendation is Jcode-authored.

Acceptance mapping:
- question-composer-pane / Integrated question pane and Keyboard and focus continuity: A1-A3.
- question-recommendations / Truthfully labeled asynchronous advice: B1-B3.
- overnight-question-decisions / Explicit bounded delegation: C1, C4.
- overnight-question-decisions / Server-owned inactivity and resolution: C2, C4.
- overnight-question-decisions / Durable accountable automatic decisions: C3-C4.

Approval required for implementation. Proposed thresholds are policy defaults, not empirically calibrated probabilities. MCP live availability and a trustworthy eligibility boundary must pass their implementation gates before automation ships.

Authoring checks passed: `openspec validate question-pane-and-overnight-advice --strict --no-interactive`; scoped `git diff --check`; no TBD/TODO markers. Static HTML comparison contains both source-grounded wireframes and is HTML-escaped. Browser rendering was not tested. No runtime tests were run because this change only defines the proposal.

## Observed check matrix (authoring follow-through)

| Requirement or public output | Concrete check | Observed result |
| --- | --- | --- |
| Proposal packaging | Public `openspec validate ... --strict --no-interactive` and `openspec show ...` | PASS: valid change and readable proposal |
| Integrated question pane | Inspect renderer/composer integration points, check WHEN/THEN scenarios and A1-A3 mapping | PASS authoring checks. Runtime NOT TESTED, approval required |
| Keyboard and focus continuity | Inspect existing question state/input handler, check scenario structure and A2-A3 coverage | PASS authoring checks. Runtime NOT TESTED |
| Truthfully labeled asynchronous advice | Live Jev MCP layout comparison, scenario checks and B1-B3 mapping | BLOCKED live success: HTTP 404. Failure documented without invented verdict. Future UI NOT TESTED |
| Explicit bounded delegation | Check per-run opt-in, exclusions, migration defaults and C1/C4 dependency | PASS specification inspection. Policy enforcement NOT TESTED |
| Server-owned inactivity and resolution | Check 300-second origin, activity distinction, races/lifecycle scenarios and C2/C4 mapping | PASS specification inspection. Timer and races NOT TESTED |
| Durable accountable automatic decisions | Check pre-delivery persistence, failure/crash/report scenarios and C3/C4 mapping | PASS specification inspection. Persistence/report behavior NOT TESTED |
| Ordered executable task list | Script verifies all 11 task IDs have explicit depends clauses | PASS |
| Before/after HTML comparison | Script checks both panels, viewport metadata, HTTP 404 disclosure and approval choices | PASS content checks. Browser rendering NOT TESTED |
| All six requirement definitions | Script checks each has scenarios, each with WHEN and THEN | PASS |

No automated continuation message grants design approval. Additional product acceptance evidence depends on implementing the approved stages. This matrix records the distinction rather than treating prospective scenarios as passing tests.

## Approved policy revision, 2026-09-24
The user approved all stages and changed the new-overnight-run default to enabled, with prompt-driven opt-out. Earlier opt-in and approval-pending observations above are historical. Old manifests remain disabled to avoid retroactive activation. The new User prompt opt-out requirement maps to C1/C2/C4: test no-preference default, direct launch/later opt-out, scope ambiguity, resolver failure, untrusted quoted text, reconnect persistence, explicit re-enable and timer races. These runtime checks remain NOT TESTED until implementation. Current authoring check: strict OpenSpec validation after revision.
