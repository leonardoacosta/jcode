# Authoring evidence and preflight

## Binding

Observed 2026-11-03, starting 18:44:06 UTC. Repository `https://github.com/leonardoacosta/jcode.git`, checkout `/home/nyaptor/.jcode`, HEAD `97e9a6caa9701bfe9c63f8fc3d16eae708435325`, Linux x86_64. Relevant implementation targets were clean. Existing unrelated untracked `councils/`, `devices.json`, `exit_records/`, `swarm-file-claims.json` and `swarm-file-claims.lock` remain untouched. Only `openspec/changes/add-config-composition/` is owned by this authoring task.

Source SHA-256 bindings:

- `crates/jcode-base/src/config.rs`: `3a40ce833b4113ab3ed49b742098d96b1c5ee845fd2a906df25657c668bb1ab1`
- `crates/jcode-base/src/config/config_file.rs`: `7b02cd19c9a4c8faacc8534d7e4966474df6947770992cf030ba6bbcd182051e`
- `crates/jcode-app-core/src/tool/config_edit_notice.rs`: `e65a57e9023461c13450154d72a60c4390004ff0b832fa4a2db6da41725a2f10`

Request: user asked to assess config clarity and composition, then invoked Feature for `config.toml comp;`. This authorizes local proposal authoring, not implementation, live migration or remote publication. The proposed contract uses design defaults, not invented user decisions. No linked issue or roadmap outcome was requested. The roadmap was inspected but no unrequested outcome was selected or rewritten.

## Observations and adversarial decisions

- `config/config_file.rs:18–93`: one path `jcode_dir()/config.toml`, direct TOML deserialization, env application after file load, and complete pretty-serialized save. Strict read-modify-write deliberately excludes environment overrides. Sparse root patching must preserve that boundary.
- `config.rs:322–341,391–447`: root metadata fingerprint, reload throttle, dependent listeners and provider context-limit refresh. Fragment edits require full source identity, including equal-length edits and recovery.
- `config/config_file.rs:659–684`: trusted-auth memo keys only root metadata plus trust env. Fixing the main reload alone would leave revoked fragment permissions stale.
- `tool/config_edit_notice.rs`: root-only comparison and watch, existing live/restart classification and whole-config defaults fallback warning. Fragment notices must compare effective values to avoid masked-edit claims.
- `crates/jcode-base/Cargo.toml:37`: `toml_edit = "0.20"` already installed. No new framework/dependency needed.
- Parsed config shape only, without printing values: 540 lines, 29 top-level tables, no comments, 235 lines of named providers. At the initial assessment revision, `Config` lacked `tasks`/`worktrees`. This observation is stale: current source defines them and `configure-task-and-worktree-policy` is archived. Composition must preserve and deserialize those typed tables. Keep forward-compatible preservation of other unknown keys without implementing or weakening task/worktree policy.
- Existing specs and active/archive proposal inventories were searched. No composition capability owner was found. Shared-file overlap is a serialization constraint, not a semantic dependency. `execution.depends_on: []` records no prerequisite change.

Rejected alternatives: automatic folder scanning hides activation/order; recursive includes add avoidable graph semantics; concatenating arrays duplicates catalogs; fragment-owner writes require multi-file transactional persistence; global unknown-key rejection breaks current compatibility. Root-only sparse overrides provide the smallest safe write target. Optional inherited deletion that cannot be expressed in TOML fails explicitly rather than inventing null syntax.

Failure-policy default deliberately retains logged whole-config defaults fallback. Last-known-good retention would be a separate behavior change and can retain stale permissions. Invalid-source errors must not include TOML excerpts/values. Path containment applies after canonicalization and again before writes.

No runtime reproduction or new Rust behavior test has run: this is authoring. Current behavior was established by source inspection and structural parsing only. Planned acceptance uses real config APIs/setters/edit watches in disposable homes, not synthetic copies of implementation or live user config.

## Pre-draft assessment

Rubric `preflight-check-v2-shadow`, phase `pre-draft`, all G1–G6 inspected. Drafting eligible. Scope, source ownership, local schema and no-credential acceptance route are identified. No material user decision blocks drafting.

| Group | Points | Evidence and limits |
| --- | --- | --- |
| G1 scope/authority | 2 | Feature invocation authorizes local artifacts only; no external effects or migration. |
| G2 verification | 1 | Installed Cargo/OpenSpec and existing tests provide a local route; new tests and independent final review are execution-time checks, not passed evidence. |
| G3 delivery/checkout | 1 | Configured Git identity exists. `main` checkout differs from local `origin/HEAD -> origin/dev`; checkout-only policy also says own branch while root/shared policy says preserve primary checkout. No branch changes or commits until reconciled. Remote default is not freshly authenticated. |
| G4 runtime owner | 2 | Jcode CLI/TUI config only, disposable `JCODE_HOME`; live user config and shared daemon excluded. |
| G5 preservation | 2 | Relevant code clean; only new proposal directory owned. Unrelated runtime files preserved. |
| G6 dependencies | 2 | No selected change prerequisites; acceptance, independent review, commit and archive mapped in tasks; adjacent work identified. |

