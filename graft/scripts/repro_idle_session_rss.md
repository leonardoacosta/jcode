# scripts/repro_idle_session_rss.py

- mem · function · L25-L31 — def mem(pid: int) -> dict[str, int]
- wait_for · function · L34-L40 — def wait_for(path: str, timeout: float) -> bool
- open_session · function · L43-L72 — def open_session(sock_path: str, cwd: str, abrupt: bool = False, realistic: bool = False) -> socket.socket
- debug · function · L75-L90 — def debug(sock_path: str, cmd: str) -> str
- main · function · L93-L179 — def main() -> int
