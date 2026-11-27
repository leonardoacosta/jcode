# scripts/test_bench_jcode_maki.py

- BenchmarkTests · class · L13-L49 — class BenchmarkTests(unittest.TestCase)
- test_snapshot_includes_live_process · method · L14-L20 — def test_snapshot_includes_live_process(self)
- test_snapshot_ignores_missing_process · method · L22-L23 — def test_snapshot_ignores_missing_process(self)
- test_application_echo_not_terminal_driver_echo · method · L25-L34 — def test_application_echo_not_terminal_driver_echo(self): # Child explicitly reads and renders input. PTY ECHO is disabled.
- test_unread_input_does_not_count_as_ready · method · L36-L43 — def test_unread_input_does_not_count_as_ready(self)
- test_stop_reaps_owned_process · method · L45-L49 — def test_stop_reaps_owned_process(self)
