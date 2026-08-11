#!/usr/bin/env bash
# beads-helpers.sh — Sourceable library for beads issue management during spec archival.
#
# Functions:
#   extract_beads_ids <tasks.md>                          → sets BEADS_EPIC_ID, BEADS_FEATURE_ID, BEADS_TASK_IDS, BEADS_TASK_IDS_CHECKED
#   close_beads_all <spec_name> <tasks.md>                → close CHECKED task IDs + feature (epic stays open per 3-level model)
#   close_orphaned_beads <spec_name> [exclude_ids]        → close leftover open issues
#
# Usage:
#   source <open-work-root>/lib/beads-helpers.sh
#   extract_beads_ids "openspec/my-feature/tasks.md"
#   close_beads_all "my-feature" "openspec/my-feature/tasks.md"
#   close_orphaned_beads "my-feature" "$BEADS_EPIC_ID $BEADS_TASK_IDS"

# strict mode only when executed directly, never when sourced (avoid mutating the caller shell)
(return 0 2>/dev/null) || set -euo pipefail

# Guard against double-sourcing
[[ -n "${_BEADS_HELPERS_LOADED:-}" ]] && return 0
_BEADS_HELPERS_LOADED=1

# ---------------------------------------------------------------------------
# BEADS_ID_RE — canonical bead-ID grammar, confirmed 2026-07-20 against a live
# bd 1.1.0 sample (`bd list --json --limit 2000`, docs/reference/bd-1.1.0-baseline.md
# does not state the ID regex explicitly): hyphenated hash-body IDs with an
# optional dotted child suffix, e.g. `bd-alpha1`, `bd-alpha1.13`. This is the ONE
# bash definition; scripts/bin/openspec-status sources this file for it rather
# than duplicating. The Python-side twin is `BEADS_ID_RE` in scripts/bin/spec-sync,
# duplicated (with a lockstep cross-reference comment) into
# scripts/bin/{triage-list-drafts,deferred-specs,wave-plan-build,
# apply-resume-detect,wave-extend-scan}. See
# openspec/changes/shared-bead-id-marker-parser/ for the full rationale — do not
# introduce a third, independently-scoped character class anywhere in scripts/.
# ---------------------------------------------------------------------------
BEADS_ID_RE='[A-Za-z0-9][A-Za-z0-9._-]*'

# ---------------------------------------------------------------------------
# extract_beads_ids <tasks.md>
# Parses a tasks.md file and sets:
#   BEADS_EPIC_ID          — from <!-- beads:epic:XXX --> comment (first match)
#   BEADS_TASK_IDS         — space-separated, deduplicated, from ALL [beads:XXX]
#                             refs regardless of checkbox state. Used as the
#                             close_orphaned_beads exclude-list so unchecked/
#                             deferred task beads are protected from being
#                             swept as "orphans" too — do NOT use this set to
#                             decide what to close.
#   BEADS_TASK_IDS_CHECKED — space-separated, deduplicated, from [beads:XXX]
#                             refs on lines whose checkbox is `- [x]` only.
#                             This is the set that is safe to close.
# Returns 1 if file does not exist.
# ---------------------------------------------------------------------------
extract_beads_ids() {
  local tasks_file="${1:?extract_beads_ids: tasks.md path required}"
  BEADS_EPIC_ID=""
  BEADS_FEATURE_ID=""
  BEADS_TASK_IDS=""
  BEADS_TASK_IDS_CHECKED=""

  if [[ ! -f "$tasks_file" ]]; then
    return 1
  fi

  BEADS_EPIC_ID=$(grep -oP "(?<=beads:epic:)${BEADS_ID_RE}" "$tasks_file" 2>/dev/null | head -1)
  BEADS_FEATURE_ID=$(grep -oP "(?<=beads:feature:)${BEADS_ID_RE}" "$tasks_file" 2>/dev/null | head -1)
  # Canonical grammar (BEADS_ID_RE, defined above) allows dots so dotted
  # sub-task IDs (the bd_mint --parent auto-suffix form, e.g. bd-child1.1) match
  # — the prior \w+-\w+ class alone silently dropped them (bd-alpha1.9;
  # reproduced closing route-improve-lenses-direct-to-proposals 2026-07-15, 3
  # dotted beads left open after close_beads_all reported clean).
  BEADS_TASK_IDS=$(grep -oP "(?<=\[beads:)${BEADS_ID_RE}(?=\])" "$tasks_file" 2>/dev/null | sort -u | tr '\n' ' ' | sed 's/ $//')
  BEADS_TASK_IDS_CHECKED=$(grep -oP "^\s*-\s*\[[xX]\].*\[beads:\K${BEADS_ID_RE}(?=\])" "$tasks_file" 2>/dev/null | sort -u | tr '\n' ' ' | sed 's/ $//')
}

