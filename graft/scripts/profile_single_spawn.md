# scripts/profile_single_spawn.py

- parse_args · function · L17-L23 — def parse_args() -> argparse.Namespace
- wait_for_socket · function · L26-L39 — def wait_for_socket(path: Path, timeout_s: float = 10.0) -> None
- create_session · function · L42-L66 — def create_session(debug_sock: Path, cwd: str) -> str
- reply_queries · function · L69-L95 — def reply_queries(master_fd: int, buffer: bytes) -> bytes
- latest_log_file · function · L98-L102 — def latest_log_file(log_dir: Path) -> Path
- extract_timing_lines · function · L105-L110 — def extract_timing_lines(log_path: Path) -> list[str]
- profile_single_spawn · function · L113-L227 — def profile_single_spawn(binary: str, cwd: str, timeout_s: float) -> dict
- main · function · L230-L246 — def main() -> None
