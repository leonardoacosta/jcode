## Context
Sessions are JSON files under `~/.jcode/sessions/`; `Session` carries `is_debug`, `saved`, `messages`. Only `.bak` pruning exists.

## Decisions
- **Pure classifier**: `classify(meta, now, policy) -> Keep(reason) | Candidate(reason)`; reasons `saved`, `live`, `unparseable`, `debug_expired`, `tiny_expired`, `recent`.
- **Rule order**: saved, live, unparseable keep rules run first so no later rule overrides them.
- **Dry-run default**: report counts, bytes, per-reason totals, sample ids. `--apply` with `enabled = false` exits non-zero naming the key.
- **Safety re-check**: re-read the file and re-check `saved` immediately before delete; skip on parse failure.
- **Auto GC**: startup background thread (like `jcode-session-bak-prune`) only when `enabled && auto_gc`.
- **Index**: debug filtering at index build; deleted ids dropped after apply.

## Risks
- Irreversible loss: mitigated by trash dir (open decision 1), saved protection, dry-run default, two-key opt-in.
