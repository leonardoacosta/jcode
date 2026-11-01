# Tasks

## 1. Config and classification

- [ ] 1.1 Add `model_fallback: HashMap<String, Vec<String>>` to config types, dropping blanks and self-references.
- [ ] 1.2 Add a provider error classifier returning `Transient` or `Permanent` (auth, invalid request, config are permanent).

## 2. Health and chain

- [ ] 2.1 Track per-model failure timestamps (bounded history) and reset on success.
- [ ] 2.2 Mark a model cooling at 3 failures within 60s; order candidates healthy-first, cooling last, then config order.
- [ ] 2.3 Iterate the chain at most 3 cycles, then surface the original error.

## 3. Turn integration and UX

- [ ] 3.1 After existing retries and account failover, switch to the next candidate only for transient errors.
- [ ] 3.2 Emit a transcript notice for each switch and each skipped candidate, including the reason class.
- [ ] 3.3 Keep the manual `fallback_switch` offer for permanent errors.

## 4. Verification

- [ ] 4.1 Unit tests: classifier, cooling window, chain ordering, 3-cycle cap, permanent error never switches.
- [ ] 4.2 Integration test with a mock provider returning 503 then success on fallback, asserting transcript notice.
- [ ] 4.3 Run `openspec validate fallback-on-transient-model-errors --strict`.
