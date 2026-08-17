#!/usr/bin/env python3
"""Accept a rendered system-map artifact through Chromium and DevTools Protocol.

The verifier deliberately uses only Python's standard library.  It launches an
isolated Chromium instance, speaks the small WebSocket subset needed by CDP,
and exercises both the file origin and a loopback HTTP origin.
"""

from __future__ import annotations

import argparse
import base64
import contextlib
import hashlib
import http.server
import json
import os
import pathlib
import re
import select
import shutil
import socket
import socketserver
import subprocess
import sys
import tempfile
import time
import urllib.parse
import urllib.request
import uuid
from typing import Any


class VerificationError(RuntimeError):
    """A user-actionable acceptance failure."""


class WebSocket:
    """Tiny synchronous RFC 6455 client sufficient for a local CDP endpoint."""

    def __init__(self, url: str, timeout: float = 12.0) -> None:
        parsed = urllib.parse.urlsplit(url)
        if parsed.scheme != "ws" or not parsed.hostname or not parsed.port:
            raise VerificationError(f"invalid CDP WebSocket URL: {url}")
        self.sock = socket.create_connection((parsed.hostname, parsed.port), timeout=timeout)
        self.sock.settimeout(timeout)
        key = base64.b64encode(os.urandom(16)).decode("ascii")
        path = parsed.path or "/"
        if parsed.query:
            path += "?" + parsed.query
        request = (
            f"GET {path} HTTP/1.1\r\n"
            f"Host: {parsed.hostname}:{parsed.port}\r\n"
            "Upgrade: websocket\r\n"
            "Connection: Upgrade\r\n"
            f"Sec-WebSocket-Key: {key}\r\n"
            "Sec-WebSocket-Version: 13\r\n\r\n"
        ).encode("ascii")
        self.sock.sendall(request)
        response = self._read_until(b"\r\n\r\n")
        if not response.startswith(b"HTTP/1.1 101"):
            raise VerificationError("Chromium CDP WebSocket handshake failed")
        self.closed = False

    def _read_until(self, marker: bytes) -> bytes:
        data = bytearray()
        while marker not in data:
            chunk = self.sock.recv(4096)
            if not chunk:
                raise VerificationError("Chromium closed the CDP connection")
            data.extend(chunk)
            if len(data) > 64 * 1024:
                raise VerificationError("CDP handshake response is unexpectedly large")
        return bytes(data)

    def _read_exact(self, size: int) -> bytes:
        data = bytearray()
        while len(data) < size:
            chunk = self.sock.recv(size - len(data))
            if not chunk:
                raise VerificationError("Chromium closed the CDP connection")
            data.extend(chunk)
        return bytes(data)

    def send(self, payload: str) -> None:
        raw = payload.encode("utf-8")
        length = len(raw)
        if length < 126:
            header = bytes((0x81, 0x80 | length))
        elif length < 65536:
            header = bytes((0x81, 0x80 | 126)) + length.to_bytes(2, "big")
        else:
            header = bytes((0x81, 0x80 | 127)) + length.to_bytes(8, "big")
        mask = os.urandom(4)
        masked = bytes(value ^ mask[index % 4] for index, value in enumerate(raw))
        self.sock.sendall(header + mask + masked)

    def receive(self) -> tuple[int, bytes]:
        first, second = self._read_exact(2)
        opcode = first & 0x0F
        masked = bool(second & 0x80)
        length = second & 0x7F
        if length == 126:
            length = int.from_bytes(self._read_exact(2), "big")
        elif length == 127:
            length = int.from_bytes(self._read_exact(8), "big")
        mask = self._read_exact(4) if masked else b""
        payload = self._read_exact(length)
        if mask:
            payload = bytes(value ^ mask[index % 4] for index, value in enumerate(payload))
        if opcode == 0x9:
            self._send_control(0xA, payload)
        elif opcode == 0x8:
            self.closed = True
        return opcode, payload

    def _send_control(self, opcode: int, payload: bytes) -> None:
        if len(payload) >= 126:
            raise VerificationError("unexpectedly large CDP control frame")
        mask = os.urandom(4)
        masked = bytes(value ^ mask[index % 4] for index, value in enumerate(payload))
        self.sock.sendall(bytes((0x80 | opcode, 0x80 | len(payload))) + mask + masked)

    def close(self) -> None:
        if not self.closed:
            with contextlib.suppress(OSError):
                self._send_control(0x8, b"")
        with contextlib.suppress(OSError):
            self.sock.close()
        self.closed = True


