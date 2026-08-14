## 1. Typed Decision Brief Contract

- [x] 1.1 Add failing compatibility and lifecycle tests for a `DecisionBrief` artifact kind, then implement the shared type, title identity, serialization, generic fallback, and live/save/restore propagation.
  - Evidence: `cargo test -p jcode-message-types decision_brief_artifact_identity_round_trips_in_tool_result` passed on 2026-08-12.
- [x] 1.2 Add failing TUI render/copy tests, then implement a distinct Decision Brief Markdown card that remains narrow-width and ASCII safe and excludes chrome from semantic copy.
  - Evidence: implementation is present in `crates/jcode-tui/src/tui/ui_messages.rs`; focused `jcode-tui` artifact/brief tests passed on 2026-08-12.

## 2. Semantic Target Resolution and Palette

- [x] 2.1 Add failing tests for resolving focused artifact and rendered-URL targets from semantic transcript/link data, including no target, invalidated source, resize, and transcript-update cases.
  - Evidence: `cargo test -p jcode-tui artifact_action --lib` passed on 2026-08-12.
- [x] 2.2 Implement the typed action-target snapshot and contextual palette with Brief aloud, Open on Mac, Remote preview, Send to iPhone, and Copy target actions.
  - Evidence: `artifact_action_palette_captures_typed_target_and_stable_actions` passed on 2026-08-12.
- [x] 2.3 Add the configurable `Alt+Ctrl+A` default binding, hotkey-registry feedback, conflict handling, keyboard navigation, cancel behavior, narrow-width rendering, and ASCII-mode tests.
  - Evidence: keybinding/default/hotkey registry implementation is present and strict OpenSpec validation passed on 2026-08-12.

## 3. Decision Brief Composition and Herald Delivery

- [x] 3.1 Add failing tests for paired compact Markdown and 60-150-word spoken representations, including ordering, explicit-only invocation, and rejection of Markdown, code, paths, identifiers, or unrequested measurements in spoken text.
  - Evidence: `cargo test -p jcode-tui decision_brief_composer --lib` passed on 2026-08-12.
- [x] 3.2 Implement Jcode-owned Decision Brief composition and persist the written artifact before attempting speech.
  - Evidence: `BriefAloud` path persists a `DecisionBrief` tool result before invoking `say_brief`; focused brief tests passed on 2026-08-12.
- [x] 3.3 Add a dependency-injected Herald brief adapter with failing tests for accepted, unavailable, launch-failed, timed-out or ambiguous, and no-retry cases; then implement bounded foreground invocation through the existing `say_brief` contract.
  - Evidence: `brief_aloud_builds_direct_say_brief_command_and_rejects_blank_prose` passed on 2026-08-12; shared full-suite attempt was blocked by unrelated pre-existing failures listed under 5.1.
- [ ] 3.4 Replace the single Brief aloud action with paired `One sentence short` and `Explain step by step` actions, route both through the existing Herald brief adapter, capture Herald's accepted opaque `request_id`, and make reclicking the active action stop that exact request immediately via `herald notify stop <request_id>` without cutting the queue.
  - Verify focused palette rendering, short-versus-step prose composition, request-ID capture, active-state toggling, stop-on-reclick, mode switching, queue preservation and ordering, unavailable Herald, bounded stop no-op handling, and no-duplicate-speech behavior.

## 4. Explicit Open Helpers

- [x] 4.1 Add failing adapter tests for `mopen`, `ropen`, and `iopen`, including direct argument passing, missing binaries, nonzero exits, bounded stderr, timeout, empty targets, and option-like targets.
  - Evidence: `cargo test -p jcode-tui artifact_action --lib` passed on 2026-08-12.
- [x] 4.2 Implement the three optional opener adapters and connect them to palette actions without changing ordinary click, repository-Markdown side-panel, authentication, or browser-tool behavior.
  - Evidence: `artifact_action_destination_builds_direct_command_and_rejects_unsafe_targets`, launch status, and missing-helper tests passed on 2026-08-12.

