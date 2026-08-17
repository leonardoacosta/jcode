## ADDED Requirements

### Requirement: Coordinated topology views
The renderer SHALL support a views-enabled Azure topology artifact with separate Runtime, Network, and ADO Pipeline views while preserving scene-only rendering compatibility.

#### Scenario: Render a views-enabled artifact
- **WHEN** a valid scene and matching companion view sidecar are rendered with the Azure theme
- **THEN** one self-contained HTML artifact exposes Runtime, Network, and ADO Pipeline tabs
- **AND** each tab renders only its declared projection

#### Scenario: Render a scene without companion views
- **WHEN** the existing three-argument renderer invocation is used without a companion sidecar
- **THEN** the renderer produces the existing standalone scene behavior without view tabs

#### Scenario: Reject a mismatched sidecar
- **WHEN** the scene and companion sidecar repository ref or commit do not match
- **THEN** validation fails with a stable diagnostic before output is written

#### Scenario: Validate the normative sidecar grammar
- **WHEN** a companion sidecar is supplied
- **THEN** it conforms to tracked JSON Schema 2020-12 contract version 1
- **AND** unknown keys, invalid enums, duplicate IDs, unknown references, containment cycles, duplicate membership, cyclic pipeline edges, and missing direct evidence fail with stable field-specific diagnostics

### Requirement: Approved SVG resource identity
Azure topology outputs SHALL make admitted package-local SVG marks and full resource labels the primary visible identity and SHALL NOT depend on cube abbreviations for comprehension.

#### Scenario: Render an Azure runtime resource
- **WHEN** a visible Runtime node has an admitted icon
- **THEN** its SVG occupies at least 78 percent of the cube roof's shorter world edge
- **AND** a visible label plate shows its full resource label and concise service type
- **AND** the Azure theme does not draw `node.code`

#### Scenario: Render a flat resource or stage card
- **WHEN** a resource appears in Network or a primitive appears in ADO Pipeline
- **THEN** the card displays an admitted SVG at least 24 CSS pixels wide in the primary desktop layout
- **AND** its full name and resource or stage type are visible without hover

#### Scenario: Encounter an unsupported icon
- **WHEN** a scene or sidecar references an icon outside the admitted sprite and declared mappings
- **THEN** validation fails instead of rendering an abbreviation or guessed service mark

#### Scenario: Use a declared family fallback
- **WHEN** an evidenced service has no service-specific admitted mark but has an explicit admitted family mapping
- **THEN** the mapped family SVG is rendered with the full service name and resource type
- **AND** the family mark never replaces or abbreviates the visible textual identity
- **AND** `azure-tokens.json` supplies both the ARM-resource-type-to-family mapping and family-to-admitted-symbol mapping without a sidecar override

### Requirement: Canonical Azure topology ontology
Azure topology outputs SHALL follow the approved pinned Bicep traffic-audit ontology and SHALL NOT promote configuration, network boundaries, or unevidenced infrastructure into resource cubes.

#### Scenario: Represent deployable resources and network boundaries
- **WHEN** a topology includes deployable Azure services, application hosts, VNets, subnets, CIDRs, or peerings
- **THEN** only the deployable services and evidenced application hosts render as uniform resource cubes or resource cards
- **AND** VNets and subnets render as containment areas, CIDRs render as fields on those areas, and peerings render as labeled edges

#### Scenario: Represent a private endpoint and its PaaS service
- **WHEN** source evidence binds a private endpoint to a subnet and a PaaS service
- **THEN** the private endpoint resource renders inside the subnet
- **AND** the PaaS resource remains outside the subnet unless direct evidence proves the service itself is integrated there

#### Scenario: Keep configuration as metadata
- **WHEN** APIM APIs, products, policies, named values, subscriptions, configurations, or partner relations are discovered
- **THEN** they enrich the owning APIM resource's metadata and evidence details
- **AND** they do not render as standalone topology nodes

