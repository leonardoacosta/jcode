---
execution:
  version: 1
  depends_on: []
---
# Proposal

Roadmap owner: [P01: Trusted core and acceptance/evidence baseline](../../../docs/plans/PRODUCT_ROADMAP.md#p01-trusted-core-and-acceptanceevidence-baseline)

## Why

`/plan` is prompt-only. `build_plan_prompt` (`crates/jcode-tui/src/tui/app/commands_plan.rs:27-40`) tells the model "Do NOT implement anything yet: do not edit files, write patches, or change git state", but nothing enforces it: the turn runs with the normal tool set, so a plan-only turn can still call `edit`, `write`, `apply_patch` or mutating `bash`. The user cannot distinguish a proposed plan from authorized execution, which is the P01 outcome.

## What Changes

- While a `/plan` turn is active, deny mutating native tools (`edit`, `write`, `apply_patch`) with a typed tool error naming plan mode.
- Read-only tools (`read`, `agentgrep`, `conversation_search`, web reads) stay available.
- `bash` is limited to a read-only allowlist; denials are recorded in the turn transcript.
- The restriction ends when the plan turn completes or is cancelled; the next ordinary user turn runs unrestricted.

## Non-scope

Persistent plan mode toggle; MCP tool read-only classification; sandboxing bash; approval UI for executing the plan.

## Capabilities

### New Capabilities

- `plan-execution-guard`: see specs/plan-execution-guard/spec.md.

## Impact

`crates/jcode-tui/src/tui/app/commands_plan.rs`, tool dispatch in `crates/jcode-app-core/src/tool/`. Local and remote sessions; the server enforces the guard, so the turn request carries a plan-only flag. Old servers ignore it, which is reported as unenforced rather than treated as safe.

## Open decisions

1. Bash handling. Options: deny all bash; allowlist read-only commands; allow all. Recommended: allowlist (`git status/diff/log`, `rg`, `ls`, `cat`) because planning needs repo inspection and deny-all breaks grounding.
2. MCP tools during plan. Options: deny all; allow all. Recommended: deny all until MCP tools carry read-only metadata, since server tools may mutate remote state.