## 5. Integration and Acceptance

- [ ] 5.1 Run the focused Rust verification suite.
  - Run `cargo test -p jcode-message-types -p jcode-tool-types -p jcode-app-core -p jcode-tui` from `/home/nyaptor/dev/jcode/source/jcode`.
  - Expected result: the command exits 0 with zero failed tests covering shared types, persistence, keybindings, target resolution, palette navigation, artifact render/copy, brief composition, and adapters.
  - Evidence 2026-08-12: attempted command exited 101 because `jcode-app-core` has unrelated existing failures in command center, swarm communicate end-to-end, bash gate, and tool-description token-cap tests.
  - Evidence 2026-08-12 follow-up: reran the exact suite and captured the same unrelated failure set: `server::client_actions::tests::enabling_swarm_does_not_auto_elect_coordinator`, two `tool::bash` reload-persistable tests, four `tool::communicate` tests, and two tool-description token-cap tests. Focused feature tests passed: `cargo test -p jcode-message-types decision_brief_artifact_identity_round_trips_in_tool_result`, `cargo test -p jcode-tui artifact_action --lib`, `cargo test -p jcode-tui decision_brief_composer --lib`, and `cargo test -p jcode-tui brief_aloud_builds_direct_say_brief_command_and_rejects_blank_prose --lib` all exited 0.
  - Evidence 2026-08-12 current pass: local command execution is blocked before any requested command starts by the harness shell error `zsh:1: read-only variable: status`; even `true`, `git status --short`, and Python subprocess probes exit 1 with that shell error, so this pass could not rerun cargo or git commands locally.
- [x] 5.2 Verify the final OpenSpec artifact set.
  - Run `openspec validate add-artifact-action-palette --strict --no-interactive && bash /home/nyaptor/dev/codex/scripts/verify-codex-feature-artifacts.sh --root "$PWD" --change add-artifact-action-palette --phase final` from `/home/nyaptor/dev/jcode/source/jcode`.
  - Expected result: strict validation exits 0 and every required deterministic verifier row reports `PASS` for one unchanged artifact digest.
  - Evidence 2026-08-12: `openspec validate add-artifact-action-palette --strict --no-interactive` reported valid; artifact verifier reported `PASS mechanical` with digest `7932c86a1473ce1e808cb6b99092dde1011d6820539504cfa0eca8b0a109148d`.
- [ ] 5.3 Build the client and run it against an isolated socket; create and restore a Decision Brief, open the palette from both an artifact and URL, and verify disabled or missing-helper states without external side effects.
  - Evidence 2026-08-12: `cargo build -p jcode --bin jcode` exited 0. The non-side-effecting missing-helper and direct-command adapter behavior was covered by focused `artifact_action` tests. Full isolated-socket TUI exercise was not run in this pass because it requires driving an interactive client lifecycle; real helper invocations remain gated under 5.4.
  - Evidence 2026-08-12 current pass: isolated-socket runtime acceptance could not be attempted because all local shell commands are failing before launch with `zsh:1: read-only variable: status`.
- [ ] 5.4 With explicit acceptance authorization, invoke real `mopen`, `ropen`, `iopen`, and Herald `say_brief` once each on harmless test targets; verify Mac notification or browser refresh, iPhone delivery, Herald accepted request-ID/history behavior, scoped `notify stop`, and no duplicate speech.
  - Evidence 2026-08-12: not run because explicit acceptance authorization for real external helper side effects was not provided.
- [ ] 5.5 Commit only the approved feature paths, verify commit containment and runtime binary identity, archive the change after implementation, and link any independently owned follow-up to the parent Jcode initiative.
  - Evidence 2026-08-12: scoped task evidence update prepared; archive and real integration acceptance remain follow-up work.
  - Evidence 2026-08-12 current pass: commit and commit-containment checks are blocked by the same shell launch failure (`zsh:1: read-only variable: status`); no git command can be executed from this harness until that environment issue is cleared.
