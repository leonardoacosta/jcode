# scripts/remote/gateway_client.py

- pair · function · L9-L15 — def pair(host, port, code, device_id, device_name)
- health · function · L18-L20 — def health(host, port)
- Gateway · class · L23-L108 — class Gateway
- __init__ · method · L24-L40 — def __init__(self, host, port, token, timeout=30)
- _frame · method · L42-L53 — def _frame(self, op, payload=b"")
- send · method · L55-L59 — def send(self, **req)
- events · method · L61-L85 — def events(self, deadline)
- _take_frame · method · L87-L103 — def _take_frame(self)
- close · method · L105-L108 — def close(self)
