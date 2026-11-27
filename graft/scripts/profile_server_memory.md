# scripts/profile_server_memory.py

- isolated_env · function · L39-L50 — def isolated_env(root: Path, inherited: dict[str, str], heap_profile: bool = False) -> dict[str, str]
- copy_auth · function · L53-L62 — def copy_auth(source: Path, home: Path) -> list[str]
- debug_cmd · function · L65-L92 — def debug_cmd(path: Path, command: str, session_id: str | None, timeout: float) -> str
- validate_population · function · L95-L104 — def validate_population(info: dict, members: list, sid: str | None) -> None
- smaps · function · L107-L116 — def smaps(pid: int) -> dict
- full_smaps · function · L119-L121 — def full_smaps(pid: int) -> dict
- save · function · L124-L125 — def save(path: Path, value) -> None
- capture_pricing_cache · function · L128-L137 — def capture_pricing_cache(home: Path, output: Path) -> dict
- run · function · L140-L274 — def run(args) -> None
- cmd · function · L185-L186 — def cmd(command, sid=None)
- capture · function · L188-L228 — def capture(stage, sid=None)
- main · function · L277-L312 — def main() -> int
