# Effort Gates

Adaptive behavior based on `${CLAUDE_EFFORT}` (v2.1.120+):

| Effort | Behavior |
|---|---|
| `low` / `medium` | Read `state/cache/changelog.md` and `references/*.md` only. Skip `scripts/refresh.sh` web-search refresh; trust the cache. |
| `high` / `xhigh` / `max` | Run full refresh: upstream fetch + changelog diff + references regeneration when sources moved. |

Rationale: cached references are fresh enough for routine "is X deprecated?" lookups; the
refresh path is only load-bearing during `/workflow:evolve` and similar full-currency audits.
