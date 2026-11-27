# scripts/check_swallowed_error_budget.py

- parse_args · function · L40-L43 — def parse_args() -> argparse.Namespace
- is_test_rust_file · function · L46-L62 — def is_test_rust_file(path: Path) -> bool
- production_rust_files · function · L65-L74 — def production_rust_files() -> list[Path]
- zero_counts · function · L77-L78 — def zero_counts() -> dict[str, int]
- current_counts · function · L81-L91 — def current_counts() -> dict[str, dict[str, int]]
- file_total · function · L94-L95 — def file_total(counts: dict[str, int]) -> int
- total_counts · function · L98-L103 — def total_counts(counts: dict[str, dict[str, int]]) -> dict[str, int]
- grand_total · function · L106-L107 — def grand_total(counts: dict[str, dict[str, int]]) -> int
- load_baseline · function · L110-L130 — def load_baseline() -> dict[str, Any]
- write_baseline · function · L133-L147 — def write_baseline(counts: dict[str, dict[str, int]]) -> None
- main · function · L150-L216 — def main() -> int