#### Scenario: Distinguish evidence strength and deployment state
- **WHEN** a relationship is directly evidenced, inferred, or gated/not deployed
- **THEN** a direct relationship is solid, an inferred relationship is visibly non-solid and labeled, and a gated/not-deployed relationship is non-animated and explicitly marked
- **AND** an unsupported relationship is omitted rather than implied by proximity

#### Scenario: Preserve legacy Runtime path compatibility
- **WHEN** a core scene path omits `evidence_level`
- **THEN** the renderer and validator treat it as `direct`
- **AND** explicit `inferred` or `held` values use the same non-solid, labeled, and non-animated semantics as Network links

### Requirement: Focused Runtime projection
The Runtime view SHALL retain the isometric request-flow language while excluding support-plane elements that do not participate in the declared runtime story.

#### Scenario: Render request traffic
- **WHEN** a Runtime projection declares node and path IDs
- **THEN** every traffic-layer member remains visible in bottom-left-to-top-right order
- **AND** only declared runtime paths are rendered
- **AND** pipeline-control, resource-group boundary, and network-only abstractions are absent unless explicitly included

#### Scenario: Read resource identity without interaction
- **WHEN** the Runtime view first loads
- **THEN** every visible cube has a readable full label and service type without requiring focus, hover, or selection

### Requirement: Evidence-backed Network topology
The Network view SHALL render explicit Azure containment and network relationships from the companion sidecar.

#### Scenario: Render nested containment
- **WHEN** evidence supports Subscription, Resource Group, VNet, Subnet, and resource membership
- **THEN** the view nests the corresponding containers in that order
- **AND** every resource card appears inside its declared parent container
- **AND** subnet CIDRs appear as fields on the subnet container rather than resource nodes

#### Scenario: Render a network relationship
- **WHEN** the sidecar declares a peering, private endpoint, DNS link, or network data path
- **THEN** the view renders a directed or bidirectional orthogonal connector with a visible intent label
- **AND** the relationship exposes its structured evidence
- **AND** its direct, inferred, or held evidence level determines a text-redundant line treatment

#### Scenario: Reject invalid containment
- **WHEN** container parents form a cycle, a member is unknown, or a link target is missing
- **THEN** validation fails with a stable diagnostic before rendering

#### Scenario: Use a narrow viewport
- **WHEN** the Network view is displayed at 320 CSS pixels wide or 200 percent zoom
- **THEN** top-level containers remain reachable and readable without horizontal clipping
- **AND** a text relationship summary remains available if connector geometry cannot be drawn

### Requirement: Evidence-backed ADO Pipeline topology
The ADO Pipeline view SHALL render source-backed delivery stages, parallelism, dependencies, gates, and deployment targets as a directed stage graph.

#### Scenario: Render a complete delivery path
- **WHEN** evidence supports repository, validation/build, artifact, gate, and deployment stages
- **THEN** the view renders those stages in directed order with admitted CI/CD SVGs and full labels
- **AND** every edge displays its transition intent

#### Scenario: Render parallel jobs
- **WHEN** multiple stages or jobs share a declared parallel group
- **THEN** the view places them in parallel lanes rather than a false serial chain

#### Scenario: Render manual, approval, or held behavior
- **WHEN** evidence identifies a manual queue, approval gate, policy prerequisite, or held stage
- **THEN** the relevant stage and transition display explicit text status and evidence
- **AND** state is not communicated by color alone

#### Scenario: Reject unsupported pipeline claims
- **WHEN** a stage or edge lacks direct structured evidence
- **THEN** validation fails rather than inventing delivery behavior

#### Scenario: Require direct evidence provenance
- **WHEN** a container, membership, network link, pipeline stage, or pipeline edge is declared
- **THEN** that exact object contains at least one non-empty `path`/`lines`/`claim` evidence entry
- **AND** evidence is not inherited from a parent, adjacent resource, or implied relationship

