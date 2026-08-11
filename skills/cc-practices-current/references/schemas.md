# State File Schemas

`state/last-checked.json`:

```json
{
  "last_checked": "2026-04-13T18:09:04-04:00",
  "docs_hash": "sha256:abc123...",
  "github_latest": "v2.1.104",
  "npm_latest": "2.1.104",
  "npm_stable": "2.1.92",
  "beads_latest": "v1.1.0",
  "openspec_latest": "v1.0.3",
  "last_change_detected": "2026-04-10T19:03:41Z"
}
```

`last_change_detected` is when we most recently transitioned from unchanged → changed. The
"Added since" section in features.md should anchor against this — everything newer than this
timestamp is the delta.

`state/decisions.json`:

The persistent decision log for adoption signals. Owned by `/workflow:evolve`; the
`cc-feature-analyst` agent fills the `research` block, the user fills the `history` entries via
the orchestrator gate. Append-only: a `defer` today can become `implement` next quarter without
losing the prior call.

```json
{
  "version": 1,
  "decisions": {
    "<signal-id>": {
      "first_seen": "YYYY-MM-DD",
      "version": "v2.1.95",
      "title": "PreToolUse defer permission decision",
      "area": "hooks",
      "official_source": "https://...",
      "research": {
        "why": "...",
        "community_usage": "...",
        "our_leverage": [{"target": "...", "action": "..."}],
        "risk": "...",
        "effort": "trivial|small|medium|large"
      },
      "history": [
        {
          "date": "YYYY-MM-DD",
          "verdict": "implement|defer|skip|legacy-import",
          "rationale": "...",
          "ref": "commit:HASH | bd:ID | spec:NAME | null"
        }
      ]
    }
  }
}
```

**Lifecycle:**

| Phase | Writer | Effect |
|-------|--------|--------|
| Bootstrap (one-time) | `scripts/decisions-bootstrap.sh` | Imports legacy `latest.json` recommendations as `verdict: legacy-import` |
| Per-run delta detection | `/workflow:evolve` orchestrator | Identifies signals not yet in `decisions.json` |
| Research fill | `cc-feature-analyst` agents (parallel) | Writes the `research` block per signal |
| Verdict capture | `/workflow:evolve` orchestrator gate | Appends to `history[]` after user confirmation |

**Idempotency:** Re-running `/workflow:evolve` skips signals whose latest `history[]` entry was
made after the most recent `last_change_detected`. Use `--re-research <id>` to force
re-analysis of a specific signal.
