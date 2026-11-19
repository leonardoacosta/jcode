## Context

The fork's current `main` is `dc00085917c130dccce7fd423c1495a44231f4cb`; the approved upstream v1.0.1 target is `10114429c8133994861d6c915a2483d397c0f7c2`. The user approved an in-place full-history merge on `main`, local commits, swarm use where useful, independent review, and an immutable RC tested in isolation before green-only promotion. This change plans the work; it does not perform the merge or activation.

## Decisions

1. **Preserve full ancestry.** Merge the exact upstream target with a normal merge commit. Do not squash, rebase, cherry-pick as a substitute, reset, force-push, or rewrite history. Retain `main` in place.
2. **Adopt upstream lifecycle ownership.** Use upstream agent/session binding, HTTP compression transport, dedicated swarm page, and report coalescing. Remove redundant fork-only inline swarm-strip controls. Preserve full reports, report-completion semantics, orphan explicit retry, and existing stop/owner safety. `make-worker-stop-durable` acceptance is unfinished and outside this change's implementation scope.
3. **Retire automatic model fallback explicitly.** Remove the custom `model_fallback` config and automatic model-switch chain where upstream supersedes it. Keep shared auth/quota classification and all no-substitution safeguards. Do not disposition/archive fallback as a shortcut. Its current runtime implementation and tests are evidence to reconcile, not authority to retain the behavior.
4. **Keep compression endpoint-owned.** Adopt upstream HTTP compression transport. Remove Jev-specific compression/cache overrides superseded by upstream, while preserving complete reports and review-output semantics. The separate native external-gateway acceptance gate stays open; local transport tests cannot close it.
5. **Preserve bounded custom behavior.** Retain thin named-provider session affinity, long but bounded gateway `Retry-After`, useful custom council/file-claim/role-slot behavior, remote routing, `/links`, and `/mouse`. Remote clipboard uses OSC 52 only unless `JCODE_CLIPBOARD=native` explicitly selects the native path.
6. **Parallelize safely.** Delegate only non-overlapping file ownership after freezing the base. The coordinator owns merge state and shared files. Resolve overlapping work serially and integrate every worker's changes before validation. Independent review must inspect the complete final diff and change requirements after fresh gates.
7. **Test the exact RC.** Build one immutable, content-identified RC. Run required unit/integration checks and isolated private-socket/runtime acceptance against that exact binary and isolated config/state. If any required gate fails or is unavailable, do not promote. Any repair creates a new RC identity and reruns affected and final gates.
8. **Keep external gates separate.** Physical-device remote clipboard testing and actual external gateway acceptance remain distinct, open acceptance items. No mock, synthetic transport, or one gate's green result closes the other.

## Execution sequence

- Reconfirm both exact refs, merge ancestry, clean/dirty ownership, required build/test commands, configured identity, and all acceptance gates before source work. Preserve unrelated dirty and untracked files.
- Integrate upstream history in place, then reconcile the approved behavior deltas by ownership group. Record conflict decisions and keep attribution.
- Run targeted tests for merge-adjacent behavior, then full required project checks. Run strict OpenSpec and dependency checks for the selected change closure.
- Produce a uniquely identified immutable RC only after source integration. Test that artifact through isolated runtime and no shared-daemon activation.
- Obtain independent review of final candidate after gates. Fix findings, mint a new immutable RC, and rerun affected gates/review.
- Promote only the exact reviewed, fully green RC through the authorized local self-dev promotion path. Preserve the preceding active binary for rollback; promotion is not deployment or external publication.

## Rollback and stop conditions

Do not undo or rewrite the merge history. If an RC fails, retain the failed artifact and evidence, repair in a new commit, and issue a new RC identity. Do not promote if any required test, independent review, external gateway gate, physical-device gate, identity, or isolated-target prerequisite is missing. External gateway and physical-device gates may remain open independently of source merge progress, but they block any completion claim that requires their acceptance.
