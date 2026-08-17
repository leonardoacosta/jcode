#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

usage() {
  cat <<'USAGE'
Usage: scripts/test-command-center.sh [--fixture-only]

Runs repository-local command-center acceptance orchestration. By default it
builds the SolidStart assets and starts an isolated Jcode daemon plus managed
Command Center host. It never uses the shared user daemon. Use --fixture-only
only to validate the versioned deterministic fixture.

Environment:
  JCODE_COMMAND_CENTER_JCODE_BIN   Optional Jcode binary for the isolated host.
                                   Defaults to target/selfdev/jcode.
  JCODE_COMMAND_CENTER_DAEMON_CMD  Optional custom isolated daemon command.
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
  if [[ -n ${ui_pid:-} ]]; then
    kill "$ui_pid" 2>/dev/null || true
    wait "$ui_pid" 2>/dev/null || true
  fi
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

if [[ -n ${JCODE_COMMAND_CENTER_DAEMON_CMD:-} ]]; then
  # shellcheck disable=SC2086 # Intentional: daemon command is operator supplied.
  $JCODE_COMMAND_CENTER_DAEMON_CMD &
  daemon_pid=$!
elif [[ -z ${JCODE_COMMAND_CENTER_BASE_URL:-} ]]; then
  jcode_bin=${JCODE_COMMAND_CENTER_JCODE_BIN:-$repo_root/target/selfdev/jcode}
  if [[ ! -x "$jcode_bin" ]]; then
    cat >&2 <<ERR
The isolated Command Center launcher requires an executable Jcode binary at:
  $jcode_bin
Build it with 'scripts/dev_cargo.sh build --profile selfdev -p jcode --bin jcode'
or set JCODE_COMMAND_CENTER_JCODE_BIN.
ERR
    exit 1
  fi

  pnpm --dir "$repo_root/apps/command-center" build
  asset_dir="$repo_root/apps/command-center/.output/public"
  if [[ ! -f "$asset_dir/index.html" ]]; then
    echo "Command Center build did not produce $asset_dir/index.html" >&2
    exit 1
  fi

  api_port=$(python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
  )
  ui_port=$(python3 - <<'PY'
import socket
with socket.socket() as sock:
    sock.bind(("127.0.0.1", 0))
    print(sock.getsockname()[1])
PY
  )
  mkdir -p "$socket_dir/runtime"
  chmod 700 "$socket_dir/runtime"
  export XDG_RUNTIME_DIR="$socket_dir/runtime"
  export JCODE_SOCKET="$XDG_RUNTIME_DIR/jcode.sock"
  export JCODE_COMMAND_CENTER_ENABLED=1
  export JCODE_COMMAND_CENTER_BIND_ADDR="127.0.0.1:$api_port"
  export JCODE_COMMAND_CENTER_API_BASE_URL="http://127.0.0.1:$api_port"
  export JCODE_COMMAND_CENTER_BASE_URL="http://127.0.0.1:$ui_port"
  export JCODE_COMMAND_CENTER_ALLOWED_ORIGINS="$JCODE_COMMAND_CENTER_BASE_URL"

  "$jcode_bin" serve \
    --provider ollama \
    --no-update \
    --no-selfdev \
    --socket "$JCODE_SOCKET" \
    --quiet &
  daemon_pid=$!

  JCODE_COMMAND_CENTER_UI_BIND="127.0.0.1:$ui_port" \
    JCODE_COMMAND_CENTER_API_URL="$JCODE_COMMAND_CENTER_API_BASE_URL" \
    JCODE_COMMAND_CENTER_PUBLIC_DIR="$asset_dir" \
    node "$repo_root/apps/command-center/server.mjs" &
  ui_pid=$!

  ready=false
  for _ in $(seq 1 120); do
    if node -e '
      const [url, needle] = process.argv.slice(1);
      fetch(url).then(async (response) => {
        const body = await response.text();
        process.exit(response.ok && body.includes(needle) ? 0 : 1);
      }).catch(() => process.exit(1));
    ' "$JCODE_COMMAND_CENTER_BASE_URL/" '<title>Jcode Command Center</title>'; then
      ready=true
      break
    fi
    if ! kill -0 "$daemon_pid" 2>/dev/null; then
      break
    fi
    sleep 0.25
  done
  if [[ "$ready" != true ]]; then
    echo "Isolated Command Center host did not become ready" >&2
    exit 1
  fi
fi

if [[ ! -f "$repo_root/apps/command-center/package.json" ]]; then
  echo "apps/command-center/package.json is required for acceptance tests" >&2
  exit 1
fi

pnpm --dir "$repo_root/apps/command-center" test:e2e -- --project repository-local
pnpm --dir "$repo_root/apps/command-center" test:e2e -- --project orca-unavailable

echo "repository-local command-center acceptance passed"
