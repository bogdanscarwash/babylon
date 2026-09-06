"""Executable RED contracts for the PER-280 canonical-reader cutover."""

from __future__ import annotations

import hashlib
import json
import struct
from copy import deepcopy
from pathlib import Path
from typing import Any
from uuid import UUID

import pytest
from tools.execute_h3_reader_parity_v1 import execute_h3_reader_parity_v1
from tools.verify_h3_reader_cutover_v1 import (
    H3ReaderCutoverRefusal,
    discover_dynamic_reader_inventory,
    inspect_sql_literal,
    load_reader_cutover_contract,
    load_reader_parity_vectors,
    require_exact_reader_inventory,
    verify_non_h3_observer_surfaces,
    verify_reader_cutover_contract,
    verify_reader_parity_vectors,
)

ROOT = Path(__file__).resolve().parents[3]
CONTRACT = ROOT / "contracts" / "h3_reader_cutover_v1.yaml"
VECTORS = ROOT / "contracts" / "h3_reader_cutover_v1_vectors.jsonl"
FOUNDATION = ROOT / "contracts" / "michigan_dynamic_hex_foundation_v1.yaml"


class _ExactParityBackend:
    def __init__(self, results: dict[str, dict[str, list[dict[str, Any]]]]) -> None:
        self._results = results
        self.calls: list[tuple[str, dict[str, Any]]] = []

    def execute_reader_case(
        self, operation: str, inputs: dict[str, Any]
    ) -> dict[str, list[dict[str, Any]]]:
        self.calls.append((operation, inputs))
        return deepcopy(self._results[operation])


def _atom_value(atom: dict[str, Any]) -> Any:
    atom_type = atom["type"]
    value = atom["value"]
    if atom_type in {"h3_cell_id", "nullable_h3_cell_id"}:
        return None if value is None else int(value)
    if atom_type == "f64_bits":
        return struct.unpack(">d", bytes.fromhex(value))[0]
    if atom_type == "uuid":
        return UUID(value)
    return value


def _expected_results(
    vectors: list[dict[str, Any]],
) -> dict[str, dict[str, list[dict[str, Any]]]]:
    return {
        row["operation"]: {
            result_name: [
                {name: _atom_value(atom) for name, atom in result_row.items()}
                for result_row in result_rows
            ]
            for result_name, result_rows in row["expected"]["sets"].items()
        }
        for row in vectors
    }


def test_checked_in_contract_pins_the_frozen_estate_and_inventory() -> None:
    contract = load_reader_cutover_contract(CONTRACT)

    assert contract["meta"] == {
        "contract": "H3ReaderCutoverV1",
        "version": 1,
        "issue": "PER-280",
        "parent": "PER-21",
    }
    assert contract["bounds"] == {
        "contract_bytes": 65_536,
        "read_edges": 0,
        "source_files": 0,
        "runtime_consumer_edges": 0,
        "bootstrap_definition_authorities": 0,
        "dynamic_authorities": 0,
        "epoch7_read_edges": 13,
        "epoch7_source_files": 5,
        "epoch7_dynamic_authorities": 1,
        "current_views": 10,
        "persistent_tables": 15,
        "identity_fields": 21,
        "vector_bytes": 131_072,
        "vector_rows": 9,
        "vector_line_bytes": 32_768,
    }
    binding = contract["authority"]["foundation_binding"]
    assert binding == {
        "contract": "contracts/michigan_dynamic_hex_foundation_v1.yaml",
        "contract_sha256": ("83f4ced209fb361c48ce7500e6cf9d60d39a688f92b34d2ef48414ea396c996c"),
        "artifact_sha256": ("81ee8f8abbee6727655d52c6d56a6f2967af9dfdf01da53dd593da8339d650a4"),
        "source_r7_digest": ("7f8d126ee81356a60605013b4b1c23942a77a4b2d6f890125d6c938dae70228b"),
        "base_reference_cohort_digest": (
            "92b21ff325bde67f26565f52882d3664daacd6d51423f2a588344da012fd4161"
        ),
        "reference_bundle_digest": (
            "84bbffa9b2388aa168c065e710a61313fbd46522d2022b628f0919ecffec9831"
        ),
        "r8_child_parent": {
            "type": "MichiganH3R8ChildParentV1",
            "rows": 319_004,
            "child_field": "child_cell_id",
            "parent_field": "parent_r7_cell_id",
            "domain_utf8": "babylon.h3.reference-r8-child-parent.v1\0",
            "digest": ("b5ebf405140f6f79ddbc44fa1005b195bed0bc28e0eacf2d8e1697cd9c839491"),
        },
    }
    assert hashlib.sha256(FOUNDATION.read_bytes()).hexdigest() == binding["contract_sha256"]

    assert contract["read_edges"] == []
    edges = {(str(row["path"]), str(row["relation"])) for row in contract["epoch7_read_edges"]}
    assert (
        "src/babylon/engine/headless_runner/trace_emitter.py",
        "view_runtime_trace_emission",
    ) not in edges
    assert {
        relation for path, relation in edges if path == "src/babylon/persistence/archival.py"
    } == {
        "dynamic_hex_state",
        "hex_spatial_map",
        "hex_state",
        "hex_terrain_state",
        "immutable_reference_lodes_od_matrix",
        "infrastructure_link_state",
    }
    assert contract["bootstrap_definition_authorities"] == []


