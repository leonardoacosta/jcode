# OAuth account routing

> Status: observed implementation with proposed factory extensions
> Claim labels: **observed**, **proposed**, **open question**

## Summary

Jcode supports multiple OAuth accounts from the same provider for Anthropic/Claude and OpenAI/Codex. The current design is optimized for interactive account switching and same-provider failover. It is not yet a fully provider-neutral, task-scoped credential-routing system for concurrent factory workers.

## Current support

| Provider | Multiple OAuth accounts | Switching | Same-provider failover | Usage-aware selection |
|---|---:|---:|---:|---:|
| Anthropic / Claude | **Observed: yes** | **Observed: yes** | **Observed: yes** | **Observed: yes** |
| OpenAI / Codex | **Observed: yes** | **Observed: yes** | **Observed: yes** | **Observed: yes** |
| Gemini | **Open question** | **Open question** | **Open question** | **Open question** |
| Antigravity | **Open question** | **Open question** | **Open question** | **Open question** |
| GitHub Copilot | **Open question** | **Open question** | **Open question** | **Open question** |
| Cursor | **Open question** | **Open question** | **Open question** | **Open question** |

The first-class multi-account implementation is concentrated in:

```text
crates/jcode-base/src/auth/account_store.rs
crates/jcode-base/src/auth/claude.rs
crates/jcode-base/src/auth/codex.rs
crates/jcode-base/src/provider/account_failover.rs
crates/jcode-app-core/src/server/provider_control.rs
crates/jcode-tui/src/tui/app/auth_account_commands.rs
crates/jcode-tui-account-picker/
```

## Account model

Anthropic and OpenAI each maintain a collection of named accounts plus an active label. Labels are canonicalized into predictable identifiers:

```text
claude-1
claude-2
claude-3

openai-1
openai-2
openai-3
```

Stored account metadata includes OAuth access and refresh tokens, expiration, provider account identity where available, email, subscription metadata, scopes, and ID tokens where applicable. Secret files use hardened storage helpers.

The implementation also supports migration from older single-account layouts and controlled import of credentials left by other tools such as Claude Code and Codex.

## User-facing behavior

The interactive command surface includes:

```text
/account
/account switch claude-2
/account switch openai-2
```

The account picker and remote command paths expose the same capability. Switching an account does not merely change a display label. Jcode also:

1. Persists the selected active account.
2. Invalidates authentication status.
3. Resets the provider session.
4. Invalidates provider credentials.
5. Clears provider and model unavailability state.
6. Refreshes usage information.
7. Publishes updated model availability.
8. Emits completion or error events.

Relevant implementation entry points are `handle_switch_anthropic_account` and `handle_switch_openai_account` in `crates/jcode-app-core/src/server/provider_control.rs`.

## Failover and usage awareness

Jcode has same-provider account failover for Anthropic and OpenAI. Candidate selection can inspect per-account usage, exclude exhausted or errored accounts, and prefer accounts with lower five-hour and seven-day usage ratios.

The failover path recognizes common quota and rate-limit failures, including `429`, `402`, quota exhaustion, billing failures, rate limits, and usage-limit messages. Configuration controls whether same-provider account failover is enabled.

Usage collection also has account-aware paths for Anthropic and OpenAI, allowing the UI and failover logic to reason about reset windows, exhausted accounts, current accounts, and recommended alternatives.

## Scope limitation

The active-account override is stored in shared process-local state keyed by provider prefix, such as `claude` or `openai`, with persisted active-account state as a fallback. The effective resolution is:

```text
runtime provider override
    ↓
persisted active account
    ↓
first stored account
```

This supports interactive switching, but it means account selection is currently **provider-global within the running Jcode process**.

The current implementation does not establish a durable binding of the form:

```text
session A → claude-1
session B → claude-2
session C → claude-3
```

Therefore the following is observed:

```text
Interactive account switching and same-provider failover: supported.
Concurrent sessions independently pinned to different same-provider accounts: not established.
```

## Factory implications

A software-factory worker should receive account identity as part of its execution contract instead of relying on ambient provider-global state:

```yaml
provider:
  family: anthropic
  model: claude-opus
  auth_mode: oauth
  account_policy:
    mode: pinned
    account_ref: claude-2
    allow_failover: true
    failover_scope: same-provider
```

Useful account policies include:

```text
active
pinned
least-used
round-robin
quota-aware
failover-only
```

A future provider account record should separate stable identity from credentials:

```text
ProviderAccount
├── account_id
├── provider
├── auth_method
├── external_subject
├── display_label
├── credential_ref
├── scopes
├── expires_at
├── status
├── usage
└── last_used_at
```

Task and run records should contain an account reference or credential reference, never raw OAuth tokens.

## Provenance requirement

If a run starts on one account and fails over to another, the run evidence should preserve the routing history:

```yaml
account_events:
  - provider: anthropic
    account: claude-1
    event: selected
  - provider: anthropic
    account: claude-1
    event: rate_limited
  - provider: anthropic
    account: claude-2
    event: failover_selected
```

This is necessary for replay, quota analysis, auditability, debugging, and preventing cross-task account interference.

## Verification evidence

Repository acceptance tests exercised the real account and provider-control paths:

- `cargo test -p jcode-base auth::codex --lib`: 17 tests passed, including multi-account active switching, numbered-label migration, legacy OAuth consent, JWT metadata, and credential fallback.
- `cargo test -p jcode-app-core provider_control_tests --lib`: provider-control tests passed, covering authentication refresh, model-catalog updates, busy-session deferral, current-session model switching, and refresh completion behavior.

A Claude-specific filter invocation completed successfully but matched zero tests because the filter did not match the module naming. Live OAuth authorization was not exercised because it requires external credentials and side effects. Concurrent sessions pinned to separate accounts remain an open integration boundary.

## Assessment

Jcode currently supports:

> Multiple Claude or OpenAI OAuth accounts for interactive switching, usage inspection, and same-provider failover.

Jcode does not yet fully support:

> Concurrent factory tasks deterministically pinned to separate accounts from the same provider, with isolated credential resolution, independent refresh, and complete account-routing provenance.

The highest-value factory extension is to make provider account identity explicit in provider selection, task contracts, worker runs, and evidence.

## Implementation references

- `crates/jcode-base/src/auth/account_store.rs`
- `crates/jcode-base/src/auth/claude.rs`
- `crates/jcode-base/src/auth/codex.rs`
- `crates/jcode-base/src/provider/account_failover.rs`
- `crates/jcode-app-core/src/server/provider_control.rs`
- `crates/jcode-tui/src/tui/app/auth_account_commands.rs`
- `crates/jcode-tui-account-picker/`
