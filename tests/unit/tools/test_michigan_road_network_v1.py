"""Physical road topology and directed turn constraints survive reference compilation."""

import json
from dataclasses import replace
from pathlib import Path

import pytest
from tools.michigan_road_network_v1 import (
    Node,
    Restriction,
    RestrictionMember,
    RoadNetworkError,
    RoadRouter,
    SourceIdentity,
    VehicleProfile,
    Way,
    build_graph,
    canonical_bytes,
)

SOURCE = SourceIdentity(
    "a" * 64,
    1,
    "https://example.invalid/dated.osm.pbf",
    "2026-09-08T20:21:01Z",
    "b" * 64,
    0,
    "fixture",
    "fixture",
)
PROFILE = VehicleProfile(18000, 4000, 2550, 16500, 4000, 10000)


def node(ident, x, y=0, **tags):
    return Node(ident, -850000000 + x * 10000, 440000000 + y * 10000, tuple(sorted(tags.items())))


def way(ident, *nodes, **tags):
    return Way(ident, tuple(nodes), tuple(sorted({"highway": "primary", **tags}.items())))


def restriction(ident, source, via, target, value, **tags):
    return Restriction(
        ident,
        (
            RestrictionMember("w", source, "from"),
            *(RestrictionMember(kind, ref, "via") for kind, ref in via),
            RestrictionMember("w", target, "to"),
        ),
        tuple(sorted({"type": "restriction", "restriction": value, **tags}.items())),
    )


def graph(nodes, ways, restrictions=()):
    return build_graph(nodes, ways, restrictions, profile=PROFILE, source=SOURCE)


def route(roads, start, destination):
    return RoadRouter(roads).shortest_path(start, destination)


def test_crossing_lines_without_shared_osm_node_never_connect():
    roads = graph(
        [node(1, 0), node(2, 2), node(3, 1, -1), node(4, 1, 1)],
        [way(10, 1, 2, bridge="yes", layer="1"), way(20, 3, 4, tunnel="yes", layer="-1")],
    )
    assert route(roads, 1, 4) is None
    assert dict(roads.edges[0].tags)["bridge"] == "yes"
    assert route(roads, 1, 2) is not None


def test_oneway_minus_one_and_hgv_access_precedence():
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2)],
        [way(10, 1, 2, oneway="-1", access="no", hgv="yes"), way(20, 2, 3, hgv="no")],
    )
    assert route(roads, 1, 2) is None
    assert route(roads, 2, 1) is not None
    assert not any(edge.way_id == 20 for edge in roads.edges)
    assert any(item.code == "access_excluded" for item in roads.diagnostics)


def test_no_turn_routes_around_restriction_instead_of_treating_junction_as_disconnected():
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2), node(4, 1, 1)],
        [way(10, 1, 2, oneway="yes"), way(20, 2, 3, oneway="yes"), way(30, 2, 4, 3, oneway="yes")],
        [restriction(100, 10, (("n", 2),), 20, "no_straight_on")],
    )
    result = route(roads, 1, 3)
    assert result is not None
    assert [roads.edge_by_id[edge].way_id for edge in result.edge_ids] == [10, 30]
    assert route(roads, 2, 3).edge_ids == (
        next(edge.id for edge in roads.edges if edge.way_id == 20),
    )


@pytest.mark.parametrize("value,allowed", [("no_straight_on", False), ("only_straight_on", True)])
def test_via_way_restriction_remembers_entry_and_constrains_early_exit(value, allowed):
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2), node(4, 3), node(5, 4), node(6, 2, 1)],
        [
            way(10, 1, 2, oneway="yes"),
            way(20, 2, 3, 4, oneway="yes"),
            way(30, 4, 5, oneway="yes"),
            way(40, 3, 6, oneway="yes"),
        ],
        [restriction(100, 10, (("w", 20),), 30, value)],
    )
    assert (route(roads, 1, 5) is not None) is allowed
    assert (route(roads, 1, 6) is not None) is not allowed
    assert route(roads, 2, 5) is not None