# ---------------------------------------------------------------------------
# close_beads_all <spec_name> <tasks.md>
#
# Closes task beads referenced by a spec WHOSE tasks.md CHECKBOX IS CHECKED
# (`- [x]`), then conditionally closes the feature bead. The capability epic
# is intentionally NOT closed — per the 3-level beads model
# (rules/BEADS.md § Hierarchy), epics live for months/years and accumulate
# multiple features over their lifetime.
#
# Task close logic:
#   - Only task beads on a `- [x]` checked line are closed (BEADS_TASK_IDS_CHECKED).
#   - A task still `- [ ]` unchecked — e.g. a deferred [user] manual-verification
#     task — is left open. Closing an unchecked task fabricates completion and
#     violates rules/CORE.md's Verification Iron Law + deferred-dialect ban.
#     (bd-example; independently reproduced in multiple repositories — see
#     rules/BEADS.md § Promotion-on-Repeat.)
#
# Feature close logic:
#   - Feature closes only if it has zero remaining open parent-child children
#     (queried live from bd, not from tasks.md text — so an unchecked task
#     left open above correctly keeps the feature open too).
#   - If [user] or deferred tasks remain open under the feature, the feature
#     stays open as the tracking thread for the residual observability work.
#
# Failures are non-fatal.
# ---------------------------------------------------------------------------
close_beads_all() {
  local spec_name="${1:?close_beads_all: spec name required}"
  local tasks_file="${2:?close_beads_all: tasks.md path required}"

  extract_beads_ids "$tasks_file"

  # Concurrent /apply sessions must not race on .beads/issues.jsonl flush.
  # Serialize the entire batch close via flock on a sidecar lock file. The
  # lock is scoped to the current repo (resolve via git rev-parse). If
  # flock is unavailable (rare — coreutils on Mac), fall through unguarded.
  # See OpenSpec apply-concurrent-session-isolation task 2.9 (bd-example).
  local repo_root lock_file
  repo_root=$(git rev-parse --show-toplevel 2>/dev/null)
  if [[ -n "$repo_root" && -d "$repo_root/.beads" ]] && command -v flock >/dev/null 2>&1; then
    lock_file="$repo_root/.beads/issues.jsonl.lock"
    touch "$lock_file" 2>/dev/null || true
    (
      # Bare-numeric fd redirect parse-fails under zsh (see worktree-helpers.sh
      # _wt_with_index_lock for the full explanation) — `{varname}>file` dynamic-fd
      # allocation is the portable form for both bash and zsh.
      exec {_close_beads_lock_fd}>"$lock_file"
      flock -w 10 -x "$_close_beads_lock_fd" || { echo "close_beads_all: flock timeout on $lock_file" >&2; exit 1; }
      _close_beads_all_inner "$spec_name"
    )
  else
    _close_beads_all_inner "$spec_name"
  fi
}

