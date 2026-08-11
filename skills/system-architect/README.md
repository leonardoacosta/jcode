
# system-architect (in-house)

Custom skill — merged and improved from two community sources.

## Lineage

| Source | Repo | What we kept |
|--------|------|-------------|
| `system-architect` | `aj-geddes/claude-code-bmad-skills` | 5-phase architecture process, anti-patterns table, component/interface templates, orchestrator integration |
| `system-design` | `qodex-ai/ai-agent-skills` | Consistency patterns, state management, data flow pipelines |

## What was removed

- **From system-architect**: Tech stack selection phase (generic pros/cons table Claude does natively), redundant resilience patterns table
- **From system-design**: Scaling patterns, reliability patterns, observability three pillars, security design, deployment topology — all content Claude already knows deeply. No anti-patterns existed to keep.

## Merge date

2026-03-05

## Maintenance

This is an in-house skill. Update directly in `~/.agents/skills/system-architect/SKILL.md`.
Symlinked at `~/.claude/skills/system-architect`.
