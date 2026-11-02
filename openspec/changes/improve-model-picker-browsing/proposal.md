---
execution:
  version: 1
  depends_on: []
---
# Improve model picker browsing and status evidence

Roadmap owner: [P09: TUI/onboarding everyday UX](../../../docs/plans/PRODUCT_ROADMAP.md#p09-tuionboarding-everyday-ux). Supporting: P06, P02.

## Why

`/model` already persists favorites and selection counts, but its implicit ordering mixes current model, favorites, new entries, usage and recommendations. Users cannot explicitly select a provider, show favorites only, or choose alphabetical versus most-used ordering. The existing provider allowlist is configuration, not an interactive filter. A route's `available` flag is not proof that a model is healthy, and provider quota is not a per-model remaining-token balance.

Local evidence and adversarial design are recorded in [design.md](design.md). This is proposal authoring only; no live health calls or implementation are authorized by the artifact.

## What Changes

- Add explicit All/Favorites scope, a composable provider filter, and Alphabetical/Most used ordering to the existing `/model` picker. Preserve text search, current selection indicators, route choice, favorites and set-default actions.
- Reuse current favorites and usage stores. Most used means the existing prospective count of agent turns with a persisted assistant response on a model route, aggregated across reasoning efforts; identify any legacy picker-selection fallback separately. It is not request count, tool-call count, all-time usage, successful-check count or spend.
- Show credential/route availability separately from health evidence. Health is route/model/account-scoped observed success/failure with source, observation age, or unknown. Do not label credentials-present as healthy.
- Show provider/account quota only when the existing usage system can attribute it to the selected route. Include measurement age, window/reset and unknown/stale states. Reuse the info widget limit bar with its existing used-percent convention.
- Keep picker opening and filtering responsive. Use cached data and existing background fetch paths; never fan out inference or billable probes while browsing. Refresh is explicit and reuses supported quota collectors, not a newly invented universal health API.
- Preserve remote-host ownership of credentials, model catalogs and usage. Unsupported telemetry or persistence remains visibly unavailable rather than falling back to the client's account or filesystem.

## Non-scope

- Automatic provider/account switching, quota purchases, credential mutation, or model eligibility changes.
- New provider probe implementations, all-model benchmarking, billing reconciliation or arbitrary HTTP health endpoints.
- Changing model selection behavior in `/agents`, council, or onboarding as an incidental refactor. Shared picker changes must retain their current behavior unless explicitly tested.
- Copying Sandcastle or replacing the model registry, usage service, or favorites persistence.

## Capabilities

### New Capabilities

- `model-picker-browsing`: explicit browsing controls and honest, scoped status evidence.

## Impact and overlap

`crates/jcode-tui/src/tui/app/inline_interactive.rs`, picker types in `crates/jcode-tui/src/tui/mod.rs`, `ui_inline_interactive.rs`, their tests, and existing catalog/usage protocol boundaries where a missing timestamp or route identity must be carried additively.

This is a new P09 feature, not an extension of `mcp-management-experience` (MCP server management), `discover-native-and-mcp-capabilities` (tool discovery), `report-provider-doctor-coverage` (diagnostic coverage), or `reconcile-usage-budget-accounting` (budget accounting). Reuse their established data where available; none is a hard implementation prerequisite for showing an honest unknown state. `show-resolved-worker-model` concerns worker attribution, not picker browsing.

## Resolved defaults

- Scope starts at All, provider at All providers, and sort at Most used. Filters remain stable during refresh and within the client session; do not introduce cross-session UI-preference storage for the first slice.
- Alphabetical means case-insensitive displayed-model-name order with stable route-ID tie-breaks. Most used uses existing tracked-turn counts; search narrows the set without silently changing the selected sort. Display the usage basis when falling back to picker selections.
- All means the base catalog returned by the existing configured loader, not every provider on the internet. Preserve and label its existing zero-match configuration fallback; interactive provider filtering never silently falls back to All. A current model excluded by a filter remains identified in a separate current-selection summary, not injected into mismatching results.
- Favorites remain scoped to the identity the existing store uses. Missing or unavailable favorites remain identifiable and cannot masquerade as selectable healthy routes.
- Use explicit controls that do not consume ordinary search characters or existing shortcuts. Empty filtered results provide reset controls, not a silent fallback to All.
- Health has no numeric score. Credentials configured, observed healthy, degraded, unavailable, stale and unknown are separate facts. Unsupported status is allowed and not treated as success.
- Quota follows the credential/account that would serve the selected route, and missing attribution yields unknown. Never sum unrelated accounts or interpret remaining subscription quota as exact model tokens.

## Verification and handoff prerequisites

Offline unit/integration tests and isolated TUI frames are required. Live provider credentials are not required for the browsing/unknown-state slice. Provider-specific live verification is optional and requires separately authorized inference if it incurs cost. Remote parity requires an isolated client/server fixture; unavailable remote evidence must be reported as blocked rather than inferred from local tests. Final implementation must follow the repository's verification, independent review and archival gates.
