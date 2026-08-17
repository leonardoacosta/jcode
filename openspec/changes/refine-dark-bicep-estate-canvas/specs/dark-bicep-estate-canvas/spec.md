## MODIFIED Requirements

### Requirement: Full estate is readable in the dark canvas
The dark canvas MUST represent every resource in the validated Brown Bicep scene without visual pile-up that makes labels or identities indistinguishable.

#### Scenario: All audited resources remain visible
- **WHEN** the Brown Bicep scene is rendered with the dark theme
- **THEN** all 28 scene nodes remain present, selectable, and associated with their source identity
- **AND** storage, Service Bus, Key Vault, SQL, pools, telemetry, App Configuration, NVA, and Managed DevOps Pool are not omitted or hidden behind unrelated cubes.

### Requirement: Centered edge ingress
Cloudflare and APIM MUST form a centered ingress band above the estate territories.

#### Scenario: Ingress is visually centered
- **WHEN** the dark canvas is viewed at desktop size
- **THEN** Cloudflare and APIM occupy a centered top ingress composition
- **AND** the primary runtime path proceeds from that band into the appropriate VNet resources without a redundant aggregate arrow.

### Requirement: Visible VNets and subnets
The canvas MUST render both VNet boundaries and their nested subnet regions as isometric terrain.

#### Scenario: Network containment is visible
- **WHEN** the map is rendered
- **THEN** Wholesale 346 and Decus 537 VNet boundaries are labeled and visually distinct
- **AND** each evidenced subnet is rendered as an inset region with its name and CIDR
- **AND** resources and endpoint markers appear within the correct containment region where the scene evidence requires it.

### Requirement: Foreground private endpoints
Private endpoints MUST render as small labeled dots or markers in front of their associated resources, not as full resource cubes.

#### Scenario: Private link association is legible
- **WHEN** a private endpoint and its associated PaaS resource are rendered
- **THEN** the endpoint marker is visually in front of the resource
- **AND** a short connector visibly runs from the marker into the resource
- **AND** the endpoint remains associated with its evidenced subnet while the PaaS resource remains outside that subnet unless direct integration is evidenced.

### Requirement: Correct NVA association and function layering
The NVA MUST be visually associated with its owning VNet, and function resources MUST not be occluded by their private endpoint relationships.

#### Scenario: Layering preserves resource identity
- **WHEN** Wholesale and Decus territories are rendered
- **THEN** the NVA is inside or explicitly attached to its associated VNet territory
- **AND** Function App cubes remain readable with endpoint connectors and markers in the foreground rather than unrelated cubes obscuring them.

### Requirement: Evidence and compatibility remain intact
The redesign MUST preserve scene validation, evidence metadata, deterministic rendering, and existing scene-only compatibility.

#### Scenario: Existing contracts still pass
- **WHEN** the updated scene and renderer tests run
- **THEN** scene validation, full topology tests, and deterministic artifact checks pass
- **AND** the renderer does not invent unsupported relationships or modify the source audit.
