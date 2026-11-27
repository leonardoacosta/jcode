# crates/jcode-app-core/src/server/idle_hibernation.rs

- CHECK_INTERVAL · constant · L11-L11 — const CHECK_INTERVAL: Duration = Duration::from_secs(10);
- IdleState · struct · L13-L17 — struct IdleState
- new · function · L20-L26 — fn new(now: Instant, last_status_change: Instant) -> Self
- observe_status · function · L28-L36 — fn observe_status(&mut self, last_status_change: Instant)
- due · function · L38-L50 — fn due(
- spawn · function · L53-L65 — pub(super) fn spawn(sessions: SessionAgents, members: Arc<RwLock<HashMap<String, SwarmMember>>>)
- reclaim_idle_caches · function · L67-L169 — async fn reclaim_idle_caches(
- tests · module · L172-L304 — mod tests
- IdleProvider · struct · L179-L179 — struct IdleProvider;
- complete · function · L182-L190 — async fn complete(
- name · function · L191-L193 — fn name(&self) -> &str
- fork · function · L194-L196 — fn fork(&self) -> Arc<dyn Provider>
- idle_hibernation_due_respects_disable_idle_and_recent_activity · function · L200-L210 — fn idle_hibernation_due_respects_disable_idle_and_recent_activity()
- idle_hibernation_rearms_for_turn_entirely_between_monitor_ticks · function · L213-L231 — fn idle_hibernation_rearms_for_turn_entirely_between_monitor_ticks()
- idle_hibernation_monitor_skips_busy_and_running_then_releases_once · function · L234-L303 — fn idle_hibernation_monitor_skips_busy_and_running_then_releases_once()
