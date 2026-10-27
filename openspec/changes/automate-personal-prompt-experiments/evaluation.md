# Automation execution evidence

## Apply preflight — 2026-10-27

The user explicitly invoked `$apply automate-personal-prompt-experiments`; the new controller design is approved for implementation and its registered local trials. Existing uncapped OmniRoute `gpt-6-luna` / `medium` scope remains in force. Publication and live prompt adoption remain excluded.

Initial dispatch bound to `585fef689` on `main`. Intervening committed JEV/todo changes are unrelated; current pre-existing dirt is `.gitignore`, `.ignore`, the three MCP source/test files, the personal review and `swarm-prompt.md`. Owned implementation is the existing evaluator, focused adjacent Python checks and this change's artifacts/feature-request log. No branch/worktree or shared-daemon mutation is needed.

Python, Git, bwrap and OpenSpec were present. A bounded networkless namespace probe succeeded. The existing `omni` named credential assignment exists; no secret value was printed/copied. Immutable launcher build remains `1ba484fa3-dirty-90b55a2744c6`; active shared build is outside this task. Runtime/model access still requires the new private preflight.

Task dependencies: preparation → grading/control/recovery → capability preflight → registered actor/assessment trials → fresh final gates/independent review/local commits/archive. Existing local-only scope excludes a push. The local main spec remains intentionally excluded; preserve its prior requirements on native archive. An independent review can use a separate tool-free Jcode session on the same required OmniRoute model, with the actual final code and evidence. That route must be demonstrated before claiming review.

Initial-dispatch `proposal-readiness-v1-shadow`: G1=2 (explicit Apply/model/local effects), G2=1 (offline route available; new runtime/reviewer unverified), G3=2 (main/local commit/archive routes known; publication excluded), G4=2 (private campaign owner and separate process/socket ownership), G5=2 (owned paths clean; unrelated dirt mapped), G6=2 (all eleven tasks and terminal gates mapped). 11/12, 91.7%, advisory only. Independent local implementation is eligible; inference/review/closeout depend on their fresh checks.

## Offline checks

`PYTHONPATH=scripts python3 -B -m unittest scripts/test_personal_prompt_eval.py -q`: initial 18 tests passed, including actual process death on either side of submission, concurrent lock refusal, corrupted audit, exact restore, all fixture/reference oracle pairs, unsupported grading evidence, threshold/transfer/counterexample cases.

`python3 -B scripts/prepare_personal_prompt_eval.py target/personal-prompt-eval/2026-10-27-scope-pilot-v4 --verify`: retained 36-slot pilot verified without mutation or inference.

`target/personal-prompt-eval/2026-10-27-automation-rehearsal-v1/` is a no-model preparation/preflight rehearsal, not a behavioral result. Five family/candidate banks and all fixture/oracle pairs were prepared successfully. Subsequent observations and final results will be recorded below.

## Implementation and private rehearsal

The existing evaluator now owns frozen five-family/candidate banks, two baseline-first tiers, sealed confirmation, separate tool-free semantic assessment, exact restoration, durable spent-slot markers, a hash-chained audit and exclusive coordinator lock. There are no new dependencies or services. Routing and continuity are explicitly unsupported by the reduced adapter; no compaction or full skill-routing success is inferred.

CLI (all outputs private under `target/personal-prompt-eval/`):

```bash
python3 -B scripts/prepare_personal_prompt_eval.py <new-root> --campaign
python3 -B scripts/prepare_personal_prompt_eval.py <root> --verify
python3 -B scripts/prepare_personal_prompt_eval.py <root> --preflight
python3 -B scripts/prepare_personal_prompt_eval.py <root> --run
python3 -B scripts/prepare_personal_prompt_eval.py <root> --assess
```

Preparation, verification, preflight and assessment-only do not submit inference. `--run` resumes spent-slot evidence; it never replaces a submitted slot. Campaign children cannot be executed directly. Only disposable personal candidates change; source-default and current personal bytes remain distinct controls under matched surrounding context. Model assessment uses separate fresh sessions with zero tools. Malformed/missing evidence stays unknown and cannot advance; deterministic protected outcome and preservation checks override a model pass. Applicable edit rules may partially reveal a condition, so blinding is not double-blind.

Fresh focused gate: `PYTHONPATH=scripts python3 -B -m unittest scripts/test_personal_prompt_eval.py -q` — **28 tests passed**. Checks cover real process death on both sides of submission, lock refusal, malformed audit, exact restoration, immutable child/driver binding, all defective/reference oracle pairs, rejected attempts versus quotations, burden/correctness, transfer/sign thresholds, explicit historical identities, simulated full KEEP/confirmation/two-lead completion, three-REVERT plateau, common failure and interruption. Controller tests use a fake backend and are not behavioral results.

Fresh legacy verification retained all 36 historical slots unchanged. `git diff --check` and `openspec validate automate-personal-prompt-experiments --strict --no-interactive` passed.

Rehearsals v1–v7 are retained, never reset into new evidence. v1 caught a launcher-versus-resolved executable argument mismatch and stopped before inference; registration now binds the resolved immutable argument. v2 established 72 fresh no-inference native sessions (24 per supported family); later source changes require fresh campaign preflight. v5–v7 review/assessment sessions are separate from actor trials. All inference used native private Jcode sessions, OmniRoute exact `gpt-6-luna`, effort `medium`; no shared-daemon replacement or global provider change occurred.

Independent pre-inference Luna review identified missing availability of captured Jcode-local skills. The runner now mounts frozen bodies/resources read-only at native `/state/skills` and the callback home path. v6 used a synthetic unique skill and resource; actual serving-process `/proc/.../root/state/skills` inventories proved both visible. The live Jcode-local skill directory is empty. Routing remains unsupported because a matched native skill/recording-tool adapter is absent.

A subsequent review raised a cumulative-target premise contrary to the approved design: candidates must each start from original personal bytes. v7 explicitly verified every candidate actor base hash equals the selected disposable target; the frozen replacement drives actual actor context. Its non-defect assessor concern was checked by a real zero-tool Luna session: a synthetic review packet produced valid evidence pointers and pass/pass scope/outcome. The historical-identity check was strengthened to require S1–S4 at all three repetitions and distinct counterexamples; its negative test passed. Final review and actual campaign observations follow below.
