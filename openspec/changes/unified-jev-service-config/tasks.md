## 1. Configuration contract

- [ ] 1.1 Define the shared Jev provider selection and optional model override in config types; document provider values, default behavior, and migration compatibility.
- [ ] 1.2 Define one resolver for provider, endpoint, effective model, and environment/private credential lookup. Never accept keys from TOML or include them in diagnostics.
- [ ] 1.3 Add unit tests for default selection, each provider, both credentials present, a single credential present, whitespace-only credentials, explicit provider with missing key, and provider-specific model defaults/overrides.

## 2. Migrate Jev decision callers

- [ ] 2.1 Route the `evaluate` tool through the shared resolver without changing its typed request/response schema or cache behavior.
- [ ] 2.2 Route Review screening through the same resolver without changing its risk policy or parsing behavior.
- [ ] 2.3 Verify that Review and Evaluate resolve to the same provider, endpoint, credential source, and model under identical configuration.
- [ ] 2.4 Preserve safe failure behavior: Evaluate returns a sanitized actionable error; Review sends failed/invalid screening items to full LLM review.
- [ ] 2.5 Leave compaction's Jev caller, endpoint, credentials, model, and fallback policy unchanged; verify the implementation diff does not touch that path.

## 3. Scope boundaries and verification

- [ ] 3.1 Verify AutoReview/AutoJudge chat provider/model fields, defaults, session selection, and model override semantics are unchanged; verify `~typesafe/jev-latest` is never supplied as their chat model.
- [ ] 3.2 Document environment/private credential setup, OpenRouter Decisions endpoint and model alias, TypeSafe compatibility, precedence, and migration behavior without exposing secrets.
- [ ] 3.3 Run focused config, evaluate, and review-screening tests plus OpenSpec validation; verify all stated provider/fallback cases have observable coverage and compaction remains untouched.
