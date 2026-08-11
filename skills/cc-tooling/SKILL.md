---
name: cc-tooling
description: CC tooling standards — MCP selection, mermaid, skill and agent mapping, config architecture. Explicit-only.
allowed-tools: Read, Glob, Grep, Bash
---

# Tooling Standards

> Loaded on-demand when working with MCP, diagrams, skills, or config. Not in system prompt.
> For core rules: `rules/CORE.md` | For deploy: `rules/DEPLOY.md`

Harness binding for the portable **`agent-tooling`** skill (promoted, revision `e1de9d968`,
`meta/skills/agent-tooling/SKILL.md`) — read that skill for the portable MCP-selection criteria,
diagram-choice table, and skill-vs-agent decision framework. This file is a dispatcher over what's
concretely local to `~/dev/claude`: this harness's config paths, project registry, and validation
gate. Each heading below is a one-paragraph teaser; read the linked `references/*.md` file for
the full tables and gotchas.

## meta Validation Gate

Working in the `cc` meta-repo itself (skills, commands, agents, hooks, scripts) has one combined
gate before calling a change verified: `bash -n scripts/**/*.sh && openspec validate --strict`.
The first half syntax-checks every shell script without executing it; the second half is the
standard OpenSpec strict validation (see § OpenSpec CLI Patterns below for the non-interactive
flags that gate needs).

## Project Registry

Per-project fleet notes (T3 fleet defaults, `xx`'s paradigm-divergent stack, `brown`/`ws`
corporate-fleet caveats, personal-automation-stack layout) — moved from `CLAUDE.md` § 7. The
manifest (`.claude/project.toml`) is the real source of truth; this table is a hint.
Read `references/project-registry.md`.

## MCP Server Inventory

Which MCP servers are connected on this harness, the three-knob resilience env bundle that
hardens long autonomous runs against connection/tool-call hangs, and the agent-browser-vs-full-
snapshot token-cost benchmark now that Playwright MCP is denied. Generic selection criteria and
anti-patterns for adding a new server: `agent-tooling` skill's `references/mcp-selection.md`.
Read `references/mcp-servers.md`.

## Mermaid Diagrams

This box's exact `PUPPETEER_EXECUTABLE_PATH` for `mmdc`. Generic rendering commands, flowchart
syntax rules (`<br/>` vs `\n`, subgraph comment restrictions), the `stateDiagram-v2` label
caveat, and the Mermaid-vs-alternative diagram-type table: `agent-tooling` skill's
`references/mermaid-diagrams.md`.
Read `references/mermaid-diagrams.md`.

## Agent-Skill Mapping

This repo's actual per-agent recommended-skills table and the six cross-cutting engineering-
discipline skills (`test-driven-development`, `verification-before-completion`, etc.), plus the
`Skill({ skill: "..." })` invocation quick reference. Generic skill-vs-agent decision criteria
and discovery-pattern rationale: `agent-tooling` skill's `references/skill-vs-agent-mapping.md`.
Read `references/agent-skill-mapping.md`.

## Remediation-Skill Authoring Idiom

A **remediation skill** is symptom-triggered, not topic-triggered — it activates when a specific
problem SHOWS UP ("page feels slow", "og image not showing"), not on a general subject area
("Next.js metadata"). Trigger phrasing should read symptom-first, never topic-first. Adapted from
ibelick/ui-skills' `fixing-*` family (`fixing-metadata`, `fixing-motion-performance`; MIT,
github.com/ibelick/ui-skills — cited once here). Two conventions to carry into any new remediation
skill:

- **Priority x impact table**: ship a table of findings/issues x priority x user-facing impact,
  closing with a row or note that establishes "tool boundaries: critical" — the skill must be
  explicit about what it fixes automatically versus what requires a tool-boundary decision it
  cannot make on its own (e.g. a business-logic call).
- **Dual-mode invocation**: support two invocation shapes — `/skill` (ambient/no-arg, runs as a
  general audit pass over the current context) and `/skill <file>` (targeted, audits one specific
  file/artifact). Document this as the design pattern for other remediation skills to follow; it
  is not something this section implements in a command.

This idiom is documented here, in-house, rather than in a new skill (or in `find-skills`/
`skill-creator`) because the natural real targets for something like this are vendored
third-party skills wiped clean on every re-vendor pass — `cc-tooling` is durable in-house
reference material for whoever authors the next remediation skill.

## NEVER

Hard-won failure modes across the tooling areas above — each one already cost a real debugging
loop the first time it was hit. Each is detailed further in its topic's reference file.

- **NEVER pipe stdin at an interactive CLI you haven't run `--help` on first.** `openspec` reads
  confirmation prompts from `/dev/tty` — `echo "y" | openspec archive <name>` doesn't fail loudly,
  it hangs or silently no-ops. Check for `--yes`/`--no-interactive`/`--force` before piping.
  `references/openspec-cli.md`.
- **NEVER hand-edit or `mv` an OpenSpec change out of `openspec/changes/`.** The project's own
  `gate.sh` hook blocks direct archive moves; use `openspec archive <name> --yes` (add
  `--skip-specs` for an ADDED-only capability with no parent spec yet). `references/openspec-cli.md`.
- **NEVER glob-expand a path inside an env var assignment.** `PUPPETEER_EXECUTABLE_PATH=~/.cache/.../chrome-headless-shell-linux-*/...`
  does NOT expand — shell globs don't fire in that position. Resolve the exact path first, every
  time the puppeteer/chrome-headless-shell version bumps. `references/mermaid-diagrams.md`.
