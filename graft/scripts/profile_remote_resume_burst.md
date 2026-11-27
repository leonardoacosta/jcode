# scripts/profile_remote_resume_burst.py

- parse_args · function · L22-L29 — def parse_args() -> argparse.Namespace
- ProcSample · class · L33-L35 — class ProcSample
- ProcTracker · class · L39-L56 — class ProcTracker
- record · method · L44-L51 — def record(self, sample: ProcSample | None) -> bool
- cpu_ms · method · L53-L56 — def cpu_ms(self) -> float
- read_proc_sample · function · L59-L73 — def read_proc_sample(pid: int) -> ProcSample | None
- wait_for_socket · function · L76-L89 — def wait_for_socket(path: Path, timeout_s: float = 10.0) -> None
- create_session · function · L92-L116 — def create_session(debug_sock: Path, cwd: str = ".") -> str
- reply_queries · function · L119-L145 — def reply_queries(master_fd: int, buffer: bytes) -> bytes
- LiveClient · class · L149-L158 — class LiveClient
- start_resume_client · function · L161-L190 — def start_resume_client(binary: str, env: dict[str, str], session_id: str) -> LiveClient
- finish_client · function · L193-L216 — def finish_client(client: LiveClient) -> dict
- run_burst · function · L219-L317 — def run_burst( binary: str, env: dict[str, str], session_ids: list[str], timeout_s: float, server_pid: int, stagger_ms: float, ) -> tuple[list[dict], dict[str, float | int]]
- sample_processes · function · L237-L248 — def sample_processes() -> None
- main · function · L320-L394 — def main() -> None
