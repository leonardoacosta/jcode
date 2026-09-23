## Why

The `evaluate` tool and Review screener independently resolve the same typed Jev service, and AutoReview/AutoJudge configuration currently provides only chat model overrides. A single service contract prevents provider, endpoint, key, and model behavior from drifting while keeping System One separate from conversational model APIs.

## Changes

- Add `jev-service-configuration` for shared provider/model/credential resolution across Evaluate and Review screening only.
- Specify TypeSafe System One versus the OpenRouter Decisions endpoint, including OpenRouter's `~typesafe/jev-latest` model alias.
- Keep AutoReview/AutoJudge chat settings and compaction's Jev caller unchanged and out of scope.

## Impact

No implementation is included in this proposal. OpenRouter is the requested default for typed Jev decisions through its Decisions endpoint, using `~typesafe/jev-latest`; direct TypeSafe System One use remains available. Credentials remain in environment or an already-supported private credential source, never TOML. Review screening must continue routing Jev failures to full LLM review. AutoReview/AutoJudge chat provider/model behavior and the compaction Jev caller remain unchanged.
