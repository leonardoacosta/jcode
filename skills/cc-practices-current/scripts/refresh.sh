#!/usr/bin/env bash
#
# Refresh CC currency cache from upstream sources.
#
# Sources:
#   1. GitHub CHANGELOG.md (markdown, canonical, always current)
#   2. GitHub releases API (JSON)
#   3. npm package metadata (JSON)
#   4. beads releases API (JSON)
#   5. OpenSpec releases API (JSON)
#   6-10. CC reference docs (env-vars, tools, hooks, plugins, channels) — sha256-hash-diffed,
#         since these five pages drift independently of CHANGELOG.md's "Added X" prose (a
#         silent reference-doc update, e.g. a new hook schema field, carries no changelog
#         bullet and would otherwise never surface as a signal). Added 2026-07-21 after
#         /workflow:evolve's changelog-only coverage was found not to track these pages at all.
#
# Exit codes:
#   0  all sources unchanged since last check, everything healthy
#   1  at least one fetch failed AND no source moved (stale caveat)
#   2  at least one source moved; references need regeneration
#
# Honors CC_PRACTICES_OFFLINE=1 to skip all network calls (returns 0).

set -uo pipefail

SKILL_DIR="$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)"
STATE_DIR="$SKILL_DIR/state"
CACHE_DIR="$STATE_DIR/cache"
STATE_FILE="$STATE_DIR/last-checked.json"

mkdir -p "$CACHE_DIR"

# Ensure state file exists with safe defaults for first run.
if [[ ! -s "$STATE_FILE" ]]; then
  cat > "$STATE_FILE" <<'EOF'
{
  "last_checked": "never",
  "docs_hash": "none",
  "github_latest": "none",
  "npm_latest": "none",
  "npm_stable": "none",
  "env_vars_hash": "none",
  "tools_hash": "none",
  "hooks_hash": "none",
  "plugins_hash": "none",
  "channels_hash": "none",
  "last_change_detected": "never"
}
EOF
fi

if [[ "${CC_PRACTICES_OFFLINE:-0}" == "1" ]]; then
  echo "cc-practices-current: offline mode, skipping fetch" >&2
  exit 0
fi

FETCH_TIMEOUT="${CC_PRACTICES_TIMEOUT:-10}"
ERRORS=0

prior_field() {
  jq -r "$1 // \"none\"" "$STATE_FILE" 2>/dev/null || echo "none"
}

# ---------- Source 1: GitHub CHANGELOG.md ----------
CHANGELOG_URL="${CC_PRACTICES_CHANGELOG_URL:-https://raw.githubusercontent.com/anthropics/claude-code/main/CHANGELOG.md}"
CHANGELOG_TMP="$CACHE_DIR/.changelog.md.tmp"
CHANGELOG_OUT="$CACHE_DIR/changelog.md"

if curl -fsSL --max-time "$FETCH_TIMEOUT" "$CHANGELOG_URL" -o "$CHANGELOG_TMP" 2>/dev/null \
   && [[ -s "$CHANGELOG_TMP" ]]; then
  mv "$CHANGELOG_TMP" "$CHANGELOG_OUT"
  DOCS_HASH="sha256:$(sha256sum "$CHANGELOG_OUT" | awk '{print $1}')"
else
  rm -f "$CHANGELOG_TMP"
  echo "cc-practices-current: changelog fetch FAILED ($CHANGELOG_URL)" >&2
  ERRORS=$((ERRORS + 1))
  DOCS_HASH=$(prior_field '.docs_hash')
fi

# ---------- Source 2: GitHub releases ----------
GH_URL="https://api.github.com/repos/anthropics/claude-code/releases?per_page=20"
GH_TMP="$CACHE_DIR/.github-releases.json.tmp"
GH_OUT="$CACHE_DIR/github-releases.json"

