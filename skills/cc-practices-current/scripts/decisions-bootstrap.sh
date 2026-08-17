#!/usr/bin/env bash
# decisions-bootstrap.sh
# One-time legacy import: convert ~/.claude/docs/audit/evolve/latest.json
# recommendations into seeded entries in state/decisions.json.
#
# Idempotent: skips if decisions.json already has bootstrap markers.
# Each imported entry gets verdict "legacy-import" with the run date as rationale.

set -euo pipefail

SKILL_DIR="$HOME/.claude/skills/cc-practices-current"
STATE_FILE="$SKILL_DIR/state/decisions.json"
LEGACY_LATEST="$HOME/.claude/docs/audit/evolve/latest.json"

# ── Init state file if missing ───────────────────────────────────
mkdir -p "$SKILL_DIR/state"
if [[ ! -f "$STATE_FILE" ]]; then
  echo '{"version":1,"decisions":{}}' > "$STATE_FILE"
fi

# ── Idempotency check ─────────────────────────────────────────────
ALREADY_BOOTSTRAPPED=$(jq -r '.decisions | to_entries | map(select(.value.history[0].verdict == "legacy-import")) | length' "$STATE_FILE")
if [[ "$ALREADY_BOOTSTRAPPED" -gt 0 ]] && [[ "${1:-}" != "--force" ]]; then
  echo "decisions.json already has $ALREADY_BOOTSTRAPPED legacy-import entries. Pass --force to re-bootstrap." >&2
  exit 0
fi

# ── Source check ─────────────────────────────────────────────────
if [[ ! -f "$LEGACY_LATEST" ]]; then
  echo "No legacy latest.json found at $LEGACY_LATEST. Skipping bootstrap." >&2
  exit 0
fi

RUN_DATE=$(date -I)
LEGACY_TIMESTAMP=$(jq -r '.timestamp // "unknown"' "$LEGACY_LATEST")

# ── Slugify helper ───────────────────────────────────────────────
slugify() {
  echo "$1" \
    | tr '[:upper:]' '[:lower:]' \
    | sed -E 's/[^a-z0-9]+/-/g; s/^-+|-+$//g' \
    | cut -c1-60
}

# ── Categorize by keyword ────────────────────────────────────────
categorize() {
  local text="$1"
  case "$text" in
    *hook*|*PreToolUse*|*PostToolUse*|*gate.sh*) echo "hooks" ;;
    *MCP*|*mcp*|*server*) echo "mcp" ;;
    *skill*|*SKILL*) echo "skills" ;;
    *agent*|*subagent*) echo "agents" ;;
    *settings*|*env*|*permission*) echo "settings" ;;
    *memory*|*compact*) echo "memory" ;;
    *command*|*slash*) echo "commands" ;;
    *) echo "general" ;;
  esac
}

# ── Build legacy-import entries ──────────────────────────────────
TMP_FILE=$(mktemp)
trap 'rm -f "$TMP_FILE"' EXIT
cp "$STATE_FILE" "$TMP_FILE"

IMPORTED=0
for priority in high medium low; do
  while IFS= read -r rec; do
    [[ -z "$rec" ]] && continue
    SLUG=$(slugify "$rec")
    [[ -z "$SLUG" ]] && continue
    SIGNAL_ID="legacy-${priority}-${SLUG}"
    AREA=$(categorize "$rec")

    # Skip if signal already in decisions.json
    EXISTS=$(jq --arg id "$SIGNAL_ID" '.decisions | has($id)' "$TMP_FILE")
    [[ "$EXISTS" == "true" ]] && continue

    # Append entry
    jq \
      --arg id "$SIGNAL_ID" \
      --arg first_seen "$LEGACY_TIMESTAMP" \
      --arg title "$rec" \
      --arg area "$AREA" \
      --arg priority "$priority" \
      --arg date "$RUN_DATE" \
      '.decisions[$id] = {
        first_seen: $first_seen,
        version: "legacy",
        title: $title,
        area: $area,
        official_source: null,
        research: {
          why: "Imported from pre-redesign /workflow:evolve run",
          community_usage: "N/A — legacy import",
          our_leverage: [],
          risk: "N/A",
          effort: "N/A"
        },
        history: [{
          date: $date,
          verdict: "legacy-import",
          rationale: ("Imported from latest.json " + $priority + "-priority recommendation"),
          ref: null
        }]
      }' "$TMP_FILE" > "${TMP_FILE}.new"
    mv "${TMP_FILE}.new" "$TMP_FILE"
    IMPORTED=$((IMPORTED + 1))
  done < <(jq -r --arg p "$priority" '.recommendations[$p] // [] | .[]' "$LEGACY_LATEST")
done

# ── Atomic write ─────────────────────────────────────────────────
mv "$TMP_FILE" "$STATE_FILE"
trap - EXIT

echo "Bootstrapped $IMPORTED entries into $STATE_FILE"
echo "Source: $LEGACY_LATEST (timestamp: $LEGACY_TIMESTAMP)"
