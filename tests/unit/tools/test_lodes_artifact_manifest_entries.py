"""Regeneration-safety tripwire for the hand-maintained LODES manifest entries
(Vol II Circulation program, Unit U2).

``data-artifacts.yaml``'s ``lodes_xwalk_tri_county_hex`` /
``lodes_od_tri_county_hex_*`` rows are hand-maintained by
``tools/make_lodes_tri_county_artifact.py`` — they carry no backing sqlite
reference-DB table, so ``tools/make_data_artifacts.py``'s ``ARTIFACTS`` tuple
never names them. ``make_data_artifacts.main()`` (no ``--check``) calls
``generate()`` (which enumerates ONLY ``ARTIFACTS`` — plus, with
``--full-coverage``, one parquet spec per governed table, still never these)
and then ``_rewrite_manifest_preserving_blocks(entries)``, which REWRITES the
manifest's whole ``artifacts:`` list from those entries alone — silently
dropping the LODES tail. This is a known, undischarged risk (see the
``EXCEPTION`` note at the top of ``data-artifacts.yaml``): no change to a
shared, actively-maintained generator without an owner ruling.

This test is the mitigation the risk note promises: it pins the LODES entries'
presence + content in the COMMITTED manifest, directly (not through the
generator). If a future ``make_data_artifacts.py`` run replaces
``data-artifacts.yaml`` and that replacement gets committed, this test reds —
turning a silent regeneration wipe into a loud, immediate failure instead of a
dormant data-path regression discovered later.
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

# Pinned from tools/make_lodes_tri_county_artifact.py's generation run (see
# tests/unit/economics/circulation/test_lodes_tri_county_artifact.py for the
# matching per-year nnz proof against the artifact files themselves).
_EXPECTED_ENTRIES: dict[str, dict[str, object]] = {
    "lodes_xwalk_tri_county_hex": {
        "rows": 813,
        "sha256": "24ec618e740e5dcc1ccf82ce572a4cd0240e2d256551bc84c290d24ab2b52487",
        "home": "src/babylon/data/reference/lodes/tri_county_hex_xwalk.csv.gz",
    },
    "lodes_od_tri_county_hex_2010": {
        "rows": 191332,
        "sha256": "f968a42454a2984116b1d41d248ad36b1b67e96c13210ea91ee9ba1617ce924f",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2010.csv.gz",
    },
    "lodes_od_tri_county_hex_2011": {
        "rows": 189957,
        "sha256": "5587486b3a2020cbbfef3d6ed3ed2b36926242ccb81c0ecff7a87622592fb33b",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2011.csv.gz",
    },
    "lodes_od_tri_county_hex_2012": {
        "rows": 194577,
        "sha256": "7a10250188102d54423035844c4c6317492f2ef3cf6e9816aad8d24748a7d784",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2012.csv.gz",
    },
    "lodes_od_tri_county_hex_2013": {
        "rows": 195680,
        "sha256": "84ff892a9c39d21ffa68dd1337972c4531cec88edf905792edfa6e784cb4f6be",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2013.csv.gz",
    },
    "lodes_od_tri_county_hex_2014": {
        "rows": 195892,
        "sha256": "3c909aea866875e7dd42c1b561fee9acd5265d902590350f1ac4560c3c2395f6",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2014.csv.gz",
    },
    "lodes_od_tri_county_hex_2015": {
        "rows": 203144,
        "sha256": "97f8476cb9549786433c36ed345793c1e66e8a06bd4f1c4db6975b20aec3b5d9",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2015.csv.gz",
    },
    "lodes_od_tri_county_hex_2016": {
        "rows": 205839,
        "sha256": "304617ad6e5e36bfa0dbecebdd4d89679fe537b3a0f91ed418bf88b27152705d",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2016.csv.gz",
    },
    "lodes_od_tri_county_hex_2017": {
        "rows": 205778,
        "sha256": "ac18d0f5cb9ec6b8275c2330357cb347220394e192de7f1351123c30911fb12a",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2017.csv.gz",
    },
    "lodes_od_tri_county_hex_2018": {
        "rows": 207309,
        "sha256": "6e200c1d6056a1f26d56240bd1a43ca56f653d3508e922235b8720b30b65f389",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2018.csv.gz",
    },
    "lodes_od_tri_county_hex_2019": {
        "rows": 205325,
        "sha256": "4462f307ae74167ac8ec8364ea0042d96d92d0f2e68bbfeeb8a5c9592d90b83c",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2019.csv.gz",
    },
    "lodes_od_tri_county_hex_2020": {
        "rows": 195984,
        "sha256": "badca82307ef53647c24ed13dc8053157b474d0c80be435c738818221ee96a3c",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2020.csv.gz",
    },
    "lodes_od_tri_county_hex_2021": {
        "rows": 200596,
        "sha256": "cad5c11af69f79cc6ad02c0bf44abdbc2344325e216c40fee085044b28359280",
        "home": "src/babylon/data/reference/lodes/od/mi_od_main_JT00_2021.csv.gz",
    },
}


def _manifest_entries_by_name() -> dict[str, dict[str, object]]:
    manifest = yaml.safe_load(_MANIFEST.read_text())
    return {entry["name"]: entry for entry in manifest["artifacts"]}


def test_lodes_entries_present_in_committed_manifest_with_pinned_content() -> None:
    """The tripwire itself: every hand-registered LODES entry survives in the
    committed manifest, with its rows/sha256/home unchanged. A real
    ``make_data_artifacts.py`` regeneration (no ``--check``) would drop all
    thirteen — this assertion is what turns that into a loud, immediate CI
    failure instead of a silent data-path regression."""
    by_name = _manifest_entries_by_name()
    missing = set(_EXPECTED_ENTRIES) - set(by_name)
    assert not missing, f"LODES manifest entries missing (regeneration wipe?): {sorted(missing)}"
    for name, expected in _EXPECTED_ENTRIES.items():
        entry = by_name[name]
        assert entry["rows"] == expected["rows"], f"{name}: row count drifted"
        assert entry["sha256"] == expected["sha256"], f"{name}: sha256 drifted"
        assert entry["home"] == expected["home"], f"{name}: home path drifted"
        assert entry["generator"] == "tools/make_lodes_tri_county_artifact.py"
        assert entry["mode"] == "generate"


def test_lodes_entries_are_not_managed_by_make_data_artifacts() -> None:
    """Documents WHY the risk exists: these names are absent from the
    ``ARTIFACTS`` tuple ``generate()`` enumerates, so
    ``_rewrite_manifest_preserving_blocks(generate(db))`` — what ``main()``
    actually runs — would never reproduce them. If this assertion ever fails
    (a name collides with a real ``ARTIFACTS`` entry), the risk this test
    guards has changed shape and both this module and the ``EXCEPTION`` note
    in ``data-artifacts.yaml`` need re-reading."""
    managed_names = {spec.name for spec in make_data_artifacts.ARTIFACTS}
    collision = managed_names & set(_EXPECTED_ENTRIES)
    assert not collision, f"LODES names now managed by ARTIFACTS: {sorted(collision)}"
