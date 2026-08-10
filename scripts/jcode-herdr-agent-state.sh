#!/bin/sh
# Jcode -> Herdr lifecycle hook adapter.
#
# Intended config:
#   [hooks]
#   session_start = ["/path/to/jcode-herdr-agent-state.sh session"]
#   turn_start = ["/path/to/jcode-herdr-agent-state.sh session"]
#   turn_end = ["/path/to/jcode-herdr-agent-state.sh session"]
#   session_end = ["/path/to/jcode-herdr-agent-state.sh session"]
#
# Herdr supports custom harnesses through semantic lifecycle reports. This
# adapter uses the custom source for visible agent-panel state while carrying
# the native Jcode session id when available.

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

SOURCE = "custom:jcode"
AGENT = "jcode"

pane_id = os.environ.get("HERDR_PANE_ID")
socket_path = os.environ.get("HERDR_SOCKET_PATH")
event = os.environ.get("JCODE_HOOK_EVENT", "")
session_id = os.environ.get("JCODE_HOOK_SESSION_ID") or None
hook_source = os.environ.get("JCODE_HOOK_SOURCE") or None
cwd = os.environ.get("JCODE_HOOK_CWD") or None
status = os.environ.get("JCODE_HOOK_STATUS") or None
model = os.environ.get("JCODE_HOOK_MODEL") or None
error = os.environ.get("JCODE_HOOK_ERROR") or None

if not pane_id or not socket_path:
    raise SystemExit(0)

try:
    payload_raw = os.environ.get("JCODE_HOOK_PAYLOAD") or "{}"
    payload = json.loads(payload_raw) if payload_raw.strip() else {}
except Exception:
    payload = {}

def count_field(*names):
    for name in names:
        value = os.environ.get(name)
        if value is None:
            value = payload.get(name.lower())
        try:
            return max(0, int(value))
        except (TypeError, ValueError):
            continue
    return 0

if not event:
    event = str(payload.get("event") or "")
if not session_id:
    candidate = payload.get("session_id")
    session_id = candidate if isinstance(candidate, str) and candidate else None
if not hook_source:
    candidate = payload.get("source")
    hook_source = candidate if isinstance(candidate, str) and candidate else None
if not status:
    candidate = payload.get("status")
    status = candidate if isinstance(candidate, str) and candidate else None
if not model:
    candidate = payload.get("model")
    model = candidate if isinstance(candidate, str) and candidate else None
if not error:
    candidate = payload.get("error")
    error = candidate if isinstance(candidate, str) and candidate else None

working_subagents = count_field("JCODE_HOOK_SUBAGENTS_WORKING", "subagents_working")
blocking_subagents = count_field(
    "JCODE_HOOK_SUBAGENTS_BLOCKING", "subagents_working_blocking", "subagents_blocking"
)
nonblocking_subagents = count_field(
    "JCODE_HOOK_SUBAGENTS_NON_BLOCKING",
    "JCODE_HOOK_SUBAGENTS_WORKING_NON_BLOCKING",
    "subagents_working_non_blocking",
    "subagents_non_blocking",
)
if working_subagents == 0:
    working_subagents = blocking_subagents + nonblocking_subagents

def working_message():
    if working_subagents <= 0:
        return f"jcode {model}" if model else "jcode working"
    parts = [f"{working_subagents} subagent{'s' if working_subagents != 1 else ''} working"]
    if blocking_subagents:
        parts.append(f"{blocking_subagents} blocking")
    if nonblocking_subagents:
        parts.append(f"{nonblocking_subagents} non-blocking")
    return "jcode " + " · ".join(parts)

seq = time.time_ns()
request_id = f"{SOURCE}:{seq}:{random.randrange(1_000_000):06d}"


def socket_request(request):
    client = socket.socket(socket.AF_UNIX, socket.SOCK_STREAM)
    client.settimeout(0.5)
    client.connect(socket_path)
    client.sendall((json.dumps(request, separators=(",", ":")) + "\n").encode("utf-8"))
    chunks = []
    try:
        while True:
            chunk = client.recv(65536)
            if not chunk:
                break
            chunks.append(chunk)
            if b"\n" in chunk:
                break
    except Exception:
        pass
    client.close()
    response_raw = b"".join(chunks).split(b"\n", 1)[0]
    if not response_raw:
        return None
    try:
        return json.loads(response_raw.decode("utf-8", "replace"))
    except Exception:
        return None


def pane_not_found(response):
    return isinstance(response, dict) and response.get("error", {}).get("code") == "pane_not_found"


def fallback_pane_for_cwd():
    if not cwd:
        return None
    response = socket_request({"id": f"{SOURCE}:snapshot:{time.time_ns()}", "method": "session.snapshot", "params": {}})
    snapshot = response.get("result", {}).get("snapshot") if isinstance(response, dict) else None
    panes = snapshot.get("panes", []) if isinstance(snapshot, dict) else []
    matches = []
    for pane in panes:
        if not isinstance(pane, dict):
            continue
        if pane.get("cwd") == cwd or pane.get("foreground_cwd") == cwd:
            candidate = pane.get("pane_id")
            if isinstance(candidate, str) and candidate:
                matches.append(candidate)
    unique = sorted(set(matches))
    return unique[0] if len(unique) == 1 else None


def report_agent(state, message=None):
    params = {
        "pane_id": pane_id,
        "source": SOURCE,
        "agent": AGENT,
        "seq": seq,
        "state": state,
    }
    if session_id:
        params["agent_session_id"] = session_id
    if message:
        params["message"] = message[:500]
    return {"id": request_id, "method": "pane.report_agent", "params": params}


request = None
if event == "session_start":
    if not session_id:
        raise SystemExit(0)
    request = report_agent("unknown", "jcode session active")
elif event == "turn_start":
    request = report_agent("working", working_message())
elif event == "turn_end":
    message = "jcode ready"
    if status == "error" and error:
        message = f"jcode turn ended with error: {error}"
    request = report_agent("idle", message)
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
    # Explicit no-op for pre_tool and post_tool. Tool boundaries are not whole
    # agent-state transitions.
    raise SystemExit(0)

try:
    response = socket_request(request)
    if pane_not_found(response):
        fallback_pane = fallback_pane_for_cwd()
        if fallback_pane:
            request["params"]["pane_id"] = fallback_pane
            socket_request(request)
except Exception:
    pass
PY
