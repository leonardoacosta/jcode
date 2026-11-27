# tests/support/notification_reaping.rs

- FIXTURE_ENV · constant · L9-L9 — const FIXTURE_ENV: &str = "JCODE_NOTIFICATION_REAP_TEST_DIR";
- notification_probe · function · L13-L23 — fn notification_probe()
- Probe · struct · L25-L28 — struct Probe
- start · function · L31-L68 — fn start(notifier_present: bool) -> Self
- wait_for · function · L70-L83 — fn wait_for(&mut self, description: &str, mut condition: impl FnMut(&Path) -> bool)
- drop · function · L87-L100 — fn drop(&mut self)
- notification_children_are_reaped_without_blocking · function · L104-L145 — fn notification_children_are_reaped_without_blocking()
- notification_missing_executable_is_nonfatal · function · L148-L159 — fn notification_missing_executable_is_nonfatal()
