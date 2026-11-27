# scripts/benchmark_swarm.py

- send_cmd · function · L45-L81 — def send_cmd(cmd: str, session_id: str = None, timeout: float = 300) -> tuple
- create_session · function · L84-L90 — def create_session(working_dir: str) -> tuple
- destroy_session · function · L93-L95 — def destroy_session(session_id: str)
- setup_workspace · function · L102-L118 — def setup_workspace(name: str) -> str
- get_cycles · function · L121-L136 — def get_cycles(workspace: str) -> int
- get_test_summary · function · L139-L151 — def get_test_summary(workspace: str) -> str
- poll_job · function · L191-L240 — def poll_job( job_id: str, session_id: str, workspace: str, start_time: float, timeout_seconds: float, check_interval: float, label: str, ) -> int
- run_single_agent · function · L247-L319 — def run_single_agent(timeout_minutes: float, check_interval: float) -> dict
- run_swarm · function · L326-L447 — def run_swarm(timeout_minutes: float, check_interval: float) -> dict
- print_comparison · function · L454-L510 — def print_comparison(results: dict)
- main · function · L517-L584 — def main()
