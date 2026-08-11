#!/usr/bin/env bash
# Post-commit deploy hook: when a commit touches buildable code, rebuild that
# exact commit in a detached worktree, install it, reload the shared server,
# and let opted-in clients re-exec when idle.
#
# Installed as .git/hooks/post-commit by scripts/install_deploy_hook.sh.
# Escape hatch: JCODE_NO_DEPLOY=1 git commit ...
set -euo pipefail

if [ "${JCODE_NO_DEPLOY:-}" = "1" ]; then
    exit 0
fi

repo_root="$(git rev-parse --show-toplevel)"
head="$(git rev-parse HEAD)"

# Docs, tests fixtures, and workflow metadata do not change the executable.
# JCODE_DEPLOY_FORCE=1 is the explicit recovery/manual-deploy path (useful if a
# previous hook run failed after its request was consumed).
if [ "${JCODE_DEPLOY_FORCE:-}" != "1" ]; then
    if ! git diff-tree --root --no-commit-id --name-only -r "$head" \
        | grep -Eq '^(crates/|src/|apps/command-center/|build\.rs$|Cargo\.(toml|lock)$|rust-toolchain|\.cargo/)'; then
        exit 0
    fi
fi

# Git exports repository-local variables to hooks (notably GIT_INDEX_FILE on
# some runners). They are valid for the triggering command but poison later
# `git -C ... worktree add` calls in the detached background worker: a relative
# `.git/index` is then resolved against the wrong worktree. We already captured
# the source repo and commit, so clear only repository-local routing variables.
unset GIT_ALTERNATE_OBJECT_DIRECTORIES GIT_COMMON_DIR GIT_DIR GIT_INDEX_FILE
unset GIT_OBJECT_DIRECTORY GIT_PREFIX GIT_WORK_TREE

state_dir="$repo_root/target/commit-deploy"
lock="$state_dir/lock"
request="$state_dir/requested-head"
worktree="$state_dir/worktree"
target_dir="$repo_root/target/commit-deploy-cargo"
log="$repo_root/target/deploy.log"
mkdir -p "$state_dir"

# Always publish the newest requested commit, even if another deploy is active.
# Atomic rename prevents the worker from reading a partial SHA.
printf '%s\n' "$head" > "$request.$$"
mv "$request.$$" "$request"

# One worker drains the request slot. Later commits only replace the slot and
# return immediately; the active worker loops and deploys the newest request.
if ! mkdir "$lock" 2>/dev/null; then
    echo "$(date -Is) deploy queued (HEAD=${head:0:12})" >> "$log"
    exit 0
fi

(
    cleanup() {
        git -C "$repo_root" worktree remove --force "$worktree" >/dev/null 2>&1 || true
        rmdir "$lock" >/dev/null 2>&1 || true
    }
    trap cleanup EXIT

    while true; do
        if [ ! -f "$request" ]; then
            # Release ownership, then close the handoff race: a hook may have
            # queued work between the absence check and rmdir. Reacquire if so;
            # otherwise that hook owns the next worker.
            rmdir "$lock"
            if [ -f "$request" ] && mkdir "$lock" 2>/dev/null; then
                continue
            fi
            trap - EXIT
            exit 0
        fi

        deploy_head="$(cat "$request")"
        rm -f "$request"
        short="${deploy_head:0:12}"
        echo "$(date -Is) deploy started (HEAD=$short)" >> "$log"

        git -C "$repo_root" worktree remove --force "$worktree" >/dev/null 2>&1 || true
        if ! git -C "$repo_root" worktree add --detach --force "$worktree" "$deploy_head" \
            >> "$log" 2>&1; then
            echo "$(date -Is) deploy FAILED: could not create worktree for $short" >> "$log"
            continue
        fi

        if (
            cd "$worktree"
            ./scripts/install_command_center_assets.sh
            CARGO_TARGET_DIR="$target_dir" ./scripts/install_release.sh --fast
        ) >> "$log" 2>&1; then
            echo "$(date -Is) deploy finished (HEAD=$short)" >> "$log"
        else
            echo "$(date -Is) deploy FAILED (HEAD=$short; see output above)" >> "$log"
        fi
        git -C "$repo_root" worktree remove --force "$worktree" >/dev/null 2>&1 || true
    done
) </dev/null >>"$log" 2>&1 &

exit 0
