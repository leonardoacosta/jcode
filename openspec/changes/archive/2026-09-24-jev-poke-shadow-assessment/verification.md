# Verification record

Status: proposal awaiting approval. No application behavior has been implemented or runtime-validated.

## Requirement-to-acceptance mapping

| Requirement | Observable acceptance | Tasks |
| --- | --- | --- |
| Explicit session consent | Public commands, disclosure, status, invalid input, default-off network count | 3.1, 4.3 |
| Non-authoritative recommendations | Queue/todo parity for all labels/errors, waits and overnight exclusions | 3.2, 4.2, 4.3 |
| Deterministic completion hooks | Hook fire order, read-only, Jev call skipped, final-response/completed-goals/user-turn scenarios | 2.0, 3.2, 4.2 |
| Bounded evidence | Captured provider request omits sentinel/raw fields and respects UTF-8/byte cap | 2.1, 4.2 |
| Strict typed assessment | Both provider routes, cache parity, malformed and uncertain abstentions | 1.2, 2.2, 4.2 |
| Bounded asynchronous lifecycle | Deadline, budget, deduplication, two-client lease, stale results, reconnect | 2.2, 2.3, 3.3, 4.3 |

Model-quality acceptance is intentionally separate from mechanics. A mocked provider validates integration, not judgment quality. The first feature cannot authorize active interventions.

## Authoring checks

- `openspec validate jev-poke-shadow-assessment --strict --no-interactive`: PASS.
- Requirement scenario checks: PASS, 6 requirements and 20 WHEN/THEN scenarios.
- Principal source paths: PASS.
- Dependency validator: pending `system-one-service` implementation.

Dependency readiness is blocked: `system-one-service` exists, validates clean, and passes dependency checks independently, but its implementation tasks are unchecked. Do not amend another change to make this one appear ready.

## Live-evaluation verification (2026-09-24)

### Public interfaces checked

| Interface | Check | Method | Result |
|---|---|---|---|
| `/poke shadow on` command string | Present in daemon binary | `strings` grep | PASS: 1 match |
| `/poke shadow off` command string | Present in daemon binary | `strings` grep | PASS: 2 matches |
| `/poke shadow status` command string | Present in daemon binary | `strings` grep | PASS: 1 match |
| `handle_poke_shadow_on` handler | Compiled into binary | `strings` grep | PASS: 1 match |
| `handle_poke_shadow_off` handler | Compiled into binary | `strings` grep | PASS: 1 match |
| `poke_shadow_status` handler | Compiled into binary | `strings` grep | PASS: 1 match |

### Integration boundaries checked

| Boundary | Check | Method | Result |
|---|---|---|---|
| Scheduler co-location | `schedule_shadow_assessment_if_needed` in same module as `schedule_turn_end_followups` | source grep | PASS: both in input.rs |
| Local turn-end | Call site in local.rs | source grep | PASS: line 609 |
| Remote turn-end (done handler) | Call site in server_events.rs | source grep | PASS: line 1063 |
| Remote turn-end (error handler) | Call site in server_events.rs | source grep | PASS: line 1190 |
| Remote turn-end (fallback handler) | Call site in server_events.rs | source grep | PASS: line 1405 |
| `block_on` integration | `block_on` and `schedule_shadow` both in binary | `strings` grep | PASS: confirmed in same binary |

### Unit tests

| Group | Tests | Result |
|---|---|---|
| Result parser | 11 tests | all passed |
| Evidence builder | 5 tests | all passed |
| Assessment lifecycle | 6 tests | all passed |
| Total | 22 tests | all passed |

### Known gap: TUI-only interface

`/poke shadow on` is a TUI slash command, not a headless CLI tool. In `jcode run` mode (headless), the model cannot use TUI commands. This is by design — the proposal specifies TUI sessions only. Verification through `jcode run` confirmed the model attempted `jcode poke shadow on` (CLI subcommand) which does not exist. The model would need an explicit tool for headless shadow toggle, which is out of scope.

### Daemon build verification

- Binary: `3fa760ee7-dirty-222d10491d2a` (built + reloaded by selfdev)
- Shadow symbols: 207 in daemon binary
- evaluate_shadow/run_shadow_assessment: 43 references
