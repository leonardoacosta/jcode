# Ratchet Contract

Evaluate every candidate against the last KEEP using the frozen suite and declared metric. The
unmodified target is the initial KEEP baseline. A later candidate becomes KEEP only when it improves
under the metric and satisfies every non-regression, ownership, bloat, diff, and budget gate.

## Decide and restore

- **KEEP:** append an audit row and advance the comparison baseline to the accepted candidate bytes
  and measurements.
- **REVERT:** append an audit row, restore the exact last KEEP target bytes, and verify the restored
  digest before another candidate begins. A reverted candidate never becomes the next baseline.

Treat a tie, missing measurement, failed assertion, metric regression, ownership escape, or limit
violation as REVERT. Do not reinterpret a failed candidate through a secondary observation.
Research, acquisition output, and changing public evidence never participate in the KEEP/REVERT
oracle; only frozen suite measurements and declared limits decide the outcome.

## Persist the audit trail

Each audit row records the run and candidate identities, parent KEEP, target digest before and after
the candidate, frozen suite digest, measurements, budget consumption, decision, rationale, and the
restored digest when applicable. Persist the row for both KEEP and REVERT so recovery can reconstruct
the last valid bytes without relying on transient context.

## Enforce declared limits

Check hard limits before starting a candidate and after receiving its measurements:

- the **elapsed-time budget** bounds total wall-clock duration;
- the **iteration budget** bounds candidate attempts;
- the **token budget** and **cost budget** bound metered evaluation resources;
- the **diff-size budget** bounds cumulative churn, while the per-candidate diff limit bounds one
  experiment; and
- the **bloat limit** rejects growth even when the primary metric improves.

Stop truthfully when a hard budget is exhausted. Do not start a candidate whose reserved work would
exceed a remaining budget.

## Stop on plateau

The plateau limit counts consecutive candidates that do not become KEEP. When that limit is reached,
end the run with a plateau outcome. Continued experimentation, suite repair, or refreshed research
requires a new run with a new descriptor; do not reset the counter inside the existing run.

Automated KEEPs are provisional. At the terminal boundary, a human reviews the cumulative target
diff, suite, audit trail, consumed budgets, plateau or stop reason, and known limitations before
integration.
