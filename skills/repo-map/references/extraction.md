# repo-map Extraction Guides

You emit **only contract JSON** (`references/contract.md`) — never HTML/CSS, never prose
alongside the JSON. Read the stack guide below matching the region you were assigned, map what
you find onto the D1 role vocabulary, then author `detail`/`source_paths`/`flows` per the
doctrines further down. Only real, observed behavior — never invent a node, edge, or flow.

## C#

Investigation semantics for ASP.NET Core:

- `[ApiController]`-attributed controllers and minimal-API `MapGet`/`MapPost`/`MapGroup`
  registrations are the request surface — one node per controller or route group, never per
  action method.
- `IHostedService`/`BackgroundService` implementations (and Hangfire recurring jobs) are the
  scheduled/background surface.
- DI registrations (`Program.cs`/`Startup.cs` `AddScoped`/`AddSingleton`) reveal a service's
  real collaborators — trace them to find store/external edges, but never node-ify the
  container itself.
- Typed `HttpClient` registrations (`AddHttpClient<T>`) name the external system being called;
  the client's base address or vendor name is the node label.
- An EF Core `DbContext` (with its `DbSet<T>` properties) is the store — one node per
  `DbContext`, never per table.

| Finding | Kind |
| --- | --- |
| Controller / minimal-API route group | `service` |
| `IHostedService` / `BackgroundService` / Hangfire job | `worker` |
| Typed `HttpClient` target | `external` |
| EF Core `DbContext` | `store` |

## Next.js

Investigation semantics for the App Router:

- A top-level route segment's `page.tsx`/`layout.tsx` is the user-facing surface — one node per
  route family (e.g. `/checkout`), never per dynamic segment.
- Route handlers (`route.ts`) and Server Actions (`"use server"` functions) are the
  request-driven backend surface those pages call.
- Vercel Cron entries (`vercel.json` schedules or their target route handlers) are the
  scheduled surface.
- AI SDK call-sites split three ways: the orchestrating loop (`generateText`/`streamText` with
  tool-calling, an agent loop) is the orchestrator; the model id passed into it is the model
  invoked; each `tool()` definition is a callable the orchestrator invokes.

| Finding | Kind |
| --- | --- |
| Page/layout route segment | `entry` |
| Route handler / Server Action | `service` |
| Cron-triggered route handler | `worker` |
| AI SDK orchestration loop | `agent` |
| AI SDK model argument | `model` |
| AI SDK `tool()` definition | `tool` |

## Go

Investigation semantics:

- `net/http` `Handle`/`HandleFunc` registrations and `gorilla/mux`/`chi` route tables are the
  request surface — one node per route group.
- A `go func()` loop reading from a channel, or a dedicated worker package started from `main`,
  is a background processor.
- The broker a worker consumes from (a Kafka topic, an SQS queue, a NATS subject) is the
  async-transport node in its own right — kept separate from the goroutine that consumes it.
- A `sql.DB`/`pgxpool.Pool` or Redis client held by a service is its store.

| Finding | Kind |
| --- | --- |
| `http.Handle`/mux route group | `service` |
| goroutine worker / dedicated worker package | `worker` |
| Message broker consumed (Kafka/SQS/NATS topic or queue) | `queue` |
| `sql.DB` / `pgxpool.Pool` / Redis client | `store` |

## Swift

Investigation semantics:

- The `App` conformance / root `Scene` (or `AppDelegate`) is the client-facing entry — one node
  for the whole app shell, not per view.
- `BGTaskScheduler` registrations and background `URLSession` configurations (background
  transfer, silent-push handling) are the background surface.
- A `URLSession` service is investigated by its target: hitting your own backend, it's a
  request to a service; hitting a third-party API/SDK, it's external — same call-site pattern,
  the target decides the kind.
- A local persistence layer (Core Data `NSPersistentContainer`, SwiftData `ModelContainer`, a
  Keychain wrapper) is the store.

| Finding | Kind |
| --- | --- |
| App/`Scene` entry point | `entry` |
| `BGTaskScheduler` / background `URLSession` | `worker` |
| `URLSession` client → own backend | `service` |
| `URLSession` client → third-party API | `external` |
| `NSPersistentContainer` / `ModelContainer` / Keychain wrapper | `store` |

## Bicep

Investigation semantics:

- A `module` block is a reusable IaC unit — one node per module reference. `dependsOn`
  (explicit, or implicit via a parameter/output reference) becomes an edge between the two
  module/resource nodes it connects.
- A `resource` block targeting a managed Azure service the module doesn't own outright (Service
  Bus, Storage, Key Vault, Cosmos DB) is external, tagged with its Azure resource type as
  `domain`.
- Params and outputs crossing a module boundary become edges — never node-ify a parameter.

| Finding | Kind |
| --- | --- |
| `module` block | `module` |
| Externally-managed `resource` (Service Bus, Storage, Key Vault, Cosmos DB, ...) | `external` |

