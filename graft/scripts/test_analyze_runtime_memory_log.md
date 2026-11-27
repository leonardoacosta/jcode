# scripts/test_analyze_runtime_memory_log.py

- sample · function · L19-L71 — def sample( *, timestamp_ms: int, instance_id: str, pss_mb: int, allocated_mb: int, live_sessions: int | None = None, connected_clients: int = 0, total_json_mb: int = 0, ) -> analyzer.Sample
- RuntimeMemoryAnalyzerTests · class · L74-L163 — class RuntimeMemoryAnalyzerTests(unittest.TestCase)
- test_latest_instance_filter_prevents_cross_reload_spikes · method · L75-L88 — def test_latest_instance_filter_prevents_cross_reload_spikes(self) -> None
- test_explicit_instance_selection_preserves_historical_incident · method · L90-L111 — def test_explicit_instance_selection_preserves_historical_incident(self) -> None
- test_runaway_session_population_is_primary_cause · method · L113-L144 — def test_runaway_session_population_is_primary_cause(self) -> None
- test_allocator_retention_has_purge_first_action · method · L146-L163 — def test_allocator_retention_has_purge_first_action(self) -> None
