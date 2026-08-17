# Repository extraction for isometric scenes

The visual quality depends on selecting the right semantic graph. Do not choose visual cubes first and
then search for facts to justify them.

## Evidence boundary

1. Resolve the requested ref and immutable commit.
2. Identify the selected architecture scope.
3. Find its callers, deployment entry points, and downstream outputs.
4. Read line-numbered source before authoring evidence objects.
5. State when the snapshot is local or cannot be proven fresh.

Useful commands:

```bash
git rev-parse <ref>
rg -n "entry|main|deploy|pipeline|workflow|dependsOn|existing|output|condition" .
nl -ba path/to/file
```

Do not mutate a read-only source repository just to fetch or build it. Use an immutable archive or
supplied snapshot when necessary.

## Trace architecture before curating it

Build a temporary inventory with these columns:

| Item | Questions |
| --- | --- |
| initiator | What starts the path? Manual queue, commit, event, CLI, request? |
| boundary | Subscription, resource group, service, process, or ownership boundary? |
| dependency | What must exist first? Is it compile/deploy/runtime? |
| runtime path | What value or request actually moves? |
| traffic layer | Is this APIM ingress, an owned project service, data access, or an external service? |
| external | Which resource is existing, cross-repo, or owned by another team? |
| status | Active, conditional, held, external, deprecated? What line proves it? |
| environment | Shared, per-environment, or overlay-only? |
| representation | Which node, path, status, or evidenced boundary will make this fact visible? |
| evidence | Exact path, lines, and supported claim? |

Only after this inventory is coherent should you select scene nodes, one global cube size, reserved
footprints, and sourced containment areas.

## Bicep and Azure Pipelines

### Entry points

Search for:

```bash
rg -n "^(targetScope|module |resource |output |param |import )|dependsOn|existing =" --glob '*.bicep'
rg -n "az deployment|AzureResourceManagerTemplateDeployment|what-if|template-file|bicepparam" .azuredevops
rg -n "^trigger:|^pr:|stages:|jobs:|dependsOn:|condition:|environment:|approval" .azuredevops
```

Read the full root Bicep and the relevant pipeline stage graph. A module block alone does not explain
when or why it deploys.

### Map source evidence to scene roles

| Evidence | Likely role or path |
| --- | --- |
| API Management or equivalent public/private request gateway | `entry` node in traffic layer 1 `ingress` |
| pipeline stage/job invoking Bicep | `pipeline` node, `delivery` or `control` path |
| approval or policy boundary | `governance` cube or control path |
| root/reusable Bicep module | `module` node |
| VNet plus attached resources | sourced `vnet` area with the VNet cube and attached resources in `member_ids` |
| subnet, NSG, route, PE, private DNS | `network` node or network path; also a VNet-area member when attachment is directly evidenced |
| App Service, Function, container, VM | `compute` node |
| SQL, storage, cache, analytics | `data` node |
| Service Bus, topic, queue, event service | `messaging` node |
| UAMI, Key Vault, role assignment, auth authority | `identity` node or identity path |
| Log Analytics, App Insights, alerts, dashboards | `observability` node or telemetry path |
| `existing` resource or cross-repo reference | usually `external` status |

For incoming-request topologies, classify the visible corridor before placing coordinates:

1. APIM or the equivalent gateway is the `entry` node in `ingress`.
2. Every owned project or application service that handles the request belongs to `projects`.
3. Every represented role `data` node, plus directly used configuration, identity, messaging, and
   private data dependencies, belongs to `data-access`.
4. Every represented role `external` node belongs to `external-services`.

Do not put a deployment pipeline in ingress merely because it starts a release. Do not put APIM in the
external-services layer merely because the selected repository does not own it; request position and
ownership status are separate facts, so an externally owned APIM can have role `entry` and status
`external`. Support-plane nodes can remain outside the corridor.

Group repetitive resources into one cube when the scene story is about the shared module or
fan-out behavior. Split them only when independent ownership, ordering, or payload routes matter.
Also split them whenever an explicit user requirement depends on environment identity, scope,
ownership, contract version, or lifecycle. Text inside a generic aggregate node does not preserve a
required topology distinction.

### Capture ARM type and roof icon

For each concrete Azure resource node, capture the exact type from its Bicep resource declaration or
the called module's declared resource. Strip the API version before storing `resource_type`:

```bicep
resource app 'Microsoft.Web/sites@2023-12-01' = { ... }
```

becomes:

```json
{
  "resource_type": "Microsoft.Web/sites",
  "icon": "az-app-service"
}
```

Use [`../assets/azure-tokens.json`](../assets/azure-tokens.json) for semantic family mapping and
[`../assets/azure-icons.svg`](../assets/azure-icons.svg) for admitted symbol IDs. Common mappings:

| Resource or delivery primitive | `resource_type` | `icon` |
| --- | --- | --- |
| App Service | `Microsoft.Web/sites` | `az-app-service` |
| SQL database | `Microsoft.Sql/servers/databases` | `az-sql-database` |
| Key Vault | `Microsoft.KeyVault/vaults` | `az-key-vault` |
| API Management | `Microsoft.ApiManagement/service` | `az-apim` |
| Service Bus | `Microsoft.ServiceBus/namespaces` | `az-service-bus` |
| VNet | `Microsoft.Network/virtualNetworks` | `az-vnet` |
| Subnet | `Microsoft.Network/virtualNetworks/subnets` | `az-subnet` |
| Private endpoint | `Microsoft.Network/privateEndpoints` | `az-private-endpoint` |
| App Insights | `Microsoft.Insights/components` | `az-app-insights` |
| Log Analytics | `Microsoft.OperationalInsights/workspaces` | `az-log-analytics` |
| Azure DevOps deployment stage | `Azure DevOps pipeline` | `az-release-pipeline` |

