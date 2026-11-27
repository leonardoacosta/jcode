# scripts/demo_shop.py

- state_path · function · L43-L47 — def state_path() -> Path
- empty_state · function · L50-L51 — def empty_state() -> dict[str, Any]
- load_state · function · L54-L62 — def load_state() -> dict[str, Any]
- save_state · function · L65-L70 — def save_state(state: dict[str, Any]) -> None
- money · function · L73-L74 — def money(value: float) -> str
- totals · function · L77-L81 — def totals(cart: list[str]) -> tuple[float, float, float, float]
- command_reset · function · L84-L87 — def command_reset(_: argparse.Namespace) -> int
- command_search · function · L90-L111 — def command_search(args: argparse.Namespace) -> int
- command_show · function · L114-L128 — def command_show(args: argparse.Namespace) -> int
- command_add · function · L131-L145 — def command_add(args: argparse.Namespace) -> int
- render_cart · function · L148-L159 — def render_cart(cart: list[str]) -> None
- command_cart · function · L162-L165 — def command_cart(_: argparse.Namespace) -> int
- command_checkout · function · L168-L186 — def command_checkout(_: argparse.Namespace) -> int
- command_prepare_checkout · function · L189-L202 — def command_prepare_checkout(args: argparse.Namespace) -> int
- build_parser · function · L205-L241 — def build_parser() -> argparse.ArgumentParser
- main · function · L244-L246 — def main() -> int
