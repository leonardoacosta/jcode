# Evaluation and closeout

## Result lineage

V12 registration binds all 35 predecessor roots (109 hashed files) from V1–V11. Prior reports remain in their original roots:

| Result | Report |
| --- | --- |
| V1 | [report](../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v1/report.json) |
| V2 | [report](../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v2/report.json) |
| V3 | [report](../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v3/report.json) |
| V4 | [report](../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v4/report.json) |
| V5 | [report](../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v5/report.json) |
| V6 | [report](../../../target/personal-prompt-eval/2026-10-27-automated-campaign-v6/report.json) |
| V7 | [report](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v7/report.json) |
| V8 | [report](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v8/report.json) |
| V9 | [report](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v9/report.json) |
| V10 | [report](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v10/report.json) |
| V11 | [report](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v11/report.json) |
| V12 | [report](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v12/report.json), [audit](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v12/audit.ndjson) |

V8 contributes two verified, prospective screens (isolation, scope). V9 retained 21 valid actor trials, one interrupted slot, 14 unattempted actor slots, and 21 valid Luna assessments; its planning screen did not complete. V10 and V11 were stopped before inference: V10 froze only two fixture contexts; V11's criterion named V10 as the required fresh screen. Their preflight-only schedules remain unattempted and do not count as evidence.

## V12 outcome

- Route: OmniRoute `gpt-6-luna` / medium only. Fresh no-inference preflights passed for 36 actor and 36 assessor contexts.
- Actor screen: 36/36 valid. Assessor screen: 18/36 attempted, 17 valid, 1 invalid, 18 unattempted. Partial assessment: 16 pass, 1 fail, 19 unknown/unjudged.
- Cumulative V1–V12: 533 actor attempts (527 valid, 6 invalid); 423 assessor attempts (422 valid, 1 invalid); 0 confirmed leads; 0 candidate decisions.
- Plateau: **not demonstrated**, two of three preregistered eligible screens. No candidate/confirmation stage became eligible. Spent V9/V12 slots were not replaced.
- Routing remains unsupported: pinned runner exposes only `bash`, `write`, `todo`, lacks matched skill selection and recording callback.
- Continuity remains unsupported: selected `api --stdio` runtime exposes no native compact request/completion event.
- Frozen default/current-personal controls matched their source digests; V12 disposable target was restored. No global prompt, credentials, provider config, shared daemon, or publication target was changed.

## External blocker

V12 assessor slot 018 failed at `https://omni.leonardoacosta.dev/v1/responses` with `401 Unauthorized`: the `cx/gpt-6-luna` upstream reported an invalidated OAuth token. OmniRoute set a model lockout until `2026-10-28T21:53:04.480Z`; 18 assessor slots remain unattempted. The exact trial is retained at [slot 018](../../../target/personal-prompt-eval/2026-10-28-automated-campaign-v12/runs/planning-tier-5-baseline-assessor/trials/slot-018.json). The resolver is re-authorization of the existing OAuth provider connection by its owner at the OmniRoute provider dashboard. Credential changes are outside this request; no replacement or further inference was submitted.

The same route is required for independent model review. No alternate model was used; independent review and archive are blocked pending restored authorized Luna access and a newly registered continuation. The active change remains unarchived and incomplete.

## Verification

- `PYTHONPATH=scripts python3 -m unittest test_personal_prompt_eval` — 47 passed.
- `python3 -m py_compile scripts/prepare_personal_prompt_eval.py scripts/test_personal_prompt_eval.py` — passed.
- `openspec validate --type change --strict --no-interactive extend-personal-prompt-experiment-adapters` — valid.
- V12 `--verify`, predecessor audit, control/target checks, and `git diff --check` — passed.
