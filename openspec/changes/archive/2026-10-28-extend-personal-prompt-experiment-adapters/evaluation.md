# Evaluation and closeout

## Result lineage

V12 registration binds all 35 predecessor roots (109 hashed files) from V1–V11. Prior reports remain in their original roots:

| Result | Report |
| --- | --- |
| V1 | [report](../../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v1/report.json) |
| V2 | [report](../../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v2/report.json) |
| V3 | [report](../../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v3/report.json) |
| V4 | [report](../../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v4/report.json) |
| V5 | [report](../../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v5/report.json) |
| V6 | [report](../../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v6/report.json) |
| V7 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v7/report.json) |
| V8 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v8/report.json) |
| V9 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v9/report.json) |
| V10 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v10/report.json) |
| V11 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v11/report.json) |
| V12 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v12/report.json), [audit](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v12/audit.ndjson) |
| V13 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v13/report.json), [audit](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v13/audit.ndjson), [registration](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v13/campaign.json) |
| V14 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v14/report.json), [audit](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v14/audit.ndjson), [registration](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v14/campaign.json) |
| V15 | [report](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v15/report.json), [audit](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v15/audit.ndjson), [registration](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v15/campaign.json) |

V8 contributes two verified, prospective screens (isolation, scope). V9 retained 21 valid actor trials, one interrupted slot, 14 unattempted actor slots, and 21 valid Luna assessments; its planning screen did not complete. V10 and V11 were stopped before inference: V10 froze only two fixture contexts; V11's criterion named V10 as the required fresh screen. Their preflight-only schedules remain unattempted and do not count as evidence.

## V12 outcome

- Route: OmniRoute `gpt-6-luna` / medium only. Fresh no-inference preflights passed for 36 actor and 36 assessor contexts.
- Actor screen: 36/36 valid. Assessor screen: 18/36 attempted, 17 valid, 1 invalid, 18 unattempted. Partial assessment: 16 pass, 1 fail, 19 unknown/unjudged.
- Cumulative V1–V12: 533 actor attempts (527 valid, 6 invalid); 423 assessor attempts (422 valid, 1 invalid); 0 confirmed leads; 0 candidate decisions.
- Plateau: **not demonstrated**, two of three preregistered eligible screens. No candidate/confirmation stage became eligible. Spent V9/V12 slots were not replaced.
- Routing remains unsupported: pinned runner exposes only `bash`, `write`, `todo`, lacks matched skill selection and recording callback.
- Continuity remains unsupported: selected `api --stdio` runtime exposes no native compact request/completion event.
- Frozen default/current-personal controls matched their source digests; V12 disposable target was restored. No global prompt, credentials, provider config, shared daemon, or publication target was changed.

## V13 outcome and blocker

- Fresh registration V13 binds all 36 V1–V12 predecessor roots (113 hashed files), cumulative counters, V8's two verified screens, original thresholds, frozen default/current-personal controls, and fresh tier-6 planning fixtures (retry 10, limits 10, mapping 4). Only OmniRoute `gpt-6-luna` / medium was registered.
- V13 fresh **no-inference** actor preflight: all 36 registered contexts checked, zero inference. Frozen controls matched source digests; disposable target matched the personal snapshot.
- Actor: 8/36 attempted, 7 valid, 1 interrupted/invalid, 28 unattempted. The partial no-inference assessment leaves missing/ungraded results unknown; 7 assessor packets were scheduled, 0 submitted.
- Cumulative V1–V13: 541 actor attempts (534 valid, 7 invalid), 423 assessor attempts (422 valid, 1 invalid), 0 confirmed leads. Plateau **not demonstrated**: 2/3 prospective screens. No candidate/confirmation stage eligible.
- V12 assessor slot 018's historical `401 Unauthorized` and OAuth lockout remain preserved; V13 fresh preflight passed after the user reported resolution. No credential/provider changes or repeated V12 slots.
- V13 actor slot 008 hit `TimeoutError: Trial/API watchdog expired` at the registered 180-second limit during tool-input streaming. Evidence: [slot 008](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v13/runs/planning-tier-6-baseline/trials/slot-008.json) and [API trace](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v13/runs/planning-tier-6-baseline/api-trace.ndjson). The slot is spent; no replacement or further campaign inference was submitted. Completion requires a newly registered continuation with fresh fixtures, V1–V13 cumulative bindings, and fresh preflight. The active change remains unarchived and incomplete.
- Independent review used only OmniRoute `gpt-6-luna` / medium, no tools. Four review passes found actionable documentation/status issues; each was repaired and rechecked. Final [review](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v13/independent-review.json) and [pre-commit review](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v13/independent-review-precommit.json) approved with no findings; earlier review outputs are retained beside them.
- V1–V13 registrations, reports, audits, and trial artifacts remain in their existing ignored `target/personal-prompt-eval/` roots; this page links the records without moving or rewriting prior results.

