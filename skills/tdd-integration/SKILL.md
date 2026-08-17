---
name: tdd-integration
description: Orchestration spec for the 3-agent RED-GREEN-REFACTOR TDD loop. Explicit-only, agent-frontmatter-loaded.
allowed-tools: Read, Glob, Grep
---

# TDD Integration (Red-Green-Refactor Orchestration)

> The TDD inner loop, expressed as three specialized agents handing structured payloads to one
> another. Source: alexop pattern adapted for Bun/Vitest/T3 Turbo.

## Gate Rules (Non-Negotiable)

Each agent runs in its own isolated context — `tdd-implementer` never sees whether RED actually
failed, it only sees what `tdd-test-writer` reported. Nothing enforces the RGR order except the
orchestrator choosing not to dispatch the next phase until the previous one's return proves its
claim. That's why gating lives here and not inside any one agent: the whole point of splitting
RGR across three agents is that no single agent can self-certify its own phase transition.

1. **No GREEN until RED is confirmed.** Verify `tdd-test-writer` returned a failure before
   dispatching `tdd-implementer`. If the test passed unexpectedly, the test is wrong —
   re-dispatch RED with that signal. Skipping this check means GREEN could be "passing" a test
   that was never actually red, which proves nothing about the implementation.
2. **No REFACTOR until GREEN.** Verify `tdd-implementer` returned a passing test before
   dispatching `tdd-refactorer`. Refactoring code that doesn't pass yet has no safety net — the
   refactorer's whole mandate assumes a green baseline to refactor against.
3. **Default-skip refactor.** If the refactorer returns `decision: "skipped"`, that is the
   correct answer most of the time. Do not re-dispatch arguing for a refactor — forcing a
   refactor the agent judged unnecessary reintroduces the speculative-cleanup risk the
   default-skip exists to prevent.
4. **One test per loop.** If the requirement spans multiple acceptance criteria, run the full
   loop once per criterion. Do not batch — batching hides which criterion a given RED/GREEN pair
   is actually proving.

## The Three Agents + Handoff Contract

Three agents hand structured payloads to one another: `tdd-test-writer` (RED) → `tdd-implementer`
(GREEN) → `tdd-refactorer` (REFACTOR), each with its own job, prohibited actions, and required
return fields, spliced by the orchestrator into an assert-gated loop per acceptance criterion.

**MANDATORY**: read `references/agent-handoff-contract.md` before the first dispatch of this loop
in a session — it holds the full per-agent field table and the orchestrator pseudocode with the
actual `assert` conditions each gate above compiles down to.

## Scope

Use for: a new feature in any T3 Turbo project where the surface is testable, a bug fix where a
regression test should land before the fix, service-layer logic in
`packages/api/src/services/` (`createCallerFactory`), or component logic in `packages/ui` /
`apps/nextjs` testable via RTL.

Skip the loop for: pure refactoring with no behavior change (run existing tests instead),
E2E-only scenarios (see `playwright-auth`), cosmetic/formatting changes, or spike/prototype code
that won't ship.

## Stack-Specific Notes (oo / T3 Turbo)

- **Test runner**: Vitest 3.2.4. Run via `pnpm --filter <package> test -- <file>`.
- **Per-package config**: see `oo-vitest-patterns` skill — every package has its own
  `vitest.config.ts` with thresholds at measured floor.
- **Service-layer tests**: use `createCallerFactory` (not raw HTTP). See `oo-vitest-patterns` →
  Recipe section.
- **Mocking**: prefer pglite for DB; use `vitest-mock-extended` for typed external-service
  mocks (Stripe, Resend). Never module-mock `@oo/db` — see project NEVERs.
- **Coverage gate**: `--coverage` is wired in `.github/workflows/ci.yml` `unit-tests` job; the
  thresholds in each package's `vitest.config.ts` are at the measured floor (post-vitest-ladder
  commit `1e5b6b9`). New code should not drag coverage below floor — but this is a CI gate, not
  a per-loop concern.

## Anti-Patterns (Orchestrator-Level)

| Smell | Why | Fix |
|---|---|---|
| Skipping RED — "I know the test will fail" | Source-only confidence; fails Verification Iron Law | Always run RED |
| Re-dispatching RED to write a "stronger" test | Mid-loop feature creep | The test was written for ONE criterion; if more are needed, run the loop again |
| Running the full suite mid-loop | Slow; flakes pollute the signal | Target test or package-level only inside the loop; suite at the end |
| Forcing the refactorer to refactor | Default-skip is the right answer | Trust the agent's reasoning |
| Treating the loop as bureaucracy | Loses TDD's design feedback | Each RED is a design decision in disguise; read the test before approving |

## Activation

Three options for triggering the loop:

1. **Explicit invocation**: `Skill({ skill: "tdd-integration" })` then dispatch `tdd-test-writer`.
2. **Implicit via test-driven-development skill**: that skill points here for projects with
   the three-agent pattern installed.
3. **Optional UserPromptSubmit hook** (alexop): a hook can inject this skill when prompts contain
   `(test|tdd|red|green|refactor)`. See alexop's blog for the script shape; not installed by
   default in this fleet — file `oo-tdd-hook` if you want to enable it per-project.
4. **`/apply` E2E-batch routing** (wired 2026-07-05, `agent-fleet-and-prime-visibility`): a
   tasks.md line tagged `[owner:tdd-integration]` in the `## E2E Batch` section triggers this
   loop automatically — no manual skill invocation needed. See
   `commands/apply/references/execution-model.md` § E2E batch and
   `commands/apply/references/batch-template.md` § TDD-Loop Dispatch for the orchestrator-side
   contract. `test-writer` remains the owner tag for the single-agent exceptions it documents
   (backfill, hot-fix regression, quick coverage repairs).

## Related Skills

| Skill | When |
|---|---|
| `test-driven-development` | Discipline doc — the principle behind the agents |
| `oo-vitest-patterns` | oo-specific Vitest config, recipes, NEVERs |
| `oo-react-testing` | RTL / jsdom patterns for component-level RED |
| `service-layer-style` | When the SUT is a service in `packages/api/src/services/` |
| `simplify` | Sometimes useful in REFACTOR phase |
| `verification-before-completion` | Iron-law guardrail every agent already follows |

## Source

- Pattern: https://alexop.dev/posts/custom-tdd-workflow-claude-code-vue/
- Stack-agnostic core; this skill swaps the Vue-specific bits for T3 Turbo / Vitest.