def test_retired_legacy_runtime_read_edges_are_absent_from_the_exact_census() -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    edges = {(str(row["path"]), str(row["relation"])) for row in contract["read_edges"]}
    retired = {
        ("src/babylon/persistence/postgres_runtime/_legacy.py", relation)
        for relation in {
            "hex_activity",
            "hex_cell",
            "hex_map",
            "hex_state",
            "infrastructure_link_state",
        }
    }

    assert edges.isdisjoint(retired)


def test_dynamic_reader_inventory_comes_from_executable_selectors() -> None:
    contract = load_reader_cutover_contract(CONTRACT)

    assert discover_dynamic_reader_inventory(contract, ROOT) == set()


def test_checked_in_sources_satisfy_the_exact_reader_inventory() -> None:
    contract = load_reader_cutover_contract(CONTRACT)

    assert verify_reader_cutover_contract(contract, ROOT) == []


@pytest.mark.parametrize(
    ("sql", "code"),
    [
        ("SELECT cell_id FROM dynamic_hex_state", "unqualified_relation"),
        ("SELECT h3_index FROM public.dynamic_hex_state", "legacy_identity_read"),
        ("SELECT * FROM public.dynamic_hex_state", "wildcard_identity_read"),
        (
            "SELECT h.cell_id FROM public.dynamic_hex_state h "
            "JOIN public.hex_spatial_map m ON m.h3_index = h.h3_index",
            "legacy_identity_read",
        ),
        ("SELECT cell_id FROM public.v_compat_dynamic_hex_state_v1", "compatibility_read"),
        ("SELECT cell_id FROM public.v_observer_dynamic_hex_state_v1", "compatibility_read"),
    ],
)
def test_sql_sentinel_refuses_noncanonical_h3_reads(sql: str, code: str) -> None:
    contract = load_reader_cutover_contract(CONTRACT)

    with pytest.raises(H3ReaderCutoverRefusal) as exc_info:
        inspect_sql_literal(contract, Path("src/babylon/probe.py"), sql)

    assert exc_info.value.code == code


def test_sql_sentinel_accepts_schema_qualified_bigint_reads() -> None:
    contract = load_reader_cutover_contract(CONTRACT)

    assert inspect_sql_literal(
        contract,
        Path("src/babylon/probe.py"),
        "SELECT h.cell_id, m.county_fips "
        "FROM public.dynamic_hex_state AS h "
        "JOIN public.hex_spatial_map AS m "
        "ON m.session_id = h.session_id AND m.cell_id = h.cell_id "
        "ORDER BY h.cell_id",
    ) == [
        ("dynamic_hex_state", "public.dynamic_hex_state"),
        ("hex_spatial_map", "public.hex_spatial_map"),
    ]


def test_non_h3_economic_observation_is_admitted_only_for_its_exact_owner() -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    sql = "SELECT register_bytes FROM public.v_observer_material_state_v1 WHERE resolve_tick=$1"
    owner = Path("rust/crates/babylon-persistence/src/observer_material.rs")
    assert inspect_sql_literal(contract, owner, sql) == []
    for foreign_path, foreign_sql in (
        (Path("src/babylon/probe.py"), sql),
        (owner, sql.replace("public.", "")),
        (owner, sql.replace("public.", "foreign_schema.")),
        (owner, sql.replace("v_observer_material_state_v1", "v_observer_unreviewed_v1")),
    ):
        with pytest.raises(H3ReaderCutoverRefusal, match="compatibility_read"):
            inspect_sql_literal(contract, foreign_path, foreign_sql)


