# Design

## Context

Existing MultiProvider persists session binding and reapplies it to lazily created runtime providers. Compatible OpenRouter runtime previously ignored set_session_id and emitted no OmniPi cache affinity hints. See proposal.md for measured investigation and approval.

## Decisions

- Add default-false session_cache_affinity to named provider config. Explicit opt-in avoids unsupported-field failures on strict endpoints.
- Store affinity in a dedicated small runtime helper with locked session-derived opaque key, scoped by profile. Reuse existing SHA256-backed fingerprint capability, no dependency.
- Bind through Provider.set_session_id. New fork has fresh provisional identity until MultiProvider applies the actual bound parent/child session. Restore rebuild derives the same key.
- Insert default key before explicit extra_body merging; resolve final identity after chat/Responses construction. Send exactly that key as x-session-id through retries. Reject invalid keys, conflicting explicit headers and auth-header collision before network.
- Isolated selfdev RC must exercise actual Jcode session lifecycle and public provider, not only copied request harnesses. Promotion has no failure waiver.

## Risks and boundaries

Shared checkout includes unrelated dirty work. Preserve unrelated files/hunks; full artifact includes them unless separately isolated with approval, so provenance and RC gates may block promotion. Existing explicit configured identities can intentionally share affinity, user overrides remain authoritative. Opt-out unchanged. No cache hit guarantee across upstream eviction/account changes. No TTL changes. Native selfdev tool unavailable in this harness; use documented wrapper fallback and isolated selfdev runtime without altering shared debug permissions.