class CDP:
    """Synchronous CDP command/event adapter."""

    def __init__(self, websocket_url: str) -> None:
        self.ws = WebSocket(websocket_url)
        self.next_id = 0
        self.events: list[dict[str, Any]] = []

    def call(self, method: str, params: dict[str, Any] | None = None, timeout: float = 15.0) -> Any:
        self.next_id += 1
        command_id = self.next_id
        self.ws.send(json.dumps({"id": command_id, "method": method, "params": params or {}}))
        deadline = time.monotonic() + timeout
        while time.monotonic() < deadline:
            opcode, payload = self.ws.receive()
            if opcode != 0x1:
                continue
            message = json.loads(payload.decode("utf-8"))
            if message.get("id") != command_id:
                if "method" in message:
                    self.events.append(message)
                continue
            if "error" in message:
                error = message["error"]
                raise VerificationError(f"CDP {method} failed: {error.get('message', error)}")
            return message.get("result")
        raise VerificationError(f"timed out waiting for CDP {method}")

    def drain(self, seconds: float = 0.2) -> None:
        deadline = time.monotonic() + seconds
        while time.monotonic() < deadline:
            ready, _, _ = select.select([self.ws.sock], [], [], max(0, deadline - time.monotonic()))
            if not ready:
                return
            opcode, payload = self.ws.receive()
            if opcode == 0x1:
                message = json.loads(payload.decode("utf-8"))
                if "method" in message:
                    self.events.append(message)

    def events_since(self, start: int, method: str | None = None) -> list[dict[str, Any]]:
        events = self.events[start:]
        return [event for event in events if method is None or event.get("method") == method]

    def evaluate(self, expression: str, timeout: float = 15.0) -> Any:
        result = self.call(
            "Runtime.evaluate",
            {"expression": expression, "returnByValue": True, "awaitPromise": True},
            timeout,
        )
        remote = result.get("result", {}) if isinstance(result, dict) else {}
        if remote.get("subtype") == "error" or remote.get("type") == "undefined":
            raise VerificationError(f"browser evaluation failed for {expression[:100]}")
        if "value" not in remote:
            raise VerificationError(f"browser evaluation returned no value for {expression[:100]}")
        return remote["value"]

    def close(self) -> None:
        self.ws.close()


class QuietHandler(http.server.SimpleHTTPRequestHandler):
    def log_message(self, format: str, *args: Any) -> None:
        return

    def do_GET(self) -> None:
        if urllib.parse.urlsplit(self.path).path == "/favicon.ico":
            self.send_response(204)
            self.end_headers()
            return
        super().do_GET()


class LoopbackServer:
    def __init__(self, directory: pathlib.Path) -> None:
        handler = lambda *args, **kwargs: QuietHandler(*args, directory=str(directory), **kwargs)
        self.server = socketserver.ThreadingTCPServer(("127.0.0.1", 0), handler)
        self.server.daemon_threads = True
        self.thread = None

    @property
    def port(self) -> int:
        return int(self.server.server_address[1])

    def start(self) -> None:
        import threading

        self.thread = threading.Thread(target=self.server.serve_forever, daemon=True)
        self.thread.start()

    def close(self) -> None:
        self.server.shutdown()
        self.server.server_close()
        if self.thread is not None:
            self.thread.join(timeout=2)


