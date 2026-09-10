"""Packaging preserves qualified physical facts and refuses mismatched authorities."""

import gzip
import hashlib
import json

import pytest
from tools.michigan_road_network_v1 import (
    Node,
    SourceIdentity,
    VehicleProfile,
    Way,
    build_graph,
    canonical_bytes,
)
from tools.qualify_michigan_road_terminals import (
    CountyAnchor,
    attach_terminals,
    matrix_document,
    qualify_paths,
    write_matrix,
)


def canonical(value):
    return (
        json.dumps(value, ensure_ascii=True, sort_keys=True, separators=(",", ":")).encode() + b"\n"
    )


def sha(path):
    return hashlib.sha256(path.read_bytes()).hexdigest()


def inputs(tmp_path):
    defines = tmp_path / "defines.toml"
    defines.write_text("""SCHEMA_VERSION = 3
[statewide]
EVIDENCE_CLASS = "Designed"
TERMINAL_ATTACHMENT_LIMIT_METERS = 50000
[transport]
EVIDENCE_CLASS = "Designed"
TRUCK_GROSS_WEIGHT_KG = 40000
TRUCK_HEIGHT_MM = 4000
TRUCK_WIDTH_MM = 2550
TRUCK_LENGTH_MM = 16500
DEFAULT_MAXHEIGHT_MM = 4000
EXTRACTION_BUFFER_DEGREES_E7 = 100000
""")
    graph = build_graph(
        [
            Node(1, -850000000, 440000000),
            Node(2, -849900000, 440000000),
            Node(3, -849800000, 440000000),
            Node(4, -840000000, 450000000),
            Node(5, -839900000, 450000000),
        ],
        [
            Way(10, (1, 2), (("highway", "primary"),)),
            Way(20, (2, 3), (("highway", "primary"), ("bridge", "yes"))),
            Way(30, (4, 5), (("highway", "secondary"),)),
        ],
        [],
        source=SourceIdentity(
            "a" * 64,
            100,
            "https://example.invalid/source.pbf",
            "2026-09-08T20:21:01Z",
            "b" * 64,
            100000,
            "fixture",
            "fixture",
        ),
        profile=VehicleProfile(40000, 4000, 2550, 16500, 4000),
    )
    graph_path = tmp_path / "graph.json"
    graph_path.write_bytes(canonical_bytes(graph))
    terminals = attach_terminals(
        [
            CountyAnchor("26001", "First County", -850000000, 440000000, 1, 1),
            CountyAnchor("26003", "Second County", -849800000, 440000000, 2, 1),
        ],
        graph,
        attachment_limit_mm=50_000_000,
    )
    matrix = tmp_path / "matrix.json.gz"
    paths = qualify_paths(graph, terminals)
    write_matrix(
        matrix_document(
            terminals,
            paths,
            {
                "graph_sha256": sha(graph_path),
                "defines_sha256": sha(defines),
                "atlas_sha256": "c" * 64,
                "atlas_pin_sha256": "d" * 64,
            },
            attachment_limit_mm=50_000_000,
        ),
        matrix,
    )
    roster = tmp_path / "roster.json.gz"
    roster.write_bytes(
        gzip.compress(
            canonical({"actors": [{"county_geoid": "26001"}, {"county_geoid": "26003"}]}), mtime=0
        )
    )
    path = paths[("26001", "26003")]
    qualification = tmp_path / "qualification.json"

    def owner(county):
        return {
            "county_geoid": county,
            "sector_code": "31-33",
            "role": "producer",
            "primary_family": "metal",
            "eligible_families": ["metal"],
            "source_file": "source.csv",
            "source_sha256": "e" * 64,
        }

    order = {
        "supplier_county_geoid": "26001",
        "supplier_sector_code": "31-33",
        "buyer_county_geoid": "26003",
        "buyer_sector_code": "31-33",
        "good": "metal",
        "unit": "kg",
        "units": 100,
        "supplier_family": "metal",
        "buyer_families": ["metal"],
        "purposes": ["production_input"],
        "local": False,
        "distance_mm": path.distance_mm,
        "edge_ids": list(path.edge_ids),
    }
    qualification.write_bytes(
        canonical(
            {
                "schema": "MichiganCommodityCircuitV1",
                "evidence_class": "Designed",
                "qualified": True,
                "owners": [owner("26001"), owner("26003")],
                "processes": [],
                "orders": [order],
                "retail_final_demands": [],
                "diagnostics": [],
                "defines_sha256": sha(defines),
                "roster_sha256": sha(roster),
                "paths_sha256": sha(matrix),
            }
        )
    )
    capacity_groups = tmp_path / "capacity-groups.json"
    capacity_groups.write_bytes(
        canonical(
            [
                {
                    "key": "supplied-pool",
                    "label": "Authored shared road service",
                    "edge_keys": list(reversed(path.edge_ids)),
                }
            ]
        )
    )
    return {
        "defines": defines,
        "qualification": qualification,
        "matrix": matrix,
        "graph": graph_path,
        "roster": roster,
        "capacity_groups": capacity_groups,
    }


