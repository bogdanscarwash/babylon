#!/usr/bin/env python3
"""Verify and reconstruct two pinned NTAD rail observations without network access.

The service's vertex order and FR/TO node identities remain untouched. This
artifact has no capacity, routing permission, actor, H3, or simulation authority.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import math
from pathlib import Path
from typing import Any, Final

import yaml

CONTRACT_PATH: Final = "contracts/ntad_border_rail_v1.yaml"
REGISTRY_NAME: Final = "ntad_detroit_windsor_rail_v1"
REGISTRY_RELATION: Final = (
    "Part of PER-28/PER-31: two Observed NTAD North American Rail Network "
    "polylines from the official July 21, 2026 service revision, preserving "
    "FRAARCID 489609/737940, original FR/TO identities and vertex order, 29 "
    "EPSG:4326 vertices, shared node 441352, recorded lengths, tracks, and "
    "ownership labels. Captured response and layer metadata have exact HTTP-body "
    "and stored-file SHA-256 pins in contracts/ntad_border_rail_v1.yaml; the "
    "generator rebuilds offline. Shared topology and semantic identity are "
    "Derived; selection and canonical representation are Designed. This is "
    "source evidence only, with no HPMS replacement, freight capacity, border "
    "delay, loss rate, H3 allocation, factory placement, Canadian actor, runtime "
    "hydration, campaign mutation, or gate completion authority."
)
LAYER_URL: Final = (
    "https://services.arcgis.com/xOi1kZaI0eWDREZv/arcgis/rest/services/"
    "NTAD_North_American_Rail_Network_Lines/FeatureServer/0"
)
QUERY: Final = {
    "f": "json",
    "where": "FRAARCID IN (489609,737940)",
    "outFields": "*",
    "returnGeometry": "true",
    "outSR": "4326",
    "orderByFields": "FRAARCID",
    "returnZ": "false",
    "returnM": "false",
    "resultRecordCount": "3",
}
MAX_BYTES: Final = 131_072
MAX_VERTICES: Final = 4096
DOMAIN: Final = b"babylon.ntad-border-rail-observation.v1\0"
ARC_IDENTITIES: Final = {
    489609: (158234, 441352, 441281, "US", "MI", "26163"),
    737940: (220803, 441352, 627250, "CA", "ON", None),
}
IDENTITY_FIELDS: Final = ("OBJECTID", "FRFRANODE", "TOFRANODE", "COUNTRY", "STATEAB", "STCNTYFIPS")
CLASSIFICATIONS: Final = {
    "source_attributes_geometry_and_metadata": "Observed",
    "shared_node_and_semantic_identity": "Derived",
    "selection_and_canonical_representation": "Designed",
}
LIMITATIONS: Final = [
    "Two source polylines; not a complete Detroit-Windsor logistics network.",
    "Original FR/TO labels and vertex order; no permission to traverse either direction inferred.",
    "Recorded lengths are source attributes, not recomputed geodesic or travel-time estimates.",
    "Track count and ownership labels do not establish available freight capacity or access rights.",
    "No capacity, border delay, loss rate, H3 assignment, factory, supplier, or Canadian actor inferred.",
    "July 2026 service evidence; not a reconstructed 2024 or September 2025 network snapshot.",
    "No engine activation, campaign mutation, or mechanics authority.",
]


class RailEvidenceError(ValueError):
    """A source, canonical artifact, or declared boundary failed validation."""


def require(condition: bool, message: str) -> None:
    if not condition:
        raise RailEvidenceError(message)


def sha256(raw: bytes) -> str:
    return hashlib.sha256(raw).hexdigest()


def canonical_bytes(value: object) -> bytes:
    """Match the repository's sorted compact JSON convention, with a final LF."""
    try:
        return (
            json.dumps(value, sort_keys=True, separators=(",", ":"), allow_nan=False) + "\n"
        ).encode("utf-8")
    except (ValueError, TypeError) as error:
        raise RailEvidenceError("noncanonical JSON value") from error


def _unique_pairs(pairs: list[tuple[str, Any]]) -> dict[str, Any]:
    result: dict[str, Any] = {}
    for key, value in pairs:
        require(key not in result, f"duplicate JSON key: {key}")
        result[key] = value
    return result