# Inner batch — invoked under the flock guard (or directly when flock is
# unavailable). Keeps the bd CLI calls in one block so the lock window is
# tight (<500ms expected per the design doc).
_close_beads_all_inner() {
  local spec_name="$1"

  # 1. Close only CHECKED task beads (- [x]) — an unchecked (- [ ]) task,
  # e.g. a deferred [user] task, must stay open. See bd-example.
  if [[ -n "$BEADS_TASK_IDS_CHECKED" ]]; then
    bd close $BEADS_TASK_IDS_CHECKED --reason="Archived via /apply $spec_name" 2>/dev/null || true
  fi

  # 2. Conditionally close the feature: only if it has no remaining open children
  #
  # bd-example: the prior form of this check queried `bd list --status=open --json`
  # for records carrying a `.deps[]` array with `{type, target}` entries -- but
  # `bd list --json` (bd 1.0.3) never emits a `.deps` field at all, only
  # `dependency_count`/`dependent_count` integers. That made `remaining_open`
  # unconditionally empty, so this step force-closed the feature regardless of
  # open children (reproduced live 2026-07-15, closed bd-feature while bd-child
  # was still open). A later revision switched to `bd show <feature-id> --json`
  # + `.dependents[]`, but bd 1.1.0's real `bd show --json` shape has no
  # `dependents[]` array at all (see docs/reference/bd-1.1.0-baseline.md §
  # `bd show <id> --json`) -- that revision silently reproduced the exact same
  # bd-example bug under the new bd version.
  #
  # FIXED 2026-07-19 (bd-1-1-0-reconciliation-and-hook-refresh, verified live):
  # `bd dep list <id> --direction=up --type parent-child --json` is the correct
  # composed command -- not a hand-rolled jq join. It returns a FLAT ARRAY of
  # the full issue record for every child that depends on <id> as its parent
  # (each record carries the child's own top-level `status` field directly,
  # plus an appended `dependency_type` field), already filtered server-side to
  # the `parent-child` type via `--type`. No second `bd show`/`bd list` lookup
  # is needed -- the join bd's own `dep list` subcommand already performs is
  # exactly what a hand-rolled two-step jq join would have reconstructed.
  # Verified live against bd-epic (a real epic with 29 parent-child children
  # in a mix of open/in_progress/deferred/closed states): `bd dep list bd-epic
  # --direction=up --type parent-child --json | jq -r '.[] | "\(.id)\t\(.status)"'`
  # returned each child's real id + status (e.g. `bd-open open`, `bd-closed
  # closed`), confirming `.status` is present directly on each record with no
  # join step. An invalid id returns `{"error":...}` + exit 1, which
  # `2>/dev/null` alone does not suppress (the error JSON is on stdout) --
  # piped through the same jq filter below it produces jq exit 5 and an empty
  # `remaining_open`, matching this function's pre-existing fail-open-on-error
  # contract (unchanged from before this fix).
  if [[ -n "$BEADS_FEATURE_ID" ]]; then
    local remaining_open
    remaining_open=$(bd dep list "$BEADS_FEATURE_ID" --direction=up --type parent-child --json 2>/dev/null | \
      jq -r '[.[] | select(.status != "closed")] | length' \
      2>/dev/null)
    if [[ -z "$remaining_open" || "$remaining_open" == "0" ]]; then
      bd close "$BEADS_FEATURE_ID" \
        --reason="All task children closed; feature complete via /apply $spec_name" \
        2>/dev/null || true
    else
      echo "  [info] feature $BEADS_FEATURE_ID has $remaining_open open child task(s) — leaving open"
    fi
  fi

  # 3. Capability epic ($BEADS_EPIC_ID) is intentionally NOT closed here.
  # See rules/BEADS.md § Hierarchy.
}

