# scripts/jcode_harbor_agent.py

- _resolve_existing_file · function · L28-L39 — def _resolve_existing_file(*, env_name: str, default_path: str | None = None, candidates: tuple[str | None, ...] = ()) -> Path
- _resolve_optional_existing_file · function · L42-L49 — def _resolve_optional_existing_file(*, candidates: tuple[str | None, ...]) -> Path | None
- _sibling_runtime_lib_candidates · function · L52-L53 — def _sibling_runtime_lib_candidates(binary: Path, stem: str) -> tuple[str, ...]
- _benchmark_instruction_preamble · function · L116-L123 — def _benchmark_instruction_preamble() -> str: # Keep Harbor runs aligned with normal TUI/jcode-run prompting by default. # The legacy preamble can still be enabled explicitly for reproducing older # runs, but new benchmark runs should rely on jcode's normal system prompt # and the official Terminal-Bench task instruction.
- _load_task_hint · function · L126-L146 — def _load_task_hint() -> str
- _load_final_payload · function · L149-L183 — def _load_final_payload(output_dir: Path) -> dict[str, Any] | None
- JcodeHarborAgent · class · L186-L320 — class JcodeHarborAgent(BaseAgent)
- __init__ · method · L187-L193 — def __init__(self, logs_dir: Path, model_name: str | None = None, *args, **kwargs)
- name · method · L196-L197 — def name() -> str
- version · method · L199-L200 — def version(self) -> str | None
- setup · method · L202-L231 — async def setup(self, environment: BaseEnvironment) -> None
- run · method · L233-L320 — async def run(self, instruction: str, environment: BaseEnvironment, context: AgentContext) -> None
