#!/usr/bin/env python3
"""Verify every spec scenario has at least one executable assertion.

Cross-references scenario names in the OpenSpec change against check()
calls in the acceptance models. A scenario with no assertion is a claim
nobody has executed.

Usage: check-scenario-coverage.py    (exit 0 full coverage, 1 gaps, 2 error)
"""
import pathlib
import re
import sys

REPO = pathlib.Path(__file__).resolve().parent.parent
SPECS = (REPO / "source/jcode/openspec/changes"
         / "add-factory-intake-capability/specs")
MODELS = [REPO / "scripts/intake-acceptance-model.py",
          REPO / "scripts/adapter-acceptance-model.py"]


def asserted_scenarios():
    names = set()
    for m in MODELS:
        if not m.is_file():
            continue
        text = m.read_text()
        names |= set(re.findall(
            r'check\(\s*"[^"]+"\s*,\s*"([^"]+)"', text))
        names |= set(re.findall(
            r'check\(\s*"[^"]+"\s*,\n\s*"([^"]+)"', text))
    return names


def main():
    if not SPECS.is_dir():
        print(f"specs not found: {SPECS}")
        return 2
    covered = asserted_scenarios()
    gaps = 0
    total = 0
    for spec in sorted(SPECS.rglob("spec.md")):
        text = spec.read_text()
        scenarios = re.findall(r"^#### Scenario: (.+)$", text, re.M)
        missing = [s for s in scenarios if s not in covered]
        total += len(scenarios)
        gaps += len(missing)
        print(f"\n{spec.parent.name}: "
              f"{len(scenarios) - len(missing)}/{len(scenarios)} scenarios asserted")
        for s in missing:
            print(f"  UNASSERTED  {s}")
    print(f"\n{total - gaps}/{total} scenarios have executable assertions")
    if gaps:
        print(f"{gaps} scenario(s) claim behavior nobody has executed")
        return 1
    return 0


if __name__ == "__main__":
    sys.exit(main())
