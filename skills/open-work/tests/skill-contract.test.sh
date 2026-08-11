#!/usr/bin/env bash
set -euo pipefail

skill_root="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
repo_root="$(cd "$skill_root/../../.." && pwd)"

required=(
  SKILL.md
  agents/openai.yaml
  references/rendering.md
  references/actions.md
  references/capabilities.md
  references/acceptance.md
  tests/check-interface.py
  tests/cross-harness.test.sh
)
for path in "${required[@]}"; do
  [[ -f "$skill_root/$path" ]] || { echo "FAIL: missing $path" >&2; exit 1; }
done

grep -Fq 'name: open-work' "$skill_root/SKILL.md"
grep -Fq 'default to `interactive`' "$skill_root/SKILL.md"
grep -Fq 'report' "$skill_root/SKILL.md"
grep -Fq 'python3 "${OPEN_WORK_ROOT}/bin/open-items" --json --live-beads' "$skill_root/SKILL.md"
grep -Fq 'python3 "${OPEN_WORK_ROOT}/bin/triage-list-drafts" --json --include-approved' "$skill_root/SKILL.md"
# Both producers carry a python3 shebang. `bash <python-source>` exits 0 while
# emitting ImageMagick `import` noise, so a wrong interpreter here reaches the
# router as a silent success — assert no producer is routed through bash.
! grep -Eq '^bash "\$\{OPEN_WORK_ROOT\}' "$skill_root/SKILL.md"

grep -Fq 'Only open proposal' "$skill_root/references/rendering.md"
grep -Fq 'Open proposals (N)' "$skill_root/references/rendering.md"
grep -Fq 'blocked > in_progress > disposition > human_only > open' "$skill_root/references/rendering.md"
grep -Fq 'archive -> disposition -> dispatch -> apply' "$skill_root/references/actions.md"
grep -Fq 'at most five workers' "$skill_root/references/actions.md"
grep -Fq 'same-repository items sequentially' "$skill_root/references/actions.md"
[[ "$(tr '\n' ' ' < "$skill_root/references/actions.md")" == *'Cross-repository items require a separate explicit confirmation before any mutation in that repository.'* ]]
grep -Fq 'one consolidated' "$skill_root/references/actions.md"
grep -Fq 'perform no mutation' "$skill_root/references/capabilities.md"

prohibited_tokens=(
  'AskUserQuestion'
  'Skill({'
  'run_in_background'
  'CLAUDE_''PLUGIN_ROOT'
  "~/"'.claude'
  'spawn_''agent('
)
for prohibited in "${prohibited_tokens[@]}"; do
  if rg -Fq "$prohibited" "$skill_root/SKILL.md" "$skill_root/references"; then
    echo "FAIL: portable contract contains harness-only token: $prohibited" >&2
    exit 1
  fi
done

python3 -B "$skill_root/tests/check-interface.py" "$skill_root"

# Release metadata lives in the authoring monorepo, not in a scrubbed standalone
# publication of this skill tree. Assert it where it exists; announce the skip
# where it does not, so the whole contract suite still runs from a bare checkout
# instead of aborting on a missing package.json and reporting nothing.
if [[ -f "$repo_root/package.json" ]]; then
  jq -e '.authoredStandards.allowlistedSkills["leo-core"] | index("open-work") != null' "$repo_root/package.json" >/dev/null
  jq -e '.authoredStandards.packages[] | select(.name=="leo-core") | .version == "0.7.0"' "$repo_root/package.json" >/dev/null
  jq -e '.version == "0.7.0"' "$repo_root/leo-core/.claude-plugin/plugin.json" >/dev/null
  jq -e '.plugins[] | select(.name=="leo-core") | .version == "0.7.0"' "$repo_root/.claude-plugin/marketplace.json" >/dev/null
  echo 'PASS: open-work router, references, interface, and release metadata agree'
else
  echo "SKIP: release metadata (no authoring monorepo at $repo_root)"
  echo 'PASS: open-work router, references, and interface agree'
fi
