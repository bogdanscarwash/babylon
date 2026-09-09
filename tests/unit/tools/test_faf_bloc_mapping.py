"""Unit tests for the FAF zone -> engine node mapping and aggregation
(Program 26, Unit U3) — ``tools/make_faf_bloc_tons_artifact.py``.

Pure-logic tests: no drive access, no real FAF CSV. The mapping-completeness
test is the disclosure contract: every canonical international node id must
be accounted for, either as a mapped node or a disclosed absence
— silent gaps are the bug class this test exists to catch.
"""

from __future__ import annotations

import csv
import gzip
import sys
from pathlib import Path

import pytest

_TOOLS_DIR = Path(__file__).resolve().parents[3] / "tools"
sys.path.insert(0, str(_TOOLS_DIR))

import make_faf_bloc_tons_artifact as faf_tool  # type: ignore[import-not-found]  # noqa: E402

pytestmark = [pytest.mark.unit]

_EXPECTED_INTERNATIONAL_NODES = {
    "canada",
    "china",
    "eu",
    "india",
    "sub_saharan_africa",
    "latin_america",
    "russia_csi",
    "southeast_asia",
}


def test_every_international_node_accounted_for() -> None:
    """Every canonical node is either FAF-mapped or a disclosed absence —
    no silent gaps."""
    mapped = set(faf_tool.FAF_ZONE_TO_NODE.values())
    uncovered = set(faf_tool.FAF_UNCOVERED_NODES)
    assert mapped & uncovered == set(), "a node cannot be both mapped and disclosed-uncovered"
    accounted = mapped | uncovered
    missing = _EXPECTED_INTERNATIONAL_NODES - accounted
    assert not missing, (
        f"international nodes with no FAF disposition (mapped or disclosed): {missing}"
    )
    extra = accounted - _EXPECTED_INTERNATIONAL_NODES
    assert not extra, f"FAF mapping references non-canonical node ids: {extra}"


def test_zone_806_deliberately_excluded() -> None:
    """Zone 806 ("SW & Central Asia") mixes India with the Middle East and
    Central Asia/Caucasus — no honest single-node assignment exists, so it
    must be absent from the mapping table entirely (not silently folded into
    some node)."""
    assert 806 not in faf_tool.FAF_ZONE_TO_NODE


def test_mapping_is_injective_per_zone() -> None:
    """Each FAF zone maps to at most one node (a node MAY receive multiple
    zones, e.g. latin_america <- Mexico + Rest of Americas, but a single
    zone's tons are never double-counted across nodes)."""
    zones = list(faf_tool.FAF_ZONE_TO_NODE)
    assert len(zones) == len(set(zones))


def test_latin_america_receives_mexico_and_rest_of_americas() -> None:
    assert faf_tool.FAF_ZONE_TO_NODE[802] == "latin_america"
    assert faf_tool.FAF_ZONE_TO_NODE[803] == "latin_america"


def _write_fake_faf_csv(path: Path) -> None:
    fieldnames = [
        "fr_orig",
        "dms_origst",
        "dms_destst",
        "fr_dest",
        "trade_type",
        "tons_2018",
        "tons_2019",
    ]
    rows = [
        # Canada import leg, 2018 + 2019.
        {
            "fr_orig": "801",
            "dms_origst": "",
            "dms_destst": "26",
            "fr_dest": "",
            "trade_type": "2",
            "tons_2018": "10.0",
            "tons_2019": "12.0",
        },
        # Canada export leg, 2018 only.
        {
            "fr_orig": "",
            "dms_origst": "26",
            "dms_destst": "",
            "fr_dest": "801",
            "trade_type": "3",
            "tons_2018": "5.0",
            "tons_2019": "",
        },
        # Mexico import leg, 2018.
        {
            "fr_orig": "802",
            "dms_origst": "",
            "dms_destst": "26",
            "fr_dest": "",
            "trade_type": "2",
            "tons_2018": "3.0",
            "tons_2019": "4.0",
        },
        # Rest of Americas export leg, 2018 -- folds onto the same node as Mexico.
        {
            "fr_orig": "",
            "dms_origst": "26",
            "dms_destst": "",
            "fr_dest": "803",
            "trade_type": "3",
            "tons_2018": "1.0",
            "tons_2019": "2.0",
        },
        # Domestic row: both fr_orig/fr_dest empty, must contribute nothing.
        {
            "fr_orig": "",
            "dms_origst": "26",
            "dms_destst": "26",
            "fr_dest": "",
            "trade_type": "1",
            "tons_2018": "999.0",
            "tons_2019": "999.0",
        },
        # Excluded zone 806, must contribute nothing to any node.
        {
            "fr_orig": "806",
            "dms_origst": "",
            "dms_destst": "26",
            "fr_dest": "",
            "trade_type": "2",
            "tons_2018": "77.0",
            "tons_2019": "77.0",
        },
    ]
    with path.open("w", newline="") as fh:
        writer = csv.DictWriter(fh, fieldnames=fieldnames)
        writer.writeheader()
        for row in rows:
            writer.writerow(row)


