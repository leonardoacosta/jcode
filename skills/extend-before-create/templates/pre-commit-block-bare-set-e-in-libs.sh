#!/usr/bin/env bash
# Pre-commit hook: block a bare file-scope `set -euo pipefail` in a SOURCED lib.
#
# A sourced lib (scripts/lib/*.sh) that runs a bare `set -euo pipefail` leaks
# `set -e`/`set -u` into the CALLING shell — `/apply`, `/feature`, and command bash
# blocks source these libs without `set -e`, so a leaked `set -e` aborts the caller
# on the next benign non-zero command. Sourced libs MUST use the source-guard idiom:
#
#     (return 0 2>/dev/null) || set -euo pipefail
#
# `return` succeeds only inside a sourced context, so the `set` runs only on direct
# execution. This guard rejects any scripts/lib/*.sh whose file-scope strict-mode line
# is bare (not prefixed by the source-guard).
#
# Canon: rules/TOOLING.md § Shell Script Strict Mode (Source-Guard Idiom).
#
# Usage:
#   - As a pre-commit hook: copy to .git/hooks/pre-commit or chain from your existing
#     pre-commit. Scans staged scripts/lib/*.sh.
#   - Ad-hoc: pass file paths as args to scan them directly, e.g.
#       ./pre-commit-block-bare-set-e-in-libs.sh scripts/lib/foo.sh
# Bypass: `git commit --no-verify` (use sparingly, document why).

# Source-guarded itself: strict mode only when executed, never when sourced.
(return 0 2>/dev/null) || set -euo pipefail

# A "bare strict-mode" line is one of these, at file scope, with nothing before it on
# the line. The source-guard form `(return 0 2>/dev/null) || set -euo pipefail` is NOT
# bare (the `set` is preceded by the guard), so it passes.
#   set -euo pipefail
#   set -eu
#   set -euo
# Leading whitespace is allowed (the line may be indented); a `#` comment, a `||`, or
# any other prefix means it is not a bare file-scope strict-mode line.
BARE_RE='^[[:space:]]*set[[:space:]]+-(eu|euo|euo[[:space:]]+pipefail|eu[[:space:]]+pipefail)[[:space:]]*$'

# Collect the lib files to scan: explicit args, else staged scripts/lib/*.sh additions/mods.
files=()
if [ "$#" -gt 0 ]; then
  files=("$@")
else
  while IFS= read -r f; do
    [ -z "$f" ] && continue
    files+=("$f")
  done < <(git diff --cached --name-only --diff-filter=ACM -- 'scripts/lib/*.sh' 2>/dev/null)
fi

failures=()

for file in "${files[@]}"; do
  [ -z "$file" ] && continue
  # Only police sourced libs under scripts/lib/.
  case "$file" in
    scripts/lib/*.sh | */scripts/lib/*.sh) ;;
    *) continue ;;
  esac
  [ -f "$file" ] || continue

  lineno=0
  while IFS= read -r line || [ -n "$line" ]; do
    lineno=$((lineno + 1))
    if printf '%s\n' "$line" | grep -Eq "$BARE_RE"; then
      failures+=("$file:$lineno — bare file-scope strict mode: '${line}'")
    fi
  done < "$file"
done

if [ "${#failures[@]}" -gt 0 ]; then
  echo "ERROR: pre-commit blocked bare strict mode in sourced lib(s):" >&2
  printf '  - %s\n' "${failures[@]}" >&2
  echo "" >&2
  echo "Sourced libs (scripts/lib/*.sh) leak 'set -e'/'set -u' into the caller shell," >&2
  echo "aborting /apply + /feature. Use the source-guard idiom instead:" >&2
  echo "    (return 0 2>/dev/null) || set -euo pipefail" >&2
  echo "Canon: rules/TOOLING.md § Shell Script Strict Mode (Source-Guard Idiom)." >&2
  echo "Bypass: git commit --no-verify (document why in commit message)." >&2
  exit 1
fi

exit 0
