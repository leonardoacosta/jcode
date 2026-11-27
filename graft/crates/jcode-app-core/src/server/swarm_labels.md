# crates/jcode-app-core/src/server/swarm_labels.rs

- LABELS_FILE · constant · L19-L19 — const LABELS_FILE: &str = "swarm-labels.json";
- MAX_SWARM_LABEL_CHARS · constant · L20-L20 — pub(super) const MAX_SWARM_LABEL_CHARS: usize = 48;
- LABELS · constant · L22-L23 — static LABELS: LazyLock<StdMutex<Option<HashMap<String, String>>>> =
- labels_path · function · L26-L28 — fn labels_path() -> PathBuf
- labels_path · function · L31-L35 — fn labels_path() -> PathBuf
- with_labels · function · L37-L43 — fn with_labels<R>(f: impl FnOnce(&mut HashMap<String, String>) -> R) -> R
- persist · function · L45-L49 — fn persist(labels: &HashMap<String, String>)
- swarm_label · function · L51-L53 — pub(super) fn swarm_label(swarm_id: &str) -> Option<String>
- normalize_swarm_label · function · L56-L58 — pub(super) fn normalize_swarm_label(label: &str) -> String
- set_swarm_label · function · L63-L95 — pub(super) fn set_swarm_label(
- resolve_swarm_target · function · L99-L124 — pub(super) fn resolve_swarm_target(
- list_swarms · function · L127-L175 — pub(super) async fn list_swarms(
- swarm_display_name · function · L178-L180 — pub(super) fn swarm_display_name(swarm_id: &str) -> String
- SWARM_LABELS_TEST_LOCK · constant · L183-L183 — pub(super) static SWARM_LABELS_TEST_LOCK: StdMutex<()> = StdMutex::new(());
- reset_swarm_labels_for_test · function · L186-L190 — pub(super) fn reset_swarm_labels_for_test()
- tests · module · L193-L221 — mod tests
- labels_are_unique_and_resolve_case_insensitively · function · L197-L220 — fn labels_are_unique_and_resolve_case_insensitively()
