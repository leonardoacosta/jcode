# scripts/phone-server/wake-lambda.py

- response · function · L20-L30 — def response(status, body="", content_type="application/json", headers=None)
- json_response · function · L33-L34 — def json_response(status, obj)
- authorized · function · L37-L41 — def authorized(event)
- instance_state · function · L44-L46 — def instance_state(ec2)
- ssm_online · function · L49-L54 — def ssm_online(ssm)
- fetch_pair_code · function · L57-L102 — def fetch_pair_code(ssm)
- landing_page · function · L105-L153 — def landing_page()
- handler · function · L156-L199 — def handler(event, context)
