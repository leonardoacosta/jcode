# Tasks

Apply authorized by the user on 2026-11-10; this supersedes the earlier planning-only boundary, not the acceptance gates. Execute groups in order, with 1.2 before code changes, 7.1 before final review, and terminal steps in group 8 strictly ordered. Do not act on historical workers, the shared daemon or the original application checkout.

## Execution preflight, 2026-11-10 13:46–13:51 UTC

- Repository `/home/nyaptor/.jcode`, origin `leonardoacosta/jcode`, HEAD `b37c875f1`; user explicitly retains `main` despite remote HEAD `dev`. Configured Git identity verified. Local-only commits and eventual verified archive authorized; no push, merge, deployment or shared-daemon activation. Existing CI/workflow deletions are intentional per the user and must remain untouched, not restored or included in scoped commits.
- Canonical dependency validator with explicit `--root /home/nyaptor/.jcode` passed selected `classify-provider-auth-failures verify-terminal-capability-matrix make-worker-stop-durable`: valid, no issues, closure size 3. Consumer execution remains gated on foundation acceptance and each consumer's additional prerequisites. No tracker is linked or required.
- Selected requirements map remains the table below. Fresh actual-binary reproduction 1.2 is the first implementation prerequisite. Existing tests/e2e public socket and private-binary fixtures are available. Native debug control currently refuses requests; do not enable it on the shared daemon. Use isolated runtime only. Independent GPT 6.1 Sol review is required after final gates; route listing alone is not evidence of successful reviewer access.
- Dirty work and untracked proposal/research files were observed before editing. Preserve earlier source hunks and all unrelated paths. Exact bounded worker ownership separates OAuth runtime, terminal fallbacks and durable-stop baseline fixtures; coordinator serializes Git and shared schema changes. Snapshot/check target baselines before each edit. Source/private artifacts must not be disclosed to new external providers.
- Completion reference resolved at discovered `priceless-development/references/completion.md` and read directly; no broader Priceless policy loaded. `preflight-check-v2-shadow`, initial-dispatch: G1=2 (explicit approved scope), G2=1 (actual baseline and final review access not yet demonstrated), G3=2 (settled main/local-only policy), G4=2 (private runtime target), G5=2 (bounded ownership and preserved dirty baseline), G6=1 (foundation acceptance and consumer gates remain unproved). Advisory 10/12=83.33, not permission or acceptance. Eligible now: baseline reproduction and independent OAuth/terminal implementation. Durable control implementation waits for 1.2; archive waits for all terminal gates.

## 1. Bind isolation and reproduce the current failure

- [ ] 1.1 Recheck repository identity, source/binary hashes, dirty files, configured Git identity, exact-file ownership and existing `add-swarm-handoff-contracts` overlap; record results and resolve branch-policy/default uncertainty before committing. Verify no unrelated paths are staged or changed, and refresh the handoff preflight over the whole selected path.
- [x] 1.2 Add a focused baseline regression through existing `tests/e2e/` and public daemon/swarm interfaces, using a unique socket, private HOME/JCODE_HOME/runtime/install directories, bounded local deterministic inference and no external credentials. Verify the serving binary identity, reproduce continued execution or old-report completion after stop on the baseline, and freeze exact commands, configured deadlines and marker/generation observations. If not reproduced, investigate this exact discrepancy and update evidence before choosing a remediation, not acceptance criteria.
- [ ] 1.3 Document the isolated acceptance entry point and cleanup in this change's evidence record, adapting existing `tests/e2e/binary_integration.rs` process isolation rather than a copied implementation. Verify the command refuses shared sockets/state and provider-network access, cleans only owned fixtures, and distinguishes real binary acceptance from scripted-provider helper coverage.

## 2. Persist authority and implement session admission

- [ ] 2.1 Extend existing session/control persistence and `jcode-swarm-core`/protocol types with retained owner ancestry, monotonic control revision, run generation and stop disposition. Add legacy/corrupt/stale-write roundtrip regressions and run `cargo test -p jcode-swarm-core` plus focused `jcode-app-core` persistence tests. Verify no stale swarm snapshot or terminal-member GC clears revocation; document compatibility fields and retention boundary with this slice.
- [ ] 2.2 Route exact-ID inspect/stop resolution through retained daemon identity before existing self/ancestor or explicit override authorization; keep aliases swarm-local. Add control tests for member removal, owner reconnect, ambiguous aliases, unrelated denial and explicit override. Verify allowed/denied results through real socket requests and update runtime command documentation without broadening permissions.
- [ ] 2.3 Add per-session serialized admission shared by stop, new work and native tool dispatch, carrying session/generation through nested tool contexts. Cover attached messages, spawn/assignment, wake/resume/retry, scheduled delivery, recovery and lightweight registration. Verify concurrent stop/wake and persistence-failure tests reject revoked admission and no valid stopped stub becomes an ownerless runnable member; document all admission entry points.

