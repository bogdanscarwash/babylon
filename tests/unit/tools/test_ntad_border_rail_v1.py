"""Small generated ArcGIS sources exercise the rail evidence boundary offline."""

from __future__ import annotations

import copy
import json
import sys
from pathlib import Path
from typing import Any

import pytest
import yaml

ROOT = Path(__file__).resolve().parents[3]
sys.path.insert(0, str(ROOT / "tools"))

import verify_ntad_border_rail_v1 as rail  # type: ignore[import-not-found]  # noqa: E402


def source() -> tuple[dict[str, Any], dict[str, Any]]:
    features = []
    for arc, obj, to_node, country, state, county, end in [
        (489609, 158234, 441281, "US", "MI", "26163", [-83.07, 42.33]),
        (737940, 220803, 627250, "CA", "ON", None, [-83.05, 42.31]),
    ]:
        attributes = {
            "FRAARCID": arc,
            "OBJECTID": obj,
            "FRFRANODE": 441352,
            "TOFRANODE": to_node,
            "COUNTRY": country,
            "STATEAB": state,
            "STCNTYFIPS": county,
            "TRACKS": 2,
            "MILES": 0.875,
            "KM": 1.4,
            "SUBDIV": "WINDSOR",
            "RROWNER1": "CPKC",
            "RROWNER2": "CN",
            "RROWNER3": None,
        }
        features.append({"attributes": attributes, "geometry": {"paths": [[[-83.06, 42.32], end]]}})
    layer = {
        "id": 0,
        "name": "North American Rail Network Lines",
        "geometryType": "esriGeometryPolyline",
        "hasZ": False,
        "hasM": False,
        "extent": {"spatialReference": {"wkid": 4326}},
        "description": "Generated fixture metadata; not an actual survey.",
        "copyrightText": "Generated fixture.",
        "editingInfo": {"dataLastEditDate": 1784743420808},
    }
    response = {
        "geometryType": "esriGeometryPolyline",
        "spatialReference": {"wkid": 4326},
        "features": features,
    }
    return response, layer


def test_source_order_twins_preserve_each_polyline_orientation_and_identity() -> None:
    response, layer = source()
    result = rail.project_observation(response, layer)
    twin = copy.deepcopy(response)
    twin["features"].reverse()
    assert rail.project_observation(twin, layer) == result
    assert rail.semantic_digest(rail.project_observation(twin, layer)) == rail.semantic_digest(
        result
    )
    features = result["geometry"]["features"]
    assert [f["properties"]["FRAARCID"] for f in features] == [489609, 737940]
    assert features[0]["geometry"]["coordinates"] == [[-83.06, 42.32], [-83.07, 42.33]]
    assert features[1]["properties"]["STCNTYFIPS"] is None
    assert result["shared_node_id"] == 441352
    assert "capacity" not in features[0]["properties"]


@pytest.mark.parametrize("defect", ["missing", "extra", "duplicate", "transfer_limit", "error"])
def test_incomplete_or_ambiguous_response_refuses(defect: str) -> None:
    response, layer = source()
    if defect == "missing":
        response["features"].pop()
    elif defect == "extra":
        extra = copy.deepcopy(response["features"][0])
        extra["attributes"]["FRAARCID"] = 123
        response["features"].append(extra)
    elif defect == "duplicate":
        response["features"][1] = copy.deepcopy(response["features"][0])
    elif defect == "transfer_limit":
        response["exceededTransferLimit"] = True
    else:
        response["error"] = {"code": 500, "message": "upstream failure"}
    with pytest.raises(rail.RailEvidenceError):
        rail.project_observation(response, layer)


