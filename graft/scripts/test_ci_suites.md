# scripts/test_ci_suites.py

- Suite · class · L29-L38 — class Suite
- command · method · L34-L38 — def command(self, *, parallel: bool) -> list[str]
- terminate_process_group · function · L52-L60 — def terminate_process_group(proc: subprocess.Popen[bytes]) -> None
- handle_signal · function · L63-L66 — def handle_signal(signum: int, _frame: FrameType | None) -> None
- parse_args · function · L69-L89 — def parse_args() -> argparse.Namespace
- selected_suites · function · L92-L95 — def selected_suites(names: list[str]) -> list[Suite]
- progress · function · L98-L101 — def progress(message: str, **extra: object) -> None
- run_suite · function · L104-L139 — def run_suite(suite: Suite, timeout_scale: float, *, parallel: bool) -> tuple[int, float]
- main · function · L142-L167 — def main() -> int
