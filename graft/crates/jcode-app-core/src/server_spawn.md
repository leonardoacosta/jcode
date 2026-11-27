# crates/jcode-app-core/src/server_spawn.rs

- ServerSpawner · type · L24-L25 — type ServerSpawner =
- DEFAULT_SERVER_SPAWNER · constant · L27-L27 — static DEFAULT_SERVER_SPAWNER: OnceLock<ServerSpawner> = OnceLock::new();
- ON_DEMAND_SERVER_SPAWNER · constant · L28-L28 — static ON_DEMAND_SERVER_SPAWNER: OnceLock<ServerSpawner> = OnceLock::new();
- ON_DEMAND_SERVER_LOCK · constant · L29-L29 — static ON_DEMAND_SERVER_LOCK: OnceLock<tokio::sync::Mutex<()>> = OnceLock::new();
- ON_DEMAND_SERVER_SPAWN_FAILED · constant · L30-L30 — static ON_DEMAND_SERVER_SPAWN_FAILED: AtomicBool = AtomicBool::new(false);
- register_default_server_spawner · function · L36-L38 — pub fn register_default_server_spawner(spawner: ServerSpawner)
- is_running · function · L41-L44 — pub async fn is_running() -> bool
- spawn_default_server · function · L50-L55 — pub async fn spawn_default_server() -> Result<()>
- register_on_demand_server_spawner · function · L71-L73 — pub fn register_on_demand_server_spawner(spawner: ServerSpawner)
- on_demand_server_spawner_registered · function · L76-L78 — pub fn on_demand_server_spawner_registered() -> bool
- ensure_on_demand_server · function · L87-L112 — pub async fn ensure_on_demand_server() -> Result<()>
