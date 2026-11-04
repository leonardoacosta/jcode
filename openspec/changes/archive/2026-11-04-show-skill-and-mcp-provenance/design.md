# Design

## Context

See proposal.md and specs/capability-provenance/spec.md. Execution base is `4bad0bb71d2871ab92e55c66df7c63c05c1168cb`. `SkillRegistry` loads global plugins before explicit global roots and overlays project roots per session. `merge_overlay` currently extends a name map, losing override history. Typed selector descriptions and optional remote metadata already exist. Preserve those boundaries.

## Decisions

- Record provenance when loading, not by classifying canonical paths after loading. Pass the fixed source label through each existing loader: `plugin`, `global`, `agents`, `project`. Keep the existing discovery path for loading/invocation; add canonical resolved-path metadata and a symlink indication. Unknown metadata remains unknown, especially across legacy peers.
- On each override, preserve displaced root labels and their accumulated shadow history, including global-to-project overlay merges. Do not change precedence, ancestor traversal, cycle detection, reload behavior or session isolation. Canonicalization failure must not turn display metadata into an invocation failure or a fabricated resolved target.
- Add exact `/skill info <name>` command handling before generic skill activation. This path reports metadata without activating the skill. Reuse existing info rendering for multiline detail and selector display-cell ellipsis for compact fixed-root labels. Sanitize control characters at display boundaries without rewriting invocation identities or filesystem paths used by the loader.
- Extend remote skill payload additively with optional provenance. Server session workspace owns remote metadata; never substitute client-local skill sources. Legacy names/descriptions remain usable and unavailable provenance displays `unknown`.
- MCP details use the owning server and verified effective config scope when the manager exposes it. Coordinate with the discovery owner's read-only tool snapshot rather than duplicate tool listing. Existing `/mcp` global-name membership is not proof of scope when project config overrides the same server name. Track effective source at config merge, or leave genuinely unavailable scope unknown. Do not expose credentials/config contents or infer trust from source labels.

## Risks and verification

Test project/global overrides and accumulated shadow roots, symlinked directories and targets, missing skill without activation, control-bearing paths, fresh remote metadata and legacy unknowns, MCP known/unknown scope including same-name project overrides, and compact widths 0/1/20/40/80. Run task-specified base skill tests and full TUI tests plus affected protocol/app-core integration, build and lint checks. Coordinator will verify actual isolated local/remote-mode TUI detail and selector output against the final binary, explicitly distinguishing loopback remote mode from actual SSH transport.

## Execution ownership and closeout

The provenance owner may first edit disjoint skill-loader/tool files. Discovery owns the first serialized window on shared protocol/server/TUI files; transfer exact file claims before provenance integration. Coordinator owns shared Git, final whole-result evidence, independent review/amendment loop, verified tasks, implementation commit, native spec synchronization/archive, roadmap evidence and authorized non-force origin/main persistence. No installations/removals, trust enforcement, migrations, RC activation, public release/tag or paid inference.
