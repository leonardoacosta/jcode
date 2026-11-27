# scripts/jcode_harbor_claude_agent.py

- _resolve_existing_file · function · L30-L41 — def _resolve_existing_file(*, env_name: str, default_path: str | None = None, candidates: tuple[str | None, ...] = ()) -> Path
- _resolve_optional_existing_file · function · L44-L51 — def _resolve_optional_existing_file(*, candidates: tuple[str | None, ...]) -> Path | None
- _sibling_runtime_lib_candidates · function · L54-L55 — def _sibling_runtime_lib_candidates(binary: Path, stem: str) -> tuple[str, ...]
- _load_key_from_env_file · function · L58-L78 — def _load_key_from_env_file(env_path: str, env_var: str, *direct_env: str) -> str | None
- _load_anthropic_key · function · L81-L87 — def _load_anthropic_key() -> str | None
- _load_openrouter_key · function · L90-L109 — def _load_openrouter_key() -> str | None: # Priority: explicit env, then the jcode openrouter.env file (raw key per line).
- _benchmark_instruction_preamble · function · L146-L147 — def _benchmark_instruction_preamble() -> str
- _load_final_payload · function · L150-L184 — def _load_final_payload(output_dir: Path) -> dict[str, Any] | None
- JcodeClaudeHarborAgent · class · L187-L329 — class JcodeClaudeHarborAgent(BaseAgent)
- __init__ · method · L196-L210 — def __init__(self, logs_dir: Path, model_name: str | None = None, *args, **kwargs)
- name · method · L213-L214 — def name() -> str
- version · method · L216-L217 — def version(self) -> str | None
- setup · method · L219-L257 — async def setup(self, environment: BaseEnvironment) -> None: # NOTE: Do NOT symlink/override the container's system OpenSSL cert dir # (e.g. ln -s ... /usr/lib/ssl/certs). The TB base images are often bare # ubuntu:24.04 with no /etc/ssl/certs, and the per-task verifier runs # `apt-get install ca-certificates curl` then bootstraps uv over https. # Hijacking the system cert dir breaks the verifier's curl (error 77, # "error setting certificate file") so tests never run. jcode itself # gets its CA bundle via SSL_CERT_FILE/OPENSSL_CERT_FILE below, so no # global cert override is needed.
- run · method · L259-L329 — async def run(self, instruction: str, environment: BaseEnvironment, context: AgentContext) -> None
