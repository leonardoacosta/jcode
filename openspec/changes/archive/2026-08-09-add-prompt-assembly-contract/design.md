# Design

## Context

The system prompt is assembled in `crates/jcode-base/src/prompt.rs` by concatenating string parts. `build_system_prompt_split_with_capabilities` (the live path) produces:

- **Static parts** (cacheable by providers): base prompt (`base_system_prompt_parts`, which embeds the builtin `prompt/system_prompt.md` via `include_str!`, applies capability modules such as Mermaid guidance, and honors a base replacement file), self-dev guidance (selfdev sessions only), `AGENTS.md` (project + global), prompt overlays (`./.jcode/prompt-overlay.md` + `~/.jcode/prompt-overlay.md`), preferred-tools files, and the available-skills section.
- **Dynamic parts** (per turn): memory prompt and the active skill prompt. The agent additionally appends a per-turn system reminder and the swarm effort directive (`crates/jcode-app-core/src/agent/prompting.rs`), and short-circuits entirely when `system_prompt_override` is set.

File conventions today:

- Base replace: `./.jcode/system-prompt.md` (project) then `~/.jcode/system-prompt.md` (global), first non-empty wins; empty/whitespace falls back to the builtin (`load_base_system_prompt`).
- Append: overlay and preferred-tools files stack in fixed positions.
- `docs/SYSTEM_PROMPT_CONFIG.md` documents that file changes affect **new sessions** only — but every builder call re-reads the files from disk, and the builder runs per turn, so the guarantee is aspirational.

`ContextInfo` already tracks per-section char counts and feeds `/context` and debug output. Hooks (`crates/jcode-base/src/hooks.rs`) dispatch observer events (`session_start`, `turn_start`, `pre_tool`, ...) with no prompt-mutation channel.

Pi compatibility references (read-only): Pi exposes base/replacement/appended prompt inputs and a `before_agent_start` hook that can mutate the prompt. jcode needs the same append/replace semantics as a named contract, not Pi's files or hook surface.

## Goals / Non-Goals

**Goals:**

- A versioned, named-layer assembly contract: fixed layer order, per-layer `id`, `source`, `mode`, content, and char count.
- Session-frozen static layers: captured once per session, reused every turn; the documented new-session semantics become real.
- A reproducible prompt digest recorded in the session record and shown in session status/debug output.
- Source attribution extending `ContextInfo`, visible in `/context` and debug state.
- Named append/replace modes with a documented Pi mapping; golden snapshots for base, append, replace, project overlay, and runtime augmentation.

**Non-Goals:**

- The full P0 session profile (provider/model/credentials/tool-policy snapshot). This change adds only the prompt portion (digest + attribution) to the session record.
- Hook-based prompt mutation (Pi `before_agent_start` parity). Hooks stay observer-only; P5 owns the hook contract.
- Pi file vendoring, copying, or adaptation of any kind.
- Unifying `.jcode/swarm-prompt.md` into the contract (analogous layering, separate change).
- Prompt editing UI, live reload commands, or a `/prompt` surface.
- New telemetry events or Herdr fields (P5 owns that contract).

## Architecture

### 1. Layer model

`crates/jcode-base/src/prompt.rs` gains:

```rust
pub const PROMPT_ASSEMBLY_VERSION: u32 = 1;

pub enum PromptLayerSource {
    Builtin,                 // embedded include_str! content
    ProjectFile(PathBuf),    // ./.jcode/... or ./AGENTS.md
    GlobalFile(PathBuf),     // ~/.jcode/... or ~/AGENTS.md
    Runtime(&'static str),   // capability module, selfdev, skills, memory, reminder, swarm directive
}

pub enum PromptLayerMode { Replace, Append }

pub struct PromptLayer {
    pub id: &'static str,        // "base", "capability:mermaid", "selfdev", "agents-md", ...
    pub source: PromptLayerSource,
    pub mode: PromptLayerMode,
    pub content: String,
}
```

Static layer order (the contract, bumped only with `PROMPT_ASSEMBLY_VERSION`):

1. `base` (mode `Replace`-able; the only replace-able layer)
2. capability modules (one layer each, e.g. `capability:mermaid`)
3. `selfdev` (selfdev sessions only)
4. `agents-md` (project and global merged as today; attribution records both origins)
5. `prompt-overlay`
6. `preferred-tools`
7. `skills-list`

