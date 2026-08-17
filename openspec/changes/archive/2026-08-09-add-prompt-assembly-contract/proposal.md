# Add Prompt Assembly Contract (Prompt Compatibility Layer, P1)

## Why

The roadmap handoff (`ROADMAP_HANDOFF.md`, block P1 — Prompt compatibility layer) requires mapping jcode's prompt sources — base prompt, capability modules, `AGENTS.md`, global/project overlays, skills, memory, replacement prompts, and append prompts — into a **versioned prompt assembly contract** with source attribution and a prompt digest, while preserving Pi's append/replace behavior as a compatibility target without copying Pi files.

Today the assembly in `crates/jcode-base/src/prompt.rs` is an **unversioned positional string join**:

- Layers have no names, no recorded origins, and no order contract; `parts.join("\n\n")` is the only structure.
- `docs/SYSTEM_PROMPT_CONFIG.md` promises "changes to these files take effect for **new sessions**; a running session keeps the prompt captured at start", but the code re-reads overlay/AGENTS.md/preferred-tools files from disk **on every turn** (`build_system_prompt_split_with_capabilities` runs per turn). The documented session-isolation guarantee is not enforced.
- Append and replace semantics exist only as file conventions (`prompt-overlay.md` appends; `system-prompt.md` replaces the base; `system_prompt_override` replaces everything). Nothing names these modes, so Pi compatibility is implicit and untestable.
- There is no prompt digest: nothing can answer "did two sessions run with the same prompt" or correlate a session's behavior with its exact prompt content.

## What Changes

- Introduce a versioned, named-layer assembly contract: every prompt layer carries `id`, `source` (builtin / project file / global file / runtime), `mode` (`replace` for the base layer only, `append` for stacked layers), content, and char count. Layer order becomes a fixed, documented constant (`PROMPT_ASSEMBLY_VERSION = 1`).
- **Session-frozen static layers**: the static assembly (base, capability modules, self-dev guidance, `AGENTS.md`, overlays, preferred tools, skills list) is captured once per session and reused for every turn, enforcing the documented "changes affect only new sessions" guarantee. Dynamic layers (memory, active skill, system reminder, swarm effort directive) remain per-turn and are enumerated explicitly in the contract.
- **Prompt digest**: a stable hash over the contract version, layer ids, source kinds, and static-layer contents, recorded in the session record at start and surfaced in session status/debug output. Identical inputs produce identical digests; any layer change changes the digest.
- **Source attribution**: per-layer origin and char counts extend the existing `ContextInfo` accounting and are visible in the `/context` breakdown and session debug state, so operators can see exactly which file or builtin produced each part of the prompt.
- Append/replace modes become named contract semantics: `replace` applies only to the base layer (project `./.jcode/system-prompt.md` > global `~/.jcode/system-prompt.md` > builtin default, first non-empty wins, empty falls back); `append` layers stack in the fixed documented order; `system_prompt_override` is modeled as a full-assembly replace for the session.
- Golden prompt snapshots cover the P1 gate: base, append, replace, project overlay, and runtime augmentation (per-turn reminder + swarm directive + memory injection).

## Capabilities

### New Capabilities

- `prompt-assembly`: A versioned, named-layer prompt assembly contract with fixed layer order, replace/append modes, per-layer source attribution, a session-recorded prompt digest, session-frozen static layers, and golden snapshot coverage.

### Modified Capabilities

None. Prompt content for unchanged inputs is byte-identical to the pre-change assembly; the only behavioral change is enforcing the already-documented freeze-at-start semantics.

## Impact

- Refactors assembly internals in `crates/jcode-base/src/prompt.rs` to build `PromptLayer` records before joining; the public builders keep their signatures or gain contract-returning variants.
- Extends `ContextInfo` (or adds a sibling `PromptAttribution`) with per-layer source labels; consumed by the existing `/context` surfaces and `crates/jcode-app-core/src/server/debug_server_state.rs`.
- Adds a session-captured prompt snapshot (static layers + digest) stored on the session record in `crates/jcode-app-core/src/agent/` and reused by `agent/prompting.rs` per turn.
- Adds golden snapshot tests: `crates/jcode-base/src/prompt_tests.rs` (assembly, digest, attribution, freeze semantics) plus fixture overlays exercised through tempdirs.
- Documents the contract: `docs/SYSTEM_PROMPT_CONFIG.md` gains the layer table, digest semantics, and the Pi append/replace compatibility mapping.
- Does not change provider behavior, tool policy, hooks dispatch, session persistence format, or any P0/P2+ surface.

