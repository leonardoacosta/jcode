# scripts/test_swarm.py

- send_cmd · function · L28-L51 — def send_cmd(cmd: str, session_id: str = None, timeout: float = 30) -> tuple
- create_session · function · L54-L59 — def create_session(working_dir: str = TEST_DIR) -> str
- destroy_session · function · L62-L64 — def destroy_session(session_id: str)
- get_swarm_id · function · L67-L72 — def get_swarm_id(path: str = TEST_DIR) -> str
- get_coordinator · function · L75-L83 — def get_coordinator(swarm_id: str) -> str
- test_coordinator_election · function · L86-L123 — def test_coordinator_election()
- test_communication · function · L126-L182 — def test_communication()
- test_invalid_dm · function · L185-L212 — def test_invalid_dm()
- test_swarm_id_non_git · function · L215-L264 — def test_swarm_id_non_git()
- test_plan_approval · function · L267-L351 — def test_plan_approval()
- test_plan_rejection · function · L354-L425 — def test_plan_rejection()
- test_coordinator_only_approval · function · L428-L463 — def test_coordinator_only_approval()
- main · function · L466-L520 — def main()
