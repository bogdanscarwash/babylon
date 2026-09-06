"""Derive the county-adjacency artifact from pinned TIGER geometry (ADR179 T1).

Produces ``src/babylon/data/game/county_adjacency.json``: every unordered pair
of US counties whose TIGER/Line 2024 boundaries touch, derived from the
``geometry_wkt`` column the reference DB already carries (itself loaded from
the Phase 0-D pinned shapefile by ``tools/ingest_tiger_geometry.py``).

Follows ``tools/generate_us_county_territories.py``'s artifact discipline
exactly: schema-versioned, provenance-stamped, content-hashed JSON committed
in-repo, so the engine never touches the data drive at runtime (the ADR121
precedent; CI/tests read the artifact, never the drive).

Determinism: shapely predicates over fixed WKT text are deterministic, pairs
are emitted sorted, and the probe run confirmed two passes produce identical
sets. ``intersects`` and ``touches`` were verified EQUIVALENT on this dataset
(zero pairs with overlapping interiors — TIGER county polygons are
topologically clean), so the faster ``intersects`` is used with a loud check
that the equivalence still holds per pair.

Usage::

    uv run python tools/derive_county_adjacency.py

Then commit the artifact; ``babylon.data.adjacency`` verifies the
stamp at load and fails loud on drift.
"""

from __future__ import annotations

import hashlib
import json
import sqlite3
import sys
from pathlib import Path

from shapely import wkt as shapely_wkt
from shapely.strtree import STRtree

_REPO_ROOT = Path(__file__).resolve().parent.parent
REFERENCE_DB = _REPO_ROOT / "data" / "sqlite" / "marxist-data-3NF.sqlite"
ARTIFACT_PATH = _REPO_ROOT / "src" / "babylon" / "data" / "game" / "county_adjacency.json"

#: Bump on any change to the pair-derivation rule or artifact shape.
SCHEMA_VERSION = 1

_GEOMETRY_SQL = """
    SELECT c.fips, g.geometry_wkt
    FROM dim_county_geometry g
    JOIN dim_county c ON c.county_id = g.county_id
    WHERE g.geometry_wkt IS NOT NULL
    ORDER BY c.fips
"""


def derive_pairs(rows: list[tuple[str, str]]) -> list[list[str]]:
    """Compute sorted unordered adjacency pairs from (fips, wkt) rows.

    Args:
        rows: ``(fips, geometry_wkt)`` tuples, fips unique, sorted.

    Returns:
        Sorted list of ``[fips_a, fips_b]`` pairs with ``fips_a < fips_b``.

    Raises:
        ValueError: If FIPS codes are not unique (a non-unique key silently
            collapses distinct pairs under set-dedup — the exact bug the
            2026-07-30 probe hit by keying on the 3-digit ``county_fips``),
            or if any candidate pair has overlapping interiors (TIGER county
            polygons must be topologically clean; an overlap means the
            geometry source drifted and 'intersects' no longer means
            'adjacent').
    """
    fips = [r[0] for r in rows]
    if len(set(fips)) != len(fips):
        raise ValueError("FIPS keys are not unique; refusing to derive adjacency")
    geoms = [shapely_wkt.loads(r[1]) for r in rows]
    tree = STRtree(geoms)
    pairs: list[list[str]] = []
    for i, geom in enumerate(geoms):
        for j_raw in tree.query(geom, predicate="intersects"):
            j = int(j_raw)
            if i >= j:
                continue
            if geom.overlaps(geoms[j]):
                raise ValueError(
                    f"counties {fips[i]} and {fips[j]} have overlapping "
                    "interiors; TIGER geometry is expected topologically "
                    "clean -- investigate the geometry source before trusting "
                    "any pair in this run"
                )
            pairs.append([fips[i], fips[j]])
    pairs.sort()
    return pairs


def main() -> int:
    """Derive and write the artifact. Returns a process exit code."""
    if not REFERENCE_DB.exists():
        print(f"ERROR: reference DB not found at {REFERENCE_DB}", file=sys.stderr)
        return 1
    conn = sqlite3.connect(REFERENCE_DB)
    try:
        rows = conn.execute(_GEOMETRY_SQL).fetchall()
    finally:
        conn.close()
    print(f"counties with geometry: {len(rows)}")
    pairs = derive_pairs(rows)
    print(f"adjacency pairs: {len(pairs)}")

    canonical = json.dumps(pairs, sort_keys=True, separators=(",", ":"))
    content_hash = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
    artifact = {
        "schema_version": SCHEMA_VERSION,
        "source": {
            "dataset": (
                "dim_county + dim_county_geometry (TIGER/Line 2024 US county "
                "boundaries, Census Bureau; loaded by tools/ingest_tiger_geometry.py)"
            ),
            "reference_db": "data/sqlite/marxist-data-3NF.sqlite",
            "rule": (
                "unordered pairs whose polygons intersect; interiors verified "
                "non-overlapping, so intersects == touches on this dataset"
            ),
            "generator": "tools/derive_county_adjacency.py",
        },
        "content_hash": content_hash,
        "pairs": pairs,
    }
    ARTIFACT_PATH.write_text(json.dumps(artifact, indent=1) + "\n")
    print(f"wrote {ARTIFACT_PATH} (content_hash {content_hash[:16]}...)")
    return 0


if __name__ == "__main__":
    sys.exit(main())