if curl -fsSL --max-time "$FETCH_TIMEOUT" \
      -H "Accept: application/vnd.github+json" \
      -H "User-Agent: cc-practices-current/1.0" \
      "$GH_URL" -o "$GH_TMP" 2>/dev/null \
   && jq -e 'type == "array"' "$GH_TMP" >/dev/null 2>&1; then
  mv "$GH_TMP" "$GH_OUT"
  GH_LATEST=$(jq -r '.[0].tag_name // "none"' "$GH_OUT")
else
  rm -f "$GH_TMP"
  echo "cc-practices-current: github fetch FAILED" >&2
  ERRORS=$((ERRORS + 1))
  GH_LATEST=$(prior_field '.github_latest')
fi

# ---------- Source 3: npm metadata ----------
NPM_TMP="$CACHE_DIR/.npm.json.tmp"
NPM_OUT="$CACHE_DIR/npm.json"

if timeout "$FETCH_TIMEOUT" npm view @anthropic-ai/claude-code --json >"$NPM_TMP" 2>/dev/null \
   && jq -e '.version' "$NPM_TMP" >/dev/null 2>&1; then
  mv "$NPM_TMP" "$NPM_OUT"
  NPM_LATEST=$(jq -r '.["dist-tags"].latest // .version // "none"' "$NPM_OUT")
  NPM_STABLE=$(jq -r '.["dist-tags"].stable // "none"' "$NPM_OUT")
else
  rm -f "$NPM_TMP"
  echo "cc-practices-current: npm fetch FAILED" >&2
  ERRORS=$((ERRORS + 1))
  NPM_LATEST=$(prior_field '.npm_latest')
  NPM_STABLE=$(prior_field '.npm_stable')
fi

# ---------- Source 4: beads releases (gastownhall/beads) ----------
BD_GH_URL="https://api.github.com/repos/gastownhall/beads/releases?per_page=20"
BD_GH_TMP="$CACHE_DIR/.beads-releases.json.tmp"
BD_GH_OUT="$CACHE_DIR/beads-releases.json"

if curl -fsSL --max-time "$FETCH_TIMEOUT" \
      -H "Accept: application/vnd.github+json" \
      -H "User-Agent: cc-practices-current/1.0" \
      "$BD_GH_URL" -o "$BD_GH_TMP" 2>/dev/null \
   && jq -e 'type == "array"' "$BD_GH_TMP" >/dev/null 2>&1; then
  mv "$BD_GH_TMP" "$BD_GH_OUT"
  BEADS_LATEST=$(jq -r '.[0].tag_name // "none"' "$BD_GH_OUT")
else
  rm -f "$BD_GH_TMP"
  echo "cc-practices-current: beads releases fetch FAILED" >&2
  ERRORS=$((ERRORS + 1))
  BEADS_LATEST=$(prior_field '.beads_latest')
fi

# ---------- Source 5: OpenSpec releases (Fission-AI/OpenSpec) ----------
OS_GH_URL="https://api.github.com/repos/Fission-AI/OpenSpec/releases?per_page=20"
OS_GH_TMP="$CACHE_DIR/.openspec-releases.json.tmp"
OS_GH_OUT="$CACHE_DIR/openspec-releases.json"

if curl -fsSL --max-time "$FETCH_TIMEOUT" \
      -H "Accept: application/vnd.github+json" \
      -H "User-Agent: cc-practices-current/1.0" \
      "$OS_GH_URL" -o "$OS_GH_TMP" 2>/dev/null \
   && jq -e 'type == "array"' "$OS_GH_TMP" >/dev/null 2>&1; then
  mv "$OS_GH_TMP" "$OS_GH_OUT"
  OPENSPEC_LATEST=$(jq -r '.[0].tag_name // "none"' "$OS_GH_OUT")
else
  rm -f "$OS_GH_TMP"
  echo "cc-practices-current: openspec releases fetch FAILED" >&2
  ERRORS=$((ERRORS + 1))
  OPENSPEC_LATEST=$(prior_field '.openspec_latest')
