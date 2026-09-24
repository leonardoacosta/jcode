## Why

The `evaluate` tool resolves TypeSafe/OpenRouter endpoint, key, and model inline. Foreman bakes the same logic with different defaults. No crate exposes a reusable service boundary. A modular service crate gives every typed-Jev caller one shared resolver, one client, and one testable contract without coupling to tool schemas.

## Changes

- Add `crates/jcode-system-one` with a typed trait (SystemOneService), provider/model resolver, credential lookup, and HTTP client.
- Migrate `evaluate` to consume the service. Leave Foreman and compaction unchanged.

## Impact

No implementation exists yet. The crate is self-contained with no dependency on tool or TUI crates. OpenRouter is the default provider when its credential is available. Credentials remain environment-only.
