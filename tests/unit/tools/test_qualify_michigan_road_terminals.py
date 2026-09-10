"""County anchors, physical attachments and directed paths share one pinned graph."""

import gzip
import hashlib
import json
from dataclasses import replace
from pathlib import Path

import pytest
from tools.michigan_road_network_v1 import (
    Node,
    Restriction,
    RestrictionMember,
    RoadRouter,
    SourceIdentity,
    VehicleProfile,
    Way,
    build_graph,
    canonical_bytes,
)
from tools.qualify_michigan_road_terminals import (
    CountyAnchor,
    TerminalQualificationError,
    attach_terminals,
    load_routing_view,
    matrix_document,
    qualify_paths,
    read_atlas_anchors,
    write_matrix,
)

ROOT = Path(__file__).resolve().parents[3]


def roads():
    return build_graph(
        [
            Node(1, -850000000, 440000000),
            Node(2, -849900000, 440000000),
            Node(3, -849800000, 440000000),
            Node(4, -840000000, 450000000),
            Node(5, -839900000, 450000000),
        ],
        [
            Way(10, (1, 2), (("highway", "primary"),)),
            Way(20, (2, 3), (("highway", "primary"), ("oneway", "yes"))),
            Way(30, (4, 5), (("highway", "primary"),)),
        ],
        [],
        source=SourceIdentity("a" * 64, 1, "fixture", "fixture", "b" * 64, 0, "fixture", "fixture"),
        profile=VehicleProfile(40000, 4000, 2550, 16500, 4000),
    )


def anchors():
    return (
        CountyAnchor("26001", "First County", -850000000, 440000000, 1, 1),
        CountyAnchor("26003", "Second County", -849900000, 440000000, 2, 1),
        CountyAnchor("26005", "Island County", -840000000, 450000000, 3, 1),
        CountyAnchor("26007", "Far County", -880000000, 470000000, 4, 1),
    )


def test_nearest_usable_node_is_bounded_and_keeps_county_identity():
    graph = roads()
    terminals = attach_terminals(anchors(), graph, attachment_limit_mm=50_000_000)
    assert [row.county_geoid for row in terminals] == ["26001", "26003", "26005", "26007"]
    assert [row.node_id for row in terminals] == [1, 2, 4, None]
    assert all(row.attachment_distance_mm > 0 for row in terminals if row.node_id is not None)
    assert terminals[-1].status == "no_usable_node_within_limit"


def test_matrix_retains_diagonal_zero_and_disconnected_pairs(monkeypatch):
    graph = roads()
    terminals = attach_terminals(anchors(), graph, attachment_limit_mm=50_000_000)
    calls = []
    original = RoadRouter.shortest_paths

    def tracked(self, start, destinations):
        calls.append(start)
        return original(self, start, destinations)

    monkeypatch.setattr(RoadRouter, "shortest_paths", tracked)
    paths = qualify_paths(graph, terminals)
    assert len(paths) == 16
    assert calls == [1, 2, 4]
    assert paths[("26001", "26001")].distance_mm == 0
    assert paths[("26001", "26003")].distance_mm == graph.edges[0].distance_mm
    assert paths[("26001", "26005")] is None
    assert paths[("26007", "26007")] is None


def test_streaming_view_preserves_paths_and_discards_drawing_metadata(tmp_path):
    graph = roads()
    path = tmp_path / "graph.json"
    path.write_bytes(canonical_bytes(graph))
    view = load_routing_view(
        path, hashlib.sha256(path.read_bytes()).hexdigest(), memory_budget_bytes=64 * 1024 * 1024
    )
    assert not hasattr(view.edges[0], "shape_e7")
    assert RoadRouter(view).shortest_paths(1, [1, 2, 3, 4]) == RoadRouter(graph).shortest_paths(
        1, [1, 2, 3, 4]
    )
    with pytest.raises(TerminalQualificationError, match="SHA-256"):
        load_routing_view(path, "0" * 64, memory_budget_bytes=64 * 1024 * 1024)


def test_pinned_atlas_provides_all_83_existing_interior_land_anchors():
    result, pins = read_atlas_anchors(
        ROOT / "assets/map/county_atlas.bin",
        ROOT / "rust/crates/babylon-client/tests/fixtures/michigan_atlas_land_probes.json",
    )
    assert len(result) == 83
    assert len({row.county_geoid for row in result}) == 83
    assert all(
        row.county_geoid.startswith("26")
        and -910000000 < row.lon_e7 < -820000000
        and 410000000 < row.lat_e7 < 490000000
        for row in result
    )
    assert (
        pins["atlas_sha256"]
        == hashlib.sha256((ROOT / "assets/map/county_atlas.bin").read_bytes()).hexdigest()
    )


def test_canonical_compression_and_row_permutations_keep_identity(tmp_path):
    graph = roads()
    first = attach_terminals(anchors(), graph, attachment_limit_mm=50_000_000)
    second = attach_terminals(tuple(reversed(anchors())), graph, attachment_limit_mm=50_000_000)
    assert first == second
    pins = {
        "graph_sha256": "a" * 64,
        "atlas_sha256": "b" * 64,
        "atlas_pin_sha256": "c" * 64,
        "defines_sha256": "d" * 64,
    }
    document = matrix_document(
        first, qualify_paths(graph, first), pins, attachment_limit_mm=50_000_000
    )
    one, two = tmp_path / "one.json.gz", tmp_path / "two.json.gz"
    assert write_matrix(document, one) == write_matrix(document, two)
    assert one.read_bytes() == two.read_bytes()
    decoded = json.loads(gzip.decompress(one.read_bytes()))
    assert decoded["schema"] == "MichiganCountyPathMatrixV1"
    assert len(decoded["paths"]) == 16
    assert decoded["terminals"][0]["evidence_class"] == "Designed"
    assert decoded["diagnostics"]


