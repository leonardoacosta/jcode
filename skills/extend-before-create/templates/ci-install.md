
# Install: Canonical CI Workflow Template

This is a vendored template, not an npm package. Each T3 Turbo repository copies
`ci-template.yml` into its own `.github/workflows/ci.yml` and owns the copy from then on.
There is no transitive dependency; repositories adopt upstream template changes deliberately.

## Step 1: Vendor the template

From the target repo root:

```bash
mkdir -p .github/workflows
cp ~/.claude/skills/extend-before-create/templates/ci-template.yml \
   .github/workflows/ci.yml
```

## Step 2: Resolve substitution placeholders

The template wraps customization points in `{{ DOUBLE_BRACES }}`. Search for them
after copying:

```bash
grep -nE '\{\{[A-Z_]+\}\}' .github/workflows/ci.yml
```

| Placeholder | What to substitute | Example |
| --- | --- | --- |
| `{{WORKSPACE}}` | The pnpm workspace scope (matches `name` field in root `package.json`) | `acme` |
| `{{REMOVE_IF_NO_INTEGRATION_TESTS}}` | If the repo has only pure-unit tests (no DB-hitting vitest tests), delete the entire `services:` block in the `test` job AND the `POSTGRES_URL` env line. If the repo HAS integration tests, just delete the comment line. | n/a — block surgery |
| `{{ENABLE_E2E_IF_PLAYWRIGHT}}` | If `packages/e2e/` with Playwright exists, delete this comment block. Otherwise delete the entire `e2e:` job. | n/a — block surgery |
| `{{ADJUST_FOR_REAL_DB_E2E}}` | Set BASE_URL secret (preview-against tests) OR adjust PORT (local dev server) | n/a — env-specific |

## Step 3: Verify required composite action exists

The template references `./tooling/github/setup` — a repo-local composite action
that handles pnpm install + node setup. If the target repo doesn't have it:

```bash
ls tooling/github/setup/action.yml
```

If missing, either:
- Copy from another repository that owns the same action and review the vendored result
- OR inline the setup steps in each job (pnpm/action-setup + actions/setup-node)

The composite-action pattern is the T3 Turbo canon; inlining is fallback.

## Step 4: Wire repo secrets + vars

For Vercel Remote Caching (optional but recommended):

```
Repo settings -> Actions -> Variables:
  TURBO_TEAM = <your-vercel-team-slug>

Repo settings -> Actions -> Secrets:
  TURBO_TOKEN = <vercel-remote-cache-token>
```

Without these, builds still succeed but you lose the cache speedup.

For e2e (if enabled):

```
Repo settings -> Actions -> Secrets:
  BASE_URL = <preview-deploy-url>  # if testing against deployed previews
```

## Step 5: Stack-specific deviations

### Standard T3 Turbo

No deviations needed — the template applies as-is after substitution.

### Bun + Effect 4.0

Bun-based services should use a Bun-specific CI workflow instead of copying the pnpm template
unchanged. A typical gate is `bun fmt`, `bun lint`, `bun typecheck`, and `bun run test`; follow the
target repository's own instructions for exact commands.

### Mobile (Expo) repos

If the target ships an Expo app under `apps/expo/`, add an EAS preview job:

```yaml
  expo-preview:
    runs-on: ubuntu-latest
    if: ${{ github.event_name == 'pull_request' }}
    steps:
      - uses: actions/checkout@v5
      - name: Setup
        uses: ./tooling/github/setup
      - name: Trigger EAS update preview
        run: pnpm --filter expo eas update --branch preview --message "${{ github.event.pull_request.title }}"
        env:
          EXPO_TOKEN: ${{ secrets.EXPO_TOKEN }}
```

### Repos without Playwright e2e

Delete the entire `e2e:` job block. No further changes.

## Step 6: Test locally before pushing

```bash
# Re-run each job's commands locally to catch obvious failures:
pnpm lint && pnpm lint:ws
pnpm format
pnpm typecheck
pnpm test
pnpm turbo run build
pnpm fallow audit
```

If all pass locally, push to a feature branch + open a draft PR to confirm CI
fires correctly before merging.

## Step 7: Commit + push

```bash
git add .github/workflows/ci.yml
# If you added tooling/github/setup: also stage that
git commit -m "ci: add canonical CI workflow"
git push
```

## Why this template is vendored

Vendoring a reviewed baseline avoids repeatedly authoring the same workflow and keeps quality gates
consistent while allowing repository-specific changes.

When this template needs updating (for example, a new Node version or Turbo flag), update it here.
Target repositories can compare their owned copy during a periodic refresh:

```bash
# Compare canon vs. repo's current ci.yml
diff "$EXTEND_BEFORE_CREATE_SKILL_DIR/templates/ci-template.yml" \
     .github/workflows/ci.yml
```

Apply reviewed deltas or re-copy the template and resolve its placeholders. Repository CI files are
owned copies, so intentional drift is expected.
