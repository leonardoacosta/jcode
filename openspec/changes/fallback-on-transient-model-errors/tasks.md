# Tasks

## 1. Config and classification

- [x] 1.1 Add `model_fallback: HashMap<String, Vec<String>>` to config types, dropping blanks and self-references.
- [x] 1.2 Add a provider error classifier returning `Transient` or `Permanent` (auth, invalid request, config are permanent).

## 2. Health and chain

- [x] 2.1 Track per-model failure timestamps (bounded history) and reset on success.
- [x] 2.2 Mark a model cooling at 3 failures within 60s; order candidates healthy-first, cooling last, then config order.
- [x] 2.3 Iterate the chain at most 3 cycles, then surface the original error.

## 3. Turn integration and UX

- [x] 3.1 After existing retries and account failover, switch to the next candidate only for transient errors.
- [ ] 3.2 Emit a transcript notice for each switch and each skipped candidate, including the reason class.
  - Partial: notices are logged and emitted as `StatusDetail` stream events before the fallback stream; not yet rendered as persistent transcript lines in the TUI.
- [x] 3.3 Keep the manual `fallback_switch` offer for permanent errors.

## 4. Verification

- [x] 4.1 Unit tests: classifier, cooling window, chain ordering, 3-cycle cap, permanent error never switches.
- [ ] 4.2 Integration test with a mock provider returning 503 then success on fallback, asserting transcript notice.
- [x] 4.3 Run `openspec validate fallback-on-transient-model-errors --strict`.
