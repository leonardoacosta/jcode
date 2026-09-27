# Local profile lifecycle

## Ownership and identity

Jcode owns profile metadata and fresh local Chrome data directories. Every profile has a stable ID and unique human-readable label; labels are not filesystem paths. Creation explicitly chooses temporary or persistent storage. Fresh means empty initial state, not a third storage mode. Agents can create, list, inspect, select, and change defaults. Temporary profiles are scoped to their owning Jcode session and cannot become a durable global default.

Persistent profiles survive session closure and restarts. Temporary profiles clean up after their owner closes and all owned browser handles are released. After a crash, recovery verifies ownership and inactivity before cleanup. If inactivity cannot be proven, retain state and report pending cleanup rather than risking data loss.

Attached profiles are labeled references to an existing Chrome user-data directory and profile identity. They are not copied, imported, or converted to managed state. Selecting the label authorizes use of its signed-in sessions without an additional login-consent prompt. This is not permission to purchase, send messages, or perform destructive website actions. Never delete attached data, remove browser locks, force-close an unrelated browser, or silently select another identity. Duplicate aliases for the same backing profile must not bypass locking or pretend to be isolated profiles.

## Runtime boundary

Use jev-ultrafast for automation, not the old Firefox bridge. Jcode may launch local Chrome with separate managed user-data directories and connect the upstream harness to that specific browser endpoint. Missing upstream profile-management APIs do not preclude this design. Pin and inspect the actual upstream dependency versions before relying on endpoint/environment support. Prove that a dead endpoint fails closed without discovering a personal browser.

The observed upstream library exposes Agent and Browser methods and a goal loop. A goal-oriented browser interface is acceptable; preserving every old one-action bridge operation is not a new prerequisite. Do not build a second action executor or maintain a speculative upstream fork. Verify supported operations and advertise only those actually implemented.

An attached running profile requires an authorized compatible connection. If its browser lock or debugging permissions prevent connection, return a precise error without changing those controls. Bind each browser session to a profile ID. Default changes affect new sessions only. Reject cross-profile tab/session references.

## Storage, cleanup, and errors

Keep managed state under a private Jcode-owned root. Validate labels, use stable IDs for paths, enforce restrictive permissions, and reject symlink/path traversal during destructive operations. Attached paths may be outside this root, but must never enter its deletion path. List/status reveal label, ownership, lifetime, availability and session binding, never cookies or credential contents.

Explicit deletion of inactive managed persistent data requires user confirmation. Temporary cleanup follows its approved lifecycle without another confirmation. Removing an attached registration affects metadata only. Cleanup failures retain truthful records and actionable errors. Serialize conflicting profile operations and respect browser-native locks.

Use bounded argument-vector subprocess calls, cancellation, child reaping, and redacted errors. No implicit package installation, cloud provisioning, personal profile discovery, credential import, or shared-daemon restart. Existing bridge files remain untouched while executable bridge paths are removed from source.

## Verification boundary

Use disposable local profiles and a local test website to prove persistent state survives restart, temporary state is removed, and separate managed profiles do not share state. Create a disposable existing profile with a harmless signed-in test session, register its label, select it and verify that state is usable without another consent prompt. Verify its data survives detach and cleanup. Real upstream integration evidence is required, not just metadata tests.
