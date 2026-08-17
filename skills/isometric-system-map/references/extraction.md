# Evidence extraction guide

This guide is optimized for Bicep repositories, but the evidence discipline applies to every stack.

## Start with the deployment boundary

1. Resolve the exact commit and requested ref.
2. Find the deployment entry point and its caller: pipeline YAML, wrapper script, CLI, or parent
   Bicep module.
3. Read the full root file before following child modules.
4. List owned resources, `existing` resources, module blocks, scopes, conditions, outputs, and
   explicit/implicit dependencies.
5. Trace only the paths needed to explain the chosen scope.

Useful searches:

```bash
rg -n "^(targetScope|module |resource |output |param |import )|dependsOn|existing =" <bicep-root>
rg -n "az deployment|AzureResourceManagerTemplateDeployment|what-if|template-file|bicepparam" .
rg -n "trigger: none|pr: none|condition:|approval|manual|held|not deployed|gated" .
```

Use `nl -ba <file>` or an equivalent line-numbered read when authoring citations.

## Bicep mapping

| Evidence | Node or edge |
| --- | --- |
| Pipeline stage/job invoking Bicep | `pipeline` node and `delivery` edge |
| Subscription/RG creation or approval boundary | `governance` node when ownership/gating matters |
| Root or reusable Bicep module | `module` node |
| App Service/Function/Container/VM resources | `compute` node |
| SQL/Storage/Cosmos/Redis/analytics resources | `data` node |
| Service Bus/Event Grid/queue/topic/SignalR | `messaging` node |
| VNet/subnet/NSG/route/PE/private DNS/firewall | `network` node |
| UAMI/Key Vault/role assignment/auth authority | `identity` node |
| LA/App Insights/AMPLS/alerts/dashboard | `observability` node |
| `existing` or cross-repo-owned resource | usually `external` status/node |
| `dependsOn` or module output-to-param reference | `dependency` edge |
| VNet/subnet/PE/DNS/peering relationship | `network` edge |
| Principal/role/secret-reference relationship | `identity` edge |
| Diagnostic settings or monitoring sink | `telemetry` edge |

Aggregate tightly coupled resources into one building when they form one operational unit. A SQL
server, pool, and databases can be one data building; a VNet plus its four load-bearing subnets may
be one network building if the flow does not require individual subnet selection. Split when the
ownership, status, or path differs.

## Dependency versus runtime flow

Bicep proves deployment topology very well and runtime application behavior only partially.

Safe runtime data evidence includes:

- app settings or connection references naming a target service;
- private endpoint/subnet/DNS wiring that names source and target;
- queue/topic/entity manifests consumed by a workload;
- explicit role assignments coupled to a data-plane resource;
- application or integration code in the same analyzed scope.

Unsafe inference:

- `module A dependsOn module B`, therefore A sends business data to B;
- two resources share a resource group, therefore they communicate;
- a private endpoint exists, therefore every compute resource uses it;
- a Bicep file exists, therefore it is deployed;
- a pipeline is present, therefore it runs automatically.

When runtime data is unproven, render a `dependency`, `network`, `identity`, or `delivery` edge
instead of inventing `data`.

## Status and ownership

Preserve explicit source qualifiers:

- `trigger: none`, manual parameters, environment conditions, approval stages, “held”, “do not
  deploy”, or missing service-connection prerequisites can justify `held`.
- `existing` resources and comments assigning ownership elsewhere justify `external`.
- A conditional module may be `active` in one selected environment and `held` in another. State the
  environment in `repository.scope` or choose one environment.
- Deprecated/archive trees should normally be excluded. Include one only when it explains a live
  migration or compatibility path, and mark it `deprecated`.

## Payload selection

Every flow needs one inspectable payload. Prefer a concrete name and shape:

| Flow | Suitable payload |
| --- | --- |
| Deployment | environment + secure inputs + module params + output resource IDs |
| Network/private access | source subnet + target private endpoint + private DNS resolution |
| Identity | principal ID + role definition + target scope |
| Messaging | queue/topic message or manifest-defined entity reference |
| Telemetry | diagnostic event/metric/trace envelope to its sink |
| Runtime data | request/record/blob only when code or configuration proves it |

Never place real secret values or private sample records in the map. Describe the schema and cite the
safe declaration.

## Flow curation patterns

Useful Bicep-centered flows include:

1. **Guarded deployment:** pipeline → approval/what-if → root module → domain module → resource.
2. **Network-first foundation:** network deployment → subnet/DNS outputs → private endpoints →
   shared services.
3. **Identity bootstrap:** managed identity → Key Vault/role assignment → compute consumer.
4. **Messaging fabric:** manifest/root module → namespace → queue/topic → workload binding.
5. **Observability path:** workload/resource → diagnostic setting/App Insights → Log Analytics →
   dashboard/alerts.

Choose flows the repository actually proves. A map with three precise flows is stronger than one
with six speculative flows.

## Layout pass

Lay out by reading direction rather than Azure icon familiarity:

- rear-left: entry, pipeline, governance;
- center: root/domain modules, network, identity;
- front/right: compute, messaging, data, observability, external sinks;
- place sequential flow steps on distinct grid points;
- reserve one empty cell around large `network`, `data`, and `governance` buildings;
- use short stable `code` labels such as `CI`, `NET`, `KV`, `APP`, `SB`, `SQL`, `LA`.

The renderer handles perspective and silhouettes. Do not encode custom dimensions or CSS.

