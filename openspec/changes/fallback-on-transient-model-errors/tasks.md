# Tasks

## Retirement contract

- [ ] 1.1 Compare the exact upstream v1.0.1 change and fork implementation/tests for `model_fallback`; record the superseding upstream behavior and every fork-only config, execution, cooling/cycle, and transcript surface to remove. Do not infer removal from file overlap alone.
- [ ] 1.2 Remove the custom automatic model-fallback config and runtime chain only as part of the approved merge reconciliation. Retain shared provider failure classification and supported user-directed recovery. Verify no automatic model/provider/account/gateway substitution remains on the retired path.
- [ ] 1.3 Add or update regression tests proving the retired chain cannot switch models automatically and confirmed quota/billing exhaustion, auth failure, failed refresh transport, selected route, and account identity retain their safeguards. Run the owning package tests and record exact command/results.
- [ ] 1.4 Reconcile references and dependencies in active proposals that assume this fallback chain exists. Do not modify canonical specs, archived artifacts, or roadmap in this change lane.
- [ ] 1.5 Run `openspec validate fallback-on-transient-model-errors --strict` and `git diff --check`. Keep external gateway acceptance and the separate full integration review gates open unless their own evidence is supplied.

## Historical implementation evidence (not retirement completion)

The checked tasks below record the previous feature implementation only. They do not satisfy tasks 1.1-1.5 or authorize retaining the behavior. The former transcript-notice integration and shared-classification acceptance were incomplete; the retirement remains pending until the upstream reconciliation and safeguards above pass.

- Historical 1.1 config addition, 1.2 classifier, 2.1-2.3 health/cooling/chain, 3.1-3.3 turn integration/manual offer, and 4.1/4.3 unit/strict validation are retained in the repository history. Previous task 3.2 transcript output was partial; previous task 4.2 integration was incomplete. These prior marks describe old implementation work only.
- Historical task 5.1 was not accepted. Its checks remain relevant as safety regression cases, not as evidence that automatic fallback should remain.
