# Isometric system-map contract

The renderer accepts one strict JSON object. Unknown fields are errors. The same valid JSON renders
deterministically to the same HTML bytes.

## Top-level object

```json
{
  "version": 1,
  "repository": {},
  "palette": "midnight",
  "zones": [],
  "nodes": [],
  "edges": [],
  "flows": []
}
```

| Field | Rule |
| --- | --- |
| `version` | Literal `1` |
| `repository` | Evidence boundary and prose summary |
| `palette` | `midnight` or `paper` |
| `zones` | 1-8 logical or operational regions |
| `nodes` | 1-24 buildings |
| `edges` | 1-48 directed paths |
| `flows` | 1-6 inspectable payload journeys |

## Repository

```json
{
  "name": "acme-infra",
  "ref": "origin/main",
  "commit": "0123456789abcdef0123456789abcdef01234567",
  "scope": "infra/foundation/main.bicep",
  "summary": "Subscription deployment that creates shared network, compute, data, and logging resources."
}
```

All five strings are required. `commit` is a 7-40 character hexadecimal Git object ID. `scope`
describes the bounded surface being visualized, not necessarily one file.

## Zone

```json
{
  "id": "control-plane",
  "label": "Control plane",
  "description": "Pipelines, approvals, and deployment orchestration"
}
```

`id` is unique lowercase kebab-case. `label` and `description` are required. A zone is a navigation
and explanation group; it does not claim Azure containment unless the evidence does.

## Node

```json
{
  "id": "shared-sql",
  "code": "SQL",
  "label": "Shared SQL estate",
  "kind": "data",
  "zone": "runtime",
  "position": { "x": 7, "y": 4 },
  "status": "active",
  "purpose": "Hosts pooled application databases.",
  "behavior": "Accepts private connections from approved workload subnets.",
  "implementation": "A server, elastic pool, databases, and optional private endpoint are composed by the data module.",
  "source_paths": [
    "infra/data/main.bicep:18-96",
    "infra/foundation/main.bicep:72-90"
  ]
}
```

### Node fields

| Field | Rule |
| --- | --- |
| `id` | Unique lowercase kebab-case |
| `code` | 1-5 character map label, unique when practical |
| `label` | Human-readable component name |
| `kind` | One value from the kind table below |
| `zone` | Must reference `zones[].id` |
| `position.x`, `position.y` | Unique integer grid point, each `0..12` |
| `status` | `active`, `held`, `external`, or `deprecated` |
| `purpose` | Why the component exists |
| `behavior` | What it does at runtime or deployment time |
| `implementation` | How the source builds or configures it |
| `source_paths` | At least one repo-relative citation |

### Node kinds

The renderer maps kinds to distinct footprints, heights, colors, and details. Kinds describe roles,
not vendor product names.

| Kind | Use for |
| --- | --- |
| `entry` | CLI, UI, API ingress, externally initiated entry |
| `pipeline` | Build, validation, approval, deployment pipeline |
| `governance` | Subscription/RG ownership, policy, explicit gate |
| `module` | Reusable IaC or orchestration module |
| `network` | VNet, subnet, route, firewall, private-link terrain |
| `compute` | App Service, Functions, containers, VM workloads |
| `data` | SQL, storage, cache, analytics stores |
| `identity` | Managed identity, Key Vault, auth authority |
| `messaging` | Queue, topic, event bus, SignalR-style transport |
| `observability` | Logs, traces, metrics, dashboards, alerts |
| `external` | Existing or externally owned service/system |

### Node status

- `active`: source evidence shows this is the live/normal path.
- `held`: source explicitly says gated, disabled, not deployed, awaiting approval, or conditional in
  the selected environment.
- `external`: referenced but owned outside the mapped scope/repository.
- `deprecated`: retained only because it affects current understanding; normally omit it.

Never infer `active` from file presence alone.

## Edge

```json
{
  "id": "pipeline-to-foundation",
  "from": "deploy-pipeline",
  "to": "foundation-root",
  "type": "delivery",
  "label": "what-if then create",
  "source_paths": [".azuredevops/deploy.yml:34-82"]
}
```

All fields are required. `from` and `to` must resolve to node IDs. `source_paths` needs at least one
repo-relative citation.

### Edge types

| Type | Meaning |
| --- | --- |
| `control` | Command, trigger, orchestration, or sequencing instruction |
| `data` | Runtime application or service data movement |
| `delivery` | Build/validation/deployment progression |
| `dependency` | Compile/deploy dependency, module output/reference |
| `identity` | Token, principal, role, secret reference, auth relationship |
| `network` | Peering, route, subnet attachment, private endpoint, DNS path |
| `telemetry` | Logs, metrics, traces, alerts, dashboards |

Do not use `data` for `dependsOn`, module outputs, or ordinary deployment parameters.

## Flow

```json
{
  "id": "guarded-release",
  "label": "Guarded release",
  "summary": "A pipeline previews the change, deploys the root template, and materializes the workload dependency chain.",
  "payload": {
    "label": "ARM deployment",
    "description": "Environment parameters and module outputs crossing the deployment path.",
    "schema": "environment + secure inputs → module params → resource IDs",
    "source_paths": [
      ".azuredevops/deploy.yml:34-82",
      "infra/foundation/main.bicep:1-120"
    ]
  },
  "steps": [
    {
      "edge": "pipeline-to-foundation",
      "label": "Preview and deploy",
      "detail": "The pipeline runs the guarded deployment entry point.",
      "source_paths": [
        ".azuredevops/deploy.yml:34-82",
        "infra/foundation/main.bicep:1-40"
      ]
    }
  ]
}
```

Each flow has 1-12 ordered steps. `steps[].edge` must reference an edge. `label` is the short action;
`detail` explains the evidence-backed transition; and `source_paths` directly cites the claim made by
that step. One edge may appear in multiple flows.

The payload is the inspectable moving dot. It may represent a deployment request, command, event,
record, resource ID, telemetry envelope, or other concrete transferred value. Do not call a vague
relationship a payload.

## Citation format

Use a repo-relative path with an optional line suffix:

```text
path/to/file.bicep
path/to/file.bicep:42
path/to/file.bicep:42-81
```

Absolute paths, parent traversal, URLs, secret values, and prose-only citations are invalid.

## Integrity checklist

- IDs are unique and every reference resolves.
- Grid positions are unique.
- Every node, edge, payload, and flow step has citations.
- Every flow has at least one step.
- Every selected node participates in the architecture story, not visual balance.
- Status and ownership are sourced facts.
- Data, dependency, delivery, network, identity, and telemetry are not conflated.
