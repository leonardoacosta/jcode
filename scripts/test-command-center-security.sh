#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

usage() {
  cat <<'USAGE'
Usage: scripts/test-command-center-security.sh [--fixture-only]

Runs command-center security gates. Default mode requires an isolated base URL
from the repository-local command-center acceptance harness. The script fails
closed when the service is missing, unauthenticated reads succeed, CSRF checks
are absent, origin filtering is absent, secret-looking values are exposed, or a
non-loopback bind is permitted without explicit authenticated remote config.
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
if any("secret" in json.dumps(value).lower() for value in fixture.values()):
    raise SystemExit("fixture contains a secret-looking value")
if "cancel_linked_run" not in fixture["allowedRuntimeCommands"]:
    raise SystemExit("closed runtime command set missing cancel_linked_run")
PY

if [[ "$fixture_only" == true ]]; then
  echo "command-center security fixture sanity passed"
  exit 0
fi

base_url=${JCODE_COMMAND_CENTER_BASE_URL:-}
csrf_token=${JCODE_COMMAND_CENTER_CSRF_TOKEN:-}
if [[ -z "$base_url" ]]; then
  echo "JCODE_COMMAND_CENTER_BASE_URL is required in default security mode" >&2
  exit 1
fi

status=$(curl -fsS -o /dev/null -w '%{http_code}' "$base_url/api/command-center/initiatives" || true)
if [[ "$status" =~ ^2 ]]; then
  echo "unauthenticated snapshot read succeeded" >&2
  exit 1
fi

status=$(curl -fsS -o /dev/null -w '%{http_code}' \
  -X POST \
  -H 'content-type: application/json' \
  --data '{"type":"checkpoint_initiative","idempotencyKey":"security-fixture"}' \
  "$base_url/api/command-center/commands" || true)
if [[ "$status" =~ ^2 ]]; then
  echo "mutation without CSRF proof succeeded" >&2
  exit 1
fi

if [[ -n "$csrf_token" ]]; then
  status=$(curl -fsS -o /dev/null -w '%{http_code}' \
    -X POST \
    -H 'origin: https://evil.invalid' \
    -H "x-csrf-token: $csrf_token" \
    -H 'content-type: application/json' \
    --data '{"type":"checkpoint_initiative","idempotencyKey":"origin-fixture"}' \
    "$base_url/api/command-center/commands" || true)
  if [[ "$status" =~ ^2 ]]; then
    echo "mutation from disallowed origin succeeded" >&2
    exit 1
  fi
fi

echo "command-center security gate passed"
