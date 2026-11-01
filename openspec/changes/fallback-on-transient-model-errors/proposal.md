# Proposal

## Why

Model fallback is manual today. After a turn error the TUI only offers a switch that the user must accept with the `fallback_switch` keybinding (`crates/jcode-config-types/src/lib.rs:1046-1048`, wired in `crates/jcode-tui/src/tui/keybind.rs` and `crates/jcode-tui/src/tui/app/tui_lifecycle.rs`). Unattended sessions and swarm workers stall on transient 429/503/network errors even though alternate models are available. Account failover exists (`crates/jcode-base/src/provider/account_failover.rs`) but swaps credentials, not models. Clean-room reference behavior (`scratch/empryo-re/adjacent-behavior.md` §1): ordered per-model fallback list, a model "cools" after 3 failures within 60s, chain iteration capped at 3 cycles, each switch shown in the transcript, auth/config errors never switch.

## What Changes

- Add config `model_fallback`: map of model id to an ordered list of fallback model ids.
- Classify turn errors as transient (429, 529, 408, 5xx, connection reset, stream drop) or permanent (auth, 4xx invalid request, config).
- On a transient error, after existing retries, try the next healthy model in the chain; cooling models (3 failures within 60s) are tried last; stop after 3 full cycles.
- Write a visible transcript line for every switch (`A -> B`, with reason) and every skipped candidate.
- Permanent errors never trigger an automatic switch; the manual `fallback_switch` offer is kept.

## Capabilities

### New Capabilities

- `model-fallback-chain`: Automatic, bounded, visible model fallback on transient errors.

## Impact

`crates/jcode-config-types/src/lib.rs` (new `model_fallback`), `crates/jcode-base/src/provider/` (error classification, health tracking), turn retry loop, TUI transcript notices. Related: `tool-timeout-and-patch-recovery` covers tool timeouts, not model errors; no overlap. Account failover runs first within a model; model fallback only after it is exhausted.

## Open decisions

1. Stickiness after a successful fallback. Recommended: stay on the fallback for the rest of the session until the user picks a model, matching reference behavior and avoiding flapping.
2. Default chain when `model_fallback` is unset. Recommended: none (feature off) to avoid surprise cost changes.
3. Cooldown thresholds configurable? Recommended: fixed 3 failures / 60s / 3 cycles initially.

## Approval

Approved 2026-11-01 with recommended defaults for all open decisions.
