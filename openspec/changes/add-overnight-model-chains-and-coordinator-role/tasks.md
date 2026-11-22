# Tasks

Prerequisites: this checkout, `scripts/dev_cargo.sh` or `cargo`, no credentials. Runtime checks (5.x) use a private `JCODE_HOME` with `JCODE_TEST_SESSION=1` and a private `--socket`. Never use the user's daemon or session store for synthetic runs.

File ownership for parallel work:
- Lane chains (groups 1-2): `crates/jcode-config-types/src/` (new `overnight` config module + `Config` field in `crates/jcode-base/src/config.rs`), `crates/jcode-app-core/src/overnight.rs` supervisor loop, `crates/jcode-app-core/src/overnight/enforce.rs`, `crates/jcode-app-core/src/overnight/admission_gate.rs`, new `crates/jcode-app-core/src/overnight/model_chain.rs`, worker spawn model default in `crates/jcode-app-core/src/tool/communicate.rs`.
- Lane coordinator (groups 3-4): `crates/jcode-app-core/src/tool/mod.rs` policy check, new `crates/jcode-app-core/src/tool/coordinator_policy.rs`, `crates/jcode-app-core/src/agent.rs` (`restrict_read_only_tools` sibling), `crates/jcode-overnight-core` prompt builders, the TUI visible-start path `crates/jcode-tui/src/tui/app/commands_overnight.rs` only for policy install/remove.
- Shared touch: `overnight.rs` `start_overnight_run`. Chains lane owns it; coordinator lane asks the chains lane (or coordinator) for a one-line hook to install the policy.

## 1. Chain config and selection

- [x] 1.1 Add `[overnight] coordinator_model_chain` and `worker_model_chain` (`Vec<String>`, defaults from the spec) to config types and `Config`. Verify: config parse test with and without the section; defaults equal the spec lists.
- [x] 1.2 Add `overnight/model_chain.rs`: a pure `ModelChain` (ordered list, cursor, `advance(kind) -> Advance::{Next(model), Exhausted(tried)}`) that advances only on `Transient | RateLimited | ModelUnavailable | QuotaExhausted` from `jcode_provider_core::failover::classify_provider_failure`. Verify: table test for every `ProviderFailureKind`; exhaustion lists all tried models.
- [x] 1.3 Preflight filter: drop chain models absent from the provider's `available_models_for_switching()` (keep all when that list is empty); record dropped models in preflight; block start when either chain is empty. Verify: unit tests for skip, empty-block, empty-list-keeps-all.

## 2. Chain execution

- [x] 2.1 Supervised loop: wrap `enforce::run_turn_monitored`; on `Err`, classify, `agent.set_model(next)` on the same provider, record `model_chain_advanced` {from,to,kind}, retry the same prompt; on non-advancing kinds or exhaustion, `block_run` with reasons naming every model tried. Set the first coordinator model at start. Verify: test with a fake provider that fails the first two models with `ModelUnavailable`, then succeeds; events show two advances; provider name/route unchanged. Test that `AuthRevoked` blocks without advancing.
- [x] 2.2 Visible current-session run: set the first served coordinator model at start, and apply the same advance on turn failure for the session that owns an active run. Verify: unit test of the shared advance helper with the visible path's inputs.
- [x] 2.3 Workers spawned by a session that owns an active overnight run: default model = first served worker-chain model (explicit `model` still wins); on worker turn failure advance through the worker chain; on exhaustion fail the worker and notify the coordinator. Verify: spawn-model resolution test (overnight owner vs normal session vs explicit model); exhaustion test asserts coordinator notice and run still `Running`.
- [x] 2.4 Normal sessions: prove no automatic switch. Verify: test that a non-overnight session failing with `ModelUnavailable` keeps its model.

## 3. Coordinator tool policy

- [x] 3.1 Add `tool/coordinator_policy.rs`: `allows(tool, input) -> Result<(), String>` implementing the spec allowlist, `.md`-only `edit`/`write` (resolved path), `apply_patch` denied, MCP allowed, read-only bash parser (split on `&&`, `||`, `;`, `|`; deny `>`, `>>`, `$(`, backticks, trailing `&`; allow listed commands and git read subcommands; deny `find -exec/-delete`). Verify: table tests for every spec scenario plus `git push`, `rm`, `sed -i`, `cargo test`, `echo x > f.md` denied.
- [x] 3.2 Enforce in `tool/mod.rs` at the same point session tool policy is checked, including `batch` members, for sessions registered as overnight coordinators; denial returns the delegation hint. Add register/unregister keyed by session id. Verify: integration test executes `edit` on `.rs` (denied, file unchanged), `edit` on `.md` (runs), `batch` mixed (read runs, edit denied).
- [ ] 3.3 Install the policy when a run starts (supervised child session and visible current session) and remove it when the run reaches a terminal status or the session's run is no longer active. Verify: test start → denied; terminal → allowed again.
- [x] 3.4 Record `coordinator_tool_denied` {tool, reason} in the run's events.jsonl. Verify: test reads events.jsonl after a denial.

## 4. Prompts

- [x] 4.1 Add the delegation rule to `build_coordinator_prompt`, `build_visible_current_session_prompt` and continuation prompts in `jcode-overnight-core`. Verify: prompt tests assert the rule text.

## 5. Verification and closeout

- [ ] 5.1 `cargo test -p jcode-app-core overnight`, `cargo test -p jcode-app-core coordinator_policy`, `cargo test -p jcode-overnight-core`, `cargo test -p jcode-config-types`; record results.
- [x] 5.2 Runtime: selfdev build; private `JCODE_HOME` + socket, a config with an `omni`-like profile pointed at a local fake OpenAI-compatible server that returns `model_not_found` for the first two models; start overnight; confirm two `model_chain_advanced` events and a denied coordinator `.rs` edit in events.jsonl.
- [ ] 5.3 `openspec validate add-overnight-model-chains-and-coordinator-role --strict`, dependency validator, `git diff --check`.
- [ ] 5.4 Independent review of the full diff; fix findings; archive.