def _invalid_constant(value: str) -> None:
    raise RailEvidenceError(f"nonfinite JSON constant: {value}")


def decode_mapping(raw: bytes) -> dict[str, Any]:
    require(len(raw) <= MAX_BYTES, "source exceeds byte bound")
    try:
        value = json.loads(raw, object_pairs_hook=_unique_pairs, parse_constant=_invalid_constant)
    except (ValueError, UnicodeDecodeError) as error:
        raise RailEvidenceError("invalid JSON mapping") from error
    return _mapping(value, "JSON")


def _mapping(value: object, label: str) -> dict[str, Any]:
    if not isinstance(value, dict):
        raise RailEvidenceError(f"expected {label} mapping")
    return value


def _number(value: object, label: str) -> int | float:
    require(type(value) in (int, float), f"invalid numeric {label}")
    assert isinstance(value, (int, float))
    require(not isinstance(value, int) or abs(value) <= 2**63 - 1, f"oversized {label}")
    require(math.isfinite(value), f"nonfinite {label}")
    return value


def _integer(value: object, label: str) -> int:
    require(type(value) is int and 0 < value <= 2**63 - 1, f"invalid integer {label}")
    assert isinstance(value, int)
    return value


def _text(value: object, label: str, *, nullable: bool = False) -> str | None:
    if value is None and nullable:
        return None
    require(isinstance(value, str) and 0 < len(value) <= 4096, f"invalid text {label}")
    assert isinstance(value, str)
    return value


def _coordinates(value: object) -> list[list[int | float]]:
    geometry = _mapping(value, "geometry")
    require(set(geometry) == {"paths"}, "only two-dimensional source paths admitted")
    paths = geometry["paths"]
    require(isinstance(paths, list) and len(paths) == 1, "expected one source path per arc")
    path = paths[0]
    require(isinstance(path, list) and 2 <= len(path) <= MAX_VERTICES, "invalid vertex count")
    result = []
    for point in path:
        require(isinstance(point, list) and len(point) == 2, "expected longitude/latitude pair")
        lon, lat = _number(point[0], "longitude"), _number(point[1], "latitude")
        require(-180 <= lon <= 180 and -90 <= lat <= 90, "coordinate outside EPSG:4326")
        result.append([lon, lat])
    require(result[0] != result[-1], "arc endpoints coincide")
    return result


def _feature(value: object) -> dict[str, Any]:
    feature = _mapping(value, "feature")
    attributes = _mapping(feature.get("attributes"), "attributes")
    arc_id = _integer(attributes.get("FRAARCID"), "FRAARCID")
    require(arc_id in ARC_IDENTITIES, "unexpected source arc")
    identity = tuple(attributes.get(key) for key in IDENTITY_FIELDS)
    require(identity == ARC_IDENTITIES[arc_id], "source node or territorial identity changed")
    for key in IDENTITY_FIELDS[:3]:
        _integer(attributes.get(key), key)
    properties = {"FRAARCID": arc_id, **dict(zip(IDENTITY_FIELDS, identity, strict=True))}
    properties["TRACKS"] = _integer(attributes.get("TRACKS"), "TRACKS")
    for key in ("MILES", "KM"):
        number = _number(attributes.get(key), key)
        require(number > 0, f"nonpositive source {key}")
        properties[key] = number
    properties["SUBDIV"] = _text(attributes.get("SUBDIV"), "SUBDIV")
    for key in ("RROWNER1", "RROWNER2", "RROWNER3"):
        require(key in attributes, f"missing source {key}")
        properties[key] = _text(attributes[key], key, nullable=True)
    return {
        "type": "Feature",
        "id": arc_id,
        "properties": properties,
        "geometry": {"type": "LineString", "coordinates": _coordinates(feature.get("geometry"))},
    }


