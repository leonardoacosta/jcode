# scripts/benchmark_attribution.py

- AttributionError · class · L98-L99 — class AttributionError(RuntimeError)
- CheckResult · class · L103-L106 — class CheckResult
- SponsorReport · class · L110-L144 — class SponsorReport
- score · method · L117-L127 — def score(self) -> int
- weight · function · L122-L123 — def weight(check: CheckResult) -> int
- cli_check · method · L130-L131 — def cli_check(self) -> CheckResult | None
- cli_verdict · method · L134-L139 — def cli_verdict(self) -> str
- cli_detail · method · L142-L144 — def cli_detail(self) -> str
- parse_args · function · L147-L161 — def parse_args() -> argparse.Namespace
- load_sponsor_expectations · function · L164-L194 — def load_sponsor_expectations(path: Path) -> tuple[str, list[dict[str, Any]]]
- load_catalog_entries · function · L197-L208 — def load_catalog_entries(path: Path) -> dict[str, dict[str, Any]]
- http_get · function · L211-L217 — def http_get(url: str, timeout: float, benchmark: bool = False) -> tuple[int, str, str]
- fetch_live_entry · function · L220-L245 — def fetch_live_entry(endpoint: str, sponsor: dict[str, Any], timeout: float) -> dict[str, Any]
- urls_in_text · function · L248-L249 — def urls_in_text(text: str) -> list[str]
- marker_in_url · function · L252-L253 — def marker_in_url(url: str, marker: str) -> bool
- check_sponsor · function · L256-L357 — def check_sponsor( sponsor: dict[str, Any], entry: dict[str, Any] | None, live_web: bool, timeout: float, ) -> SponsorReport
- main · function · L360-L452 — def main() -> int
