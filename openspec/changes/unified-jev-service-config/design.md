## Context

`crates/jcode-app-core/src/tool/evaluate.rs` and `tool/review/screener.rs` each implement their own TypeSafe/OpenRouter endpoint, key, and model selection. Both currently prefer a non-empty `TYPESAFE_API_KEY` and fall back to `OPENROUTER_API_KEY`. The Evaluate path already has an explicit OpenRouter Decisions endpoint model default (`~typesafe/jev-latest`) when OpenRouter is selected. This proposal covers these two Jev decision callers only. The compaction Jev caller is explicitly out of scope and remains unchanged. AutoReview/AutoJudge chat provider/model configuration is also out of scope and remains unchanged; `~typesafe/jev-latest` must never be used as their natural-language chat model.

## Goals / Non-Goals

**Goals:**
- Give Evaluate and Review screening one provider/model/credential resolution path.
- Make the OpenRouter Decisions endpoint the default Jev provider when its credential is available, with model alias `~typesafe/jev-latest`.
- Retain direct TypeSafe System One access as an explicit provider, with `jev-latest` as its native model default.
- Keep credentials out of TOML and diagnostics; use environment/private credential sources only.
- Preserve safe fallback: screening failures never become a low-risk result.
- Preserve existing Review/Judge chat-provider/model semantics.

**Non-Goals:**
- Do not modify AutoReview/AutoJudge chat provider/model fields, defaults, or session selection behavior; these remain unchanged.
- Do not migrate the compaction Jev caller; it remains unchanged and out of scope.
- Do not add a credential-writing/setup flow as part of this configuration change.
- Do not modify the user's current global config while authoring this proposal.

## Decisions

1. **One shared Jev service config and resolver.** Both Evaluate and Review screening use the same resolved endpoint, credential, and effective model. Keep request-specific state/questions and response parsing at their current call sites. Avoid a second copy of provider precedence.

2. **OpenRouter Decisions is the default Jev provider when its credential is available.** With no provider explicitly selected, use `OPENROUTER_API_KEY` and the OpenRouter Decisions endpoint with `~typesafe/jev-latest`. If that credential is unavailable, retain compatibility with a configured TypeSafe direct key and native `jev-latest`. An explicitly selected provider uses only its own key and fails clearly if missing; it does not silently send credentials to another provider. Empty/whitespace environment values count as absent. Provider-specific model defaults apply; explicit model overrides are honored only where compatible.

3. **Keep AutoReview/AutoJudge chat settings untouched.** Their chat provider/model selection semantics and defaults remain unchanged. The `~typesafe/jev-latest` alias is only for the OpenRouter Decisions API and is never applied to a conversational Review/Judge session.

4. **Keep compaction unchanged and out of scope.** This change covers only Evaluate and Review screening, the two Jev decision paths named by the proposal. Do not change compaction's endpoint, credentials, model, or fallback policy in this work.

5. **No secrets in configuration files or output.** TOML may select provider and model but cannot carry API keys. Use environment/private credential sources. Errors and status may identify the provider and missing-key variable, but must never include the credential or request authorization data.

6. **Safe fallback remains mandatory.** If Jev is unavailable, Review screening marks affected hunks for full LLM review. Evaluate returns a sanitized actionable error. Never interpret service errors or malformed answers as low risk.

## Risks / Trade-offs

- **Changing default provider can affect users with both keys.** Document the precedence and permit explicit selection. Add tests for both-key, one-key, empty-key, and explicitly selected provider cases.
- **OpenRouter decisions API compatibility may differ from direct TypeSafe.** Keep provider-specific endpoint and model defaults in the shared resolver and test request construction for each.
- **Model overrides may not be supported identically across providers.** Validate or clearly document compatibility rather than silently rewriting the requested model.
- **Other Jev callers may remain inconsistent.** Compaction is intentionally out of scope and unchanged. Record any later consolidation as a separate proposal rather than expanding this scope during implementation.

## Migration Plan

1. Add shared resolver and optional provider/model configuration fields. With no explicit selection, prefer OpenRouter when its key is available; fall back to TypeSafe for existing TypeSafe-only installations.
2. Route Evaluate and Review screening through the resolver and verify both paths agree on provider, endpoint, and effective model.
3. Document the OpenRouter Decisions endpoint/model alias, direct TypeSafe compatibility, and explicit-selection behavior.
4. Roll back by removing the new optional settings; existing environment-based key setup remains usable.

## Open Questions

None affecting this proposal's scope. AutoReview/AutoJudge chat configuration and compaction remain unchanged; any changes to those paths require separate design work.