class Browser:
    def __init__(self, executable: pathlib.Path) -> None:
        self.executable = executable
        self.temp_dir = pathlib.Path(tempfile.mkdtemp(prefix="verify-views-browser-"))
        self.process: subprocess.Popen[str] | None = None
        self.cdp: CDP | None = None

    def start(self) -> None:
        self.process = subprocess.Popen(
            [
                str(self.executable),
                "--headless=new",
                "--no-sandbox",
                "--disable-gpu",
                "--disable-dev-shm-usage",
                "--disable-extensions",
                "--disable-background-networking",
                "--disable-component-update",
                "--disable-sync",
                "--no-first-run",
                "--no-default-browser-check",
                "--remote-debugging-address=127.0.0.1",
                "--remote-debugging-port=0",
                f"--user-data-dir={self.temp_dir}",
                "about:blank",
            ],
            stdout=subprocess.DEVNULL,
            stderr=subprocess.PIPE,
            text=True,
        )
        active_port = self.temp_dir / "DevToolsActivePort"
        deadline = time.monotonic() + 15
        while time.monotonic() < deadline and not active_port.exists():
            if self.process.poll() is not None:
                detail = self.process.stderr.read() if self.process.stderr else ""
                raise VerificationError(f"Chromium exited during startup: {detail.strip()}")
            time.sleep(0.05)
        if not active_port.exists():
            raise VerificationError("Chromium did not publish DevToolsActivePort")
        lines = active_port.read_text(encoding="utf-8").splitlines()
        if not lines or not lines[0].isdigit():
            raise VerificationError("Chromium published an invalid DevTools port")
        port = int(lines[0])
        deadline = time.monotonic() + 15
        targets: list[dict[str, Any]] = []
        while time.monotonic() < deadline:
            try:
                with urllib.request.urlopen(f"http://127.0.0.1:{port}/json/list", timeout=2) as response:
                    targets = json.load(response)
                break
            except (OSError, ValueError):
                time.sleep(0.05)
        page = next((target for target in targets if target.get("type") == "page"), None)
        if not page or not page.get("webSocketDebuggerUrl"):
            raise VerificationError("Chromium did not expose a page target")
        self.cdp = CDP(str(page["webSocketDebuggerUrl"]))
        for method in ("Runtime.enable", "Page.enable", "Network.enable", "DOM.enable", "Log.enable"):
            self.cdp.call(method)
        self.cdp.call("Network.setCacheDisabled", {"cacheDisabled": True})
        self.cdp.call("Network.setBypassServiceWorker", {"bypass": True})

    def close(self) -> None:
        if self.cdp is not None:
            self.cdp.close()
            self.cdp = None
        if self.process is not None:
            with contextlib.suppress(Exception):
                self.process.terminate()
            with contextlib.suppress(Exception):
                self.process.wait(timeout=3)
            if self.process.poll() is None:
                with contextlib.suppress(Exception):
                    self.process.kill()
            self.process = None
        shutil.rmtree(self.temp_dir, ignore_errors=True)


def _require(condition: bool, message: str) -> None:
    if not condition:
        raise VerificationError(message)


def _wait_ready(cdp: CDP) -> None:
    deadline = time.monotonic() + 15
    while time.monotonic() < deadline:
        try:
            if cdp.evaluate("document.readyState") == "complete":
                cdp.drain(0.25)
                return
        except VerificationError:
            pass
        time.sleep(0.1)
    raise VerificationError("document did not reach readyState=complete")


def _navigate(cdp: CDP, url: str) -> int:
    start = len(cdp.events)
    cdp.call("Page.navigate", {"url": url})
    _wait_ready(cdp)
    return start


