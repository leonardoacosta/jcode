---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P06: Provider/auth/usage-budget reliability](../../../docs/plans/PRODUCT_ROADMAP.md#p06-providerauthusage-budget-reliability)

## Why

`jcode provider-doctor` covers native kinds `OpenAi, Gemini, Cursor, Copilot, Bedrock, Jcode, Azure` (`crates/jcode-provider-doctor/src/provider_e2e.rs:1308-1330`) plus separate Claude and Antigravity probes (`live_provider_probes.rs:465,1084`). Stages that cannot run are marked `Skipped` (`provider_e2e.rs:119`) but there is no single coverage view showing which supported providers have which failure scenarios (expired auth, rate limit, unavailable model, interrupted stream) checked live, offline, or not at all.

## What Changes

- Add `jcode provider-doctor --coverage` emitting a provider × scenario matrix with status `live-pass|live-fail|offline-only|skipped(<reason>)|not-covered`.
- JSON output for CI; table output fits terminal width.

## Non-scope

- Adding new live probes.
- New providers.

## Capabilities

### New Capabilities

- `provider-diagnostic-coverage`

## Impact

`jcode-provider-doctor` crate and `src/cli/provider_doctor.rs`.

## Open decisions

None