## 3. Cancel actual execution and observe settlement

- [ ] 3.1 Reuse actual active-turn cancellation signals and retain per-generation completion accounting for live/headless/attached turns. Add idle/provider-busy/multiple-attachment stop regressions and verify `stop_requested` then draining/settled output without holding global locks during waits. Run focused `turn_cancel_registry` and server lifecycle tests; document the lifecycle transition table in `docs/SWARM_ARCHITECTURE.md`.
- [ ] 3.2 Track native tool-operation settlement and known owned process/background work independently of async caller/task lifetime. Verify a cancelled caller with continuing filesystem work cannot report stopped, and an uncertain detached/remote effect returns unresolved with category diagnostics. Test real native operations and owned child lifetime, not only a dropped future; document shell/MCP/external-effect limits and no unrelated-process termination.
- [ ] 3.3 Persist final settlement before releasing resources and implement idempotent stop request/retry plus deadline reporting. Verify repeated stop, delayed settlement, persistence errors and status inspection retain the barrier and cannot return `Stopped agent` early. Run stop/deadline protocol and tool formatter tests, including older clients receiving explicit nonterminal diagnostics.

## 4. Keep file ownership until settlement

- [ ] 4.1 Bind native claim acquisition/release/reservation cleanup to the admitted generation and reject revoked edit/write/patch/apply_patch independently of worker membership and off/warn/block mode. Extend real registry/claim tests and verify unchanged bytes after revoked calls, preserved live conflict behavior, continuing mutation protection and no old-generation cleanup of new claims.
- [ ] 4.2 Move stop/member-removal/orphan claim release to the settlement boundary and forbid automatic replace/reassignment while predecessor work is unresolved. Verify native tool-busy stop and reconciled-orphan tests preserve claims until actual IO completes, then allow a new owner. Document the safer release semantics and explicit unresolved recovery path alongside the claim tests.

## 5. Bind accepted work, reports and waits to generations

- [ ] 5.1 Allocate one monotonic generation per accepted work request, including queued requests, and serialize explicit post-stop resume/reassignment against predecessor settlement. Verify concurrent resume has one winner, stale requests cannot become fresh work, and generation G tools never gain G+1 authority. Add public socket admission regressions and document new-work intent semantics.
- [ ] 5.2 Carry generation identity in completion reports, wait targets, mutation/cache keys and notifications; bind legacy waits at invocation and retain old reports as historical. Verify old ready reports and cached finals cannot satisfy new work, unknown membership is not done, stopped wait is not task success, restart preserves pending bindings, and stale notifications do not wake revoked work. Run focused `comm_await`, `await_members_state` and report protocol tests; update swarm report documentation without taking over `add-swarm-handoff-contracts` revision/check/budget enrichment.

## 6. Reconcile restart and mixed-version operation

- [ ] 6.1 Reconcile persisted owner/control before restoring membership or runnable work; preserve cancellation across restart, retry and legacy snapshots. Verify restart mid-stop, lost sessions, uncertain child liveness, stale snapshots and corrupt control records through persistence and real daemon restart tests. Document exact recovery decisions, keeping cancelled work non-runnable and unknown work unresolved.
- [ ] 6.2 Define the backward-readable field defaults and enforce a safe compatibility floor at supported startup/reload boundaries. Verify old clients read history, pending stop never appears successful, and an unsupported downgrade cannot serve state while ignoring revocation. Record rollback that preserves bytes and barriers without destructive migrations; run protocol compatibility and startup/reload tests.

## 7. Final integration acceptance and evidence

- [ ] 7.1 Run the documented actual selfdev binary on the isolated public socket from 1.3. Exercise idle/model-busy/native-tool-busy stop, stop/wake and resume races, owner control after membership churn, unrelated denial, stale wait replay, restart mid-stop and unresolved background/external work. Verify request/generation identity, zero new native mutations after confirmed settlement, no stale readmission and correct claims transfer. Record complete commands/results plus unsupported paths. Synthetic fixtures or unit passes cannot substitute for this acceptance step.
- [ ] 7.2 Run fresh final-diff gates through coordinated `selfdev test`: `cargo test -p jcode-app-core --lib`, `cargo test -p jcode-swarm-core`, `cargo test -p jcode-protocol`, `cargo test -p jcode --test e2e` for non-ignored offline tests, relevant SDK/harness compatibility tests, `git diff --check`, `openspec validate make-worker-stop-durable --strict --no-interactive` and canonical selected-closure dependency validation. Verify all affected checks pass, failures are investigated, and public acceptance from 7.1 uses this exact final diff/binary; rerun affected checks after any repair.
- [ ] 7.3 Update architecture/runtime/control documentation and reciprocal P04 evidence links from observed final behavior, and revalidate the overlap boundary with `add-swarm-handoff-contracts`. Verify every scenario below has fresh acceptance/test evidence or remains explicitly blocked, without claiming P04-wide completion.

