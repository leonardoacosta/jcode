## Reviewer-only context recovery, 2026-11-09

User approved reviewer-only context-size repair, preserving required policies, exact configured provider/model and quota limits. Prior reviewers received an unsolicited46758character late-MCP schema catalog before reading source. Root cause traced to Agent::announce_late_mcp_tools. Bounded recovery uses parent_id plus explicit worker_role=reviewer, not text inference. Only unsolicited MCP discovery prose is compacted; schemas remain discoverable through existing MCP search/call. Existing system/policy prompt, tool definitions/permissions, ordinary/edit/root notices, provider/model and quota stay unchanged. No shared-daemon activation.

Owned source: crates/jcode-app-core/src/agent/turn_execution.rs and agent_tests/deferred_tools.rs. Coordinated RED576359uknf failed missing compact reviewer discovery hint. GREEN630433yxdg passed late-MCP5/deferred8tests and app-core all-target Clippy. Tests cover reviewer parent/role boundary, ordinary/root/edit unchanged schema notices, compact notice<400bytes, once-only announcement, same locked tools/system prompt and retained discoverable schemas. TUI build683948h783 passed, source fingerprintfe8c901a0def774a.

Private actual daemon acceptance uses rebuilt binary, isolated state/socket, existing omni-azure:combo/azure/DeepSeek-V4.1-Flash and effort, required policy overlays plus synthetic local40tool MCP catalog. First fixture waited for history rather than observed session event and timed out before spawn; corrected. Second fixture hit rolling quota429 and deadline240s, transcript checker incorrectly ignored journal; corrected. Third actual reviewer reached completed and APPROVE, request shapes105539→117308→134314→149628→169891characters with no request-size400; harness incorrectly stopped at report before true turn settlement and ignored verdict inside report tool input. Thus not a passing clean terminal acceptance. Current run729405eelz waits turn-unregistered plus ready/completed and reads snapshot+journal+explicit report. No source/acceptance/commit/archive completion claimed before observed final result. This scoped repair does not implement the larger budget ratchet, ledger or CI tasks below.

The729405eelz run grew to195064request characters after extra source/policy reads and failed rolling quota429; it did not pass terminal acceptance. Added separate reviewer-only startup message using the existing server automatic capture of final assistant text instead of mandatory report-tool plus another final model request. Other worker prompts retain the original report requirement. Test097511wq0j failed missing helper first, then final329160fv0j passed deferred8/spawn40 and full workspaceClippy. Interim946542h31x failed a test-only call to private method; replaced assertion with public registry.definitions(None), no production visibility widening. Final build376095pvln passed (c5bbd66b1397b83c). Current actual acceptance407414cjh4 runs final discovery+automatic-final-report changes, waits turn-unregistered and settled status. No shared activation, quota/route change or complete-ratchet claim.

### Verified scoped recovery, 2026-11-09 03:08 UTC

Actual private native acceptance2547700t6i passed exit0 after337s: independent reviewer session_chicken_1794192255171_711812605a4982d6 reached ready with turn unregistered and substantive APPROVE for this repair. Exact configured omni-azure:combo/azure/DeepSeek-V4.1-Flash, policy overlays retained, no quota/routing changes. It inspected all scoped diffs, schema discovery and final-response capture; found no actionable defect. Source hashes match coordinator readback: turn_execution7aa7c8d7f19a8d6d8d71bb5d4ff0266a504ef2c1f83dbe4b889b6cb0b5b4a33f; deferred_toolsfc2d94f37709becf219839fd25e1fa2de4468f3cf6bdaa1b9b5a6ea187b5fb78; comm_session8c7661d3f223105dbf08bcd6fe09710e1bd7f22d8e1423b79409d8e1eaf8eddd; comm_session_testse650feeb0269f8e6589579afd50a952dbe45732cdf429e4be38aae8fd9f13056. Runtime binarybaa04c53cf7347c1bccdd9ecffbd0c0bbc04603e5d0baabe6c67ccac67bd04b4, isolated daemon cleaned up, shared daemon never activated. Native catalog arrived before list lock in successful run, so no late notice was required; compact late arrival specifically verified by deterministic regression, not falsely claimed from this run. Actual final ready+unregistered fixes previous report-before-failed-continuation acceptance gap. Initial105KB request versus prior197KB is representative, not a controlled identical-catalog benchmark. Existing rolling429/admission503 retries remain honored and are not eliminated.

Final329160fv0j passed deferred8/spawn40/fullworkspaceClippy, named OpenSpec and diffcheck pass. Review APPROVE does not complete broader ratchet or original integrated roadmap wave. Required policies and evidence remain unchanged. Optional invalid-role null regression was a reviewer suggestion, not an actionable finding. Only this bounded reviewer recovery is ready for scoped local commit; other task checkboxes remain unchanged.

## CI decision, 2026-11-10

The user confirmed that existing GitHub Actions CI/workflow deletions are intentional. Preserve those deletions. Task 2.3 remains blocked because this checkout has no verified repository-owned CI target; this decision does not authorize replacement CI or waive CI acceptance. Local measurement, ratchet and ledger tasks remain independent. No workflow file was restored or included in implementation commits.

## 1. Measurement

- [ ] 1.1 Expose per-section prompt assembly (system, instructions, skills) from `crates/jcode-app-core/src/agent/prompting.rs`
- [ ] 1.2 Deterministic offline token counter and fixture workspace

## 2. Ratchet

- [ ] 2.1 `prompt-budget.toml` format and `jcode debug prompt-budget --check` with tolerance
- [ ] 2.2 `--accept --reason <text>` applies explicit budget changes, including proposed tightening after a drop greater than 5%, and appends ledger entry; reject empty reason. Verify `--check` never writes and counter method/version is reported as an approximation.
- [ ] 2.3 Add the read-only check to the repository-owned CI only after the current `.github/workflows/ci.yml` deletion is resolved by its owner. Do not restore/replace unrelated CI; record the dependent gate blocked if no verified CI target exists.

## 3. Ledger

- [ ] 3.1 `prompt-budget-ledger.jsonl` entries: commit, section deltas, optional experiment score delta, decision KEEP/REVERT, reason
- [ ] 3.2 Wire `scripts/prepare_personal_prompt_eval.py` result into ledger entries

## 4. Tests and validation

- [ ] 4.1 Tests: under budget passes, over budget fails naming the section, accept requires reason, ledger append-only
- [ ] 4.2 `openspec validate add-prompt-budget-ratchet --strict`
