#!/usr/bin/env python3
"""Validate companion topology view documents for isometric system-map scenes."""

from __future__ import annotations

import argparse
import json
import re
import sys
from pathlib import Path
from typing import Any


ROOT = Path(__file__).resolve().parents[1]
AZURE_ICONS = ROOT / "assets" / "azure-icons.svg"
NETWORK_CONTAINER_KEYS = {"cidr", "evidence", "id", "kind", "label", "parent_id", "status"}
NETWORK_LINK_KINDS = {"peering", "private-endpoint", "dns", "data"}
NETWORK_LINK_DIRECTIONS = {"forward", "reverse", "both"}
NETWORK_LINK_EVIDENCE_LEVELS = {"direct", "inferred", "held"}


def _is_string(value: Any) -> bool:
    return isinstance(value, str) and bool(value.strip())


def _require_object(value: Any, path: str, errors: list[str]) -> bool:
    if not isinstance(value, dict):
        errors.append(f"{path}: required object")
        return False
    return True


def _require_array(value: Any, path: str, errors: list[str]) -> bool:
    if not isinstance(value, list):
        errors.append(f"{path}: required array")
        return False
    return True


def _require_string(value: Any, path: str, errors: list[str]) -> None:
    if not _is_string(value):
        errors.append(f"{path}: required non-empty string")


def _validate_string_array(value: Any, path: str, errors: list[str]) -> None:
    if not _require_array(value, path, errors):
        return
    for index, item in enumerate(value):
        _require_string(item, f"{path}[{index}]", errors)


def _validate_object_array(value: Any, path: str, errors: list[str]) -> None:
    if not _require_array(value, path, errors):
        return
    for index, item in enumerate(value):
        if not isinstance(item, dict):
            errors.append(f"{path}[{index}]: must be an object")


def _validate_unknown_keys(value: dict[str, Any], allowed_keys: set[str], path: str, errors: list[str]) -> None:
    for key in value:
        if key not in allowed_keys:
            errors.append(f"{path}.{key}: unknown key")


def _scene_node_ids(scene: dict[str, Any]) -> set[str]:
    nodes = scene.get("nodes")
    if not isinstance(nodes, list):
        return set()
    return {node.get("id") for node in nodes if isinstance(node, dict) and _is_string(node.get("id"))}


def _scene_nodes_by_id(scene: dict[str, Any]) -> dict[str, dict[str, Any]]:
    nodes = scene.get("nodes")
    if not isinstance(nodes, list):
        return {}
    return {node.get("id"): node for node in nodes if isinstance(node, dict) and _is_string(node.get("id"))}


def _validate_scene_azure_ontology(scene: dict[str, Any], errors: list[str]) -> None:
    nodes = scene.get("nodes")
    if not isinstance(nodes, list):
        return
    for index, node in enumerate(nodes):
        if not isinstance(node, dict):
            continue
        node_id = node.get("id")
        resource_type = node.get("resource_type")
        if not _is_string(node_id) or not _is_string(resource_type):
            continue
        resource_type_lower = resource_type.lower()
        if node.get("role") == "pipeline" or node.get("zone") == "delivery":
            errors.append(
                f"scene.nodes[{index}]: delivery primitive '{node_id}' belongs in the ADO sidecar, not as a views-enabled Azure topology node"
            )
        if resource_type_lower == "microsoft.resources/resourcegroups":
            errors.append(
                f"scene.nodes[{index}]: resource group '{node_id}' belongs in Network containment, not as a topology node"
            )
        if "microsoft.apimanagement/service/apis/policies" in resource_type_lower:
            errors.append(
                f"scene.nodes[{index}]: APIM policy/configuration '{node_id}' is metadata and must not be modeled as a topology node"
            )
        if resource_type_lower in {
            "microsoft.network/virtualnetworks",
            "microsoft.network/virtualnetworks/subnets",
            "microsoft.network/virtualnetworkpeerings",
        }:
            errors.append(
                f"scene.nodes[{index}]: VNet/subnet/peering boundary '{node_id}' must be modeled as an area or edge, not as a topology node"
            )


