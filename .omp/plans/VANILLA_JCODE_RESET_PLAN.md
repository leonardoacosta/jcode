## Context
Tear down the current jcode install and run vanilla upstream v1.1.2. Stop and disable the systemd daemon, remove the client launcher from PATH, repoint the build channels at the already-installed vanilla `versions/1.1.2`, then run it. "Reset the build" means the binary channels under `~/.jcode/builds/`. It does not mean `git reset` on `~/.jcode` (fork `main`, 3 commits ahead of `origin/main`; global policy preserves it, its 13+ worktrees and its 4 stashes).

## Verified facts (do not re-investigate)
- Latest stable upstream release: `v1.1.2`, tag commit `4f874c9ea6b7c3e228045fa8269e4d84bec66a6b`, released 2026-11-23 (`gh release list -R 1jehuang/jcode`). Upstream default branch is `master`.
- `~/.jcode/builds/versions/1.1.2/` is genuine vanilla. `jcode-linux-x86_64.bin` has sha256 `edcb7a23f17af8be0aaa223ca1c977fcc49f07f606d62b3e83daddd664bc7089`. That equals the binary extracted (streamed, not saved to disk) from the release tarball whose sha256 `74ae84a1...efc56` matches the published `SHA256SUMS`. `versions/1.1.2/jcode --version` prints `jcode v1.1.2 (4f874c9ea)`, the tag commit. `versions/1.1.2/jcode` is a 505-byte wrapper script that sets `LD_LIBRARY_PATH` and execs the sibling `.bin`.
- Conclusion: do NOT run upstream `scripts/install.sh`. It also sends telemetry, writes `~/.jcode/install_conversion_id`, rewrites `~/.jcode/config.toml` (`update_channel`), and would not advance `shared-server` (because `shared-server-version` differs from `stable-version`). Do the symlink repoint by hand.
- Daemon: user unit `jcode.service` is enabled and active. The unit file is a symlink to `~/dev/personal/homelab/services/jcode/jcode.service`, with drop-ins `10-hardening.conf` and `20-nova-vault-key.conf` in `~/.config/systemd/user/jcode.service.d/`. `ExecStart=%h/.local/bin/jcode serve --provider-profile omni --server-name homelab`, `Restart=always`, `WantedBy=default.target`. The only reverse dependency is `default.target`. The daemon is PID 1354 (cgroup `jcode.service`) on `/run/user/1000/jcode.sock`. It runs `builds/shared-server/jcode`, which points at `versions/81527d7f2-dirty-patch`.
- Launcher chain: `~/.local/bin/jcode` -> `~/.jcode/builds/current/jcode` -> `versions/81527d7f2-dirty-patch/jcode`. That is the only `jcode` on PATH. `~/.cargo/bin/jcode` does not exist.
- Channels today: `stable` -> `versions/1.0.1`; `canary` -> `versions/a65876997`; `current` and `shared-server` -> `versions/81527d7f2-dirty-patch`. Version-marker files `current-version` and `shared-server-version` contain `81527d7f2-dirty-patch`; `stable-version` contains `1.0.1`. All three have no trailing newline.
- NOT jcode-owned, leave alone: PID 1359 `serve.py` (cgroup `skull-lifecycle-preview-8764.service`) and PID 4643 `.../versions/774344983-dirty-05a2fd63e815/jcode companion` (cgroup `herdr.service`). Both have PPID 1288 (the systemd user manager).
- Live clients (do not kill): PIDs 4076094, 4079020 and 4080338 (`jcode --resume session_{whale,monkey,rhino}_*`, each under a ghostty window) and PID 1188214 (`jcode`, with `systemd-inhibit` sleepers showing active streaming).
- Stale files from an earlier vanilla preflight at 05:16, no process attached: `/run/user/1000/jcode-vanilla-preflight.sock`, `.sock.daemon.lock`, `.sock.hash`, `jcode-vanilla-preflight-debug.sock`.
- Upstream supports `--provider-profile` (`crates/jcode-base/src/auth/mod.rs`) and the `JCODE_HOME` env var.

## Approach (execute in order; each step leaves the system consistent)

### 1. Stop and disable the daemon
1. Record pre-change state (runbook: `~/dev/personal/homelab/services/jcode/README.md`, "Later scoped deployment"). Save to `local://vanilla-reset-snapshot.txt`: daemon PID 1354, `systemctl --user show jcode.service -p FragmentPath -p DropInPaths -p KillMode -p UnitFileState`, `readlink -f` of `~/.local/bin/jcode`, `builds/{stable,current,shared-server}/jcode`, and `ls -l ~/.config/systemd/user/jcode.service ~/.config/systemd/user/jcode.service.d/`. The runbook says `KillMode=control-group`, so stopping PID 1354 also kills its tool child processes. Record those children (`pgrep -P 1354`) before stopping.
2. Run `systemctl --user disable --now jcode.service`. The user requested removal, so stopping and disabling is in scope. Do NOT run `daemon-reload`: no unit file changes, and the homelab runbook gates `daemon-reload` as maintenance-only.
   - This removes the `default.target.wants` link and stops PID 1354.
   - Leave `~/.config/systemd/user/jcode.service` (the symlink into the homelab repo) and the `.d/` drop-ins in place. The runbook forbids relinking or restoring `~/.config/systemd/user`. Unit removal needs a separate homelab change.