def test_unsupported_conditional_rule_fails_closed_with_specific_diagnostic():
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2)],
        [way(10, 1, 2), way(20, 2, 3)],
        [
            restriction(
                100,
                10,
                (("n", 2),),
                20,
                "no_straight_on",
                **{"restriction:conditional": "no_straight_on @ (Mo-Fr)"},
            )
        ],
    )
    assert not any(edge.way_id == 10 for edge in roads.edges)
    assert any(
        item.code == "unsupported_restriction" and item.osm_id == 100 for item in roads.diagnostics
    )


def test_missing_node_reference_refuses_instead_of_shortening_a_road():
    with pytest.raises(RoadNetworkError, match="way 10 references missing node 2"):
        graph([node(1, 0)], [way(10, 1, 2)])


def test_input_permutations_keep_bytes_and_equal_cost_route_identity():
    nodes = [node(1, 0), node(2, 1, 1), node(3, 1, -1), node(4, 2)]
    ways = [way(20, 1, 2, 4, oneway="yes"), way(10, 1, 3, 4, oneway="yes")]
    first = graph(nodes, ways)
    first = replace(first, edges=tuple(replace(edge, distance_mm=100) for edge in first.edges))
    permuted = graph(list(reversed(nodes)), list(reversed(ways)))
    permuted = replace(
        permuted, edges=tuple(replace(edge, distance_mm=100) for edge in permuted.edges)
    )
    assert canonical_bytes(first) == canonical_bytes(permuted)
    result = route(first, 1, 4)
    assert result == route(permuted, 1, 4)
    assert result is not None and first.edge_by_id[result.edge_ids[0]].way_id == 10


def test_degree_two_geometry_is_retained_without_inventing_factory_precision():
    roads = graph([node(1, 0), node(2, 1), node(3, 2)], [way(10, 1, 2, 3)])
    assert len(roads.edges) == 2
    assert len(roads.edges[0].shape_e7) == 3
    assert all(edge.distance_mm > 0 for edge in roads.edges)


def test_no_u_turn_on_the_same_way_preserves_straight_travel():
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2)],
        [way(10, 1, 2, 3)],
        [restriction(100, 10, (("n", 2),), 10, "no_u_turn")],
    )
    result = route(roads, 1, 3)
    assert result is not None and len(result.edge_ids) == 2
    for rule in roads.turn_rules:
        incoming = roads.edge_by_id[rule.prefix_edge_ids[0]]
        assert all(
            roads.edge_by_id[ident].to_node == incoming.from_node for ident in rule.next_edge_ids
        )


def test_conflicting_no_and_only_rules_cannot_open_a_turn():
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2), node(4, 1, 1)],
        [way(10, 1, 2, oneway="yes"), way(20, 2, 3, oneway="yes"), way(30, 2, 4, oneway="yes")],
        [
            restriction(100, 10, (("n", 2),), 20, "only_straight_on"),
            restriction(101, 10, (("n", 2),), 20, "no_straight_on"),
        ],
    )
    assert route(roads, 1, 3) is None
    assert route(roads, 1, 4) is None


def test_roundabout_direction_and_explicit_vehicle_limits():
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2), node(4, 3)],
        [
            way(10, 1, 2, junction="roundabout", maxweight="18 t"),
            way(20, 2, 3, maxweight="17999 kg"),
            way(30, 3, 4, **{"maxheight:conditional": "3 @ (Mo-Fr)"}),
        ],
    )
    assert route(roads, 1, 2) is not None
    assert route(roads, 2, 1) is None
    assert {edge.way_id for edge in roads.edges} == {10}
    assert {
        item.osm_id for item in roads.diagnostics if item.code == "vehicle_restriction_excluded"
    } == {20, 30}


def test_hgv_turn_exception_and_closed_barrier_are_explicit():
    # Use the literal source key rather than the Python keyword spelling.
    exception = restriction(100, 10, (("n", 2),), 20, "no_straight_on", **{"except": "hgv"})
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2), node(4, 3, barrier="bollard")],
        [way(10, 1, 2, oneway="yes"), way(20, 2, 3, oneway="yes"), way(30, 3, 4)],
        [exception],
    )
    assert route(roads, 1, 3) is not None
    assert route(roads, 3, 4) is None
    assert any(
        item.code == "barrier_or_access_excluded" and item.osm_id == 4 for item in roads.diagnostics
    )