@pytest.mark.parametrize(
    "defect",
    [
        "crs",
        "nan",
        "infinity",
        "out_of_range",
        "multipart",
        "z",
        "reversed",
        "disconnected",
        "node",
        "tracks_bool",
        "missing_length",
    ],
)
def test_geometry_topology_and_quantity_faults_refuse(defect: str) -> None:
    response, layer = source()
    feature = response["features"][1]
    path = feature["geometry"]["paths"][0]
    if defect == "crs":
        response["spatialReference"]["wkid"] = 3857
    elif defect == "nan":
        path[1][0] = float("nan")
    elif defect == "infinity":
        feature["attributes"]["KM"] = float("inf")
    elif defect == "out_of_range":
        path[1][0] = 190
    elif defect == "multipart":
        feature["geometry"]["paths"].append(copy.deepcopy(path))
    elif defect == "z":
        path[1].append(5)
    elif defect == "reversed":
        path.reverse()
    elif defect == "disconnected":
        path[0][0] += 0.001
    elif defect == "node":
        feature["attributes"]["FRFRANODE"] = 7
    elif defect == "tracks_bool":
        feature["attributes"]["TRACKS"] = True
    else:
        del feature["attributes"]["KM"]
    with pytest.raises(rail.RailEvidenceError):
        rail.project_observation(response, layer)


@pytest.mark.parametrize("payload", [b'{"x":1,"x":2}', b'{"x":NaN}', b"[]"])
def test_ambiguous_json_refuses(payload: bytes) -> None:
    with pytest.raises(rail.RailEvidenceError):
        rail.decode_mapping(payload)


def test_exact_source_rebuild_is_deterministic_and_retains_provenance() -> None:
    response, layer = source()
    a, b = json.dumps(response).encode(), json.dumps(layer).encode()
    first = rail.build_artifact(a, b)
    assert first == rail.build_artifact(a, b)
    changed = copy.deepcopy(response)
    changed["features"][0]["geometry"]["paths"][0][1][0] -= 0.001
    second = rail.build_artifact(json.dumps(changed).encode(), b)
    assert first != second
    assert first["semantic_sha256"] != second["semantic_sha256"]
    assert first["provenance"]["response_sha256"] != second["provenance"]["response_sha256"]


def test_source_vertex_order_is_not_canonicalized_away() -> None:
    response, layer = source()
    path = response["features"][0]["geometry"]["paths"][0]
    path.insert(1, [-83.063, 42.323])
    path.insert(2, [-83.066, 42.326])
    original = rail.project_observation(response, layer)
    path[1], path[2] = path[2], path[1]
    altered = rail.project_observation(response, layer)
    assert rail.semantic_digest(original) != rail.semantic_digest(altered)


def test_quantities_and_source_vintage_both_bind_semantic_identity() -> None:
    response, layer = source()
    original = rail.semantic_digest(rail.project_observation(response, layer))
    response["features"][0]["attributes"]["TRACKS"] = 1
    assert rail.semantic_digest(rail.project_observation(response, layer)) != original
    response, layer = source()
    layer["editingInfo"]["dataLastEditDate"] += 1
    assert rail.semantic_digest(rail.project_observation(response, layer)) != original


def test_checked_source_and_artifact_verify_without_data_drive() -> None:
    result = rail.verify_repository(ROOT)
    assert result["arcs"] == 2
    assert result["vertices"] == 29


def test_verifier_rejects_changed_geometry_even_with_recomputed_file_hash(tmp_path: Path) -> None:
    contract = yaml.safe_load((ROOT / rail.CONTRACT_PATH).read_text())
    for entry in [contract["artifact"], *contract["sources"].values()]:
        target = tmp_path / entry["path"]
        target.parent.mkdir(parents=True, exist_ok=True)
        target.write_bytes((ROOT / entry["path"]).read_bytes())
    path = tmp_path / contract["artifact"]["path"]
    value = rail.decode_mapping(path.read_bytes())
    value["observation"]["geometry"]["features"][0]["geometry"]["coordinates"][1][0] += 0.001
    path.write_bytes(rail.canonical_bytes(value))
    contract["artifact"]["sha256"] = rail.sha256(path.read_bytes())
    contract_path = tmp_path / rail.CONTRACT_PATH
    contract_path.parent.mkdir(parents=True, exist_ok=True)
    contract_path.write_text(yaml.safe_dump(contract))
    with pytest.raises(rail.RailEvidenceError, match="rebuild"):
        rail.verify_repository(tmp_path)


