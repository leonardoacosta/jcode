---
execution:
  version: 1
  depends_on: [add-typed-skill-selector-metadata]
---
# Proposal

Roadmap owner: [P02: Discoverable native tools, MCP and skills provenance](../../../docs/plans/PRODUCT_ROADMAP.md#p02-discoverable-native-tools-mcp-and-skills-provenance).
Supporting: [mcp-management-experience](../mcp-management-experience/proposal.md) (already shows MCP server scope source in `/mcp`).

## Why

Skills load from several roots with silent override order: Claude plugin installs first, then `~/.jcode/skills`, then agent and project roots, later winning (`crates/jcode-base/src/skill.rs:260-275,322`). The loaded `Skill` keeps only `path` (`skill.rs:21`); the discovery root, whether it overrode another skill, and whether it is a symlink are lost. Users cannot tell before invocation where a skill came from or which authority owns it.

## What Changes

- Record on each loaded skill: discovery root label (plugin, jcode global, agents global, project), resolved path, symlink flag, and the names of roots it shadowed.
- Show these in a `/skill info <name>` detail view and as a compact root label in selector rows.
- For MCP tools, show owning server and its config scope by reading data exposed by `mcp-management-experience`; when unavailable show "unknown".
- Provenance is display-only; invocation behavior and precedence are unchanged.

## Non-scope

- Signing, hashing or trust enforcement.
- Installing or removing skills.
- Name-folder lint (`lint-skill-name-and-folder`).

## Capabilities

### New Capabilities

- `capability-provenance`: Discovery source, override and symlink details for skills and MCP tools.

## Impact

`crates/jcode-base/src/skill.rs` loader, skill tool info output, TUI selector, remote skill payload (additive).

## Open decisions

1. Discovery-root labels. Options: path-derived labels; fixed enum (`plugin`, `global`, `agents`, `project`). Recommended: fixed enum, short and width-safe; roadmap still requires design acceptance.
2. Trust wording. Options: show "trusted/untrusted"; show source only. Recommended: source only, since no trust model exists and a label would imply verification.
