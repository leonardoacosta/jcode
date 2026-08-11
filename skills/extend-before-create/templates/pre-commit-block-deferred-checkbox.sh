set -eu

# Blocks the [DEFERRED]/[SKIP]/[BLOCKED]/**DEFERRED** bracketed checkbox
# dialect on newly-added tasks.md checkbox lines (rules/CORE.md Iron Law —
# Verification: "deferral requires beads escalation, not a checkbox
# dialect"). Mirrors the repository validator's `_chk_deferred_dialect` Tier 3 ratchet row
# (workflow-harmony-guardrails Req-1) as a fast local pre-commit gate.
#
# Only inspects ADDED lines in the staged diff (not the whole file) so an
# unrelated commit touching an existing tasks.md never re-flags historical
# content, and strips backtick-quoted spans so a task that illustrates the
# banned tokens as prose (inside backticks) isn't mistaken for real usage.
# The sanctioned alternative, plain prose like "blocked: <reason>" with no
# brackets, never matches.

pattern='\[DEFERRED\]|\[SKIP\]|\[BLOCKED\]|\*\*DEFERRED\*\*'

violations=$(git diff --cached --unified=0 --diff-filter=AM -- '*tasks.md' \
  | grep -E '^\+- \[[ xX]\]' \
  | sed -E 's/`[^`]*`//g' \
  | grep -E "$pattern" \
  || true)

if [ -n "$violations" ]; then
  echo "ERROR: pre-commit blocked the [DEFERRED]/[SKIP]/[BLOCKED] checkbox dialect:" >&2
  echo "" >&2
  printf '%s\n' "$violations" | sed 's/^/  /' >&2
  echo "" >&2
  echo "Deferral requires beads escalation, not a checkbox annotation" >&2
  echo "(rules/CORE.md Iron Law — Verification). Use plain prose instead" >&2
  echo "('blocked: <reason>') and file a beads issue, or drop the annotation." >&2
  echo "" >&2
  echo "Bypass: git commit --no-verify (document why in the commit message)." >&2
  exit 1
fi