fi

# ---------- Sources 6-10: CC reference docs (sha256 hash-diff, same pattern as changelog) ----------
fetch_doc_hash() {
  # $1=name (for logging/vars) $2=url $3=cache-out-path
  # Unlike changelog.md (inherently a delta), these are full reference pages -- a hash
  # change alone doesn't say WHAT moved. Preserve the pre-change copy as <name>.prev.md
  # so Step 2 extraction can `diff` old vs new instead of re-reading the whole page cold.
  local name="$1" url="$2" out="$3" tmp="${3}.tmp"
  if curl -fsSL --max-time "$FETCH_TIMEOUT" "$url" -o "$tmp" 2>/dev/null && [[ -s "$tmp" ]]; then
    local new_hash="sha256:$(sha256sum "$tmp" | awk '{print $1}')"
    local prior_hash
    prior_hash=$(prior_field ".${name}_hash")
    if [[ -f "$out" && "$new_hash" != "$prior_hash" ]]; then
      cp "$out" "${out%.md}.prev.md"
    fi
    mv "$tmp" "$out"
    echo "$new_hash"
  else
    rm -f "$tmp"
    echo "cc-practices-current: $name fetch FAILED ($url)" >&2
    ERRORS=$((ERRORS + 1))
    prior_field ".${name}_hash"
  fi
}

DOCS_BASE="${CC_PRACTICES_DOCS_BASE:-https://code.claude.com/docs/en}"
ENV_VARS_HASH=$(fetch_doc_hash "env_vars" "$DOCS_BASE/env-vars.md" "$CACHE_DIR/env-vars.md")
TOOLS_HASH=$(fetch_doc_hash "tools" "$DOCS_BASE/tools-reference.md" "$CACHE_DIR/tools-reference.md")
HOOKS_HASH=$(fetch_doc_hash "hooks" "$DOCS_BASE/hooks.md" "$CACHE_DIR/hooks.md")
PLUGINS_HASH=$(fetch_doc_hash "plugins" "$DOCS_BASE/plugins-reference.md" "$CACHE_DIR/plugins-reference.md")
CHANNELS_HASH=$(fetch_doc_hash "channels" "$DOCS_BASE/channels-reference.md" "$CACHE_DIR/channels-reference.md")

# ---------- Compare against prior state ----------
PRIOR_DOCS=$(prior_field '.docs_hash')
PRIOR_GH=$(prior_field '.github_latest')
PRIOR_NPM=$(prior_field '.npm_latest')
PRIOR_BEADS=$(prior_field '.beads_latest')
PRIOR_OS=$(prior_field '.openspec_latest')
PRIOR_ENV_VARS=$(prior_field '.env_vars_hash')
PRIOR_TOOLS=$(prior_field '.tools_hash')
PRIOR_HOOKS=$(prior_field '.hooks_hash')
PRIOR_PLUGINS=$(prior_field '.plugins_hash')
PRIOR_CHANNELS=$(prior_field '.channels_hash')
PRIOR_CHANGE=$(prior_field '.last_change_detected')

CHANGED=0
# A "change" means: current value is known AND differs from prior. Unknown ("none")
# does not flip the flag, because we don't want a transient fetch failure to force
# a full regeneration of the references.
for pair in "$DOCS_HASH|$PRIOR_DOCS" "$GH_LATEST|$PRIOR_GH" "$NPM_LATEST|$PRIOR_NPM" "$BEADS_LATEST|$PRIOR_BEADS" "$OPENSPEC_LATEST|$PRIOR_OS" \
            "$ENV_VARS_HASH|$PRIOR_ENV_VARS" "$TOOLS_HASH|$PRIOR_TOOLS" "$HOOKS_HASH|$PRIOR_HOOKS" "$PLUGINS_HASH|$PRIOR_PLUGINS" "$CHANNELS_HASH|$PRIOR_CHANNELS"; do
  current="${pair%%|*}"
  prior="${pair#*|}"
  if [[ "$current" != "none" && "$current" != "$prior" ]]; then
    CHANGED=1
    break
  fi
