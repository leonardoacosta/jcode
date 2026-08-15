#!/usr/bin/env python3
"""Check an OpenSpec change for factory-intake boundary violations.

The OpenSpec validator checks structure, not design. It accepts a change that
stores task state in chat history and keys records by provider identifiers.
This script checks the anti-patterns from docs/inbox-factory-extensibility.md
that no other tool in the repository catches.

Usage:
  check-intake-boundary.py <path-to-change-dir>   check one OpenSpec change
  check-intake-boundary.py --selftest             verify the checker itself

Exit codes: 0 clean, 1 violations found, 2 usage error.
"""
import pathlib
import re
import sys
import tempfile

# Provider vocabulary that must not appear in provider-neutral specs.
PROVIDER_TERMS = [
    "chat.id", "chat_id", "update_id", "thread_ts", "channel_post",
    "callback_query", "my_chat_member", "message_thread_id",
]

# Phrases describing prohibited designs, from the anti-pattern list.
PROHIBITED_PHRASES = [
    (r"store\s+task\s+state\s+in\s+(the\s+)?chat", "task state stored in chat history"),
    (r"chat\s+(thread|history)\s+(is|as)\s+.{0,20}(source of truth|authority)",
     "chat thread treated as authority"),
]

# Specs under these prefixes are transport adapters, where naming provider
# fields is the entire point. They are exempt from the vocabulary rule only.
ADAPTER_DIR_PREFIXES = ("channel-adapter",)


def is_adapter_spec(path: pathlib.Path) -> bool:
    return any(p.name.startswith(ADAPTER_DIR_PREFIXES) for p in path.parents)


def check(root: pathlib.Path, quiet: bool = False) -> int:
    """Return 0 clean, 1 violations found, 2 usage error."""
    specs = root / "specs"
    if not specs.is_dir():
        if not quiet:
            print(f"no specs/ directory under {root}")
        return 2

    violations = []
    for spec in sorted(specs.rglob("spec.md")):
        low = spec.read_text().lower()
        if not is_adapter_spec(spec):
            for term in PROVIDER_TERMS:
                if re.search(r"\b" + re.escape(term) + r"\b", low):
                    violations.append(
                        f"{spec}: provider term '{term}' in a provider-neutral spec"
                    )
        for pattern, label in PROHIBITED_PHRASES:
            if re.search(pattern, low):
                violations.append(f"{spec}: {label}")

    if not quiet:
        for v in violations:
            print(f"VIOLATION  {v}")
    if violations:
        if not quiet:
            print(
                f"\n{len(violations)} boundary violation(s). "
                "See docs/inbox-factory-extensibility.md"
            )
        return 1
    if not quiet:
        print("intake boundary: clean")
    return 0


NEUTRAL_SPEC = """## ADDED Requirements

### Requirement: Provider-neutral envelope
Intake SHALL normalize payloads into an envelope with a content-derived dedupe key.

#### Scenario: Message delivered
- **WHEN** any adapter delivers a message
- **THEN** intake produces a content-derived dedupe key
"""

LEAKING_SPEC = """## ADDED Requirements

### Requirement: Chat-keyed intake
Intake SHALL key records by `chat_id`, using `update_id` for deduplication.

#### Scenario: Message arrives
- **WHEN** a message arrives
- **THEN** intake records it
"""

CHAT_STATE_SPEC = """## ADDED Requirements

### Requirement: Thread-backed state
Intake SHALL store task state in the chat thread.

#### Scenario: Message arrives
- **WHEN** a message arrives
- **THEN** state lives in the thread
"""


def selftest() -> int:
    """Verify the checker fires on bad input and stays quiet on good input."""
    cases = [
        ("neutral spec passes", NEUTRAL_SPEC, "factory-intake", 0),
        ("provider terms in neutral spec are caught", LEAKING_SPEC, "factory-intake", 1),
        ("provider terms in adapter spec are exempt", LEAKING_SPEC,
         "channel-adapter-telegram", 0),
        ("chat-state phrasing is caught anywhere", CHAT_STATE_SPEC,
         "channel-adapter-telegram", 1),
    ]
    failures = []
    for label, body, capability, expected in cases:
        with tempfile.TemporaryDirectory() as d:
            spec_dir = pathlib.Path(d) / "specs" / capability
            spec_dir.mkdir(parents=True)
            (spec_dir / "spec.md").write_text(body)
            got = check(pathlib.Path(d), quiet=True)
        ok = got == expected
        if not ok:
            failures.append(f"{label}: expected exit {expected}, got {got}")
        print(f"  [{'ok' if ok else 'FAIL'}] {label} (exit {got})")

    with tempfile.TemporaryDirectory() as d:
        got = check(pathlib.Path(d), quiet=True)
    ok = got == 2
    if not ok:
        failures.append(f"missing specs/ dir: expected exit 2, got {got}")
    print(f"  [{'ok' if ok else 'FAIL'}] missing specs/ dir returns usage error (exit {got})")

    for f in failures:
        print(f"SELFTEST FAILURE: {f}")
    print("selftest: all cases passed" if not failures else "selftest: FAILED")
    return 1 if failures else 0


def main() -> int:
    if len(sys.argv) != 2:
        print(__doc__)
        return 2
    if sys.argv[1] == "--selftest":
        return selftest()
    return check(pathlib.Path(sys.argv[1]))


if __name__ == "__main__":
    sys.exit(main())
