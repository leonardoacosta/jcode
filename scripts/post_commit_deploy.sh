#!/usr/bin/env bash
# Post-commit deploy hook (issue-driven dev loop): when a commit touches
# buildable code, rebuild + install the release in the background and reload
# the shared server. Clients that opted in (`display.auto_client_reload = true`
# or self-dev canary sessions) then re-exec onto the new binary when idle.
#
# Installed as .git/hooks/post-commit (see scripts/install_deploy_hook.sh).
# Escape hatch: JCODE_NO_DEPLOY=1 git commit ...
set -euo pipefail

# Escape hatch for commits that touch buildable code but should not deploy
# (WIP snapshots, experimental branches).
if [ "${JCODE_NO_DEPLOY:-}" = "1" ]; then
    exit 0
fi

repo_root="$(git rev-parse --show-toplevel)"

# Only rebuild when the commit touches buildable code.
if ! git diff-tree --no-commit-id --name-only -r HEAD \
    | grep -Eq '^(crates/|Cargo\.(toml|lock)|rust-toolchain|\.cargo/)'; then
    exit 0
fi

# Never block the committer on a build, and never stack overlapping builds
# (rapid commit sequences): one deploy at a time, extra commits are picked up
# because the build always compiles the worktree HEAD at start time... which
# may be newer than the commit that triggered it. That is fine: the install
# hash is derived from the repo state at install time.
mkdir -p "$repo_root/target"
lock="$repo_root/target/.deploy.lock"
log="$repo_root/target/deploy.log"

if ! mkdir "$lock" 2>/dev/null; then
    echo "$(date -Is) deploy already running, skipping (HEAD=$(git rev-parse --short HEAD))" >> "$log"
    exit 0
fi

(
    trap 'rmdir "$lock"' EXIT
    echo "$(date -Is) deploy started (HEAD=$(git rev-parse --short HEAD))" >> "$log"
    if "$repo_root/scripts/install_release.sh" --fast >> "$log" 2>&1; then
        echo "$(date -Is) deploy finished" >> "$log"
    else
        echo "$(date -Is) deploy FAILED (see output above)" >> "$log"
    fi
) </dev/null >>"$log" 2>&1 &

exit 0
