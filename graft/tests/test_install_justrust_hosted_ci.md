# tests/test_install_justrust_hosted_ci.py

- HostedClientBootstrapTests · class · L12-L84 — class HostedClientBootstrapTests(unittest.TestCase)
- setUp · method · L13-L39 — def setUp(self)
- write_tool · method · L41-L44 — def write_tool(self, name, content)
- run_installer · method · L46-L48 — def run_installer(self, **extra)
- test_exact_source_pin_locked_compiler_and_embedded_helper · method · L50-L63 — def test_exact_source_pin_locked_compiler_and_embedded_helper(self)
- test_wrong_revision_never_compiles_or_activates · method · L65-L69 — def test_wrong_revision_never_compiles_or_activates(self)
- test_backport_failure_never_compiles_or_activates · method · L71-L76 — def test_backport_failure_never_compiles_or_activates(self)
- test_compiler_failure_never_activates · method · L78-L80 — def test_compiler_failure_never_activates(self)
- test_missing_capability_never_activates · method · L82-L84 — def test_missing_capability_never_activates(self)
