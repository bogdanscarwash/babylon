#!/usr/bin/env python3
"""Qualify finite representative commodity relationships on supplied county paths.

This is content selection, not production, dispatch, allocation, or realization.
Local relationships retain their identity without a fictitious physical path.
"""

from __future__ import annotations

import argparse
import gzip
import hashlib
import json
import tomllib
from collections import defaultdict
from collections.abc import Mapping, Sequence
from dataclasses import asdict, dataclass, replace
from pathlib import Path
from typing import Any, Final

import make_michigan_commodity_roster as roster
import verify_michigan_commodity_roster_v1 as roster_verifier

MAX_U64: Final = 2**64 - 1
SCHEMA: Final = "MichiganCommodityCircuitV1"
PATH_SCHEMA: Final = "MichiganCountyPathMatrixV1"
MAX_PATH_MATRIX_BYTES: Final = 512 * 1024 * 1024
INDUSTRY_BY_FAMILY: Final = {
    family: industry for industry, family in roster.FAMILY_BY_INDUSTRY.items()
}


class QualificationError(ValueError):
    """An invalid input or missing source family prevents content qualification."""

    def __init__(self, code: str, detail: str) -> None:
        self.code, self.detail = code, detail
        super().__init__(f"{code}: {detail}")


@dataclass(frozen=True)
class Owner:
    county_geoid: str
    sector_code: str
    role: str
    primary_family: str | None
    eligible_families: tuple[str, ...]
    source_file: str = ""
    source_sha256: str = ""

    @property
    def identity(self) -> tuple[str, str]:
        return self.county_geoid, self.sector_code


@dataclass(frozen=True)
class CountyPath:
    distance_mm: int
    edge_ids: tuple[str, ...]


type CountyPaths = Mapping[tuple[str, str], CountyPath | None]


@dataclass(frozen=True)
class Good:
    name: str
    unit: str
    grams_per_unit: int
    disposition: str


@dataclass(frozen=True)
class InputCoefficient:
    good: str
    units_per_batch: int
    opening_units: int


@dataclass(frozen=True)
class Recipe:
    family: str
    output_good: str
    output_units_per_batch: int
    batches_per_week: int
    inputs: tuple[InputCoefficient, ...]


@dataclass(frozen=True)
class AuthoredCircuit:
    goods: tuple[Good, ...]
    recipes: tuple[Recipe, ...]
    finite_order_periods: int
    weeks_per_period: int
    horizon_periods: int
    source_sha256: str


@dataclass(frozen=True)
class Process:
    county_geoid: str
    sector_code: str
    family: str
    enrollment: str
    source_industry_code: str
    source_file: str
    source_sha256: str

    @property
    def identity(self) -> tuple[str, str, str]:
        return self.county_geoid, self.sector_code, self.family


@dataclass(frozen=True)
class FiniteOrder:
    supplier_county_geoid: str
    supplier_sector_code: str
    buyer_county_geoid: str
    buyer_sector_code: str
    good: str
    unit: str
    units: int
    supplier_family: str | None
    buyer_families: tuple[str, ...]
    purposes: tuple[str, ...]
    local: bool
    distance_mm: int
    edge_ids: tuple[str, ...]

    @property
    def identity(self) -> tuple[str, str, str, str, str, str]:
        return (
            self.supplier_county_geoid,
            self.supplier_sector_code,
            self.buyer_county_geoid,
            self.buyer_sector_code,
            self.good,
            self.unit,
        )


@dataclass(frozen=True)
class RetailFinalDemand:
    county_geoid: str
    sector_code: str
    good: str
    unit: str
    units: int


@dataclass(frozen=True)
class Diagnostic:
    code: str
    county_geoid: str
    sector_code: str
    family: str | None
    good: str
    eligible_source_counties: tuple[str, ...]


@dataclass(frozen=True)
class Qualification:
    owners: tuple[Owner, ...]
    processes: tuple[Process, ...]
    orders: tuple[FiniteOrder, ...]
    retail_final_demands: tuple[RetailFinalDemand, ...]
    diagnostics: tuple[Diagnostic, ...]
    defines_sha256: str

    @property
    def qualified(self) -> bool:
        return not self.diagnostics


