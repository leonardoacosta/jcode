#!/usr/bin/env bash
set -Eeuo pipefail
IFS=$'\n\t'

usage() {
  cat <<'USAGE'
Usage: scripts/test-command-center-mac-smoke.sh --mac-host HOST --jcode-host HOST [--remote-command CMD]

Terminal post gate for the managed Mac/homelab topology. This is intentionally
separate from repository-local deterministic gates. It requires explicit host
arguments and never guesses SSH aliases.
USAGE
}

mac_host=""
jcode_host=""
remote_command=""
while [[ $# -gt 0 ]]; do
  case "$1" in
    --mac-host) mac_host=${2:-}; shift 2 ;;
    --jcode-host) jcode_host=${2:-}; shift 2 ;;
    --remote-command) remote_command=${2:-}; shift 2 ;;
    --help|-h) usage; exit 0 ;;
    *) usage >&2; exit 64 ;;
  esac
done

if [[ -z "$mac_host" || -z "$jcode_host" ]]; then
  usage >&2
  exit 64
fi

if [[ -z "$remote_command" ]]; then
  remote_command='command -v jcode >/dev/null && jcode server start --json >/dev/null'
fi

ssh -o BatchMode=yes -o ConnectTimeout=10 "$jcode_host" "$remote_command"
ssh -o BatchMode=yes -o ConnectTimeout=10 "$mac_host" "ssh -o BatchMode=yes -o ConnectTimeout=10 $jcode_host 'echo command-center-topology-ok'"

ssh -o BatchMode=yes -o ConnectTimeout=10 "$mac_host" bash -s -- "$jcode_host" <<'REMOTE'
set -Eeuo pipefail

jcode_host=$1
remote_port=${JCODE_COMMAND_CENTER_REMOTE_PORT:-43118}
local_port=${JCODE_COMMAND_CENTER_LOCAL_PORT:-43118}
control=$(mktemp -u "$HOME/.ssh/command-center-smoke.XXXXXX")

cleanup() {
  ssh -S "$control" -O exit "$jcode_host" >/dev/null 2>&1 || true
  rm -f "$control"
}
trap cleanup EXIT

ssh -M -S "$control" -fN \
  -o BatchMode=yes \
  -o ConnectTimeout=10 \
  -o ExitOnForwardFailure=yes \
  -L "${local_port}:127.0.0.1:${remote_port}" \
  "$jcode_host"

url="http://127.0.0.1:${local_port}/initiatives"
for _ in $(seq 1 50); do
  if curl -fsS "$url" >/dev/null; then
    break
  fi
  sleep 0.1
done
curl -fsS "$url" >/dev/null

python3 - "$url" <<'PY'
import pathlib
import shutil
import subprocess
import sys
import tempfile

url = sys.argv[1]
chrome = pathlib.Path("/Applications/Google Chrome.app/Contents/MacOS/Google Chrome")
if not chrome.is_file():
    raise SystemExit(f"Google Chrome is required for the Mac smoke gate: {chrome}")

profile = tempfile.mkdtemp(prefix="jcode-command-center-smoke-")
command = [
    str(chrome),
    "--headless=new",
    "--disable-gpu",
    "--disable-background-networking",
    "--no-first-run",
    "--no-default-browser-check",
    "--virtual-time-budget=3000",
    f"--user-data-dir={profile}",
    "--dump-dom",
    url,
]
process = subprocess.Popen(command, stdout=subprocess.PIPE, stderr=subprocess.PIPE, text=True)
try:
    stdout, stderr = process.communicate(timeout=15)
except subprocess.TimeoutExpired:
    process.terminate()
    try:
        stdout, stderr = process.communicate(timeout=5)
    except subprocess.TimeoutExpired:
        process.kill()
        stdout, stderr = process.communicate()
finally:
    shutil.rmtree(profile, ignore_errors=True)

required = (
    "Jcode Command Center",
    "Initiatives",
    "Unify initiatives, schedules, messages, approvals, runs, and agent execution",
)
missing = [text for text in required if text not in stdout]
if missing:
    raise SystemExit(
        f"Mac Chrome did not render required Command Center content: {missing}; "
        f"bytes={len(stdout)} stderr={stderr[-500:]}"
    )

secret_markers = ("sk-", "provider_token", "provider-token", "api_key", "api-key")
for marker in secret_markers:
    if marker.lower() in stdout.lower():
        raise SystemExit(f"secret-like provider material appeared in browser DOM: {marker}")

print(f"managed Mac Chrome rendered Command Center through SSH tunnel ({len(stdout)} bytes)")
PY
REMOTE

echo "managed Mac/homelab command-center browser smoke passed"