def test_capture_keeps_only_selected_edges_and_pins_compressed_rust_inputs(tmp_path):
    from tools.capture_michigan_statewide_sources import capture_sources

    paths = inputs(tmp_path)
    source_qualification = json.loads(paths["qualification"].read_bytes())
    source_matrix = json.loads(gzip.decompress(paths["matrix"].read_bytes()))
    source_graph = json.loads(paths["graph"].read_bytes())
    manifest = capture_sources(**paths)
    assert manifest == json.loads((tmp_path / "statewide-sources.json").read_bytes())
    assert set(manifest) == {
        "schema",
        "defines_sha256",
        "qualification_sha256",
        "physical_network_sha256",
    }
    assert manifest["schema"] == "MichiganStatewideSourcesV1"
    assert manifest["defines_sha256"] == sha(paths["defines"])
    packed_qualification = tmp_path / "statewide-qualification.json.gz"
    packed_physical = tmp_path / "statewide-physical.json.gz"
    assert manifest["qualification_sha256"] == sha(packed_qualification)
    assert manifest["physical_network_sha256"] == sha(packed_physical)
    assert json.loads(gzip.decompress(packed_qualification.read_bytes())) == source_qualification
    physical = json.loads(gzip.decompress(packed_physical.read_bytes()))
    assert source_graph["routing_profile_version"] == "michigan-freight-routing-v1"
    assert physical["source"] == {
        **source_graph["source"],
        "routing_profile_version": source_graph["routing_profile_version"],
        "graph_sha256": sha(paths["graph"]),
    }
    assert physical["profile"] == source_graph["profile"]
    assert physical["terminal_source_pins"] == source_matrix["source_pins"]
    assert physical["terminal_policy"] == source_matrix["policy"]
    assert physical["terminals"] == source_matrix["terminals"]
    assert physical["terminal_attachment_limit_meters"] == 50000
    assert [edge["id"] for edge in physical["edges"]] == ["w10:0:1:f", "w20:0:1:f"]
    assert physical["edges"] == [
        edge for edge in source_graph["edges"] if edge["id"] in {"w10:0:1:f", "w20:0:1:f"}
    ]
    assert physical["capacity_groups"] == [
        {
            "key": "supplied-pool",
            "label": "Authored shared road service",
            "edge_keys": ["w10:0:1:f", "w20:0:1:f"],
        }
    ]
    first = {
        path.name: path.read_bytes()
        for path in (packed_qualification, packed_physical, tmp_path / "statewide-sources.json")
    }
    assert capture_sources(**paths) == manifest
    assert first == {name: (tmp_path / name).read_bytes() for name in first}


@pytest.mark.parametrize("changed", ["defines", "roster", "matrix", "graph"])
def test_capture_refuses_stale_input_hash_without_publication(tmp_path, changed):
    from tools.capture_michigan_statewide_sources import CaptureError, capture_sources

    paths = inputs(tmp_path)
    paths[changed].write_bytes(paths[changed].read_bytes() + b"\n")
    with pytest.raises(CaptureError, match="differs"):
        capture_sources(**paths)
    assert not list(tmp_path.glob("statewide-*"))


def update_pins(paths, *, graph=None, matrix=None, qualification=None):
    if graph is not None:
        paths["graph"].write_bytes(canonical(graph))
    matrix = (
        matrix if matrix is not None else json.loads(gzip.decompress(paths["matrix"].read_bytes()))
    )
    matrix["source_pins"]["graph_sha256"] = sha(paths["graph"])
    write_matrix(matrix, paths["matrix"])
    qualification = (
        qualification
        if qualification is not None
        else json.loads(paths["qualification"].read_bytes())
    )
    qualification["paths_sha256"] = sha(paths["matrix"])
    paths["qualification"].write_bytes(canonical(qualification))


@pytest.mark.parametrize("version", [None, "unknown-routing-policy"])
def test_capture_refuses_unsupported_routing_profile_even_with_matching_hashes(tmp_path, version):
    from tools.capture_michigan_statewide_sources import CaptureError, capture_sources

    paths = inputs(tmp_path)
    graph = json.loads(paths["graph"].read_bytes())
    if version is None:
        graph.pop("routing_profile_version", None)
    else:
        graph["routing_profile_version"] = version
    update_pins(paths, graph=graph)
    with pytest.raises(CaptureError, match="routing profile|physical graph schema"):
        capture_sources(**paths)
    assert not list(tmp_path.glob("statewide-*"))