def test_equal_attachment_distances_use_stable_node_id():
    original = roads()
    twin_graph = build_graph(
        [
            Node(1, -850000000, 440000000),
            Node(2, -850000000, 440000000),
            Node(3, -849900000, 440000000),
        ],
        [Way(10, (1, 3), (("highway", "primary"),)), Way(20, (2, 3), (("highway", "primary"),))],
        [],
        source=original.source,
        profile=original.profile,
    )
    reversed_graph = replace(
        twin_graph, nodes=tuple(reversed(twin_graph.nodes)), edges=tuple(reversed(twin_graph.edges))
    )
    first = attach_terminals(anchors()[:1], twin_graph, attachment_limit_mm=50_000_000)
    assert first == attach_terminals(anchors()[:1], reversed_graph, attachment_limit_mm=50_000_000)
    assert first[0].node_id == 1


def test_streamed_view_keeps_via_way_entry_history(tmp_path):
    original = roads()
    graph = build_graph(
        [Node(index, -850000000 + index * 10000, 440000000) for index in range(1, 5)],
        [
            Way(10, (1, 2), (("highway", "primary"), ("oneway", "yes"))),
            Way(20, (2, 3), (("highway", "primary"), ("oneway", "yes"))),
            Way(30, (3, 4), (("highway", "primary"), ("oneway", "yes"))),
        ],
        [
            Restriction(
                100,
                (
                    RestrictionMember("w", 10, "from"),
                    RestrictionMember("w", 20, "via"),
                    RestrictionMember("w", 30, "to"),
                ),
                (("type", "restriction"), ("restriction", "no_straight_on")),
            )
        ],
        source=original.source,
        profile=original.profile,
    )
    path = tmp_path / "graph.json"
    path.write_bytes(canonical_bytes(graph))
    view = load_routing_view(
        path, hashlib.sha256(path.read_bytes()).hexdigest(), memory_budget_bytes=64 * 1024 * 1024
    )
    assert RoadRouter(view).shortest_path(1, 4) is None
    assert RoadRouter(view).shortest_path(2, 4) is not None
    assert view.turn_rules == graph.turn_rules


def test_streamed_view_refuses_missing_turn_edges_even_with_a_matching_file_hash(tmp_path):
    path = tmp_path / "graph.json"
    document = json.loads(canonical_bytes(roads()))
    document["turn_rules"] = [
        {"relation_id": 100, "kind": "only", "prefix_edge_ids": ["w999:0:1:f"], "next_edge_ids": []}
    ]
    path.write_text(json.dumps(document))
    with pytest.raises(TerminalQualificationError, match="absent directed edge"):
        load_routing_view(
            path,
            hashlib.sha256(path.read_bytes()).hexdigest(),
            memory_budget_bytes=64 * 1024 * 1024,
        )


def test_matrix_writer_failure_preserves_prior_artifact(tmp_path, monkeypatch):
    import tools.qualify_michigan_road_terminals as qualifier

    path = tmp_path / "matrix.json.gz"
    path.write_bytes(b"qualified previous matrix")

    def refuse(*_):
        raise OSError("publication fault")

    monkeypatch.setattr(qualifier.os, "replace", refuse)
    with pytest.raises(OSError, match="publication fault"):
        write_matrix({"schema": "fixture"}, path)
    assert path.read_bytes() == b"qualified previous matrix"
    assert list(tmp_path.iterdir()) == [path]


def test_cli_writes_exact_83_by_83_matrix_with_observed_atlas_and_tiny_graph(tmp_path, capsys):
    from tools.qualify_michigan_road_terminals import main

    graph_path = tmp_path / "graph.json"
    graph_path.write_bytes(canonical_bytes(roads()))
    defines = tmp_path / "defines.toml"
    defines.write_text(
        '[statewide]\nEVIDENCE_CLASS="Designed"\nTERMINAL_ATTACHMENT_LIMIT_METERS=50000\n[transport]\nTRUCK_GROSS_WEIGHT_KG=40000\nTRUCK_HEIGHT_MM=4000\nTRUCK_WIDTH_MM=2550\nTRUCK_LENGTH_MM=16500\nDEFAULT_MAXHEIGHT_MM=4000\n'
    )
    output = tmp_path / "matrix.json.gz"
    assert (
        main(
            [
                "--graph",
                str(graph_path),
                "--graph-sha256",
                hashlib.sha256(graph_path.read_bytes()).hexdigest(),
                "--defines",
                str(defines),
                "--memory-budget-mib",
                "64",
                "--output",
                str(output),
            ]
        )
        == 0
    )
    artifact = json.loads(gzip.decompress(output.read_bytes()))
    assert len(artifact["terminals"]) == 83
    assert len(artifact["paths"]) == 83 * 83
    assert (
        artifact["source_pins"]["defines_sha256"]
        == hashlib.sha256(defines.read_bytes()).hexdigest()
    )
    reports = [json.loads(line) for line in capsys.readouterr().out.splitlines()]
    preflight = next(row for row in reports if row["stage"] == "matrix_preflight")
    assert preflight["graph_nodes"] == len(roads().nodes)
    assert preflight["rss_bytes"] > 0
    assert reports[-1]["sha256"] == hashlib.sha256(output.read_bytes()).hexdigest()
