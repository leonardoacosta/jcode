# scripts/diagnose_idle_render_cost.py

- wait_for_socket · function · L77-L90 — def wait_for_socket(path: Path, timeout_s: float = 30.0) -> None
- _recv · function · L93-L110 — def _recv(sock: socket.socket, timeout: float) -> dict
- dbg · function · L113-L124 — def dbg(debug_sock: Path, command: str, timeout: float = 30.0) -> str
- client_cmd · function · L127-L140 — def client_cmd(cmd_path: Path, resp_path: Path, command: str, timeout_s: float = 10.0) -> str
- Client · class · L144-L215 — class Client
- _pump · method · L150-L178 — def _pump(self) -> None
- start_pump · method · L180-L182 — def start_pump(self) -> None
- send · method · L184-L185 — def send(self, data: bytes) -> None
- cpu_seconds · method · L187-L197 — def cpu_seconds(self) -> float | None
- shutdown · method · L199-L215 — def shutdown(self) -> None
- launch · function · L218-L237 — def launch(binary: str, env: dict, session_id: str, cmd_path: Path, resp_path: Path) -> Client
- settle · function · L240-L248 — def settle(cmd_path: Path, resp_path: Path, timeout_s: float = 60.0) -> bool
- counters · function · L251-L265 — def counters(cmd_path: Path, resp_path: Path) -> dict
- probe_state · function · L268-L310 — def probe_state(client: Client, cmd_path: Path, resp_path: Path, label: str, window_s: float) -> dict
- delta · function · L279-L280 — def delta(key: str) -> int
- main · function · L313-L443 — def main() -> int
- record · function · L391-L402 — def record(label: str) -> dict