def _set_viewport(cdp: CDP, width: int, height: int, scale: float = 1) -> None:
    cdp.call(
        "Emulation.setDeviceMetricsOverride",
        {
            "width": width,
            "height": height,
            "deviceScaleFactor": 1,
            "mobile": False,
            "screenWidth": width,
            "screenHeight": height,
        },
    )
    cdp.call("Emulation.setPageScaleFactor", {"pageScaleFactor": scale})
    time.sleep(0.15)


def _clear_emulation(cdp: CDP) -> None:
    with contextlib.suppress(VerificationError):
        cdp.call("Emulation.setEmulatedMedia", {"features": []})
    with contextlib.suppress(VerificationError):
        cdp.call("Emulation.setPageScaleFactor", {"pageScaleFactor": 1})


def _errors_and_requests(cdp: CDP, event_start: int, origin: str, artifact_url: str) -> tuple[list[str], list[str]]:
    errors: list[str] = []
    requests: list[str] = []
    for event in cdp.events_since(event_start):
        method = event.get("method")
        params = event.get("params", {})
        if method == "Runtime.consoleAPICalled" and params.get("type") in {"error", "assert"}:
            args = params.get("args", [])
            text = " ".join(str(arg.get("value", arg.get("description", ""))) for arg in args)
            errors.append(f"console {params.get('type')}: {text}".strip())
        elif method == "Runtime.exceptionThrown":
            details = params.get("exceptionDetails", {})
            errors.append(str(details.get("text") or details.get("exception", {}).get("description", "page exception")))
        elif method == "Log.entryAdded" and params.get("entry", {}).get("level") in {"error", "assert"}:
            errors.append(str(params.get("entry", {}).get("text", "page log error")))
        elif method == "Network.requestWillBeSent":
            url = str(params.get("request", {}).get("url", ""))
            if url and not _allowed_request(url, origin, artifact_url):
                requests.append(url)
    return errors, requests


def _allowed_request(url: str, origin: str, artifact_url: str) -> bool:
    if url in {artifact_url, "about:blank"}:
        return True
    if url.startswith("data:") or url.startswith("blob:") or url.startswith("devtools://"):
        return True
    parsed = urllib.parse.urlsplit(url)
    expected = urllib.parse.urlsplit(origin)
    if parsed.scheme != expected.scheme or parsed.hostname != expected.hostname or parsed.port != expected.port:
        return False
    if parsed.scheme == "file":
        return parsed.path == expected.path
    return parsed.path == "/favicon.ico" and not parsed.query and not parsed.fragment


def _outer_html_without_javascript(cdp: CDP) -> str:
    document = cdp.call("DOM.getDocument", {"depth": 0})
    node_id = document.get("root", {}).get("nodeId")
    _require(isinstance(node_id, int), "CDP did not return a document node for no-JS verification")
    result = cdp.call("DOM.getOuterHTML", {"nodeId": node_id})
    html = result.get("outerHTML") if isinstance(result, dict) else None
    _require(isinstance(html, str), "CDP did not return document HTML for no-JS verification")
    return html


def _check_no_js_order(cdp: CDP, url: str) -> None:
    cdp.call("Emulation.setScriptExecutionDisabled", {"value": True})
    try:
        _navigate(cdp, url)
        html = _outer_html_without_javascript(cdp)
        positions = [html.find(f'id="{view}"') for view in ("runtime", "network", "ado")]
        _require(all(position >= 0 for position in positions), "no-JS document is missing a Runtime, Network, or ADO panel")
        _require(positions == sorted(positions), "no-JS document order is not Runtime, Network, ADO")
        _require("role=\"tabpanel\"" in html, "no-JS document lacks tabpanel semantics")
    finally:
        cdp.call("Emulation.setScriptExecutionDisabled", {"value": False})


