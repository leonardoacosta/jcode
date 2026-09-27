## Context

The observed request lifecycle logs show `subscribe_start` followed by server stall warnings, with no member-registration or subscribe-completion event. Subscribe registration uses shared member/index maps. Coordinator removal previously read both maps in one scope, creating an avoidable cross-map lock-order hazard.

## Goals / Non-Goals

**Goals:** Avoid holding both maps during coordinator election and make future subscribe lock stalls observable by stage.

**Non-Goals:** Change provider routing, session persistence, protocol events, or swarm coordinator selection policy.

## Decisions

- Copy the swarm member IDs while holding only the swarm-index read guard, release it, then inspect the member map. This preserves the existing best-effort election semantics and removes nested map-lock acquisition.
- Emit structured lifecycle events before and after the relevant subscribe map-lock acquisitions and after recording a join event. This gives the next runtime trace a precise last-completed stage.

## Risks / Trade-offs

- The two independent snapshots can reflect concurrent membership changes. The existing implementation already performs asynchronous membership updates, so election remains best-effort; missing or headless members are filtered as before.
- Additional lifecycle events increase log volume slightly. They are limited to subscribe registration and carry existing session/connection identifiers.

## Migration Plan

Run focused server tests, build the self-dev binary, run an isolated-socket subscribe smoke test, then restart the shared daemon. No session files or database rows are migrated.