def test_multi_destination_search_matches_individual_paths_with_turn_history():
    roads = graph(
        [node(1, 0), node(2, 1), node(3, 2), node(4, 1, 1)],
        [way(10, 1, 2, oneway="yes"), way(20, 2, 3, oneway="yes"), way(30, 2, 4, 3, oneway="yes")],
        [restriction(100, 10, (("n", 2),), 20, "no_straight_on")],
    )
    router = RoadRouter(roads)
    results = router.shortest_paths(1, [3, 2, 1, 999])
    assert results == {
        destination: router.shortest_path(1, destination) for destination in [1, 2, 3, 999]
    }
    assert results[3] is not None and len(results[3].edge_ids) == 2


def test_zero_distance_source_segment_is_excluded_without_inventing_a_cost():
    roads = graph([node(1, 0), node(2, 0), node(3, 1)], [way(10, 1, 2), way(20, 2, 3)])
    assert {edge.way_id for edge in roads.edges} == {20}
    assert any(
        item.code == "invalid_distance_excluded" and item.osm_id == 10 for item in roads.diagnostics
    )


@pytest.mark.parametrize(
    "limit,height,allowed",
    [
        ("14'0\"", 4267, True),
        ("14'0\"", 4268, False),
        ("13'1.5\"", 4000, True),
        ("13'1.5\"", 4001, False),
        ("13'1.480314960629921259842519685039\"", 4000, False),
        ("3.999999999999999999999999999999 m", 4000, False),
        (" 14' 0\" ", 4000, True),
    ],
)
def test_dimensional_height_never_rounds_up_to_admit_a_taller_vehicle(limit, height, allowed):
    roads = build_graph(
        [node(1, 0), node(2, 1)],
        [way(10, 1, 2, maxheight=limit)],
        [],
        profile=replace(PROFILE, height_mm=height),
        source=SOURCE,
    )
    assert (route(roads, 1, 2) is not None) is allowed


@pytest.mark.parametrize(
    "tags",
    [
        {"maxheight": "14'12\""},
        {"maxheight": "14'0"},
        {"maxheight": "14.5'0\""},
        {"maxheight": "14'-1\""},
        {"maxheight": "14'NaN\""},
        {"maxheight": "14'1e1\""},
        {"maxheight": "14'0\"", "maxheight:conditional": "14'0\" @ (Mo-Fr)"},
        {"maxweight": "14'0\""},
    ],
)
def test_malformed_conditional_or_mass_feet_inches_limits_stay_closed(tags):
    roads = graph([node(1, 0), node(2, 1)], [way(10, 1, 2, **tags)])
    assert route(roads, 1, 2) is None
    assert any(item.code == "vehicle_restriction_excluded" for item in roads.diagnostics)


@pytest.mark.parametrize(
    "tags,allowed",
    [
        ({"access": "yes"}, True),
        ({"access": "yes", "hgv": "no"}, False),
        ({"access": "yes", "motor_vehicle": "no"}, False),
        ({"access": "yes", "vehicle": "no"}, False),
        ({"access": "no", "hgv": "yes"}, True),
        ({"access": "private"}, False),
        ({}, False),
        ({"access": "yes", "hgv:conditional": "yes @ (Mo-Fr)"}, False),
        ({"access": "yes", "barrier": "bollard"}, False),
    ],
)
def test_lift_gate_requires_explicit_permission_and_preserves_vehicle_overrides(tags, allowed):
    roads = graph(
        [node(1, 0), node(2, 1, **{"barrier": "lift_gate", **tags}), node(3, 2)],
        [way(10, 1, 2, 3, hgv="designated")],
    )
    assert (route(roads, 1, 3) is not None) is allowed


