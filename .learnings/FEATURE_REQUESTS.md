# Feature Requests

## [FEAT-20261027-001] automated_personal_prompt_experiments

**Logged**: 2026-10-27T19:59:55.517218+00:00
**Priority**: high
**Status**: pending
**Area**: tests

### Requested Capability

Automatically vary experiments, assess outcomes, and continue until there are replicated useful leads or a declared evidence plateau.

### User Context

Mature Jcode using source-grounded ideas from Codex, Claude Code, and Pi. The completed 36-slot pilot had 35 valid successful trials, with no demonstrated benefit over the current personal prompt. Harder representative cases and unseen confirmation are needed rather than more repetitions of ceiling cases. The user authorizes uncapped usage through OmniRoute `gpt-6-luna`.

### Complexity Estimate

medium

### Suggested Implementation

Extend `scripts/prepare_personal_prompt_eval.py` with a frozen family queue, one bounded prompt atom per run, matched default/personal controls, conservative grading, resume/audit, sealed confirmation, and plateau stops. Keep live prompt adoption separate. Proposed contract: `openspec/changes/automate-personal-prompt-experiments/`.

### Metadata

- Frequency: first_time
- Related Features: personal_prompt_evaluation
- Source: user_feedback
- Existing log search: no project `.learnings/` or matching home/agent/Codex learning store found before creation.
- Promotion: none; one campaign is not evidence across distinct tasks for a permanent instruction change.

---