def _scene_traffic_member_ids(scene: dict[str, Any]) -> list[str]:
    traffic = scene.get("traffic")
    if not isinstance(traffic, dict):
        return []
    layers = traffic.get("layers")
    if not isinstance(layers, list):
        return []

    member_ids: list[str] = []
    for layer in layers:
        if not isinstance(layer, dict):
            continue
        members = layer.get("member_ids")
        if not isinstance(members, list):
            continue
        for member_id in members:
            if _is_string(member_id):
                member_ids.append(member_id)
    return member_ids


def _validate_runtime_node_ids(runtime: dict[str, Any], scene: dict[str, Any], errors: list[str]) -> None:
    node_ids = runtime.get("node_ids")
    if not isinstance(node_ids, list):
        return

    scene_node_ids = _scene_node_ids(scene)
    for index, node_id in enumerate(node_ids):
        if _is_string(node_id) and node_id not in scene_node_ids:
            errors.append(f"runtime.node_ids[{index}]: unknown scene node '{node_id}'")

    runtime_node_ids = {node_id for node_id in node_ids if _is_string(node_id)}
    for member_id in _scene_traffic_member_ids(scene):
        if member_id not in runtime_node_ids:
            errors.append(f"runtime.node_ids: missing traffic-layer member '{member_id}'")


def _has_direct_evidence(value: Any) -> bool:
    if not isinstance(value, list):
        return False
    for item in value:
        if not isinstance(item, dict):
            continue
        if _is_string(item.get("path")) and _is_string(item.get("lines")) and _is_string(item.get("claim")):
            return True
    return False


def _validate_evidence(value: dict[str, Any], path: str, errors: list[str]) -> None:
    if not _has_direct_evidence(value.get("evidence")):
        errors.append(f"{path}.evidence: requires at least one path/lines/claim evidence object")


