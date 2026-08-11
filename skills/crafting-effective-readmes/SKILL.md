---
name: crafting-effective-readmes
description: Use when writing or improving README files for T3 Turbo projects. Covers monorepo apps, internal workspace packages, CLI tools, and APIs. Triggers on "write a README for [project]", "document this package", "add setup docs", "improve my README", or any request for project documentation. Provides decision matrix, project-type templates, and T3-specific anti-patterns.
source: ~/.agents/skills@2026-07-13
---


# Crafting Effective READMEs (T3 Stack)

## Step 1 — Identify Project Type

Determine the project type from context before loading any template.

| Type | Signals |
|------|---------|
| **T3 monorepo app** | Lives in `apps/`, Next.js, has `pnpm dev`, Doppler, Vercel deploy |
| **T3 internal package** | Lives in `packages/`, exports TypeScript, consumed by other packages |
| **CLI tool** | Rust / Go / Node binary, primary interface is the terminal |
| **OSS / public lib** | Intended for public release, needs badges, CONTRIBUTING, LICENSE |
| **Personal / side project** | Portfolio piece, experiment, solo dev |
| **Config / XDG directory** | Dotfiles, `~/.config/[tool]/`, script folders |
| **Internal service** | Team codebase, internal API, not open-sourced |

## Step 2 — Load the Right Template

**MANDATORY**: Load exactly one template based on the project type detected above. Do NOT load
templates that don't match.

| Project Type | Template to Load | Do NOT Load |
|-------------|-----------------|-------------|
| T3 monorepo app (`apps/`) | Use the inline T3 App template below | Any file in `templates/` |
| T3 internal package (`packages/`) | Use the inline T3 Package template below | Any file in `templates/` |
| CLI tool (Rust/Go/Node) | Use the inline T3 CLI template below | Any file in `templates/` |
| OSS / public library | **MANDATORY**: Read [`templates/oss.md`](templates/oss.md) before writing | `templates/internal.md`, `templates/personal.md`, `templates/xdg-config.md` |
| Personal / side project | **MANDATORY**: Read [`templates/personal.md`](templates/personal.md) before writing | `templates/oss.md`, `templates/internal.md`, `templates/xdg-config.md` |
| Config / XDG directory | **MANDATORY**: Read [`templates/xdg-config.md`](templates/xdg-config.md) before writing | `templates/oss.md`, `templates/internal.md`, `templates/personal.md` |
| Internal service (non-T3) | **MANDATORY**: Read [`templates/internal.md`](templates/internal.md) before writing | `templates/oss.md`, `templates/personal.md`, `templates/xdg-config.md` |

## Step 3 — Validate Sections

**MANDATORY**: Read [`section-checklist.md`](section-checklist.md) to confirm which sections are
required vs optional for this project type before finalizing.

**Do NOT load** `section-checklist.md` if you are only doing a quick single-section edit (e.g.,
fixing a typo or updating one env var table row).

## Step 4 — Apply Style Rules

**MANDATORY**: Read [`style-guide.md`](style-guide.md) and check the README against the common
mistakes list before delivering.

**Do NOT load** `style-guide.md` if only generating a structural scaffold (user will fill content).

## Step 5 — Reference Depth (Optional)

Load references only when the task requires depth beyond the template. Pick one — do not load all.

| Situation | Load |
|-----------|------|
| User asks about README philosophy / why sections matter | **MANDATORY**: Read [`using-references.md`](using-references.md) first, then follow its guidance to pick one reference |
| OSS project wants Standard README compliance | [`references/standard-readme-spec.md`](references/standard-readme-spec.md) |
| Need a full Standard README example | [`references/standard-readme-example-maximal.md`](references/standard-readme-example-maximal.md) |
| Minimal compliant OSS example | [`references/standard-readme-example-minimal.md`](references/standard-readme-example-minimal.md) |
| Understanding how readers scan READMEs | [`references/art-of-readme.md`](references/art-of-readme.md) |
| Section-by-section writing guidance | [`references/make-a-readme.md`](references/make-a-readme.md) |

**Do NOT load** any references file for T3 monorepo/package/CLI work — the inline templates below
and the T3 anti-patterns section are sufficient.

---

## README Judgment Framework — Deciding What Belongs

The templates and decision matrix below cover the cases this skill anticipated. New situations
come up that don't map cleanly onto a row in that matrix — a section type the templates don't
name, a project type that's a hybrid, content that seems useful but isn't listed either way.
Apply these three tests, in order, to decide:

1. **Audience test** — who actually opens this file, and what do they need to do right after
   reading it? A package consumer needs the exports and a usage example; a contributor needs
   local-dev setup; an on-call engineer needs the runbook. If the content doesn't serve the
   action the real reader takes next, it doesn't belong here even if it's true and relevant.

2. **Staleness-risk test** — does this content drift independently of the code, or does it drift
   the moment the code changes? Env var names and deploy targets change on their own schedule
   (low drift risk — safe to document in prose). Function signatures, tRPC procedure shapes, and
   schema fields change every time someone touches the code they describe (high drift risk — a
   README documenting them WILL go stale, silently, because nothing forces the doc update in
   the same commit). This is the actual reasoning behind "don't document tRPC signatures, use
   TypeScript types" — it's not a T3-specific rule, it's this test applied to that specific
   case. Apply the same test to any new content you're deciding whether to add: high-drift
   content belongs in the artifact that can't drift from itself (types, schema, code comments
   next to the thing), not in prose that has no mechanism keeping it current.

