# Proposal

## Why

Jcode searches must use OmniRoute's centrally owned Firecrawl-first policy, with Brave eligible only after confirmed Firecrawl billing or quota exhaustion. Existing custom HTTP search can carry this integration without adding another client backend or duplicating gateway policy.

## What Changes

- Document an opt-in POST recipe for `https://pi5.tail296462.ts.net:8443/v1/search` using existing custom search configuration.
- Disable provider-native preference and local fallback engines in that recipe so Jcode does not bypass the gateway policy.
- Test the verified gateway request selector, query/count JSON encoding, header environment resolution, result mapping, and fail-closed HTTP behavior using local fixtures.
- Document exact native-provider boundaries rather than promise interception of all providers.
- Keep policy selection and fallback classification in OmniRoute. Do not change default Jcode behavior, user configuration, credentials, or gateway deployment in this client lane.

## Capabilities

### New Capabilities

- `omniroute-search`: an opt-in configuration contract that routes ordinary Jcode websearch through the gateway-owned search policy without client-side fallback.

### Modified Capabilities

None. Existing generic custom search behavior remains unchanged.

## Impact

Implementation targets are `docs/custom-web-search.md` and `crates/jcode-app-core/src/tool/websearch_tests.rs`. Existing production request and parsing functions remain unchanged. No new dependencies, MCP integration, provider changes, commits, or deployment belong to this worker lane.

The gateway lane must define the exact policy selector, authentication header support, and result shape before the recipe and contract-specific tests become executable. Coordinator owns combined validation, independent review, authorized persistence, runtime activation, and archival.
