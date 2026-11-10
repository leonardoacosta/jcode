# Opt-in session cache affinity

## Scope and approval

User approved the session-affinity plan and swarm implementation on 2026-11-10, requiring self-dev Test RC and promotion only when RC is fully green. Add named-provider opt-in default false, persisted-session-derived opaque prompt_cache_key and matching x-session-id, explicit override validation, separate forks and stable resume/rebind. Enable only verified OmniPi profiles after isolated acceptance. No retention defaults, global provider changes, or compression changes.

## Evidence and approach

Public request experiments showed stable key-only/header-only/both warm reuse approximately98%, and key-only approximately99.79% with79Ktokens/24tools. Exact request retained98.5% after12minutes idle. This is candidate request evidence, not Jcode runtime proof. Reuse Provider.set_session_id and MultiProvider.bind_provider_session. Runtime preserves final explicit body/header identity, validates conflicts before network, and defaults off for strict endpoints.

## Preflight and preservation

Verified checkout origin github.com/leonardoacosta/jcode, HEADcc84a9723, local main while origin/HEAD points dev. Preserve branch and report mismatch, no switch. Checkout contains substantial unrelated tracked/untracked changes; only scoped hunks may be committed. azure_retry_tests.rs already dirty, coordinator owns only new None argument adaptation. Worker ownership separates runtime/config from new base lifecycle tests. Native selfdev tool unavailable in harness and debug control disabled on running shared server. Documented scripts/dev_cargo.sh fallback is available; isolated RC socket avoids shared reload. A build includes unrelated dirty source, so exact snapshot provenance and full RC gates are required before promotion. Unknown or failed RC gates block promotion, not local implementation.

## Acceptance

Config default-off/serialization, wire chat and Responses, invalid/conflicting overrides, stable retries, session resume/rebuild/pool rebind, fork isolation. Run relevant full crate suites, formatting and broader RC gates. Build selfdev binary, test actual isolated Jcode public gateway workflow with tools/append/resume/idle, verify executable identity. Independent final review and scoped commits/archive required. Shared promotion only after fully green RC and verified snapshot, never bypass unrelated failures or quietly promote unrelated work.
