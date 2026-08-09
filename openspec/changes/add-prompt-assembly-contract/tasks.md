# Tasks

## 1. Layer model and assembly

- [ ] 1.1 Add `PromptLayer`, `PromptLayerSource`, `PromptLayerMode`, `PROMPT_ASSEMBLY_VERSION`, and `PromptAssembly` to the prompt module, with static layer order as a fixed contract.
  - touches: `crates/jcode-base/src/prompt.rs`
  - depends on: none
  - Done when layers carry id/source/mode/content, the static order matches the documented contract, and `static_text` from layers is byte-identical to the pre-change join for unchanged inputs.

- [ ] 1.2 Add `build_prompt_assembly(...)` and delegate the existing split builders to it, including the `system_prompt_override` mapping to a one-layer replace assembly.
  - touches: `crates/jcode-base/src/prompt.rs`, `crates/jcode-app-core/src/agent/prompting.rs`
  - depends on: 1.1
  - Done when the existing builder signatures keep working, override sessions produce a digest and attribution through the same path, and the pre-existing prompt test suite passes unmodified.

## 2. Digest and attribution

- [ ] 2.1 Compute the prompt digest (version + layer ids + source kinds + static contents, stable 64-bit hash rendered hex) on every assembly.
  - touches: `crates/jcode-base/src/prompt.rs`
  - depends on: 1.2
  - Done when identical inputs across processes produce identical digests, any content/order/version change changes the digest, and path-only source differences do not.

- [ ] 2.2 Extend `ContextInfo` with per-layer attribution (`id`, origin label, chars) and the digest; surface both in `/context` and session debug state.
  - touches: `crates/jcode-base/src/prompt.rs`, `crates/jcode-app-core/src/server/debug_server_state.rs`
  - depends on: 2.1
  - Done when `/context` data lists every static layer's origin (builtin / project file / global file / runtime) with char counts, and debug state exposes `prompt_digest` plus the attribution list.

## 3. Session freeze

- [ ] 3.1 Capture `FrozenPromptAssembly` (static layers, static text, digest, captured_at) at session/agent first build and store it on the session record.
  - touches: `crates/jcode-app-core/src/agent/`
  - depends on: 2.1
  - Done when the snapshot is captured once per session and the digest is recorded.

- [ ] 3.2 Use the frozen static text for every later turn's prompt build; dynamic layers (memory, active skill, turn reminder, swarm directive) continue to build per turn.
  - touches: `crates/jcode-app-core/src/agent/prompting.rs`
  - depends on: 3.1
  - Done when editing an overlay file after capture does not change that session's prompt or digest on later turns, a fresh session observes the edit, and resume/reload captures a fresh snapshot on first build.

## 4. Golden snapshots and gate evidence

- [ ] 4.1 Add golden snapshot tests for base, append (global, project, both), replace (project, global, precedence, empty fallback, runtime override), and runtime augmentation (memory, turn reminder, swarm directive).
  - touches: `crates/jcode-base/src/prompt_tests.rs`
  - depends on: 3.2
  - Done when every gate scenario has a golden, goldens for unchanged inputs match the pre-change assembly byte-for-byte, and the freeze/digest tests from sections 2-3 are included.

- [ ] 4.2 Document the contract: layer table, digest semantics, freeze semantics, and the Pi append/replace mapping.
  - touches: `docs/PROMPT_ASSEMBLY.md (new)`, `docs/SYSTEM_PROMPT_CONFIG.md`
  - depends on: 4.1
  - Done when the doc records the layer order and ids, digest inputs and rendering, frozen vs dynamic layers, the Pi mapping table, validation commands, and rollback notes.

## 5. Validation

- [ ] 5.1 Run strict OpenSpec validation and focused prompt test suites.
  - touches: `openspec/changes/add-prompt-assembly-contract/*`
  - depends on: 4.2
  - Done when `openspec validate add-prompt-assembly-contract --strict --no-interactive` passes, the prompt test suites pass, and the full `jcode-base` / `jcode-app-core` suites show no new failures against the recorded baseline.