3. Run `pgrep -af 'jcode serve --socket /run/user/1000/jcode.sock'` after 10 s. Use this exact pattern: the bare `jcode.*serve` also matches the unrelated `serve.py` (PID 1359). `Restart=always` is stopped by `disable --now`, but a live client may auto-spawn a daemon from `builds/shared-server/jcode`.
   - If a `jcode serve --socket` PID appears, send SIGTERM to that PID only. Never touch the client PIDs listed above, and never touch PID 1359 (`serve.py`, owned by `skull-lifecycle-preview-8764.service`).
4. Delete stale runtime files in `/run/user/1000/`, only after `ss -xlp` and `lsof` show no holder:
   - `jcode.sock`, `jcode.sock.hash`, `jcode-debug.sock`, `jcode.bak` (a stale reload record from PID 1354, optional cleanup), `jcode-vanilla-preflight.sock`, `jcode-vanilla-preflight.sock.daemon.lock`, `jcode-vanilla-preflight.sock.hash`, `jcode-vanilla-preflight-debug.sock`.
   - Delete `jcode-daemon.lock` only after `lsof /run/user/1000/jcode-daemon.lock` shows no holder. PID 1354 holds it (`10wW`) until Step 1.2 stops it.
   - Before deleting any `jcode-vanilla-preflight*` file, run `lsof` on it. The `-debug.sock` holder is unverified.
   - Do not delete `jcode-companion.sock` (herdr's companion, PID 4643, holds it).
   - Do not delete `jcode-cargo-build.lock`, `jcode-swarm-mutations/`, `jcode-usage/` or `jcode-await-members/` (live sessions use them).

### 2. Remove the client from PATH
1. Confirm `readlink ~/.local/bin/jcode` prints `/home/nyaptor/.jcode/builds/current/jcode`.
2. Run `rm ~/.local/bin/jcode` (symlink only), then `hash -r`.
3. Expected side effect: the ghostty `jcode --resume` launchers and the `jcode.service` ExecStart fail until Step 3 recreates the launcher. The unit is disabled, so only the launchers matter.

### 3. Repoint the build channels at vanilla 1.1.2
Run with `B=~/.jcode/builds`:
1. `ln -sfn "$B/versions/1.1.2/jcode" "$B/stable/jcode"`
2. `ln -sfn "$B/versions/1.1.2/jcode" "$B/current/jcode"`
3. `ln -sfn "$B/versions/1.1.2/jcode" "$B/shared-server/jcode"`
4. `printf '%s' 1.1.2 > "$B/stable-version"`, and the same for `"$B/current-version"` and `"$B/shared-server-version"` (no trailing newline, matching the existing files).
5. `ln -s "$B/stable/jcode" ~/.local/bin/jcode` (same launcher shape as upstream `install.sh`, which points the launcher at `stable`).
6. Leave untouched: `canary/`, `rc/`, `manifest.json`, `manifest.bak`, every `versions/*` directory (immutable, recoverable), and the git checkout.

### 4. Isolated smoke run (daemon + client)
1. Start an isolated daemon that this step owns: `~/.jcode/builds/versions/1.1.2/jcode serve --socket /run/user/1000/jcode-vanilla-smoke.sock --no-update` in the background. Record its PID. `serve` is required: a headless `run` does not bind a socket until a tool needs one.
2. Wait for the socket: poll `ss -xl | grep jcode-vanilla-smoke.sock` for up to 15 s.
3. Confirm the smoke daemon runs vanilla: `readlink /proc/<recorded pid>/exe` ends in `versions/1.1.2/jcode-linux-x86_64.bin`. Its cmdline begins with `jcode-linux-x86_64.bin serve`, because the wrapper execs the `.bin`. Use the recorded PID, not a pgrep pattern.
4. Run `~/.jcode/builds/versions/1.1.2/jcode --no-update run --socket /run/user/1000/jcode-vanilla-smoke.sock 'reply with the single word ok'`.
5. Pass: the client prints a reply, or a provider credentials-required error. A credentials error still proves server start and client connect. Do not log in or provision credentials.
6. If the daemon fails on the existing `~/.jcode` config: capture the exact error, retry once with `JCODE_HOME=$(mktemp -d)`, and do NOT edit the user's config.
7. Stop only the recorded smoke daemon PID. Remove `jcode-vanilla-smoke.sock*` from `/run/user/1000/`.

### 5. Run on the default socket (daemon + client)
1. Run `~/.local/bin/jcode serve --socket /run/user/1000/jcode.sock --provider-profile omni --server-name homelab` as the daemon, in the background, and record its PID. This reproduces the removed unit's ExecStart, so vanilla runs the same arguments the old service did.
2. Run `~/.local/bin/jcode --no-update run --socket /run/user/1000/jcode.sock 'reply with the single word ok'`.
3. This daemon is not systemd-managed. Re-enabling `jcode.service` is out of scope.

## Critical files & anchors
- `~/.jcode/builds/{stable,current,shared-server}/jcode` — symlinks to repoint (Step 3).
- `~/.jcode/builds/{stable,current,shared-server}-version` — marker files to rewrite (Step 3).
- `~/.local/bin/jcode` — launcher symlink: remove (Step 2), recreate to `stable` (Step 3).
- `~/dev/personal/homelab/services/jcode/jcode.service` — unit target; read-only here.
- `~/.jcode/.agents/AGENTS.md` — "Verifying a change at runtime": isolated `--socket` smoke pattern.

## Verification
- Step 1: `systemctl --user is-active jcode.service` prints `inactive`. `systemctl --user is-enabled jcode.service` prints `linked` or `disabled`. `pgrep -af 'jcode serve --socket /run/user/1000/jcode.sock'` prints nothing. `lsof /run/user/1000/jcode-daemon.lock` prints nothing. `ss -xl | grep -E 'jcode(\.sock|-debug)'` prints nothing. PIDs 4076094, 4079020, 4080338 and 1188214 still exist (`ps -p`). PID 1359 (`serve.py`) is untouched.
- Step 2: `command -v jcode` prints nothing. `which -a jcode` exits 1. `ls ~/.empryo/bin | grep -i jcode` prints nothing.
- Step 3: `readlink -f` on `$B/stable/jcode`, `$B/current/jcode`, `$B/shared-server/jcode` and `~/.local/bin/jcode` each ends in `versions/1.1.2/jcode` (the wrapper). `jcode --version` prints `jcode v1.1.2 (4f874c9ea)` with no `-dirty` or `-patch`. The wrapper's exec target is verified with `sha256sum "$B/versions/1.1.2/jcode-linux-x86_64.bin"`, which prints `edcb7a23f17af8be0aaa223ca1c977fcc49f07f606d62b3e83daddd664bc7089`. The wrapper `$B/versions/1.1.2/jcode` prints `059d05f0a167b5496bc5a7bdf963edff8f8a483d61f290847b5765104c476298`, the sha256 of the tarball's `jcode-linux-x86_64` member. `cat` of the three marker files prints `1.1.2` each.
- Step 4: the smoke client prints a reply or a provider credentials error. The recorded smoke daemon PID's `/proc/<pid>/exe` resolves into `versions/1.1.2/jcode-linux-x86_64.bin`. `ss -xl | grep jcode-vanilla-smoke.sock` showed the socket before cleanup.
- Step 4 cleanup: `pgrep -f 'jcode-vanilla-smoke.sock'` prints nothing after step 4.7. `ss -xl | grep jcode-vanilla-smoke.sock` prints nothing.
- Step 5: `ss -xl | grep 'run/user/1000/jcode.sock'` shows the socket. The recorded daemon PID's `/proc/<pid>/exe` resolves into `versions/1.1.2/jcode-linux-x86_64.bin`. `pgrep -af 'jcode-linux-x86_64.bin serve --socket /run/user/1000/jcode.sock'` shows exactly one daemon. A second `readlink -f` of the three channels shows they were not moved by auto-update; if they moved, report the new version rather than reverting.
- No tests or verification assets are changed (no source files are touched).

## Assumptions & contingencies
- Assumption (user-overridable): "reset the build" means the binary channels, not `git reset` on `main`.
- Assumption (user-overridable): the vanilla launcher goes back on PATH via `stable` after the reset. To keep vanilla off PATH, skip Step 3.5 and run `~/.jcode/builds/stable/jcode` directly.
- Assumption (user-overridable): the `jcode.service` symlink and drop-ins stay in place, disabled.
- Contingency: if `jcode --version` after Step 3 does not print `v1.1.2 (4f874c9ea)`, stop. Re-check the three `readlink -f` outputs and do not continue to Step 4.
- Contingency: if `JCODE_HOME=$(mktemp -d)` also fails in Step 4, vanilla 1.1.2 has a startup defect unrelated to config. Report the captured error and stop. Do not fall back to the dirty-patch build.
- Contingency: if a `jcode serve` daemon respawns repeatedly after Step 1, find its parent with `ps -o pid,ppid,args` and report which client spawns it. Do not kill clients.
