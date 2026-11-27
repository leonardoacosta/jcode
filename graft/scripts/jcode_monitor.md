# scripts/jcode_monitor.py

- Colors · class · L20-L37 — class Colors
- MonitorState · class · L41-L53 — class MonitorState
- get_socket_path · function · L56-L59 — def get_socket_path() -> str
- connect_to_socket · function · L62-L70 — def connect_to_socket(path: str) -> Optional[socket.socket]
- send_request · function · L73-L80 — def send_request(sock: socket.socket, request: dict) -> bool
- read_events · function · L83-L107 — def read_events(sock: socket.socket) -> list
- format_tokens · function · L110-L119 — def format_tokens(n: int) -> str
- format_tool · function · L122-L130 — def format_tool(name: str, status: str = "active") -> str
- truncate · function · L133-L137 — def truncate(s: str, max_len: int) -> str
- render_dashboard · function · L140-L217 — def render_dashboard(state: MonitorState, width: int = 80)
- process_event · function · L220-L278 — def process_event(event: dict, state: MonitorState)
- main · function · L281-L337 — def main()
