# Tasks

## Execution model: swimlanes + subagents

Run with a coordinator plus parallel swarm workers (`Swarm spawn`, one `label` per lane). Lanes have **disjoint file ownership**, so workers never edit the same file. The coordinator alone stages, commits and pushes, which avoids racing on the shared git index. It reviews each lane's diff, runs the gate, then commits that lane's hunks.

Shared rules for every lane:
- Never reload or restart the shared daemon. `builds/current` stays at 1.0.1. Runtime checks use the private socket `/run/user/1000/jcode-rcrepro.sock` and `JCODE_HOME=scratch/rc-home` through `scratch/rc-core/{repro.sh,lock_exit.sh}`, or the worker's own unique socket.
- Do not touch hunks outside the lane's list. The tree also holds other sessions' work in the same files, such as the `background_tasks::reattach_background_tasks_on_startup` hunk in `crates/jcode-app-core/src/server.rs`.
- Cargo builds serialize on the shared lock. Prefer `Selfdev test` or `scripts/dev_cargo.sh`. A worker blocked on the build lock waits; it does not start a second build.
- Report back: the diff summary, the exact commands run and their observed output, and the hunks to stage.

```
Phase 1 (parallel)                                   Phase 2         Phase 3
lane A  core-verified (Fix 1,4,5,7)  ──┐
lane B  client-start-gate (Fix 3)    ──┼──► coordinator: per-fix commits ──► full gate + validate
lane C  resume-quarantine (Fix 2)    ──┤
lane D  promote-gate (Fix 6)         ──┘  (D depends on A's Fix 1 only for its "good binary" check)
```

Ownership:

| Lane | Owns (hunks only) |
|---|---|
| A | `src/main.rs`; `server/runtime.rs`; `server/socket.rs` (lock wait, `pub` on `server_start_matches_existing_server`, stderr line ~503); `server/util.rs`; `server.rs` (`supervised_by_service_manager` import plus idle branch, `pub use ...server_start_matches_existing_server`); `server/await_members_state.rs`; `src/cli/dispatch.rs` hunk @@175 (exit 78) |
| B | `src/cli/dispatch.rs` hunks @@1393, @@1407, @@1517; `src/cli/output.rs`; `src/cli/dispatch_tests.rs` |
| C | `server/session_manager.rs`; `server/session_manager_tests.rs` |
| D | `src/cli/commands.rs` (`run_server_promote_command`); `src/cli/args.rs` (`Promote`); `src/cli/args/tests.rs`; the promote call site in `src/cli/dispatch.rs` (coordinate with B: D edits only the `Promote` match arm) |

## 1. Lane A: core-verified (worker, label `rc-core-verified`)

- [ ] 1.1 Rebuild `target/selfdev/jcode` from the current tree. Run `scratch/rc-core/repro.sh target/selfdev/jcode` and expect `server alive: yes` and `Session restored: 842 messages`. Fix 1 and Fix 7: expect no `missing field` warnings in `scratch/rc-home/serve.log`.
- [ ] 1.2 Run `scratch/rc-core/lock_exit.sh target/selfdev/jcode`. Fix 4: expect unsupervised `exit=78` after about 0 s, supervised `exit=78` after about 10 s, and supervised success when the lock frees after 3 s.
- [ ] 1.3 Fix 5: start the binary on a private socket with `JCODE_SERVER_SYSTEMD_UNIT=jcode-test.service`. Confirm the log line saying the idle monitor is skipped. Without the variable, confirm the idle monitor runs.
- [ ] 1.4 Run `cargo test -p jcode-app-core` scoped to `socket`, `await_members_state` and `util`. Update `socket_tests.rs:506` if it asserts the old `pub(super)` path or the old lock error.
- [ ] 1.5 Hand back four hunk sets: Fix 1, Fix 7, Fix 5, Fix 4.

## 2. Lane B: client-start-gate (worker, label `rc-client-start-gate`)

- [ ] 2.1 Compile the kept Fix 3 code. Fix compile errors inside owned hunks only.
- [ ] 2.2 Run `cargo test -p jcode --lib dispatch_tests`, including `client_start_refused_once_systemd_gave_up`. Add unit tests for `unit_refusal_from_show`: `Result=success` gives None; `start-limit-hit` gives text naming the unit, the status, `journalctl` and `reset-failed`; and a test that `NRestarts=50` alone does not refuse.
- [ ] 2.3 Manual, private socket: use a dummy user unit `jcode-rctest.service` whose `ExecStart=/bin/false`. Drive it to `Result=start-limit-hit`. Then, with `JCODE_SERVER_SYSTEMD_UNIT=jcode-rctest.service` and a private `--socket`, run a client. Expect the refusal error and no `systemctl start` in `journalctl --user -u jcode-rctest`. Clean up with `systemctl --user reset-failed jcode-rctest` and remove the unit.
- [ ] 2.4 Raw-mode stderr: start a TUI against a private socket inside a pty (or use a `debug_socket` tester), kill its private server, and confirm the captured pty bytes contain no `Starting server via systemd` or `systemctl` text outside the frames.
- [ ] 2.5 Hand back the Fix 3 hunk set.

## 3. Lane C: resume-quarantine (worker, label `rc-resume-quarantine`)

- [ ] 3.1 Compile the kept Fix 2 code. Run `cargo test -p jcode-app-core session_manager_tests`, including `poisoned_resume_marker_blocks_only_when_writer_is_dead`.
- [ ] 3.2 Add tests: an unparseable marker proceeds; the own pid proceeds; the marker is removed after a failed restore; a marker for session S does not block session T.
- [ ] 3.3 Runtime: pre-seed `scratch/rc-home/resume-in-progress/<sanitized SID>` with a dead pid (a pid from an exited `sleep`), then run `repro.sh`. Expect the resume error to contain `died while restoring` and the marker path, and expect `server alive: yes`. Remove the marker and rerun: expect the resume to succeed.
- [ ] 3.4 Hand back the Fix 2 hunk set.

## 4. Lane D: promote-gate (worker, label `rc-promote-gate`)

- [ ] 4.1 Implement the Fix 6 design (`design.md` § Fix 6): the newest-session pick, temporary `JCODE_HOME`, private socket, `Client::resume_session_with_options`, the 60 s timeout, a watcher on the child process, cleanup on all paths, `--skip-resume-check`, and `resume_check` in the report.
- [ ] 4.2 Unit tests: session selection skips test sessions; an empty sessions dir gives `skipped_no_session`; the flag gives `skipped_flag`; args parse `--skip-resume-check`.
- [ ] 4.3 Acceptance on an isolated `HOME`/`JCODE_HOME` with build `8a2141d07` installed as a version: `jcode server promote <rc>` is refused, the report names the exit status, and the `shared-server` symlink is unchanged. Then promote a version built from this tree and expect success with the temporary directory removed. Never run this against the real `~/.jcode`.
- [ ] 4.4 Hand back the Fix 6 hunk set.

## 5. Coordinator: integrate and commit (self)

- [ ] 5.1 Review each lane's diff against `specs/shared-server-crash-resilience/spec.md`.
- [ ] 5.2 Make one commit per fix on `main`, in this order: Fix 1, 7, 5, 4, 3, 2, 6. Stage only that fix's hunks (`git add -p` or `git apply --cached`) and check `git diff --cached` before each commit. Push after each commit.
- [ ] 5.3 Run `cargo test -p jcode-app-core -p jcode` for the touched modules, scoped clippy, and `cargo fmt --check` on the owned files.
- [ ] 5.4 Run `openspec validate harden-rc-crash-loop --strict` and the dependency validator. Record the evidence in this file.
