
# Architecture Templates

Load this file when producing architecture documentation. Do NOT load for decision-making phases.

---

## Component Template

```markdown
## Component: {Name}

### Responsibility
{One sentence: what this component does and why it exists}

### Owns
- Data: {tables/collections}
- Processes: {business logic encapsulated}

### Interfaces
- Exposes: {APIs, events, queues provided}
- Consumes: {APIs, events, queues depended on}

### NFR Targets
- Latency: {P95 < Xms}
- Throughput: {N req/s}
- Availability: {X%}
```

---

## Interface Contract Template

```markdown
### Interface: {Component A} --> {Component B}

Protocol: {REST/gRPC/Event/Queue}
Auth: {JWT/mTLS/API key}
Frequency: {calls/sec at peak}

| Operation | Input | Output | Latency Target | Error Handling |
|-----------|-------|--------|----------------|----------------|
| {name} | {request shape} | {response shape} | {P95 < Xms} | {retry/circuit-break/dead-letter} |

#### Failure Scenarios
| Failure | Detection | Recovery |
|---------|-----------|----------|
| {Target} unavailable | Timeout Xms | Retry 3x exponential backoff |
| {Target} slow | P95 > Xms | Circuit breaker opens |
| Invalid payload | Validation error | Dead letter + alert |
| Partial failure | Inconsistent state | Compensating transaction |

Contract Evolution: {versioning strategy, breaking change policy}
```

---

## Data Flow Template

```markdown
### Data Flow: {Name}

Source: {where data originates}
Volume: {records/sec, GB/day}
Latency Requirement: {real-time / near-real-time / batch}

| Stage | Technology | Input | Output | SLA |
|-------|-----------|-------|--------|-----|
| Ingest | {Kafka/API/Webhook} | {raw} | {validated} | {< Xms} |
| Transform | {Worker/Stream} | {validated} | {enriched} | {< Xms} |
| Store | {Postgres/S3/Redis} | {enriched} | {queryable} | {durable} |
| Serve | {API/Cache/CDN} | {query} | {response} | {< Xms P95} |

Data Contracts:
- Schema: {JSON Schema / Protobuf / Avro}
- Evolution: {backward compatible / versioned}
```

---

## State Transition Template

```markdown
### Entity: {Name}

States: {list of valid states}
Transitions:
  {State A} --> {State B}: {trigger, who can trigger, side effects}
  {State B} --> {State C}: {trigger, who can trigger, side effects}

Invariants:
  - {rule that must always hold}

Invalid Transitions:
  - {State C} --> {State A}: {why this is forbidden}
```

---

## Failure Mode Template

```markdown
### Failure Mode: {What fails}

- Impact: {What users experience}
- Detection: {Metrics, alerts, health checks}
- Mitigation: {Automatic — retry, circuit breaker, fallback}
- Recovery: {Manual steps if mitigation fails}
- Prevention: {Testing, chaos engineering}
```
