# scripts/compare_token_usage.py

- TokenUsage · class · L39-L56 — class TokenUsage
- total_input · method · L49-L51 — def total_input(self) -> int
- total · method · L54-L56 — def total(self) -> int
- RunResult · class · L60-L67 — class RunResult
- find_jcode_binary · function · L70-L83 — def find_jcode_binary() -> str
- run_claude_cli · function · L86-L161 — def run_claude_cli(prompt: str, workdir: str, model: str = "opus") -> RunResult
- run_jcode · function · L164-L241 — def run_jcode(prompt: str, workdir: str, jcode_binary: str, model: str = "claude-opus-4-5-20251101") -> RunResult
- compare_usage · function · L244-L295 — def compare_usage(claude_result: RunResult, jcode_result: RunResult, verbose: bool = False) -> dict
- pct_diff · function · L257-L260 — def pct_diff(a: int, b: int) -> float
- print_comparison · function · L298-L321 — def print_comparison(comparison: dict, prompt: str, verbose: bool = False)
- run_test_suite · function · L324-L380 — def run_test_suite(verbose: bool = False, runs: int = 1) -> list
- summarize_results · function · L383-L451 — def summarize_results(results: list) -> bool
- main · function · L454-L496 — def main()
