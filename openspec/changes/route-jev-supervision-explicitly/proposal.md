---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P03: Jev semantic poke continuation](../../../docs/plans/PRODUCT_ROADMAP.md#p03-jev-semantic-poke-continuation)

Selected outcome: P03.O2, requirement “Explicit supervision route” and both scenarios, tasks 1–6 in file order. Routing configuration and status only, no inference or runtime activation.

## Why

`JevPurpose::Supervision` exists with capability `supervision_jev` (`crates/jcode-base/src/jev.rs:55`, `:74`) and `JevClient::for_supervision(provider, model)` requires an explicit non-`auto` provider (`:257`), but its selector paths are `unreachable!` (`:87`, `:95`) and no code outside `jev.rs` calls `for_supervision`. No configuration exists, so supervision cannot be routed without reusing another purpose's route or credentials.

## What Changes

- Add config `[jev.supervision] provider` and `model` (both required; no default, no `auto`).
- Resolve a supervision client only from that config; missing or invalid config returns a typed `NotConfigured`, never Memory/Browser/Voice routing.
- Show supervision route status in existing provider status output.

## Non-scope

- Using supervision output (see `admit-semantic-poke-in-shadow`).
- New providers or credential storage.

## Capabilities

### New Capabilities

- `jev-supervision-routing`: explicit supervision provider routing.

## Impact

`crates/jcode-base/src/jev.rs`, config types, status output; tests in `jcode-base`.

## Resolved defaults

None.
