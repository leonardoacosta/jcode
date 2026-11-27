# scripts/verify_light_theme.py

- reply_queries · function · L29-L57 — def reply_queries(master_fd: int, buffer: bytes, bg: str) -> bytes: # bg is "ffff" (white) or "0000" (black) per channel
- run_capture · function · L60-L103 — def run_capture(bg: str, seconds: float = 12.0) -> bytes
- fg_colors · function · L106-L107 — def fg_colors(raw: bytes) -> set[tuple[int, int, int]]
- lum · function · L110-L112 — def lum(c: tuple[int, int, int]) -> float
- main · function · L115-L151 — def main() -> int
