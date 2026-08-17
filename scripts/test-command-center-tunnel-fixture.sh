#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

usage() {
  cat <<'USAGE'
Usage: scripts/test-command-center-tunnel-fixture.sh [--fixture-only]

Validates repository-local tunnel/bridge invariants using deterministic fixture
checks. Default mode also probes a supplied forwarded endpoint and verifies that
no direct non-loopback command-center URL is accepted by the harness.

Environment:
  JCODE_COMMAND_CENTER_FORWARDED_URL  Authenticated forwarded endpoint.
  JCODE_COMMAND_CENTER_DIRECT_URL     Optional direct endpoint. If it returns 2xx,
                                      the gate fails.
USAGE
}

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fixture_only=false
case ${1:-} in
  --fixture-only) fixture_only=true ;;
  --help|-h) usage; exit 0 ;;
  "") ;;
  *) usage >&2; exit 64 ;;
esac

bash "$repo_root/scripts/check-command-center-contracts.sh" --fixture-only
python3 - <<'PY' "$repo_root/fixtures/command-center/vertical-slice.fixture.json"
import json, pathlib, sys
fixture = json.loads(pathlib.Path(sys.argv[1]).read_text())
stream_ids = {event["streamId"] for event in fixture["events"]}
if len(stream_ids) != 1:
    raise SystemExit("fixture must be scoped to exactly one authorization stream")
if any("/home/" in json.dumps(event) for event in fixture["events"]):
    raise SystemExit("fixture events expose host-local execution paths")
PY

if [[ "$fixture_only" == true ]]; then
  echo "command-center tunnel fixture sanity passed"
  exit 0
fi

forwarded=${JCODE_COMMAND_CENTER_FORWARDED_URL:-}
if [[ -z "$forwarded" ]]; then
  echo "JCODE_COMMAND_CENTER_FORWARDED_URL is required in default tunnel mode" >&2
  exit 1
fi

if [[ -n ${JCODE_COMMAND_CENTER_DIRECT_URL:-} ]]; then
  status=$(curl -fsS -o /dev/null -w '%{http_code}' "$JCODE_COMMAND_CENTER_DIRECT_URL" || true)
  if [[ "$status" =~ ^2 ]]; then
    echo "direct non-forwarded endpoint was reachable" >&2
    exit 1
  fi
fi

status=$(curl -fsS -o /dev/null -w '%{http_code}' "$forwarded" || true)
if [[ ! "$status" =~ ^(2|3|4) ]]; then
  echo "forwarded endpoint did not respond with an HTTP status: $status" >&2
  exit 1
fi

echo "command-center tunnel gate passed"
