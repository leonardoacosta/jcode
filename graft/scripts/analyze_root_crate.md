# scripts/analyze_root_crate.py

- _split_top_level · function · L34-L53 — def _split_top_level(body: str) -> list[str]
- _module_imports · function · L56-L99 — def _module_imports(joined: str)
- top_level_modules · function · L102-L136 — def top_level_modules() -> dict[str, list[str]]
- loc · function · L139-L147 — def loc(files: list[str]) -> int
- facade_ratio · function · L150-L175 — def facade_ratio(name: str) -> tuple[float, int, int]
- classify_facade · function · L178-L196 — def classify_facade(name: str, total_loc: int = 0) -> str
- is_facade · function · L199-L201 — def is_facade(name: str, files: list[str]) -> bool: # "Extracted enough" to no longer count as a real in-root blocker for bulk.
- outbound_refs · function · L204-L268 — def outbound_refs(files: list[str], self_name: str, exclude_tests: bool = True)
- strongly_connected_components · function · L271-L315 — def strongly_connected_components(graph: dict[str, set[str]]) -> list[list[str]]
- strongconnect · function · L290-L310 — def strongconnect(v: str) -> None
- main · function · L318-L480 — def main() -> int
- is_blocker · function · L344-L345 — def is_blocker(dep: str) -> bool
- eades_order · function · L483-L522 — def eades_order(nodes, out_edges)
- score · function · L515-L518 — def score(n)