## Terraform

Investigation semantics:

- A `module` block (local or registry-sourced) is the reusable unit.
- A `resource` provisioning a managed queue/broker (`aws_sqs_queue`, `google_pubsub_topic`,
  `azurerm_servicebus_queue`) is a queue; a `resource` provisioning a datastore
  (`aws_db_instance`, `aws_s3_bucket`, `google_storage_bucket`) is a store.
- The reference graph is the source of truth for edges — explicit `depends_on` AND any
  `<resource>.<attr>` interpolation both count, not `depends_on` alone.
- A `provider` block or a resource for a third-party SaaS (Datadog, PagerDuty, Cloudflare) not
  owned by this repo is external.

| Finding | Kind |
| --- | --- |
| `module` block | `module` |
| Resource provisioning a queue/broker (SQS, Pub/Sub, Service Bus) | `queue` |
| Resource provisioning a datastore (RDS, S3, Cloud Storage) | `store` |
| Third-party SaaS resource / provider not owned by this repo | `external` |

## Curation

Favor the few flows that matter over an exhaustive dependency dump. Node at the grain each
table above describes — one node per controller/route-group/module/DbContext, never per file
or per action — and stop well inside the contract's caps (`nodes` ≤ 24, `edges` ≤ 48; full
table in `references/contract.md`). A region that would blow the cap is a sign it should be
split into two enumerate-prompt regions, not compressed into one overloaded node.

## Detail authoring

`detail`'s six keys (`why`, `effects`, `fails`, `sends`, `auth`, `ordering` — shapes and caps in
`references/contract.md`) follow one doctrine: **`why` is the headline purpose, never a
paraphrase of the label; omit any key you're unsure of — blank is correct, never guess.** Author
`detail` only on nodes where the extra context earns its place (a node with non-obvious
failure/auth/ordering behavior) — not on every node reflexively.

## source_paths authoring

`source_paths` is what `--freshness` (D6) diffs against, so it must be specific: the file or
directory that actually implements the node, never the whole repo and never a single incidental
import. For a route-group/controller, the file or directory containing it; for a module, the
module's directory; for an external client, the file registering it — never the vendor's own
repo. Over-broad paths make unrelated changes look stale; over-narrow paths let real changes to
the node go undetected.

## Flow curation

Name the 3-6 journeys that matter (checkout, login, onboarding) — same discipline as the
contract's `flows[]` cap. Don't register a flow with zero tagged edges. Tag only the edges that
actually participate in that journey; a single edge may carry more than one flow id.

## Enumerate prompt

Modeled on `skills/blueprint/enumerate-prompt.md`, adapted from sequence flows to architecture
regions. Handed to ONE agent before the D9 multi-select gate.

Your job is to produce the **menu** of architecture regions worth mapping. You do **not** map
anything yet — you return a JSON list the command shows the user to pick from.

Read the repo's surface: entry points, top-level route/controller groups, background job
definitions, IaC root modules, top-level directories. Cover the whole repo.

Output — ONLY a JSON array, one item per region:

```json
[
  { "id": "checkout-api", "title": "Checkout API + worker",
    "subtitle": "one line: what this region covers",
    "source_paths": ["services/checkout", "services/checkout-worker"],
    "size": "M" }
]
```

Rules:

- **EXCLUDE anything already mapped.** You will be given the existing map's node ids and
  `source_paths` — return only what's missing or new.
- **One region per cohesive architectural area** (a service + its worker + its store, a UI
  surface + its API routes, one IaC stack). Don't split one region into several entries, and
  don't merge two unrelated regions into one.
- **`source_paths` must be specific** — the files/dirs that actually implement the region, so
  `--freshness` can re-check it later from git.
- **Only REAL regions** — read the code; never invent a region.
- Output the JSON array and nothing else (no prose, no fences if avoidable).

## Per-region prompt

Modeled on `skills/blueprint/scenario-prompt.md`. Handed to each region agent in the D9 fan-out;
each reads only its own region.

You extract **exactly one** region as contract JSON. You'll be given the region's `id`, `title`,
and `source_paths`, the repo path, and the path to the existing master scan JSON (if any).

Steps:

1. **Read the existing master file.** Note node `id`s, `kind`s, and `group`s already in use and
   reuse them wherever the same real component appears — this keeps merge-by-id stable and
   avoids duplicate nodes across regions.
2. **Read the `source_paths`** (and what they reference) using the matching stack guide above;
   trace only THIS region's nodes and edges.
3. **Author `detail`/`source_paths`/`flows`** per the doctrines above.
4. **Emit only this region's nodes and edges** (plus any new `flows[]` entries this region
   introduces) — never the full contract object, and never touch `rails`/`stats`/`provenance`
   (the render step recomputes those after every region is merged).

Output: a JSON object `{ "nodes": [...], "edges": [...], "flows": [...] }` scoped to this region
only — no prose, no top-level `version`/`rails`/`stats`/`provenance` keys.
