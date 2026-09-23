## Why

Jev-backed decisions are configured independently in the `evaluate` tool and the existing Review screener, while AutoReview and AutoJudge can override a model but do not expose a consistent provider selection. The duplicated paths can drift in endpoint, key precedence, and model defaults. Define one configuration contract so both Review/Judge and screening use the same selected Jev service without storing credentials in TOML.

When an OpenRouter credential is available and no provider is explicitly selected, typed Jev decisions use the OpenRouter Decisions endpoint with `~typesafe/jev-latest` as the model alias. Direct TypeSafe System One remains available with native model `jev-latest`. The alias is not a natural-language chat model. AutoReview and AutoJudge chat provider/model settings are separate and remain unchanged by this proposal; no chat model defaults or new chat provider fields are introduced. Compaction's Jev caller is explicitly out of scope and unchanged.

## What Changes

- Add one shared Jev service configuration for provider selection and optional model selection, consumed by the `evaluate` tool and Review screening.
- Support explicit OpenRouter as the default provider and `~typesafe/jev-latest` as its default model.
- Keep TypeSafe direct access available as an explicit provider, with its compatible `jev-latest` model default.
- Resolve provider credentials only from supported environment/private credential sources. Never place API keys in `config.toml`, project specs, logs, or status output.
- Preserve graceful behavior when credentials are missing or a Jev request fails: evaluation reports a clear sanitized error, and screening routes affected items to full Review rather than treating them as low risk.
- Keep AutoReview/AutoJudge chat provider/model behavior unchanged; do not add or change chat provider/model fields in this proposal.
- Keep compaction's Jev caller unchanged and out of scope. This proposal covers `evaluate` and Review screening only; compaction configuration is a separate follow-up if desired.

## Capabilities

### New Capabilities

- `jev-service-configuration`: Shared provider, model, and credential-resolution behavior for JEV typed decisions.

### Modified Capabilities

- `evaluate`: Use the unified Jev service configuration and configured defaults.
- `review`: Use the same configured Jev service for screening, preserving safe fallback to full review.
- `auto-review-and-judge`: No behavior change. Its chat provider/model fields and existing selection semantics remain unchanged and are outside this proposal's scope.

## Impact

This is a configuration and provider-resolution change for JEV evaluate and Review screening only. Existing configurations without new fields must continue to work through a documented migration/fallback path. AutoReview/AutoJudge chat provider/model behavior and compaction's Jev caller are explicitly unchanged and out of scope. No chat defaults are defined or modified here. The change does not alter global user configuration as part of proposal authoring.