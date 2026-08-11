#!/usr/bin/env bash
set -euo pipefail

skill_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
renderer="$skill_root/scripts/bin/render-open-work"
inventory="$skill_root/tests/fixtures/normalized-inventory.json"

[[ -f "$renderer" ]] || { echo "FAIL: missing normalized renderer: $renderer" >&2; exit 1; }

scratch="$(mktemp -d)"
trap 'rm -rf -- "$scratch"' EXIT
cp -f "$inventory" "$scratch/inventory.json"
before="$(find "$scratch" -type f -print0 | sort -z | xargs -0 sha256sum | sha256sum)"

for harness in claude codex pi; do
  HARNESS_BINDING="$harness" python3 "$renderer" --inventory "$scratch/inventory.json" --mode report >"$scratch/$harness.report"
done
cmp -s "$scratch/claude.report" "$scratch/codex.report"
cmp -s "$scratch/claude.report" "$scratch/pi.report"

report="$(cat "$scratch/claude.report")"
for exact in \
  '6 Beads items remain unresolved: 3 open, 2 in progress, 1 blocked.' \
  'In progress:' \
  'Only open proposal:' \
  'P1 work:' \
  'Other actionable work:' \
  'Open capability containers:' \
  'Active OpenSpec changes:' \
  'Blocked:' \
  'Human-only:' \
  'Open plan rows:' \
  'Archive-ready proposals:' \
  'Source warnings:'; do
  grep -Fqx -- "$exact" <<<"$report" || { echo "FAIL: missing exact heading: $exact" >&2; exit 1; }
done
[[ "$(grep -Fc '`task-open`' <<<"$report")" -eq 1 ]]
[[ "$(grep -Fc '`task-blocked`' <<<"$report")" -eq 1 ]]
[[ "$(grep -Fc '`cap-one`' <<<"$report")" -eq 1 ]]
! grep -Fq 'cached-only' <<<"$report"

python3 "$renderer" --inventory "$scratch/inventory.json" --mode interactive --capabilities none >"$scratch/unavailable.report"
grep -Fq 'Actions unavailable: respond, shell, delegate, apply.' "$scratch/unavailable.report"

after="$(find "$scratch" -type f ! -name '*.report' -print0 | sort -z | xargs -0 sha256sum | sha256sum)"
[[ "$before" == "$after" ]] || { echo 'FAIL: report fixture mutated its inputs' >&2; exit 1; }

echo 'PASS: Claude, Codex, and Pi normalized reports match'
echo 'PASS: report and unavailable-interactive modes preserve fixture bytes'
