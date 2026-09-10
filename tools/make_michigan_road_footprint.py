#!/usr/bin/env python3
"""Pin Geofabrik's clipping extent for Michigan and transport-only connectors.

These simplified provider polygons bound extraction; they are not legal borders.
Original polygon bytes stay with the source snapshot. The output records their
URLs and hashes, and a downstream extractor records its explicit angular buffer.
"""

from __future__ import annotations

import argparse
import hashlib
import json
from pathlib import Path

import shapely
from shapely.geometry import MultiPolygon, Polygon, mapping
from shapely.ops import unary_union

REGIONS = (
    ("illinois", "us"),
    ("indiana", "us"),
    ("michigan", "us"),
    ("ohio", "us"),
    ("ontario", "canada"),
    ("wisconsin", "us"),
)


def read_polygon(path: Path) -> Polygon | MultiPolygon:
    """Decode the provider poly format, refusing unpaired holes or open rings."""
    lines = iter(path.read_text(encoding="utf-8").splitlines())
    next(lines)  # Provider display name has no geometric meaning.
    shells: list[Polygon] = []
    for label in lines:
        if label == "END":
            if any(line.strip() for line in lines):
                raise ValueError(f"trailing polygon data: {path}")
            break
        points = []
        for line in lines:
            if line == "END":
                break
            lon, lat = (float(number) for number in line.split())
            if not (-180 <= lon <= 180 and -90 <= lat <= 90):
                raise ValueError(f"invalid coordinate: {path}")
            points.append((lon, lat))
        if len(points) < 4 or points[0] != points[-1]:
            raise ValueError(f"unclosed polygon: {path}")
        ring = Polygon(points)
        if ring.is_empty or not ring.is_valid:
            raise ValueError(f"invalid polygon: {path}")
        if label.startswith("!"):
            if not shells or not shells[-1].contains(ring):
                raise ValueError(f"unpaired polygon hole: {path}")
            shells[-1] = Polygon(shells[-1].exterior.coords, [*shells[-1].interiors, points])
        else:
            shells.append(ring)
    else:
        raise ValueError(f"missing polygon terminator: {path}")
    geometry = unary_union(shells)
    if (
        not isinstance(geometry, Polygon | MultiPolygon)
        or geometry.is_empty
        or not geometry.is_valid
    ):
        raise ValueError(f"invalid clipping extent: {path}")
    return geometry


def main() -> None:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--source-dir", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    sources = []
    geometries = []
    for region, parent in REGIONS:
        path = args.source_dir / f"{region}.poly"
        content = path.read_bytes()
        sources.append(
            {
                "region": region,
                "url": f"https://download.geofabrik.de/north-america/{parent}/{region}.poly",
                "sha256": hashlib.sha256(content).hexdigest(),
                "byte_count": len(content),
            }
        )
        geometries.append(read_polygon(path))
    geometry = unary_union(geometries).normalize()
    document = {
        "type": "Feature",
        "properties": {
            "schema": "MichiganRoadExtractionFootprintV1",
            "purpose": "clipping extent; outside Michigan is transport-only",
            "source_attribution": "Geofabrik; OpenStreetMap contributors",
            "sources": sources,
            "geometry_operation": "union of original provider polygons; no buffer",
            "shapely_version": shapely.__version__,
            "geos_version": shapely.geos_version_string,
        },
        "geometry": mapping(geometry),
    }
    content = (
        json.dumps(document, sort_keys=True, separators=(",", ":"), allow_nan=False) + "\n"
    ).encode()
    args.output.parent.mkdir(parents=True, exist_ok=True)
    args.output.write_bytes(content)
    print(json.dumps({"bytes": len(content), "sha256": hashlib.sha256(content).hexdigest()}))


if __name__ == "__main__":
    main()