def test_aggregate_faf_tons_imports_and_exports(tmp_path: Path) -> None:
    faf_csv = tmp_path / "faf.csv"
    _write_fake_faf_csv(faf_csv)
    agg = faf_tool.aggregate_faf_tons(faf_csv)
    # Canada 2018: import 10.0 + export 5.0 = 15.0; 2019: import 12.0 only (export blank).
    assert agg[(801, 2018)] == pytest.approx(15.0)
    assert agg[(801, 2019)] == pytest.approx(12.0)
    # Mexico 2018: import 3.0. Rest of Americas 2018: export 1.0.
    assert agg[(802, 2018)] == pytest.approx(3.0)
    assert agg[(803, 2018)] == pytest.approx(1.0)
    # Domestic row (trade_type=1, both zones blank) contributes to no zone key.
    assert 999.0 not in agg.values()
    # Zone 806 is aggregated at this stage (exclusion happens at the mapping step).
    assert agg[(806, 2018)] == pytest.approx(77.0)


def test_map_zone_tons_to_nodes_sums_and_excludes_806() -> None:
    zone_year_tons = {
        (801, 2018): 15.0,
        (802, 2018): 3.0,
        (803, 2018): 1.0,
        (806, 2018): 77.0,  # must be dropped
    }
    out = faf_tool.map_zone_tons_to_nodes(zone_year_tons)
    assert out[("canada", 2018)] == pytest.approx(15.0)
    assert out[("latin_america", 2018)] == pytest.approx(4.0)  # 3.0 + 1.0 folded together
    assert ("india", 2018) not in out
    assert not any(node == "russia_csi" for node, _year in out)


def test_aggregate_faf_tons_raises_on_empty_csv(tmp_path: Path) -> None:
    faf_csv = tmp_path / "empty.csv"
    faf_csv.write_text("fr_orig,fr_dest,tons_2018\n")
    with pytest.raises(faf_tool.ArtifactGenerationError):
        faf_tool.aggregate_faf_tons(faf_csv)


def test_checked_in_artifact_schema_and_pinned_values() -> None:
    """Read the REAL checked-in artifact and assert schema + a few pinned
    aggregate values computed against the actual FAF5.7.1 CSV."""
    artifact = (
        Path(__file__).resolve().parents[3]
        / "src/babylon/data/reference/faf_bloc_trade_tons.csv.gz"
    )
    assert artifact.is_file(), "checked-in FAF artifact missing"
    with gzip.open(artifact, mode="rt", newline="") as fh:
        reader = csv.DictReader(fh)
        assert reader.fieldnames == ["node_id", "year", "tons"]
        rows = list(reader)
    assert len(rows) == 42  # 6 covered nodes x 7 years (2018-2024)
    by_key = {(r["node_id"], int(r["year"])): float(r["tons"]) for r in rows}
    covered_nodes = {node_id for node_id, _year in by_key}
    assert covered_nodes == {
        "canada",
        "china",
        "eu",
        "latin_america",
        "southeast_asia",
        "sub_saharan_africa",
    }
    for year in range(2018, 2025):
        assert ("canada", year) in by_key
    # Pinned aggregate spot-checks (computed against the real FAF5.7.1 CSV).
    assert by_key[("canada", 2018)] == pytest.approx(613124.597691, rel=1e-9)
    assert by_key[("canada", 2024)] == pytest.approx(621326.198523, rel=1e-9)
    assert by_key[("china", 2018)] == pytest.approx(384931.169127, rel=1e-9)
