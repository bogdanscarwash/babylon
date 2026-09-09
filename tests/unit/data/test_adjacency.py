"""Pins for the county-adjacency artifact and its loader (ADR179 T1).

The artifact is a committed, content-hashed derivation from pinned TIGER
geometry — these tests pin its integrity contract, its invariants, and a
small golden neighborhood (Wayne County MI) whose ground truth is checkable
against any map. They never touch the reference DB or the data drive.
"""

from __future__ import annotations

import json

import pytest

from babylon.data import adjacency
from babylon.data.adjacency import (
    ARTIFACT_PATH,
    adjacency_pairs_for_scope,
    load_adjacency_pairs,
)


@pytest.mark.unit
class TestArtifactIntegrity:
    def test_artifact_loads_and_hash_verifies(self) -> None:
        pairs = load_adjacency_pairs()
        assert len(pairs) > 0

    def test_pair_count_is_pinned(self) -> None:
        # 9,477 unordered pairs over the 3,222 counties with TIGER geometry.
        # Same magnitude as the Census county-adjacency file (~9.5k unordered
        # pairs over 3,234 counties). A regenerated artifact that moves this
        # number means the geometry source changed — that is a declared data
        # change, not noise.
        assert len(load_adjacency_pairs()) == 9477

    def test_tampered_artifact_fails_loud(self, tmp_path, monkeypatch) -> None:
        data = json.loads(ARTIFACT_PATH.read_text())
        data["pairs"][0] = ["00000", "99999"]  # hand-edit without restamping
        tampered = tmp_path / "county_adjacency.json"
        tampered.write_text(json.dumps(data))
        monkeypatch.setattr(adjacency, "ARTIFACT_PATH", tampered)
        load_adjacency_pairs.cache_clear()
        try:
            with pytest.raises(ValueError, match="content_hash mismatch"):
                load_adjacency_pairs()
        finally:
            load_adjacency_pairs.cache_clear()

    def test_wrong_schema_version_fails_loud(self, tmp_path, monkeypatch) -> None:
        data = json.loads(ARTIFACT_PATH.read_text())
        data["schema_version"] = 999
        stale = tmp_path / "county_adjacency.json"
        stale.write_text(json.dumps(data))
        monkeypatch.setattr(adjacency, "ARTIFACT_PATH", stale)
        load_adjacency_pairs.cache_clear()
        try:
            with pytest.raises(ValueError, match="schema_version mismatch"):
                load_adjacency_pairs()
        finally:
            load_adjacency_pairs.cache_clear()


@pytest.mark.unit
class TestPairInvariants:
    def test_every_pair_is_ordered_low_to_high(self) -> None:
        assert all(a < b for a, b in load_adjacency_pairs())

    def test_no_self_adjacency(self) -> None:
        assert all(a != b for a, b in load_adjacency_pairs())

    def test_pairs_are_unique_and_sorted(self) -> None:
        pairs = list(load_adjacency_pairs())
        assert pairs == sorted(set(pairs))

    def test_every_fips_is_five_digits(self) -> None:
        for a, b in load_adjacency_pairs():
            assert len(a) == 5 and a.isdigit(), a
            assert len(b) == 5 and b.isdigit(), b


@pytest.mark.unit
class TestWayneCountyGolden:
    """Wayne County MI (26163): four land neighbors, checkable on any map."""

    def test_wayne_neighbors(self) -> None:
        neighbors = sorted(
            b if a == "26163" else a for a, b in load_adjacency_pairs() if "26163" in (a, b)
        )
        # Macomb, Monroe, Oakland, Washtenaw. The Detroit River boundary with
        # Essex County, Ontario is not a US county and correctly absent.
        assert neighbors == ["26099", "26115", "26125", "26161"]


@pytest.mark.unit
class TestScopeFilter:
    def test_single_county_scope_yields_nothing(self) -> None:
        # Why the single_county regression scenario is untouched by this
        # producer: one county has nothing in-scope to be adjacent to.
        assert adjacency_pairs_for_scope(frozenset({"26163"})) == []

    def test_tri_county_scope_yields_only_internal_pairs(self) -> None:
        # Wayne + Macomb + Monroe: Wayne touches both, Macomb and Monroe do
        # not touch each other (Wayne lies between them).
        scope = frozenset({"26163", "26099", "26115"})
        assert adjacency_pairs_for_scope(scope) == [
            ("26099", "26163"),
            ("26115", "26163"),
        ]

    def test_empty_scope_yields_nothing(self) -> None:
        assert adjacency_pairs_for_scope(frozenset()) == []

    def test_output_is_deterministically_ordered(self) -> None:
        # Order comes from the artifact's sorted order, never from set
        # iteration — two differently-constructed but equal scopes agree.
        scope_a = frozenset({"26163", "26099", "26115", "26125", "26161"})
        scope_b = frozenset(sorted(scope_a, reverse=True))
        assert adjacency_pairs_for_scope(scope_a) == adjacency_pairs_for_scope(scope_b)
