# repo-map Scan Contract

Single source of truth for scan JSON. An extraction agent emits data matching this contract;
`scripts/bin/repo-map-render --validate` implements every rule below. Adapted from
foglamp-labs/foglamp `packages/contracts/src/scan.ts` (Apache-2.0) + functioncall/blueprint
`validate_master.py` (MIT) — vocabulary and shapes broadened for our fleet, mechanics preserved.

## Version

```json
{ "version": 1 }
```

`version` is a literal `1`. Any other value is a structural error.

## Top-level shape

```json
{
  "version": 1,
  "nodes": [ Node, ... ],
  "edges": [ Edge, ... ],
  "flows": [ Flow, ... ],
  "rails": { "topModels": [...], "topTools": [...], "topIntegrations": [...] },
  "stats": { "controlNodes": 0, "intelligenceNodes": 0, "dataNodes": 0, "infraNodes": 0 },
  "provenance": { "git_sha": "..." }
}
```

Unknown top-level keys are rejected (structural error) — same as every nested object below.

## Node

| Field | Type | Cap | Required |
| --- | --- | --- | --- |
| `id` | string | — | yes, unique |
| `label` | string | 28 chars | yes |
| `kind` | enum | — | yes |
| `domain` | string | — | no (brand/service domain, e.g. `stripe.com` — drives favicon lookup) |
| `sub` | string | 40 chars | no (role-clarifying subtitle rendered under the label — never a technology name) |
| `group` | string | 24 chars | no (pipeline-stage cluster key; nodes sharing a `group` render inside one box) |
| `source_paths` | string[] | — | yes (repo-relative paths/dirs this node was extracted from — see D6 freshness) |
| `detail` | object | see below | no |
| `src` | string | — | no (single file-provenance path for the detail card footer) |

### Kind enum (exactly 10 — roles, never technologies)

| Kind | Role |
| --- | --- |
| `entry` | User- or client-facing entry point (web app, CLI, mobile shell) |
| `service` | Request-driven backend service (API, controller-based service) |
| `worker` | Background/scheduled processor (cron, `IHostedService`, goroutine consumer) |
| `agent` | AI agent/orchestrator that plans and calls `model`/`tool` nodes |
| `model` | LLM/AI model invoked by an agent — folds to a parent chip in the renderer |
| `tool` | Callable tool/function an agent invokes — folds to a parent chip in the renderer |
| `store` | Persistent data store (database, blob storage, cache) |
| `queue` | Async transport (message queue, event bus, channel) |
| `external` | Third-party/external system outside this repo's control |
| `module` | IaC unit (Bicep module, Terraform module) |

## Edge

| Field | Type | Cap | Required |
| --- | --- | --- | --- |
| `from` | string | — | yes (must resolve to a node `id`) |
| `to` | string | — | yes (must resolve to a node `id`) |
| `label` | string | 24 chars | no |
| `flows` | string[] | — | no (each entry must resolve to a registered `flows[].id`) |

An edge into a `model`/`tool` node stays in the data even though the renderer folds it to a
chip — folding is a render-time concern, never a contract-time one.

## Flows registry

```json
{ "flows": [ { "id": "checkout", "label": "Checkout" } ] }
```

`flows[]` — max **6** entries. Each: `id` (string, unique), `label` (string, 24 chars). Curate
the 3-6 journeys that matter; do not register a flow with zero tagged edges.

## Rails

```json
{
  "rails": {
    "topModels": [ { "id": "...", "label": "...", "domain": "..." } ],
    "topTools": [ { "id": "...", "label": "...", "domain": "..." } ],
    "topIntegrations": [ { "id": "...", "label": "...", "domain": "..." } ]
  }
}
```

Each rail entry is `{ id, label, domain? }` — `id` should reference a real node id where one
exists. Rails are pre-ranked and pre-capped by the extracting agent; the renderer draws them
verbatim, never re-derives or re-sorts.

| Rail | Max entries |
| --- | --- |
| `topModels` | 3 |
| `topTools` | 10 |
| `topIntegrations` | 10 |

## Stats

Four integer counts, each `>= 0`, summing to the total node count. Kinds are bucketed into four
rough classes (feeds the D5 personality-archetype scoring in `references/renderer.md`):

| Stat | Kinds counted |
| --- | --- |
| `controlNodes` | `entry`, `service`, `worker`, `agent` |
| `intelligenceNodes` | `model`, `tool` |
| `dataNodes` | `store`, `queue` |
| `infraNodes` | `external`, `module` |

## Provenance

```json
{ "provenance": { "git_sha": "<40-char HEAD sha at render time>" } }
```

`git_sha` is stamped by `repo-map-render` post-render (never authored by the extraction agent).
Drives `--freshness` (design.md D6): unreachable SHA fails safe (everything reported stale).

## Detail object

All keys optional; unknown keys rejected. Each present key's value is a string capped at
**160 characters**. Authoring doctrine (see `references/extraction.md`): `why` is the headline
purpose, never a paraphrase of `label`; omit any key you're unsure of — blank is correct, a
guess is not.

| Key | Meaning |
| --- | --- |
| `why` | Headline purpose of this node |
| `effects` | What changes/happens when this node runs |
| `fails` | What happens on failure |
| `sends` | What this node sends downstream and to whom |
| `auth` | Authentication/authorization this node enforces or relies on |
| `ordering` | Ordering/concurrency guarantees this node depends on or provides |

## Caps table