def _check_fragments(cdp: CDP) -> None:
    checks = cdp.evaluate(
        """(() => ({
          runtime: Boolean(document.querySelector('#runtime')) && /Runtime/.test(document.querySelector('#runtime').textContent),
          network: Boolean(document.querySelector('#network .network-diagram')) && Boolean(document.querySelector('#network [data-network-connectors]')),
          ado: Boolean(document.querySelector('#ado .ado-pipeline')) && Boolean(document.querySelector('#ado .ado-stage-card')),
          panels: ['runtime', 'network', 'ado'].every(id => document.getElementById(id)?.getAttribute('role') === 'tabpanel')
        }))()"""
    )
    _require(checks.get("runtime"), "Runtime fragment is missing or empty")
    _require(checks.get("network"), "Network fragment is missing its measured diagram")
    _require(checks.get("ado"), "ADO fragment is missing its pipeline")
    _require(checks.get("panels"), "view panels do not expose tabpanel semantics")


def _check_direct_fragment_urls(cdp: CDP, base_url: str) -> None:
    base = base_url.split("#", 1)[0]
    for view_id, label in (("runtime", "Runtime"), ("network", "Network"), ("ado", "ADO")):
        _navigate(cdp, f"{base}#{view_id}")
        state = cdp.evaluate(
            f"""(() => {{
              const panel = document.getElementById({json.dumps(view_id)});
              const tab = document.getElementById({json.dumps(f'tab-{view_id}')});
              return {{
                hash: location.hash,
                panelVisible: Boolean(panel) && !panel.hidden,
                selected: tab?.getAttribute('aria-selected') === 'true'
              }};
            }})()"""
        )
        _require(state.get("hash") == f"#{view_id}", f"{label} fragment did not remain in the URL")
        _require(state.get("panelVisible"), f"{label} fragment did not reveal its panel")
        _require(state.get("selected"), f"{label} fragment did not select its native tab")


def _check_semantics(cdp: CDP) -> None:
    result = cdp.evaluate(
        """(() => {
          const levels = [...document.querySelectorAll('[data-evidence-level]')].map(node => node.dataset.evidenceLevel);
          return {levels, hasDirect: levels.includes('direct'), hasInferred: levels.includes('inferred'), hasHeld: levels.includes('held')};
        })()"""
    )
    levels = result.get("levels", [])
    _require(result.get("hasDirect"), "direct semantics are absent from view markup")
    _require(result.get("hasInferred"), "inferred semantics are absent from view markup")
    _require(result.get("hasHeld"), "held semantics are absent from view markup")
    _require(all(level in {"direct", "inferred", "held"} for level in levels), f"unknown evidence semantics: {levels}")


def _check_keyboard_and_focus(cdp: CDP) -> None:
    cdp.evaluate("document.getElementById('tab-runtime').focus(); true")
    cdp.call("Input.dispatchKeyEvent", {"type": "keyDown", "key": "ArrowRight", "code": "ArrowRight"})
    cdp.call("Input.dispatchKeyEvent", {"type": "keyUp", "key": "ArrowRight", "code": "ArrowRight"})
    active = cdp.evaluate("document.activeElement && document.activeElement.id")
    _require(active == "tab-network", f"native keyboard tab behavior focused {active!r}, expected tab-network")
    cdp.call("Input.dispatchKeyEvent", {"type": "keyDown", "key": "Enter", "code": "Enter", "windowsVirtualKeyCode": 13})
    cdp.call("Input.dispatchKeyEvent", {"type": "keyUp", "key": "Enter", "code": "Enter", "windowsVirtualKeyCode": 13})
    _require(cdp.evaluate("!document.getElementById('network').hidden"), "native keyboard activation did not reveal Network")
    focus = cdp.evaluate(
        """(() => {
          const candidates = [...document.querySelectorAll('button, [tabindex="0"]')];
          return {count: candidates.length, visible: candidates.filter(node => !node.hidden && node.offsetParent !== null).length};
        })()"""
    )
    _require(focus.get("count", 0) >= 3 and focus.get("visible", 0) >= 3, "focusability check found too few keyboard targets")


