# scripts/check_code_size_budget.py

- parse_args · function · L30-L37 — def parse_args() -> argparse.Namespace
- is_production_rust_file · function · L40-L58 — def is_production_rust_file(path: Path) -> bool
- rust_file_line_count · function · L61-L63 — def rust_file_line_count(path: Path) -> int
- current_oversized_files · function · L66-L77 — def current_oversized_files(threshold: int) -> dict[str, int]
- load_baseline · function · L80-L95 — def load_baseline() -> dict[str, Any]
- write_baseline · function · L98-L107 — def write_baseline(threshold: int, tracked_files: dict[str, int]) -> None
- main · function · L110-L171 — def main() -> int
