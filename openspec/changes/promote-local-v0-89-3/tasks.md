## 1. Preflight
- [ ] 1.1 Record frozen main HEAD, dirty paths, active session owners, old channel targets and immutable rollback binary.
- [ ] 1.2 Resolve pending activation ownership and establish exclusive promotion ownership without overwriting another owner's work.

## 2. Candidate validation (depends on 1)
- [ ] 2.1 Run formatting and base/core/root suites on frozen source, record exact commands and limitations.
- [ ] 2.2 Build TUI with coordinated selfdev tooling and record exact binary fingerprint/version.
- [ ] 2.3 Exercise exact candidate on fresh isolated runtime/socket: version, daemon request, small real-provider response and read-only MCP access. Stop on failure.

## 3. Promotion (depends on 2)
- [ ] 3.1 Coordinate active sessions and promote launcher/shared daemon via supported selfdev reload with old immutable binary retained.
- [ ] 3.2 Verify launcher target, actual running shared version, session continuity/reconnection and read-only MCP access. Roll back on failure.

## 4. Evidence (depends on 3)
- [ ] 4.1 Record requirement-to-check results, before/after targets and rollback procedure. Commit owned artifacts and report outcome and limitations.