def project_observation(response: dict[str, Any], layer: dict[str, Any]) -> dict[str, Any]:
    """Extract a bounded observation; preserve all selected source vertices and values."""
    for document in (response, layer):
        require("error" not in document, "ArcGIS error payload")
        require(document.get("geometryType") == "esriGeometryPolyline", "source is not polyline")
    require(response.get("exceededTransferLimit", False) is False, "incomplete ArcGIS response")
    crs = _mapping(response.get("spatialReference"), "response CRS")
    extent = _mapping(layer.get("extent"), "layer extent")
    layer_crs = _mapping(extent.get("spatialReference"), "layer CRS")
    require(crs.get("wkid") == 4326 and layer_crs.get("wkid") == 4326, "unexpected CRS")
    require(layer.get("hasZ") is False and layer.get("hasM") is False, "unsupported dimensions")
    require(type(layer.get("id")) is int and layer["id"] == 0, "unexpected layer ID")
    rows = response.get("features")
    require(isinstance(rows, list) and len(rows) == 2, "expected exactly two source arcs")
    assert isinstance(rows, list)
    features = sorted((_feature(value) for value in rows), key=lambda row: row["id"])
    require([row["id"] for row in features] == sorted(ARC_IDENTITIES), "duplicate or missing arc")
    starts = [row["geometry"]["coordinates"][0] for row in features]
    require(starts[0] == starts[1], "shared FR node has disconnected or reversed source geometry")
    editing = _mapping(layer.get("editingInfo"), "editing info")
    return {
        "source": {
            "layer_url": LAYER_URL,
            "layer_name": _text(layer.get("name"), "layer name"),
            "description": _text(layer.get("description"), "description"),
            "copyright": _text(layer.get("copyrightText"), "copyright"),
            "data_last_edit_unix_ms": _integer(editing.get("dataLastEditDate"), "data edit date"),
            "returned_crs": "EPSG:4326",
            "coordinate_order": "longitude-latitude",
        },
        "classifications": CLASSIFICATIONS,
        "limitations": LIMITATIONS,
        "shared_node_id": 441352,
        "geometry": {"type": "FeatureCollection", "features": features},
    }


def semantic_digest(observation: dict[str, Any]) -> str:
    return sha256(DOMAIN + canonical_bytes(observation))


def build_artifact(response_raw: bytes, layer_raw: bytes) -> dict[str, Any]:
    observation = project_observation(decode_mapping(response_raw), decode_mapping(layer_raw))
    return {
        "schema": "NtadBorderRailV1",
        "observation": observation,
        "semantic_sha256": semantic_digest(observation),
        "provenance": {
            "response_sha256": sha256(response_raw),
            "layer_sha256": sha256(layer_raw),
            "query_parameters": QUERY,
        },
    }


def bounded_read(path: Path) -> bytes:
    try:
        with path.open("rb") as stream:
            raw = stream.read(MAX_BYTES + 1)
    except OSError as error:
        raise RailEvidenceError(f"cannot read {path}") from error
    require(len(raw) <= MAX_BYTES, "file exceeds byte bound")
    return raw


def _pinned_file(root: Path, entry: dict[str, Any]) -> bytes:
    path = Path(entry["path"])
    require(not path.is_absolute() and ".." not in path.parts, "source path escapes repository")
    raw = bounded_read(root / path)
    require(sha256(raw) == entry["sha256"], f"file digest mismatch: {path}")
    if "http_body_sha256" in entry:
        require(raw.endswith(b"\n"), "captured source must have its declared terminal LF")
        require(sha256(raw[:-1]) == entry["http_body_sha256"], "captured HTTP body digest mismatch")
    return raw


def expected_registry_entry(contract: dict[str, Any]) -> dict[str, Any]:
    """The single register-mode entry admitted by this source-only contract."""
    return {
        "name": REGISTRY_NAME,
        "format": "json",
        "source_table": None,
        "generator": "tools/make_ntad_border_rail_artifact.py",
        "mode": "register",
        "rows": contract["artifact"]["arcs"],
        "sha256": contract["artifact"]["sha256"],
        "home": contract["artifact"]["path"],
        "material_relation": REGISTRY_RELATION,
    }


def _registry(raw: bytes) -> dict[str, Any]:
    require(len(raw) <= MAX_BYTES, "registry exceeds byte bound")
    try:
        document = _mapping(yaml.safe_load(raw), "registry")
    except yaml.YAMLError as error:
        raise RailEvidenceError("invalid registry YAML") from error
    require(isinstance(document.get("artifacts"), list), "registry artifact list missing")
    require(all(isinstance(row, dict) for row in document["artifacts"]), "invalid registry row")
    return document


