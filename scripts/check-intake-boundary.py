#!/usr/bin/env python3
"""Check an OpenSpec change for factory-intake boundary violations.

The OpenSpec validator checks structure, not design. It accepts a change that
stores task state in chat history and keys records by provider identifiers.
This script checks the anti-patterns from docs/inbox-factory-extensibility.md
that no other tool in the repository catches.

Usage: check-intake-boundary.py <path-to-change-dir>
Exit: 0 clean, 1 violations found, 2 usage error.
"""
import pathlib
import re
import sys

# Provider vocabulary that must not appear in provider-neutral specs.
PROVIDER_TERMS = [
    "chat.id", "chat_id", "update_id", "thread_ts", "channel_post",
    "callback_query", "my_chat_member", "socket mode", "message_thread_id",
]

# Phrases describing prohibited designs, from the anti-pattern list.
PROHIBITED_PHRASES = [
    (r"store\s+task\s+state\s+in\s+(the\s+)?chat", "task state stored in chat history"),
    (r"chat\s+(thread|history)\s+(is|as)\s+.{0,20}(source of truth|authority)",
     "chat thread treated as authority"),
]

NEUTRAL_DIR_PREFIXES = ("channel-adapter",)


def is_adapter_spec(path: pathlib.Path) -> bool:
    return any(p.name.startswith(NEUTRAL_DIR_PREFIXES) for p in path.parents)


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    root = pathlib.Path(sys.argv[1])
    specs = root / "specs"
    if not specs.is_dir():
        print(f"no specs/ directory under {root}")
        return 2

    violations = []
    for spec in sorted(specs.rglob("spec.md")):
        text = spec.read_text()
        low = text.lower()
        if not is_adapter_spec(spec):
            for term in PROVIDER_TERMS:
                if re.search(r"\b" + re.escape(term) + r"\b", low):
                    violations.append(
                        f"{spec}: provider term '{term}' in a provider-neutral spec"
                    )
        for pattern, label in PROHIBITED_PHRASES:
            if re.search(pattern, low):
                violations.append(f"{spec}: {label}")

    for v in violations:
        print(f"VIOLATION  {v}")
    if violations:
        print(f"\n{len(violations)} boundary violation(s). See docs/inbox-factory-extensibility.md")
        return 1
    print("intake boundary: clean")
    return 0


if __name__ == "__main__":
    sys.exit(main())