def test_economic_observer_admission_cannot_hide_a_legacy_h3_join() -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    with pytest.raises(H3ReaderCutoverRefusal, match="legacy_identity_read"):
        inspect_sql_literal(
            contract,
            Path("rust/crates/babylon-persistence/src/observer_material.rs"),
            "SELECT hex.h3_index FROM public.v_observer_material_state_v1 AS material "
            "JOIN public.dynamic_hex_state AS hex ON hex.resolve_tick = material.resolve_tick",
        )


@pytest.mark.parametrize("drift", ["legacy_dependency", "unknown_view"])
def test_non_h3_observer_definition_cannot_become_an_h3_adapter(tmp_path: Path, drift: str) -> None:
    migration_root = Path("rust/crates/babylon-persistence/migrations")
    (tmp_path / migration_root).mkdir(parents=True)
    for name in ("observer_economy_v1.sql", "observer_material_v1.sql"):
        source = (ROOT / migration_root / name).read_text()
        if name == "observer_material_v1.sql":
            if drift == "legacy_dependency":
                source = source.replace(
                    "FROM babylon_state.material_campaign_foundation_v2",
                    "FROM public.dynamic_hex_state",
                )
            else:
                source += "\nCREATE VIEW public.v_observer_extra_v1 AS SELECT 1;\n"
        (tmp_path / migration_root / name).write_text(source)
    parent = load_reader_cutover_contract(ROOT / "contracts/h3_estate_contract_v1.yaml")
    with pytest.raises(H3ReaderCutoverRefusal) as error:
        verify_non_h3_observer_surfaces(parent, tmp_path)
    assert error.value.code == (
        "compatibility_read" if drift == "legacy_dependency" else "observer_declaration_drift"
    )


def test_tagged_lodes_read_requires_closed_kind_and_canonical_cell_columns() -> None:
    contract = load_reader_cutover_contract(CONTRACT)

    assert inspect_sql_literal(
        contract,
        Path("src/babylon/domain/economics/lodes_commute_matrix.py"),
        "SELECT home_cell_id, workplace_cell_id, workplace_dest_kind, "
        "workplace_dest, s000_workers "
        "FROM public.immutable_reference_lodes_od_matrix "
        "WHERE session_id = %s AND year = %s",
    ) == [
        (
            "immutable_reference_lodes_od_matrix",
            "public.immutable_reference_lodes_od_matrix",
        )
    ]


def test_numbered_historical_migrations_are_not_runtime_reader_surfaces() -> None:
    contract = load_reader_cutover_contract(CONTRACT)

    assert (
        inspect_sql_literal(
            contract,
            Path("src/babylon/persistence/migrations/0030_views_current.sql"),
            "SELECT h3_index FROM dynamic_hex_state",
        )
        == []
    )


def test_repository_gate_runs_the_canonical_reader_sentinel() -> None:
    mise = (ROOT / ".mise.toml").read_text(encoding="utf-8")

    assert '[tasks."check:h3-readers"]' in mise
    static_gate = mise.split('[tasks."check:sentinels-static"]', 1)[1].split("\n[", 1)[0]
    assert '"check:h3-readers"' in static_gate


def test_reader_inventory_refuses_deletion_of_one_governed_edge() -> None:
    expected = {
        ("src/babylon/first.py", "dynamic_hex_state"),
        ("src/babylon/second.py", "hex_spatial_map"),
    }
    observed = set(expected)
    observed.remove(("src/babylon/second.py", "hex_spatial_map"))

    with pytest.raises(H3ReaderCutoverRefusal) as exc_info:
        require_exact_reader_inventory(expected, observed)

    assert exc_info.value.code == "missing_reader"
    assert "second.py" in exc_info.value.detail


def test_language_neutral_parity_vectors_bind_every_frozen_reader_case() -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    vectors = load_reader_parity_vectors(VECTORS)

    assert [row["id"] for row in vectors] == contract["parity_cases"]
    assert verify_reader_parity_vectors(contract, vectors, ROOT) == []


