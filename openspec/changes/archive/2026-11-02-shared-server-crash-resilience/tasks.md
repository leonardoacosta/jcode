## 1. Exit record
- [x] 1.1 Define exit-record file per server PID (cause, signal, panic message/location, timestamp)
- [x] 1.2 Write record on graceful shutdown and caught SIGTERM/SIGINT/SIGHUP
- [x] 1.3 Install panic hook for `serve` that writes the record before exit
- [x] 1.4 Unit tests for record write/read and stale-record cleanup

## 2. Crash classification
- [x] 2.1 `detect_crash` reads exit record; falls back to systemd unit `Result`/`ExecMainStatus`; else `unknown`
- [x] 2.2 Replace fixed "no shutdown signal captured" text with the classified reason
- [x] 2.3 Tests: signal, panic, oom-kill, missing/corrupt record

## 3. Supervised gate (existing WIP in src/cli/dispatch.rs)
- [x] 3.1 Finish `enforce_supervised_shared_server`; error when delegation fails
- [x] 3.2 Tests: no marker, empty marker, in-cgroup, out-of-cgroup, temporary server bypass

## 4. Grouped recovery
- [x] 4.1 Group crashed sessions by `last_pid` into crash events
- [x] 4.2 One summary notice per event; bulk recover via `recover_crashed_sessions_by_ids`
- [x] 4.3 Mark in-flight turns interrupted; never auto-resume them
- [x] 4.4 Test: 100 sessions on one PID yields one event, all recoverable

## 5. Verify
- [x] 5.1 Runtime check on an isolated socket: `kill -TERM` and `kill -KILL` the server, confirm classified reasons and grouped recovery
