---
execution:
  version: 1
  depends_on: []
---
# Configure task admission and worktree policy

Roadmap owner: [P05: Durable overnight, scheduling and recovery](../../../docs/plans/PRODUCT_ROADMAP.md#p05-durable-overnight-scheduling-and-recovery). Supporting: P04, P10.

## Why

The user wants durable work selected by configured task readiness and authorization rather than an overnight time window. Existing global TOML loading is reusable, but the inspected `Config` and `AgentsConfig` do not define a typed task-triage/worktree policy. The SDK worktree creator derives a sibling `<repo>-worktrees` root, which conflicts with the user's global `~/worktrees/<repo-key>/<creator>/<task>--<unique-id>` convention.

Evidence and public pattern comparisons: [Sandcastle/task/worktree research](../../../docs/research/sandcastle-task-worktree-config-2026-11-02.md). This change defines native configuration and admission contracts; it does not install Sandcastle, create a worktree, activate unattended work, or change the user's live TOML.

## What Changes

- Extend existing Jcode configuration with typed `[tasks]` and `[worktrees]` settings, defaults, validation and effective-value provenance. Keep machine-local policy in existing global TOML; do not introduce executable TypeScript config or a competing root configuration file.
- Task settings select admission/restart behavior and existing finite concurrency/retry/resource limits. A disabled-by-default unattended admission mode evaluates authorization, task scope, dependencies, cancellation, remaining limits and required workspace readiness, producing `ready`, `waiting`, `blocked`, or `needs-human` with reasons. An optional deadline is a constraint, not task authorization.
- Worktree settings define allowed root/layout, explicit creation authority, reuse ownership/freshness checks and manual cleanup. Default creation requires explicit task-level permission; cleanup never automatically removes worktrees or branches. A deliberate user-scoped policy may authorize narrowly bounded future creation, but repository content cannot grant that authority.
- Route Jcode-owned worktree creation through one policy resolver, reusing the SDK implementation rather than adding another allocator. All callers must validate repository identity, creator identity, canonical destination, matching branch/base and active consumers before creation or reuse.
- Permit project-level task/setup declarations only through the repository's verified existing config mechanism. Project policy may narrow user restrictions, never widen filesystem access, spend limits, host-hook execution or publication authority. Reject unsupported project overrides with an explanation.
- Expose a read-only effective-policy view and admission reasons so the user can distinguish a configured default, a task's persisted authorization and a runtime denial. OpenSpec `tasks.md` stays the implementation ledger; runtime state records attempts, ownership and evidence only.

## Non-scope

- Auto-running all backlog proposals, new issue trackers, new schedulers or another editable task ledger.
- Enabling unattended execution in the user's current config or granting this session worktree permission.
- Automatic reset/clean/stash, branch switching, force pushes, worktree deletion, migrations or publication.
- Running repository setup scripts or adding new hook execution semantics. Existing `add-lifecycle-hooks` owns execution; future setup integration must retain its trust boundary.
- Replacing existing budget enforcement or using a semantic model as the authorization authority.

## Capabilities

### New Capabilities

- `task-admission-policy`: typed configuration and explainable admission decisions.
- `worktree-policy`: globally governed worktree placement, permission and safe reuse.

## Impact and related changes

`crates/jcode-base/src/config.rs`, `crates/jcode-config-types/src/lib.rs`, `crates/jcode-sdk/src/worktrees.rs`, existing native task/overnight admission and restart callers. Inventory all actual creation callers before changing shared behavior.

`admit-overnight-tasks-on-evidence` remains the evidence-aware execution consumer; `reconcile-overnight-runs-after-restart` consumes the new admission contract. This change owns configuration and deterministic decisions, not their whole executor. Existing global policy and user decisions constrain defaults.

## Resolved defaults

- No unattended admission or automatic worktree creation is enabled by adding configuration support. Unknown authorization, ownership or repository identity fails closed with an actionable reason.
- Reuse the current global config location and supported precedence, but project config cannot broaden user-scoped restrictions. Record source/provenance rather than silently accepting a conflicting override.
- The personal default root/layout follows `~/worktrees/<repo-key>/<creator>/<task>--<unique-id>`; creator and repository keys come from verified identity, not invented labels or credential-bearing remote URLs. Other installations may explicitly choose their own root within their policy.
- Model/agent recommendations can classify work, but deterministic checks own permissions, limits, cancellation, path safety and dispatch eligibility. Re-evaluate admission after relevant policy/dependency changes and before restart.

## Authoring and handoff boundary

The exact Rust/TOML field names and effective-policy CLI syntax are implementation choices to finalize against the existing parser/commands. Acceptance below fixes their behavior without pretending they already exist. Offline config/path/admission fixtures require no provider credentials. Live unattended execution, setup hooks and user-owned worktree mutations are outside authoring and require their normal scoped authorization. No implementation has started.

Selected roadmap outcome: `P05.O1`. Exact requirement/task binding and acceptance are recorded in [the selected first-wave table](../../../docs/plans/PRODUCT_ROADMAP.md#sequencing-clarification-and-selected-first-wave-bindings).
