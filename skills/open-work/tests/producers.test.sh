#!/usr/bin/env bash
set -euo pipefail

skill_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
open_items="$skill_root/scripts/bin/open-items"

[[ -f "$open_items" ]] || {
  echo "FAIL: packaged open-items is missing: $open_items" >&2
  exit 1
}

fixture="$(mktemp -d)"
trap 'rm -rf -- "$fixture"' EXIT
project="$fixture/project"
fake_bin="$fixture/bin"
mkdir -p "$project/.beads" "$project/openspec/changes/live-proposal" "$fake_bin"
git -C "$project" init -q
git -C "$project" config user.email fixture@example.com
git -C "$project" config user.name Fixture
printf '# fixture\n' >"$project/README.md"
git -C "$project" add README.md
git -C "$project" commit -qm init

cat >"$fake_bin/bd" <<'SH'
#!/usr/bin/env bash
printf '%s\n' "$*" >"$FAKE_BD_ARGS"
case "${FAKE_BD_MODE:-ok}" in
  fail) exit 7 ;;
  malformed) printf 'not-json\n' ;;
  *) printf '%s\n' "$FAKE_BD_JSON" ;;
esac
SH
chmod +x "$fake_bin/bd"

live_json="$(cat "$skill_root/tests/fixtures/live-beads.json")"
cached_fixture="$skill_root/tests/fixtures/cached-beads.jsonl"
cp -f "$cached_fixture" "$project/.beads/issues.jsonl"

run_open_items() {
  local mode="$1"
  (
    cd "$project"
    PATH="$fake_bin:$PATH" \
      FAKE_BD_ARGS="$fixture/bd.args" \
      FAKE_BD_MODE="$mode" \
      FAKE_BD_JSON="$live_json" \
      OPEN_WORK_ROOT="$skill_root/scripts" \
      python3 "$open_items" --json --live-beads
  )
}

live_output="$(run_open_items ok)"
jq -e '
  .beads.available == true and
  .beads.counts_source == "bd list --all --json --limit=0 (live)" and
  .beads.total_open == 6 and
  .beads.active_epics == 1 and
  ([.beads.containers[] | select(.id=="container") | .title] == ["[CAPABILITY] durable container"]) and
  .beads.active_proposal_linked == 1 and
  ([.beads.items[].id] | index("cached-only") | not) and
  ([.beads.items[] | select(.id=="live-blocked") | .bucket] == ["blocked"]) and
  ([.beads.items[] | select(.id=="live-human") | .bucket] == ["human_only"]) and
  ([.beads.items[] | select(.id=="live-progress") | .bucket] == ["in_progress"]) and
  ([.beads.items[] | select(.id=="live-answered") | .bucket] == ["open"]) and
  ([.beads.items[] | select(.id=="live-answered") | .dispositioned] == [true]) and
  ([.beads.items[] | select(.id=="live-answered") | .bucket_reason] == ["answered 08-02: proceed with the portable implementation"])
' <<<"$live_output" >/dev/null
[[ "$(cat "$fixture/bd.args")" == "list --all --json --limit=0" ]]

project_code_json="$(jq 'map(if .id == "live-open" then .title = "api migration" else . end)' <<<"$live_json")"
project_code_output="$({
  cd "$project"
  PATH="$fake_bin:$PATH" \
    FAKE_BD_ARGS="$fixture/bd.args" \
    FAKE_BD_JSON="$project_code_json" \
    OPEN_WORK_PROJECT_CODES="api,web" \
    OPEN_WORK_ROOT="$skill_root/scripts" \
    python3 "$open_items" --json --live-beads
})"
jq -e '([.beads.items[] | select(.id=="live-open") | .cross_repo] == ["api"])' \
  <<<"$project_code_output" >/dev/null

for failure_mode in fail malformed; do
  failure_output="$(run_open_items "$failure_mode")"
  jq -e '
    .beads.available == false and
    .beads.source == "live" and
    ([.. | strings] | index("cached-only") | not)
  ' <<<"$failure_output" >/dev/null
done

cached_output="$(cd "$project" && OPEN_WORK_ROOT="$skill_root/scripts" python3 "$open_items" --json)"
jq -e '
  .beads.available == true and
  .beads.counts_source == "issues.jsonl (last bd flush)" and
  ([.beads.items[].id] | index("cached-only") != null)
' <<<"$cached_output" >/dev/null

source_local_output="$(cd "$project" && env -u OPEN_WORK_ROOT PATH="$fake_bin:$PATH" FAKE_BD_ARGS="$fixture/bd.args" FAKE_BD_JSON="$live_json" python3 "$open_items" --json --live-beads)"
jq -e '.beads.available == true and .beads.source == "live"' <<<"$source_local_output" >/dev/null

large_live_json="$(jq -n '[range(0; 31) | {id:("many-" + (.|tostring)), title:"queued work", status:"open", priority:2, issue_type:"task", labels:[], dependencies:[]}]')"
truncated_output="$(
  cd "$project"
  PATH="$fake_bin:$PATH" FAKE_BD_ARGS="$fixture/bd.args" FAKE_BD_JSON="$large_live_json" \
    python3 "$open_items" --json --live-beads
)"
jq -e '.beads.total_open == 31 and .beads.truncated == true and (.beads.items | length) == 30' <<<"$truncated_output" >/dev/null
# Buckets describe every retained bead, not just the visible page: the compact
# headline pairs total_open with these counts, so a capped tally renders
# arithmetic that does not add up.
jq -e '
  .beads.item_cap == 30 and
  ([.beads.bucket_counts[]] | add) == .beads.total_open and
  .beads.bucket_counts.open == 31
