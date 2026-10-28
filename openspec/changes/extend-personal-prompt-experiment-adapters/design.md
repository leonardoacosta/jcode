# Design

## Context

See [proposal.md](proposal.md). The evaluator in `scripts/prepare_personal_prompt_eval.py` freezes candidate banks and trials, but generates only two baseline tiers. Its custom tool set is fixed to `bash,write,todo`; its callback accepts only `bash` and `write`. `Request::Compact` is present in the client wire protocol, but `jcode-harness-api::RuntimeCommand` and its event contract do not expose a compact request/completion pair. These are concrete adapter gaps; this change will preserve routing and continuity as unsupported and record these exact reasons rather than claim simulated behavior.

V7's `report.json` records all 144 actor and assessor slots valid, three supported families baseline-sufficient, zero leads, and routing/continuity unsupported. Its registration binds only a descriptor digest; its prior-root integrity check was added after execution. Prior reports V1–V7 together record 367 actor attempts, 362 valid, 5 invalid, and zero leads. The new registration will bind all seven roots and recompute these historical counters without treating them as prospective plateau observations.

## Goals / Non-Goals

**Goals:** Add one prospective screen over three independently authored contexts for each supported family; retain the original thresholds and candidate/confirmation rules; make a local, bounded plateau claim only after every preregistered screen is valid and no family meets its existing signal threshold; bind all predecessor evidence and cumulative counters before inference.

**Non-Goals:** Do not alter the Jcode product API to manufacture adapter support; do not claim a global prompt plateau, candidate benefit, adoption, publication, or production superiority; do not reopen or replace any spent prior slot.

## Decisions

1. **Record exact adapter blockers and exclude those families before inference.** `prepare_child` freezes `bash,write,todo`, `run_pilot` rejects any other custom callback, and callbacks run without network access. The API compact request exists in `crates/jcode-protocol/src/wire.rs`, but the selected harness surface (`crates/jcode-harness-api/src/runtime_commands.rs`, `requests.rs`, and `events.rs`) does not forward it or expose its completion. A product API change is outside this evaluator-only successor; unsupported dispositions remain terminal and explicit.
2. **Use three fresh, independently authored contexts per supported family.** Add distinct planning, isolation, and scope fixtures, each with challenge and useful-action counterexample cases. Freeze their files, references, oracles and prompts before running. Keep the original 3-repetition, paired-control schedule. Only a replicated signal under the existing threshold can open the existing candidate bank; preserve the original practical-effect, boundary, counterexample, and confirmation thresholds.
3. **Define a local evidence plateau prospectively.** After all three contexts in each supported family have complete valid paired actor and assessor evidence, if no family meets the already registered baseline signal threshold, report `evidence-plateau-within-registered-screen`. Invalid or unknown evidence cannot count. This bounded result describes the new fixture domain only; it is not a global plateau or proof of prompt ineffectiveness.
4. **Bind lineage and counters into registration.** Before inference, hash the complete V1–V7 root inventories and bind each root's registration, report, and audit. Compute historical attempted/valid/invalid slots, family dispositions, candidate decisions, and leads from those artifacts. Keep prior counters intact and separately initialize prospective plateau evidence at zero; prior baseline-sufficiency labels do not satisfy the new rule.
5. **Keep controls and runtime isolated.** Reuse the existing pinned Luna/medium runner, per-slot private session and networkless callback. Snapshot default/current-personal controls and the selected disposable target; refuse drift. Each candidate continues to start from the original personal bytes. Any candidate/confirmation child receives its own fresh no-inference preflight.

## Risks / Trade-offs

- [New fixtures may fail to expose a signal] → Report a plateau only within the exact registered suite; retain task/context denominators and limitations.
- [Historical report schemas differ] → Recompute counters from verified roots using schema-aware parsing; fail registration if a root or required count is ambiguous.
- [Repeated provider failures consume slots] → Preserve the existing stop rule, invalid counts and spent-slot markers; do not replace or reset.
- [Unsupported adapters could be mistaken for negative results] → Report unsupported separately and exclude those families from the plateau claim.

## Migration Plan

Keep V1–V7 unchanged. Extend the existing runner compatibly, add offline tests and fresh independent review, then register a new private campaign with predecessor hashes/counters frozen. Run its no-inference preflight before any actor or assessor inference. Complete only the supported registered schedule, assess retained evidence, verify controls/lineage, update tasks and commit scoped changes. Archive this change only after required gates, review, evidence persistence and history checks pass. No prompt adoption or shared-daemon action is part of this plan.
