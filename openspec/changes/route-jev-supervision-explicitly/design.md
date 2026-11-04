# Design

## Context

See `proposal.md` for motivation and `specs/jev-supervision-routing/spec.md` for the contract. `JevClient::for_supervision(provider, model)` and `JevShadowEvaluator` already exist. The root `Config` remains in `jcode-base`; agent defaults are reexported from `jcode-config-types`. Public `jcode auth status` is implemented in `src/cli/commands/report_info.rs`.

## Goals / Non-Goals

**Goals:** Keep supervision resolution explicit, locally inspectable, reusable by a later shadow caller, and independent of other purpose selectors.

**Non-Goals:** No caller activation, inference, live entitlement probe, new credential store, provider, or environment override for this route.

## Decisions

1. Add local `JevConfig` and `JevSupervisionConfig` under root `Config.jev`. The optional table and optional fields preserve absence for a typed resolver error rather than inventing defaults or rejecting an otherwise usable config file. Both values remain required to construct a client. This avoids changing unrelated agent defaults or the config-types crate.
2. Expose `JevClient::from_supervision_config(&JevConfig) -> Result<JevClient, NotConfigured>`. Refuse missing, blank, and `auto` values, then reuse `for_supervision`. Do not duplicate transport, endpoint, credential, purpose, timeout, or capability logic. Static reasons do not echo input values or underlying errors. A generic purpose selector returns an error for supervision instead of panicking.
3. Explicit provider selection uses only that provider's existing credential store. A provider credential can already serve several purposes; that is not permission to inspect another purpose's selectors, borrow another provider's credentials, or fall back on failure. Requiring separate supervision-only secrets would introduce an unrequested credential store. The existing Jcode `supervision_jev` entitlement check remains in evaluation, not resolution.
4. Add a `jev_supervision` JSON object and a separate `jev.supervision` plain row to existing auth status. Existing provider rows and `any_available` retain their meaning. `configured` means local client construction succeeded, never live health or entitlement. Both are explicitly `not_checked`. Invalid routes expose a static reason and no raw configured values. Plain output replaces control characters in model/provider fields to prevent row or terminal-control injection; JSON uses serde escaping.

## Risks / Trade-offs

- [Local credential presence cannot prove health or entitlement] → Label both as `not_checked`; keep existing evaluation-time enforcement.
- [Malformed TOML or wrong field types use existing config-load behavior] → Never introduce a supervision default or fallback. Resolution sees an absent route and reports `NotConfigured`.
- [Transport construction errors lose detailed diagnostics] → Use a bounded static local-configuration reason so status never leaks credentials or attacker-controlled error text.
- [Additive plain status row affects consumers assuming only providers] → Use the explicit `jev.supervision` label and leave existing provider row fields unchanged.

## Migration Plan

No automatic migration or activation. Existing configurations leave supervision disabled. A user can explicitly set `[jev.supervision]` with `provider` and `model`, using the selected provider's existing setup. Removing the table disables config-based supervision resolution. This implementation does not alter runtime shadow admission, the shared daemon, or credentials.