def _validate_network_semantics(network: dict[str, Any], scene: dict[str, Any], errors: list[str]) -> None:
    containers = network.get("containers")
    memberships = network.get("memberships")
    links = network.get("links")
    if not isinstance(containers, list) or not isinstance(memberships, list) or not isinstance(links, list):
        return

    container_ids: set[str] = set()
    container_kind_by_id: dict[str, str] = {}
    parent_by_id: dict[str, str] = {}
    duplicate_container_ids: set[str] = set()
    for index, container in enumerate(containers):
        if not isinstance(container, dict):
            continue
        container_path = f"network.containers[{index}]"
        _validate_unknown_keys(container, NETWORK_CONTAINER_KEYS, container_path, errors)
        _validate_evidence(container, container_path, errors)
        container_id = container.get("id")
        if _is_string(container_id):
            if container_id in container_ids:
                duplicate_container_ids.add(container_id)
                errors.append(f"{container_path}.id: duplicate container id '{container_id}'")
            container_ids.add(container_id)
            kind = container.get("kind")
            if _is_string(kind):
                container_kind_by_id[container_id] = kind
        if container.get("kind") == "subnet" and "cidr" in container and not _is_string(container.get("cidr")):
            errors.append(
                f"{container_path}.cidr: subnet CIDR must be a string such as '10.42.1.0/24'"
            )
        parent_id = container.get("parent_id")
        if _is_string(parent_id):
            if parent_id not in container_ids and not any(
                isinstance(other, dict) and other.get("id") == parent_id for other in containers
            ):
                errors.append(f"{container_path}.parent_id: unknown container '{parent_id}'")
            if _is_string(container_id):
                parent_by_id[container_id] = parent_id

    reported_cycles: set[str] = set()
    for container_id in [container.get("id") for container in containers if isinstance(container, dict)]:
        if not _is_string(container_id) or container_id in duplicate_container_ids:
            continue
        seen: set[str] = set()
        current = container_id
        while _is_string(current) and current in parent_by_id:
            if current in seen:
                if current not in reported_cycles:
                    errors.append(f"network.containers: containment cycle includes '{current}'")
                    reported_cycles.add(current)
                break
            seen.add(current)
            current = parent_by_id[current]

    scene_node_ids = _scene_node_ids(scene)
    scene_nodes_by_id = _scene_nodes_by_id(scene)
    valid_targets = container_ids | scene_node_ids
    subnet_container_ids = {
        container_id for container_id, kind in container_kind_by_id.items() if kind == "subnet"
    }
    canonical_subnet_id = sorted(subnet_container_ids)[0] if subnet_container_ids else "subnet"
    member_node_ids: set[str] = set()
    for index, membership in enumerate(memberships):
        if not isinstance(membership, dict):
            continue
        _validate_evidence(membership, f"network.memberships[{index}]", errors)
        container_id = membership.get("container_id")
        if _is_string(container_id) and container_id not in container_ids:
            errors.append(f"network.memberships[{index}].container_id: unknown container '{container_id}'")
        node_id = membership.get("node_id")
        if _is_string(node_id):
            if node_id not in scene_node_ids:
                errors.append(f"network.memberships[{index}].node_id: unknown scene node '{node_id}'")
            if node_id in member_node_ids:
                errors.append(f"network.memberships[{index}]: duplicate node membership '{node_id}'")
            member_node_ids.add(node_id)
            node = scene_nodes_by_id.get(node_id, {})
            resource_type = node.get("resource_type")
            resource_type_lower = resource_type.lower() if _is_string(resource_type) else ""
            container_kind = container_kind_by_id.get(container_id) if _is_string(container_id) else None
            if resource_type_lower.endswith("/databases") and container_kind == "subnet":
                errors.append(
                    f"network.memberships[{index}]: SQL PaaS resource '{node_id}' is resource-group scoped and cannot be directly contained by subnet '{container_id}'"
                )
            if "privateendpoints" in resource_type_lower and container_kind != "subnet":
                errors.append(
                    f"network.memberships[{index}]: private endpoint '{node_id}' must be directly contained by subnet container '{canonical_subnet_id}'"
                )

    link_ids: set[str] = set()
    for index, link in enumerate(links):
        if not isinstance(link, dict):
            continue
        _validate_evidence(link, f"network.links[{index}]", errors)
        link_id = link.get("id")
        if _is_string(link_id):
            if link_id in link_ids:
                errors.append(f"network.links[{index}].id: duplicate link id '{link_id}'")
            link_ids.add(link_id)
        kind = link.get("kind")
        if kind not in NETWORK_LINK_KINDS:
            errors.append(
                f"network.links[{index}].kind: unsupported canonical network link kind '{kind}'; expected peering, private-endpoint, dns, or data"
            )
        direction = link.get("direction")
        if direction not in NETWORK_LINK_DIRECTIONS:
            errors.append(
                f"network.links[{index}].direction: unsupported canonical network link direction '{direction}'; expected forward, reverse, or both"
            )
        evidence_level = link.get("evidence_level")
        if "evidence_level" not in link:
            errors.append(
                f"network.links[{index}].evidence_level: required canonical evidence level direct, inferred, or held"
            )
        elif evidence_level not in NETWORK_LINK_EVIDENCE_LEVELS:
            errors.append(
                f"network.links[{index}].evidence_level: unsupported canonical network link evidence_level '{evidence_level}'; expected direct, inferred, or held"
            )
        for key in ("source_id", "target_id"):
            target_id = link.get(key)
            if _is_string(target_id) and target_id not in valid_targets:
                errors.append(f"network.links[{index}].{key}: unknown network target '{target_id}'")


def _supported_icon_ids() -> set[str]:
    try:
        return set(re.findall(r'<symbol\s+id="([^"]+)"', AZURE_ICONS.read_text()))
    except OSError:
        return set()


def _first_cycle_node(stage_ids: list[str], adjacency: dict[str, list[str]]) -> str | None:
    visiting: set[str] = set()
    visited: set[str] = set()

    def visit(node_id: str) -> str | None:
        if node_id in visiting:
            return node_id
        if node_id in visited:
            return None
        visiting.add(node_id)
        for target_id in adjacency.get(node_id, []):
            cycle_node = visit(target_id)
            if cycle_node is not None:
                return cycle_node
        visiting.remove(node_id)
        visited.add(node_id)
        return None

    for stage_id in stage_ids:
        cycle_node = visit(stage_id)
        if cycle_node is not None:
            return cycle_node
    return None