def _check_layout(cdp: CDP, width: int, height: int, label: str) -> None:
    metrics = cdp.evaluate(
        """(() => ({
          innerWidth: window.innerWidth,
          innerHeight: window.innerHeight,
          clientWidth: document.documentElement.clientWidth,
          scrollWidth: Math.max(document.documentElement.scrollWidth, document.body.scrollWidth),
          scrollHeight: Math.max(document.documentElement.scrollHeight, document.body.scrollHeight)
        }))()"""
    )
    _require(abs(metrics["innerWidth"] - width) <= 1, f"{label} viewport width is {metrics['innerWidth']}, expected {width}")
    _require(metrics["scrollWidth"] <= metrics["clientWidth"] + 1, f"{label} viewport has horizontal overflow")
    _require(metrics["scrollHeight"] >= height, f"{label} viewport did not produce a measurable document")


def _check_zoom_and_motion(cdp: CDP) -> None:
    cdp.call("Emulation.setPageScaleFactor", {"pageScaleFactor": 2})
    zoom = cdp.evaluate("window.visualViewport ? window.visualViewport.scale : 1")
    _require(float(zoom) >= 1.9, f"200% zoom was not applied (visual viewport scale {zoom})")
    cdp.call("Emulation.setEmulatedMedia", {"features": [{"name": "prefers-reduced-motion", "value": "reduce"}]})
    motion = None
    deadline = time.monotonic() + 3
    while time.monotonic() < deadline:
        motion = cdp.evaluate(
            """(() => {
              const debug = window.__ISO_MAP_DEBUG__;
              return {
                media: window.matchMedia('(prefers-reduced-motion: reduce)').matches,
                debug: Boolean(debug && debug.reducedMotion),
                paused: Boolean(debug && debug.paused),
                rafActive: debug ? Boolean(debug.rafActive) : true
              };
            })()"""
        )
        if motion.get("media") and motion.get("debug") and not motion.get("rafActive"):
            break
        time.sleep(0.05)
    _require(isinstance(motion, dict), "reduced motion state was not readable from the page")
    _require(motion.get("media") and motion.get("debug"), "reduced motion emulation was not observed by the page")
    _require(not motion.get("rafActive"), "reduced motion did not stop animation")


def _check_selection_retention(cdp: CDP) -> None:
    selection = cdp.evaluate(
        """(() => {
          const semanticNode = [...document.querySelectorAll('[data-semantic-id]')]
            .find(candidate => candidate.textContent?.trim());
          const node = semanticNode || document.querySelector('#network .network-relationship title, #network li');
          if (!node || !window.getSelection) return {supported: false, text: ''};
          const range = document.createRange();
          range.selectNodeContents(node);
          const current = window.getSelection();
          current.removeAllRanges();
          current.addRange(range);
          return {
            supported: true,
            text: current.toString(),
            semanticId: node.closest('[data-semantic-id]')?.getAttribute('data-semantic-id') || null,
          };
        })()"""
    )
    if selection.get("supported"):
        retained = cdp.evaluate(
            """(() => {
              const current = window.getSelection();
              const anchor = current?.anchorNode?.parentNode?.closest?.('[data-semantic-id]')
                || [...document.querySelectorAll('[data-semantic-id]')].find(candidate => candidate.textContent?.includes(current?.toString() || ''));
              return {
                text: current?.toString() || '',
                semanticId: anchor?.getAttribute('data-semantic-id') || null,
              };
            })()"""
        )
        _require(retained.get("text") == selection.get("text"), "selection retention failed while the page supported text selection")
        if selection.get("semanticId") is not None and retained.get("semanticId") is not None:
            _require(
                retained.get("semanticId") == selection.get("semanticId"),
                "semantic selection retention failed while data-semantic-id support was present",
            )


