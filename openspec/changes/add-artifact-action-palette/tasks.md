## 1. Typed Decision Brief Contract

- [ ] 1.1 Add failing compatibility and lifecycle tests for a `DecisionBrief` artifact kind, then implement the shared type, title identity, serialization, generic fallback, and live/save/restore propagation.
- [ ] 1.2 Add failing TUI render/copy tests, then implement a distinct Decision Brief Markdown card that remains narrow-width and ASCII safe and excludes chrome from semantic copy.

## 2. Semantic Target Resolution and Palette

- [ ] 2.1 Add failing tests for resolving focused artifact and rendered-URL targets from semantic transcript/link data, including no target, invalidated source, resize, and transcript-update cases.
- [ ] 2.2 Implement the typed action-target snapshot and contextual palette with Brief aloud, Open on Mac, Remote preview, Send to iPhone, and Copy target actions.
- [ ] 2.3 Add the configurable `Alt+Ctrl+A` default binding, hotkey-registry feedback, conflict handling, keyboard navigation, cancel behavior, narrow-width rendering, and ASCII-mode tests.

## 3. Decision Brief Composition and Herald Delivery

- [ ] 3.1 Add failing tests for paired compact Markdown and 60-150-word spoken representations, including ordering, explicit-only invocation, and rejection of Markdown, code, paths, identifiers, or unrequested measurements in spoken text.
- [ ] 3.2 Implement Jcode-owned Decision Brief composition and persist the written artifact before attempting speech.
- [ ] 3.3 Add a dependency-injected Herald brief adapter with failing tests for accepted, unavailable, launch-failed, timed-out or ambiguous, and no-retry cases; then implement bounded foreground invocation through the existing `say_brief` contract.

## 4. Explicit Open Helpers

- [ ] 4.1 Add failing adapter tests for `mopen`, `ropen`, and `iopen`, including direct argument passing, missing binaries, nonzero exits, bounded stderr, timeout, empty targets, and option-like targets.
- [ ] 4.2 Implement the three optional opener adapters and connect them to palette actions without changing ordinary click, repository-Markdown side-panel, authentication, or browser-tool behavior.

## 5. Integration and Acceptance

- [ ] 5.1 Run the focused Rust verification suite.
  - Run `cargo test -p jcode-message-types -p jcode-tool-types -p jcode-app-core -p jcode-tui` from `/home/nyaptor/dev/jcode/source/jcode`.
  - Expected result: the command exits 0 with zero failed tests covering shared types, persistence, keybindings, target resolution, palette navigation, artifact render/copy, brief composition, and adapters.
- [ ] 5.2 Verify the final OpenSpec artifact set.
  - Run `openspec validate add-artifact-action-palette --strict --no-interactive && bash /home/nyaptor/dev/codex/scripts/verify-codex-feature-artifacts.sh --root "$PWD" --change add-artifact-action-palette --phase final` from `/home/nyaptor/dev/jcode/source/jcode`.
  - Expected result: strict validation exits 0 and every required deterministic verifier row reports `PASS` for one unchanged artifact digest.
- [ ] 5.3 Build the client and run it against an isolated socket; create and restore a Decision Brief, open the palette from both an artifact and URL, and verify disabled or missing-helper states without external side effects.
- [ ] 5.4 With explicit acceptance authorization, invoke real `mopen`, `ropen`, `iopen`, and Herald `say_brief` once each on harmless test targets; verify Mac notification or browser refresh, iPhone delivery, Herald accepted/history behavior, and no duplicate speech.
- [ ] 5.5 Commit only the approved feature paths, verify commit containment and runtime binary identity, archive the change after implementation, and link any independently owned follow-up to the parent Jcode initiative.
