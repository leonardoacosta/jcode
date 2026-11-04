## 1. Implementation
- [x] 1.1 Add provenance fields to `Skill` populated by each loader root in `skill.rs`; track shadowed roots on override
- [x] 1.2 Add `/skill info <name>` detail output and compact root label in selector rows
- [x] 1.3 Show MCP tool server/scope from MCP manager data; "unknown" fallback
- [x] 1.4 Extend remote skill payload additively

## 2. Verification
- [x] 2.1 Tests: override shadowing, symlink, not-found, control text in paths, MCP unknown scope, widths 0/1/20/40/80
- [x] 2.2 `cargo test -p jcode-base skill` and `cargo test -p jcode-tui`
- [x] 2.3 `openspec validate show-skill-and-mcp-provenance --strict`


## Final integrated acceptance, 2026-11-04

Both selected P02.O2/O3 changes passed complete independent review after original-owner amendments. Reviewed base `4bad0bb71d2871ab92e55c66df7c63c05c1168cb`, final implementation diff SHA256 `3ca1b1e89f2be1cb5ab778780f4cd4c7f13806128242c37e25c7c06fd688e61f`. Actual candidate `4bad0bb71-dirty-ce33efe04406`, no shared activation.

Fresh whole-result checks used `scripts/dev_cargo.sh` with `--profile selfdev`: full TUI 2924 passed / 20 ignored; base 2034 passed / 8 ignored plus separately isolated proxy test 1 passed; protocol 107; MCP collision 9; MCP tool 27; client-state tests 20; root all-targets check, module resolution and dependency boundaries passed. Exact commands/results and raw logs: `scratch/p02-final-acceptance/final-recovery-results.json` and `final-recovery-*.log`. Changed-file rustfmt passed. Clippy completed with explicit `--cap-lints warn`, retaining baseline warnings, not a warnings-denied pass. Repository-wide formatting still fails eight unchanged files, not repaired in this scope.

Actual isolated daemon/wire and real MCP stdio fixtures verified mixed native/MCP listing, namespace collision, query/no-match, no lazy start or tool invocation. Bound-handle probes verified ad-hoc unknown and actual global source across pooled project overrides. Real local and loopback-remote PTYs and forced debug frames verified visible tool rows, `skill[project]` selector, all skill detail fields, resolved symlink target/true flag, global/project shadow roots, missing-skill non-activation and actual old-daemon fallback. Public commands: `public-discovery.py`, `public-tui.py`, `review-scope-probes.py`, with exact candidate supplied. Clean final detail root `scratch/p02-tui-7c3e8wno`; independent repeat root `scratch/p02-tui-yy7v3ykx`. This is remote-mode protocol/rendering acceptance, not an actual SSH network test.

Independent reviewer butterfly approved the complete collection, report `scratch/p02-final-acceptance/INDEPENDENT-REVIEW-2.md`. Earlier actual Enter-routing failure, false MCP source attribution and compacted detail rendering were repaired by original developer ox with regressions and fresh whole-result checks. Original discovery owner owl retained attribution and coordinated integration. Earlier full-suite HTTP/timing/scroll failures remain in separate raw logs; unchanged whole-result rerun passed, but exact intermittent causes and routine suite stability are not established. The contaminated `review-fixed-tui.log` is not used as clean public evidence. No paid inference, public release, installation or activation occurred.
