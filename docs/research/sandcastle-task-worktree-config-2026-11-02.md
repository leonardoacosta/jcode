# Sandcastle, task admission and worktree configuration

Date: 2026-11-02. Research and design direction only; no runtime configuration, worktree, sandbox, or provider was changed.

## Findings

1. Jcode has global configuration at `~/.jcode/config.toml`, or `$JCODE_HOME/config.toml`, and environment overrides (`crates/jcode-base/src/config.rs:1-4`). The inspected `Config` and `AgentsConfig` expose agent/provider/safety settings, but do not contain a typed global worktree or task-triage policy (`config.rs:598`, `crates/jcode-config-types/src/lib.rs:573`). This is a source finding about the inspected schema, not proof that no separate launcher implements worktree behavior.
2. A worktree mechanism already exists in `crates/jcode-sdk/src/worktrees.rs`. Its creation path derives a sibling `<repo>-worktrees` root (`:104-131`), which does not match this user's `~/worktrees/<repo-key>/<creator>/<task>--<unique-id>` policy. A global policy should govern reusable mechanisms rather than introducing another worktree allocator. No allocator was invoked.
3. Restart admission should be task-driven, not inferred solely from an overnight wake time. Keep finite task/resource limits, explicit authorization, cancellation, current ownership and dependencies. Resume eligibility is not permission to start unrelated tasks or create worktrees.

## Sandcastle in the original sources

Three separate observations must not be collapsed into a claim of original inclusion:

- The historical external reference register at commit `a1888d2e8`, `docs/factory/sources-and-limitations.md`, lists Anthropic, Google ADK, OpenAI, DORA and NIST references, not Sandcastle. That historical register is not proven to be the user's entire original list.
- The current roadmap/research material inspected does not establish a Sandcastle binding. Some older dossier/exploration paths returned by the code index are not present in this checkout; indexed names alone are not readable evidence.
- A prior Firecrawl snapshot of `mattpocock/sandcastle` exists from Oct 28 (`skull-mine-mattpocock-sandcastle.json`), together with Docker/completion-related snapshots. This proves it was previously fetched, not that it was admitted to the original roadmap source register or mined across all phases.

Conclusion: prior research touched Sandcastle, but original-list inclusion is not established. Register it explicitly in this research record and preserve the distinction rather than answering an unsupported yes.

## Public reference mechanisms

| Reference | Observed mechanism | Useful Jcode adaptation | Do not copy blindly |
| --- | --- | --- | --- |
| [Sandcastle configuration](https://github.com/mattpocock/sandcastle#configuration) | Per-repository `.sandcastle/`; initialized scaffolding and a customizable sandbox Dockerfile | Versioned project setup recipe distinct from user-wide policy and ephemeral run state | A TypeScript executable config or Docker dependency is not necessary for a Rust/TOML task policy |
| [Sandcastle `.sandcastle/`](https://github.com/mattpocock/sandcastle/tree/main/.sandcastle) and [runner](https://github.com/mattpocock/sandcastle/blob/main/.sandcastle/run.ts) | Project runner and separate plan, implement, review and merge prompts | Explicit task stages and testable transitions; human handoff as an outcome, not an exception loop | A project's example runner does not prove a universal built-in triage/config service |
| [Sandcastle hooks](https://github.com/mattpocock/sandcastle#hooks) | Host versus sandbox setup, ordered worktree preparation, hook timeouts, cancellation and fail-fast setup | Declare execution location; setup errors block admission; cancel setup with its task | Never execute repository-provided host hooks or copy secrets just because a config file exists |
| [Sandcastle reuse ADR](https://github.com/mattpocock/sandcastle/blob/main/docs/adr/0003-reuse-worktree-by-default.md) | Worktree reuse has branch freshness and dirty-state implications | Reuse only a matching owned worktree after verifying branch/base, dirtiness and active consumers | No reset/clean/pull into user-owned or dirty worktrees as automatic recovery |
| [Worktrunk config](https://worktrunk.dev/config/) | User TOML for path preferences, project TOML for shared hooks; configuration inspection | Separate machine-local policy from versioned setup; expose effective settings and provenance | Its example `.worktrees/` path is not this user's approved root |
| [Workmux](https://github.com/raine/workmux#configuration) | Global config plus project overrides, pane layout and agent-status integration | Shared task/worktree identity across native status and terminal pane integration | Do not add a competing pane manager or automatically install hooks/skills |

## Recommended design direction

- Extend existing Jcode TOML loading with typed global task and worktree policy. Candidate groups are `[tasks]` and `[worktrees]`; these names and fields are proposed, not implemented settings.
- Keep project setup under the existing `.jcode` configuration convention rather than creating `.sandcastle` in Jcode projects. First verify supported project config loading and trust before selecting a new path or precedence rule.
- Task policy describes admission, resource/concurrency limits, dependency/readiness checks, retry/recovery and human escalation. An optional deadline is a constraint, not the source of task authorization.
- Worktree policy describes allowed root/template, explicit creation authority, ownership and reuse checks, setup commands and retention. Default to no automatic creation/deletion and preserve current policy until the user deliberately configures a bounded alternative.
- Repository config may narrow user policy and declare setup, but cannot grant itself host execution, broader filesystem access, publication permission or more budget. Global defaults also do not imply this session was authorized to create a worktree.
- Reconcile persisted tasks with current policy on restart. Retain the narrower of current policy and original task authorization, reject duplicate live owners, and expose `ready`, `waiting`, `blocked`, `needs-human`, `running` and terminal outcomes with reasons.
- Keep OpenSpec `tasks.md` authoritative for implementation tasks. A runtime task registry records attempts/ownership/evidence, not a second editable planning ledger.

## Evidence and limits

Firecrawl CLI v1.23.3 was authenticated. Public search/scrape was bounded to Sandcastle, Worktrunk and Workmux pages; existing shared cache was reused before fetching new URLs. No authenticated target pages, private source uploads, browser interaction, install commands or project setup scripts were executed. Public URLs are references, not authority to change local policy.

Sandcastle README evidence is an Oct 28 cached snapshot; `.sandcastle/`, its runner, the reuse ADR and the comparable-tool configuration pages were retrieved on Nov 2. These are documentation/source observations, not live acceptance tests. Twenty requested GPT-6.1 Sol Omni miners are assigned in pairs to P01-P10; their phase-specific findings are separate and must be reviewed and deduplicated before becoming proposals. Provider 503 failures are a coverage limitation, not evidence of absent features.