@pytest.mark.parametrize(
    ("path", "value"),
    [
        (("contract_sha256",), "0" * 64),
        (("artifact_sha256",), "0" * 64),
        (("reference_bundle_digest",), "0" * 64),
        (("r8_child_parent", "digest"), "0" * 64),
        (("r8_child_parent", "rows"), 0),
        (("r8_child_parent", "type"), "OpaqueR8Rows"),
        (("r8_child_parent", "child_field"), "parent_r7_cell_id"),
        (("r8_child_parent", "domain_utf8"), "babylon.h3.reference-r8-child-parent.v1"),
    ],
)
def test_reader_foundation_binding_refuses_every_authority_weakening(
    path: tuple[str, ...], value: object
) -> None:
    contract = deepcopy(load_reader_cutover_contract(CONTRACT))
    cursor = contract["authority"]["foundation_binding"]
    for component in path[:-1]:
        cursor = cursor[component]
    cursor[path[-1]] = value

    with pytest.raises(H3ReaderCutoverRefusal) as exc_info:
        verify_reader_parity_vectors(contract, load_reader_parity_vectors(VECTORS), ROOT)

    assert exc_info.value.code == "foundation_binding"


@pytest.mark.parametrize(
    ("vector_index", "set_name", "row_index", "field", "expected"),
    [
        (
            0,
            "asof_rows",
            1,
            "c",
            {"type": "f64_bits", "value": "4000000000000000"},
        ),
        (
            1,
            "national_rows",
            0,
            "national_id",
            {"type": "text", "value": "USA"},
        ),
    ],
)
def test_parity_vectors_share_the_exact_live_fixture_values(
    vector_index: int,
    set_name: str,
    row_index: int,
    field: str,
    expected: dict[str, Any],
) -> None:
    vectors = load_reader_parity_vectors(VECTORS)

    actual = vectors[vector_index]["expected"]["sets"][set_name][row_index][field]
    assert actual == expected


def test_r8_parity_case_is_a_direct_reference_identity_projection() -> None:
    vector = load_reader_parity_vectors(VECTORS)[4]

    assert vector["id"] == "resolution_8_parent_reference_identity"
    assert vector["operation"] == "r8_parent_reference_identity"
    assert vector["expected"]["sets"] == {
        "parent_rows": [
            {
                "cell_id": {
                    "type": "h3_cell_id",
                    "value": "613164958701584383",
                },
                "parent_cell_id": {
                    "type": "h3_cell_id",
                    "value": "608661359088893951",
                },
            }
        ]
    }


def test_pagination_parity_rows_are_direct_ordered_cell_ids() -> None:
    vector = load_reader_parity_vectors(VECTORS)[7]

    assert vector["expected"]["sets"] == {
        "page_one": [
            {
                "cell_id": {
                    "type": "h3_cell_id",
                    "value": "608661359088893951",
                }
            },
            {
                "cell_id": {
                    "type": "h3_cell_id",
                    "value": "608661359105671167",
                }
            },
        ],
        "page_two": [
            {
                "cell_id": {
                    "type": "h3_cell_id",
                    "value": "608661359122448383",
                }
            }
        ],
    }


def test_archive_parity_has_no_unfrozen_manifest_digest_atom() -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    vector = load_reader_parity_vectors(VECTORS)[8]

    assert set(vector["expected"]["sets"]) == {"export_counts", "query_rows"}
    assert "sha256" not in contract["parity_vectors"]["atom_types"]


