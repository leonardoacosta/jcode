# MODELS, routing, catalog, usage: WebUI characterization

Source snapshot reviewed at `172691406`. Principal claims were sample-verified against cited source spans. This is not an exhaustive repository inventory.

## 1. Entities & data model
- A route is `(model, provider, api_method)` plus `available`, `detail`, optional `cheapness`, optional `usage`; concrete runtime identity is more specific via `RuntimeKey`. `crates/jcode-provider-core/src/lib.rs:682-705`
- Catalog construction spawns Anthropic/OpenAI refreshes, checks auth, then appends Anthropic, OpenAI, OpenAI-compatible profiles, Copilot, Gemini, Antigravity, Cursor, Bedrock and OpenRouter routes. `crates/jcode-base/src/provider/catalog_routes.rs:223-247`
- Profile metadata covers provider id/name, API base, key env var/file, setup URL, default model and key requirement. It does not define model context/prices/capabilities. `crates/jcode-provider-metadata/src/lib.rs:121-143`; examples `crates/jcode-provider-metadata/src/catalog.rs:6-36`
- Login provider metadata separately defines auth kind/state key/status method, aliases, menu detail, recommended, target and per-surface ordering. `crates/jcode-provider-metadata/src/lib.rs:1-9,107-119`
- Selection policy has providers Claude/OpenAI/Copilot/Antigravity/Gemini/Cursor/Bedrock/OpenRouter, configured booleans, plus Copilot premium-zero. Default priority: premium-zero Copilot, Claude, OpenAI, Copilot, Antigravity, Gemini, Cursor, Bedrock, OpenRouter. `crates/jcode-provider-core/src/selection.rs:4-26,44-66`
- Local model usage is response-turn count + last-used + tracking epoch + picker selection count/time. It is prospective and tool continuations count once. `crates/jcode-usage-types/src/lib.rs:978-992`
- Provider quota data: named percentage limits/reset time, hard-limit flag, supplemental info, errors and provider-specific reset offers. `crates/jcode-usage-types/src/lib.rs:25-44`

## 2. Persistence & runtime paths
- `~/.jcode/models/` contains `all-MiniLM-L6-v2/model.onnx` and `tokenizer.json`; no model-list config exists at this path.
- `~/.jcode/model-usage-v1.sqlite3` schema: `tracking(id INTEGER PRIMARY KEY CHECK(id=1), started INTEGER NOT NULL)`; `turns(turn_id, model, provider, api_method, last_used, PRIMARY KEY(turn_id,model,provider,api_method))`; index `turns_route(model,provider,api_method)`. (`sqlite3 .schema`)
- `~/.jcode/provider_activity.json` currently has entries `omni` and `9router`, each `last_used_unix_secs`.
- `~/.jcode/devices.json` currently has paired device metadata and no pending codes; unrelated to provider/model accounting.
- The inspected `config.toml` has no `[models]` section. At the cited paths, model selection/config appears via runtime catalog and config/auth-backed routes, not this section.

## 3. Lifecycle (discovery, selection, failover)
- Catalog refreshes start asynchronously and routes are assembled based on provider auth/config, then deduped/enriched. `crates/jcode-base/src/provider/catalog_routes.rs:223-247` (enrichment and return: `:248-326`)
- Route identity includes API method, so same model/provider with different auth/transport routes remain distinct. `crates/jcode-provider-core/src/lib.rs:696-705`
- Fallback picker filters unavailable candidates, matches models/method/provider, and excludes the exact failed route. `crates/jcode-provider-core/src/fallback_pick.rs:109-163`
- Combo/default provider choices are policy-driven by availability; provider aliases are parsed by `parse_provider_hint`. `crates/jcode-provider-core/src/selection.rs:44-79`
- Runtime model catalog activity persists provider last-used timestamps (`provider_activity.json` observed runtime data); model-usage ledger records route-level successful agent turns (SQLite schema + `ModelUsage` semantics above).

## 4. Existing API surface
- Wire request `get_model_catalog` / `GetModelCatalog { id, subscribe_usage_updates }` fetches provider/model metadata and available models; incremental usage events require explicit opt-in. `crates/jcode-protocol/src/wire.rs:203-211`
- Event `model_usage_updated` exists and carries a route. `crates/jcode-protocol/src/wire.rs:1429-1434`; opt-in compatibility test `crates/jcode-protocol/src/protocol_tests/model_usage.rs:2-23`
- Catalog-bearing event includes `available_model_routes: Vec<ModelRoute>`. `crates/jcode-protocol/src/wire.rs:1443-1450`
- Other wire payload includes `model_routes: Vec<ModelRoute>`. `crates/jcode-protocol/src/wire.rs:1564`
- Harness event `routes: Vec<ModelRouteInfo>`; fields definition at `crates/jcode-harness-api/src/events.rs:260-277,410-435`.

## 5. Read-only vs mutating ops
- Read-only: `get_model_catalog`, catalog metadata, provider quota fetches and usage/reset offers. Reset types are documented read-only. `crates/jcode-usage-types/src/lib.rs:1-23`
- Mutating: login/auth setup, choosing/changing route, request execution, and usage ledger writes on successful turns. Catalog refresh is background cache mutation, not a user-facing catalog edit. `crates/jcode-base/src/provider/catalog_routes.rs:223-230`; usage semantics `crates/jcode-usage-types/src/lib.rs:978-983`
- The inspected wire surface does not expose quota reset/claim as an operation; reset offers are data only.

## 6. WebUI-relevant fields + relationships
- Provider → one/many route → model; route key should preserve `(model, provider, api_method)` and preferably concrete `RuntimeKey` for endpoint/auth distinction.
- Route fields usable directly: `model`, `provider`, `api_method`, `available`, `detail`, `cheapness`, `usage`. `crates/jcode-provider-core/src/lib.rs:682-694`
- Auth projection: provider id/display name/auth kind/auth state key/status method/aliases/target/recommended/order. Profile projection adds API base, env-file/key names and setup URL. `crates/jcode-provider-metadata/src/lib.rs:107-143`
- Usage projection: count/last used/tracking epoch/selection count/last selected; quota: name/percent/reset time/hard-limit/error/last credential use. `crates/jcode-usage-types/src/lib.rs:25-44,978-992`
- Keep catalog availability, auth state, health, and quota state distinct; sources model them as distinct route/auth/usage information.

## 7. Gaps blocking a WebUI
- No observed per-model metadata contract for context window, token pricing, or capability flags; provider profile metadata is endpoint/auth setup metadata only (`crates/jcode-provider-metadata/src/lib.rs:121-143`).
- No single documented API object joins catalog routes, auth status, health, quota, and historical usage. `get_model_catalog` is the route/catalog surface; usage is separate/opt-in event stream (`crates/jcode-protocol/src/wire.rs:203-211,1429-1450`).
- Exact WebUI health-check contract, quota refresh/caching TTL, and per-session token counts were not evidenced in inspected sources. Current model ledger counts turns, not tokens (`crates/jcode-usage-types/src/lib.rs:978-983`).
- No `[models]` configuration section or runtime model-list file was present at inspected paths; UI must treat catalog as discovered data, not assume these files configure it.