- **NEVER generate a Mermaid Live URL as a rendering shortcut.** Pako encoding, padding bugs, and
  `<br/>` incompatibilities make them fragile, and they open on the homelab box, not the user's
  Mac — `mmdc` is the only sanctioned renderer. `agent-tooling` skill's `references/mermaid-diagrams.md`.
- **NEVER add an MCP server that duplicates a CLI tool already in reach** (`git`, `gh`, `npm`),
  has no auth on a public API (use `WebFetch` instead), holds an always-on connection for
  event-driven work, or has sat unmaintained >6mo. All five are the concrete anti-patterns behind
  the MCP Selection Criteria, not abstract advice. `agent-tooling` skill's `references/mcp-selection.md`.
- **NEVER let `nova-*` MCP servers back into `mcp.json`.** Deprecated and removed three times now
  (git history: removed, re-added "to restore access", removed again) — `nx-send` is the
  successor. `scripts/bin/validate-cc` Tier 3's `nova-absent` check catches a re-add, but don't
  rely on the ratchet to notice before you do. `references/mcp-servers.md`.
- **NEVER wrap `ropen <file>` in `Bash({run_in_background: true})`.** It already exits in ~30ms;
  backgrounding it adds noise, not value. Also never write generated HTML to `/tmp/` and expect
  `ropen` to serve it — it serves from the file's git root, so out-of-repo files have no stable
  URL. `references/ropen.md`.
- **NEVER assume a hook, MCP server, or skill citation firing is the same as it having fired.**
  "Connected" in the MCP inventory and "wired into settings.json" both describe configured intent,
  not confirmed execution — verify with a log line, matcher test, or liveness check before
  trusting the mechanism (§ Thinking Patterns below expands on this).

## Shared Config Architecture

Directory layout for `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/shared/`, what stays global vs. project-local, and the
machine-local `.claude/local/` override pattern.
Read `references/shared-config.md`.

## RTK (Rust Token Killer)

Token-compression CLI proxy, on trial via rtk's own PreToolUse hook plus a trusted
`.rtk/filters.toml`. Project filters fire only on the explicit `rtk <cmd>` path, not through the
hook. Savings are measured on ccusage billing (`docs/rtk-upstream-trial.md`), never on rtk's own
ledger, which overstates them (bead `cc-w83ov.217`); never use `rtk discover`, because Claude
transcripts record pre-rewrite commands and produce misleading adoption results.
Read `references/rtk.md`.

## Opening Files in a Browser (ropen)

`ropen <file>` — fire-and-forget SSH+AppleScript dispatch to a Mac browser tab, backed by a
systemd-user HTTP server the CLI never starts or stops. Use whenever Leo needs to *see* a file
on this headless Arch box. The whole `*open` family (`ropen`/`sopen`/`gopen`/`mopen` for
viewing, `copen`/`vopen`/`zopen` for editing) is now `if`-owned and chezmoi-deployed — cc no
longer builds, symlinks, or otherwise puts these on `$PATH`.
Read `references/ropen.md`.

## OpenSpec CLI Patterns (non-interactive)

The `openspec` CLI reads confirmation prompts from `/dev/tty` — piping `echo "y" |` does not
work and burns loops. The correct non-interactive flags for validate/archive, plus the
anti-patterns that dead-end.
Read `references/openspec-cli.md`.

## Thinking Patterns (frame before acting)

These are the recurring judgment calls the reference tables above can't make for you — work them
before reaching for a tool. The first two are portable and covered in full by `agent-tooling`
skill's own § Thinking Patterns; the remaining two are cc-specific.

1. **Config presence != runtime liveness.** A hook wired into `settings.json`, an MCP server
   listed as "Connected", or a skill cited in a routing table all describe *intent to run*, not
   *confirmation it ran*. Before trusting a mechanism, verify it fired (a log line, a matcher
   test, a hook-liveness check) — the § MCP Server Inventory table logs status per server, but
   status is what was configured, not what last executed.
2. **A benchmark number is a claim about a specific comparison, not a universal ranking.** The
   agent-browser-vs-snapshot token benchmark is only valid because Playwright MCP is denied — the
   moment the compared-against tool changes, the "86% fewer tokens" number needs re-measuring, not
   re-quoting. Treat cited benchmarks as scoped to their stated comparison.
3. **Prefer the dedicated script's `--json` over re-deriving state.** When a detection script
   already exists (deploy state, gate state, wave state), calling it and reading its JSON is
   cheaper and more correct than re-inspecting files/git yourself — the script encodes edge cases
   you'd otherwise re-derive from scratch, and re-deriving invites drift between two sources of
   truth for the same fact.
4. **An interactive CLI's silence about non-interactive flags is not permission to loop.** Piping
   `echo "y" |` into a prompt that reads `/dev/tty` doesn't fail loudly — it hangs or loops
   silently (§ OpenSpec CLI Patterns). Before piping stdin at any unfamiliar CLI, run `--help`
   once and look for `--yes`/`--no-interactive`/`--force` — cheaper than discovering the hang
   after burning a loop.

## Related Skills

- `agent-tooling` (promoted, `meta/skills/agent-tooling/`) — portable MCP-selection,
  diagram-choice, and skill-vs-agent-mapping guidance this file binds to
- `documentation-writer` (`references/mermaid-diagrams/`) — Syntax reference and diagram type selection for creating new diagrams
- `orchestrator-patterns` — Multi-agent orchestration patterns for complex workflows
