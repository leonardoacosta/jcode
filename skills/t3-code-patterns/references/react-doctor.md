
# React-Doctor: Fleet React Scanner

`react-doctor` (npx, zero-install) is a deterministic React scanner — 100+ rules across bugs,
performance, accessibility, security, and maintainability/architecture. It complements, not
replaces, ESLint: plain ESLint presets don't catch unstable context values, index keys on
reorderable lists, effect misuse, or auth tokens landing in web storage. Adapted from
millionco/react-doctor (`docs/recon/millionco-react-doctor.md` Card A).

## Post-change regression check

Run this after any React change in a fleet repo (acme, storefront, operations, backoffice, ...):

```bash
npx react-doctor@latest --verbose --scope changed
```

`--scope changed` limits the scan to files touched since the base branch — the score (0-100,
counting unique rule keys, not occurrences) must not regress relative to the pre-change
baseline. Treat a regression the same as a failing gate: fix before commit, or escalate per the
Scope Explosion rule if the fix is out of scope.

## `--json` contract summary

```bash
npx react-doctor@latest --json --yes
```

- `schemaVersion: 3` — versioned report shape.
- Each diagnostic carries a deterministic `id` (`<file>::<line>:<col>::<plugin>/<rule>::<digest>`),
  a `plugin/rule` key, severity, category, and `normalizedFilePath:line`.
- Per-project `complete` flag + `skippedCheckReasons` — **coverage honesty is load-bearing**: an
  empty `diagnostics` array does NOT mean "clean" unless `complete` is also true. A scan reported
  incomplete is reported as incomplete, never as clean (same "config presence != runtime
  liveness" doctrine as the ratchet lane — see `rules/TOOLING.md` § Config Ratchet Lane).

## Flags

| Flag | Effect |
| --- | --- |
| `--scope changed` | Limit to files diffed against the base branch |
| `--json` | Machine-readable report (schemaVersion 3) |
| `--verbose` | Human-readable per-finding detail |
| `--yes` | Skip the npx install confirmation prompt (non-interactive/CI use) |

## License note

react-doctor is a standalone npx tool consumed as-is — nothing from its source is vendored or
copied into this repo. This reference documents invocation and contract only.
