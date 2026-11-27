# scripts/rust_safety_scan.py

- code_only · function · L9-L41 — def code_only(text: str) -> str
- module_paths · function · L48-L56 — def module_paths(path: Path, text: str) -> dict[str, set[Path]]
- test_module_files · function · L59-L86 — def test_module_files(files: list[Path], production_lines) -> set[Path]
- production_lines · function · L92-L132 — def production_lines(path: Path) -> list[str]
