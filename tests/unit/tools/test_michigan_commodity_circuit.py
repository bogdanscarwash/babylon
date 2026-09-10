"""Finite representative circuit qualification, independent of roads and runtime."""

from __future__ import annotations

import copy
import gzip
import json
import sys
from dataclasses import asdict, fields, replace
from pathlib import Path
from typing import Any

import pytest

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))

import qualify_michigan_commodity_circuit as circuit  # type: ignore[import-not-found]  # noqa: E402


def catalogue() -> dict[str, Any]:
    goods = {
        name: {
            "UNIT": "item" if name == "machinery" else "kg",
            "GRAMS_PER_UNIT": 50000 if name == "machinery" else 1000,
            "DISPOSITION": "finite_opening" if name == "ore_deposit" else "traded",
        }
        for name in ("ore_deposit", "metal_ore", "metal_stock", "metal_parts", "machinery")
    }
    inputs = {
        "metal_ore": {"ore_deposit": 3},
        "metal_stock": {"metal_ore": 2},
        "metal_parts": {"metal_stock": 2},
        "machinery": {"metal_parts": 2},
    }
    return {
        "SCHEMA_VERSION": 3,
        "TICK_DURATION_DAYS": 28,
        "HORIZON_PERIODS": 16,
        "statewide": {"EVIDENCE_CLASS": "Designed", "FINITE_ORDER_PERIODS": 4},
        "commodity": goods,
        "template": {
            name: {
                "OUTPUT_GOOD": name,
                "OUTPUT_UNITS_PER_BATCH": 1,
                "INPUT_UNITS_PER_BATCH": recipe,
                "OPENING_INPUT_UNITS": dict.fromkeys(recipe, 100),
                "BATCHES_PER_WEEK": 1,
                "LABOR_HOURS_PER_BATCH": 1,
                "EMPLOYED_PEOPLE": 1,
                "RESERVE_PEOPLE": 0,
            }
            for name, recipe in inputs.items()
        },
    }


def scenario() -> tuple[list[circuit.Owner], dict[tuple[str, str], circuit.CountyPath | None]]:
    owners = [
        circuit.Owner("26001", "31-33", "producer", "machinery", ("machinery", "metal_stock")),
        circuit.Owner("26003", "31-33", "producer", "metal_parts", ("metal_parts", "metal_stock")),
        circuit.Owner("26005", "21", "producer", "metal_ore", ("metal_ore",)),
    ]
    for geoid in ("26001", "26003", "26005"):
        owners.extend(
            [
                circuit.Owner(geoid, "42", "wholesaler", None, ()),
                circuit.Owner(geoid, "44-45", "retailer", None, ()),
            ]
        )
    paths = {
        (source, destination): circuit.CountyPath(
            abs(int(source) - int(destination)) * 1000,
            () if source == destination else (f"{source}-{destination}",),
        )
        for source in ("26001", "26003", "26005")
        for destination in ("26001", "26003", "26005")
    }
    return owners, paths


def test_sparse_completion_preserves_owner_identity_and_local_input_relationship() -> None:
    owners, paths = scenario()
    result = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    assert len(result.owners) == 9
    assert [(process.county_geoid, process.family) for process in result.processes] == [
        ("26001", "machinery"),
        ("26003", "metal_parts"),
        ("26003", "metal_stock"),
        ("26005", "metal_ore"),
    ]
    assert result.diagnostics == ()
    order = next(
        order
        for order in result.orders
        if order.good == "metal_stock" and "production_input" in order.purposes
    )
    assert (order.supplier_county_geoid, order.buyer_county_geoid, order.units) == (
        "26003",
        "26003",
        32,
    )
    assert (order.local, order.distance_mm, order.edge_ids) == (True, 0, ())


def test_source_recipe_owner_and_path_permutations_preserve_qualification_identity() -> None:
    owners, paths = scenario()
    first = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    document = catalogue()
    document["commodity"] = dict(reversed(list(document["commodity"].items())))
    document["template"] = dict(reversed(list(document["template"].items())))
    other = [
        replace(owner, eligible_families=tuple(reversed(owner.eligible_families)))
        for owner in reversed(owners)
    ]
    second = circuit.qualify(
        other, circuit.parse_defines(document), dict(reversed(list(paths.items())))
    )
    assert asdict(first) == asdict(second)


