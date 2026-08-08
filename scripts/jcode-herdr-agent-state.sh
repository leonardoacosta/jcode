#!/bin/sh
# Jcode -> Herdr lifecycle hook adapter.
#
# Intended config:
#   [hooks]
#   session_start = ["/path/to/jcode-herdr-agent-state.sh session"]
#   session_end = ["/path/to/jcode-herdr-agent-state.sh session"]
#
# The adapter intentionally reports only durable session identity and normal
# release. It does not claim working/idle/blocked authority from turn/tool hooks
# because Jcode does not yet expose a complete blocked/approval/interrupt
# lifecycle that would let Herdr avoid stale state.

set -eu

action="${1:-session}"
case "$action" in
  session|hook) ;;
  *) exit 0 ;;
esac

[ "${HERDR_ENV:-}" = "1" ] || exit 0
[ -n "${HERDR_SOCKET_PATH:-}" ] || exit 0
[ -n "${HERDR_PANE_ID:-}" ] || exit 0
command -v python3 >/dev/null 2>&1 || exit 0

python3 - <<'PY'
import json
import os
import random
import socket
import time

SOURCE = "herdr:jcode"
AGENT = "jcode"

pane_id = os.environ.get("HERDR_PANE_ID")
socket_path = os.environ.get("HERDR_SOCKET_PATH")
event = os.environ.get("JCODE_HOOK_EVENT", "")
session_id = os.environ.get("JCODE_HOOK_SESSION_ID") or None
hook_source = os.environ.get("JCODE_HOOK_SOURCE") or None

if not pane_id or not socket_path:
    raise SystemExit(0)

try:
    payload_raw = os.environ.get("JCODE_HOOK_PAYLOAD") or "{}"
    payload = json.loads(payload_raw) if payload_raw.strip() else {}
except Exception:
    payload = {}

if not event:
    event = str(payload.get("event") or "")
if not session_id:
    candidate = payload.get("session_id")
    session_id = candidate if isinstance(candidate, str) and candidate else None
if not hook_source:
    candidate = payload.get("source")
    hook_source = candidate if isinstance(candidate, str) and candidate else None

seq = time.time_ns()
request_id = f"{SOURCE}:{seq}:{random.randrange(1_000_000):06d}"

request = None
if event == "session_start":
    if not session_id:
        raise SystemExit(0)
    params = {
        "pane_id": pane_id,
        "source": SOURCE,
        "agent": AGENT,
        "seq": seq,
        "agent_session_id": session_id,
    }
    if hook_source in {"create", "attach"}:
        params["session_start_source"] = "startup"
    elif hook_source == "resume":
        params["session_start_source"] = "resume"
    request = {
        "id": request_id,
        "method": "pane.report_agent_session",
        "params": params,
    }
elif event == "session_end":
    request = {
        "id": request_id,
        "method": "pane.release_agent",
        "params": {
            "pane_id": pane_id,
            "source": SOURCE,
            "agent": AGENT,
            "seq": seq,
        },
    }
else:
    # Explicit no-op for turn_start, turn_end, pre_tool, and post_tool. Those
    # hooks are useful for future integration, but should not claim Herdr state
    # authority until Jcode has complete blocked/approval/interrupt events.
    raise SystemExit(0)

try:
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    client.settimeout(0.5)
    client.connect(socket_path)
    client.sendall((json.dumps(request, separators=(",", ":")) + "\n").encode("utf-8"))
    try:
        client.recv(4096)
    except Exception:
        pass
    client.close()
except Exception:
    pass
PY
