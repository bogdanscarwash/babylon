"""Detroit tri-county LODES study area and committed reference artifact paths."""

from __future__ import annotations

from functools import lru_cache
from pathlib import Path

import h3

# Approximate bounding polygon for the Wayne+Oakland+Macomb tri-county region.
# IDENTICAL to tests/constants_063.py's _TRI_COUNTY_BOUNDS_LATLNG (kept
# independent per the default_tri_county_aggregate_hex() precedent in
# border_commute_synthesis.py — production and tests resolve the same hex
# set without one importing the other).
_TRI_COUNTY_BOUNDS_LATLNG: tuple[tuple[float, float], ...] = (
    (42.045, -83.290),  # SW corner (Wayne County southwest)
    (42.045, -82.870),  # SE corner (Wayne County southeast — at Detroit River)
    (42.890, -82.795),  # NE corner (Macomb County northeast)
    (42.890, -83.690),  # NW corner (Oakland County northwest)
    (42.045, -83.290),  # close
)

#: Michigan is the only state the checked-in LODES artifact covers.
LODES_STUDY_AREA_STATES: frozenset[str] = frozenset({"26"})

#: Repo root: this file is ``src/babylon/domain/economics/lodes_study_area.py``.
_BABYLON_PKG_ROOT: Path = Path(__file__).resolve().parents[1]

#: Checked-in artifact root (Tier-1 in-repo, per ADR076's size-tiering
#: convention — the pruned tri-county files are small enough to commit
#: directly rather than route through the ci-data release channel).
LODES_ARTIFACT_ROOT: Path = _BABYLON_PKG_ROOT / "data" / "reference" / "lodes"
LODES_ARTIFACT_OD_ROOT: Path = LODES_ARTIFACT_ROOT
LODES_ARTIFACT_CROSSWALK: Path = LODES_ARTIFACT_ROOT / "tri_county_hex_xwalk.csv.gz"


@lru_cache(maxsize=1)
def lodes_tri_county_hexes_res7() -> frozenset[str]:
    """Return the H3 res-7 cell set covering the Detroit tri-county study area.

    Lazy + cached (mirrors ``tests/constants_063.py``'s
    ``_compute_tri_county_hexes_res7``) to avoid paying the
    ``polygon_to_cells`` cost at import time.
    """
    polygon = h3.LatLngPoly(_TRI_COUNTY_BOUNDS_LATLNG)
    return frozenset(h3.polygon_to_cells(polygon, 7))


__all__ = [
    "LODES_ARTIFACT_CROSSWALK",
    "LODES_ARTIFACT_OD_ROOT",
    "LODES_ARTIFACT_ROOT",
    "LODES_STUDY_AREA_STATES",
    "lodes_tri_county_hexes_res7",
]
