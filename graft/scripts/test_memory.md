# scripts/test_memory.py

- log · function · L33-L37 — def log(msg, color=None)
- log_pass · function · L39-L39 — def log_pass(msg)
- log_fail · function · L40-L40 — def log_fail(msg)
- log_section · function · L41-L41 — def log_section(msg)
- DebugSocketClient · class · L43-L64 — class DebugSocketClient
- __init__ · method · L44-L46 — def __init__(self, socket_path)
- connect · method · L48-L51 — def connect(self)
- close · method · L53-L55 — def close(self)
- send · method · L57-L64 — def send(self, cmd, session_id=None)
- run_tests · function · L66-L209 — def run_tests(client, providers)
- check · function · L69-L76 — def check(condition, msg)
- main · function · L211-L279 — def main()