def _uint(value: object, identity: str, *, positive: bool = True) -> int:
    if type(value) is not int or not (1 if positive else 0) <= value <= MAX_U64:
        raise QualificationError("unsigned_quantity", identity)
    return value


def _mapping(value: object, identity: str) -> dict[str, Any]:
    if not isinstance(value, dict) or any(not isinstance(key, str) for key in value):
        raise QualificationError("mapping_required", identity)
    return value


def parse_defines(document: dict[str, Any], *, source_sha256: str = "") -> AuthoredCircuit:
    """Read only the authored catalogue boundary needed for finite order quantities."""
    if (
        type(document.get("SCHEMA_VERSION")) is not int
        or document.get("SCHEMA_VERSION") != 3
        or type(document.get("TICK_DURATION_DAYS")) is not int
        or document.get("TICK_DURATION_DAYS") != 28
    ):
        raise QualificationError("defines_version", "schema 3 and 28-day periods required")
    horizon = _uint(document.get("HORIZON_PERIODS"), "HORIZON_PERIODS")
    statewide = _mapping(document.get("statewide"), "statewide")
    periods = _uint(statewide.get("FINITE_ORDER_PERIODS"), "FINITE_ORDER_PERIODS")
    if not periods <= horizon <= 16 or statewide.get("EVIDENCE_CLASS") != "Designed":
        raise QualificationError(
            "finite_order_horizon", "Designed orders must fit the sixteen-period horizon"
        )
    goods = []
    for name, raw in sorted(_mapping(document.get("commodity"), "commodity").items()):
        value = _mapping(raw, f"commodity.{name}")
        if set(value) != {"UNIT", "GRAMS_PER_UNIT", "DISPOSITION"}:
            raise QualificationError("good_fields", name)
        if value["UNIT"] not in {"kg", "item"} or value["DISPOSITION"] not in {
            "traded",
            "finite_opening",
        }:
            raise QualificationError("good_identity", name)
        goods.append(
            Good(name, value["UNIT"], _uint(value["GRAMS_PER_UNIT"], name), value["DISPOSITION"])
        )
    goods_by_name = {good.name: good for good in goods}
    recipes = []
    for family, raw in sorted(_mapping(document.get("template"), "template").items()):
        value = _mapping(raw, f"template.{family}")
        expected = {
            "OUTPUT_GOOD",
            "OUTPUT_UNITS_PER_BATCH",
            "INPUT_UNITS_PER_BATCH",
            "OPENING_INPUT_UNITS",
            "BATCHES_PER_WEEK",
            "LABOR_HOURS_PER_BATCH",
            "EMPLOYED_PEOPLE",
            "RESERVE_PEOPLE",
        }
        if set(value) != expected or family not in INDUSTRY_BY_FAMILY:
            raise QualificationError("template_fields", family)
        output = value["OUTPUT_GOOD"]
        if (
            not isinstance(output, str)
            or output not in goods_by_name
            or goods_by_name[output].disposition != "traded"
        ):
            raise QualificationError("template_output", family)
        coefficients = _mapping(value["INPUT_UNITS_PER_BATCH"], family)
        openings = _mapping(value["OPENING_INPUT_UNITS"], family)
        if set(coefficients) != set(openings) or not coefficients:
            raise QualificationError("template_inputs", family)
        inputs = []
        for name, units in sorted(coefficients.items()):
            if name not in goods_by_name:
                raise QualificationError("unknown_input", f"{family}/{name}")
            inputs.append(
                InputCoefficient(
                    name,
                    _uint(units, f"{family}/{name}"),
                    _uint(openings[name], f"{family}/{name}/opening", positive=False),
                )
            )
        for key in ("LABOR_HOURS_PER_BATCH", "EMPLOYED_PEOPLE", "RESERVE_PEOPLE"):
            _uint(value[key], f"{family}/{key}", positive=key != "RESERVE_PEOPLE")
        recipes.append(
            Recipe(
                family,
                output,
                _uint(value["OUTPUT_UNITS_PER_BATCH"], family),
                _uint(value["BATCHES_PER_WEEK"], family),
                tuple(inputs),
            )
        )
    outputs = [recipe.output_good for recipe in recipes]
    if len(outputs) != len(set(outputs)):
        raise QualificationError(
            "duplicate_output_family", "one representative family per traded good required"
        )
    return AuthoredCircuit(tuple(goods), tuple(recipes), periods, 4, horizon, source_sha256)


