---
execution:
  version: 1
  depends_on: []
---
Roadmap owner: [P08: SDK/API/tool integration contracts, including browser/voice](../../../docs/plans/PRODUCT_ROADMAP.md#p08-sdkapitool-integration-contracts-including-browservoice)

## Why

The Rust harness API and the TypeScript SDK duplicate the protocol by hand: both declare `API_VERSION_MAJOR = 1` / `API_VERSION_MINOR = 11` (`crates/jcode-harness-api/src/lib.rs:54-56`, `sdk/typescript/src/protocol.ts:10-11`) and both handshake with `min_version = max_version = MAJOR` (`crates/jcode-harness-api/src/client.rs:88-92`, `sdk/typescript/src/client.ts:329-334`). The policy that additive changes bump minor (`crates/jcode-harness-api/src/lib.rs:13-14`) has no test that the two sides agree, and no shared fixture covers malformed frames, `unsupported_version`, or event ordering after `Cancel` (`crates/jcode-harness-api/src/requests.rs:123-124`).

## What Changes

- Add shared JSON golden fixtures (requests, events, error frames) generated from the Rust types and consumed by a TypeScript test.
- Add a check that Rust and TypeScript version constants match.
- Add server contract tests for: unsupported major version, malformed frame, and cancel producing exactly one terminal event after in-flight deltas.

## Non-scope

- New API features or a version bump.
- Desktop client changes (consumer boundary only).
- Lifecycle hook events (add-lifecycle-hooks).

## Capabilities

### New Capabilities
- `harness-sdk-contracts`: cross-language protocol compatibility checks.

### Modified Capabilities
- None.

## Impact

`crates/jcode-harness-api` tests, `crates/jcode-harness-api-server` tests, `sdk/typescript` test suite. No runtime behavior change unless tests expose a bug.

## Open decisions

- Fixture direction: Rust-generated goldens or hand-written shared JSON? Recommended: Rust-generated with a checked-in copy and a drift test, because Rust types are the server source of truth.
