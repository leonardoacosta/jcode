# src/cli/cloud_mux.rs

- KEEPALIVE · constant · L23-L28 — const KEEPALIVE: [&str; 4] = [
- base · function · L30-L38 — fn base(config: &[String]) -> Command
- master_running · function · L40-L47 — fn master_running(config: &[String]) -> bool
- master_args · function · L49-L70 — fn master_args() -> Vec<&'static str>
- ensure_master · function · L74-L105 — pub(crate) fn ensure_master(deadline: Duration) -> Result<bool>
- spawn_master · function · L107-L160 — fn spawn_master(config: &[String], dir: &std::path::Path, deadline: Duration) -> Result<()>
- ensure · function · L164-L190 — pub(crate) async fn ensure(host: &str, deadline: Duration)
- tests · module · L193-L225 — mod tests
- master_detaches_owns_the_socket_and_cannot_forward · function · L197-L213 — fn master_detaches_owns_the_socket_and_cannot_forward()
- other_hosts_never_get_a_shared_master · function · L216-L224 — async fn other_hosts_never_get_a_shared_master()
