"""Regeneration-safety tripwire for the hand-maintained national-incidence
manifest entries (#334 Phase 0, T5).

``data-artifacts.yaml``'s ``county_fips_vintage_crosswalk`` /
``national_incidence_county_pole`` / ``national_reproduction_floor`` rows are
hand-maintained by ``tools/make_fips_vintage_crosswalk.py`` (the first) and
``tools/make_national_incidence_artifact.py`` (the other two) — they carry no
backing sqlite reference-DB table (ADR098/ADR171 second-order disposition;
plan §2: "these are SECOND-ORDER PRODUCTS... they get no schema.sql table"),
so ``tools/make_data_artifacts.py``'s ``ARTIFACTS`` tuple never names them.
``make_data_artifacts.main()`` (no ``--check``) REWRITES the manifest's whole
``artifacts:`` list from those entries alone, which would silently drop all
three rows. Mirrors ``tests/unit/tools/test_faf_artifact_manifest_entry.py``'s
two-test shape exactly (pinned-content test + "not managed by
``make_data_artifacts``" test) — see that module (and the ``EXCEPTION`` note
at the top of ``data-artifacts.yaml``) for the full risk writeup.
"""

from __future__ import annotations

import sys
from pathlib import Path

import yaml

_REPO_ROOT = Path(__file__).resolve().parents[3]
_MANIFEST = _REPO_ROOT / "data-artifacts.yaml"

_TOOLS_DIR = _REPO_ROOT / "tools"
sys.path.insert(0, str(_TOOLS_DIR))

import make_data_artifacts  # type: ignore[import-not-found]  # noqa: E402

# Pinned from the generators' own printed manifest blocks (never hand-typed —
# docs/how-to/reference-data-pipeline.rst:63-65) — the T5 real-run report has
# the full provenance (verify_source_provenance against the sha-pinned
# dist/data-artifacts/*.parquet sources, then main()'s printed blocks).
_EXPECTED_ENTRIES: dict[str, dict[str, object]] = {
    "county_fips_vintage_crosswalk": {
        "rows": 17,
        "sha256": "bb53dad2eb1b5dba6690dd0fde995743d47bfea5898bae4a6e9afb5502ba8efe",
        "home": "src/babylon/data/reference/national/county_fips_vintage_crosswalk.csv",
        "generator": "tools/make_fips_vintage_crosswalk.py",
    },
    "national_incidence_county_pole": {
        "rows": 12612,
        "sha256": "8995ba396c691f86e9f92b879eca26f826311c1aa2b7fa0d4a9d6a14789531b4",
        "home": "src/babylon/data/reference/national/national_incidence_county_pole.csv.gz",
        "generator": "tools/make_national_incidence_artifact.py",
    },
    "national_reproduction_floor": {
        "rows": 15,
        "sha256": "cb2c5602008b02a53871153963aa931a81fa02e6a9b69b478b3f787ad4775016",
        "home": "src/babylon/data/reference/national/national_reproduction_floor.csv",
        "generator": "tools/make_national_incidence_artifact.py",
    },
}


def _manifest_entries_by_name() -> dict[str, dict[str, object]]:
    manifest = yaml.safe_load(_MANIFEST.read_text())
    return {entry["name"]: entry for entry in manifest["artifacts"]}


def test_national_incidence_entries_present_in_committed_manifest_with_pinned_content() -> None:
    """The tripwire itself: every hand-registered national-incidence entry
    survives in the committed manifest, with its rows/sha256/home/generator
    unchanged. A real ``make_data_artifacts.py`` regeneration (no
    ``--check``) would drop all three — this assertion is what turns that
    into a loud, immediate CI failure instead of a silent data-path
    regression (the #334 Phase 0 artifact program quietly reverting to
    nothing registered)."""
    by_name = _manifest_entries_by_name()
    missing = set(_EXPECTED_ENTRIES) - set(by_name)
    assert not missing, (
        f"national-incidence manifest entries missing (regeneration wipe?): {sorted(missing)}"
    )
    for name, expected in _EXPECTED_ENTRIES.items():
        entry = by_name[name]
        assert entry["rows"] == expected["rows"], f"{name}: row count drifted"
        assert entry["sha256"] == expected["sha256"], f"{name}: sha256 drifted"
        assert entry["home"] == expected["home"], f"{name}: home path drifted"
        assert entry["generator"] == expected["generator"], f"{name}: generator drifted"
        assert entry["mode"] == "generate"


def test_national_incidence_entries_are_not_managed_by_make_data_artifacts() -> None:
    """Documents WHY the risk exists: these three names are absent from the
    ``ARTIFACTS`` tuple ``generate()`` enumerates, so
    ``_rewrite_manifest_preserving_blocks(generate(db))`` — what ``main()``
    actually runs — would never reproduce them. If this assertion ever fails
    (a name collides with a real ``ARTIFACTS`` entry), the risk this test
    guards has changed shape and both this module and the ``EXCEPTION`` note
    in ``data-artifacts.yaml`` need re-reading."""
    managed_names = {spec.name for spec in make_data_artifacts.ARTIFACTS}
    collision = managed_names & set(_EXPECTED_ENTRIES)
    assert not collision, f"national-incidence names now managed by ARTIFACTS: {sorted(collision)}"
