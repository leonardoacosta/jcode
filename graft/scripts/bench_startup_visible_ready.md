# scripts/bench_startup_visible_ready.py

- ToolSpec · class · L50-L55 — class ToolSpec
- parse_args · function · L58-L74 — def parse_args() -> argparse.Namespace
- detect_pi_bin · function · L77-L85 — def detect_pi_bin() -> str
- shutil_which · function · L88-L94 — def shutil_which(name: str) -> str | None
- build_tool_specs · function · L97-L119 — def build_tool_specs() -> list[ToolSpec]
- configure_pty · function · L122-L127 — def configure_pty(slave_fd: int, rows: int = 24, cols: int = 80) -> None
- reply_queries · function · L130-L157 — def reply_queries(master_fd: int, buffer: bytes) -> bytes
- first_meaningful_line · function · L160-L168 — def first_meaningful_line(screen: pyte.Screen) -> str | None
- run_once · function · L171-L264 — def run_once(spec: ToolSpec, cwd: Path, timeout_s: float) -> dict[str, object]
- summarize · function · L267-L278 — def summarize(samples: list[float | None]) -> dict[str, float | int] | None
- version_for · function · L281-L289 — def version_for(spec: ToolSpec) -> str
- main · function · L292-L327 — def main() -> None
