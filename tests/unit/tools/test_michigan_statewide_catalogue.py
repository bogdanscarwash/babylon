"""Commodity admission must cover the authored physical economy without imputation."""

from __future__ import annotations

import tomllib
from pathlib import Path

ROOT = Path(__file__).resolve().parents[3]


def test_statewide_catalogue_has_exact_mass_and_two_binding_food_inputs() -> None:
    content = tomllib.loads((ROOT / "content/scenarios/michigan/defines.toml").read_text())
    assert content["SCHEMA_VERSION"] == 3
    goods = content["commodity"]
    templates = content["template"]
    assert len(templates) == 16
    assert all(row["GRAMS_PER_UNIT"] > 0 for row in goods.values())
    assert {row["UNIT"] for row in goods.values()} == {"kg", "item"}
    food = templates["prepared_food"]
    assert food["INPUT_UNITS_PER_BATCH"]["grain"] > 0
    assert food["INPUT_UNITS_PER_BATCH"]["paper_packaging"] > 0
    assert all(row["LABOR_HOURS_PER_BATCH"] > 0 for row in templates.values())
    for name, row in templates.items():
        assert row["OUTPUT_GOOD"] in goods, name
        assert row["OUTPUT_UNITS_PER_BATCH"] > 0, name
        assert row["INPUT_UNITS_PER_BATCH"], name
        assert set(row["INPUT_UNITS_PER_BATCH"]) <= set(goods), name
        assert set(row["OPENING_INPUT_UNITS"]) == set(row["INPUT_UNITS_PER_BATCH"]), name
    assert content["transport"]["ROAD_TRAVEL_PERIODS"] == 1
    assert content["transport"]["EVIDENCE_CLASS"] == "Designed"
