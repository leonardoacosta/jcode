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

echo "managed Mac/homelab command-center smoke preconditions passed"
