#!/usr/bin/env bash
# Pre-commit hook: block hand-written TypeScript migration files.
#
# Rejects staged files under packages/*/src/migrations/*.ts
#
# Why: drizzle-kit generates SQL migrations into packages/*/drizzle/ — these
# are the canonical migration artifact. TypeScript migration scripts under
# src/migrations/ bypass the drizzle-kit workflow, miss the dotenv -e wrapper
# that other scripts use, and (per the operations audit finding) can ship with real
# correctness bugs that are hard to review.
#
# Allowed locations for backfill/seed scripts:
#   packages/*/src/scripts/<name>.ts   (wired via pnpm with-env tsx)
#   packages/db/src/seed.ts            (drizzle seed convention)
#
# Install: copy to .git/hooks/pre-commit or chain from your existing pre-commit.
# Bypass: `git commit --no-verify` (use sparingly, document why).

set -eu

failures=()

while IFS= read -r file; do
  [ -z "$file" ] && continue
  case "$file" in
    packages/*/src/migrations/*.ts)
      failures+=("$file")
      ;;
  esac
done < <(git diff --cached --name-only --diff-filter=AM)

if [ ${#failures[@]} -gt 0 ]; then
  echo "ERROR: pre-commit blocked .ts migration files:" >&2
  printf '  - %s\n' "${failures[@]}" >&2
  echo "" >&2
  echo "Rationale: migrations should be drizzle-kit SQL artifacts in packages/*/drizzle/." >&2
  echo "If this is a one-off backfill, move to packages/*/src/scripts/<name>.ts and" >&2
  echo "wire as a pnpm script with 'pnpm with-env tsx src/scripts/<name>.ts'." >&2
  echo "Bypass: git commit --no-verify (document why in commit message)." >&2
  exit 1
fi
