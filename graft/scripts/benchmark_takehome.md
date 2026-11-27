# scripts/benchmark_takehome.py

- send_cmd · function · L31-L67 — def send_cmd(cmd: str, session_id: str = None, timeout: float = 300) -> tuple
- create_session · function · L70-L76 — def create_session(working_dir: str) -> tuple
- destroy_session · function · L79-L81 — def destroy_session(session_id: str)
- setup_workspace · function · L84-L90 — def setup_workspace(name: str) -> str
- get_cycles · function · L93-L108 — def get_cycles(workspace: str) -> int
- make_single_prompt · function · L111-L135 — def make_single_prompt(workspace: str) -> str
- run_single_agent · function · L138-L228 — def run_single_agent() -> dict
- run_swarm · function · L231-L354 — def run_swarm(n_agents: int = 2) -> dict
- print_results · function · L357-L373 — def print_results(results: dict)
- main · function · L376-L404 — def main()
