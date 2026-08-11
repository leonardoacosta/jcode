#!/usr/bin/env python3
"""Reject live executable or prescriptive uses of the nonexistent ``bd sync`` CLI.

Historical incident records, CLI non-existence evidence, and explicit prohibitions remain
valid documentation. Current workflow settings, active normative specs, Beads onboarding,
and generated settings templates are governed.
"""
from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path

BD_SYNC = re.compile(r"\bbd\s+sync\b", re.IGNORECASE)
ALLOWED_SPEC_MARKERS = (
    "shall not",
    "does not exist",
    "`bd sync` does not",
    "nonexistent `bd sync",
    "never invoked",
    "never call",
    "never invent",
    "no longer",
    "previously",
    "historical",
    "incident",
    "non-existence",
    "re-verified",
    "verified against",
    "used \"bd sync\"",
    "called `bd sync",
    "not `bd sync",
    "returns no output",
    "sync-equivalent",
    "pre-202",
    "git log --oneline | grep",
)


def command_values(value: object):
    if isinstance(value, dict):
        for key, child in value.items():
            if key == "command" and isinstance(child, str):
                yield child
            yield from command_values(child)
    elif isinstance(value, list):
        for child in value:
            yield from command_values(child)


def scan_settings(root: Path, label: str) -> list[str]:
    findings: list[str] = []
    if not root.exists():
        return findings
    for path in sorted(root.rglob("*.settings.json.tmpl")):
        try:
            data = json.loads(path.read_text())
        except (OSError, json.JSONDecodeError) as error:
            findings.append(f"{label}:{path}: invalid settings template: {error}")
            continue
        for command in command_values(data):
            if BD_SYNC.search(command):
                findings.append(f"{label}:{path}: executable bd sync: {command}")
    return findings


def scan_prescriptive_markdown(root: Path, patterns: tuple[str, ...]) -> list[str]:
    findings: list[str] = []
    paths = sorted({path for pattern in patterns for path in root.glob(pattern)})
    for path in paths:
        lines = path.read_text(errors="replace").splitlines()
        for number, line in enumerate(lines, 1):
            if not BD_SYNC.search(line):
                continue
            lowered = line.lower()
            if "bd sync --help" in lowered or any(marker in lowered for marker in ALLOWED_SPEC_MARKERS):
                continue
            findings.append(f"cc:{path}:{number}: prescriptive bd sync: {line.strip()}")
    return findings


def scan_beads_guidance(root: Path) -> list[str]:
    findings: list[str] = []
    for relative in (Path(".beads/README.md"), Path(".beads/config.yaml")):
        path = root / relative
        if not path.exists():
            continue
        for number, line in enumerate(path.read_text(errors="replace").splitlines(), 1):
            if BD_SYNC.search(line):
                findings.append(f"cc:{relative}:{number}: live Beads guidance: {line.strip()}")
    return findings


def scan(cc_root: Path, pi_root: Path | None, dist_root: Path | None) -> list[str]:
    findings = scan_settings(cc_root / "templates" / "workflow", "cc")
    findings.extend(
        scan_prescriptive_markdown(
            cc_root,
            ("openspec/specs/**/*.md", "docs/commands/**/*.md", "commands/**/*.md"),
        )
    )
    findings.extend(scan_beads_guidance(cc_root))
    if pi_root is not None:
        findings.extend(scan_settings(pi_root, "pi-source"))
        findings.extend(scan_prescriptive_markdown(pi_root, ("**/*.md",)))
    if dist_root is not None:
        findings.extend(scan_settings(dist_root, "generated"))
        findings.extend(scan_prescriptive_markdown(dist_root, ("**/*.md",)))
    return findings


def main() -> int:
    parser = argparse.ArgumentParser()
    parser.add_argument("--cc-root", type=Path, required=True)
    parser.add_argument("--pi-root", type=Path)
    parser.add_argument("--dist-root", type=Path)
    args = parser.parse_args()
    findings = scan(args.cc_root.resolve(), args.pi_root, args.dist_root)
    for finding in findings:
        print(finding, file=sys.stderr)
    print(len(findings))
    return 1 if findings else 0


if __name__ == "__main__":
    raise SystemExit(main())
