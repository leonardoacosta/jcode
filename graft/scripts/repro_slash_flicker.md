# scripts/repro_slash_flicker.py

- wait_for_socket · function · L84-L97 — def wait_for_socket(path: Path, timeout_s: float = 30.0) -> None
- _recv · function · L100-L117 — def _recv(sock: socket.socket, timeout: float) -> dict
- dbg · function · L120-L131 — def dbg(debug_sock: Path, command: str, timeout: float = 30.0) -> str
- client_cmd · function · L134-L147 — def client_cmd(cmd_path: Path, resp_path: Path, command: str, timeout_s: float = 8.0) -> str
- Screen · class · L151-L233 — class Screen
- __post_init__ · method · L168-L169 — def __post_init__(self) -> None
- _pump · method · L171-L204 — def _pump(self) -> None
- start_pump · method · L206-L208 — def start_pump(self) -> None
- rows · method · L210-L212 — def rows(self) -> list[str]
- send · method · L214-L215 — def send(self, data: bytes) -> None
- shutdown · method · L217-L233 — def shutdown(self) -> None
- launch · function · L236-L255 — def launch(binary: str, env: dict, session_id: str, cmd_path: Path, resp_path: Path) -> Screen
- settle · function · L258-L266 — def settle(cmd_path: Path, resp_path: Path, timeout_s: float = 60.0) -> bool
- menu_rows · function · L270-L272 — def menu_rows(rows: list[str], needles: tuple[str, ...]) -> int
- watch_menu · function · L275-L312 — def watch_menu(client: Screen, needles: tuple[str, ...], seconds: float, verbose: bool = False) -> dict
- watch_static_rows · function · L315-L337 — def watch_static_rows(client: Screen, seconds: float) -> dict
- main · function · L340-L495 — def main() -> int
