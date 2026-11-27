# scripts/mock_sponsor_service.py

- encode_token · function · L29-L32 — def encode_token(payload: dict[str, Any], secret: bytes) -> str
- decode_token · function · L35-L49 — def decode_token(token: str, secret: bytes, now: int | None = None) -> dict[str, Any]
- connect_db · function · L52-L59 — def connect_db(path: Path) -> sqlite3.Connection
- json_request · function · L62-L66 — def json_request(url: str, payload: dict[str, Any] | None = None) -> dict[str, Any]
- make_handler · function · L69-L166 — def make_handler(db_path: Path, secret: bytes)
- Handler · class · L70-L164 — class Handler(BaseHTTPRequestHandler)
- send_json · method · L71-L77 — def send_json(self, status: int, payload: dict[str, Any]) -> None
- read_json · method · L79-L80 — def read_json(self) -> dict[str, Any]
- do_POST · method · L82-L103 — def do_POST(self) -> None: # noqa: N802
- do_GET · method · L105-L161 — def do_GET(self) -> None: # noqa: N802
- log_message · method · L163-L164 — def log_message(self, _format: str, *_args: object) -> None
- server_url · function · L169-L171 — def server_url(server: ThreadingHTTPServer) -> str
- run_server · function · L174-L177 — def run_server(args: argparse.Namespace) -> None
- cli_signup · function · L180-L182 — def cli_signup(args: argparse.Namespace) -> None
- cli_confirm · function · L185-L186 — def cli_confirm(args: argparse.Namespace) -> None
- cli_account · function · L189-L191 — def cli_account(args: argparse.Namespace) -> None
- parse_args · function · L194-L216 — def parse_args() -> argparse.Namespace
