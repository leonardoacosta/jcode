# Agent-Skill Mapping (harness binding)

Portable — see `agent-tooling` skill's `references/skill-vs-agent-mapping.md` for the
skill-vs-agent decision criteria, generic discovery patterns, and why cross-cutting discipline
skills stay broad by design. This file is cc's concrete instance of that mapping: the real
agent names and real skill names in this repo's `agents/` and `skills/` directories.

## Agent-Skill Mapping

> 27 agents total (3 archived, 4 merged/consolidated). Not every agent needs skills — only those with clear skill matches are listed.

| Agent                        | Recommended Skills                                                       | When to Use                                              |
| ---------------------------- | ------------------------------------------------------------------------ | -------------------------------------------------------- |
| `api-engineer`               | `openapi-to-typescript`, `better-auth-best-practices`, `trpc-patterns`\*, `frontend-api-contracts`, `t3-code-patterns`\* | API design, auth endpoints, tRPC routers                 |
| `architecture-reviewer`      | `c4-architecture`, `mermaid-diagrams`, `t3-monorepo-patterns`\*, `code-review` | System design, architecture documentation, monorepo review |
| `cc-practices-analyst`       | `agent-architecture`, `skill-judge`, `orchestrator-patterns`, `cc-tooling`, `cc-reference` | Auditing CC setup and agent configurations               |
| `codebase-health-orchestrator`    | `reducing-entropy`\*, `code-review`, `eslint-audit`                         | Structural completeness reviews, health analysis, code quality |
| `db-analyst`                 | `database-schema-designer-ext`                                           | Analyzing existing database schemas                      |
| `db-engineer`                | `database-schema-designer-ext`, `better-auth-best-practices`, `t3-code-patterns`\*, `drizzle-best-practices`\* | Designing schemas, migrations, import path fixes         |
| `cloudpc-specialist`         | `azure-devops-cli`, `azc`, `azure-diagnostics`, `bb-fortify`, `ado-task-management` | cloudpc/ADO/Bicep + Fortify SAST/DAST pipeline & API work |
| `dev-health-check`           | `webapp-testing`                                                         | Pre-flight dev environment validation                    |
| `devops-engineer`            | `mcp-builder`, `deploy-detection`, `deploy-and-env`\*, `bb-fortify`      | Building MCP servers, deployment strategy, Fortify pipeline wiring |
| `e2e-engineer`               | `webapp-testing`\*, `qa-test-planner`                                    | Writing Playwright tests, planning test coverage         |
| `infra-analyst`              | `system-architect`                                                       | Infrastructure diagnostics and analysis                  |
| `infra-consultant`           | `system-architect`, `deploy-and-env`                                     | Cloud provider mapping, cost estimation, Terraform stubs |
| `mobile-engineer`            | `react-dev`\*                                                            | React Native component development                       |
| `plan`                       | `system-architect`, `orchestrator-patterns`, `t3-monorepo-patterns`, `cc-reference`\* | Implementation planning, multi-agent orchestration, monorepo structure |
| `playwright-validator`       | `webapp-testing`, `qa-test-planner`                                      | Browser automation validation                            |
| `security-reviewer`          | `better-auth-best-practices`, `bb-fortify`                               | Auth security audits; B&B Fortify SAST/DAST scans        |
| `stripe-engineer`            | `database-schema-designer-ext`, `better-auth-best-practices`, `t3-code-patterns`\* | Stripe payment integration                               |
| `technical-writer`           | `crafting-effective-readmes`                                             | Documentation tasks                                      |
| `types-engineer`             | `t3-code-patterns`\*                                                     | TypeScript type definitions, shared type packages        |
| `ui-engineer`                | `shadcn`\*, `react-dev`, `nextjs-app-router`, `frontend-design`, `ui-ux-pro-max`, `motion-and-transitions`, `vercel-react-best-practices`, `reui-gantt-calendar-patterns`, `frontend-api-contracts`, `state-handling`\*, `t3-code-patterns`\* | Building React components, animations, API contracts     |
| `ux-specialist`              | `design-system-starter`, `ui-ux-pro-max`, `frontend-design`, `motion-and-transitions` | UX design, accessibility audits, Figma-to-code, microinteractions |

> \* = preloaded via agent frontmatter `skills` key (available without manual invocation)

> **Known drift**: `dev-health-check` and `technical-writer` no longer exist as agent definitions
> (verified against `agents/` 2026-07-06) — this table carries rows for both, moved verbatim from
> the pre-split SKILL.md rather than silently edited during the reference-split remediation. Flag
> for a separate cleanup pass, not fixed here.

## Engineering-Discipline Skills (obra/superpowers)

Six methodology-enforcement skills cross-cut multiple agents. Unlike the domain table above, these
are broad by design: every engineer benefits from `test-driven-development`, every agent benefits
from `verification-before-completion`.

| Skill | Agents | Rationale |
| ----- | ------ | --------- |
| `test-driven-development` | `api-engineer`, `db-engineer`, `ui-engineer`, `tdd-test-writer`, `tdd-implementer`, `tdd-refactorer`, `stripe-engineer`, `e2e-engineer` | All engineers that write or modify code should follow RED-GREEN-REFACTOR |
| `systematic-debugging` | `db-analyst`, `infra-analyst`, `e2e-engineer`, `codebase-health-orchestrator` | No fix without root-cause investigation — applies wherever failures are diagnosed |
| `verification-before-completion` | ALL agents | Reinforces `rules/CORE.md` Iron Law #1 (evidence-before-claim) at the skill-discovery surface |
| `writing-plans` | `plan`, `architecture-reviewer` | Engineer-facing implementation briefs — distinct from product PRDs |
| `receiving-code-review` | `api-engineer`, `db-engineer`, `ui-engineer`, `tdd-test-writer`, `tdd-implementer`, `tdd-refactorer`, `stripe-engineer`, `e2e-engineer` | All engineers process inbound review feedback before implementing |
| `brainstorming` | `plan`, `explore`, `architecture-reviewer` | Reinforces `rules/CORE.md` Iron Law #2 (Design Gate) for multi-file changes |

## Maintenance

Generic maintenance principles (keep it short, review periodically, treat as additive, flag
drift): `agent-tooling` skill's `references/skill-vs-agent-mapping.md` § Maintaining a
Skill-to-Agent Mapping. cc-specific: review on `/workflow:evolve` cycles; skill search index at
`~/.claude/scripts/bin/skill-index.json` (regenerate when skills change).

## Quick Reference

| Task                   | Command                                            |
| ---------------------- | -------------------------------------------------- |
| Invoke known skill     | `Skill({ skill: "skill-name" })`                   |
| Find skills by keyword | `Skill({ skill: "find-skills", args: "keyword" })` |
| List all skills        | `Skill({ skill: "find-skills", args: "" })`        |
