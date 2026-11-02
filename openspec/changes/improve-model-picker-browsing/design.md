# Design and adversarial authoring record

## Local context

- Favorites/selection persistence: `crates/jcode-tui/src/tui/app/inline_interactive.rs:70-96`, `193-284`; SSH-backed clients explicitly do not use client-local preference stores.
- Existing provider allowlist: `inline_interactive.rs:102-145`; it is a configured catalog boundary, not a new UI provider selector.
- Current mixed ranking: `inline_interactive.rs:2050-2087`; current/favorite/recent prefixes precede usage/recommendation/availability factors.
- Current fuzzy filtering: `inline_interactive.rs:4123-4165`; shared picker filtering cannot be changed blindly for all picker kinds.
- Picker route metadata: `crates/jcode-tui/src/tui/mod.rs:2120-2127` has provider, API method, availability, detail and reference cost, but no proven per-model health or account-scoped quota snapshot.
- Existing usage collectors return provider-level limits, errors, hard-limit state and usage/reset metadata: `crates/jcode-usage-types/src/lib.rs:26-54`. `usage_percent` means used, not remaining. `last_used_unix_secs` is credential usage time, not quota measurement time.
- Current key hints: `crates/jcode-tui/src/tui/ui_inline_interactive.rs:202` and corresponding `ui_inline_interactive_tests.rs`. Existing favorite/default controls must remain compatible.

These references ground feasibility, not a claim that every provider already exposes model health, remaining quota or measurement timestamps.

## Why a separate change

P09 owns model browsing and terminal UX. P06 owns auth, quota/accounting and provider diagnostics; P02 owns tool discovery. None of the existing changes owns interactive model catalog filtering/sorting. This change consumes existing data with explicit unknown states rather than depending on unimplemented provider coverage or inventing new health APIs.

## Adversarial decisions

1. **Availability is not health.** Configured credentials and a known catalog entry do not prove a particular model can serve requests. Keep availability, observed outcome and quota separate.
2. **A model is not an account.** Route/account failure and quota must not leak across aliases, providers or accounts. If the current protocol cannot attribute status, show unknown until an additive field can carry authoritative identity.
3. **Most used is ambiguous.** Prefer the existing prospective tracked-turn count (`crates/jcode-usage-types/src/lib.rs:1016-1066`), not request count or successful-check count. Identify existing picker-selection fallback as selections, not completed model work. Do not silently mix incomparable provider counters.
4. **Filtering is not authorization.** All uses the existing catalog-loader output, including its zero-match allowlist fallback and current-route exception (`inline_interactive.rs:171-179`), with a visible fallback explanation. Interactive filters never silently widen their requested scope. Catalog visibility is not an authorization boundary. An excluded current model belongs in a separate summary.
5. **Favorites can outlive catalogs.** Preserve identity and explain unavailable/missing routes. Never silently substitute a provider or account.
6. **Async data can select the wrong route.** Bind input and refresh results to stable identities and current filter generation; protect focus while sorting.
7. **Health probes can cost money and leak data.** Browsing issues no inference. Existing quota collectors may refresh explicitly; new diagnostic inference requires separate user consent.
8. **Stale data can be worse than no data.** Show source and age, do not use credential-last-used time as collection time, and do not map unsupported status to zero quota or green health.
9. **UI chrome cannot wrap.** Prefer compact textual controls and detail-on-selection over adding several wide columns; reuse existing bar renderer and Unicode-cell truncation.
10. **Remote clients are not account owners.** Catalog, telemetry and storage must stay with the serving host. Capability gaps are visible; local files are not a remote fallback.

## Authoring preflight and handoff

Independent proposal review on 2026-11-02 found no blocking feasibility findings and three actionable refinements. They are incorporated into the spec/tasks: preserve and label existing zero-match catalog configuration fallback; explicitly implement selected sorting rather than blindly reusing timestamp/fuzzy comparators; immediately refilter and repair focus when the sole/current favorite is removed. This review inspected local source and artifacts only, not runtime behavior.

Authoring inputs are current local source, active proposals, repository TUI constraints and the explicit user request. No external API behavior is assumed. No new dependencies, provider accounts, worktrees or runtime settings are installed or changed.

Eligible now: proposal/spec/task authoring, local structural validation and dependency validation. Implementation is not started by authoring. At implementation time, ordinary Rust tests require the local toolchain and coordinated compile access; isolated tester/remote fixtures require their existing runtime facilities. Missing provider credentials do not block browsing and unknown-state tests; they block only optional live telemetry checks. Inference-based health checks are outside the selected scope. Final acceptance, independent review, commit and archive remain mandatory implementation closeout steps, not facts established by proposal validation.