def read_defines(path: Path) -> AuthoredCircuit:
    raw = path.read_bytes()
    try:
        document = tomllib.loads(raw.decode("utf-8"))
    except (tomllib.TOMLDecodeError, UnicodeDecodeError) as error:
        raise QualificationError("defines_decode", str(path)) from error
    return parse_defines(document, source_sha256=hashlib.sha256(raw).hexdigest())


def owners_from_roster(document: dict[str, Any]) -> tuple[Owner, ...]:
    """Retain the already-qualified cohort owner and county acquisition identities."""
    if document.get("schema") != roster.SCHEMA:
        raise QualificationError("roster_schema", "MichiganCommodityRosterV1 required")
    sources = {row["county_geoid"]: row for row in document["sources"]}
    return tuple(
        Owner(
            row["county_geoid"],
            row["sector_code"],
            row["role"],
            row["primary_family"],
            tuple(row["eligible_families"]),
            sources[row["county_geoid"]]["file"],
            sources[row["county_geoid"]]["sha256"],
        )
        for row in document["actors"]
    )


def _validate_inputs(
    owners: Sequence[Owner], config: AuthoredCircuit, paths: CountyPaths
) -> tuple[Owner, ...]:
    ordered = tuple(
        replace(owner, eligible_families=tuple(sorted(owner.eligible_families)))
        for owner in sorted(owners, key=lambda owner: owner.identity)
    )
    if not ordered or len({owner.identity for owner in ordered}) != len(ordered):
        raise QualificationError("owner_identity", "nonempty unique county-sector owners required")
    families = {recipe.family for recipe in config.recipes}
    for owner in ordered:
        if owner.role not in {"producer", "wholesaler", "retailer"}:
            raise QualificationError("owner_role", str(owner.identity))
        if (
            len(owner.eligible_families) != len(set(owner.eligible_families))
            or not set(owner.eligible_families) <= families
        ):
            raise QualificationError("owner_eligibility", str(owner.identity))
        if owner.role == "producer":
            if owner.primary_family not in owner.eligible_families:
                raise QualificationError("owner_primary", str(owner.identity))
        elif owner.primary_family is not None or owner.eligible_families:
            raise QualificationError("merchant_family", str(owner.identity))
    counties = {owner.county_geoid for owner in ordered}
    expected = {(source, destination) for source in counties for destination in counties}
    if set(paths) != expected:
        raise QualificationError(
            "path_matrix_coverage", "every directed county pair needs a path or explicit None"
        )
    for (source, destination), path in paths.items():
        if path is not None:
            if not isinstance(path, CountyPath):
                raise QualificationError("physical_path", f"{source}/{destination}")
            _uint(path.distance_mm, f"path/{source}/{destination}", positive=source != destination)
        if source == destination:
            if path != CountyPath(0, ()):
                raise QualificationError("local_path", source)
        elif path is not None:
            if (
                not isinstance(path.edge_ids, tuple)
                or not path.edge_ids
                or any(not isinstance(edge, str) or not edge for edge in path.edge_ids)
            ):
                raise QualificationError("physical_path", f"{source}/{destination}")
    return ordered


def _process(owner: Owner, family: str, enrollment: str) -> Process:
    return Process(
        owner.county_geoid,
        owner.sector_code,
        family,
        enrollment,
        INDUSTRY_BY_FAMILY[family],
        owner.source_file,
        owner.source_sha256,
    )


