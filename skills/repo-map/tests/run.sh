#!/usr/bin/env bash
# skills/repo-map/tests/run.sh — golden-file regression tests for
# repo-map-render (design.md D8).
#
# For each fixtures/*.json this renders through the real pipeline
# (--offline --regen-golden, so provenance is never stamped and network
# favicon fetches never happen — both are non-deterministic across
# machines/runs) and byte-compares the result against a committed
# golden/*.html. A mismatch means the renderer's output changed — either a
# real regression, or an intentional template/pipeline change whose golden
# needs regenerating:
#   node scripts/bin/repo-map-render --offline --regen-golden <fixture> -o <golden>
#
# Also asserts two cheap structural invariants per fixture, by parsing the
# `var DATA = {...}` blob embedded in the rendered HTML rather than the
# source fixture directly — this catches a renderer bug that silently drops
# data on the way into the template, which a byte-compare against a STALE
# golden would miss:
#   - node count: DATA.nodes.length matches the fixture's own node count
#   - flow-picker data: DATA.flows.length matches the fixture's own flow
#     count (the picker's <option> list is populated from this at runtime —
#     see templates/renderer.html's flowPicker.forEach block)
#
# Finally, renders the same fixture twice and diffs the two outputs — must
# be byte-identical (determinism of the pinned ELK layout invocation, D8).

set -uo pipefail

SCRIPT_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")" && pwd)"
REPO_ROOT="$(cd "$SCRIPT_DIR/../../.." && pwd)"
RENDER="$REPO_ROOT/scripts/bin/repo-map-render"
FIXTURES_DIR="$SCRIPT_DIR/fixtures"
GOLDEN_DIR="$SCRIPT_DIR/golden"

SCRATCH=$(mktemp -d -t repo-map-golden-XXXXXXXX)
trap 'rm -rf "$SCRATCH"' EXIT

FAILED=0

fail() {
  echo "FAIL: $1" >&2
  FAILED=1
}

# extract_field <html> <js-property> — pulls an integer count out of the
# `var DATA = {...}` blob embedded in a rendered HTML file, without needing
# a browser: JSON.parse the blob, print (data[prop] || []).length.
extract_count() {
  local html="$1" prop="$2"
  node -e '
    const fs = require("fs");
    const html = fs.readFileSync(process.argv[1], "utf8");
    const m = html.match(/var DATA = (\{.*?\});\n\s*var COORDS/s);
    if (!m) { console.error("no DATA blob found in " + process.argv[1]); process.exit(1); }
    const data = JSON.parse(m[1]);
    console.log((data[process.argv[2]] || []).length);
  ' "$html" "$prop"
}

run_case() {
  local name="$1"
  local fixture="$FIXTURES_DIR/$name.json"
  local golden="$GOLDEN_DIR/$name.html"
  local out="$SCRATCH/$name.html"

  if [[ ! -f "$fixture" ]]; then
    fail "$name: fixture missing at $fixture"
    return
  fi
  if [[ ! -f "$golden" ]]; then
    fail "$name: golden missing at $golden (seed it: node $RENDER --offline --regen-golden $fixture -o $golden)"
    return
  fi

  if ! node "$RENDER" --offline --regen-golden "$fixture" -o "$out" >/dev/null 2>&1; then
    fail "$name: render failed"
    return
  fi

  if ! diff -q "$golden" "$out" >/dev/null 2>&1; then
    fail "$name: rendered output drifted from golden — if intentional, run --regen-golden (node $RENDER --offline --regen-golden $fixture -o $golden)"
    return
  fi
  echo "PASS  $name: byte-identical to golden"

  local expected_nodes actual_nodes expected_flows actual_flows
  expected_nodes=$(python3 -c "import json; print(len(json.load(open('$fixture'))['nodes']))")
  actual_nodes=$(extract_count "$out" nodes)
  if [[ "$actual_nodes" != "$expected_nodes" ]]; then
    fail "$name: structural — rendered DATA.nodes.length=$actual_nodes, expected $expected_nodes"
  else
    echo "PASS  $name: structural — node count $actual_nodes"
  fi

  expected_flows=$(python3 -c "import json; print(len(json.load(open('$fixture')).get('flows', [])))")
  actual_flows=$(extract_count "$out" flows)
  if [[ "$actual_flows" != "$expected_flows" ]]; then
    fail "$name: structural — rendered DATA.flows.length=$actual_flows (drives flow-picker option count), expected $expected_flows"
  else
    echo "PASS  $name: structural — flow-picker data count $actual_flows"
  fi

  # Determinism (D8): two consecutive renders of the same fixture must be
  # byte-identical — ELK's layout invocation is pinned, never re-seeded.
  local out2="$SCRATCH/$name.rerender.html"
  node "$RENDER" --offline --regen-golden "$fixture" -o "$out2" >/dev/null 2>&1
  if ! diff -q "$out" "$out2" >/dev/null 2>&1; then
    fail "$name: determinism — two consecutive renders of the same fixture differ"
  else
    echo "PASS  $name: determinism — two consecutive renders are byte-identical"
  fi
}

run_case "valid"
run_case "minimal"

if [[ "$FAILED" == "1" ]]; then
  echo "FAILED: skills/repo-map/tests/run.sh" >&2
  exit 1
fi
echo "PASS: skills/repo-map/tests/run.sh (2/2 fixtures)"
exit 0
