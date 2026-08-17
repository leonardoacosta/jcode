#!/usr/bin/env python3
"""Pure geometry helpers for design-language-agnostic isometric scenes."""

from __future__ import annotations

from collections.abc import Iterable, Mapping, Sequence
from math import isclose
from typing import Any


def assert_two_to_one(tile_width: float, tile_height: float) -> None:
    """Reject projection dimensions that would distort the isometric terrain."""

    if tile_width <= 0 or tile_height <= 0 or not isclose(
        float(tile_width), float(tile_height) * 2, rel_tol=0, abs_tol=1e-9
    ):
        raise ValueError("tile_width must equal 2 × tile_height")


def project(
    x: float,
    y: float,
    z_px: float,
    tile_width: float,
    tile_height: float,
    origin_x: float,
    origin_y: float,
) -> tuple[float, float]:
    """Project a grid point onto a 2:1 isometric plane.

    ``z_px`` is vertical screen-space elevation. Scene height units should be converted to pixels by
    the renderer so art direction can control how tall the same semantic building feels.
    """

    assert_two_to_one(tile_width, tile_height)
    return (
        float(origin_x + (x - y) * tile_width / 2),
        float(origin_y + (x + y) * tile_height / 2 - z_px),
    )


def footprints_overlap(a: Mapping[str, float], b: Mapping[str, float]) -> bool:
    """Return whether two axis-aligned grid footprints overlap with positive area."""

    return (
        a["x"] < b["x"] + b["width"]
        and a["x"] + a["width"] > b["x"]
        and a["y"] < b["y"] + b["depth"]
        and a["y"] + a["depth"] > b["y"]
    )


def cuboid_faces(
    x: float,
    y: float,
    width: float,
    depth: float,
    height_px: float,
    tile_width: float,
    tile_height: float,
    origin_x: float,
    origin_y: float,
) -> dict[str, list[tuple[float, float]]]:
    """Return roof and visible wall polygons for a rectangular isometric mass."""

    floor = [
        project(x, y, 0, tile_width, tile_height, origin_x, origin_y),
        project(x + width, y, 0, tile_width, tile_height, origin_x, origin_y),
        project(x + width, y + depth, 0, tile_width, tile_height, origin_x, origin_y),
        project(x, y + depth, 0, tile_width, tile_height, origin_x, origin_y),
    ]
    roof = [
        project(x, y, height_px, tile_width, tile_height, origin_x, origin_y),
        project(x + width, y, height_px, tile_width, tile_height, origin_x, origin_y),
        project(x + width, y + depth, height_px, tile_width, tile_height, origin_x, origin_y),
        project(x, y + depth, height_px, tile_width, tile_height, origin_x, origin_y),
    ]
    return {
        "roof": roof,
        "left": [roof[3], roof[2], floor[2], floor[3]],
        "right": [roof[1], roof[2], floor[2], floor[1]],
    }


def route_points(
    route: Iterable[Mapping[str, float]],
    tile_width: float,
    tile_height: float,
    origin_x: float,
    origin_y: float,
    z_px: float = 0,
) -> list[tuple[float, float]]:
    """Project a ground-grid route into screen points."""

    return [
        project(point["x"], point["y"], z_px, tile_width, tile_height, origin_x, origin_y)
        for point in route
    ]


def svg_polyline(points: Sequence[tuple[float, float]]) -> str:
    """Serialize projected route points for an SVG polyline or path."""

    return " ".join(f"{x:.2f},{y:.2f}" for x, y in points)


def depth_key(node: Mapping[str, Any]) -> tuple[float, float, float]:
    """Sort nodes back-to-front using the far edge of each footprint."""

    position = node["position"]
    footprint = node["footprint"]
    far_x = float(position["x"] + footprint["width"])
    far_y = float(position["y"] + footprint["depth"])
    return (far_x + far_y, far_y, far_x)
