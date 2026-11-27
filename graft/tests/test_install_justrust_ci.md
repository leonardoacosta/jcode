# tests/test_install_justrust_ci.py

- InstallerTests · class · L14-L165 — class InstallerTests(unittest.TestCase)
- setUp · method · L15-L38 — def setUp(self)
- stub · method · L40-L43 — def stub(self, name, text)
- fixture · method · L45-L61 — def fixture(self, target="x86_64-unknown-linux-musl", status=0, bad=False, missing=False)
- run_install · method · L63-L65 — def run_install(self)
- test_all_targets · method · L67-L91 — def test_all_targets(self)
- assert_preserved · method · L93-L102 — def assert_preserved(self, **fixture_args)
- test_checksum_failure_preserves_existing · method · L104-L107 — def test_checksum_failure_preserves_existing(self)
- test_version_failure_preserves_existing · method · L109-L110 — def test_version_failure_preserves_existing(self)
- test_missing_member_preserves_existing · method · L112-L113 — def test_missing_member_preserves_existing(self)
- test_unsupported_host · method · L115-L118 — def test_unsupported_host(self)
- test_override_requires_opt_in · method · L120-L123 — def test_override_requires_opt_in(self)
- test_default_directory_without_github_path · method · L125-L131 — def test_default_directory_without_github_path(self)
- test_shasum_fallback · method · L133-L144 — def test_shasum_fallback(self): # Isolate PATH so sha256sum is genuinely unavailable.
- test_malformed_checksum_preserves_existing · method · L146-L156 — def test_malformed_checksum_preserves_existing(self)
- test_pinned_production_url · method · L158-L165 — def test_pinned_production_url(self)
