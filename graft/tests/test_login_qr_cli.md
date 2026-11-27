# tests/test_login_qr_cli.py

- LoginQrCliTests · class · L22-L92 — class LoginQrCliTests(unittest.TestCase)
- run_login · method · L23-L57 — def run_login(self, flags, *, terminal=False, qr_opt_in=False)
- assert_human_qr · method · L59-L65 — def assert_human_qr(self, result)
- test_browserless_flags_print_qr_without_opt_in · method · L67-L73 — def test_browserless_flags_print_qr_without_opt_in(self)
- test_detected_headless_environment_prints_qr · method · L76-L79 — def test_detected_headless_environment_prints_qr(self)
- test_json_remains_machine_readable_without_qr · method · L81-L92 — def test_json_remains_machine_readable_without_qr(self)
