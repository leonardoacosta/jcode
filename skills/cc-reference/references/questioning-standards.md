# Questioning Standards

> Derived from gepetto interview-protocol. Applies to all clarification and discovery.
> Load WHEN running a `/feature`, `/bootstrap:*`, discovery, or any flow that clarifies requirements
> with the user.

## Tool Enforcement

**Use AskUserQuestion tool** for all multi-question clarification — not freeform text. The tool
enforces structured options, prevents vague closers, and gives the user a scannable UI.

| When | Use AskUserQuestion | Use freeform text |
|------|---------------------|-------------------|
| 2+ questions with options | Always | Never |
| Single yes/no confirmation | Overkill | Fine |
| Presenting a recommendation | Overkill | Fine |
| Scope/design/trade-off decisions | Always | Never |

## Philosophy

Ask as a **senior expert accountable for the outcome** — not as an assistant gathering a
checklist. Assume the initial request is incomplete. Your job is to surface what the user
knows but hasn't said, so you can proceed without making wrong assumptions.

## Technique

- **2-4 focused questions per message** — never a wall, never a single vague closer
- **Open-ended over yes/no** — "What happens when X fails?" not "Should we handle failures?"
- **Skip the obvious** — if the answer is already in the spec/context, don't ask
- **Dig when complexity surfaces** — if an answer reveals depth, follow it before moving on
- **Summarize to confirm** — periodically reflect back your understanding, let them correct it

## Good vs Bad Questions

| Good | Bad |
|------|-----|
| "What happens when X fails — retry, log, or surface to user?" | "Anything else?" |
| "Are there existing patterns in the codebase we should follow for Y?" | "Is that all?" |
| "What's the expected scale — dozens, thousands, or millions?" | "Do you have other requirements?" |
| "Which edge cases do you care about most?" | "Can you tell me more?" |

**Bad questions share one trait**: they hand control back without extracting anything. They are
filler. Never use them.

## When to Stop

Stop when you can:
1. Proceed without making a wrong assumption about requirements
2. Handle all edge cases the user cares about
3. Write or implement without needing to guess intent

If the user answers "I don't know" or "up to you" consistently across a round — they've
delegated that decision. Accept it and move on. Don't keep asking.

## Read-Before-Ask & Premise Verification

Hardened by `hoist-run-policy-and-ask-contract` off the 2026-07-20 churn audit's
`ask-construction-defects` cluster (7 of 55 asks dead in `InputValidationError`, false-premise
walk-backs, a 47-min context-starved stall on "I need enough context to make a decision").

**(a) Read-before-ask.** Before firing an `AskUserQuestion` about a TRACKED item (a bead, a
`tasks.md` task, an OpenSpec proposal), check for a prior decision first: `bd comment`s on the
bead, the task's own checkbox/annotation state in `tasks.md`, and — for anything under
`openspec/changes/<slug>/` — that spec's `decisions.jsonl` (see
`commands/apply/references/user-gate-preflight.md` § Decision Ledger for the row schema). If a
prior decision exists, CITE it (with its date) in the question body instead of re-deriving the
question from scratch — re-asking a decision already on record is the re-ask-chain failure mode
the churn audit measured (19 re-ask chains across 30 sessions).

**(b) Premise verification.** Every factual premise an ask's wording asserts MUST be verified by
at least one command immediately before the ask fires — never assumed from memory, a stale
cache, or an earlier turn's state. The churn audit's two counterexamples: an ask that asserted a
nova MCP server's queue was "empty" (it wasn't — the check that would have shown otherwise was
never run before asking) and an ask that asserted a dev branch was "stale" (a single `git fetch`
+ `git log` would have shown it wasn't) — both forced a walk-back mid-conversation once the real
state surfaced, burning the exchange the premise-check would have skipped entirely. Treat "I
already know this" as a hypothesis to confirm, not a fact to assert.

**(c) Recommendation required.** Every ask ships a recommendation with per-option tradeoffs,
recommendation FIRST, per `feedback_recommend_and_defend` — never a bare option list. This
applies to every `AskUserQuestion` call, not only the discovery-flow ones this skill otherwise
scopes to.

**(d) The >4-question shape is schema-caught, not hook-catchable.** The Anti-Patterns rule below
("NEVER ask more than 4 questions") is enforced by `AskUserQuestion`'s own tool-input schema
validation, which rejects a 5+-question array before the call ever reaches a `PreToolUse` hook —
by the time a hook could inspect `tool_input`, the call has already failed validation and
retried. This is a DIFFERENT failure class from the nested-tag malformed-ask footgun (memory:
`askuserquestion-nested-tags-parse-failure`), which IS hook-catchable because embedding XML
tool-call syntax inside a string field does not itself violate the schema — only
`scripts/hooks/askq-lint.sh` catches that class; nothing catches (or needs to catch) the
question-count class, since the harness already does.

## Anti-Patterns

- **NEVER** ask more than 4 questions in a single message
- **NEVER** ask questions the context already answers
- **NEVER** use vague closers ("Anything else?", "Is there anything I missed?")
- **NEVER** ask hypothetical questions when you can make a reasonable judgment call
- **NEVER** ask about things that don't change what you'd do
