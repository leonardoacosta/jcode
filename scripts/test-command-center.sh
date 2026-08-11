#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

usage() {
  cat <<'USAGE'
Usage: scripts/test-command-center.sh [--fixture-only]

Runs repository-local command-center acceptance orchestration. The default mode
requires the implemented app test commands and an isolated daemon command. It
never uses the shared user daemon. Use --fixture-only only to validate the
versioned deterministic fixture before implementation lands.

Environment:
  JCODE_COMMAND_CENTER_DAEMON_CMD  Required default-mode daemon command.
  JCODE_COMMAND_CENTER_BASE_URL    Optional prestarted isolated base URL.
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

runtime_dir=${XDG_RUNTIME_DIR:-${TMPDIR:-/tmp}}
socket_dir=$(mktemp -d "$runtime_dir/jcode-command-center.XXXXXX")
cleanup() {
  if [[ -n ${daemon_pid:-} ]]; then
    kill "$daemon_pid" 2>/dev/null || true
    wait "$daemon_pid" 2>/dev/null || true
  fi
  rm -rf "$socket_dir"
}
trap cleanup EXIT
export JCODE_SOCKET="$socket_dir/jcode.sock"
export JCODE_HOME="$socket_dir/home"
mkdir -p "$JCODE_HOME"

bash "$repo_root/scripts/check-command-center-contracts.sh" --fixture-only
python3 - <<'PY' "$repo_root/fixtures/command-center/vertical-slice.fixture.json"
import json, pathlib, sys
fixture = json.loads(pathlib.Path(sys.argv[1]).read_text())
for name in fixture["forbiddenRuntimeCommands"]:
    if name in fixture["allowedRuntimeCommands"]:
        raise SystemExit(f"forbidden command also allowed: {name}")
if not fixture["linkedRun"].get("orcaProjectId", "").startswith("orca-project-"):
    raise SystemExit("fixture does not preserve canonical Orca project identity")
PY

if [[ "$fixture_only" == true ]]; then
  echo "command-center deterministic fixture acceptance sanity passed"
  exit 0
fi

if [[ -z ${JCODE_COMMAND_CENTER_DAEMON_CMD:-} && -z ${JCODE_COMMAND_CENTER_BASE_URL:-} ]]; then
  cat >&2 <<'ERR'
No isolated command-center daemon was supplied.
Failing closed. Set JCODE_COMMAND_CENTER_DAEMON_CMD to a noninteractive command
that starts an isolated daemon/web host using JCODE_SOCKET/JCODE_HOME, or set
JCODE_COMMAND_CENTER_BASE_URL to a prestarted isolated instance.
ERR
  exit 1
fi

if [[ -n ${JCODE_COMMAND_CENTER_DAEMON_CMD:-} ]]; then
  # shellcheck disable=SC2086 # Intentional: daemon command is operator supplied.
  $JCODE_COMMAND_CENTER_DAEMON_CMD &
  daemon_pid=$!
fi

if [[ ! -f "$repo_root/apps/command-center/package.json" ]]; then
  echo "apps/command-center/package.json is required for acceptance tests" >&2
  exit 1
fi

pnpm --dir "$repo_root/apps/command-center" test:e2e -- --project repository-local
pnpm --dir "$repo_root/apps/command-center" test:e2e -- --project orca-unavailable

echo "repository-local command-center acceptance passed"
