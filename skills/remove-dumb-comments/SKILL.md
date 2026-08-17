---
name: remove-dumb-comments
description: Flag comments that merely restate code instead of explaining why, rank redundant comments with git age and nearby context, and remove only explicitly approved comments. Use this Jcode skill whenever the user asks to remove dumb, redundant, obvious, stale, or noisy comments, clean up comments, or invoke `/remove-dumb-comments [<number>|all]`.
---

# Remove Dumb Comments

Flag comments that say *what* the code already says; keep every comment that explains *why*. The user chooses which flagged comments to remove.

## Invocation

| Command | Behavior |
|---|---|
| `/remove-dumb-comments` | Find the 10 lowest-value comments. |
| `/remove-dumb-comments <number>` | Find that many lowest-value comments. |
| `/remove-dumb-comments all` | Find every low-value comment. |

## Never Remove

Keep any comment that carries a *why* the code cannot convey:

- backports, compatibility, or version-specific behavior;
- infrastructure, deployment, or architecture;
- workarounds, gotchas, or non-obvious reasons;
- documentation, specifications, RFCs, or ADRs;
- bugs, issues, tickets, or contextual TODOs/FIXMEs;
- intent, trade-offs, or constraints.

When unsure, keep it. Flag only pure restatements.

## Judgment Rubric

Classify a comment as **Remove** only when all of these are true:

- The code immediately beside it already communicates the same fact.
- Removing it would not hide behavior, intent, constraints, or historical context.
- It is not an API/documentation comment whose absence would reduce the public contract.
- It does not preserve a warning that is easy to miss from the implementation alone.

Classify it as **Keep** when any of these apply. Prefer `Keep` when the comment is ambiguous,
even if it is somewhat verbose. A comment can be stale or awkward without being redundant.

Do not infer approval from the user's initial request. The initial request authorizes discovery
and recommendation only. Deletion requires an explicit approval response, and approval applies
only to the listed candidates in the current review.

## Workflow

1. Resolve the limit from the invocation (default 10).
2. Search source files; skip generated output, vendored dependencies, lockfiles, and documentation.
3. Inspect one to three adjacent code lines and apply the Judgment Rubric. Do not judge from the comment text alone.
4. Rank candidates from most redundant to least, but never trade confidence for the requested count.
5. Get each candidate's age with `git blame` (see Comment Age).
6. Present the table below, then ask whether to remove all recommended.
7. If yes, treat every `Remove` item as approved. If no, ask `Remove` or `Keep` for each, naming it by its exact text, not its location.
8. Before editing, re-check that the file and line are unchanged. Remove only approved comments, preserving surrounding formatting and code.
9. Run the project's focused tests, lint, and typecheck when available. Report commands that were unavailable or already failing; do not silently claim success.

If the requested number exceeds the number of high-confidence candidates, report fewer rather
than padding the list with questionable comments. If a file has uncommitted edits, avoid
rewriting unrelated lines and mark the candidate age `uncommitted`.

### Edge Cases and Fallbacks

- Treat a multiline block comment as one candidate. Quote its exact text without truncating the
  rationale, and remove the whole block only after approval.
- For comments inside an expression, loop, or chained call, inspect the complete statement before
  judging. A nearby comment may explain evaluation order or a surprising side effect.
- Skip generated-file markers, license headers, compiler directives, suppression pragmas, and
  formatter directives even when they look mechanically obvious.
- Prefer the repository's parser or search tooling when available. If unavailable, use targeted
  text search and state that syntax-aware classification was not possible.
- If `git blame` fails because the file is untracked, copied, or outside the repository, report
  `uncommitted` or `unknown` rather than inventing an age.
- If verification fails after an approved removal, restore only the removed comments, rerun the
  failing check, and report the rollback. Never broaden the edit while diagnosing the failure.

Delegate the read-only search to a fast, low-reasoning subagent when one is available: request at most the limit, each with exact path, line, comment text, and one to three adjacent code lines. Otherwise search directly.

## Comment Age

For each candidate, run:

```bash
git blame -L <line>,<line> --date=relative -- <file>
```

Use the relative date; mark uncommitted lines `uncommitted`.

## Required Output

Use exactly these columns:

```markdown
| Comment | Age | Why |
|---------|-----|-----|
| `// increment the counter` | 8 months ago | *Remove.* Restates `count++` verbatim. |
| `/** Returns the user id. */` | 3 weeks ago | *Remove.* Describes the function word by word. |
| `// debounce avoids hammering the API on each keypress` | 1 year ago | *Keep.* Explains intent, not mechanics. |
```

- **Comment**: Include the exact comment text in backticks.
- **Age**: Use the relative `git blame` age.
- **Why**: Start with `*Remove.*` or `*Keep.*`, then give one short reason.

## Feedback

After presenting the table, ask:

> **Remove all recommended?**
> - Yes, remove all recommended
> - No, I want to review each comment
