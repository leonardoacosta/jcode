## Why

The completed personal-prompt pilot produced 35 valid successful trials and no demonstrated candidate benefit. Repeating those easy cases cannot establish which changes help. The next step needs automated discovery, matched experiments, unseen confirmation, and an honest stopping rule.

## What Changes

- Extend the existing isolated runner with a resumable queue of single-atom personal-prompt experiments across the five reviewed improvement families.
- Keep the frozen source-default and current personal bases as controls; modify only disposable personal candidates. Use OmniRoute `gpt-6-luna`, matched `medium` reasoning, and the user's existing uncapped usage authorization.
- Screen baseline weaknesses or measurable planning overhead before testing candidates. Advance at most one candidate per family to previously sealed confirmation cases; report boundary, outcome, burden, and validity separately.
- Freeze inputs, metrics, candidate bounds, and stop rules; retain failed, interrupted, inconclusive, and rejected observations. Stop at confirmed leads, plateau, exhausted coverage, or an actual blocker.
- Produce a reviewable evidence report and candidate diffs. Prompt adoption remains a separate human decision.

## Capabilities

### New Capabilities

None.

### Modified Capabilities

- `workflow-behavior-evaluation`: Add automated experiment selection, frozen family runs, conservative assessment, unseen confirmation, and durable recovery without changing the completed pilot's historical schedule.

## Impact

Implementation should extend `scripts/prepare_personal_prompt_eval.py`, reuse its private native-API confinement and the completed campaign's grading procedure, and add only focused automation checks. Private descriptors, fixtures, and evidence remain under `target/personal-prompt-eval/`; curated findings belong to this change. No new dependency, service, skill, global hook, provider default, live prompt edit, shared-daemon replacement, publication, or runtime ranking of Codex/Claude Code/Pi is included.

This is a proposed design, not implemented automation or a new behavioral result. See [design.md](design.md) for the evidence threshold and [tasks.md](tasks.md) for the execution path.
