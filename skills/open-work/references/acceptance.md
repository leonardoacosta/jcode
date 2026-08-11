# Acceptance Map

| Normative case | Executable evidence |
| --- | --- |
| Live state differs from cached export | `tests/producers.test.sh` live/cached fixture |
| Live command fails or emits malformed JSON | `tests/producers.test.sh` failure modes |
| No Beads workspace | `tests/producers.test.sh` no-Beads fixture |
| Bucket overlap and proposal/container suppression | `tests/producers.test.sh` classification fixture |
| Labels, comments, progress, truncation | `tests/producers.test.sh` disposition and cap assertions |
| Source-local, read-only helper execution | `tests/producers.test.sh` mode and path assertions |
| Router, modes, headings, capabilities, action ordering | `tests/skill-contract.test.sh` |
| Cross-repository dispatch requires separate confirmation before mutation | `tests/skill-contract.test.sh` exact action-policy assertion |
| Codex interface metadata | `tests/check-interface.py` |
| Equivalent harness reports and no-mutation boundaries | `tests/cross-harness.test.sh` |
| Authored roster, payload policy, and version agreement | `scripts/tests/authored-packages.test.sh` |

Every normative scenario must retain at least one row here and a passing executable assertion before
release. An acceptance prose claim without an executable row is incomplete.