def _suppliers(
    processes: Sequence[Process],
    good: str,
    buyer_county: str,
    recipes: Mapping[str, Recipe],
    paths: CountyPaths,
) -> list[Process]:
    candidates = [
        process
        for process in processes
        if recipes[process.family].output_good == good
        and paths[(process.county_geoid, buyer_county)] is not None
    ]
    return sorted(
        candidates,
        key=lambda process: (
            _distance(paths, process.county_geoid, buyer_county),
            process.identity,
        ),
    )


def _distance(paths: CountyPaths, source: str, destination: str) -> int:
    path = paths[(source, destination)]
    if path is None:
        raise QualificationError("disconnected_path", f"{source}/{destination}")
    return path.distance_mm


def _complete_processes(
    owners: Sequence[Owner], config: AuthoredCircuit, paths: CountyPaths
) -> tuple[tuple[Process, ...], tuple[Diagnostic, ...]]:
    recipes = {recipe.family: recipe for recipe in config.recipes}
    goods = {good.name: good for good in config.goods}
    output_families = {recipe.output_good: recipe.family for recipe in config.recipes}
    enrolled = {
        process.identity: process
        for owner in owners
        if owner.primary_family is not None
        for process in (_process(owner, owner.primary_family, "primary"),)
    }
    while True:
        uncovered: dict[str, list[Process]] = defaultdict(list)
        current = tuple(enrolled[key] for key in sorted(enrolled))
        for buyer in current:
            for coefficient in recipes[buyer.family].inputs:
                if goods[coefficient.good].disposition == "traded" and not _suppliers(
                    current, coefficient.good, buyer.county_geoid, recipes, paths
                ):
                    uncovered[coefficient.good].append(buyer)
        added = False
        for good, buyers in sorted(uncovered.items()):
            family = output_families.get(good)
            eligible = [owner for owner in owners if family in owner.eligible_families]
            if family is None or not eligible:
                raise QualificationError("family_source_coverage", good)
            candidates = []
            for owner in eligible:
                if (*owner.identity, family) in enrolled:
                    continue
                reachable = [
                    buyer
                    for buyer in buyers
                    if paths[(owner.county_geoid, buyer.county_geoid)] is not None
                ]
                if reachable:
                    candidates.append(
                        (
                            -len(reachable),
                            sum(
                                _distance(paths, owner.county_geoid, buyer.county_geoid)
                                for buyer in reachable
                            ),
                            owner.identity,
                            owner,
                        )
                    )
            if candidates:
                owner = min(candidates, key=lambda row: row[:3])[3]
                process = _process(owner, family, "upstream_completion")
                enrolled[process.identity] = process
                added = True
        if not added:
            diagnostics = []
            for good, buyers in sorted(uncovered.items()):
                family = output_families[good]
                counties = tuple(
                    sorted(
                        {
                            owner.county_geoid
                            for owner in owners
                            if family in owner.eligible_families
                        }
                    )
                )
                diagnostics.extend(
                    Diagnostic(
                        "disconnected_input",
                        buyer.county_geoid,
                        buyer.sector_code,
                        buyer.family,
                        good,
                        counties,
                    )
                    for buyer in buyers
                )
            return tuple(enrolled[key] for key in sorted(enrolled)), tuple(diagnostics)


def _quantity(per_batch: int, recipe: Recipe, config: AuthoredCircuit) -> int:
    return _uint(
        per_batch * recipe.batches_per_week * config.weeks_per_period * config.finite_order_periods,
        f"finite-order/{recipe.family}",
    )


def _make_order(
    supplier: Owner,
    buyer: Owner,
    good: Good,
    units: int,
    paths: CountyPaths,
    purpose: str,
    *,
    supplier_family: str | None = None,
    buyer_family: str | None = None,
) -> FiniteOrder:
    path = paths[(supplier.county_geoid, buyer.county_geoid)]
    if path is None:
        raise QualificationError("disconnected_order", f"{supplier.identity}/{buyer.identity}")
    return FiniteOrder(
        supplier.county_geoid,
        supplier.sector_code,
        buyer.county_geoid,
        buyer.sector_code,
        good.name,
        good.unit,
        _uint(units, good.name),
        supplier_family,
        (buyer_family,) if buyer_family is not None else (),
        (purpose,),
        supplier.county_geoid == buyer.county_geoid,
        path.distance_mm,
        path.edge_ids,
    )