def test_upstream_distance_tie_chooses_county_identity() -> None:
    owners, paths = scenario()
    owners = [
        replace(owner, eligible_families=("metal_parts",))
        if owner.primary_family == "metal_parts"
        else owner
        for owner in owners
    ]
    owners.append(
        circuit.Owner("26005", "31-33", "producer", "machinery", ("machinery", "metal_stock"))
    )
    result = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    added = [process for process in result.processes if process.enrollment == "upstream_completion"]
    assert [(process.county_geoid, process.family) for process in added] == [
        ("26001", "metal_stock")
    ]


def test_supplier_distance_tie_uses_stable_owner_identity() -> None:
    owners, paths = scenario()
    owners.append(circuit.Owner("26005", "31-33", "producer", "metal_parts", ("metal_parts",)))
    paths[("26005", "26001")] = circuit.CountyPath(2000, ("equal-distance-directed-edge",))
    result = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    order = next(
        order
        for order in result.orders
        if order.buyer_county_geoid == "26001" and "production_input" in order.purposes
    )
    assert order.supplier_county_geoid == "26003"


def test_every_producer_and_merchant_has_positive_finite_coverage_with_native_units() -> None:
    owners, paths = scenario()
    result = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    purchases = [order for order in result.orders if "merchant_purchase" in order.purposes]
    resales = [order for order in result.orders if "merchant_resale" in order.purposes]
    assert {
        (order.supplier_county_geoid, order.supplier_sector_code, order.supplier_family)
        for order in purchases
    } == {process.identity for process in result.processes}
    assert {(order.buyer_county_geoid, order.buyer_sector_code) for order in purchases} == {
        owner.identity for owner in owners if owner.role == "wholesaler"
    }
    assert {(order.supplier_county_geoid, order.supplier_sector_code) for order in resales} == {
        owner.identity for owner in owners if owner.role == "wholesaler"
    }
    assert {(row.county_geoid, row.sector_code) for row in result.retail_final_demands} == {
        owner.identity for owner in owners if owner.role == "retailer"
    }
    assert all(
        order.local and order.distance_mm == 0 and order.edge_ids == ()
        for order in purchases + resales
    )
    assert all(order.units == 16 for order in purchases)
    assert {
        (order.good, order.unit, order.units) for order in purchases if order.good == "machinery"
    } == {("machinery", "item", 16)}
    assert all(order.good != "ore_deposit" for order in result.orders)
    assert {field.name for field in fields(circuit.RetailFinalDemand)} == {
        "county_geoid",
        "sector_code",
        "good",
        "unit",
        "units",
    }


def test_directed_disconnection_is_diagnostic_without_a_reverse_or_zero_path() -> None:
    owners, paths = scenario()
    paths[("26005", "26003")] = None
    paths[("26005", "26001")] = None
    result = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    assert not result.qualified
    assert result.diagnostics == (
        circuit.Diagnostic(
            "disconnected_input", "26003", "31-33", "metal_stock", "metal_ore", ("26005",)
        ),
    )
    assert not any(
        order.good == "metal_ore" and order.buyer_county_geoid == "26003" for order in result.orders
    )
    assert paths[("26003", "26005")] is not None


def test_disconnected_regions_enroll_only_reachable_upstream_alternatives() -> None:
    owners, paths = scenario()
    owners.append(
        circuit.Owner("26005", "31-33", "producer", "metal_parts", ("metal_parts", "metal_stock"))
    )
    paths = {pair: path if pair[0] == pair[1] else None for pair, path in paths.items()}
    result = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    assert {
        (process.county_geoid, process.family)
        for process in result.processes
        if process.enrollment == "upstream_completion"
    } == {("26003", "metal_stock"), ("26005", "metal_stock")}
    assert all(order.local and order.edge_ids == () for order in result.orders)
    assert any(row.code == "disconnected_input" for row in result.diagnostics)


def test_missing_source_family_is_an_error_not_an_invented_process() -> None:
    owners, paths = scenario()
    owners = [
        replace(
            owner,
            eligible_families=tuple(
                family for family in owner.eligible_families if family != "metal_stock"
            ),
        )
        for owner in owners
    ]
    with pytest.raises(circuit.QualificationError, match="family_source_coverage: metal_stock"):
        circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)