## 8. Review and native closeout

- [ ] 8.1 After fresh final gates, obtain independent review against final diff, requirements and acceptance evidence. Verify actionable findings are repaired, affected gates rerun and independent review refreshed; no self-approval substitutes for this terminal step.
- [ ] 8.2 Update authoritative tasks and any verified issue state only from accepted evidence, then commit scoped implementation/workflow changes using the configured identity after resolving checkout policy. Verify exact staged paths and commit range preserve worker attribution and exclude unrelated changes. No remote publication or issue transition is implicit when no linked issue/target authority exists.
- [ ] 8.3 Only after implementation tasks 1–7, fresh gates, independent review 8.1 and implementation commit 8.2 pass, archive via native `openspec archive make-worker-stop-durable` and verify synchronized specs, roadmap links and archive scope. Verify commit-range review includes task/proposal/archive changes and commit the archive state; confirm any required authorized persistence separately. Do not archive merely because planning artifacts validate or silently skip blocked delivery gates.

## Requirement and task coverage

| Capability / requirement | Implementation and local checks | Public/final acceptance |
| --- | --- | --- |
| worker-session-control / Daemon-scoped identity and preserved authorization | 2.1, 2.2 | 7.1, 7.2 |
| worker-session-control / Durable stop admission barrier | 2.1, 2.3, 3.3 | 1.2, 7.1 |
| worker-session-control / Observable execution settlement | 3.1–3.3 | 7.1, 7.2 |
| worker-session-control / Explicit safe generation transition | 2.3, 5.1 | 7.1 |
| worker-session-control / External effect boundaries remain explicit | 3.2, 4.1, 4.2 | 7.1 |
| worker-session-control / Protocol compatibility cannot synthesize stop success | 3.3, 6.2 | 7.1, 7.2 |
| swarm-state-persistence / Worker recovery after reload | 2.1, 6.1 | 7.1 |
| swarm-state-persistence / Control persistence is monotonic and fail closed | 2.1, 6.1, 6.2 | 7.1, 7.2 |
| swarm-communicate-reliability / Per-action request deadlines | 3.3 | 7.1, 7.2 |
| swarm-communicate-reliability / Report-based completion | 5.1, 5.2 | 1.2, 7.1 |
| swarm-communicate-reliability / Wait replay respects generation identity | 5.2, 6.1 | 7.1, 7.2 |
| swarm-file-claims / Claim release | 3.2, 4.1, 4.2, 6.1 | 7.1 |
| swarm-file-claims / Revoked native writes cannot bypass claim checks | 2.3, 4.1 | 7.1, 7.2 |

All scenario variants under each mapped requirement are required, including failure cases. Group 8 covers the whole final diff and all these requirements.

## Tools, environment and authority prerequisites

| Tasks / operation | Required route and target | Access and decision boundary |
| --- | --- | --- |
| 1–6 local implementation and checks | Verified Jcode checkout, Rust/Cargo, exact file claims, native storage/control tests; coordinated selfdev test/build | A later implementation request is required. No external credentials or source upload. Preserve unrelated changes and serialize overlapping schemas. |
| 1.2, 1.3, 7.1 actual binary acceptance | Existing selfdev binary, unique owned Unix socket, isolated HOME/JCODE_HOME/JCODE_RUNTIME_DIR/JCODE_INSTALL_DIR, loopback-only deterministic endpoint and fixture directory | No shared daemon restart, production workers, remote provider calls or original application access. If isolation cannot be proved, block only runtime acceptance and dependent closeout. |
| 7.2 schema/dependency gates | Installed OpenSpec; `/usr/bin/python3 /home/nyaptor/dev/personal/.agents/scripts/openspec/validate_dependencies.py --root /home/nyaptor/.jcode make-worker-stop-durable --json` on this host | No new dependency install. Resolve canonical validator location again on another host, never copy private instructions into code. |
| 8.1 independent review | Separate native read-only reviewer after fresh final gates | Included in later Apply terminal contract, not satisfied by authoring workers. |
| 8.2 commit | Configured Git identity; resolved branch/default policy; exact scoped stage and commit range | Currently unverified/blocked policy resolution. Do not switch primary checkout or invent identity. |
| 8.3 archive and persistence | Native OpenSpec archive after implementation review/commits; exact history and spec-delta inspection | Not part of proposal authoring. Shared activation, remote push/PR and destructive downgrade need their own scoped authority. |

Task commands are planned verification recipes, not claims that tests ran. New acceptance regression names and command entry point are deliverables of 1.2–1.3, not invented existing tests. Missing runtime or closeout prerequisites remain visible and cannot be bypassed by marking tasks done.
