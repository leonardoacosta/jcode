## Why

Jcode now has durable initiative persistence, a native daemon/API authority, TUI initiative surfaces, and a custom SolidStart Command Center. The current documentation and implementation make these boundaries discoverable, but the product language still allows the web UI to be mistaken for an external or separate system. We need to formalize the native authority, make the web UI an explicit projection, and provide a documented extension path for future initiative workflows without creating parallel persistence.

## What Changes

- Document the native Jcode initiative and Command Center architecture as a first-class product surface.
- Define app-core goal persistence as the sole authority for initiatives, milestones, checkpoints, revisions, and idempotency.
- Define the Command Center web application as a browser projection and interaction client, not an independent data store.
- Align TUI and web terminology, capability states, and initiative lifecycle semantics.
- Add an adoption and extension guide for building new initiative views on existing DTOs, commands, projections, and security boundaries.
- Add an explicit comparison of native Jcode surfaces versus the custom web presentation so future work starts with discovery rather than duplicate UI construction.
- Preserve degraded behavior when the experimental web feature, Orca adapter, scheduler, or browser host is unavailable.

## Capabilities

### New Capabilities

- `native-initiative-command-center`: Documented native initiative authority and extension contract across app-core, TUI, daemon API, and web projections.

### Modified Capabilities

- `command-center`: Clarify that the SolidStart UI is a Jcode-owned projection over durable app-core state and define supported mutation, replay, reconnect, and degraded-state behavior.
- `initiative`: Align TUI, tool, API, and web terminology and document the durable lifecycle contract.

## Impact

- Updates Command Center, initiative, architecture, and contributor documentation.
- May add a small native-surface capability matrix and UI extension guide under `docs/`.
- May update public DTO/command documentation and generated contract comments without changing wire compatibility.
- Does not add a second database, replace the TUI, or make Orca the source of truth.
- Does not require new Installfest-specific UI in this change.
- Existing `apps/command-center` remains the web presentation unless a later proposal replaces it after an explicit comparison.

## Preconditions

- Existing initiative persistence and Command Center contracts remain available.
- The current Command Center vertical slice and native initiative tests are green.
- No active change owns the same documentation or contract surfaces.

## Done Means

- A new contributor can identify the authoritative initiative store, mutation path, web projection path, and external Orca boundary from one documented entry point.
- TUI, tool, API, and web UI use the same lifecycle vocabulary.
- The web UI has no independent persistence path.
- Extension guidance identifies where new views and commands belong and which security/revision/idempotency contracts are mandatory.
- Focused documentation and contract checks pass, with no regression to existing initiative or Command Center workflows.

## Testing

- Run the focused initiative and Command Center Rust tests.
- Run generated contract drift checks.
- Run the existing Command Center component and browser tests where available.
- Run strict OpenSpec validation:
  `openspec validate formalize-native-initiative-command-center --strict --no-interactive`.