def coalesce_orders(orders: Sequence[FiniteOrder]) -> tuple[FiniteOrder, ...]:
    """Combine requested units, retaining all participating processes and purposes."""
    result: dict[tuple[str, str, str, str, str, str], FiniteOrder] = {}
    for order in orders:
        _uint(order.units, str(order.identity))
        previous = result.get(order.identity)
        if previous is None:
            result[order.identity] = replace(
                order,
                buyer_families=tuple(sorted(set(order.buyer_families))),
                purposes=tuple(sorted(set(order.purposes))),
            )
            continue
        if (previous.supplier_family, previous.local, previous.distance_mm, previous.edge_ids) != (
            order.supplier_family,
            order.local,
            order.distance_mm,
            order.edge_ids,
        ):
            raise QualificationError("order_path_conflict", str(order.identity))
        result[order.identity] = replace(
            previous,
            units=_uint(previous.units + order.units, str(order.identity)),
            buyer_families=tuple(sorted(set(previous.buyer_families + order.buyer_families))),
            purposes=tuple(sorted(set(previous.purposes + order.purposes))),
        )
    return tuple(result[key] for key in sorted(result))


def _nearest(
    owners: Sequence[Owner], origin: Owner, paths: CountyPaths, *, inbound: bool = False
) -> Owner | None:
    def pair(owner: Owner) -> tuple[str, str]:
        return (
            (owner.county_geoid, origin.county_geoid)
            if inbound
            else (origin.county_geoid, owner.county_geoid)
        )

    candidates = [owner for owner in owners if paths[pair(owner)] is not None]
    return (
        min(candidates, key=lambda owner: (_distance(paths, *pair(owner)), owner.identity))
        if candidates
        else None
    )


def _merchant_orders(
    owners: Sequence[Owner],
    processes: Sequence[Process],
    config: AuthoredCircuit,
    paths: CountyPaths,
) -> tuple[list[FiniteOrder], list[Diagnostic]]:
    by_owner = {owner.identity: owner for owner in owners}
    recipes = {recipe.family: recipe for recipe in config.recipes}
    goods = {good.name: good for good in config.goods}
    wholesale = [owner for owner in owners if owner.role == "wholesaler"]
    retail = [owner for owner in owners if owner.role == "retailer"]
    orders, diagnostics = [], []

    def purchase(process: Process, merchant: Owner) -> None:
        recipe = recipes[process.family]
        orders.append(
            _make_order(
                by_owner[(process.county_geoid, process.sector_code)],
                merchant,
                goods[recipe.output_good],
                _quantity(recipe.output_units_per_batch, recipe, config),
                paths,
                "merchant_purchase",
                supplier_family=process.family,
            )
        )

    for process in processes:
        producer = by_owner[(process.county_geoid, process.sector_code)]
        merchant = _nearest(wholesale, producer, paths)
        if merchant is None:
            diagnostics.append(
                Diagnostic(
                    "disconnected_wholesale",
                    *producer.identity,
                    process.family,
                    recipes[process.family].output_good,
                    tuple(sorted({owner.county_geoid for owner in wholesale})),
                )
            )
        else:
            purchase(process, merchant)
    covered = {(order.buyer_county_geoid, order.buyer_sector_code) for order in orders}
    for merchant in wholesale:
        if merchant.identity in covered:
            continue
        candidates = [
            process
            for process in processes
            if paths[(process.county_geoid, merchant.county_geoid)] is not None
        ]
        if candidates:
            process = min(
                candidates,
                key=lambda process: (
                    _distance(paths, process.county_geoid, merchant.county_geoid),
                    process.identity,
                ),
            )
            purchase(process, merchant)
        else:
            diagnostics.append(
                Diagnostic(
                    "disconnected_merchant_source",
                    *merchant.identity,
                    None,
                    "",
                    tuple(sorted({process.county_geoid for process in processes})),
                )
            )
    quantities: dict[tuple[str, str], dict[str, int]] = defaultdict(dict)
    for order in orders:
        group = quantities[(order.buyer_county_geoid, order.buyer_sector_code)]
        group[order.good] = _uint(group.get(order.good, 0) + order.units, order.good)

    def resale(merchant: Owner, destination: Owner) -> None:
        for good, units in sorted(quantities[merchant.identity].items()):
            orders.append(
                _make_order(merchant, destination, goods[good], units, paths, "merchant_resale")
            )

    for merchant in wholesale:
        destination = _nearest(retail, merchant, paths)
        if destination is None:
            diagnostics.append(
                Diagnostic(
                    "disconnected_retail",
                    *merchant.identity,
                    None,
                    "",
                    tuple(sorted({owner.county_geoid for owner in retail})),
                )
            )
        elif quantities[merchant.identity]:
            resale(merchant, destination)
    covered_retail = {
        (order.buyer_county_geoid, order.buyer_sector_code)
        for order in orders
        if "merchant_resale" in order.purposes
    }
    for destination in retail:
        if destination.identity in covered_retail:
            continue
        candidates = [merchant for merchant in wholesale if quantities[merchant.identity]]
        merchant = _nearest(candidates, destination, paths, inbound=True)
        if merchant is None:
            diagnostics.append(
                Diagnostic(
                    "disconnected_retail_source",
                    *destination.identity,
                    None,
                    "",
                    tuple(sorted({owner.county_geoid for owner in candidates})),
                )
            )
        else:
            resale(merchant, destination)
    return orders, diagnostics