3. **Single-source test** — is this already documented at a broader scope (root README, a
   package's own types, an existing runbook)? If yes, link to it — don't fork a second copy that
   can now disagree with the original. This is why every app/package README opens with a pointer
   to the root README rather than re-explaining `pnpm install`: two copies of the same setup
   instructions are one that's wrong the moment the real one changes.

A section that fails any one of these tests is a candidate for cutting, even if the decision
matrix above doesn't explicitly forbid it — the matrix encodes known instances of these tests;
it isn't exhaustive.

---

## Decision Matrix (T3 Projects)

| Section | Monorepo App | Internal Package | Public CLI |
|---------|-------------|-----------------|-----------|
| Description (1–2 sentences) | MUST | MUST | MUST |
| Prerequisites | MUST | SHOULD | MUST |
| Local dev (`pnpm dev`) | MUST | — | MUST |
| Env vars (table) | MUST | — | SHOULD |
| Package API / exports | — | MUST | — |
| Usage examples | SHOULD | MUST | MUST |
| Architecture overview | SHOULD | — | — |
| Deployment | SHOULD | — | — |
| Runbooks / troubleshooting | SHOULD | — | — |
| License | — | — | MUST |

**Rule:** if setup belongs to the monorepo root (`pnpm install`, Doppler, Turbo), do NOT repeat
it — link to root README instead.

---

## Template: T3 Monorepo App (`apps/web`, `apps/dashboard`)

```markdown
# [App Name]

[One sentence: what this app does and who it's for.]

> Part of the [monorepo] monorepo. See [root README](../../README.md) for monorepo setup.

## Prerequisites

- Node 20+, pnpm 9+, monorepo installed (`pnpm install` from root)
- Doppler CLI with `DOPPLER_PROJECT=[project] DOPPLER_CONFIG=dev`

## Running Locally

\`\`\`bash
pnpm --filter [app-name] dev        # http://localhost:[PORT]
\`\`\`

## Environment Variables

| Variable | Required | Notes |
|----------|----------|-------|
| `POSTGRES_URL` | Yes | Neon — auto-injected via Doppler |
| `NEXTAUTH_SECRET` | Yes | Auth — auto-injected |

## Architecture

[2–4 sentences on key data flow. E.g., "All mutations go through tRPC routers in
packages/api — no direct DB access from the app layer."]

## Deployment

Deployed to Vercel. Branch → Preview. `main` → Production.

## Runbooks

### Reset local DB

\`\`\`bash
pnpm --filter @[scope]/db drizzle-kit push
\`\`\`
```

---

## Template: T3 Internal Package (`packages/db`, `packages/api`, `packages/ui`)

```markdown
# @[scope]/[package-name]

[One sentence: what this package exports and which consumers use it.]

## Exports

\`\`\`typescript
import { [MainExport] } from "@[scope]/[name]";
\`\`\`

| Export | Type | Purpose |
|--------|------|---------|
| `[ExportName]` | [function/type/const] | [What it does] |

## Usage

\`\`\`typescript
import { [thing] } from "@[scope]/[name]";
[minimal working example]
\`\`\`

## Adding [Schemas / Procedures / Components]

1. [Step 1 specific to this package's extension pattern]
2. [Step 2]

## Gotchas

- [Non-obvious behavior that will bite the next developer]
```

---

## Template: CLI Tool (Rust / Go / Node)

```markdown
# [cli-name]

[One sentence: what problem this solves and the primary command.]

## Install

\`\`\`bash
[cargo install / go install / npm i -g]
\`\`\`

## Usage

\`\`\`bash
[cli-name] [primary-command] [--flags]   # Most common case
[cli-name] --help                         # Full reference
\`\`\`

## Configuration

[Config file location and format, or env vars.]

## License

[LICENSE]
```

---

## T3 Monorepo Anti-Patterns

Each DON'T below is one instance of the Judgment Framework above (staleness-risk or
single-source test) — apply the framework directly when a new case isn't on this list.

- **DON'T** duplicate root-level setup (pnpm install, Doppler, Docker, Turbo) — link to root README.
- **DON'T** document tRPC endpoint signatures — use TypeScript types. READMEs go stale; types don't.
- **DON'T** put schema docs in READMEs — `packages/db/src/schemas/` + JSDoc is the source of truth.
- **DON'T** add contributing guide or LICENSE to individual `packages/*` — both live at root.
- **DON'T** document env vars for packages that don't read `process.env` directly (most don't).
- **DO** add a Gotchas section: Drizzle camelCase↔snake_case, peer dep constraints, circular imports.
- **DO** open every app/package README with `> Part of the [repo] monorepo. See root README for setup.`

## Quick Checklist

- [ ] Description: "what + why" in one sentence
- [ ] No duplicated root-level setup instructions
- [ ] Env var table only covers vars this layer actually reads
- [ ] All code examples are copy-paste runnable
- [ ] Gotchas covers anything that surprised you during development