## V14 outcome and blocker

- V14 bound 37 V1–V13 roots (117 hashed files), V8's two verified plateau screens, and cumulative V1–V13 counters. It registered fresh tier-7 planning fixtures (retry 11, limits 11, mapping 5), unchanged thresholds, frozen controls, and only OmniRoute `gpt-6-luna` / medium.
- Fresh no-inference preflight passed all 36 actor contexts; zero inference started during preflight. Frozen controls and disposable target matched their snapshots.
- Actor slot 006 stopped on `Responses request failed: server_error (server_error): stream content stall: no model output within 80000ms (lifecycle/heartbeat events only)`. The exact provider error is retained in [slot 006](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v14/runs/planning-tier-7-baseline/trials/slot-006.json). V14 totals: 6/36 actor attempts, 5 valid but ungraded, 1 invalid, 30 unattempted; no assessor/candidate inference.
- Cumulative V1–V14: 547 actor attempts (539 valid, 8 invalid), 423 assessor attempts (422 valid, 1 invalid), 0 leads. Plateau remains **not demonstrated** at 2/3 eligible screens. Slot 006 is spent; no replacement submission.
- V15 is registered with new tier-8 fixtures, V1–V14 lineage/counters, and a fresh no-inference preflight. This is a new independent screen, not a retry of V14 slot 006.

## V15 outcome

- V15 bound 38 V1–V14 roots (121 hashed files), V8's two valid prospective screens, and cumulative counters. It registered new tier-8 fixtures (retry 12, limits 12, mapping 6) and retained all original thresholds and frozen controls.
- Fresh no-inference preflights passed for actor and assessor, 36 contexts each, only OmniRoute `gpt-6-luna` / medium. No inference ran during preflight.
- Actor: 36/36 valid. Assessor: 36/36 valid. No invalid, interrupted, or unattempted V15 slots. The existing assessor produced a complete semantic assessment.
- V1–V15 cumulative: 583 actor attempts (575 valid, 8 invalid), 459 assessor attempts (458 valid, 1 invalid), 0 confirmed leads. No candidate/confirmation stage was eligible.
- Plateau: **demonstrated within the preregistered screen**, 3/3 eligible planning/isolation/scope screens. This is limited to the registered fixtures and conditions; it is not a global prompt plateau or proof of prompt ineffectiveness.
- Routing and continuity remain unsupported for the exact adapter gaps recorded above. Default/current-personal controls and disposable target match their snapshots; no prompt, credential, provider, daemon, or publication state changed.
- Independent review used only OmniRoute `gpt-6-luna` / medium, no tools. It approved the V15 implementation and registered evidence with no findings; the retained [review](../../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v15/independent-review.json) is logged in V15's audit chain. Its conclusion is limited to the supplied diff and preregistered three-family screen.

## Verification

- `PYTHONPATH=scripts python3 -m unittest test_personal_prompt_eval` — 47 passed.
- `python3 -m py_compile scripts/prepare_personal_prompt_eval.py scripts/test_personal_prompt_eval.py` — passed.
- `openspec validate --type change --strict --no-interactive extend-personal-prompt-experiment-adapters` — valid.
- V13, V14, and V15 registration/preflight/audit evidence is retained; V15 `--verify`, controls, fixtures, and task assessment passed. Independent V15 review approved with no findings; native archive follows the scoped commit.