def qualify(owners: Sequence[Owner], config: AuthoredCircuit, paths: CountyPaths) -> Qualification:
    """Enroll a sparse source-supported circuit and its finite intended orders."""
    owners = _validate_inputs(owners, config, paths)
    processes, source_diagnostics = _complete_processes(owners, config, paths)
    recipes = {recipe.family: recipe for recipe in config.recipes}
    goods = {good.name: good for good in config.goods}
    by_owner = {owner.identity: owner for owner in owners}
    orders = []
    for buyer in processes:
        recipe = recipes[buyer.family]
        for coefficient in recipe.inputs:
            if goods[coefficient.good].disposition != "traded":
                continue
            suppliers = _suppliers(processes, coefficient.good, buyer.county_geoid, recipes, paths)
            if suppliers:
                supplier = suppliers[0]
                orders.append(
                    _make_order(
                        by_owner[(supplier.county_geoid, supplier.sector_code)],
                        by_owner[(buyer.county_geoid, buyer.sector_code)],
                        goods[coefficient.good],
                        _quantity(coefficient.units_per_batch, recipe, config),
                        paths,
                        "production_input",
                        supplier_family=supplier.family,
                        buyer_family=buyer.family,
                    )
                )
    merchant_orders, merchant_diagnostics = _merchant_orders(owners, processes, config, paths)
    combined = coalesce_orders(orders + merchant_orders)
    demands: dict[tuple[str, str, str, str], int] = {}
    for order in combined:
        if "merchant_resale" in order.purposes:
            key = (order.buyer_county_geoid, order.buyer_sector_code, order.good, order.unit)
            demands[key] = _uint(demands.get(key, 0) + order.units, str(key))
    diagnostics = tuple(
        sorted(
            source_diagnostics + tuple(merchant_diagnostics),
            key=lambda row: (
                row.code,
                row.county_geoid,
                row.sector_code,
                row.family or "",
                row.good,
                row.eligible_source_counties,
            ),
        )
    )
    return Qualification(
        owners,
        processes,
        combined,
        tuple(RetailFinalDemand(*key, units) for key, units in sorted(demands.items())),
        diagnostics,
        config.source_sha256,
    )


def qualification_document(result: Qualification) -> dict[str, Any]:
    return {
        "schema": SCHEMA,
        "evidence_class": "Designed",
        "qualified": result.qualified,
        **asdict(result),
    }


