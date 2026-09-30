## Why
System One poke observer parsing currently converts nonnumeric probability values to zero. A malformed distribution can therefore pass validation when its numeric entries already sum to one. Observer evidence should fail closed rather than accept invalid data.

## What Changes
- Reject every probability entry that cannot be decoded as a JSON number.
- Retain existing finite/range, sum, recommendation, and confidence-margin checks.
- Add regression coverage for malformed zero-weight entries and valid numeric zero entries.

## Capabilities

### New Capabilities
- `poke-observer-response-validation`: Strict validation of probability values returned by the System One poke observer.

### Modified Capabilities
None.

## Impact
Scoped to `crates/jcode-app-core/src/agent/poke_shadow.rs` and its existing tests. No new dependency, configuration, observer policy, or overnight scheduling changes. Malformed responses abstain with the existing `MalformedResponse` reason.
