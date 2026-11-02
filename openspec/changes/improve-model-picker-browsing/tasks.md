## 1. Reuse and identity contract

- [ ] 1.1 Trace normal `/model`, onboarding, `/agents`, local/SSH catalog paths and document which picker contexts receive new controls; retain existing behavior elsewhere.
- [ ] 1.2 Define explicit scope/provider/sort state, stable model/route/account keys and selection-count fallback semantics using existing favorites/usage stores. Test multiple routes, accounts and reasoning efforts.

## 2. Browsing controls

- [ ] 2.1 Add All/Favorites and provider filters that compose with search over the existing catalog-loader output; label its zero-match configuration fallback, test its current-route exception, and include explicit reset and current-selection summary. Interactive zero matches must not widen the filter.
- [ ] 2.2 Add Alphabetical/Most used sorting with deterministic name/identity ties; test equal tracked counts with different timestamps and legacy selections, and nonempty search under both modes without changing other picker kinds' fuzzy ranking. Retain filters during async refresh and map focus by stable identity.
- [ ] 2.3 Add discoverable keyboard controls without conflicting with search, route selection, favorite toggle or set-default; test empty/unavailable/removed favorites and unfavoriting the focused or sole row followed immediately by Enter.

## 3. Health and quota evidence

- [ ] 3.1 Reuse passive request outcomes and route availability separately; carry source, scope and timestamp, and expose unknown/stale when evidence is absent. Do not add inference probes.
- [ ] 3.2 Reuse supported provider/account usage collectors and info-widget limit bars; show window/reset/age and explicit attribution failures. Implement only supported mappings; keep others unknown.
- [ ] 3.3 Keep refresh outside rendering/input, reject stale results, preserve remote serving-host ownership and capability-detect older servers without local credential fallback.

## 4. Verification and closeout

- [ ] 4.1 Run `cargo test --profile selfdev -p jcode-tui --lib model_picker` and `cargo test --profile selfdev -p jcode-tui --lib inline_interactive`; cover combined filters, search/sort ties, unavailable favorites, account mismatch, stale/unknown statuses and out-of-order catalog replies.
- [ ] 4.2 Run `cargo test --profile selfdev -p jcode-usage-types`; add focused usage/identity protocol tests wherever additive fields change existing boundaries.
- [ ] 4.3 Capture isolated `debug_socket` tester frames at 60/80/120 columns and monochrome; verify no wrapped chrome, keyboard-only access, Unicode width and unchanged info-widget quota convention. Verify opening/browsing issues no inference calls.
- [ ] 4.4 Verify local and isolated remote client/server fixtures, including older servers with missing telemetry. Record unavailable integration prerequisites rather than claiming parity from local-only checks.
- [ ] 4.5 Run `openspec validate improve-model-picker-browsing --strict` and the shared dependency validator; independently review final diff after fresh gates, resolve findings, commit and archive through the normal workflow. No live inference or publication beyond existing user authorization.
