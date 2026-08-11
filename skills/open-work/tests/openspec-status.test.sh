#!/usr/bin/env bash
set -euo pipefail

# Regression test for the 2026-08-06 bd-timeout fix: PHASE 4 used to fan out
# one `bd show <id>` process per unique epic/feature/task ID (xargs -P
# MAX_PARALLEL), which on a repo with many open specs meant hundreds of
# concurrent `bd` invocations racing a SQLite-backed store with no busy
# timeout — reproduced live on otaku-odyssey as 90-300+s wall time and up to
# a 45% "unknown" status rate, timing out the 5-minute preprocessor budget
# `/apply` and `/explore` rely on. The fix replaces the fan-out with a single
# `bd list --id <all-ids> --all --json --limit 0` call. This test asserts
# both halves: (1) `bd` is invoked exactly once for the enrichment step, not
# once per ID, and (2) epic/feature status still resolves correctly from
# that one call.

skill_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
openspec_status="$skill_root/scripts/bin/openspec-status"

[[ -f "$openspec_status" ]] || {
  echo "FAIL: packaged openspec-status is missing: $openspec_status" >&2
  exit 1
}

fixture="$(mktemp -d)"
trap 'rm -rf -- "$fixture"' EXIT
project="$fixture/project"
fake_bin="$fixture/bin"
mkdir -p "$project/openspec/changes/spec-with-open-epic" \
  "$project/openspec/changes/spec-with-closed-epic" \
  "$fake_bin"
git -C "$project" init -q
git -C "$project" config user.email fixture@example.com
git -C "$project" config user.name Fixture
printf '# fixture\n' >"$project/README.md"
git -C "$project" add README.md
git -C "$project" commit -qm init

cat >"$project/openspec/changes/spec-with-open-epic/proposal.md" <<'MD'
# Proposal: spec-with-open-epic
MD
cat >"$project/openspec/changes/spec-with-open-epic/tasks.md" <<'MD'
<!-- beads:epic:fx-open-epic -->
<!-- beads:feature:fx-open-feature -->
# Implementation Tasks
- [ ] 1.1 do the thing [beads:fx-task-1]
MD

cat >"$project/openspec/changes/spec-with-closed-epic/proposal.md" <<'MD'
# Proposal: spec-with-closed-epic
MD
cat >"$project/openspec/changes/spec-with-closed-epic/tasks.md" <<'MD'
<!-- beads:epic:fx-closed-epic -->
# Implementation Tasks
- [x] 1.1 already done [beads:fx-task-2]
MD

# Fake `bd`: records every invocation's args (one line per call, so the test
# can assert call COUNT, not just final content) and, for `bd list --id ...`,
# returns a fixed JSON array covering every ID this fixture references.
cat >"$fake_bin/bd" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$*" >>"$FAKE_BD_CALLS"
if [[ "$1" == "list" ]]; then
  cat <<'JSON'
[
  {"id": "fx-open-epic", "status": "open", "priority": 2},
  {"id": "fx-open-feature", "status": "open", "priority": 2},
  {"id": "fx-task-1", "status": "open", "priority": 2},
  {"id": "fx-closed-epic", "status": "closed", "priority": 2},
  {"id": "fx-task-2", "status": "closed", "priority": 2}
]
JSON
  exit 0
fi
echo '[]'
SH
chmod +x "$fake_bin/bd"

calls_file="$fixture/bd.calls"
: >"$calls_file"

output="$(
  cd "$project"
  PATH="$fake_bin:$PATH" FAKE_BD_CALLS="$calls_file" \
    bash "$openspec_status" --json
)"

call_count="$(wc -l <"$calls_file" | tr -d ' ')"
[[ "$call_count" == "1" ]] || {
  echo "FAIL: expected exactly 1 bd invocation for enrichment, got $call_count:" >&2
  cat "$calls_file" >&2
  exit 1
}
grep -q '^list --id .*--all --json --limit 0$' "$calls_file" || {
  echo "FAIL: the one bd call was not the expected bulk 'bd list --id ... --all --json --limit 0':" >&2
  cat "$calls_file" >&2
  exit 1
}
# All IDs must be present in the single --id argument (comma-separated, order
# not guaranteed since ALL_IDS is a bash associative array).
call_line="$(cat "$calls_file")"
for id in fx-open-epic fx-open-feature fx-task-1 fx-closed-epic fx-task-2; do
  [[ "$call_line" == *"$id"* ]] || {
    echo "FAIL: bulk call did not include $id: $call_line" >&2
    exit 1
  }
done

jq -e '
  ([.[] | select(.name=="spec-with-open-epic") | .epic_status] == ["open"]) and
  ([.[] | select(.name=="spec-with-closed-epic") | .epic_status] == ["closed"])
' <<<"$output" >/dev/null || {
  echo "FAIL: epic_status did not resolve correctly from the bulk bd list result:" >&2
  echo "$output" >&2
  exit 1
}

# --no-enrich must skip the bd call entirely (not just skip using its result).
: >"$calls_file"
(
  cd "$project"
  PATH="$fake_bin:$PATH" FAKE_BD_CALLS="$calls_file" \
    bash "$openspec_status" --json --no-enrich >/dev/null
)
noenrich_call_count="$(wc -l <"$calls_file" | tr -d ' ')"
[[ "$noenrich_call_count" == "0" ]] || {
  echo "FAIL: --no-enrich should make zero bd calls, made $noenrich_call_count" >&2
  exit 1
}

echo "PASS: openspec-status enrichment uses a single bulk bd call, not a per-ID fan-out"
