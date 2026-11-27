# scripts/compile_isolation_report.py

- CrateStats · class · L24-L40 — class CrateStats
- run_metadata · function · L43-L51 — def run_metadata() -> dict[str, Any]
- lib_or_root_src · function · L54-L62 — def lib_or_root_src(package: dict[str, Any]) -> Path | None
- iter_rust_files · function · L65-L74 — def iter_rust_files(src_path: Path | None) -> list[Path]
- count_file · function · L77-L79 — def count_file(path: Path) -> tuple[int, str]
- collect_stats · function · L82-L134 — def collect_stats(package: dict[str, Any], workspace_names: set[str]) -> CrateStats
- target_state_violations · function · L137-L169 — def target_state_violations(stats_by_name: dict[str, CrateStats]) -> list[str]
- main · function · L172-L247 — def main() -> int