' <<<"$truncated_output" >/dev/null

# --limit=0 is the full-list command the renderer names when truncated is true.
unlimited_output="$(
  cd "$project"
  PATH="$fake_bin:$PATH" FAKE_BD_ARGS="$fixture/bd.args" FAKE_BD_JSON="$large_live_json" \
    python3 "$open_items" --json --live-beads --limit=0
)"
jq -e '
  .beads.truncated == false and .beads.item_cap == 0 and
  (.beads.items | length) == 31 and
  ([.beads.bucket_counts[]] | add) == .beads.total_open
' <<<"$unlimited_output" >/dev/null
jq -e '.error == "unsupported arguments" and (.arguments == ["--limit=all"])' \
  <<<"$(cd "$project" && python3 "$open_items" --json --limit=all)" >/dev/null

# A multi-paragraph disposition comment must stay one renderable bullet line.
multiline_json="$(jq '
  map(if .id == "live-answered"
      then .comments = [{text:"HITL answered: first line\n\nsecond line", created_at:"2026-08-02T12:00:00Z"}]
      else . end)' <<<"$live_json")"
multiline_output="$(
  cd "$project"
  PATH="$fake_bin:$PATH" FAKE_BD_ARGS="$fixture/bd.args" FAKE_BD_JSON="$multiline_json" \
    python3 "$open_items" --json --live-beads
)"
jq -e '([.beads.items[] | select(.id=="live-answered") | .bucket_reason]
       == ["answered 08-02: first line second line"])' <<<"$multiline_output" >/dev/null

# One unreadable source degrades that key alone — never the whole document.
if [[ "$(id -u)" != 0 ]]; then
  mkdir -p "$project/plans"
  printf '| # | Title | Status |\n| 001 | a plan | OPEN |\n' >"$project/plans/README.md"
  chmod 000 "$project/plans/README.md"
  isolation_output="$(cd "$project" && OPEN_WORK_ROOT="$skill_root/scripts" python3 "$open_items" --json)"
  chmod 644 "$project/plans/README.md"
  rm -rf "$project/plans"
  jq -e '
    .plans.available == false and (.plans.error | length) > 0 and
    .beads.available == true and (.beads.items | length) > 0 and
    (has("error") | not)
  ' <<<"$isolation_output" >/dev/null
fi

no_beads="$fixture/no-beads"
mkdir -p "$no_beads"
git -C "$no_beads" init -q
no_beads_output="$(cd "$no_beads" && OPEN_WORK_ROOT="$skill_root/scripts" python3 "$open_items" --json --live-beads)"
jq -e '.beads.available == false and .beads.source == "live"' <<<"$no_beads_output" >/dev/null

# triage-list-drafts frontmatter regression. No fixture previously gave that producer a
# proposal.md WITH frontmatter, so parse_frontmatter's return shape was uncovered: a
# tuple-returning success path crashed both `--next-order-code` and the main draft scan
# with AttributeError, while every existing test stayed green because collect_drafts
# skips a directory that has no proposal.md.
triage_bin="$skill_root/scripts/bin/triage-list-drafts"
triage_fixture="$fixture/triage-repo"
mkdir -p "$triage_fixture/openspec/changes/sample-change"
git -C "$triage_fixture" init -q
printf -- '---\norder: 0804a\nafter: other-slug -- waits on the other one\n---\n\n# Proposal\n\n## Context\n- depends on: `other-slug`\n' \
  >"$triage_fixture/openspec/changes/sample-change/proposal.md"
printf -- '## DB Batch\n\n- [ ] 1.1 do the thing\n' \
  >"$triage_fixture/openspec/changes/sample-change/tasks.md"

triage_out="$(cd "$triage_fixture" && python3 "$triage_bin" --json)"
jq -e '
  (.drafts | length) == 1 and
  (.drafts[0].order == "0804a") and
  (.drafts[0].after == "other-slug") and
  (.drafts[0].depends_on == ["other-slug"]) and
  (has("error") | not)
' <<<"$triage_out" >/dev/null

order_out="$(cd "$triage_fixture" && python3 "$triage_bin" --next-order-code --json)"
jq -e '.order_code == "0804b" and (has("error") | not)' <<<"$order_out" >/dev/null

find "$skill_root/scripts" -type f -perm /111 -print -quit | grep -q . && {
  echo 'FAIL: packaged helper source must remain non-executable' >&2
  exit 1
}

echo 'PASS: live Beads is authoritative and never falls back to cached JSONL'
echo 'PASS: cached mode remains explicit and source-labelled'
echo 'PASS: bucket precedence, containers, proposal suppression, and no-Beads behavior hold'
echo 'PASS: disposition comments, progress, truncation, and source-local execution hold'
echo 'PASS: bucket counts cover every retained bead and --limit=0 lifts the cap'
echo 'PASS: free-text disposition text renders on one line'
echo 'PASS: an unreadable source degrades alone, not the whole inventory'
echo 'PASS: triage-list-drafts parses frontmatter and allocates the next order code'
echo 'PASS: packaged helper source is interpreter-invoked read-only content'
