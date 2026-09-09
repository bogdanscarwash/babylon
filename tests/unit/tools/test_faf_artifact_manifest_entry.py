"""Regeneration-safety tripwire for the hand-maintained FAF manifest entry
(Program 26, Unit U3).

``data-artifacts.yaml``'s ``faf_bloc_trade_tons`` row is hand-maintained by
``tools/make_faf_bloc_tons_artifact.py`` — it carries no backing sqlite
reference-DB table, so ``tools/make_data_artifacts.py``'s ``ARTIFACTS`` tuple
never names it. ``make_data_artifacts.main()`` (no ``--check``) REWRITES the
manifest's whole ``artifacts:`` list from those entries alone, which would
silently drop this row. Mirrors
``tests/unit/tools/test_lodes_artifact_manifest_entries.py`` exactly — see
that module (and the ``EXCEPTION`` note at the top of ``data-artifacts.yaml``)
for the full risk writeup.
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

# Pinned from tools/make_faf_bloc_tons_artifact.py's generation run against
# the real FAF5.7.1_State_2018-2024.csv (see
# tests/unit/tools/test_faf_bloc_mapping.py for the aggregation-logic proof).
_EXPECTED_ENTRY: dict[str, object] = {
    "rows": 42,
    "sha256": "47b553a0f04234abd61d5d5a2290786e2fe5dce64664c3f1473304a94a039141",
    "home": "src/babylon/data/reference/faf_bloc_trade_tons.csv.gz",
}


def _manifest_entries_by_name() -> dict[str, dict[str, object]]:
    manifest = yaml.safe_load(_MANIFEST.read_text())
    return {entry["name"]: entry for entry in manifest["artifacts"]}


def test_faf_entry_present_in_committed_manifest_with_pinned_content() -> None:
    """The tripwire itself: the hand-registered FAF entry survives in the
    committed manifest, with its rows/sha256/home unchanged. A real
    ``make_data_artifacts.py`` regeneration (no ``--check``) would drop it —
    this assertion turns that into a loud, immediate CI failure instead of a
    silent data-path regression (``bilateral_trade_tons`` quietly reverting
    to its permanent 0.0 stub)."""
    by_name = _manifest_entries_by_name()
    assert "faf_bloc_trade_tons" in by_name, (
        "faf_bloc_trade_tons entry missing (regeneration wipe?)"
    )
    entry = by_name["faf_bloc_trade_tons"]
    assert entry["rows"] == _EXPECTED_ENTRY["rows"], "faf_bloc_trade_tons: row count drifted"
    assert entry["sha256"] == _EXPECTED_ENTRY["sha256"], "faf_bloc_trade_tons: sha256 drifted"
    assert entry["home"] == _EXPECTED_ENTRY["home"], "faf_bloc_trade_tons: home path drifted"
    assert entry["generator"] == "tools/make_faf_bloc_tons_artifact.py"
    assert entry["mode"] == "generate"


def test_faf_entry_is_not_managed_by_make_data_artifacts() -> None:
    """Documents WHY the risk exists: this name is absent from the
    ``ARTIFACTS`` tuple ``generate()`` enumerates, so
    ``_rewrite_manifest_preserving_blocks(generate(db))`` — what ``main()``
    actually runs — would never reproduce it."""
    managed_names = {spec.name for spec in make_data_artifacts.ARTIFACTS}
    assert "faf_bloc_trade_tons" not in managed_names
