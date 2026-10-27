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

### Execution progress — 2026-10-27

The isolated controller is implemented in the existing runner, with local commits `3b91b6c2a`, `791e7ed20` and `9955ed740`; index-grading repair `29b469f12` adds semantic preservation checks, and `6b5816aa7` binds trial checkpoints into the audit. Three retained registrations contain 58 actor attempts: 55 runtime-valid, 3 invalid; V3 also contains 24 separate assessor attempts, five with unresolved semantic pointers. No candidate has been tested, confirmed or adopted. V3 stopped on OmniRoute HTTP 404; routing and compaction remain unsupported by the reduced adapter. Final independent Luna review approved the repaired implementation and truthful blocked report with no findings; 40 focused tests pass. Required live execution and Apply archive remain pending; this request is not resolved. See `openspec/changes/automate-personal-prompt-experiments/evaluation.md`.

---