def _validate_pipeline_semantics(pipelines: Any, errors: list[str]) -> None:
    if not isinstance(pipelines, list):
        return

    supported_icons = _supported_icon_ids()
    for pipeline_index, pipeline in enumerate(pipelines):
        if not isinstance(pipeline, dict):
            continue
        stages = pipeline.get("stages")
        edges = pipeline.get("edges")
        if not isinstance(stages, list) or not isinstance(edges, list):
            continue

        stage_ids: list[str] = []
        stage_id_set: set[str] = set()
        for stage_index, stage in enumerate(stages):
            if not isinstance(stage, dict):
                continue
            _validate_evidence(stage, f"pipelines[{pipeline_index}].stages[{stage_index}]", errors)
            stage_id = stage.get("id")
            if _is_string(stage_id):
                stage_ids.append(stage_id)
                stage_id_set.add(stage_id)
            icon = stage.get("icon")
            if _is_string(icon) and icon not in supported_icons:
                errors.append(f"pipelines[{pipeline_index}].stages[{stage_index}].icon: unsupported icon '{icon}'")

        adjacency: dict[str, list[str]] = {}
        for edge_index, edge in enumerate(edges):
            if not isinstance(edge, dict):
                continue
            source_id = edge.get("source_id")
            target_id = edge.get("target_id")
            if _is_string(source_id) and source_id not in stage_id_set:
                errors.append(
                    f"pipelines[{pipeline_index}].edges[{edge_index}].source_id: unknown pipeline stage '{source_id}'"
                )
            if _is_string(target_id) and target_id not in stage_id_set:
                errors.append(
                    f"pipelines[{pipeline_index}].edges[{edge_index}].target_id: unknown pipeline stage '{target_id}'"
                )
            if _is_string(source_id) and _is_string(target_id) and source_id in stage_id_set and target_id in stage_id_set:
                adjacency.setdefault(source_id, []).append(target_id)

        cycle_node = _first_cycle_node(stage_ids, adjacency)
        if cycle_node is not None:
            errors.append(f"pipelines[{pipeline_index}].edges: cycle includes '{cycle_node}'")


def _scene_repository(scene: dict[str, Any]) -> dict[str, Any] | None:
    repository = scene.get("repository")
    if isinstance(repository, dict):
        return repository
    return None


def _validate_repository(views: dict[str, Any], scene: dict[str, Any], errors: list[str]) -> None:
    repository = views.get("repository")
    if not _require_object(repository, "repository", errors):
        return

    scene_repository = _scene_repository(scene)
    for key in ("name", "ref", "commit"):
        value = repository.get(key)
        _require_string(value, f"repository.{key}", errors)
        if scene_repository is not None and value != scene_repository.get(key):
            errors.append(f"repository.{key}: must match scene.repository.{key}")


def validate_views(views: Any, scene: Any) -> list[str]:
    """Return stable field-path validation errors for a version 1 views document."""

    errors: list[str] = []
    if not isinstance(views, dict):
        return ["$: document must be an object"]
    if not isinstance(scene, dict):
        return ["scene: document must be an object"]

    allowed_top_level_keys = {
        "default_view",
        "network",
        "pipelines",
        "repository",
        "runtime",
        "version",
    }
    for key in views:
        if key not in allowed_top_level_keys:
            errors.append(f"$.{key}: unknown key")

    if views.get("version") != 1:
        errors.append("$.version: must equal 1")
    if scene.get("version") != 1:
        errors.append("scene.version: must equal 1")
    _validate_scene_azure_ontology(scene, errors)

    _validate_repository(views, scene, errors)
    _require_string(views.get("default_view"), "default_view", errors)

    runtime = views.get("runtime")
    if _require_object(runtime, "runtime", errors):
        for key in ("node_ids", "path_ids"):
            _validate_string_array(runtime.get(key), f"runtime.{key}", errors)
        if "flow_ids" in runtime:
            _validate_string_array(runtime.get("flow_ids"), "runtime.flow_ids", errors)
        _validate_runtime_node_ids(runtime, scene, errors)

    network = views.get("network")
    if _require_object(network, "network", errors):
        for key in ("containers", "memberships", "links"):
            _validate_object_array(network.get(key), f"network.{key}", errors)
        _validate_network_semantics(network, scene, errors)

    pipelines = views.get("pipelines")
    _validate_object_array(pipelines, "pipelines", errors)
    _validate_pipeline_semantics(pipelines, errors)

    return errors


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("views", type=Path)
    parser.add_argument("scene", type=Path)
    args = parser.parse_args(argv)

    errors = validate_views(
        json.loads(args.views.read_text()),
        json.loads(args.scene.read_text()),
    )
    for error in errors:
        print(error, file=sys.stderr)
    return 1 if errors else 0


if __name__ == "__main__":
    raise SystemExit(main())
