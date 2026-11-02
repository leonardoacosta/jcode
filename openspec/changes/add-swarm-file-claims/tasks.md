# Tasks

## 1. Claim store

- [x] 1.1 Add a canonical-path to session claim map in swarm server state, persisted with swarm state.
- [x] 1.2 Add `swarm.file_claims` config: `off`, `warn` (default), `block`.

## 2. Enforcement

- [x] 2.1 Claim on first write by edit/write/patch tools.
- [x] 2.2 On conflicting write, return a warning naming the holder, or refuse in `block` mode.
- [x] 2.3 Release claims on finish, stop, and orphan detection (via `recover-swarm-workers-after-reload`).
- [x] 2.4 Show active claims in `swarm status`.

## 3. Verification

- [x] 3.1 Unit tests: claim, conflict warn, conflict block, release on stop and orphan.
- [x] 3.2 Test claims survive server reload.
- [x] 3.3 Run `openspec validate add-swarm-file-claims --strict`.
- [x] 3.4 Exercise real tool registry calls in isolated processes for default/off/warn/block, edit/write/both patch formats, coordinator bypass, blocked multi-file/move/delete preflight, status snapshots, resumed workers, and persisted orphan recovery.
