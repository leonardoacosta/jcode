---
execution:
  version: 1
  depends_on: [classify-provider-auth-failures]
---
# Proposal

Roadmap owner: [P09: TUI/onboarding everyday UX](../../../docs/plans/PRODUCT_ROADMAP.md#p09-tuionboarding-everyday-ux)

## Why

First-run onboarding (`crates/jcode-tui/src/tui/ui_onboarding.rs:138`, `:284`) covers credential import and telemetry, but a user who imports nothing reaches chat with no explicit provider state. The actionable next step exists only in the separate `jcode provider doctor` CLI (`src/cli/provider_doctor.rs:179`). Auth failures therefore surface late, on first prompt.

## What Changes

- At the end of onboarding, show the selected resolved provider/model as configured/untested when credentials are only present, or missing credentials plus the supported `jcode login --provider <provider>` remedy. Presence is not working/healthy evidence. Reuse shared failure classification and doctor messages without probing other accounts.
- On a first-prompt auth failure, show the same next step inline instead of only the raw provider error.
- Rows obey display-cell truncation.

## Non-scope

New login flows, provider additions, credential storage changes.

## Capabilities

### New Capabilities

- `first-run-provider-setup`: show the resolved provider or an actionable login next step at the end of first run and on first-prompt auth failure.

## Impact

`ui_onboarding.rs`, provider doctor message helper (shared, not duplicated), TUI auth-error rendering. No data crosses a provider boundary beyond existing requests.

## Resolved defaults

1. Probe credentials live at onboarding end or only check presence? Options: live probe (accurate, adds network and latency); presence only (instant, can be stale). Selected default: presence only, with live failure handled by the first-prompt path, to avoid network calls during onboarding.