@pytest.mark.parametrize("height,allowed", [(4000, True), (4268, False)])
def test_pinned_houghton_hancock_bridge_preserves_both_real_directed_traversals(height, allowed):
    fixture = json.loads(
        (
            Path(__file__).parents[2] / "fixtures/michigan_houghton_hancock_bridge_v1.json"
        ).read_bytes()
    )
    nodes = [
        Node(**{**row, "tags": tuple(sorted(row["tags"].items()))}) for row in fixture["nodes"]
    ]
    ways = [
        Way(
            **{
                **row,
                "node_ids": tuple(row["node_ids"]),
                "tags": tuple(sorted(row["tags"].items())),
            }
        )
        for row in fixture["ways"]
    ]
    roads = build_graph(
        nodes,
        ways,
        [],
        source=SourceIdentity(**fixture["source"]),
        profile=replace(VehicleProfile(**fixture["profile"]), height_mm=height),
    )
    traversals = (
        (184563120, 2304478679, (221552319, 348028891, 17812636)),
        (2305812327, 2305812317, (221552320, 221424745, 348028892, 1234013533, 221552316)),
    )
    for start, end, source_way_ids in traversals:
        result = route(roads, start, end)
        assert (result is not None) is allowed
        if result is not None:
            assert (
                tuple(roads.edge_by_id[edge].way_id for edge in result.edge_ids) == source_way_ids
            )
        assert route(roads, end, start) is None  # Each actual bridge carriageway is one-way.
    if allowed:
        assert not roads.diagnostics
        assert all(
            dict(row.tags)["maxheight"] == "14'0\""
            for row in roads.edges
            if row.way_id in {221424745, 348028891}
        )


@pytest.mark.parametrize("weight,allowed", [(40000, True), (65317, True), (65318, False)])
def test_pinned_mackinac_bridge_honors_short_tons_in_both_directions(weight, allowed):
    fixture = json.loads(
        (Path(__file__).parents[2] / "fixtures/michigan_mackinac_bridge_v1.json").read_bytes()
    )
    roads = build_graph(
        [Node(**{**row, "tags": tuple(sorted(row["tags"].items()))}) for row in fixture["nodes"]],
        [
            Way(
                **{
                    **row,
                    "node_ids": tuple(row["node_ids"]),
                    "tags": tuple(sorted(row["tags"].items())),
                }
            )
            for row in fixture["ways"]
        ],
        [],
        source=SourceIdentity(**fixture["source"]),
        profile=replace(VehicleProfile(**fixture["profile"]), gross_weight_kg=weight),
    )
    # Original separate one-way carriageways, with a 72-short-ton limit.
    for start, end in [(184072649, 456044401), (185090726, 184334480)]:
        path = route(roads, start, end)
        assert (path is not None) is allowed
        assert route(roads, end, start) is None
        if path is not None:
            assert 2_000_000 < path.distance_mm < 10_000_000
            assert all(
                dict(roads.edge_by_id[key].tags)["maxweight"] == "72 st" for key in path.edge_ids
            )


@pytest.mark.parametrize("height,allowed", [(4000, True), (4001, False)])
def test_pinned_mackinac_crossing_includes_default_height_toll_plaza(height, allowed):
    fixture = json.loads(
        (Path(__file__).parents[2] / "fixtures/michigan_mackinac_bridge_v1.json").read_bytes()
    )
    roads = build_graph(
        [Node(**{**row, "tags": tuple(sorted(row["tags"].items()))}) for row in fixture["nodes"]],
        [
            Way(
                **{
                    **row,
                    "node_ids": tuple(row["node_ids"]),
                    "tags": tuple(sorted(row["tags"].items())),
                }
            )
            for row in fixture["ways"]
        ],
        [],
        source=SourceIdentity(**fixture["source"]),
        profile=replace(VehicleProfile(**fixture["profile"]), height_mm=height),
    )
    for start, end, toll_way in [
        (184072649, 4115216830, 525615140),
        (185136944, 184334480, 525615141),
    ]:
        path = route(roads, start, end)
        assert (path is not None) is allowed
        assert route(roads, end, start) is None
        if path is not None:
            assert toll_way in {roads.edge_by_id[key].way_id for key in path.edge_ids}
            assert 5_000_000 < path.distance_mm < 10_000_000


@pytest.mark.parametrize(
    "tags,allowed",
    [
        ({"maxheight": "default"}, True),
        ({"maxheight": "default", "maxheight:physical": "3.9"}, False),
        ({"maxheight": "default", "hgv": "no"}, False),
        ({"maxheight": "default", "maxheight:conditional": "3 @ wet"}, False),
        ({"maxheight": "below_default"}, False),
        ({"maxheight": "no_indications"}, False),
        ({"maxheight": "none"}, False),
        ({"maxweight": "default"}, False),
    ],
)
def test_default_height_preserves_other_restrictions(tags, allowed):
    roads = graph([node(1, 0), node(2, 1)], [way(10, 1, 2, **tags)])
    assert (route(roads, 1, 2) is not None) is allowed
