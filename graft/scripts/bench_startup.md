# scripts/bench_startup.py

- StartupProfile · class · L37-L40 — class StartupProfile
- Budget · class · L44-L47 — class Budget
- parse_args · function · L50-L67 — def parse_args() -> argparse.Namespace
- median · function · L70-L74 — def median(values: Iterable[float]) -> float
- median_or_none · function · L77-L81 — def median_or_none(values: Iterable[float]) -> float | None
- print_stats · function · L84-L94 — def print_stats(name: str, times: list[float]) -> None
- run_simple_timing · function · L97-L103 — def run_simple_timing(binary: str, *args: str, runs: int) -> list[float]
- isolated_env · function · L106-L114 — def isolated_env(root: str) -> dict[str, str]
- wait_for_socket · function · L117-L129 — def wait_for_socket(path: str, timeout_s: float) -> bool
- measure_server_startup · function · L132-L158 — def measure_server_startup(binary: str, runs: int) -> list[float]
- require_script_binary · function · L161-L165 — def require_script_binary() -> str
- parse_startup_profile · function · L168-L200 — def parse_startup_profile(log_path: Path) -> StartupProfile
- measure_cold_client_startup · function · L203-L227 — def measure_cold_client_startup(binary: str, runs: int) -> list[StartupProfile]
- print_cold_profile_stats · function · L230-L239 — def print_cold_profile_stats(profiles: list[StartupProfile]) -> None
- collect_budgets · function · L242-L289 — def collect_budgets( help_times: list[float], version_times: list[float], server_times: list[float], cold_profiles: list[StartupProfile], args: argparse.Namespace, ) -> list[Budget]
- main · function · L292-L363 — def main() -> int
