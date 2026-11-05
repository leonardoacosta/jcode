# Spec Delta

## Purpose

Route ordinary Jcode websearch requests through one gateway-owned search policy while preserving credential safety and preventing unrequested client-side provider bypass.

## ADDED Requirements

### Requirement: Opt-in gateway search configuration
The documented configuration SHALL send ordinary websearch requests to `https://pi5.tail296462.ts.net:8443/v1/search` with POST, a JSON string `query`, a JSON integer `max_results`, and the gateway-defined explicit policy selector. It SHALL use existing custom configuration without changing Jcode defaults.

#### Scenario: Query and count reach the selected policy
- **WHEN** a user applies the verified recipe and requests a search
- **THEN** the custom request includes the exact gateway-owned selector and dynamically encoded query and result count
- **AND** query text does not become a URL parameter

### Requirement: Gateway owns fallback policy
The recipe SHALL disable provider-native preference and all local fallback engines. Jcode SHALL NOT implement Firecrawl-to-Brave classification or silently substitute another search engine for gateway errors or empty results.

#### Scenario: Gateway rejects a search
- **WHEN** the configured gateway returns an HTTP error
- **THEN** the custom client reports the HTTP status without exposing the response body or credentials
- **AND** the recipe provides no alternative local search destination

#### Scenario: Gateway returns no results
- **WHEN** the configured gateway returns a valid empty result array
- **THEN** Jcode reports no results without calling a local fallback engine

### Requirement: Existing credential and result contracts
The recipe SHALL resolve gateway authentication from an environment-backed complete header value and map gateway results to title, URL, and snippet using existing JSON pointers. It SHALL NOT require inline secrets, MCP, or a client provenance framework.

#### Scenario: Configured search result is mapped
- **WHEN** the gateway returns a result matching its verified response contract
- **THEN** Jcode exposes the mapped title, URL, and snippet subject to the requested count
- **AND** gateway metadata outside those mapped fields is not claimed as visible client provenance

### Requirement: Explicit native-search coverage boundary
Documentation SHALL distinguish ordinary custom websearch from provider-hosted search and SHALL NOT claim that the custom endpoint intercepts every provider's native search.

#### Scenario: Custom gateway model transport
- **WHEN** the model request uses a custom OpenAI or Anthropic gateway transport
- **THEN** documentation states that existing Jcode native-search injection is first-party-gated and does not automatically add hosted search to that transport
- **AND** gateway-native interception is claimed only for separately verified gateway routes
