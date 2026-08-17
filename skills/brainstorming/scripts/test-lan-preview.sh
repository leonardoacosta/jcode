#!/usr/bin/env bash
set -euo pipefail
ROOT="$(cd "$(dirname "$0")/../../.." && pwd)"
START="$ROOT/skills/brainstorming/scripts/start-server.sh"
# Contract checks intentionally fail until LAN-first serving is implemented.
grep -q 'BIND_HOST="0.0.0.0"' "$START"
grep -q 'LAN_HOST' "$START"
grep -q 'lan_url' "$ROOT/skills/brainstorming/scripts/server.cjs"
printf 'LAN preview contract passed\n'
