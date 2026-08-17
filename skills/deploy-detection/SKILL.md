---
name: deploy-detection
description: Detects deployment type and branch strategy; script-backed data producer for /p2p. Explicit-only.
user-invocable: false
disable-model-invocation: false
allowed-tools: Bash(${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/deploy-detect *)
---

# Deploy Detection

The `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/deploy-detect` script is the single source of truth for deploy-type,
branch-strategy, and prod-URL detection. This skill runs it at render time and inlines the
resulting JSON directly into your context.

## Live Detection

```!
${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/deploy-detect --json
```

## How to Read the Injected JSON

The block above is replaced at skill invocation time with a JSON object matching this shape:

```json
{
  "timestamp": "2026-04-13T00:00:00Z",
  "project": "/absolute/path",
  "deploy_type": "vercel|docker|git_hook|github_actions|unknown",
  "branch_strategy": "dev_to_main|develop_to_main|feature_to_main|feature_to_master|unknown",
  "prod_url": "https://... (may be empty)",
  "detected_signals": ["vercel.json", ".github/workflows/"]
}
```

**Reading rules:**

1. Trust `deploy_type` as the routing key for downstream dispatch.
2. If `deploy_type == "unknown"`, fall back to the default git-push path and log a warning; never
   guess.
3. `prod_url` may be empty even when `deploy_type` is known — derive it from project config if
   needed, do not re-run detection.
4. `detected_signals` is diagnostic only — use it for log messages, not for branching.
5. The script exits 0 even on failure; an `error` key may be present. Check for it before trusting
   other fields.

## Interpretation Matrix

| deploy_type | Monitor helper | Health check |
|-------------|---------------|--------------|
| `vercel` | `monitor_vercel_deploy "$PROJECT" "$BRANCH"` | `curl $prod_url/api/health` |
| `git_hook` | None — hook fires synchronously on push | `curl` against homelab/Tailscale URL |
| `docker` | Poll `docker compose ps` | `docker compose exec app curl localhost:$PORT/api/health` |
| `github_actions` | `gh run watch --exit-status` | `curl` against whatever the workflow deploys to |
| `unknown` | Skip monitor, log warning | Skip, log warning |

## Footguns

- **`detected_signals` is diagnostic-only.** It's the evidence list (`vercel.json`,
  `.github/workflows/`) that produced the `deploy_type` verdict — use it for a log line, never
  as a branch condition. Branching on raw signals instead of `deploy_type` re-implements the
  script's own detection logic ad hoc, and will drift from it the next time a new signal is added.
- **Never re-run detection to derive `prod_url`.** A known `deploy_type` with an empty `prod_url`
  is a valid state (reading rule 3) — derive the URL from project config (env var, project.toml)
  instead of shelling out to `deploy-detect` again expecting a different answer.
- **`error`-key precedence.** If the JSON carries an `error` key, treat every other field as
  untrustworthy — don't cherry-pick `deploy_type` or `prod_url` out of a payload that already
  told you detection failed. Fall back to the default git-push path per reading rule 2.

## Related

- **Source of truth**: `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/deploy-detect`
- **PR workflow detection** (complementary): `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/pr-detect`
- **Gate strategy detection** (quality gates): `${CLAUDE_PLUGIN_ROOT:-$HOME/.claude}/scripts/bin/gates-detect`
- **Environment / deploy operations**: `deploy-and-env` skill
