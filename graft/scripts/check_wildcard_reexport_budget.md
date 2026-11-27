# scripts/check_wildcard_reexport_budget.py

- parse_args · function · L40-L43 — def parse_args() -> argparse.Namespace
- production_rust_files · function · L46-L56 — def production_rust_files() -> list[Path]
- count_wildcards · function · L59-L71 — def count_wildcards(path: Path) -> int
- collect_counts · function · L74-L81 — def collect_counts() -> dict[str, int]
- load_baseline · function · L84-L90 — def load_baseline() -> dict[str, Any] | None
- write_baseline · function · L93-L98 — def write_baseline(counts: dict[str, int]) -> None
- main · function · L101-L161 — def main() -> int
