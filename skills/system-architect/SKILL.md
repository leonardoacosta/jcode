---
name: system-architect
description: "Design system architecture: component decomposition, interface contracts, data flow, state management, consistency strategies, and failure modes. Use when planning new systems, evaluating architecture decisions, defining component boundaries, designing data pipelines, or choosing consistency models."
source: ~/.agents/skills@2026-07-13
license: MIT
metadata:
  version: 2.2.0
  tags: [architecture, system-design, components, interfaces, data-flow, consistency, state-management, nfr]
---


# System Architect

Design robust system architectures with clear component boundaries, well-defined interfaces, explicit data flows, and principled consistency choices.

---

## Triggers

| Trigger | Example |
|---------|---------|
| Architecture design | "Design the architecture for X" |
| Component boundaries | "How should we decompose this into services?" |
| Interface contracts | "Define the API contract between A and B" |
| Data flow planning | "Design the data pipeline for X" |
| Consistency strategy | "How do we keep data consistent between A and B?" |
| State management | "Where does this state live and how does it change?" |
| NFR analysis | "What are the scalability concerns for Z?" |
| Migration planning | "How do we migrate from monolith to services?" |

---

## Phase Scaling

Not every system needs all 5 phases. Match rigor to scope:

| System Size | Skip | Lightweight | Full |
|-------------|------|-------------|------|
| Single service / MVP | Phases 3-5 | Phases 1-2 | — |
| 2-5 services | Phase 5 | Phases 1, 3 | Phases 2, 4 |
| 6+ services / multi-team | — | — | All phases |
| Migration / restructure | Phase 2 | Phase 1 | Phases 3-5 |

**The litmus test**: If the system has 1 team and <10k LOC, skip straight to Phase 2 with a lightweight Phase 1. Over-documenting a small system wastes more time than under-documenting it.

---

## Phase 1: Context Gathering

Before designing anything, establish:

**Problem Domain**: What problem? Who are the users? What are the critical user journeys? What data flows through the system?

**Constraints**: Existing infrastructure, team expertise, budget (compute + licensing + ops), regulatory requirements, timeline.

**Quality Attributes (NFRs)** — Don't list all NFRs. Identify the 2-3 that constrain your design:

```
"What kills us if we get it wrong?"
├─ Money/permissions wrong → Consistency first
├─ Users leave after 3s → Latency first (P95/P99 targets)
├─ 3am pages cost $10k/hr → Availability first (SLA + downtime cost)
├─ Regulated data (SOC2/HIPAA/GDPR) → Security first
└─ System changes quarterly → Evolvability first (extension points)
```

Everything else is secondary. Trying to optimize all NFRs equally optimizes none.

---

## Phase 2: Component Decomposition

### The Fundamental Decision: Monolith vs Services

```
Is this greenfield?
├─ Yes → Start as modular monolith with clear module boundaries
│   └─ Extract to services ONLY when:
│      • Independent scaling needed (one module 100x traffic, others idle)
│      • Different deployment cadence (one module ships hourly, others weekly)
│      • Team ownership boundary (Conway's Law — org structure drives architecture)
└─ No → Existing system being restructured?
    ├─ Strangler fig: new functionality in new services, migrate gradually
    │   └─ Requires: API gateway or routing layer to split traffic
    └─ Big bang rewrite: NEVER unless system is small (<10k LOC) AND team is <5
        └─ Why: rewrites take 2-3x longer than estimated, and you lose
           institutional knowledge embedded in the "ugly" code

How many teams will own this?
├─ 1 team → Monolith. Period. Microservices add ops overhead without org benefit.
│   Each service needs: CI pipeline, monitoring, deployment config, on-call rotation.
│   For 1 team, that's all overhead with zero coordination benefit.
├─ 2-3 teams → Modular monolith with clear API boundaries between modules
│   └─ Extract to services only at the seams where teams block each other
└─ 4+ teams → Services aligned to team boundaries
    └─ Each team owns 1-3 services max. More = context-switching tax.
```

Design for the "seams" — where components meet. The seams are where all the hard problems live.

For component documentation templates, see `references/templates.md`.

**Checkpoint** — before proceeding to Phase 3, verify:
- [ ] Every component has exactly ONE owner (team or person)
- [ ] No two components share a data store
- [ ] You can explain each component's reason to exist in one sentence
If any fail → revisit decomposition before designing interfaces.