done

## Force first-run population if references are still empty stubs.
## Catches the multi-machine case: state synced via git, but references not yet populated.
if [[ "$CHANGED" == "0" ]]; then
  for ref in "$SKILL_DIR/references"/*.md; do
    if grep -q "Not yet refreshed" "$ref" 2>/dev/null; then
      CHANGED=1
      echo "cc-practices-current: references still empty stubs — forcing first-run population" >&2
      break
    fi
  done
fi

NOW="$(date -Iseconds)"
LAST_CHANGE="$PRIOR_CHANGE"
if [[ "$CHANGED" == "1" ]]; then
  LAST_CHANGE="$NOW"
fi

# ---------- Write new state atomically ----------
STATE_TMP="$STATE_FILE.tmp"
jq -n \
  --arg last_checked "$NOW" \
  --arg docs_hash "$DOCS_HASH" \
  --arg github_latest "$GH_LATEST" \
  --arg npm_latest "$NPM_LATEST" \
  --arg npm_stable "$NPM_STABLE" \
  --arg beads_latest "$BEADS_LATEST" \
  --arg openspec_latest "$OPENSPEC_LATEST" \
  --arg env_vars_hash "$ENV_VARS_HASH" \
  --arg tools_hash "$TOOLS_HASH" \
  --arg hooks_hash "$HOOKS_HASH" \
  --arg plugins_hash "$PLUGINS_HASH" \
  --arg channels_hash "$CHANNELS_HASH" \
  --arg last_change_detected "$LAST_CHANGE" \
  '{
     last_checked: $last_checked,
     docs_hash: $docs_hash,
     github_latest: $github_latest,
     npm_latest: $npm_latest,
     npm_stable: $npm_stable,
     beads_latest: $beads_latest,
     openspec_latest: $openspec_latest,
     env_vars_hash: $env_vars_hash,
     tools_hash: $tools_hash,
     hooks_hash: $hooks_hash,
     plugins_hash: $plugins_hash,
     channels_hash: $channels_hash,
     last_change_detected: $last_change_detected
   }' > "$STATE_TMP"
mv "$STATE_TMP" "$STATE_FILE"

# ---------- Report & exit ----------
if [[ "$CHANGED" == "1" ]]; then
  echo "cc-practices-current: upstream moved — references need regeneration"
  echo "  docs:     $PRIOR_DOCS -> $DOCS_HASH"
  echo "  github:   $PRIOR_GH -> $GH_LATEST"
  echo "  npm:      $PRIOR_NPM -> $NPM_LATEST"
  [[ "$ENV_VARS_HASH" != "$PRIOR_ENV_VARS" ]] && echo "  env-vars: $PRIOR_ENV_VARS -> $ENV_VARS_HASH"
  [[ "$TOOLS_HASH" != "$PRIOR_TOOLS" ]] && echo "  tools:    $PRIOR_TOOLS -> $TOOLS_HASH"
  [[ "$HOOKS_HASH" != "$PRIOR_HOOKS" ]] && echo "  hooks:    $PRIOR_HOOKS -> $HOOKS_HASH"
  [[ "$PLUGINS_HASH" != "$PRIOR_PLUGINS" ]] && echo "  plugins:  $PRIOR_PLUGINS -> $PLUGINS_HASH"
  [[ "$CHANNELS_HASH" != "$PRIOR_CHANNELS" ]] && echo "  channels: $PRIOR_CHANNELS -> $CHANNELS_HASH"
  exit 2
fi

if [[ "$ERRORS" -gt 0 ]]; then
  echo "cc-practices-current: $ERRORS fetch errors, serving stale references" >&2
  exit 1
fi

echo "cc-practices-current: all sources unchanged (last checked $NOW)"
exit 0
