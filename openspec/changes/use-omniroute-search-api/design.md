# Design

## Context

See proposal.md for the motivation. At Jcode revision `5b101c6f8bf4de983b7db822fb4ee93ff65e5d8c`, `CustomWebSearchConfig` already supports POST, a static JSON object, dynamic top-level query/count keys, environment-backed headers, and configurable JSON pointers. `custom_request` and `search_custom` in `crates/jcode-app-core/src/tool/websearch.rs:434-555` provide the complete transport. Existing tests cover generic request construction and credential-safe GET/error handling.

Native injection is a separate model-request path. OpenAI Responses injects hosted search only for the first-party API or ChatGPT backend, non-Codex models, native-enabled configuration, and a session offering local websearch. Anthropic injects server search only for OAuth or the exact first-party Messages API and a session offering websearch. Custom gateways retain the local tool. These are client eligibility rules, not proof of gateway interception or provider availability.

## Goals / Non-Goals

**Goals:** Verify and document the opt-in recipe using existing production code, preserve one policy owner, and state native-search limits precisely.

**Non-Goals:** Product default changes, client fallback classification, provenance storage, secret provisioning, actual user configuration edits, gateway changes or deployment, native-search enablement across providers, commits, or independent worker spawning.

## Decisions

- Reuse the custom client rather than add an OmniRoute backend. Static `body` carries the gateway-owned selector. `query_param = "query"` and `count_param = "max_results"` map the local query and result count to the verified gateway keys.
- Use `engine = "custom"`, `prefer_native = false`, and `fallback_engines = []` together. Otherwise first-party native preference or the default Bing fallback can bypass central policy.
- Use a verified gateway-supported header via `header_env`. Its environment value is the entire header, including `Bearer ` only when that header requires it. No guessed authentication scheme or inline credential is permitted.
- Keep production result parsing unchanged. It maps title/URL/snippet and ignores other envelope fields. Gateway provenance remains gateway evidence, not a newly invented client output contract.
- Gateway contract relayed by the coordinator at `2026-11-05T22:17:39Z`: POST JSON `routing_policy = "firecrawl-first-brave-quota"`, omit `provider`. Missing selector preserves legacy routing. Unknown/non-string selectors or any simultaneous `provider` are rejected before provider calls. The named policy bypasses the legacy cache.
- Contract confirmation at `2026-11-05T22:21:26Z`: coordinator observed authenticated GET `/v1/search` returning 200 with `x-api-key`, without performing a search. Gateway worker confirmed compiled normalizer/route output `results` items with `title`, `url`, and `snippet`. Recipe therefore uses `header_env = { "x-api-key" = "JCODE_PROVIDER_OMNIPI_API_KEY" }` with raw-key header value and `/results`, `/title`, `/url`, `/snippet` pointers. This does not prove candidate-policy deployment or live POST behavior.
- Add local request-construction and loopback POST tests before documentation. No production behavior change is planned, so these are characterization/integration tests rather than a claimed red-to-green product change.

## Risks / Trade-offs

- First-party native search can bypass the gateway if left preferred. Mitigation: explicit disabled native preference in the recipe.
- Existing default fallback sends queries to Bing after errors or empty results. Mitigation: explicit empty fallback list, checked in tests.
- The 30-second custom-client timeout may be shorter than a slow gateway chain. Mitigation: report the existing limit rather than silently change timeouts or claim live reliability.
- An API recipe can become stale independently of Jcode. Mitigation: use the gateway lane's verified contract and test its exact request shape.
- The worker cannot prove live gateway behavior or complete commit/review/archive gates. Mitigation: coordinator retains those tasks and must not infer deployment from local fixtures.

## Verification and handoff

Use `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core tool::websearch::tests --lib` under the coordinated selfdev test lock. Validate artifacts with `openspec validate use-omniroute-search-api --strict`. Inspect only granted file diffs and preserve the nine initial untracked paths.

Coordinator owns final combined contract review, authorized live API checks and runtime activation, independent final-diff review, verified commits/persistence, and archive. None is marked complete by this worker's local test result.