def test_missing_matrix_cell_is_not_interpreted_as_disconnected_or_local() -> None:
    owners, paths = scenario()
    del paths[("26001", "26005")]
    with pytest.raises(circuit.QualificationError, match="path_matrix_coverage"):
        circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)


def test_local_relationship_refuses_fabricated_physical_edge() -> None:
    owners, paths = scenario()
    paths[("26001", "26001")] = circuit.CountyPath(1, ("fake-local-road",))
    with pytest.raises(circuit.QualificationError, match="local_path"):
        circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)


def test_duplicate_relationships_coalesce_units_and_consumer_families() -> None:
    first = circuit.FiniteOrder(
        "26001",
        "11",
        "26003",
        "31-33",
        "grain",
        "kg",
        10,
        "grain",
        ("prepared_food",),
        ("production_input",),
        False,
        10,
        ("physical-edge",),
    )
    second = replace(first, units=20, buyer_families=("packaged_beverages",))
    combined = circuit.coalesce_orders([second, first])
    assert len(combined) == 1
    assert combined[0].units == 30
    assert combined[0].buyer_families == ("packaged_beverages", "prepared_food")
    assert combined[0].unit == "kg"
    assert combined == circuit.coalesce_orders([first, second])
    with pytest.raises(circuit.QualificationError, match="order_path_conflict"):
        circuit.coalesce_orders([first, replace(second, edge_ids=("different-edge",))])


def test_finite_quantity_overflow_refuses() -> None:
    owners, paths = scenario()
    document = catalogue()
    document["template"]["metal_stock"]["INPUT_UNITS_PER_BATCH"]["metal_ore"] = circuit.MAX_U64
    with pytest.raises(circuit.QualificationError, match="unsigned_quantity"):
        circuit.qualify(owners, circuit.parse_defines(document), paths)


def test_added_process_keeps_its_existing_owner_source_identity() -> None:
    owners, paths = scenario()
    owners = [
        replace(owner, source_file="pinned-county.csv", source_sha256="1" * 64) for owner in owners
    ]
    result = circuit.qualify(owners, circuit.parse_defines(catalogue()), paths)
    process = next(
        process for process in result.processes if process.enrollment == "upstream_completion"
    )
    assert (process.source_industry_code, process.source_file, process.source_sha256) == (
        "331",
        "pinned-county.csv",
        "1" * 64,
    )


def test_path_input_represents_none_and_preserves_directed_edge_identity(tmp_path: Path) -> None:
    path = tmp_path / "paths.json.gz"
    path.write_bytes(
        gzip.compress(
            json.dumps(
                {
                    "schema": circuit.PATH_SCHEMA,
                    "source_pins": dict.fromkeys(
                        ("atlas_sha256", "atlas_pin_sha256", "graph_sha256", "defines_sha256"),
                        "1" * 64,
                    ),
                    "policy": {"terminal_evidence_class": "Designed"},
                    "terminals": [],
                    "diagnostics": [],
                    "paths": [
                        {
                            "source_county_geoid": "26001",
                            "destination_county_geoid": "26003",
                            "path": None,
                        },
                        {
                            "source_county_geoid": "26003",
                            "destination_county_geoid": "26001",
                            "path": {"distance_mm": 123, "edge_ids": ["road-b", "road-a"]},
                        },
                    ],
                }
            ).encode(),
            mtime=0,
        )
    )
    assert circuit.read_paths(path) == {
        ("26001", "26003"): None,
        ("26003", "26001"): circuit.CountyPath(123, ("road-b", "road-a")),
    }


@pytest.mark.parametrize(
    "field", ["BATCHES_PER_WEEK", "OUTPUT_UNITS_PER_BATCH", "LABOR_HOURS_PER_BATCH"]
)
def test_fractional_authored_recipe_values_refuse(field: str) -> None:
    document = copy.deepcopy(catalogue())
    document["template"]["metal_stock"][field] = 1.5
    with pytest.raises(circuit.QualificationError, match="unsigned_quantity"):
        circuit.parse_defines(document)