Shadow quality: 10/12 = 83.33%, not an admission gate. Repair response: clarify local test selection and preservation, record precise Git policy blocker, and keep implementation/review unverified instead of inventing results. No unrelated access/network probes, branch mutation or endless wording repair to increase a score.

## Handoff boundaries

Authoring/inspection and structural validation are eligible. Apply is not requested. Future implementation acceptance remains unverified until actual final code and test evidence exist. Independent execution-readiness/final implementation review has not occurred. No overnight-ready claim.

Git commit closeout is blocked by the conflict between `.agents/AGENTS.md` (own branch), root `AGENTS.md` (primary branch preferred), shared policy (preserve checkout) and current `main` versus local remote HEAD `dev`. Resolver: user selects an allowed persistence route after seeing the conflict. Keep proposal files locally intact meanwhile. Publishing, live migration and shared-daemon restart are outside the request, not prerequisites to local authoring.

No authoring archive is appropriate: implementation remains unchecked. Next bounded step after authoring is reviewing the proposed contract, then an explicit Apply request. Recheck source digests, task prerequisites and shared-file ownership if any relevant source/artifact or authority changes.

## Handoff assessment and observed checks

Phase `handoff`, rubric `preflight-check-v2-shadow`, 2026-11-03 UTC. All six groups reassessed against proposal/design/specs/tasks and full planned completion route. Scores remain G1=2, G2=1, G3=1, G4=2, G5=2, G6=2, 10/12 (83.33%, shadow only). G2 remains partial because future implementation acceptance and independent review have not run. G3 remains partial because local authoring commit policy is unresolved. Neither blocks existing local artifacts or structural validation. No implementation, independent review, archival, deployment or remote persistence is claimed.

Observed native checks: `openspec validate add-config-composition --strict` passed; canonical `/home/nyaptor/dev/personal/jev/scripts/openspec/validate_dependencies.py --root /home/nyaptor/.jcode add-config-composition --json` returned exit 0, one valid item, no issues, no prerequisites. `openspec status --change add-config-composition` reports 4/4 planning artifacts complete, not implementation completion. `git diff --check` passed for tracked changes; additional explicit whitespace/scenario/link checks cover new untracked artifacts before final delivery.

Local verification route correction: `scripts/remote_config.sh` documents `JCODE_REMOTE_CARGO=0` as the supported local override; tasks now use that exact setting. Cargo/rustc/OpenSpec commands resolve, but no Rust tests have run for nonexistent implementation. No credentials are needed. `ropen` did not resolve (`command -v ropen` returned no path); tailnet presentation unavailable. Local canonical artifacts are retained. No alternative publisher, transport or artifact disclosure was attempted.

Acceptance mapping: explicit sources and merge → tasks 1.2–1.3; sparse persistence → 2.1–2.3; reload/auth → 3.1; edit feedback → 3.2; opt-in documentation/equivalence → 3.3; fresh integrated gates/review/commit/archive → 4.1–4.3. No task is checked and no execution approval state is created.

## Scoped planning publication, 2026-11-04 UTC

User explicitly requested committing and syncing proposals, implementation later. Publish only these planning artifacts on owned `docs/config-session-proposals` ref from the current HEAD without switching the primary checkout. This preserves unrelated concurrent changes and satisfies own-branch isolation without moving `main` (remote default remains `dev`). No implementation checkboxes are marked complete, no archive or merge is included. Fresh strict validation and selected dependency checks pass. Historical preflight observations above remain revision-bound; implementation must refresh them.

## Apply inventory (task 1.1), 2026-11-19

Observed by grep over `crates/` (non-test unless noted). Owner = task in tasks.md that handles it.