- touches: `crates/jcode-base/src/prompt.rs`
- touches: `crates/jcode-base/src/prompt_tests.rs`
- touches: `crates/jcode-app-core/src/agent/prompting.rs`
- touches: `crates/jcode-app-core/src/server/debug_server_state.rs`
- touches: `docs/SYSTEM_PROMPT_CONFIG.md`
- touches: `docs/PROMPT_ASSEMBLY.md (new)`

## Preconditions

- Prompt text for unchanged inputs is byte-identical to the pre-change assembly; existing golden/prompt tests keep passing without content edits.
- File conventions keep working: `system-prompt.md` (project > global, first non-empty wins), `prompt-overlay.md`, `preferred-tools.md`, `AGENTS.md`, `system_prompt_override`, `swarm-prompt.md` (separate, unchanged).
- Hooks remain observer-only: no hook gains the ability to mutate the prompt (Pi's `before_agent_start` mutation is a documented compatibility note, not a v1 feature).
- The static/dynamic split for provider caching is preserved: dynamic layers stay out of the cached static prefix.

## Decisions

- **Freeze static layers at session start, per the existing doc promise.** The current per-turn disk re-read is the bug; the doc already promises new-session semantics. Users relying on mid-session overlay edits get the documented behavior instead. Dynamic layers (memory, active skill, reminder, swarm directive) are enumerated and stay per-turn.
- **Digest over structure, not just text.** The digest covers contract version + layer ids + source kinds + static contents, so two sessions with identical text but different assembly versions or layer provenance differ. A stable 64-bit hash rendered hex is sufficient (collision tolerance matches build-info use cases; this is correlation, not security).
- **`replace` is base-layer-only by contract.** Append layers always apply, even when the base is replaced (current behavior, now explicit). `system_prompt_override` is modeled separately as full-assembly replace so its "everything replaced" semantics stop being a special case in prose only.
- **Attribution extends `ContextInfo` instead of a parallel system.** Char accounting already exists per layer; adding source labels there keeps `/context` and debug output on one accounting path.
- **Pi compatibility is a mapping table, not code.** The design documents Pi `systemPrompt` (replace) ≈ jcode base replace, Pi `appendSystemPrompt` ≈ jcode append layers, Pi hook mutation ≈ not supported (observer hooks only). No Pi files are vendored, copied, or edited.
- **Swarm prompt stays a separate contract.** `.jcode/swarm-prompt.md` has analogous layering for swarm routing; unifying it is a later change, explicitly out of scope here.
- **Telemetry unchanged.** The digest is recorded in the session record and debug state; no new Herdr events or telemetry fields (P5 owns that contract).

## Done Means

- Golden snapshots pass for: base-only assembly, append overlay (global and project), base replace (project and global, including empty-file fallback), project overlay precedence over global, and runtime augmentation (memory injection, per-turn system reminder, swarm effort directive).
- Digest is reproducible: identical inputs across two assemblies produce identical digests; any layer content, order, or version change produces a different digest.
- Freeze semantics are enforced by test: editing an overlay file after session capture does not change that session's prompt on later turns; a new session observes the edit.
- Source attribution is visible: `/context` output and session debug state list each static layer with its origin (builtin / project file / global file / runtime) and char count, plus the prompt digest.
- The pre-existing prompt test suite passes without content changes (byte-identical assembly for unchanged inputs).
- `openspec validate add-prompt-assembly-contract --strict --no-interactive` passes.
- Rollback: the contract is an internal refactor plus one documented behavior fix; reverting restores per-turn file reads, and no config or persistence migration is needed.

## Relationship to the roadmap

This is roadmap block P1 (Prompt compatibility layer), authored after P2 closed early (P2's surfaces were visual and independent). P1's gate — golden prompt snapshots for base, append, replace, project overlay, and augmentation, with prompt changes affecting only new sessions — is earned entirely within this change. The session-recorded digest and attribution give P0's fuller session profile its prompt portion when P0 is authored, and unblock P3–P5, which the handoff gates on a stable prompt contract.
