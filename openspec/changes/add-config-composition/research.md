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