def verify_registry(contract: dict[str, Any], raw: bytes) -> None:
    """Require one exact registry entry, including its source authority limits."""
    expected = expected_registry_entry(contract)
    rows = _registry(raw)["artifacts"]
    matches = [row for row in rows if row.get("name") == REGISTRY_NAME]
    require(matches == [expected], "registry entry differs from rail contract")
    require(
        sum(row.get("home") == expected["home"] for row in rows) == 1,
        "registry has duplicate artifact home",
    )


def verify_registration_delta(prior: bytes, current: bytes, expected: dict[str, Any]) -> int:
    """Qualify one append against an explicit prior snapshot, without freezing future work."""
    require(current.startswith(prior), "prior registry bytes changed")
    before, after = _registry(prior), _registry(current)
    require(
        {key: value for key, value in before.items() if key != "artifacts"}
        == {key: value for key, value in after.items() if key != "artifacts"},
        "prior registry schema, product, or root metadata changed",
    )
    require(
        all(row.get("name") != REGISTRY_NAME for row in before["artifacts"]),
        "prior registry already contains rail entry",
    )
    require(
        after["artifacts"] == [*before["artifacts"], expected],
        "prior entries changed or registration was not one exact append",
    )
    return len(before["artifacts"])


def verify_repository(root: Path, prior_registry: Path | None = None) -> dict[str, Any]:
    """Read-only source replay, byte identity, and contract verification."""
    try:
        contract = yaml.safe_load(bounded_read(root / CONTRACT_PATH))
    except yaml.YAMLError as error:
        raise RailEvidenceError("invalid YAML contract") from error
    require(isinstance(contract, dict), "contract mapping required")
    require(
        contract.get("meta")
        == {
            "contract": "NtadBorderRailV1",
            "version": 1,
            "issues": ["PER-28", "PER-31"],
            "delivery": "source-observation-only",
        },
        "contract identity drift",
    )
    require(contract.get("classifications") == CLASSIFICATIONS, "evidence classification drift")
    require(contract.get("limitations") == LIMITATIONS, "source authority drift")
    require(contract.get("query") == QUERY, "source selection drift")
    require(contract.get("layer_url") == LAYER_URL, "source endpoint drift")
    sources = contract["sources"]
    rebuilt = build_artifact(
        _pinned_file(root, sources["response"]), _pinned_file(root, sources["layer"])
    )
    raw = _pinned_file(root, contract["artifact"])
    require(raw == canonical_bytes(rebuilt), "artifact does not match exact source rebuild")
    require(
        rebuilt["semantic_sha256"] == contract["artifact"]["semantic_sha256"],
        "semantic digest drift",
    )
    features = rebuilt["observation"]["geometry"]["features"]
    result = {
        "arcs": len(features),
        "vertices": sum(len(row["geometry"]["coordinates"]) for row in features),
        "bytes": len(raw),
        "sha256": sha256(raw),
        "semantic_sha256": rebuilt["semantic_sha256"],
    }
    require(
        all(contract["artifact"].get(key) == result[key] for key in ("arcs", "vertices", "bytes")),
        "artifact census drift",
    )
    require(
        contract.get("source_data_last_edit_unix_ms")
        == rebuilt["observation"]["source"]["data_last_edit_unix_ms"],
        "source edition drift",
    )
    registry = bounded_read(root / "data-artifacts.yaml")
    verify_registry(contract, registry)
    if prior_registry is not None:
        result["preserved_prior_entries"] = verify_registration_delta(
            bounded_read(prior_registry), registry, expected_registry_entry(contract)
        )
    return result


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--root", type=Path, default=Path(__file__).resolve().parents[1])
    parser.add_argument(
        "--prior-registry", type=Path, help="Verify exactly one append to this snapshot"
    )
    args = parser.parse_args()
    try:
        print(json.dumps(verify_repository(args.root, args.prior_registry), sort_keys=True))
    except (RailEvidenceError, KeyError, TypeError) as error:
        parser.exit(1, f"NTAD rail evidence refused: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
