## Why

The dark canvas now embeds the Brown Wholesale Bicep-derived scene, but its full-estate topology is visually unreadable: Cloudflare and APIM are not centered, VNet and subnet containment is absent from the canvas, private endpoints compete with resource cubes, functions overlap their endpoint relationships, and the complete audited estate is not legible as a network map.

## What Changes

- Refine the dark canvas into a complete Brown Wholesale and Decus 537 estate view while preserving the existing validated scene facts and evidence.
- Render both VNets as visible isometric terrain boundaries with nested subnet regions, labels, and CIDRs.
- Center Cloudflare and APIM as a shared ingress band above the two VNet territories.
- Place all 28 audited resources according to their VNet/subnet or resource-group relationship, including NVA association and currently obscured storage, messaging, identity, data, telemetry, configuration, and DevOps resources.
- Render private endpoints as small labeled foreground dots positioned in front of their associated PaaS resources, with short private-link connectors into the resource.
- Preserve private endpoint containment within the correct subnet while keeping associated PaaS resources outside subnet regions unless source evidence proves direct integration.
- Keep the runtime request path visually emphasized without hiding non-runtime estate resources or inventing relationships.

## Non-Goals

- Do not modify the source Bicep repository or the pinned Markdown audit.
- Do not change the scene ontology, evidence claims, or resource inventory without source-backed corrections.
- Do not replace the existing Network and ADO projections.
- Do not add new aggregate traffic arrows or infer unsupported peering, application, or database relationships.

## Impact

Affects the isometric Canvas renderer/theme, Brown Bicep scene fixture and checked-in dark example, scene validation/rendering tests, and topology documentation. Paper and Azure directional examples remain unchanged unless a shared renderer contract requires compatible updates.
