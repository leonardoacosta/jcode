# Harness Adapter Contract

An adapter may choose its own durable descriptor representation, but it records these frozen fields
before the first candidate mutation:

- `run_id`
- `target_identity`
- `target_baseline_digest`
- `suite_identity`
- `suite_digest`
- `adapter_name`
- `adapter_version`
- `metric`
- `non_regression_rule`
- `keep_baseline`
- `bloat_limit`
- `diff_limit`
- `elapsed_time_budget`
- `iteration_budget`
- `token_budget`
- `cost_budget`
- `diff_budget`
- `plateau_limit`
- `human_review_required`

When optional research is used, also freeze:

- `recon_record_id`
- `recon_manifest_digest`
- `recon_finding_ids`

The adapter provides outcomes rather than prescribing a universal implementation. It must:

1. isolate the selected target and expose only the authorized atom for candidate mutation;
2. execute the frozen suite without changing its inputs;
3. report measurements and consumption required by the metric, non-regression, bloat, diff,
   elapsed-time, iteration, token, and cost gates;
4. persist an audit record for every candidate;
5. restore the exact last KEEP bytes after REVERT or interrupted candidate work; and
6. produce a terminal handoff suitable for human review.

If isolation, suite freezing, restoration, or durable audit evidence is unavailable, refuse to
claim a conforming run. Adapter-native mechanics remain outside the portable method.

## Prepare immutable evidence before a run

Research is optional, and Firecrawl absence is nonfatal. A valid no-research run omits all optional
Recon fields and still uses the complete frozen suite and ordinary ratchet contract.

When research is useful:

1. Issue a canonical Recon query using the smallest relevant combination of source, finding, tag,
   normalized text, acquisition, tool, and inclusive capture-date filters.
2. Review record currency, coverage, uncertainty, and limitations. If evidence is missing or stale,
   the operator may request acquisition through Recon and Firecrawl for public evidence.
3. Verify the resulting canonical v2 record and manifest before using it. Raw acquisition state is
   not a durable record and does not belong in the run descriptor.
4. Freeze the verified `recon_record_id`, `recon_manifest_digest`, and selected finding IDs before
   candidate mutation. Use finding IDs to support hypotheses or preparation of a new suite.
5. Freeze the suite only after any finding-based preparation is complete.

Do not refresh evidence, change selected findings, call acquisition, or edit the suite after
candidate mutation begins. Fresher evidence, post-plateau research, or a repaired suite requires a
new canonical record when applicable and a new run. Firecrawl output and live web state never enter
the KEEP/REVERT oracle; only frozen suite measurements do.
