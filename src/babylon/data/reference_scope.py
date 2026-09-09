"""Read-only county scope definitions for Python data tooling.

The playable runtime no longer resolves scopes in Python. This module owns
the remaining data-periphery need: reproducible county universes for artifact
builders and frozen reference tests.
"""

from __future__ import annotations

import json
import sqlite3
from pathlib import Path
from typing import NamedTuple

DEFAULT_SQLITE_PATH = Path("data/sqlite/marxist-data-3NF.sqlite")
_COUNTY_ARTIFACT = Path(__file__).with_name("game") / "us_county_territories.json"


class Scope(NamedTuple):
    """Concrete reference-data scope."""

    scope_fips: frozenset[str]
    external_node_ids: frozenset[str]


def _artifact_counties() -> frozenset[str]:
    payload = json.loads(_COUNTY_ARTIFACT.read_text(encoding="utf-8"))
    counties = payload.get("counties")
    if not isinstance(counties, list):
        raise ValueError(f"{_COUNTY_ARTIFACT}: counties must be a list")
    fips: list[str] = []
    for row in counties:
        if not isinstance(row, dict):
            raise ValueError(f"{_COUNTY_ARTIFACT}: county rows must be objects")
        value = row.get("fips")
        if not isinstance(value, str) or len(value) != 5:
            raise ValueError(f"{_COUNTY_ARTIFACT}: invalid county FIPS")
        fips.append(value)
    if fips != sorted(fips) or len(fips) != len(set(fips)):
        raise ValueError(f"{_COUNTY_ARTIFACT}: county FIPS must be sorted and unique")
    return frozenset(fips)


MICHIGAN_FIPS = frozenset(value for value in _artifact_counties() if value.startswith("26"))
DETROIT_TRI_COUNTY_FIPS = frozenset({"26099", "26125", "26163"})


class UnknownScopeError(ValueError):
    """Raised when an unrecognized reference-data scope is requested."""


def resolve_scope(name: str, *, sqlite_path: Path = DEFAULT_SQLITE_PATH) -> Scope:
    """Resolve one named data scope without granting runtime authority."""
    if name == "michigan-canada":
        return Scope(MICHIGAN_FIPS, frozenset({"canada"}))
    if name == "michigan-statewide-no-canada":
        return Scope(MICHIGAN_FIPS, frozenset())
    if name == "detroit-tri-county":
        return Scope(DETROIT_TRI_COUNTY_FIPS, frozenset({"canada"}))
    if name == "national":
        return Scope(_load_national_fips(sqlite_path), frozenset({"canada", "china"}))
    raise UnknownScopeError(
        f"Unknown scope {name!r}; expected one of: michigan-canada, "
        "michigan-statewide-no-canada, detroit-tri-county, national"
    )


def _load_national_fips(sqlite_path: Path) -> frozenset[str]:
    """Read the bounded US county universe from deterministic SQLite."""
    if not sqlite_path.exists():
        raise FileNotFoundError(
            f"SQLite reference DB not found at {sqlite_path} (needed for national scope)."
        )
    with sqlite3.connect(sqlite_path) as connection:
        rows = connection.execute(
            "SELECT fips FROM dim_county "
            "WHERE substr(fips, 1, 2) < '60' "
            "AND substr(fips, 3, 3) != '999' "
            "ORDER BY fips"
        ).fetchall()
    return frozenset(str(row[0]) for row in rows)


__all__ = [
    "DEFAULT_SQLITE_PATH",
    "DETROIT_TRI_COUNTY_FIPS",
    "MICHIGAN_FIPS",
    "Scope",
    "UnknownScopeError",
    "resolve_scope",
]
