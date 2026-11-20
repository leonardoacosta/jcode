# Design

## Context
See proposal.md. Cached MCP registration currently awaits a health snapshot before spawning connection work; Subscribe awaits registration before SessionId and subsequent GetHistory. The pool holds its connecting mutex across handles/client-map acquisition. The live lock owner is unknown; this design addresses the established dependency and reproducible convoy rather than claiming a proven deadlock.

## Goals / Non-Goals
**Goals:** bootstrap remains responsive under MCP contention, cached proxies precede the first tool snapshot, and leadership/backoff/redaction semantics remain intact.

**Non-Goals:** manager dispatch changes, shutdown redesign, reference-count reordering, general cancellation recovery, watchdog changes, shared-daemon reload, publication.

## Decisions
- Initial cached McpStatus carries empty health, matching existing initialization; existing background events supply live health. Moving all registration into a task would race initial tool snapshots and is rejected.
- Pool connection coordination must not retain its global mutex while waiting for other maps. Preserve per-server leadership, publication-before-notification, matching-entry removal, and cancellation safety. Do not trade the convoy for an orphaned reservation.
- Health uses independently scoped snapshots of connecting names, connected handles, and cloned errors. It is diagnostic rather than an atomic transaction; preserve state precedence and fingerprint filtering.
- Reuse TestHomeGuard, storage environment locks, schema-cache helpers, existing local MCP fixtures, and notification tests. Tests use explicit gates instead of sleeps for synchronization.

## Risks / Trade-offs
- Temporarily empty health omits failure details until background sampling → keep cached counts and test eventual live status delivery.
- Leadership refactoring can lose notifications or strand followers → cover connected fast path, concurrent leaders, both notification orderings, and cancellation at new waits.
- Background work can outlive temporary test state → stop fixtures and await owned tasks before dropping environment guards.
- Concurrent repository work can invalidate observations → recheck exact target diffs before edits; do not switch the primary checkout.

## Migration Plan
No migration or configuration changes. Validate the local binary against a separate socket and isolated JCODE_HOME without external MCP or model calls. Do not repoint installed binaries or restart the shared daemon. Rollback is a targeted reversal of this repair only, preserving unrelated edits.

## Decisions Record
- Local repair and isolated acceptance; no shared-runtime restart or publication: approved plan, decided-by: leo.