### Requirement: Shared identity, selection, and evidence
Views-enabled artifacts SHALL share stable semantic identity and complete evidence presentation across projections.

#### Scenario: Select a shared resource
- **WHEN** a resource card or Runtime cube is selected
- **THEN** the selected semantic ID is retained while switching tabs
- **AND** every view containing that ID renders the selected treatment

#### Scenario: Inspect evidence
- **WHEN** a user focuses or selects a node, container, stage, or connector
- **THEN** a persistent details region displays its full label, type, status, relationship, every citation, and every claim
- **AND** hover is not required to access the information

### Requirement: Accessible and URL-addressable view navigation
Views-enabled artifacts SHALL provide native, keyboard-accessible tabs and resilient static content.

#### Scenario: Navigate tabs by keyboard
- **WHEN** focus is on the tab list
- **THEN** Left/Right Arrow, Home, and End move focus according to the tab pattern
- **AND** Enter or Space activates the focused tab
- **AND** visible focus remains present

#### Scenario: Open a deep link
- **WHEN** an artifact opens with `#runtime`, `#network`, or `#ado`
- **THEN** the corresponding tab is active
- **AND** switching tabs updates the URL fragment without a page reload

#### Scenario: Load without JavaScript enhancement
- **WHEN** JavaScript does not execute
- **THEN** full resource, containment, stage, relationship, and evidence text remains present and navigable in document order

### Requirement: Deterministic local-only generation
Views-enabled artifacts SHALL be reproducible, self-contained, and protected by semantic validation.

#### Scenario: Regenerate an unchanged artifact
- **WHEN** unchanged scene, sidecar, theme, template, and admitted assets are rendered twice
- **THEN** the resulting HTML bytes and recorded scene/views semantic hashes are identical

#### Scenario: Inspect asset loading
- **WHEN** a generated artifact is opened from `file://` or a static server
- **THEN** it makes no remote asset or runtime data requests
- **AND** only used admitted SVG symbols are embedded

#### Scenario: Publish Brown and Decus review artifacts
- **WHEN** the Brown and Decus private outputs are generated
- **THEN** each page exposes Runtime, Network, and ADO Pipeline views
- **AND** the gallery links directly to every view and previews the configured default Network view
- **AND** desktop and narrow browser acceptance checks complete without console errors, clipped controls, or inaccessible content

#### Scenario: Preserve the audited Brown and Decus estate boundary
- **WHEN** the private Brown and Decus review artifacts are generated from the pinned audit snapshot
- **THEN** Wholesale 346 is shown as the shared Cloudflare and APIM edge and Decus 537 is shown as a separate landing-zone and data foundation
- **AND** Aggregates and DecusDirect compute hosts are absent until deployment evidence is supplied
- **AND** Decus hub peering is held/not deployed rather than shown as an active resource or solid traffic path

#### Scenario: Deliver a reproducible private review bundle
- **WHEN** a Brown or Decus private artifact is published to the private gallery
- **THEN** its directory contains `scene.json`, `views.json`, `map.html`, `generation-receipt.json`, and `run-notes.md`
- **AND** the deterministic receipt records the exact command and SHA-256 values for every generation input and output
- **AND** regeneration from the delivered bundle inputs reproduces the HTML and receipt byte-for-byte

#### Scenario: Run canonical browser acceptance
- **WHEN** the tracked Chromium acceptance command runs against the generic, Brown, and Decus artifacts
- **THEN** it exercises file and loopback-HTTP loading, all direct view fragments, keyboard tab operation, selection retention, desktop, 320 CSS-pixel, 200-percent zoom, reduced-motion, and JavaScript-disabled modes
- **AND** it verifies direct/inferred/held line treatment and labels in Runtime and Network
- **AND** it opens the private gallery and verifies every deep link plus each default Network preview
- **AND** it fails on console or page errors, horizontal clipping, inaccessible controls, missing document-order fallback content, or unexpected network requests