def registry_fixture() -> tuple[dict[str, Any], dict[str, Any]]:
    contract = yaml.safe_load((ROOT / rail.CONTRACT_PATH).read_text())
    prior = {
        "version": "2.0.0",
        "schema": {"sha256": "s" * 64, "tables": 76},
        "product": {"sha256": "p" * 64, "sqlite_version": "3.53.1"},
        "artifacts": [
            {"name": "qcew_county_economics_mi_2024", "sha256": "q" * 64, "rows": 83},
            {"name": "qcew_county_sectors_mi_2024", "sha256": "r" * 64, "rows": 1603},
            {"name": "fact_hpms_road_segment", "sha256": "h" * 64, "rows": 0},
        ],
    }
    return contract, prior


def test_registered_artifact_exactly_agrees_with_contract() -> None:
    contract, _ = registry_fixture()
    rail.verify_registry(contract, (ROOT / "data-artifacts.yaml").read_bytes())


@pytest.mark.parametrize(
    "field,value",
    [
        ("mode", "generate"),
        ("rows", 3),
        ("sha256", "0" * 64),
        ("source_table", "mutable_runtime"),
        ("home", "elsewhere.json"),
        ("material_relation", "Measured capacity"),
    ],
)
def test_registry_rejects_pin_mode_and_authority_drift(field: str, value: object) -> None:
    contract, document = registry_fixture()
    entry = rail.expected_registry_entry(contract)
    entry[field] = value
    document["artifacts"].append(entry)
    with pytest.raises(rail.RailEvidenceError, match="registry"):
        rail.verify_registry(contract, yaml.safe_dump(document).encode())


@pytest.mark.parametrize("count", [0, 2])
def test_registry_requires_one_unambiguous_entry(count: int) -> None:
    contract, document = registry_fixture()
    document["artifacts"].extend([rail.expected_registry_entry(contract)] * count)
    with pytest.raises(rail.RailEvidenceError, match="registry"):
        rail.verify_registry(contract, yaml.safe_dump(document).encode())


def test_registration_delta_preserves_existing_bytes_schema_product_and_entries() -> None:
    contract, document = registry_fixture()
    before = (
        b"# Preserve this existing comment.\n" + yaml.safe_dump(document, sort_keys=False).encode()
    )
    entry = rail.expected_registry_entry(contract)
    suffix = yaml.safe_dump([entry], sort_keys=False).encode()
    # PyYAML's default list indentation is zero; the prior artifact list is last.
    after = before + suffix
    assert rail.verify_registration_delta(before, after, entry) == 3


@pytest.mark.parametrize(
    "section", ["version", "schema", "product", "prior_entry", "prior_comment"]
)
def test_registration_delta_rejects_every_prior_surface_change(section: str) -> None:
    contract, document = registry_fixture()
    before = b"# Keep me.\n" + yaml.safe_dump(document, sort_keys=False).encode()
    entry = rail.expected_registry_entry(contract)
    changed = copy.deepcopy(document)
    if section == "version":
        changed["version"] = "3.0.0"
    elif section in ("schema", "product"):
        changed[section]["sha256"] = "0" * 64
    elif section == "prior_entry":
        changed["artifacts"][1]["sha256"] = "0" * 64
    changed["artifacts"].append(entry)
    prefix = b"# Replaced.\n" if section == "prior_comment" else b"# Keep me.\n"
    after = prefix + yaml.safe_dump(changed, sort_keys=False).encode()
    with pytest.raises(rail.RailEvidenceError, match="prior"):
        rail.verify_registration_delta(before, after, entry)


def test_appended_root_override_cannot_change_product_even_with_unchanged_prefix() -> None:
    contract, document = registry_fixture()
    before = yaml.safe_dump(document, sort_keys=False).encode()
    entry = rail.expected_registry_entry(contract)
    after = before + yaml.safe_dump([entry], sort_keys=False).encode() + b"product: {}\n"
    with pytest.raises(rail.RailEvidenceError, match="prior"):
        rail.verify_registration_delta(before, after, entry)
