# scripts/runtime_upgrade_acceptance.py

- Wire · class · L23-L51 — class Wire
- __init__ · method · L24-L29 — def __init__(self, path)
- close · method · L31-L33 — def close(self)
- send · method · L35-L36 — def send(self, kind, request_id, **fields)
- until · method · L38-L51 — def until(self, kind, request_id=None)
- wait_until · function · L54-L65 — def wait_until(callback, timeout=20)
- PrivateDaemon · class · L68-L139 — class PrivateDaemon
- __init__ · method · L69-L100 — def __init__(self, binary, root, version)
- close · method · L102-L110 — def close(self)
- ping · method · L112-L118 — def ping(self)
- info · method · L120-L128 — def info(self)
- upgrade · method · L130-L139 — def upgrade(self, candidate)
- run · function · L142-L233 — def run(binary, artifacts)
- main · function · L236-L249 — def main()
