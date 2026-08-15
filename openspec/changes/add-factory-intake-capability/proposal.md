## Why

Jcode's factory lifecycle assumes the operator is at a terminal. Every intent enters through a session, and every approval requires returning to one. The result is that work only starts when the operator is at their machine, and any approval blocks until then.

Chat is where the operator already is. Treating it as a *factory intake surface* rather than a chat feature makes messages a new source of intents and a new approval channel on the existing lifecycle, without duplicating that lifecycle inside a messaging provider.

The distinction matters because the obvious implementation is wrong in a way that is expensive to reverse. If provider identifiers reach the core, or task state lives in chat threads, then the second provider is a rewrite and chat history becomes load-bearing infrastructure that no one can migrate. Research for this change is in `docs/inbox-factory-extensibility.md`.

## What Changes

- Add a provider-neutral `factory-intake` capability: an envelope, a dedupe rule, a durable intake store, and an explicit promotion transition from message to tracked work.
- Add `channel-adapter-telegram` as the first transport adapter, which is the only component permitted to know Telegram's vocabulary.
- Record every inbound message permanently, before any interpretation, with credential-shaped strings scrubbed at ingress.
- Make the default outcome of an inbound message a proposal awaiting approval, never a directly created initiative. Two read-only exceptions: research requests and status requests.
- Derive dedupe keys from message content and identity, never from provider sequence numbers.
- Add a boundary check to the acceptance path so the intake seam is enforced mechanically rather than by review attention.

## Capabilities

### New Capabilities

- `factory-intake`: A provider-neutral intake surface that records inbound messages durably, deduplicates them, and governs their promotion into tracked work.
- `channel-adapter-telegram`: A transport adapter that maps Telegram updates into intake envelopes and delivers outbound responses.

## Impact

- Adds an intake store under local state, alongside the existing `ambient/queue.json` and `memory/global.json` state files.
- Introduces the first embedded database dependency in the workspace. No `rusqlite`, `sqlx`, `libsql`, `redb`, `sled`, or `duckdb` currently appears in any workspace `Cargo.toml`; this is a new precedent and is justified in `design.md` by measured dedupe-lookup cost, not by convention.
- Adds a Telegram bot credential to the existing secret-handling path.
- Does not change the initiative lifecycle, the ambient queue, approval semantics, or any existing command. Intake feeds the existing lifecycle; it does not replace or parallel it.
- Does not grant chat any authority the terminal does not already have. Intake is deliberately *stricter* than terminal input, because it is the lowest-friction input path in the system.