Dynamic layer ids (per-turn, excluded from the frozen snapshot and the static digest): `memory`, `active-skill`, `turn-reminder`, `swarm-directive`.

**Why a layer record instead of attributed strings:** the digest, the freeze, and the attribution UI all need the same per-layer metadata. Building records first and joining second keeps one source of truth and makes byte-identity with the pre-change join a testable property (`layers.map(content).join("\n\n") == legacy_parts.join("\n\n")`).

### 2. Assembly and digest

`PromptAssembly { version, static_layers, dynamic_layers, static_text, digest }`.

- `static_text` joins static layer contents exactly as today (byte-identical requirement).
- `digest`: stable 64-bit hash (the repo already standardizes on xxh3-style non-cryptographic hashes for content addressing) over `version`, each static layer's `id`, source *kind* (not path, so identical content on two machines correlates), and content. Rendered as hex (`prompt:1:<hex16>`).
- Builders keep current signatures; a new `build_prompt_assembly(...)` returns the contract object, and the existing `SplitSystemPrompt` builders delegate to it. `system_prompt_override` maps to a one-layer assembly (`id = "override"`, mode `Replace`) so its digest and attribution flow through the same path.

### 3. Session freeze

The session record gains `prompt_snapshot: Option<FrozenPromptAssembly>` where `FrozenPromptAssembly { static_layers, static_text, digest, captured_at }`.

- Captured at agent/session construction (first prompt build), stored alongside the existing session state in `crates/jcode-app-core/src/agent/`.
- `agent/prompting.rs::build_system_prompt_split` uses the frozen `static_text` for every subsequent turn instead of re-reading files; dynamic parts are still built per turn and appended exactly as today.
- Resume/reload paths capture on first build after resume; the digest is recomputed then and may differ from the original session's (correct: files may have changed while the session was closed, and a resumed session is a new prompt capture, matching the doc's "existing sessions do not retroactively change" within their live lifetime).

**Why freeze only static layers:** memory, active skill, reminder, and swarm directive are turn-scoped by design and documented as dynamic. Freezing them would break memory injection and skill invocation. Enumerating them in the contract makes the boundary explicit instead of emergent.

### 4. Attribution surface

`ContextInfo` gains `layer_attribution: Vec<PromptLayerAttribution>` where `PromptLayerAttribution { id, source_label, chars }` and `prompt_digest: String`.

- `/context` breakdown lines gain the digest and per-layer origin labels (builtin / `project: ./.jcode/prompt-overlay.md` / `global: ~/.jcode/...` / runtime kind).
- Session debug state (`debug_server_state.rs`) exposes `prompt_digest` and the attribution list for operator inspection and future P0 profile composition.

### 5. Pi compatibility mapping (documentation only)

| Pi concept | jcode contract |
| --- | --- |
| base `systemPrompt` | `base` layer (builtin default) |
| replacement prompt | `base` layer with mode `Replace` from `system-prompt.md`, or full-assembly `override` |
| appended prompt | `prompt-overlay` append layer |
| `before_agent_start` mutation | not supported; hooks are observer-only (documented gap) |

No Pi sources are read at runtime or copied into the repo; the mapping lives in `docs/PROMPT_ASSEMBLY.md`.

## Testing strategy

Golden snapshots in `crates/jcode-base/src/prompt_tests.rs` (tempdir fixtures for project/global files):

1. **base**: builtin-only assembly matches the pre-change golden string exactly.
2. **append**: global overlay, project overlay, and both, with project/global precedence and join order frozen in goldens.
3. **replace**: project file replace, global file replace, project-over-global precedence, empty-file fallback to builtin, runtime `override` assembly.
4. **augmentation**: memory injection, per-turn reminder, swarm effort directive land in the dynamic part with correct ids.
5. **digest**: identical inputs → identical digest across processes; any content/order/version change → different digest; source *path* differences with identical content → same digest.
6. **freeze**: capture, edit overlay file on disk, rebuild for a later turn → identical prompt and digest; fresh capture → observes the edit.
7. **attribution**: `/context` data shows each layer's origin label and chars; debug state carries the digest.
8. **byte identity**: the full pre-existing prompt test suite passes unmodified.

## Rollback

The change is an internal refactor plus the freeze behavior fix. Reverting restores per-turn file reads; no config, persistence, or protocol migration exists to unwind. If the freeze causes surprise for a workflow that depended on mid-session edits, the mitigation is the documented one (start a new session), matching the long-standing doc promise.