def _check_debug_snapshot(cdp: CDP) -> None:
    snapshot = cdp.evaluate("window.__ISO_MAP_DEBUG__")
    _require(isinstance(snapshot, dict), "interactive map did not expose its render snapshot")
    _require(snapshot.get("layerCount") == 3, "interactive map did not render all three canvas layers")
    _require(snapshot.get("nodeCount", 0) > 0 and snapshot.get("pathCount", 0) > 0, "interactive map has no rendered nodes or paths")


def _verify_origin(cdp: CDP, url: str, origin_label: str, artifact_url: str) -> list[str]:
    event_start = _navigate(cdp, url)
    _clear_emulation(cdp)
    _check_fragments(cdp)
    _check_debug_snapshot(cdp)
    _check_semantics(cdp)
    _check_direct_fragment_urls(cdp, url)
    _check_keyboard_and_focus(cdp)
    _set_viewport(cdp, 1440, 900)
    _check_layout(cdp, 1440, 900, "desktop viewport")
    _set_viewport(cdp, 320, 720)
    _check_layout(cdp, 320, 720, "320px viewport")
    _check_zoom_and_motion(cdp)
    _check_selection_retention(cdp)
    errors, requests = _errors_and_requests(cdp, event_start, url, artifact_url)
    _require(not errors, f"{origin_label} produced page/console errors: {' | '.join(errors)}")
    _require(not requests, f"{origin_label} produced unexpected network requests: {', '.join(requests)}")
    _check_no_js_order(cdp, url)
    return [
        f"PASS {origin_label} origin",
        "PASS Runtime fragment",
        "PASS Network fragment",
        "PASS ADO fragment",
        "PASS native keyboard tab behavior",
        "PASS no-JS document order",
        "PASS desktop viewport",
        "PASS 320px viewport",
        "PASS 200% zoom",
        "PASS reduced motion",
        "PASS focusability",
        "PASS selection retention",
        "PASS direct semantics",
        "PASS inferred semantics",
        "PASS held semantics",
        "PASS zero console errors",
        "PASS zero page errors",
        "PASS zero horizontal clipping",
        "PASS zero unexpected network requests",
    ]


def _relative_http_url(root: pathlib.Path, path: pathlib.Path, port: int) -> str:
    relative = path.relative_to(root)
    encoded = "/".join(urllib.parse.quote(part) for part in relative.parts)
    return f"http://127.0.0.1:{port}/{encoded}"


def _check_gallery(
    cdp: CDP,
    gallery_url: str,
    artifact_urls: list[str],
    origin_label: str,
) -> list[str]:
    event_start = _navigate(cdp, gallery_url)
    _set_viewport(cdp, 1440, 900)
    _check_layout(cdp, 1440, 900, f"{origin_label} gallery desktop viewport")
    discovered = cdp.evaluate(
        """(() => ({
          links: [...document.querySelectorAll('a[href]')].map(node => node.href),
          previews: [...document.querySelectorAll('iframe[src]')].map(node => node.src)
        }))()"""
    )
    links = set(discovered.get("links", []))
    previews = set(discovered.get("previews", []))
    for artifact_url in artifact_urls:
        base = artifact_url.split("#", 1)[0]
        for view_id in ("runtime", "network", "ado"):
            _require(
                f"{base}#{view_id}" in links,
                f"{origin_label} gallery is missing {view_id} deep link for {base}",
            )
        _require(
            f"{base}#network" in previews,
            f"{origin_label} gallery preview does not target the Network default for {base}",
        )

    cdp.drain(0.25)
    errors: list[str] = []
    unexpected: list[str] = []
    allowed = {gallery_url.split("#", 1)[0], *(url.split("#", 1)[0] for url in artifact_urls)}
    for event in cdp.events_since(event_start):
        method = event.get("method")
        params = event.get("params", {})
        if method == "Runtime.consoleAPICalled" and params.get("type") in {"error", "assert"}:
            errors.append(str(params))
        elif method == "Runtime.exceptionThrown":
            errors.append(str(params.get("exceptionDetails", {}).get("text", "page exception")))
        elif method == "Log.entryAdded" and params.get("entry", {}).get("level") in {"error", "assert"}:
            errors.append(str(params.get("entry", {}).get("text", "page log error")))
        elif method == "Network.requestWillBeSent":
            request_url = str(params.get("request", {}).get("url", "")).split("#", 1)[0]
            if request_url and request_url not in allowed and not request_url.endswith("/favicon.ico"):
                unexpected.append(request_url)
    _require(not errors, f"{origin_label} gallery produced page/console errors: {' | '.join(errors)}")
    _require(not unexpected, f"{origin_label} gallery produced unexpected requests: {', '.join(unexpected)}")
    return [
        f"PASS {origin_label} gallery",
        "PASS gallery Runtime/Network/ADO deep links",
        "PASS gallery Network default previews",
    ]


