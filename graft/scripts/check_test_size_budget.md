# scripts/check_test_size_budget.py

- parse_args · function · L25-L28 — def parse_args() -> argparse.Namespace
- is_test_rust_file · function · L31-L47 — def is_test_rust_file(path: Path) -> bool
- rust_file_line_count · function · L50-L52 — def rust_file_line_count(path: Path) -> int
- current_oversized_files · function · L55-L66 — def current_oversized_files(threshold: int) -> dict[str, int]
- load_baseline · function · L69-L83 — def load_baseline() -> dict[str, Any]
- write_baseline · function · L86-L95 — def write_baseline(threshold: int, tracked_files: dict[str, int]) -> None
- main · function · L98-L145 — def main() -> int