| Surface | Path | Owner |
| --- | --- | --- |
| Strict file load (single root read) | `jcode-base/src/config/config_file.rs` `load_from_file_strict` | 1.2 (now `read_composed_table` + `config/composition.rs`) |
| Non-strict `Config::load`/`load_from_file` (defaults fallback) | `config_file.rs` | unchanged, strict path shared (1.2, 3.1) |
| `load_strict` callers | `jcode-app-core/src/overnight.rs:288,323`, `overnight/admission.rs:342` (unattended task policy), `server/runtime_ssh_host.rs:456`, `jcode-base/src/provider_catalog.rs:823`, `tool/config_edit_notice.rs:51` | 3.1 / 3.2 |
| `effective_task_policy` (read root text directly) | `jcode-base/src/config/task_policy.rs` | 1.2 (now composed) |
| `Config::save` full-serialize | `config_file.rs:save` | 2.1 |
| Setters via `load_for_update`+`save` | `config_file.rs` (update_channel, agent model, anthropic ttl, copilot premium, default model/provider, reasoning effort, transport, service tier, display toggles, latex, launch hotkeys, `allow_external_auth_source` and sibling trust setters) | 2.1 |
| Direct `Config::load()`+`save()` callers | `jcode-base/src/gateway/control.rs:217` (`set_gateway_enabled`), `jcode-tui/src/tui/app/inline_interactive/helpers.rs:103`, `jcode-tui/src/tui/app/commands_colors.rs:254` (`persist`) | 2.1 (use env-applied `load()` baseline; must move to file-only baseline) |
| Root-only legacy migrations (line rewrites on `config.toml`) | `config_file.rs` `migrate_legacy_swarm_spawn_mode_once`, `migrate_idle_animation_off_once` | 2.1 |
| Main reload fingerprint (root mtime+len+env) | `jcode-base/src/config.rs` `ConfigCacheFingerprint` | 3.1 |
| Trusted-auth memo (root mtime+len+env) | `config_file.rs` `trusted_external_auth_lists` / `TRUSTED_EXTERNAL_AUTH_MEMO` | 3.1 |
| Hook consumers (non-strict `config()`) | `jcode-base/src/hooks.rs` (`hook_commands`, `transform_tool_input`, timeouts), `jcode-app-core/src/tool/mod.rs:1007` | 3.1 (strict deny) |
| Edit watch/notice (root only) | `jcode-app-core/src/tool/config_edit_notice.rs` `ConfigEditWatch`, tests `config_edit_notice_tests.rs` | 3.2 |
| Change report (text diff) | `jcode-base/src/config/change_report.rs` | 3.2 |
| Tests | `jcode-base/src/config_tests.rs`, `config_tests/default_provider.rs`, new `config_composition_tests.rs` | all |

The grep above was incomplete (review M5). Further root-only readers found: `jcode-base/src/auth/router_transfer.rs` routing snippet (now routed through `read_composed_table`, test `routing_config_includes_fragment_held_keys_and_fails_on_broken_sources`), and three that stay root-only as KNOWN LIMITS because `jcode-setup-hints`/`jcode-sdk` do not depend on `jcode-base` (adding that dependency or moving composition is out of scope): `jcode-sdk/src/launch_credential_links.rs:31` (copies root `config.toml` only; an `include` root arrives without its fragments), `jcode-setup-hints/src/lib.rs:328` (`launch_hotkeys`, memoized on root mtime/len) and `jcode-setup-hints/src/macos_terminal.rs:116` (`[terminal] preferred`). Documented in `docs/CONFIG_COMPOSITION.md`.

## Apply evidence (tasks 1.3 to 3.3), 2026-11-19

Observed locally with `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh`, disposable `JCODE_HOME` temp dirs only; live `~/.jcode/config.toml` untouched.

| Task | Evidence |
| --- | --- |
| 1.3 | `docs/CONFIG_COMPOSITION.md` (syntax, precedence, errors, adopt/downgrade). Examples exercised by `config_composition` tests (`explicit_order_root_wins_and_siblings_survive`, `unsafe_include_syntax_is_rejected`, `shape_conflicts_name_key_and_both_sources`, `split_layout_equals_single_file_and_is_never_discovered_or_rewritten`). |
| 2.1 | `config/sparse_save.rs`; tests `setter_on_fragment_value_writes_only_a_root_override`, `legacy_single_file_save_is_sparse_and_keeps_comments_and_unknown_tables`, `no_op_save_does_not_write_and_new_file_is_sparse`, `environment_overrides_are_not_persisted_by_setters`, `provider_model_array_override_replaces_whole_array_in_root`, `unrepresentable_removal_of_inherited_value_fails_without_mutation`, `legacy_migrations_use_effective_values_and_never_rewrite_fragments`. Fragments byte-identical asserted. |
| 2.2 | Atomic replace + drift check. Tests `replacement_keeps_permissions_and_symlink`, `concurrent_fragment_edit_before_replace_is_refused`, `concurrent_root_edit_before_replace_is_refused`, `write_and_replace_failures_preserve_original_bytes_and_permissions`, `invalid_baseline_blocks_save_and_keeps_bytes` (owned-temp cleanup asserted). |
| 2.3 | Root overrides, manual removal and race limitation documented in `docs/CONFIG_COMPOSITION.md`. Round trip keeping `[tasks]`/`[worktrees]`/unknown tables in the legacy-save test. |
| 3.1 | `ConfigCacheFingerprint` and trusted-auth memo now key on `SourceFingerprint`; `config_strict()` added; hooks (`hook_commands`, `transform_tool_input`, `run_pre_tool_gate` fail closed) and trust checks deny on composition failure; trust checks no longer answer from the throttled snapshot. Tests `global_cache_sees_equal_length_fragment_edit_membership_and_recovery`, `trust_revocation_in_a_fragment_is_seen_on_the_next_check`, `invalid_composition_denies_trust_hooks_and_task_policy`. Strict callers `overnight*`, `runtime_ssh_host`, `provider_catalog` already use `load_strict` and so compose. |
| 3.2 | `ConfigEditWatch` snapshots root plus listed fragments (and picks up newly listed ones); notice covers listed fragments, masked edits, invalid fragments (names file, no values). Tests in `config_edit_notice_tests.rs`: `fragment_edit_is_reported_and_broken_fragment_warns`, `config_edit_watch_covers_fragments_and_new_includes`, `masked_fragment_edit_does_not_claim_live_effect`; old syntax-break test wording updated. |
| 3.3 | Fixture `scratch/applyall/home-config/{split,single}` (nonsecret); test `split_layout_equals_single_file_and_is_never_discovered_or_rewritten` asserts identical typed results, unchanged files, unlisted/`config.d` files ignored. `restart_only_sections_are_labeled_needs_restart`; docs state gateway/acp/launch_hotkeys need restart. |

