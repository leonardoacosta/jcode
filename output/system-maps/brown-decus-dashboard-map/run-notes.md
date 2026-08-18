# Brown Decus dashboard map, consolidated final pass

## Scope and identity

- Output directory: `/home/nyaptor/dev/jcode/output/system-maps/brown-decus-dashboard-map/`.
- Repository identity in both sidecars: `brown-wholesale`, ref/commit `46be8f57ead08d0f957c4af7ff9321224a668f6a`.
- Evidence boundary: the Decus/Wholesale traffic audit plus cited Brown Wholesale Bicep and Azure DevOps definitions, read-only.
- `map.html` was intentionally not rendered or updated in this pass.
- No tracked skill, OpenSpec, test, documentation, Decus directory, or commit was changed.

## Approved ontology applied

- Nodes are deployable Azure resources or application hosts only. VNets, subnets, CIDRs, resource groups, subscriptions, and peering remain Network containment or links.
- Wholesale 346 is the shared Cloudflare and API Management edge and decomposed shared foundation. Decus 537 is a distinct East US 2 foundation.
- Private endpoints remain scene nodes and are direct members of their evidenced DATA subnet. PaaS resources remain resource-group scoped.
- APIM APIs, backend configuration, policies, products, named values, subscriptions, and partner relations remain metadata on the APIM node. No standalone configuration nodes were invented.
- No Aggregates, DecusDirect, collapsed shared-infrastructure node, or unsupported direct application-to-database path was added.
- Decus hub peering is an explicit `held` Network link, not an active path or resource cube.
- Evidence levels are explicit as `direct`, `inferred`, or `held` on scene paths and Network links.
- Runtime remains a focused four-layer story: ingress, directly evidenced projects, Wholesale data access/telemetry, and external egress.
- The review page defaults to the Network projection; direct `#runtime` and `#ado` links remain available.
- The ADO pipeline projection is sidecar-only and is not mixed into the actual Network topology.
- The redundant aggregate incoming-traffic flow has been removed; APIM/backend and network relationships remain represented by their individual evidenced paths.

## Final counts

- `scene.json`: 2 VNet areas, 28 resource cubes, 22 directed paths, 4 payload definitions, 1 focused flow.
- `views.json` Runtime: 19 selected nodes, 18 selected paths, 1 selected flow.
- `views.json` Network: 19 containers, 26 memberships, 13 links.
- `views.json` ADO: 2 actual Brown pipelines, 19 evidence-backed stages/jobs/gates, 16 directed edges.
  - Wholesale pipeline stages include `LinkBetsVnet`, `TestBetsDnsResolution`, `F_BicepCache`, parallel `F_Foundation` jobs, `D_Data`, `D_Compute`, `R_Routing`, `SatelliteInfra`, and `D_ObsExtras`.
  - Fireball pipeline stages include conditional prod `Approval`, `CodeDeploy` jobs `BuildDeployMatrix`, `BuildUI`, and `RunTests`, followed by `APIMRouteSync`, `HealthCheck`, and `MigrateApimSubKeys`.

## Citations represented

- Decus/Wholesale audit: `docs/diagrams/decus-wholesale-bicep-traffic-audit.md`, including executive conclusions, traffic diagram, estate breakdown, direct/inferred paths, and intentionally omitted relationships at lines 15-23, 25-156, 158-215.
- Wholesale network: `bicep/foundation/network/vnets/VNET-WHS-346-CentralUS-DEV.bicep` and `bicep/foundation/network/consts.bicep`.
- Wholesale APIM and application backends: `bicep/foundation/wholesale/wholesale-rg/routing/apim/index.bicep`, `.../apis/fireball-eventpublisher.bicep`, and `.../apis/fireball-bridgeepay.bicep`.
- Wholesale messaging, identity, data, storage, and telemetry: `.../messaging/service-bus.bicep`, `.../core/keyvault.bicep`, `bicep/foundation/wholesale/data/sql-private-endpoint.bicep`, `.../data/main.bicep`, `.../compute/function-storage.bicep`, and `.../routing/apim/loggers.bicep`.
- Actual ADO pipeline definitions: `.azuredevops/build/satellites/wholesale.yml`, `.azuredevops/build/satellites/fireball.yml`, `.azuredevops/build/_templates/approval-stage.yml`, and `.azuredevops/build/_templates/bicep-deploy-step.yml`.

## Validation

Passed exactly:

```text
python3 skills/isometric-system-map/scripts/validate_scene.py \
  output/system-maps/brown-decus-dashboard-map/scene.json
valid isometric scene: 28 cubes, 22 paths, 1 flows

python3 skills/isometric-system-map/scripts/validate_views.py \
  output/system-maps/brown-decus-dashboard-map/views.json \
  output/system-maps/brown-decus-dashboard-map/scene.json
exit 0, no diagnostics

python3 -m unittest \
  skills.isometric-system-map.tests.test_scene \
  skills.isometric-system-map.tests.test_views \
  skills.isometric-system-map.tests.test_views_ontology \
  skills.isometric-system-map.tests.test_views_schema
Ran 34 tests in 0.063s
OK
```

The only intermediate validation issue was two non-admitted icon names in the new ADO stages. They were replaced with package-local `az-static-web-app` and `az-app-insights`; the final validators and all 34 relevant tests pass.

## Pinned evidence reconciliation

- Decus VNet and subnet CIDRs cite `bicep/_noship/reference/decus-537-eastus2/vnet.json` at the pinned commit.
- The unsupported external managed-pool pipeline citation was removed; the pool is marked external and cited only to the approved audit depiction.
- The Decus VNet Integration CIDR is corrected to `10.218.58.0/26`.
