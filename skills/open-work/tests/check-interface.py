#!/usr/bin/env python3
"""Validate the portable Codex-facing interface metadata for open-work."""
from pathlib import Path
import re
import sys


def main() -> int:
    root = Path(sys.argv[1]).resolve()
    skill = (root / "SKILL.md").read_text()
    metadata = (root / "agents" / "openai.yaml").read_text()
    checks = {
        "skill name": re.search(r"(?m)^name: open-work$", skill),
        "display name": 'display_name: "Open Work"' in metadata,
        "default prompt": 'default_prompt: "Use $open-work' in metadata,
        "interactive default": "interactive mode" in metadata,
        "report mode": "report mode" in metadata,
    }
    failed = [name for name, result in checks.items() if not result]
    if failed:
        print("FAIL: " + ", ".join(failed), file=sys.stderr)
        return 1
    print("PASS: Open Work interface metadata")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