Gates: `-p jcode-base config` 249 passed 0 failed (after rustfmt); `-p jcode-base hooks` 22 passed; `-p jcode-app-core config_edit_notice` 11 passed; scoped `clippy -p jcode-base -p jcode-app-core --all-targets` no warnings; rustfmt --check clean on touched files; `git diff --check` on owned paths clean; `openspec validate add-config-composition --strict` valid.

Known limits: `tool/mod.rs:1007` sandbox category check still reads the non-strict `config()`; under a broken composition hooks themselves are denied so no hook child runs. Atomic save cannot lock against an editor writing during the final rename.

## Review-fix evidence (M1-M5, m4/m7/m9/m13), 2026-11-19

- M1: `SwarmSpawnMode::Visible` gets `#[serde(alias = "headed")]` (jcode-config-types), and `migrate_once` now falls back to `Config::patch_root_document` (text-level `toml_edit` patch of only the legacy leaf, no compose/typing) when the composed load fails, as the pre-composition line rewrite tolerated. Tests `legacy_headed_spawn_mode_still_migrates_to_inline`, `migrations_tolerate_an_unrelated_composition_failure` (also: unparseable root left untouched, no marker).
- M2: `hooks::hook_policy_denial()` (strict-config denial reason); `Registry::execute` denies before `transform_tool_input` and the `hook_configured("pre_tool")` skip; `run_pre_tool_gate` shares it. Test `registry_execute_denies_tools_when_config_composition_is_broken` runs `bash` through `Registry::execute` with a deleted policy fragment (side-effect file must not exist), then repairs and confirms the call runs.
- M3: docs narrowed (load unchanged for no-`include` installs, save is sparse/comment-preserving for all); array-removal-as-empty-override and Option-unset-from-fragment documented. Realistic round-trip test `realistic_legacy_file_round_trip_changes_only_the_requested_leaf` (dotted keys, inline tables, literal strings, CRLF, trailing comments) caught a real defect: `toml_edit` normalized CRLF to LF, rewriting every line; fixed with `keep_line_endings` in `sparse_save.rs`. `legacy_file_without_include_is_untouched_on_non_change_paths` asserts load/strict/no-op save leave bytes and mtime unchanged and compose equals plain parse.
- M4: `redact_message` replaces quoted, backticked and apostrophe-literal segments (multi-line included); `diff_toml` masks values of leaf keys matching key/token/secret/password/auth/credential. Tests `error_text_redacts_literal_numeric_and_multiline_values`, `secret_named_keys_never_echo_values_in_notices` (jcode-base), `valid_edit_notices_never_echo_secret_values` (app-core). Residual: serde may echo a numeric value inside prose for some error shapes; the test covers bool-expected cases with integer and float tokens.
- M5: see inventory correction above.
- m4: test `removing_a_fragment_from_the_manifest_reports_and_deactivates_it`. m7: old `fs::write` also used umask defaults, so new root files keep the default; test `new_root_file_uses_default_permissions_like_the_previous_writer` documents it. m9: `Origins::record` records nested keys of whole-table inserts; test `nested_key_conflicts_name_both_sources`. m13: `winning_keys` compares exact keys against winning sources, read once; test `masking_compares_the_exact_key_not_a_prefix_overlap`.
