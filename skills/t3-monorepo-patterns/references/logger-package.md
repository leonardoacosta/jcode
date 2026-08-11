
# Logger Package

Codified from a production monorepo pattern. A representative implementation belongs in
`packages/logger/src/index.ts`.

`packages/logger` is a pure leaf (depends only on `pino` and `@opentelemetry/api`) that wires
pino's `mixin()` to inject the active OTel trace/span IDs into every log line. This gives every
log entry a `traceId` and `spanId` automatically — making Sentry-to-log joins trivial in
production.

```typescript
// packages/logger/src/index.ts shape:
import pino from "pino";
import { trace } from "@opentelemetry/api";

export const logger = pino({
  mixin() {
    const span = trace.getActiveSpan();
    if (!span) return {};
    const ctx = span.spanContext();
    return { traceId: ctx.traceId, spanId: ctx.spanId };
  },
});
```

The tRPC middleware (`packages/api/src/trpc.ts` § timingMiddleware) bolts a per-request child
logger onto `ctx.logger` via `withRequestContext()`, so every router has a request-scoped logger
with `requestId`, `path`, `userId` baked in — without thinking about it.

**Rule:** T3 projects that ship to production SHOULD include `packages/logger` with this shape.
`console.log` is acceptable in dev-only scripts (seeders, one-off backfills).