Do not infer a concrete service icon from a module filename alone. Follow the module and cite the
resource declaration. If a curated node groups several services, choose a neutral family-aligned icon
only when the grouping claim is explicit; otherwise omit the icon.

### Dependency is not data flow

Classify each relationship before drawing it:

- `dependsOn`, module output references, scopes, and deployment stage ordering are `dependency` or
  `delivery` paths.
- resource IDs, secure parameters, principals, and role bindings are `identity` or `dependency`
  paths unless they represent runtime auth traffic.
- private endpoints, subnets, DNS links, routes, and peerings are `network` paths.
- diagnostic settings and log forwarding are `telemetry` paths.
- application requests, records, messages, events, or commands are `data` or `control` paths.

Do not turn every Bicep parameter into a moving payload.

## Status and ownership rules

Status is evidence, not mood:

- `trigger: none` or `pr: none` means manually initiated unless other evidence says blocked. It does
  not make the pipeline `held`.
- a Bicep `if (...)` condition usually means `conditional`, not held.
- placeholder CIDRs, disabled stages, explicit no-op jobs, pending service connections, or comments
  stating not deployed can support `held`.
- `existing` commonly supports `external`, but confirm whether ownership is outside the selected
  scene or merely outside that module.
- approval is a gateway in an active flow unless approval cannot currently be satisfied.
- do not infer active deployment from the presence of a `.bicep` file.

Record ownership boundaries as scene facts only when source or authoritative documentation supports
them.

## Environment topology

Environment mapping often contains architecture that a root module hides. Inspect:

- constants and lookup objects;
- parameter files;
- service-connection selectors;
- region and resource-group naming;
- dev/test/stage sharing versus production separation;
- prod-only or nonprod-only modules;
- central hub plus environment overlays;
- placeholder or live-scanned network ranges.

If several environments share one object, do not draw one copy per environment. If production has a
small overlay, do not draw a second complete topology.

The inverse matters too: do not collapse a distinct production object into generic non-prod text.
Represent one shared object once, then add each genuinely distinct object or overlay as its own node.
When a resource group, subscription, or team boundary is material, use an evidenced boundary
boundary cube or a distinct sourced region. A visual zone label or prose mention alone is not proof
that the boundary survived curation.

## Requirement coverage gate

Before layout, map every explicit prompt requirement and decisive source-backed distinction to a
scene element. Keep the table in the run notes:

| Requirement or fact | Node/path/boundary/status | Evidence | Checked in scene JSON? |
| --- | --- | --- | --- |

Use these rules when checking it:

- preserve exact names and flags when they carry architecture meaning, such as API versions, product
  names, `subscriptionRequired`, environment selectors, or placeholder CIDRs;
- model external imports and cross-team prerequisites as external nodes and dependency/control paths;
- for request-oriented maps, check that APIM, projects, data access, and external services are assigned
  to the four ordered traffic layers and that their projected centers progress bottom left to top right;
- model central hubs and environment overlays separately without duplicating the whole stack;
- model shared versus distinct environment objects as independently identifiable nodes;
- require route-claim congruence: the cited evidence must support the rendered source, target, kind,
  and payload together;
- do not count a description or citation claim as coverage when the corresponding geometry is absent.

If the normal 8-24 node target conflicts with an explicit requirement, preserve the requirement up to
the contract cap before considering aggregation.

## Pipeline DAG extraction

Write the stage graph in text before drawing it:

```text
queue -> approval? -> compile/cache -> foundation jobs -> data/compute -> satellite fan-out -> alerts
```

Check whether jobs are parallel inside one stage or separate top-level stages. Check which branches
run in parallel after a shared dependency. Preserve no-op stages and drain/convergence controls when
they explain orchestration behavior.

Useful evidence includes:

- `dependsOn` arrays;
- stage and job nesting;
- conditions;
- artifact publish/download steps;
- manifest-driven loops;
- approvals and environment gates;
- terminal alert or convergence jobs.

## Payload selection

Choose values that actually cross a path:

- deployment request or compiled template artifact;
- environment and secure parameter bundle, described without values;
- resource ID or module output;
- identity principal or token relationship;
- network session or DNS resolution;
- command, event, message, or record;
- log, metric, trace, or alert envelope.

A payload description names its shape and role, never a secret value.

## Curation pass

Select resource cubes that satisfy at least one criterion:

- initiates a named flow;
- changes ordering or ownership;
- fans out to multiple dependencies;
- receives or transforms a concrete payload;
- marks a network, identity, data, or telemetry boundary;
- explains a surprising environment overlay or gate;
- is the terminal output of the selected architecture story.

Omit decorative or duplicate nodes. A 14-cube scene with four clear flows is usually more useful
than a 40-cube inventory.

## Evidence audit

Before rendering:

- every node has direct evidence for role, status, and description;
- every path has evidence for direction and kind;
- every payload has evidence for its existence or shape;
- every flow step has evidence for that transition;
- every environment-sharing or ownership statement is cited;
- no claim relies solely on a filename or directory name;
- no secret or private identifier is copied into the scene.
