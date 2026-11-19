## 1. Implementation

- [ ] 1.1 Separate shared and temporary lifecycle policy so the shared `jcode serve` path cannot exit on idle timeout or return `EXIT_IDLE_TIMEOUT`; preserve temporary cleanup and owner-PID liveness behavior.
- [ ] 1.2 Add a work-presence guard covering pending/active scheduled, ambient, and background work before any temporary idle exit; use existing work and recovery state.
- [ ] 1.3 Make SIGTERM and reload stop admission, preserve or hand off pending work, drain saves, and clean up scope-appropriate registry/socket artifacts without claiming unfinished work completed.
- [ ] 1.4 Add CLI and TUI server status with live server count, connected-client count, and per-server uptime from registry `started_at`; use stale cleanup and cell-width truncation.
- [ ] 1.5 Add regression tests: shared daemon passes its idle threshold and remains alive under `Restart=always`; temporary idle exit requires no pending work; dead owner PID shuts down temporary server; stale registry entries are excluded; SIGTERM/reload preserves pending work; malformed timestamps degrade to unavailable uptime; narrow terminal rows never wrap.

## 2. Verification

- [ ] 2.1 Run focused lifecycle and status tests by exact name, then `cargo test -p jcode-app-core` and tests for the CLI/TUI crates changed.
- [ ] 2.2 `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] 2.3 `openspec validate keep-shared-daemon-resident --strict`
- [ ] 2.4 `python3 ~/dev/personal/jev/scripts/openspec/validate_dependencies.py --root /home/nyaptor/.jcode keep-shared-daemon-resident`