---

## Phase 3: Interface & Interaction Design

### Communication Pattern Decision Tree

```
Does the caller need a response to continue?
├─ Yes (synchronous) →
│   Is it internal service-to-service?
│   ├─ Yes → gRPC (binary, typed, ~10x faster than REST for internal calls)
│   └─ No (external/browser-facing) → REST or tRPC
│
└─ No (asynchronous) →
    Is ordering critical?
    ├─ Yes → Message queue with partitioning (Kafka, SQS FIFO)
    │   └─ Partition key = entity ID to preserve per-entity order
    └─ No →
        Is fan-out needed (multiple consumers)?
        ├─ Yes → Pub-sub (SNS+SQS, Redis Streams, Kafka consumer groups)
        └─ No → Simple queue (SQS, BullMQ, QStash)
```

**NEVER use shared database as integration pattern.** It couples schema evolution, prevents independent deployment, and makes ownership ambiguous.

### Saga Decision Tree

```
Multi-service transaction needed?
├─ Can services be called sequentially with compensation on failure?
│   ├─ Yes, and <4 steps → Orchestration saga (central coordinator)
│   │   └─ Why: easier to reason about, single place to see the flow,
│   │      simpler error handling. The coordinator IS the documentation.
│   ├─ Yes, but 4+ steps → Orchestration saga with state machine
│   │   └─ Why: choreography with 4+ steps becomes untraceable.
│   │      Nobody can draw the full flow from reading event handlers.
│   └─ No, steps are truly independent → Choreography (event-driven)
│       └─ Why: no coordinator to become a bottleneck or SPOF.
│          But: add correlation IDs and a dead-letter topic, or you
│          will lose failed events silently.
└─ Actually, can this be a single database transaction?
    └─ Yes → Do that. Distributed transactions are a last resort, not a first choice.
```

**Checkpoint** — before proceeding to Phase 4, verify:
- [ ] Every interface has an error handling strategy (retry, circuit-break, dead-letter)
- [ ] Sync vs async choice is justified by caller dependency, not preference
- [ ] Saga pattern chosen only after ruling out single-transaction option
If any fail → simplify before adding consistency complexity.

For interface contract and failure scenario templates, see `references/templates.md`.

---

## Phase 4: Data Flow & Consistency

### Consistency Decision Tree

```
What happens if a user reads stale data?

They see wrong money / inventory / permissions →
  Strong consistency (read-your-writes)
  └─ Cost: higher latency, lower throughput, single-region constraint
     Mitigation: strong only for the write path; reads can be eventual
     with a "refresh" button or optimistic UI

They see a feed/dashboard slightly behind →
  Eventual consistency
  └─ Acceptable staleness window: _____ (seconds? minutes? hours?)
     DANGER: "eventual" without a staleness SLA means "whenever, maybe never"
     Always define max acceptable lag

They see their OWN stale data after writing →
  Session consistency (read-your-writes within session)
  └─ Implementation: sticky sessions, write-through cache, or
     read from primary after write (with timeout fallback to replica)

They see messages/events out of order →
  Causal consistency
  └─ Implementation: vector clocks, logical timestamps, or
     partition by conversation/entity to preserve per-entity order
```

**The trap**: "We need strong consistency everywhere" kills throughput. "We need eventual consistency everywhere" causes data corruption in financial flows. The answer is ALWAYS a map: which data needs which model.

### State Management

Every system has state. Be explicit about where it lives and how it transitions.

| State Type | Examples | Storage | Consistency Need |
|-----------|---------|---------|-----------------|
| Source of truth | User records, orders | Primary database | Strong |
| Derived | Aggregations, indexes | Read replicas, caches | Eventual |
| Ephemeral | Sessions, locks | In-memory (Redis) | Session |
| Configuration | Feature flags, settings | Config service / env vars | Eventual |

**State transitions must be explicit.** If you can't draw the state machine, you don't understand the system. Include invalid transitions — they prevent bugs that only appear under race conditions.

For state transition and data flow templates, see `references/templates.md`.

### Critical Data Patterns

