---
name: tc-reference
description: Reference lookup tables for tribal-cities (tc) — domain glossary, key routes, full build-command reference, and /apply batch ordering. Priceless-scope project (tc's primary use case). Use when working in tribal-cities and you need the burn/camp/ticket/volunteer domain glossary, a route's purpose, a specific pnpm/turbo command, or how /apply batches DB->API->UI->E2E for this repo.
---

# tribal-cities Reference Tables

Split out of `.claude/CLAUDE.md` per the CLAUDE.md-Split Pattern (`rules/TOOLING.md` § CLAUDE.md-Split Pattern) — these are lookup tables, not foot-gun rules, so they don't need to be paid on every turn.

## Domain Glossary

| Term | Meaning |
| ---- | ------- |
| Burn Series | A recurring event series (e.g. annual burn) containing individual burn events |
| Burn | A single event instance within a burn series |
| Camp | A themed camp within a burn — a group of participants |
| Participant | An attendee registered for a burn event |
| Volunteer | A participant who signs up for volunteer shifts |
| Ticket | Entry credential purchased for a burn event |

## Key Routes

| Route | Purpose |
| ----- | ------- |
| /burns | Burn event creation and management |
| /tickets | Ticket purchase flow |
| /check-in | Attendee check-in at burn |
| /volunteers | Volunteer signup and shift management |
| /camps | Camp registration and management |

## Build Commands

| Command | Purpose |
| ------- | ------- |
| `pnpm dev` | Start dev server |
| `pnpm build` | Full monorepo build |
| `pnpm typecheck` | TypeScript check (no emit) |
| `pnpm lint` | ESLint |
| `turbo run build` | Turbo-cached build |
| `pnpm --filter @tc/db db:generate` | Generate a SQL migration from `src/schema/` changes (commit the result) |
| `pnpm --filter @tc/db db:migrate` | Apply migrations (advisory-locked, dev/main-gated) — runs on deploy via the Vercel buildCommand |
| `pnpm --filter @tc/db db:push` | LOCAL schema iteration ONLY — never on deploy/build/CI (pre-commit guard enforces) |

**Quality gate order:** `pnpm typecheck && pnpm lint && pnpm build`

## Batch Ordering (for /apply)

| Batch | Scope | Gate |
| ----- | ----- | ---- |
| DB | `packages/db/` schema (edit schema → `db:generate` → commit migration) | `tsc --noEmit` |
| API | `packages/api/src/routers/` + Zod validators | `pnpm build` |
| UI | `apps/web/src/` pages + components | `pnpm build` |
| E2E | `apps/web/e2e/` tests | `pnpm lint && pnpm test` |
