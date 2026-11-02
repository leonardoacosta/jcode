# Overnight task-card schema

Create one `*.json` file per meaningful task. Keep it current while you work. The generated review page and TUI progress card read these files continuously.

Required spirit: make the morning review objectively useful. Each completed or important attempted task should show why it was selected, what was true before, what changed after, and exactly how it was validated.

```json
{
  "id": "task-001",
  "title": "Fix deterministic provider reload timeout",
  "status": "active | completed | blocked | deferred | failed | skipped",
  "priority": "high | medium | low",
  "source": "GH issue | failing test | static analysis | code quality | user request",
  "why_selected": "Objective, bounded, high-confidence reason for choosing this task.",
  "verifiability": "How we can prove the problem and prove the fix.",
  "risk": "low | medium | high",
  "outcome": "Concise final outcome or current state.",
  "before": {
    "problem": "Observed bug/code state before work.",
    "evidence": ["validation/task-001-before.txt"]
  },
  "after": {
    "change": "Changed behavior or code state after work.",
    "files_changed": ["src/example.rs"],
    "evidence": ["validation/task-001-after.txt"]
  },
  "validation": {
    "commands": ["cargo test provider_reload"],
    "result": "passed",
    "evidence": ["validation/task-001-test.txt"]
  },
  "followups": ["Optional remaining safe next step"],
  "updated_at": "2026-05-01T08:00:00Z"
}
```

Notes:
- Use `active` for the task currently being worked.
- Use `completed` only when validation evidence exists or the task is intentionally documentation/issue-draft only.
- Use `blocked` when user input, credentials, external access, or taste is required.
- Use `deferred` or `skipped` when a task was considered but not pursued.