| Pattern | When to Use | Why it matters |
|---------|------------|----------------|
| Event Sourcing | Audit trails, temporal queries | Reconstruct any past state. But: read models become complex, and replaying 10M events takes time |
| Outbox Pattern | Reliable event publishing from DB writes | Avoids dual-write problem: DB commits but event publish fails = inconsistent state. Outbox makes it atomic |
| Idempotency Keys | Payment processing, retries, webhooks | Without these, network retries = duplicate charges. Cost of missing this: real money lost |
| Change Data Capture | Keeping services in sync without code changes | Non-invasive: reads DB log, no application code needed. But: couples to DB schema |

---

## Phase 5: Failure Mode Analysis

For each component and interface, document impact → detection → mitigation → recovery → prevention.

The non-obvious failures to always check:

| Failure class | Why it's missed | How to catch it |
|--------------|----------------|-----------------|
| Partial success | API returns 200 but only processed 3 of 5 items | Assert on response counts, not just status codes |
| Slow dependency | Not down, just slow — 30s responses that don't trigger timeouts | P99 latency alerts, not just error rate |
| Clock skew | Distributed timestamps disagree by seconds | Use logical clocks for ordering, wall clocks only for display |
| Poison message | One bad event blocks entire queue | Dead-letter queue + alerting after N retries |
| Cascading retry storm | Service A retries → Service B retries → exponential amplification | Jittered backoff + circuit breakers + retry budgets (max 10% of requests are retries) |

For failure mode documentation template, see `references/templates.md`.

---

## Anti-Patterns

| Anti-Pattern | Why It Fails (the math) | Better Approach |
|-------------|------------------------|-----------------|
| Shared database between services | Schema change in Service A breaks Service B's queries. Deploying A requires testing B. N services × M tables = N×M coupling surface | Each service owns its data store. Sync via events or API |
| Distributed monolith | Network latency (1-10ms/hop) + serialization overhead + no independent deploy. You pay the microservice tax without the microservice benefit | True monolith OR true microservices. The middle is the worst place |
| Premature microservices | Each service needs: CI pipeline, monitoring, dashboards, deployment config, on-call rotation. For 5 services that's 5× ops overhead. One team can't operate 5 services well | Start monolith, extract when a specific force demands it (scale, team boundary, deploy cadence) |
| God service | All traffic funnels through one component. Deploy risk = total risk. Scaling means scaling everything | Decompose by bounded context. The service that "does everything" actually does nothing well |
| Chatty interfaces | 10 calls × 5ms network each = 50ms overhead vs 1 batch call × 8ms. Latency multiplies per hop — 3 hops of 10 calls = 150ms wasted | Batch operations, BFF pattern, or GraphQL for aggregation |
| One consistency model everywhere | Strong consistency at 10k writes/sec requires single-leader DB. At 100k writes/sec you physically can't. Eventual consistency on financial data = incorrect balances | Consistency map: strong for money, eventual for feeds, session for user state |
| Dual writes without outbox | `db.save()` succeeds, `queue.publish()` fails → DB has record, consumers never see event. Retry publishes? Now you might double-publish | Outbox pattern: write event to outbox table in same DB transaction, relay separately |
| Ignoring state transition invariants | Without guards: race condition where two concurrent requests both see state=A, both transition to B and C simultaneously → invalid state | Explicit state machines with optimistic locking or DB-level constraints |
| Designing for 10x scale you don't have | YAGNI. Sharding a database that has 10k rows. Building event-driven architecture for 100 req/min | Design for current + 3x. Document the plan for 10x. Execute the plan at 5x |

---

## Architecture Documentation Output

**MANDATORY**: Load `references/templates.md` before producing any documentation. Do NOT load it during Phases 1-4 (decision-making phases).

The final architecture document should include:

1. **Context Diagram** — System in its environment
2. **Component Diagram** — Internal decomposition with interfaces
3. **Data Flow Diagram** — How data moves through the system
4. **Consistency Map** — Which strategy applies to which data
5. **Decision Log** — Key architecture decisions with rationale
6. **Risk Register** — Known risks with mitigation strategies

Use `documentation-writer`'s `references/mermaid-diagrams/` for diagrams. Use `/c4-architecture` for layered views.

---

## Orchestrator Integration

When used within the `/explore system` workflow:

1. Component decomposition feeds into gepetto's section splitting
2. Interface contracts become implementation task boundaries
3. Data flow design shapes implementation order (data-first)
4. NFR analysis informs the ambiguity audit

The architect does NOT write code. The architect defines what exists, how it connects, and what quality it must achieve. Implementation is handled by engineer agents via `/apply`.
