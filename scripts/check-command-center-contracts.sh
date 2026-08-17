#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

usage() {
  cat <<'USAGE'
Usage: scripts/check-command-center-contracts.sh [--fixture-only]

Regenerates command-center TypeScript contracts into an isolated temporary
folder and compares them with the checked-in generated client. This gate is
repository-local and deterministic. It does not prove the managed Mac/homelab
terminal post gate.

Environment:
  JCODE_COMMAND_CENTER_CONTRACT_GENERATOR  Generator command. It must accept an
                                          output directory as its final argument.
  JCODE_COMMAND_CENTER_GENERATED_DIR       Checked-in generated directory.
                                          Default: apps/command-center/src/generated
USAGE
}

repo_root=$(cd "$(dirname "${BASH_SOURCE[0]}")/.." && pwd)
fixture="$repo_root/fixtures/command-center/vertical-slice.fixture.json"
fixture_only=false

if [[ ${1:-} == "--help" || ${1:-} == "-h" ]]; then
  usage
  exit 0
elif [[ ${1:-} == "--fixture-only" ]]; then
  fixture_only=true
elif [[ $# -gt 0 ]]; then
  usage >&2
  exit 64
fi

require_file() {
  local path=$1
  if [[ ! -f "$path" ]]; then
    echo "missing required file: $path" >&2
    exit 1
  fi
}

require_file "$fixture"
python3 - <<'PY' "$fixture"
import json, pathlib, sys
path = pathlib.Path(sys.argv[1])
data = json.loads(path.read_text())
required = ["schemaVersion", "initiative", "linkedSchedule", "linkedRun", "events", "allowedRuntimeCommands"]
missing = [key for key in required if key not in data]
if missing:
    raise SystemExit(f"fixture missing keys: {missing}")
seq = [event["sequence"] for event in data["events"]]
if seq != list(range(1, len(seq) + 1)):
    raise SystemExit(f"fixture event sequence is not contiguous from 1: {seq}")
if set(data.get("allowedRuntimeCommands", [])) != {"start_initiative_run", "retry_linked_run", "cancel_linked_run"}:
    raise SystemExit("fixture runtime command set does not match approved closed set")
PY

if [[ "$fixture_only" == true ]]; then
  echo "fixture contract sanity check passed"
  exit 0
fi

generated_dir=${JCODE_COMMAND_CENTER_GENERATED_DIR:-"$repo_root/apps/command-center/src/generated"}
generator=${JCODE_COMMAND_CENTER_CONTRACT_GENERATOR:-}

if [[ ! -d "$generated_dir" ]]; then
  echo "checked-in generated directory does not exist: $generated_dir" >&2
  exit 1
fi

tmp_parent=${TMPDIR:-/tmp}
workdir=$(mktemp -d "$tmp_parent/jcode-command-center-contracts.XXXXXX")
cleanup() { rm -rf "$workdir"; }
trap cleanup EXIT

out_dir="$workdir/generated"
mkdir -p "$out_dir"
(
  cd "$repo_root"
  if [[ -n "$generator" ]]; then
    bash -c "$generator \"\$1\"" _ "$out_dir"
  else
    cargo run --quiet -p jcode-command-center --bin generate-command-center-types -- "$out_dir"
  fi
)

diff -ruN "$generated_dir" "$out_dir"
echo "command-center generated contracts are current"