### Current evidence

- Binding: `https://github.com/leonardoacosta/jcode.git`, revision `5b101c6f8bf4de983b7db822fb4ee93ff65e5d8c`, main checkout preserved. Remote HEAD resolves to `origin/dev`; no branch operation is authorized or required. Relevant implementation/doc targets were clean before this lane. Original production file SHA-256: `4f7f3993c23786a77762e68f02935612ad5b76df314c021ca9bb63ce9bc6ac5d`. Initial test/doc SHA-256: `3fe32cdd8040e600dd66abdbc87896763689146fbd4e0d320f170fecdc41837a` / `8d424a5e68886b0cd09158858c746317d992b753038c425ea07cbff6732dd45d`.
- Initial-dispatch preflight, `preflight-check-v2-shadow`, `2026-11-05T22:18:09Z`: granted docs/tests and local checks eligible. Final recipe blocked on gateway auth/result shape. Live acceptance, independent review, commit/persistence, and archive remain coordinator-owned and unverified. G1=2, G2=1, G3=1, G4=2, G5=2, G6=1, total 9/12 = 75/100 advisory shadow quality. This score grants no authority and is not a completion claim. Recheck affected bindings when the gateway contract or final diff changes.
- Baseline command `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core tool::websearch::tests --lib` passed: 22 tests, exit 0 (`720899zf3v`).
- After characterization additions, `rustfmt --edition 2024 crates/jcode-app-core/src/tool/websearch_tests.rs && JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core tool::websearch::tests --lib` passed: 24 tests, exit 0 (`1319858glv`). New tests exercise exact policy/query/count encoding, no native/local bypass, and loopback POST empty-success plus 400/401/402/429/500 error redaction without retries. They do not make external searches or prove gateway failover behavior.
- `openspec validate use-omniroute-search-api --strict` passed after initial artifact authoring. Cargo emits an existing unmatched `yazi` profile-package warning; test results are successful, not warning-free.
- Previous intermediate client-test run (`270366dr6o`) passed 24/24 with environment-backed `x-api-key`, response mapping, and local count limiting, before gateway source corrected the literal result-count key. The earlier `count` request binding is superseded, not evidence of gateway count support. Existing native-boundary suites passed with `JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-provider-openai-runtime -p jcode-provider-anthropic-runtime native_web_search --lib`: Anthropic 15/15, OpenAI 6/6 (`198653dvfm`).
- Count-key reconciliation at `2026-11-05T22:24:01Z`: gateway worker quoted `let K=Math.min(w.max_results,G.maxMaxResults)` followed by executor `maxResults:K`, with query from `w.query`. Coordinator confirmed semantic query/count wording did not specify the literal JSON count key. Final recipe/spec/tests use `max_results` through existing `count_param`.
- Final gates after that correction (`468558n7x6`) passed exit 0: `rustfmt --edition 2024 --check crates/jcode-app-core/src/tool/websearch_tests.rs && JCODE_REMOTE_CARGO=0 scripts/dev_cargo.sh test --profile selfdev -p jcode-app-core tool::websearch::tests --lib && openspec validate use-omniroute-search-api --strict && git diff --check`. Websearch tests passed 24/24. Python `tomllib` parsed the final documented recipe and asserted all fields, including `max_results`.
- Final docs/test targets bound at `2026-11-05T22:26:01Z`: test SHA-256 `f68a27602a44fc9bdde51781ffe654cef58827458990d3305afe1492f04affad`, docs SHA-256 `85151a449962c33864925d03f7b90ba190927167f21ddc5f1c503c40fe9b82b1`. Product file digest remains identical to baseline.
- JIT refresh after full contract confirmation: local client lane verified and ready for coordinator handoff. Coordinator-owned live POST policy acceptance, final independent review, commit/persistence, and archive remain unchecked. Existing native tests prove Jcode eligibility and replay behavior, not all-provider gateway interception. No user config, secrets, deployment, branch, worktree, commit, or external search was changed or performed by this worker.
