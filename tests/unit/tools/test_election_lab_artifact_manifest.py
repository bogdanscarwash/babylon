"""Regeneration-safety tripwire + content contract for the MIT Election Lab
county Republican-vote-share artifact (P25 U6, ADR132; ratifies ADR049).

``data-artifacts.yaml``'s ``mit_countypres_rep_share`` row is hand-maintained
by ``tools/make_election_lab_artifact.py`` — no backing sqlite reference-DB
table, so ``tools/make_data_artifacts.py``'s ``ARTIFACTS`` tuple never names
it, and a plain ``make_data_artifacts.py`` regeneration would silently drop
the entry (the same documented EXCEPTION-class risk the LODES tail carries;
see ``tests/unit/tools/test_lodes_artifact_manifest_entries.py``). This module
is the promised mitigation: it pins the entry's presence + content in the
COMMITTED manifest and the committed artifact's bytes/shape, directly.

The content leg doubles as the behavioral contract for the FR-039 electoral
component: the Detroit tri-county rows carry the real 2024 county returns
(verified against independently known official results at derivation time),
which ``compute_seed_influences`` consumes for FAC_RESTORATIONIST.
"""

from __future__ import annotations

import csv
import hashlib
import sys
from pathlib import Path

import yaml

_REPO_ROOT = Path(__file__).resolve().parents[3]
_MANIFEST = _REPO_ROOT / "data-artifacts.yaml"
_ARTIFACT = _REPO_ROOT / "src/babylon/data/reference/election/mit_countypres_rep_share.csv"

_TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(_TOOLS_DIR))

import make_data_artifacts  # type: ignore[import-not-found]  # noqa: E402

_ENTRY_NAME = "mit_countypres_rep_share"

# Pinned from tools/make_election_lab_artifact.py's generation run (2026-07-22,
# raw mirror sha256 1a2323d8...; validation census: 3053 fips-correct, 54
# repaired, 39 non-county excluded, 42 unmatched excluded).
_EXPECTED_ROWS = 3107
_EXPECTED_SHA256 = "72c9ba04c3d9bc8001867a794338f77940b38b47dbd352084101126d380f7a90"
_EXPECTED_HOME = "src/babylon/data/reference/election/mit_countypres_rep_share.csv"

# The Detroit tri-county 2024 presidential returns (Wayne/Oakland/Macomb) —
# spot-verified against known official county results at derivation time.
_TRI_COUNTY_PINS = {
    "26163": ("MI", 288860, 856690, "0.337181"),
    "26125": ("MI", 337791, 772145, "0.437471"),
    "26099": ("MI", 284660, 509152, "0.559086"),
}


def _manifest_entry() -> dict[str, object]:
    manifest = yaml.safe_load(_MANIFEST.read_text())
    by_name = {entry["name"]: entry for entry in manifest["artifacts"]}
    assert _ENTRY_NAME in by_name, (
        f"{_ENTRY_NAME} missing from data-artifacts.yaml (regeneration wipe?)"
    )
    return by_name[_ENTRY_NAME]


def test_manifest_entry_present_with_pinned_content() -> None:
    """The tripwire: the hand-registered entry survives in the committed
    manifest with rows/sha256/home unchanged."""
    entry = _manifest_entry()
    assert entry["rows"] == _EXPECTED_ROWS, "row count drifted"
    assert entry["sha256"] == _EXPECTED_SHA256, "sha256 drifted"
    assert entry["home"] == _EXPECTED_HOME, "home path drifted"
    assert entry["generator"] == "tools/make_election_lab_artifact.py"
    assert entry["mode"] == "generate"


def test_entry_is_not_managed_by_make_data_artifacts() -> None:
    """Documents WHY the tripwire exists: the name is absent from the
    ``ARTIFACTS`` tuple, so a plain regeneration would never reproduce it."""
    managed_names = {spec.name for spec in make_data_artifacts.ARTIFACTS}
    assert _ENTRY_NAME not in managed_names, (
        "mit_countypres_rep_share is now managed by ARTIFACTS — the risk this "
        "test guards has changed shape; re-read the EXCEPTION note in "
        "data-artifacts.yaml"
    )


def test_committed_artifact_matches_manifest_pin() -> None:
    """The committed CSV's bytes hash to the manifest's sha256 and its data
    row count matches the manifest's rows."""
    text = _ARTIFACT.read_text(encoding="utf-8")
    digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
    assert digest == _EXPECTED_SHA256, "committed artifact bytes drifted from manifest pin"
    assert len(text.splitlines()) - 1 == _EXPECTED_ROWS


def test_artifact_shape_and_bounds() -> None:
    """Column contract + value bounds: 5-digit unique sorted FIPS keys, vote
    counts positive and consistent, shares in [0, 1]."""
    with _ARTIFACT.open(newline="", encoding="utf-8") as handle:
        reader = csv.DictReader(handle)
        assert reader.fieldnames == [
            "county_fips",
            "state_po",
            "county_name",
            "rep_votes",
            "total_votes",
            "rep_vote_share",
        ]
        rows = list(reader)
    fips = [row["county_fips"] for row in rows]
    assert fips == sorted(fips), "artifact must be FIPS-sorted"
    assert len(fips) == len(set(fips)), "duplicate county FIPS"
    assert all(len(f) == 5 and f.isdigit() for f in fips)
    for row in rows:
        rep = int(row["rep_votes"])
        total = int(row["total_votes"])
        share = float(row["rep_vote_share"])
        assert 0 <= rep <= total
        assert total > 0
        assert 0.0 <= share <= 1.0
        assert abs(share - rep / total) < 1e-6


def test_tri_county_rows_pin_real_2024_returns() -> None:
    """The FR-039 behavioral contract rows: the Detroit tri-county 2024
    presidential returns are the real official county results."""
    with _ARTIFACT.open(newline="", encoding="utf-8") as handle:
        by_fips = {row["county_fips"]: row for row in csv.DictReader(handle)}
    for fips, (state, rep, total, share) in _TRI_COUNTY_PINS.items():
        row = by_fips[fips]
        assert row["state_po"] == state
        assert int(row["rep_votes"]) == rep
        assert int(row["total_votes"]) == total
        assert row["rep_vote_share"] == share