| Element | Cap |
| --- | --- |
| `nodes[]` | 24 |
| `edges[]` | 48 |
| `flows[]` | 6 |
| `rails.topModels[]` | 3 |
| `rails.topTools[]` | 10 |
| `rails.topIntegrations[]` | 10 |
| `node.label` | 28 chars |
| `node.sub` | 40 chars |
| `node.group` | 24 chars |
| `edge.label` | 24 chars |
| `flow.label` | 24 chars |
| `detail.<key>` | 160 chars |

## Integrity rules

- Every `edge.from`/`edge.to` MUST reference an existing `node.id`.
- Every `node.id` MUST be unique.
- Every `edge.flows[]` entry MUST reference a registered `flows[].id`.
- `detail` keys MUST be drawn only from the six-key allowlist above.
- Unknown keys are rejected at every level: top-level, node, edge, flow, rail entry, detail,
  provenance.

## Error vs warning split

| Violation class | Result |
| --- | --- |
| Dangling edge reference, duplicate node id, unregistered flow tag, unknown key, wrong `version`, exceeding a count cap (`nodes`/`edges`/`flows`/rail arrays) | **Error** — exit 1, one `path: message` line per violation |
| Exceeding a length cap (`label`/`sub`/`group`/`detail.<key>`) | **Warning** — reported, does not fail the run |

## Complete literal JSON example

A small realistic repo: a Next.js web app fronting a Go API gateway, a background worker, an
AI support agent (with a folded model + tool), a Postgres store, a job queue, a Stripe
integration, and a Terraform module provisioning the gateway. Exercises groups, folded
model/tool nodes, `detail`+`src`, flows, rails, and provenance.

```json
{
  "version": 1,
  "nodes": [
    {
      "id": "web-app",
      "label": "Next.js Web App",
      "kind": "entry",
      "sub": "Customer-facing storefront",
      "group": "frontend",
      "source_paths": ["apps/web"]
    },
    {
      "id": "api-gateway",
      "label": "API Gateway",
      "kind": "service",
      "group": "backend",
      "source_paths": ["services/gateway"],
      "detail": {
        "why": "Single entry point for all storefront and support traffic",
        "effects": "Authenticates the request, routes to a store, queue, or agent",
        "fails": "Returns 503; client retries with backoff",
        "auth": "Validates a signed session cookie before routing"
      },
      "src": "services/gateway/main.go"
    },
    {
      "id": "job-worker",
      "label": "Job Worker",
      "kind": "worker",
      "group": "backend",
      "source_paths": ["services/worker"]
    },
    {
      "id": "assistant-agent",
      "label": "Assistant Agent",
      "kind": "agent",
      "group": "ai",
      "source_paths": ["services/gateway/agent"],
      "detail": {
        "why": "Answers support questions and can search the web for current info"
      },
      "src": "services/gateway/agent/assistant.go"
    },
    {
      "id": "gpt-4o",
      "label": "GPT-4o",
      "kind": "model",
      "group": "ai",
      "domain": "openai.com",
      "source_paths": ["services/gateway/agent"]
    },
    {
      "id": "web-search-tool",
      "label": "Web Search Tool",
      "kind": "tool",
      "group": "ai",
      "source_paths": ["services/gateway/agent/tools"]
    },
    {
      "id": "postgres-db",
      "label": "Postgres DB",
      "kind": "store",
      "group": "backend",
      "source_paths": ["services/gateway/db"]
    },
    {
      "id": "job-queue",
      "label": "Job Queue",
      "kind": "queue",
      "group": "backend",
      "source_paths": ["services/gateway/queue"]
    },
    {
      "id": "stripe-api",
      "label": "Stripe API",
      "kind": "external",
      "domain": "stripe.com",
      "source_paths": ["services/worker/billing"],
      "detail": {
        "sends": "Charge requests for completed orders",
        "fails": "Order marked payment-pending; worker retries 3x"
      }
    },
    {
      "id": "tf-network-module",
      "label": "Network Module",
      "kind": "module",
      "group": "infra",
      "source_paths": ["infra/modules/network"]
    }
  ],
  "edges": [
    { "from": "web-app", "to": "api-gateway", "label": "HTTP", "flows": ["checkout"] },
    { "from": "api-gateway", "to": "assistant-agent", "label": "invoke", "flows": ["support"] },
    { "from": "assistant-agent", "to": "gpt-4o", "label": "completion", "flows": ["support"] },
    { "from": "assistant-agent", "to": "web-search-tool", "label": "search", "flows": ["support"] },
    { "from": "api-gateway", "to": "postgres-db", "label": "query", "flows": ["checkout"] },
    { "from": "api-gateway", "to": "job-queue", "label": "enqueue", "flows": ["checkout"] },
    { "from": "job-worker", "to": "job-queue", "label": "consume", "flows": ["checkout"] },
    { "from": "job-worker", "to": "stripe-api", "label": "charge", "flows": ["checkout"] },
    { "from": "tf-network-module", "to": "api-gateway", "label": "provisions" }
  ],
  "flows": [
    { "id": "checkout", "label": "Checkout" },
    { "id": "support", "label": "AI Support" }
  ],
  "rails": {
    "topModels": [ { "id": "gpt-4o", "label": "GPT-4o", "domain": "openai.com" } ],
    "topTools": [ { "id": "web-search-tool", "label": "Web Search Tool" } ],
    "topIntegrations": [ { "id": "stripe-api", "label": "Stripe API", "domain": "stripe.com" } ]
  },
  "stats": {
    "controlNodes": 4,
    "intelligenceNodes": 2,
    "dataNodes": 2,
    "infraNodes": 2
  },
  "provenance": {
    "git_sha": "a1b2c3d4e5f6a7b8c9d0e1f2a3b4c5d6e7f8a9b0"
  }
}
```
