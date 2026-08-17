# Prompt Assembly Contract

jcode builds the system prompt from an ordered, versioned set of named layers.
This document is the contract: layer order, digest semantics, freeze semantics,
and the Pi compatibility mapping. It is enforced by golden tests in
`crates/jcode-base/src/prompt_tests.rs` and
`crates/jcode-app-core/src/agent/prompting.rs` (freeze tests).

Contract version: `PROMPT_ASSEMBLY_VERSION = 1`
(`crates/jcode-base/src/prompt.rs`). Any change to layer order, layer identity,
or join semantics requires bumping the version.

## Layer model

Each layer is a `PromptLayer { id, source, mode, content }`:

- `id` — stable name (`"base"`, `"agents-md-project"`, ...), participates in the
  digest.
- `source` — where the content came from:
  - `Builtin` — embedded at compile time (`include_str!`).
  - `ProjectFile(path)` — `./AGENTS.md`, `./.jcode/...`.
  - `GlobalFile(path)` — `~/AGENTS.md`, `~/.jcode/...`.
  - `Runtime(kind)` — in-process content (capability modules, selfdev, skills,
    memory, reminders, swarm directives).
- `mode` — `Replace` (substitutes the builtin default for its slot; only the
  `base` layer may use it) or `Append` (stacks in the fixed contract order).

## Static layer order (the contract)

| # | Layer id | Source | Notes |
|---|----------|--------|-------|
| 1 | `base` | builtin, or project/global `system-prompt.md` | the only `Replace`-able layer; empty file falls back to builtin |
| 2 | `capability:<name>` | runtime | one per enabled capability, e.g. `capability:mermaid` |
| 3 | `selfdev` | runtime | self-dev sessions only |
| 4 | `agents-md-project`, `agents-md-global` | project/global files | one layer per file, in that order |
| 5 | `prompt-overlay-project`, `prompt-overlay-global` | project/global files | append guidance |
| 6 | `preferred-tools-project`, `preferred-tools-global` | project/global files | tool preferences |
| 7 | `skills-list` | runtime | available skills section |

Static layers join with `\n\n` into `static_text`, byte-identical to the
pre-contract builders (the legacy suites prove this by running unmodified).

## Dynamic layers (per-turn, never frozen)

| Layer id | Source | Notes |
|----------|--------|-------|
| `memory` | runtime | recalled memory for this conversation |
| `active-skill` | runtime | body of the currently invoked skill |
| `turn-reminder` | runtime | current-turn system reminder |
| `swarm-directive` | runtime | swarm effort/task-graph directive |

Dynamic layers join into `dynamic_text` and are rebuilt every turn. They are
excluded from the digest, the frozen snapshot, and static attribution.

## Digest

`prompt:<version>:<hex16>`, computed over the contract version plus every
static layer's `id`, source *kind* (not path), and content, hashed with
SHA-256; the first 8 bytes are rendered as 16 lowercase hex chars.

- Identical inputs always produce identical digests (reproducible).
- Any content, order, identity, or version change changes the digest
  (sensitive).
- Paths deliberately do **not** participate: identical content at different
  paths correlates to the same digest across machines. Source *kind* does
  participate (builtin vs project-file with identical content differ).
- `Runtime` labels do not participate either; only the `runtime` kind.

The digest is exposed on `ContextInfo.prompt_digest`, surfaced by `/context`,
and frozen with the session snapshot.

## Freeze semantics

On the first prompt build of a session, the static assembly is captured into a
`FrozenPromptAssembly { version, static_text, digest, attribution,
context_info, captured_at }`:

- **TUI sessions**: stored on `App` (`prompt_snapshot`).
- **Server/CLI sessions**: stored on the app-core `Agent` (`OnceLock`).

Every later turn reuses the frozen `static_text`, digest, and attribution;
only the dynamic layers rebuild. This implements the documented
[SYSTEM_PROMPT_CONFIG](SYSTEM_PROMPT_CONFIG.md) promise that file changes take
effect for **new sessions** — previously overlay and AGENTS.md files were
re-read every turn.

Resuming a session constructs a fresh `Agent`/`App`, which captures fresh on
its first build. The digest may differ from the original session's — correct,
because files may have changed while the session was closed.

`system_prompt_override` (ambient mode) flows through the same path as a
one-layer full-assembly replace (`id = "override"`, mode `Replace`), with its
own digest and attribution.

## Attribution

`ContextInfo.layer_attribution: Vec<PromptLayerAttribution { id, source_label,
chars }>` covers the static layers. `/context` renders the digest and a
"Prompt Layers" section (`- {id}: {chars} chars ({source_label})`). After the
freeze, `/context` shows the **frozen** static accounting (what the provider
actually receives), while per-turn dynamic fields (memory chars) stay live.

## Pi compatibility mapping

Documentation-only parity with Pi's prompt inputs; no Pi sources are read or
copied.

| Pi concept | jcode contract |
| --- | --- |
| base `systemPrompt` | `base` layer (builtin default) |
| replacement prompt | `base` layer with mode `Replace` from `system-prompt.md`, or full-assembly `override` |
| appended prompt | `prompt-overlay-*` append layers |
| `before_agent_start` mutation | not supported; hooks are observer-only (documented gap, owned by the hook contract roadmap block) |

## Rollback

The contract is additive: legacy builders delegate to the assembly and remain
byte-identical for unchanged inputs. Reverting means deleting the delegation
and restoring the inline builders; no persisted state or schema is involved
(the frozen snapshot is in-memory only).
