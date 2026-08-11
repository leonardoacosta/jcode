---
name: ratchet-ops
description: >
  cc harness binding for the ratchet-ops concept (validate-cc row triage + Ledger-closure
  authoring). Triggers: [Ratchet] FAIL, ratchet row, validate-cc, POLICY_CHECKS, hook-contract,
  Tier 3, ratchet-last-run.
allowed-tools: Read, Bash, Grep, Glob, Edit, Write
---

# Ratchet Ops — cc harness binding

The portable model (30-Second Model, Triage Procedure, Narrowest-Control Ladder, Ledger-closure
rule, NEVER table) lives in the promoted skill: `leo-core:ratchet-ops`
(`~/dev/personal/skills/leo-core/skills/ratchet-ops/SKILL.md`, released
`e1de9d9680ab028e4d0777ae23009081e43582ac`). Read it first. This file is only the concrete names
that copy had to generalize away — cc's runner, row constant, and lookup paths.

## Runner

```bash
# Reproduce a row live (fresh evidence — the nightly snapshot may be a day old)
CLAUDE_PROJECT_DIR=~/.claude ${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/validate-cc --tier 3 --quiet; echo $?
```

Nightly-timer invocation (timer-only — never run `--file-issues` interactively, it double-files
beads):

```bash
validate-cc --json --file-issues > ~/.claude/state/ratchet-last-run.json
```

## Row source of truth

- Rows: the `POLICY_CHECKS` array + `_chk_*` counter functions, both in `scripts/bin/validate-cc`.
- hook-contract check (not a `POLICY_CHECKS` row): a `# requires-settings: <key>=<value>` header
  walk over every `scripts/hooks/*.sh`.
- Snapshot: `~/.claude/state/ratchet-last-run.json`. `session-primer` reads it and emits
  `[Ratchet] FAIL: <ids>` (or `[Ratchet] STALE` past 48h) at session start; silent when green.

## Where to look things up

| Need | File |
| --- | --- |
| Per-row table (asserts / counter fn / landed-by), generated from `POLICY_CHECKS` | `docs/reference/ratchet-inventory.md` |
| Checks documentation — lane contract, Ledger-closure rule, three-tier header policy | `docs/reference/tooling-standards.md` § Config Ratchet Lane |
| Root-cause narratives / incident history per row | `docs/notes/config-ratchet-lane-history.md` |
| Per-row remediation map (fix pattern, known traps) for this harness | [references/row-runbook.md](references/row-runbook.md) |

For the 30-Second Model, the Triage Procedure, the Narrowest-Control Ladder, and the
Ledger-closure rule itself: see the promoted skill cited above. Do not restate them here — a
copy drifts from the concept it was copied from.
