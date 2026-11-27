# scripts/check_panic_budget.py

- parse_args · function · L33-L36 — def parse_args() -> argparse.Namespace
- is_test_rust_file · function · L39-L55 — def is_test_rust_file(path: Path) -> bool
- production_rust_files · function · L58-L67 — def production_rust_files() -> list[Path]
- current_counts · function · L70-L76 — def current_counts() -> dict[str, int]
- load_baseline · function · L79-L93 — def load_baseline() -> dict[str, Any]
- write_baseline · function · L96-L105 — def write_baseline(counts: dict[str, int]) -> None
- main · function · L108-L158 — def main() -> int