def _parse_args(argv: list[str]) -> argparse.Namespace:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--chromium", required=True, type=pathlib.Path, help="Chromium executable")
    parser.add_argument(
        "--artifact",
        required=True,
        action="append",
        type=pathlib.Path,
        help="Rendered HTML artifact; repeat for each map",
    )
    parser.add_argument("--gallery", type=pathlib.Path, help="Optional gallery that links the supplied artifacts")
    return parser.parse_args(argv)


def main(argv: list[str] | None = None) -> int:
    args = _parse_args(argv or sys.argv[1:])
    if not args.chromium.is_file() or not os.access(args.chromium, os.X_OK):
        print(f"FAIL Chromium executable is not runnable: {args.chromium}", file=sys.stderr)
        return 2
    artifacts = [artifact.resolve() for artifact in args.artifact]
    for artifact in artifacts:
        if not artifact.is_file():
            print(f"FAIL artifact does not exist: {artifact}", file=sys.stderr)
            return 2
    gallery = args.gallery.resolve() if args.gallery is not None else None
    if gallery is not None and not gallery.is_file():
        print(f"FAIL gallery does not exist: {gallery}", file=sys.stderr)
        return 2

    served_paths = [*artifacts, *([gallery] if gallery is not None else [])]
    serve_root = pathlib.Path(os.path.commonpath([str(path.parent) for path in served_paths]))
    server: LoopbackServer | None = None
    browser: Browser | None = None
    try:
        server = LoopbackServer(serve_root)
        server.start()
        browser = Browser(args.chromium.resolve())
        browser.start()
        _require(browser.cdp is not None, "CDP connection was not established")
        output: list[str] = []
        for artifact in artifacts:
            file_url = artifact.as_uri()
            http_url = _relative_http_url(serve_root, artifact, server.port)
            output.append(f"PASS artifact {artifact}")
            output.extend(_verify_origin(browser.cdp, file_url, f"file:// {artifact.name}", file_url))
            output.extend(_verify_origin(browser.cdp, http_url, f"loopback HTTP {artifact.name}", http_url))
        if gallery is not None:
            gallery_file_url = gallery.as_uri()
            gallery_http_url = _relative_http_url(serve_root, gallery, server.port)
            output.extend(
                _check_gallery(
                    browser.cdp,
                    gallery_file_url,
                    [artifact.as_uri() for artifact in artifacts],
                    "file://",
                )
            )
            output.extend(
                _check_gallery(
                    browser.cdp,
                    gallery_http_url,
                    [_relative_http_url(serve_root, artifact, server.port) for artifact in artifacts],
                    "loopback HTTP",
                )
            )
        for line in output:
            print(line)
        return 0
    except (OSError, VerificationError, ValueError, json.JSONDecodeError) as exc:
        print(f"FAIL {exc}", file=sys.stderr)
        return 1
    finally:
        if browser is not None:
            browser.close()
        if server is not None:
            server.close()


if __name__ == "__main__":
    raise SystemExit(main())
