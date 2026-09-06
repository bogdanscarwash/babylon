"""Checked material demonstration provenance, independent of the acquisition host."""

from __future__ import annotations

import hashlib
import json
from pathlib import Path

import yaml

ROOT = Path(__file__).resolve().parents[3]


def test_bounded_content_and_industry_sources_match_exact_pins() -> None:
    contract = yaml.safe_load((ROOT / "contracts/michigan_material_scenario_v1.yaml").read_text())
    for entry in contract["artifacts"]:
        assert hashlib.sha256((ROOT / entry["path"]).read_bytes()).hexdigest() == entry["sha256"]
    observed = json.loads((ROOT / contract["artifacts"][0]["path"]).read_text())
    assert len(observed["rows"]) == 5
    pins = {
        entry["file"]: entry["sha256"]
        for entry in json.loads(
            (ROOT / "tools/qcew_county_economics_v1_source_manifest.json").read_text()
        )["entries"]
    }
    for row in observed["rows"]:
        assert row["disclosure_code"] == ""
        assert row["own_code"] == "5"
        assert row["source_sha256"] == pins[row["source_file"]]
        assert row["annual_avg_emplvl"] > 0
    assert (
        observed["source_url"]
        == "https://data.bls.gov/cew/data/files/2024/csv/2024_annual_by_area.zip"
    )


def test_design_does_not_turn_observed_jobs_or_wages_into_material_quantities() -> None:
    contract = yaml.safe_load((ROOT / "contracts/michigan_material_scenario_v1.yaml").read_text())
    assert (
        contract["classifications"]["physical_recipe_inventory_labor_demand_capacity_and_delays"]
        == "Designed"
    )
    designed = json.loads((ROOT / contract["artifacts"][1]["path"]).read_text())
    assert designed["horizon_ticks"] == 16
    assert designed["terminal_output_disposition"] == "on_hand_unsold"
    assert len(designed["processes"]) == 5
    assert len(designed["routes"]) == 3
    assert all(
        "employment" not in key and "wage" not in key
        for process in designed["processes"]
        for key in process
    )
    assert all(
        "h3" not in site and "latitude" not in site and "longitude" not in site
        for site in designed["sites"]
    )
