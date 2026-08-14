#!/usr/bin/env python3
"""Backfill privacy-safe Jcode session aggregates into Loki.

The payload intentionally contains no prompts, tool inputs, tool outputs, or
transcript text. Each session becomes one structured Loki log line.
"""
from __future__ import annotations

import argparse
import glob
import json
import os
import urllib.request
from datetime import datetime


def ns(value: str) -> str:
    return str(int(datetime.fromisoformat(value.replace("Z", "+00:00")).timestamp() * 1_000_000_000))


def summarize(path: str) -> dict:
    with open(path, encoding="utf-8") as handle:
        session = json.load(handle)
    messages = session.get("messages") or []
    content_types: dict[str, int] = {}
    tool_calls = tool_results = 0
    token_messages = 0
    token_input = token_output = 0
    tool_duration_ms = 0
    for message in messages:
        usage = message.get("token_usage")
        if isinstance(usage, dict):
            token_messages += 1
            token_input += int(usage.get("input_tokens", usage.get("prompt_tokens", 0)) or 0)
            token_output += int(usage.get("output_tokens", usage.get("completion_tokens", 0)) or 0)
        tool_duration_ms += int(message.get("tool_duration_ms", 0) or 0)
        for block in message.get("content", []) if isinstance(message.get("content"), list) else []:
            kind = block.get("type") if isinstance(block, dict) else None
            if kind:
                content_types[kind] = content_types.get(kind, 0) + 1
                tool_calls += kind == "tool_use"
                tool_results += kind == "tool_result"
    created = session.get("created_at")
    updated = session.get("updated_at") or created
    start = datetime.fromisoformat(created.replace("Z", "+00:00"))
    end = datetime.fromisoformat(updated.replace("Z", "+00:00"))
    return {
        "event": "jcode_session_reconstructed",
        "source": "jcode-session-reconstruction",
        "session_id": session.get("id"),
        "provider": session.get("provider_key"),
        "model": session.get("model"),
        "version": _version(messages),
        "working_dir": session.get("working_dir"),
        "status": _status(session.get("status")),
        "created_at": created,
        "updated_at": updated,
        "duration_ms": max(0, int((end - start).total_seconds() * 1000)),
        "message_count": len(messages),
        "tool_calls": int(tool_calls),
        "tool_results": int(tool_results),
        "tool_duration_ms": tool_duration_ms,
        "token_usage_messages": token_messages,
        "input_tokens": token_input,
        "output_tokens": token_output,
        "content_types": content_types,
    }


def _version(messages: list) -> str | None:
    for message in messages:
        for block in message.get("content", []) if isinstance(message.get("content"), list) else []:
            if isinstance(block, dict) and block.get("type") == "text":
                text = block.get("text", "")
                marker = "Jcode version: "
                if marker in text:
                    return text.split(marker, 1)[1].splitlines()[0].strip()
    return None


def _status(value: object) -> str:
    if isinstance(value, dict) and "Crashed" in value:
        return "crashed"
    return str(value or "unknown").lower()


def push(endpoint: str, token: str | None, entries: list[dict], batch_size: int = 100) -> None:
    for offset in range(0, len(entries), batch_size):
        _push_batch(endpoint, token, entries[offset : offset + batch_size])


def _push_batch(endpoint: str, token: str | None, entries: list[dict]) -> None:
    streams: dict[tuple[tuple[str, str], ...], list[list[str]]] = {}
    for entry in entries:
        labels = {
            "service_name": "jcode",
            "source": "jcode-session-reconstruction",
            "provider": str(entry.get("provider") or "unknown"),
            "status": str(entry.get("status") or "unknown"),
        }
        key = tuple(sorted(labels.items()))
        streams.setdefault(key, []).append([ns(entry["created_at"]), json.dumps(entry, separators=(",", ":"))])
    payload = json.dumps({"streams": [{"stream": dict(key), "values": values} for key, values in streams.items()]}).encode()
    request = urllib.request.Request(endpoint, data=payload, method="POST", headers={"Content-Type": "application/json"})
    if token:
        request.add_header("Authorization", f"Bearer {token}")
    with urllib.request.urlopen(request, timeout=30) as response:
        if response.status >= 300:
            raise RuntimeError(f"Loki returned HTTP {response.status}")


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--sessions", default=os.path.expanduser("~/.jcode/sessions"))
    parser.add_argument("--endpoint", default=os.getenv("JCODE_GRAFANA_LOKI_URL", "http://127.0.0.1:3100/loki/api/v1/push"))
    parser.add_argument("--token", default=os.getenv("JCODE_GRAFANA_LOKI_TOKEN"))
    parser.add_argument("--limit", type=int)
    parser.add_argument("--batch-size", type=int, default=100)
    parser.add_argument("--dry-run", action="store_true")
    args = parser.parse_args()
    files = sorted(glob.glob(os.path.join(args.sessions, "session_*.json")))
    if args.limit:
        files = files[: args.limit]
    entries = [summarize(path) for path in files]
    print(json.dumps({"sessions": len(entries), "endpoint": args.endpoint, "dry_run": args.dry_run}, indent=2))
    if not args.dry_run and entries:
        push(args.endpoint, args.token, entries, args.batch_size)
        print(f"backfilled {len(entries)} sessions")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
