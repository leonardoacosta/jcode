# scripts/phone-server/test_wake_lambda.py

- FakeEc2 · class · L22-L31 — class FakeEc2
- __init__ · method · L23-L25 — def __init__(self, state="running")
- describe_instances · method · L27-L28 — def describe_instances(self, **_kwargs)
- start_instances · method · L30-L31 — def start_instances(self, **_kwargs)
- InvocationDoesNotExist · class · L34-L35 — class InvocationDoesNotExist(Exception)
- FakeSsm · class · L38-L53 — class FakeSsm
- exceptions · class · L39-L40 — class exceptions
- describe_instance_information · method · L42-L43 — def describe_instance_information(self, **_kwargs)
- send_command · method · L45-L46 — def send_command(self, **_kwargs)
- get_command_invocation · method · L48-L53 — def get_command_invocation(self, **_kwargs)
- WakeLambdaTests · class · L56-L94 — class WakeLambdaTests(unittest.TestCase)
- event · method · L57-L64 — def event(self, method, *, token=None, body=None, query=None)
- test_landing_page_never_embeds_token · method · L66-L70 — def test_landing_page_never_embeds_token(self)
- test_legacy_query_redirects_token_into_fragment · method · L72-L75 — def test_legacy_query_redirects_token_into_fragment(self)
- test_post_requires_bearer_token · method · L77-L79 — def test_post_requires_bearer_token(self)
- test_status_uses_ec2_and_ssm_health · method · L81-L88 — def test_status_uses_ec2_and_ssm_health(self)
- test_pairing_parses_ansi_output_and_returns_tailnet_host · method · L90-L94 — def test_pairing_parses_ansi_output_and_returns_tailnet_host(self)
