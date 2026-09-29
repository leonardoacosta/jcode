# Jcode title-fix candidate adoption audit

**Date:** 2026-09-16 19:28 UTC
**Source owner:** `/home/nyaptor/.jcode`
**Candidate:** `scratch/herdr-radar-candidate-target/selfdev/jcode`
**Scope:** Read-only investigation. No symlinks, channel markers, shared server, update, reload, or live sessions were changed.

## Conclusion

**Not safe to adopt as the default launcher under the stated boundary.** The candidate is a private selfdev binary (`jcode v0.77.0-dev (e187b55, dirty)`) while the active launcher, `builds/current`, and `builds/shared-server` all resolve to `/home/nyaptor/.jcode/builds/versions/64946cc3d/jcode`, and the channel markers are `current=64946cc3d`, `shared-server=64946cc3d`, `stable=1bbf2816f-v084-homelab`. Installing the candidate as default would require mutating the launcher/current channel, which is explicitly prohibited here. It would also create a client/server version mismatch risk and, on an ordinary launch, the candidate's default background update behavior can run update logic.

The safe disposition is to retain the candidate isolated and provide an explicitly invoked candidate launcher only after separate review. No separate launcher was created by this audit.

## Evidence

### 1. Bundled documentation

`docs/WRAPPERS.md` recommends `--no-update` to avoid update-check work and `--no-selfdev` to avoid repository auto-detection changing runtime behavior.

`docs/SERVER_ARCHITECTURE.md` states that self-dev mode connects to the normal shared Jcode server and that `/reload` hot-reloads the shared server and clients.

`docs/NATIVE_SSH.md` describes a separate private local socket adapter for native connections. This does not make a normal local launch private: the ordinary socket defaults to the runtime socket unless overridden.

### 2. Default launcher and channel state

Read-only commands:

```text
readlink -f ~/.local/bin/jcode
/home/nyaptor/.jcode/builds/versions/64946cc3d/jcode

readlink -f builds/current/jcode
/home/nyaptor/.jcode/builds/versions/64946cc3d/jcode

readlink -f builds/shared-server/jcode
/home/nyaptor/.jcode/builds/versions/64946cc3d/jcode

sha256sum scratch/herdr-radar-candidate-target/selfdev/jcode
80cc47512b1e85715a95e1f791ae62e5fd8874dccc92b6cb725bc7083798ed18

candidate --version
jcode v0.77.0-dev (e187b55, dirty)
```

`crates/jcode-build-support/src/lib.rs:664-676` shows that current and shared-server channel operations write symlinks and version markers. In particular, `update_shared_server_symlink` writes both the shared-server symlink and marker. `:705-706` shows local publication also updates `current` and the launcher. `:725-744` shows promotion to shared-server is an explicit channel mutation.

### 3. Normal launch auto-update behavior

`src/cli/args.rs:46-52` defines `--no-update` and `--auto-update`, with `auto_update` defaulting to true.

`src/cli/startup.rs:126-127` starts the background update check during startup. `:447-456` shows the check is skipped only for `--no-update`, quiet mode, disabled config, selected commands, or resume mode. A normal interactive launch therefore remains eligible.

`src/cli/startup.rs:328-340` starts a background thread and calls `update::check_and_maybe_update(auto_update)` for release builds. For source/selfdev builds, `:384-413` calls `hot_exec::check_for_updates`, and when auto-update is enabled it calls `hot_exec::run_auto_update`.

`src/cli/hot_exec.rs:306-318` implements source auto-update by fast-forward pulling the repository and running `cargo build --release`. `:332-334` calls `build::install_local_release`, which is an installation/publishing operation, not a read-only check. `:346-362` can request a graceful client reload when a live session exists. `:365-375` otherwise re-execs the updated candidate with `--no-update`.

Thus, invoking the candidate as an ordinary launcher without `--no-update` does not satisfy the no-update boundary. It can fetch, build, install, and reload/re-exec.

### 4. Client/server selection and handshake boundary

`src/cli/dispatch.rs:1267-1302` implements server startup. It first checks the default socket and waits for an existing reload server. If no server is live, it selects `build::shared_server_update_candidate(...)`, falling back to the current executable. `:1303-1322` then starts that selected executable as `serve` on the default socket.

`crates/jcode-app-core/src/server/socket.rs:7-12` shows the default socket is the runtime directory's `jcode.sock`, unless `JCODE_SOCKET` or `--socket` supplies a private path. `:48-69` connects to that socket and deliberately avoids unlinking a refused socket. This protects the daemon but confirms that a normal launch targets the shared/default socket, not the candidate scratch path.

`crates/jcode-app-core/src/server/client_state.rs:536-540` includes the server's build version and `server_has_update` in the initial History payload. This is the observable client/server version-advertisement boundary. The source also uses update/reload handling when the server reports a newer binary. A candidate client attached to the existing server therefore does not replace the server binary merely by being invoked.

`src/cli/startup.rs:95-104` registers the normal shared-server spawner used by the TUI reconnect loop. It delegates to `dispatch::spawn_server`, so reconnect/startup uses the shared-server candidate selection above.

## Safe procedure if adoption is later approved

1. Keep the candidate at its immutable scratch path and record its SHA-256 and `--version` output.
2. Do not repoint `~/.local/bin/jcode`, `builds/current/jcode`, `builds/stable/jcode`, or `builds/shared-server/jcode`; do not edit their marker files.
3. Use an explicit wrapper or direct candidate invocation with `--no-update`, `--no-selfdev`, and a task-owned `--socket` path. The wrapper must not be installed at the default launcher path until its behavior is separately reviewed.
4. Ensure the task-owned socket and any server process are isolated and clean up only those owned resources. Do not use `/reload`, `self-dev --build`, update, or shared-server promotion.
5. Validate title behavior from the candidate's PTY/OSC output. Treat existing clients as unchanged until their next normal launch after an approved binary selection.
6. If the desired outcome is truly “next normal launch,” perform a separate explicit activation review. That action necessarily changes launcher/current selection and may require coordinating the server target, so it is outside this audit's authorization.

## Final disposition

- Candidate built and retained: **yes**.
- Default launcher installed/selected: **no**.
- Shared-server restart/update/reload triggered: **no**.
- Symlink or channel marker mutated: **no**.
- Safe default adoption under current boundaries: **no**.
- Safe next step: **retain isolated candidate; review an explicit no-update/private-socket launcher separately**.