# ---------------------------------------------------------------------------
# close_orphaned_beads <spec_name> [exclude_ids]
# Searches for open issues matching spec name, filters out already-closed IDs,
# and closes any remaining orphans. Failures are non-fatal.
# ---------------------------------------------------------------------------
close_orphaned_beads() {
  local spec_name="${1:?close_orphaned_beads: spec name required}"
  local exclude_ids="${2:-}"

  local orphans
  orphans=$(bd search "$spec_name" --status=open --json 2>/dev/null | \
    jq -r --arg name "$spec_name" \
    '.[] | select(.title | test("(^|\\W)" + $name + "(\\W|$)")) | .id' 2>/dev/null)

  if [[ -n "$exclude_ids" ]]; then
    orphans=$(echo "$orphans" | grep -v -F "$exclude_ids" 2>/dev/null | grep -v '^$' | tr '\n' ' ')
  else
    orphans=$(echo "$orphans" | grep -v '^$' | tr '\n' ' ')
  fi

  if [[ -n "$orphans" ]]; then
    bd close $orphans --reason="Orphaned issue -- spec $spec_name archived" 2>/dev/null || true
  fi
}

# ---------------------------------------------------------------------------
# bd_mint — single sanctioned mint path for bash sites (advisor-plans/037).
#
# DESIGN (multi-file Design Gate, rules/CORE.md):
#   Inputs: --title/--type/--priority required; exactly one of
#     --parent <id> | --unsorted | --standalone required for non-epic types;
#     optional --labels a,b,c / --description / --spec-id / --json.
#   Validation runs BEFORE any bd call: hard failures (missing required arg,
#     banned label shape, missing parent choice) exit 1 with stderr reason —
#     reject, never guess. Soft violations (banned title token, unsanctioned
#     [PREFIX], overlong title) are FIXED or WARNED on stderr and the create
#     proceeds — the mint must not drop work on style; plan 038's lint is the
#     ratchet. Title >72 truncates at a word boundary + "..." and the FULL
#     title is prepended to --description so nothing is lost. bd create
#     failure (network/lock) exits 2 so callers can distinguish validation
#     rejects from runtime faults.
#   Calls: one `bd create` (with --parent when given — creates the
#     parent-child dep AND inherits parent labels), plus at most one epic
#     lookup for --unsorted. No set -e (sourced lib; guard at file top).
#
# Usage:
#   bd_mint --title "..." --type task|bug|feature|chore|epic --priority 0..4
#           [--parent <id> | --unsorted | --standalone]
#           [--labels a,b,c] [--description "..."] [--spec-id <slug>] [--json]
# Emits created id on stdout (full JSON with --json).
# ---------------------------------------------------------------------------
# BD_MINT_TITLE_MAX — sourced from scripts/config/bead-hygiene.json's
# title_max at source time (cached in this variable, not re-read per call).
# Falls back silently to the doctrine default (72, rules/BEADS.md § Bead
# Hygiene Standard rule 1) if the config is missing, unreadable, or fails to
# parse — no info-level stderr on the fallback path (spec-sync-mint-dedup
# 2.1). scripts/bin/spec-sync + scripts/bin/bead-intake read the same file.
#
# NOTE: resolved relative to THIS file's own location (BASH_SOURCE[0]), not
# `git rev-parse --show-toplevel` of the caller's cwd. The packaged helper may
# run while cwd is any downstream project, so cwd-relative configuration would
# silently select the wrong repository.
BD_MINT_TITLE_MAX=72
_bd_hygiene_lib_dir="$(dirname "${BASH_SOURCE[0]}")"
_bd_hygiene_cfg="$_bd_hygiene_lib_dir/../config/bead-hygiene.json"
if [[ -f "$_bd_hygiene_cfg" ]] && command -v jq >/dev/null 2>&1; then
  _bd_hygiene_title_max=$(jq -r '.title_max // empty' "$_bd_hygiene_cfg" 2>/dev/null)
  [[ "$_bd_hygiene_title_max" =~ ^[0-9]+$ ]] && BD_MINT_TITLE_MAX="$_bd_hygiene_title_max"
  unset _bd_hygiene_title_max
fi
unset _bd_hygiene_cfg _bd_hygiene_lib_dir

