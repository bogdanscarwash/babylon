"""Schema-conformance test for spec 062 NodeKind and BoundaryEdgeKind.

Verifies T017 / T020: the enum values match the schema constants in
contracts/boundary_register.yaml.
"""

from __future__ import annotations

import pytest

from babylon.data.node_kinds import BoundaryEdgeKind, NodeKind

# These are the canonical schema constants from
# specs/062-cross-scale-integration/contracts/boundary_register.yaml.
EXPECTED_NODE_KINDS = {
    "hex",
    "county",
    "state",
    "national",
    "external",
    "sovereign",
    "social_class",
}
EXPECTED_EDGE_KINDS = {
    "trade_edge",
    "drain_edge",
    "commute_out",
    "commute_in",
    "physical_exchange",
    "exploitation_flow",
    "fiscal_funding",
    "social_wage",
}


@pytest.mark.cross_scale
class TestNodeKind:
    def test_values_match_schema(self) -> None:
        assert {k.value for k in NodeKind} == EXPECTED_NODE_KINDS

    def test_is_str_enum(self) -> None:
        assert NodeKind.HEX == "hex"
        assert NodeKind.COUNTY.value == "county"


@pytest.mark.cross_scale
class TestBoundaryEdgeKind:
    def test_values_match_schema(self) -> None:
        assert {k.value for k in BoundaryEdgeKind} == EXPECTED_EDGE_KINDS

    def test_is_str_enum(self) -> None:
        assert BoundaryEdgeKind.DRAIN_EDGE == "drain_edge"
        assert BoundaryEdgeKind.COMMUTE_OUT.value == "commute_out"