@pytest.mark.parametrize(
    ("mutation", "code"),
    [
        ("missing_case", "parity_case_coverage"),
        ("unknown_operation", "vector_operation"),
        ("stale_digest", "vector_digest"),
        ("duplicate_id", "duplicate_vector_id"),
        ("relation_coverage", "persistent_relation_coverage"),
        ("view_coverage", "current_view_coverage"),
        ("reader_edge_coverage", "runtime_reader_edge_coverage"),
        ("atom_type", "vector_atom"),
        ("atom_type_coverage", "vector_atom_coverage"),
    ],
)
def test_parity_vectors_refuse_every_anti_weakening_mutation(mutation: str, code: str) -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    vectors = deepcopy(load_reader_parity_vectors(VECTORS))
    if mutation == "missing_case":
        vectors.pop()
    elif mutation == "unknown_operation":
        vectors[0]["operation"] = "invented_fallback"
    elif mutation == "stale_digest":
        first_set = next(iter(vectors[0]["expected"]["sets"].values()))
        first_atom = next(iter(first_set[0].values()))
        first_atom["value"] = "4000000000000000"
    elif mutation == "duplicate_id":
        vectors[-1]["id"] = vectors[0]["id"]
    elif mutation == "relation_coverage":
        vectors[0]["coverage"]["relations"].remove("hex_latest")
    elif mutation == "view_coverage":
        vectors[0]["coverage"]["views"].remove("v_hex_state_asof")
    elif mutation == "reader_edge_coverage":
        covered = next(row for row in vectors if row["coverage"]["reader_edges"])
        covered["coverage"]["reader_edges"].pop()
    elif mutation == "atom_type":
        first_set = next(iter(vectors[0]["expected"]["sets"].values()))
        first_atom = next(iter(first_set[0].values()))
        first_atom["type"] = "json_number"
    else:
        isolation = vectors[3]["expected"]
        isolation["sets"]["isolation_counts"][0]["foreign_rows_absent"] = {
            "type": "i64",
            "value": 1,
        }
        canonical = json.dumps(
            isolation["sets"],
            ensure_ascii=True,
            separators=(",", ":"),
            sort_keys=True,
            allow_nan=False,
        ).encode("ascii")
        isolation["sha256"] = hashlib.sha256(
            b"babylon.h3-reader-parity-result.v1\0" + canonical
        ).hexdigest()

    with pytest.raises(H3ReaderCutoverRefusal) as exc_info:
        verify_reader_parity_vectors(contract, vectors, ROOT)

    assert exc_info.value.code == code


def test_parity_executor_dispatches_all_nine_cases_and_matches_exact_typed_rows() -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    vectors = load_reader_parity_vectors(VECTORS)
    backend = _ExactParityBackend(_expected_results(vectors))

    assert execute_h3_reader_parity_v1(contract, vectors, ROOT, backend) == []
    assert [operation for operation, _ in backend.calls] == [row["operation"] for row in vectors]


def test_parity_executor_reports_one_row_level_semantic_mismatch() -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    vectors = load_reader_parity_vectors(VECTORS)
    results = _expected_results(vectors)
    results["stable_pagination"]["page_two"][0]["cell_id"] = 608661359105671167
    backend = _ExactParityBackend(results)

    findings = execute_h3_reader_parity_v1(contract, vectors, ROOT, backend)

    assert len(findings) == 1
    assert findings[0].case_id == "stable_ordering_and_pagination"
    assert findings[0].code == "result_digest"


def test_parity_executor_has_one_injected_read_only_seam_and_no_sql_authority() -> None:
    source = (ROOT / "tools" / "execute_h3_reader_parity_v1.py").read_text(encoding="utf-8")

    assert "backend.execute_reader_case" in source
    for forbidden in ["SELECT ", "JOIN ", "INSERT ", "UPDATE ", "DELETE ", "CREATE "]:
        assert forbidden not in source


@pytest.mark.parametrize(
    ("replacement", "code"),
    [
        ({"type": "i64", "value": True}, "vector_atom"),
        ({"type": "h3_cell_id", "value": "0"}, "vector_atom"),
        ({"type": "f64_bits", "value": "7ff0000000000000"}, "vector_atom"),
        ({"type": "uuid", "value": "27900000-0000-0000-0000-00000000000A"}, "vector_atom"),
    ],
)
def test_parity_atom_vocabulary_is_closed_and_canonical(
    replacement: dict[str, Any], code: str
) -> None:
    contract = load_reader_cutover_contract(CONTRACT)
    vectors = deepcopy(load_reader_parity_vectors(VECTORS))
    first_set = next(iter(vectors[0]["expected"]["sets"].values()))
    first_name = next(iter(first_set[0]))
    first_set[0][first_name] = replacement

    with pytest.raises(H3ReaderCutoverRefusal) as exc_info:
        verify_reader_parity_vectors(contract, vectors, ROOT)

    assert exc_info.value.code == code
