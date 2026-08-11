#!/usr/bin/env bash
# Pre-commit hook: block agent-generated doc rot from landing in the repo.
#
# Rejects staged files matching the audit-flagged doc-rot patterns:
#   - *_SUMMARY.md, *_IMPLEMENTATION.md, *_QUICK_REFERENCE.md, *_AT_A_GLANCE.md
#   - *.bak, *.log at any depth
#   - Root-level: test.js, update-*.{ts,js}, *.py (one-off scripts that should live in scripts/)
#
# Install: copy to .git/hooks/pre-commit or chain from your existing pre-commit.
# Bypass: `git commit --no-verify` (use sparingly, document why).

set -eu

# Patterns to reject (basename + path glob mix)
declare -a REJECT_PATTERNS=(
  '*_SUMMARY.md'
  '*_IMPLEMENTATION.md'
  '*_QUICK_REFERENCE.md'
  '*_AT_A_GLANCE.md'
  '*.bak'
  '*.log'
)

# Root-level only patterns (depth=0)
declare -a ROOT_REJECT_PATTERNS=(
  'test.js'
  'update-*.ts'
  'update-*.js'
  '*.py'
)

failures=()

# Check staged files via git diff --cached --name-only --diff-filter=A
while IFS= read -r file; do
  [ -z "$file" ] && continue

  # Anywhere-in-tree rejections
  for pat in "${REJECT_PATTERNS[@]}"; do
    case "$(basename "$file")" in
      $pat)
        failures+=("$file — matches doc-rot pattern '$pat'")
        ;;
    esac
  done

  # Root-only rejections (file path has no slash = root level)
  case "$file" in
    */*) ;;
    *)
      for pat in "${ROOT_REJECT_PATTERNS[@]}"; do
        case "$file" in
          $pat)
            failures+=("$file — root-level one-off script (move to scripts/)")
            ;;
        esac
      done
      ;;
  esac
done < <(git diff --cached --name-only --diff-filter=A)

if [ ${#failures[@]} -gt 0 ]; then
  echo "ERROR: pre-commit blocked the following files:" >&2
  printf '  - %s\n' "${failures[@]}" >&2
  echo "" >&2
  echo "Rationale: audit-flagged doc-rot or root-level junk. Move/delete or rename." >&2
  echo "Bypass: git commit --no-verify (document why in commit message)." >&2
  exit 1
fi
