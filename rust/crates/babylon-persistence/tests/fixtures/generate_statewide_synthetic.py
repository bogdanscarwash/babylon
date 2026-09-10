"""Regenerate synthetic scale evidence; this is not a real transport qualification.

Run with the project Python from the repository root. The only graph is a
bidirectional chain through the 83 sorted roster counties. The canonical
qualifier chooses relationships; this script adds no allocation semantics.

Regenerate after every canonical defines byte change, including comments or the
real experiment table: both qualification and physical pins bind its raw hash.
The qualifier ignores intervention metadata but re-evaluates recipe/order inputs.
From the repository root:
    mise exec -- uv run --frozen python \
      rust/crates/babylon-persistence/tests/fixtures/generate_statewide_synthetic.py \
      --repo-root .
The default output is only this directory's statewide_synthetic.json.gz fixture.
No real statewide source or experiment artifact is written.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import importlib
import json
import sys
from pathlib import Path


def canonical(value: object) -> bytes:
    return (json.dumps(value, sort_keys=True, separators=(",", ":")) + "\n").encode()


def digest(data: bytes) -> str:
    return hashlib.sha256(data).hexdigest()


def generate(root: Path, output: Path) -> None:
    sys.path.insert(0, str(root / "tools"))
    circuit = importlib.import_module("qualify_michigan_commodity_circuit")
    roster_path = (
        root / "src/babylon/data/reference/economy/michigan_commodity_roster_mi_2024.json.gz"
    )
    roster_bytes = roster_path.read_bytes()
    roster = json.loads(gzip.decompress(roster_bytes))
    counties = sorted(row["county_geoid"] for row in roster["sources"])
    if len(counties) != 83 or len(roster["actors"]) != 397:
        raise ValueError("the pinned roster must contain 83 counties and 397 owners")
    defines_path = root / "content/scenarios/michigan/defines.toml"
    config = circuit.read_defines(defines_path)
    positions = {county: i for i, county in enumerate(counties)}
    paths = {}
    for source in counties:
        for destination in counties:
            start, end = positions[source], positions[destination]
            step = 1 if end > start else -1
            edges = tuple(
                f"synthetic-{counties[n]}-{counties[n + step]}" for n in range(start, end, step)
            )
            paths[(source, destination)] = circuit.CountyPath(abs(end - start) * 1_000_000, edges)
    qualified = circuit.qualify(circuit.owners_from_roster(roster), config, paths)
    if not qualified.qualified:
        raise ValueError(f"synthetic paths did not qualify: {qualified.diagnostics}")
    paths_bytes = canonical(
        [
            [source, destination, path.distance_mm, path.edge_ids]
            for (source, destination), path in sorted(paths.items())
        ]
    )
    document = {
        **circuit.qualification_document(qualified),
        "roster_sha256": digest(roster_bytes),
        "paths_sha256": digest(paths_bytes),
    }
    selected = sorted({edge for order in qualified.orders for edge in order.edge_ids})
    coordinate = {
        county: [-850_000_000 + index * 10_000, 430_000_000]
        for index, county in enumerate(counties)
    }
    geometry = []
    for edge in selected:
        _, source, destination = edge.split("-")
        geometry.append(
            {
                "id": edge,
                "way_id": int(source) * 100_000 + int(destination),
                "from_node": int(source),
                "to_node": int(destination),
                "distance_mm": 1_000_000,
                "shape_e7": [coordinate[source], coordinate[destination]],
                "tags": {"fixture": "synthetic-scale-only"},
                "way_version": 1,
                "way_timestamp": "synthetic",
            }
        )
    graph_bytes = canonical(geometry)
    graph_hash = digest(graph_bytes)
    synthetic_hash = digest(b"synthetic-scale-only; no OSM, real atlas or road source")
    physical = {
        "source": {
            "pbf_sha256": synthetic_hash,
            "pbf_bytes": len(graph_bytes),
            "pbf_url": "synthetic://statewide-scale-fixture/not-an-osm-source",
            "replication_timestamp": "synthetic",
            "footprint_sha256": synthetic_hash,
            "buffer_degrees_e7": 0,
            "extraction_version": "synthetic-chain-v1",
            "distance_version": "synthetic-integer-cost-v1",
            "routing_profile_version": "michigan-freight-routing-v1",
            "graph_sha256": graph_hash,
        },
        "profile": {
            "gross_weight_kg": 40000,
            "height_mm": 4000,
            "width_mm": 2550,
            "length_mm": 16500,
            "default_maxheight_mm": 4000,
            "axle_load_kg": None,
            "evidence_class": "Designed",
        },
        "terminal_source_pins": {
            "atlas_sha256": synthetic_hash,
            "atlas_pin_sha256": synthetic_hash,
            "defines_sha256": config.source_sha256,
            "graph_sha256": graph_hash,
        },
        "terminal_policy": {
            "terminal_evidence_class": "Designed",
            "anchor": "synthetic-chain-coordinate",
            "attachment_limit_mm": 50_000_000,
            "attachment_distance": "zero-synthetic-attachment",
            "usable_node": "roster-county-identity",
            "physical_path_distance": "integer-chain-edge-count",
            "projection": "synthetic-scale-only",
        },
        "terminal_attachment_limit_meters": 50000,
        "terminals": [
            {
                "county_geoid": county,
                "node_id": int(county),
                "county_name": f"Synthetic terminal for {county}",
                "anchor_lon_e7": coordinate[county][0],
                "anchor_lat_e7": coordinate[county][1],
                "atlas_grid_x": index,
                "atlas_grid_y": 0,
                "node_lon_e7": coordinate[county][0],
                "node_lat_e7": coordinate[county][1],
                "attachment_distance_mm": 0,
                "status": "synthetic",
                "evidence_class": "Designed",
            }
            for index, county in enumerate(counties)
        ],
        "edges": geometry,
        "capacity_groups": [
            {
                "key": "synthetic-shared-road",
                "label": "Synthetic shared road capacity; not real infrastructure",
                "edge_keys": selected,
            }
        ],
    }
    fixture = {
        "scope": "synthetic-scale-only; not real infrastructure qualification",
        "qualification": document,
        "physical": physical,
    }
    encoded = canonical(fixture)
    output.write_bytes(gzip.compress(encoded, mtime=0))
    print(
        json.dumps(
            {
                "owners": len(qualified.owners),
                "processes": len(qualified.processes),
                "orders": len(qualified.orders),
                "final_demands": len(qualified.retail_final_demands),
                "selected_synthetic_edges": len(selected),
                "json_bytes": len(encoded),
                "gzip_bytes": output.stat().st_size,
                "sha256": digest(output.read_bytes()),
            },
            sort_keys=True,
        )
    )


if __name__ == "__main__":
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument(
        "--out", type=Path, default=Path(__file__).with_name("statewide_synthetic.json.gz")
    )
    arguments = parser.parse_args()
    generate(arguments.repo_root.resolve(), arguments.out)
