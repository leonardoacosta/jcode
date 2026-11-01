# Tasks

## 1. Claim store

- [ ] 1.1 Add a canonical-path to session claim map in swarm server state, persisted with swarm state.
- [ ] 1.2 Add `swarm.file_claims` config: `off`, `warn` (default), `block`.

## 2. Enforcement

- [ ] 2.1 Claim on first write by edit/write/patch tools.
- [ ] 2.2 On conflicting write, return a warning naming the holder, or refuse in `block` mode.
- [ ] 2.3 Release claims on finish, stop, and orphan detection (via `recover-swarm-workers-after-reload`).
- [ ] 2.4 Show active claims in `swarm status`.

## 3. Verification

- [ ] 3.1 Unit tests: claim, conflict warn, conflict block, release on stop and orphan.
- [ ] 3.2 Test claims survive server reload.
- [ ] 3.3 Run `openspec validate add-swarm-file-claims --strict`.
