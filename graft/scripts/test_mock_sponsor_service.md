# scripts/test_mock_sponsor_service.py

- MockSponsorEndToEndTests · class · L26-L160 — class MockSponsorEndToEndTests(unittest.TestCase)
- setUpClass · method · L28-L34 — def setUpClass(cls)
- tearDownClass · method · L37-L41 — def tearDownClass(cls)
- cli · method · L43-L49 — def cli(self, *args: str, check: bool = True) -> subprocess.CompletedProcess[str]
- signup_and_confirm · method · L51-L58 — def signup_and_confirm(self, email: str, via: str | None) -> str
- account · method · L60-L62 — def account(self, email: str) -> dict
- test_cli_flag_survives_magic_link_and_is_persisted · method · L64-L68 — def test_cli_flag_survives_magic_link_and_is_persisted(self)
- test_first_acquisition_source_is_immutable · method · L70-L74 — def test_first_acquisition_source_is_immutable(self)
- test_omitted_flag_creates_unattributed_account · method · L76-L79 — def test_omitted_flag_creates_unattributed_account(self)
- test_tampered_magic_link_is_rejected · method · L81-L94 — def test_tampered_magic_link_is_rejected(self)
- test_magic_link_is_one_time_use · method · L96-L100 — def test_magic_link_is_one_time_use(self)
- test_expired_token_is_rejected · method · L102-L105 — def test_expired_token_is_rejected(self)
- test_real_attribution_benchmark_accepts_discovery_to_cli_contract · method · L107-L141 — def test_real_attribution_benchmark_accepts_discovery_to_cli_contract(self)
- test_exact_discovery_served_command_creates_attributed_account · method · L143-L160 — def test_exact_discovery_served_command_creates_attributed_account(self)