def read_paths(path: Path) -> dict[tuple[str, str], CountyPath | None]:
    """Read directed prequalified paths; physical topology is qualified upstream."""
    try:
        with gzip.open(path, "rb") as source:
            payload = source.read(MAX_PATH_MATRIX_BYTES + 1)
        if len(payload) > MAX_PATH_MATRIX_BYTES:
            raise QualificationError("path_byte_bound", str(MAX_PATH_MATRIX_BYTES))
        document = json.loads(payload)
    except (OSError, EOFError, UnicodeError, json.JSONDecodeError) as error:
        raise QualificationError(
            "path_encoding", "a gzip JSON county matrix is required"
        ) from error
    if (
        not isinstance(document, dict)
        or set(document) != {"schema", "source_pins", "policy", "terminals", "paths", "diagnostics"}
        or document["schema"] != PATH_SCHEMA
        or not isinstance(document["paths"], list)
        or len(document["paths"]) > 83 * 83
        or not isinstance(document["terminals"], list)
        or not isinstance(document["diagnostics"], list)
    ):
        raise QualificationError("path_schema", PATH_SCHEMA)
    pins = document["source_pins"]
    if (
        not isinstance(pins, dict)
        or set(pins) != {"atlas_sha256", "atlas_pin_sha256", "graph_sha256", "defines_sha256"}
        or any(
            not isinstance(value, str)
            or len(value) != 64
            or any(char not in "0123456789abcdef" for char in value)
            for value in pins.values()
        )
    ):
        raise QualificationError(
            "path_source_pins", "four explicit SHA256 source identities required"
        )
    if (
        not isinstance(document["policy"], dict)
        or document["policy"].get("terminal_evidence_class") != "Designed"
    ):
        raise QualificationError("path_policy", "aggregate county terminals must be Designed")
    result = {}
    for row in document["paths"]:
        if not isinstance(row, dict) or set(row) != {
            "source_county_geoid",
            "destination_county_geoid",
            "path",
        }:
            raise QualificationError("path_row", "unexpected fields")
        identity = (row["source_county_geoid"], row["destination_county_geoid"])
        if any(not isinstance(value, str) for value in identity) or identity in result:
            raise QualificationError("path_identity", str(identity))
        value = row["path"]
        if value is None:
            result[identity] = None
        elif (
            isinstance(value, dict)
            and set(value) == {"distance_mm", "edge_ids"}
            and isinstance(value["edge_ids"], list)
        ):
            result[identity] = CountyPath(
                _uint(value["distance_mm"], str(identity), positive=False), tuple(value["edge_ids"])
            )
        else:
            raise QualificationError("path_value", str(identity))
    return result


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--repo-root", type=Path, default=Path.cwd())
    parser.add_argument("--defines", type=Path)
    parser.add_argument("--paths", type=Path, required=True)
    parser.add_argument("--out", type=Path, required=True)
    args = parser.parse_args(argv)
    contract = roster_verifier.load_contract(args.repo_root / roster_verifier.CONTRACT_PATH)
    roster_verifier.verify_contract(contract)
    roster_verifier.verify_artifact(contract, args.repo_root)
    source = args.repo_root / roster.ARTIFACT_PATH
    document = roster_verifier.decode_artifact(source.read_bytes())
    defines = args.defines or args.repo_root / "content/scenarios/michigan/defines.toml"
    if args.out.resolve() in {source.resolve(), defines.resolve(), args.paths.resolve()}:
        raise QualificationError("output_source_overlap", str(args.out))
    result = qualify(owners_from_roster(document), read_defines(defines), read_paths(args.paths))
    output = {
        **qualification_document(result),
        "roster_sha256": hashlib.sha256(source.read_bytes()).hexdigest(),
        "paths_sha256": hashlib.sha256(args.paths.read_bytes()).hexdigest(),
    }
    args.out.parent.mkdir(parents=True, exist_ok=True)
    args.out.write_bytes(roster.canonical_json(output))
    print(
        json.dumps(
            {
                "qualified": result.qualified,
                "owners": len(result.owners),
                "processes": len(result.processes),
                "orders": len(result.orders),
                "diagnostics": len(result.diagnostics),
            },
            sort_keys=True,
        )
    )
    return 0 if result.qualified else 1


if __name__ == "__main__":
    raise SystemExit(main())
