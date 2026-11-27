# scripts/jcode_memory_snapshot.py

- ProcMem · class · L16-L28 — class ProcMem
- Totals · class · L32-L39 — class Totals
- parse_args · function · L53-L66 — def parse_args() -> argparse.Namespace
- read_text · function · L69-L75 — def read_text(path: Path, binary: bool = False) -> str | None
- read_argv · function · L78-L85 — def read_argv(path: Path) -> list[str] | None
- parse_socket_path · function · L88-L90 — def parse_socket_path(cmd: str) -> str | None
- parse_session_id · function · L93-L95 — def parse_session_id(cmd: str) -> str | None
- first_non_option · function · L98-L110 — def first_non_option(argv: list[str]) -> str | None
- classify_process · function · L113-L140 — def classify_process(argv: list[str], cmd: str, main_socket: str) -> tuple[str | None, bool]
- parse_smaps_rollup · function · L143-L155 — def parse_smaps_rollup(pid: int) -> dict[str, float] | None
- iter_jcode_processes · function · L158-L184 — def iter_jcode_processes(main_socket: str, include_aux: bool) -> Iterable[ProcMem]
- sum_totals · function · L187-L196 — def sum_totals(procs: list[ProcMem]) -> Totals
- print_human · function · L199-L230 — def print_human(server: list[ProcMem], clients: list[ProcMem], aux: list[ProcMem]) -> None
- print_group · function · L200-L213 — def print_group(name: str, procs: list[ProcMem]) -> None
- main · function · L233-L256 — def main() -> int
