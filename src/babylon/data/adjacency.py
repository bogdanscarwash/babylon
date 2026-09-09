"""Load and verify the committed county adjacency artifact for atlas generation."""

from __future__ import annotations

import hashlib
import json
from functools import lru_cache
from pathlib import Path
from typing import Any

ARTIFACT_PATH = Path(__file__).resolve().parent / "game" / "county_adjacency.json"

_EXPECTED_SCHEMA_VERSION = 1


def _verify_schema_version(data: dict[str, Any]) -> None:
    """Loud failure (Constitution III.11) on a stale artifact shape.

    :param data: the parsed artifact payload.
    :raises ValueError: if ``schema_version`` is not the expected value —
        regenerate via ``tools/derive_county_adjacency.py``.
    """
    actual = data.get("schema_version")
    if actual != _EXPECTED_SCHEMA_VERSION:
        raise ValueError(
            f"{ARTIFACT_PATH.name} schema_version mismatch: expected "
            f"{_EXPECTED_SCHEMA_VERSION}, got {actual!r} -- regenerate via "
            "tools/derive_county_adjacency.py"
        )


def _verify_content_hash(data: dict[str, Any]) -> None:
    """Loud failure (Constitution III.11) on a tampered/stale artifact.

    :param data: the parsed artifact payload.
    :raises ValueError: if the stamped ``content_hash`` doesn't match the
        recomputed SHA-256 over ``pairs`` — the file was hand-edited or only
        partially regenerated.
    """
    canonical = json.dumps(data["pairs"], sort_keys=True, separators=(",", ":"))
    recomputed = hashlib.sha256(canonical.encode("utf-8")).hexdigest()
    if recomputed != data["content_hash"]:
        raise ValueError(
            f"{ARTIFACT_PATH.name} content_hash mismatch: stamped="
            f"{data['content_hash'][:16]}... recomputed={recomputed[:16]}... "
            "-- regenerate via tools/derive_county_adjacency.py"
        )


@lru_cache(maxsize=1)
def load_adjacency_pairs() -> tuple[tuple[str, str], ...]:
    """Load and cache every county-adjacency pair.

    :returns: sorted ``(fips_a, fips_b)`` tuples with ``fips_a < fips_b`` —
        each unordered adjacency appears exactly once.
    :raises FileNotFoundError: if the artifact is missing from the wheel/repo.
    :raises ValueError: propagated from the schema/hash verifiers.
    """
    data = json.loads(ARTIFACT_PATH.read_text())
    _verify_schema_version(data)
    _verify_content_hash(data)
    return tuple((pair[0], pair[1]) for pair in data["pairs"])


def adjacency_pairs_for_scope(scope_fips: frozenset[str]) -> list[tuple[str, str]]:
    """The adjacency pairs internal to one scope of counties.

    Pure and deterministic: output order is the artifact's sorted order,
    filtered — never set-iteration order.

    :param scope_fips: 5-digit county FIPS codes present in the world.
    :returns: sorted ``(fips_a, fips_b)`` pairs where BOTH endpoints are in
        scope. A single-county scope yields ``[]`` (nothing to be adjacent
        to), which is why the ``single_county`` regression scenario is
        untouched by this producer.
    """
    return [
        pair for pair in load_adjacency_pairs() if pair[0] in scope_fips and pair[1] in scope_fips
    ]