bd_find_open_issue_or_wisp_title() {
  local title="${1:?bd_find_open_issue_or_wisp_title: title required}"
  local found=""

  found=$(bd list --status open --json 2>/dev/null \
    | jq -r --arg title "$title" '[.[] | select(.title == $title)] | first | .id // empty' 2>/dev/null)
  if [ -n "$found" ]; then
    printf '%s\n' "$found"
    return 0
  fi

  bd mol wisp list --json 2>/dev/null \
    | jq -r --arg title "$title" '[.wisps[]? | select(.title == $title and .status == "open")] | first | .id // empty' 2>/dev/null
}

bd_mint() {
  local title="" type="" priority="" parent="" description="" spec_id="" labels="" wisp_type=""
  local unsorted=false standalone=false want_json=false ephemeral=false
  while [ $# -gt 0 ]; do
    case "$1" in
      --title)       title="$2"; shift 2 ;;
      --type)        type="$2"; shift 2 ;;
      --priority)    priority="$2"; shift 2 ;;
      --parent)      parent="$2"; shift 2 ;;
      --unsorted)    unsorted=true; shift ;;
      --standalone)  standalone=true; shift ;;
      --labels)      labels="$2"; shift 2 ;;
      --description) description="$2"; shift 2 ;;
      --spec-id)     spec_id="$2"; shift 2 ;;
      --ephemeral)   ephemeral=true; shift ;;
      --wisp-type)   wisp_type="$2"; shift 2 ;;
      --json)        want_json=true; shift ;;
      --help|-h)
        echo "usage: bd_mint --title T --type task|bug|feature|chore|epic --priority 0..4 [--parent ID | --unsorted | --standalone | --ephemeral --wisp-type TYPE] [--labels a,b,c] [--description D] [--spec-id S] [--json]"
        return 0 ;;
      *) echo "bd_mint: unknown arg: $1" >&2; return 1 ;;
    esac
  done

  # ── Hard validation (exit 1 before any bd call) ──
  [ -n "$title" ]    || { echo "bd_mint: --title required" >&2; return 1; }
  case "$type" in task|bug|feature|chore|epic) ;; *) echo "bd_mint: --type must be task|bug|feature|chore|epic (got '$type')" >&2; return 1 ;; esac
  case "$priority" in 0|1|2|3|4) ;; *) echo "bd_mint: --priority must be 0..4 (got '$priority')" >&2; return 1 ;; esac
  local parent_flags=0
  [ -n "$parent" ] && parent_flags=$((parent_flags+1))
  $unsorted && parent_flags=$((parent_flags+1))
  $standalone && parent_flags=$((parent_flags+1))
  if $ephemeral; then
    [ -n "$wisp_type" ] || { echo "bd_mint: --ephemeral requires --wisp-type" >&2; return 1; }
    if [ "$parent_flags" -ne 0 ]; then
      echo "bd_mint: --ephemeral rejects --parent/--unsorted/--standalone (wisps are hierarchy-exempt by design)" >&2
      return 1
    fi
  else
    [ -z "$wisp_type" ] || { echo "bd_mint: --wisp-type requires --ephemeral" >&2; return 1; }
    if [ "$type" != "epic" ] && [ "$parent_flags" -ne 1 ]; then
      echo "bd_mint: non-epic mint requires exactly one of --parent/--unsorted/--standalone (rules/BEADS.md § Bead Hygiene Standard rule 4)" >&2
      return 1
    fi
  fi
  local lbl
  if [ -n "$labels" ]; then
    # Split on newlines via `read`, never on shell word-splitting: this file is
    # sourced by bash scripts AND from an interactive zsh, and zsh does not
    # word-split an unquoted `${labels//,/ }`, so the whole comma-list arrived as
    # one malformed label and every multi-label mint failed [beads:bd-example.2].
    # A here-string (not a pipe) keeps the loop in the current shell so `return 1`
    # still aborts the mint.
    while IFS= read -r lbl; do
      [ -n "$lbl" ] || continue
      if printf '%s' "$lbl" | grep -qE '^P[0-9]$|^parallel-[0-9]+$'; then
        echo "bd_mint: banned label '$lbl' (priority-as-label / suffixed structural)" >&2; return 1
      fi
      if ! printf '%s' "$lbl" | grep -qE '^(owner|type|source|gt|hitl):[a-z0-9-]+$|^[a-z][a-z0-9-]*$'; then
        echo "bd_mint: label '$lbl' fails taxonomy (namespaced owner:/type:/source:/gt:/hitl: or bare lowercase word)" >&2; return 1
      fi
    done <<<"$(printf '%s' "$labels" | tr ',' '\n')"
  fi

  # ── Title normalization (warn + fix, never reject) ──
  local orig_title="$title"
  title=$(printf '%s' "$title" | sed -E 's/^\(?\[?[0-9]+(\.[0-9]+)*\]?\)?[.):]?[[:space:]]+//')
  local stripped
  stripped=$(printf '%s' "$title" | sed -E 's/^\[(BUG|TASK|FEATURE|EPIC)\][[:space:]]*//; s/^P[0-4][[:space:]:-]+//')
  if [ "$stripped" != "$title" ]; then
    echo "bd_mint: warn: stripped banned title token (type/priority are fields, not title text)" >&2
    title="$stripped"
  fi
  if printf '%s' "$title" | grep -qE '^\[' && \
     ! printf '%s' "$title" | grep -qE '^\[(CAPABILITY|PROPOSAL|audit:[^]]+|ratchet|docs-sweep|PROPAGATE|TARGET|user|BLOCKER|MERGED INTO [^]]+)\]'; then
    echo "bd_mint: warn: unsanctioned title prefix in '$title' (see rules/BEADS.md § Bead Hygiene Standard rule 2); creating anyway" >&2
  fi
  if [ "${#title}" -gt "$BD_MINT_TITLE_MAX" ]; then
    local cut="${title:0:$BD_MINT_TITLE_MAX}"
    cut="${cut% *}"
    cut=$(printf '%s' "$cut" | sed -E 's/[ ,;:—-]+$//')
    title="$cut ..."
    description="$orig_title"$'\n\n'"$description"
    echo "bd_mint: warn: title truncated to ${#title} chars; full text preserved in description" >&2
  fi

  # ── Parent resolution ──
  local mint_labels="$labels"
  if $standalone; then
    mint_labels="${mint_labels:+$mint_labels,}standalone"
  fi
  if $unsorted; then
    local unsorted_id
    unsorted_id=$(bd list --type epic --status all --limit 1000 --json 2>/dev/null \
      | jq -r '[.[] | select(.title == "[CAPABILITY] unsorted")] | sort_by(.status == "closed") | first | .id // empty')
    if [ -z "$unsorted_id" ]; then
      unsorted_id=$(bd create "[CAPABILITY] unsorted" --type epic --priority 4 \
        --description "Landing pad for features without a named capability." --json 2>/dev/null \
        | jq -r 'if type=="array" then .[0].id else .id end // empty')
      [ -n "$unsorted_id" ] || { echo "bd_mint: failed to resolve/create unsorted landing pad" >&2; return 2; }
    fi
    parent="$unsorted_id"
  fi

  # ── Single create call ──
  local -a args=(create "$title" --type "$type" --priority "$priority" --json)
  [ -n "$parent" ]      && args+=(--parent "$parent")
  [ -n "$description" ] && args+=(--description "$description")
  [ -n "$spec_id" ]     && args+=(--spec-id "$spec_id")
  [ -n "$mint_labels" ] && args+=(--labels "$mint_labels")
  $ephemeral            && args+=(--ephemeral --wisp-type "$wisp_type")
  local out
  out=$(bd "${args[@]}" 2>/dev/null) || { echo "bd_mint: bd create failed" >&2; return 2; }
  if $want_json; then
    printf '%s\n' "$out"
  else
    printf '%s\n' "$out" | jq -r 'if type=="array" then .[0].id else .id end // empty'
  fi
}
