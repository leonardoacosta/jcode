## 1. Evidence gathering

- [ ] 1.1 Record the vocabulary inventory. Run `rg -n 'GoalStatus|GoalMilestone|GoalStep|GoalUpdate|crate::goal::' crates --stats` and `rg -n '\binitiative\b' crates/jcode-app-core/src/tool`. Verify: `docs/` evidence file lists every internal `Goal*` type and every user-facing `initiative` string, with counts.
- [ ] 1.2 Confirm each durability claim against source. Verify: for revision, idempotency, checkpoints, memory sync, and step/milestone status typing, the recorded file:line still matches the code at the base commit. Any mismatch blocks the change until the table is corrected.
- [ ] 1.3 Confirm the sequencing precondition. Verify: `optimize-orca-command-center-orchestration` task 4.1 is checked complete before any edit to `docs/COMMAND_CENTER.md`. If open, stop.

## 2. Documentation

- [ ] 2.1 Add the native initiative guide under `docs/` covering canonical vocabulary, the authority table with citations, the failure-mode list, and the TUI direct-write exception. Verify: every durability statement in the file carries a `file:line` citation, and `rg -c 'crates/' <guide>` is at least equal to the number of durability claims.
- [ ] 2.2 Update `docs/COMMAND_CENTER.md` to link the new guide and correct any statement that overstates durability. Verify: `rg -n 'idempoten|revision' docs/COMMAND_CENTER.md` shows no claim of durability across restart.
- [ ] 2.3 Add contributor guidance for extending native surfaces before creating a new UI or persistence path. Verify: the guide names the existing read seam (`GoalInitiativeRepository`) and mutation seam (daemon command path) by path.

## 3. Enforcement

- [ ] 3.1 Add the no-frontend-persistence check. Verify: it exits `0` on the current tree; after temporarily inserting `localStorage.getItem('x')` into a file under `apps/command-center/src`, it exits non-zero and names that file and line; the temporary edit is then reverted and the check returns to `0`.
- [ ] 3.2 Wire the check into the existing check script alongside `scripts/check-command-center-contracts.sh`. Verify: running the aggregate check surfaces the new check by name in its output.

## 4. Verification and closeout

- [ ] 4.1 Run `cargo test -p jcode-app-core command_center`. Verify: exit `0`, zero failures.
- [ ] 4.2 Run `cargo test -p jcode-command-center`. Verify: exit `0`, zero failures.
- [ ] 4.3 Run `scripts/check-command-center-contracts.sh`. Verify: exit `0`, no drift reported.
- [ ] 4.4 Confirm no behavioral change. Verify: `git diff 5f8df69c0 --stat -- crates apps` shows changes only to doc comments and the new check, with no modification to any DTO field, enum variant, or function signature.
- [ ] 4.5 Run `openspec validate formalize-native-initiative-command-center --strict --no-interactive`. Verify: exit `0`, output `Change 'formalize-native-initiative-command-center' is valid`.
- [ ] 4.6 Record validation evidence in the proposal and archive only after 4.1 through 4.5 are all green.