@pytest.mark.parametrize(
    "fault",
    [
        "missing_edge",
        "bad_way_identity",
        "bad_geometry_endpoint",
        "bad_distance",
        "disconnected_sequence",
        "absent_county_path",
        "terminal_node_mismatch",
    ],
)
def test_capture_refuses_broken_selected_path_even_with_matching_file_hashes(tmp_path, fault):
    from tools.capture_michigan_statewide_sources import CaptureError, capture_sources

    paths = inputs(tmp_path)
    graph = json.loads(paths["graph"].read_bytes())
    matrix = json.loads(gzip.decompress(paths["matrix"].read_bytes()))
    qualification = json.loads(paths["qualification"].read_bytes())
    if fault == "missing_edge":
        graph["edges"] = [edge for edge in graph["edges"] if edge["id"] != "w20:0:1:f"]
    elif fault == "bad_way_identity":
        graph["edges"][0]["way_id"] = 11
    elif fault == "bad_geometry_endpoint":
        graph["edges"][0]["shape_e7"].reverse()
    elif fault == "bad_distance":
        graph["edges"][0]["distance_mm"] += 1
    elif fault == "disconnected_sequence":
        qualification["orders"][0]["edge_ids"].reverse()
        matrix["paths"][1]["path"]["edge_ids"].reverse()
    elif fault == "absent_county_path":
        matrix["paths"][1]["path"] = None
    else:
        matrix["terminals"][0]["node_lon_e7"] += 1
    update_pins(paths, graph=graph, matrix=matrix, qualification=qualification)
    with pytest.raises(CaptureError, match="physical|original|county|qualified"):
        capture_sources(**paths)
    assert not list(tmp_path.glob("statewide-*"))


@pytest.mark.parametrize("fault", ["uncovered", "unused", "duplicate_key", "duplicate_edge"])
def test_capture_requires_exact_supplied_capacity_membership(tmp_path, fault):
    from tools.capture_michigan_statewide_sources import CaptureError, capture_sources

    paths = inputs(tmp_path)
    groups = json.loads(paths["capacity_groups"].read_bytes())
    if fault == "uncovered":
        groups[0]["edge_keys"].pop()
    elif fault == "unused":
        groups[0]["edge_keys"].append("w30:0:1:f")
    elif fault == "duplicate_key":
        groups.append(groups[0])
    else:
        groups[0]["edge_keys"].append(groups[0]["edge_keys"][0])
    paths["capacity_groups"].write_bytes(canonical(groups))
    with pytest.raises(CaptureError, match="capacity group"):
        capture_sources(**paths)
    assert not list(tmp_path.glob("statewide-*"))


def test_capture_refuses_unqualified_diagnostics_and_preserves_existing_files(tmp_path):
    from tools.capture_michigan_statewide_sources import CaptureError, capture_sources

    paths = inputs(tmp_path)
    qualification = json.loads(paths["qualification"].read_bytes())
    qualification["diagnostics"] = [{"code": "disconnected_input"}]
    paths["qualification"].write_bytes(canonical(qualification))
    with pytest.raises(CaptureError, match="disconnected or not qualified"):
        capture_sources(**paths)
    qualification["diagnostics"] = []
    paths["qualification"].write_bytes(canonical(qualification))
    existing = tmp_path / "statewide-physical.json.gz"
    existing.write_bytes(b"recoverable unrelated data")
    with pytest.raises(CaptureError, match="unrelated output overwrite"):
        capture_sources(**paths)
    assert existing.read_bytes() == b"recoverable unrelated data"
    assert not (tmp_path / "statewide-qualification.json.gz").exists()
    assert not (tmp_path / "statewide-sources.json").exists()


def test_failed_publication_removes_only_new_outputs(tmp_path, monkeypatch):
    import tools.capture_michigan_statewide_sources as capture

    paths = inputs(tmp_path)
    original_link = capture.os.link
    calls = 0

    def fail_second(source, destination):
        nonlocal calls
        calls += 1
        if calls == 2:
            raise OSError("fixture failed publication")
        original_link(source, destination)

    monkeypatch.setattr(capture.os, "link", fail_second)
    with pytest.raises(OSError, match="failed publication"):
        capture.capture_sources(**paths)
    assert not list(tmp_path.glob("statewide-*"))
    assert not list(tmp_path.glob(".statewide-*"))
    assert all(path.exists() for path in paths.values())


def test_cli_uses_fixed_sibling_filenames_and_reports_manifest(tmp_path, capsys):
    from tools.capture_michigan_statewide_sources import main

    paths = inputs(tmp_path)
    argv = [
        value for key, path in paths.items() for value in ("--" + key.replace("_", "-"), str(path))
    ]
    assert main(argv) == 0
    assert json.loads(capsys.readouterr().out) == json.loads(
        (tmp_path / "statewide-sources.json").read_bytes()
    )
