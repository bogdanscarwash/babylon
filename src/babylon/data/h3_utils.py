"""Convert reference geography into H3 cells during TIGER geometry ingestion."""

from __future__ import annotations

import logging
from typing import TYPE_CHECKING

import h3

if TYPE_CHECKING:
    from shapely.geometry import Polygon as ShapelyPolygon  # type: ignore[import-untyped]
    from shapely.geometry.base import BaseGeometry  # type: ignore[import-untyped]

logger = logging.getLogger(__name__)


def wkt_to_geometry(wkt: str | None) -> BaseGeometry | None:
    """Parse WKT string to Shapely geometry (Polygon or MultiPolygon).

    Args:
        wkt: Well-Known Text geometry string.

    Returns:
        Shapely geometry or None if invalid/empty.
    """
    if not wkt or not wkt.strip():
        return None

    from shapely import wkt as shapely_wkt  # type: ignore[import-untyped]

    try:
        geom = shapely_wkt.loads(wkt)
        return geom if geom.is_valid else None
    except Exception:
        return None


def _generate_h3_cells_single(polygon: ShapelyPolygon, resolution: int) -> set[str]:
    """Generate H3 cells for a single polygon."""
    coords = list(polygon.exterior.coords)
    geojson_coords = [[x, y] for x, y in coords]
    geojson = {"type": "Polygon", "coordinates": [geojson_coords]}

    try:
        cells = h3.geo_to_cells(geojson, resolution)
        return set(cells)
    except Exception as exc:
        logger.debug("Failed to generate H3 cells: %s", exc)
        centroid = polygon.centroid
        cell = h3.latlng_to_cell(centroid.y, centroid.x, resolution)
        return {cell}


def generate_h3_cells(geometry: BaseGeometry, resolution: int) -> set[str]:
    """Generate H3 cells that cover a geometry (Polygon or MultiPolygon).

    Args:
        geometry: Shapely Polygon or MultiPolygon geometry.
        resolution: H3 resolution level (0-15).

    Returns:
        Set of H3 cell index strings.
    """
    if geometry.geom_type == "MultiPolygon":
        cells: set[str] = set()
        for poly in geometry.geoms:
            cells.update(_generate_h3_cells_single(poly, resolution))
        return cells
    elif geometry.geom_type == "Polygon":
        return _generate_h3_cells_single(geometry, resolution)
    else:
        logger.warning("Unsupported geometry type: %s", geometry.geom_type)
        centroid = geometry.centroid
        cell = h3.latlng_to_cell(centroid.y, centroid.x, resolution)
        return {cell}
