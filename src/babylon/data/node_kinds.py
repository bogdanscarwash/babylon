"""Reference LODES destination categories."""

from __future__ import annotations

from enum import StrEnum


class NodeKind(StrEnum):
    """Identifier-space discriminator for boundary-register endpoints.

    HEX:      H3 res-7 index (15 chars).
    COUNTY:   5-digit FIPS county code.
    STATE:    2-digit FIPS state code.
    NATIONAL: "USA" (literal sentinel for the national aggregate).
    EXTERNAL: external_node.node_id (e.g., "canada", "china", "rest_of_usa").
    """

    HEX = "hex"
    COUNTY = "county"
    STATE = "state"
    NATIONAL = "national"
    EXTERNAL = "external"
    # P25 U12 (ADR139, L-RECEIPTS): the social wage's supply chain runs
    # through non-spatial endpoints — the disbursing sovereign and the
    # receiving/exploited class.
    SOVEREIGN = "sovereign"
    SOCIAL_CLASS = "social_class"


class BoundaryEdgeKind(StrEnum):
    """Classification of a boundary flow's economic role.

    TRADE_EDGE:       Bidirectional value flow (FAF tons + Ricci $-value).
                      Positive magnitude = study-area export.
    DRAIN_EDGE:       Directional periphery → core Φ (Hickel drain).
    COMMUTE_OUT:      Vol II worker exit (study-area boundary).
    COMMUTE_IN:       Vol II worker entry (study-area boundary).
    PHYSICAL_EXCHANGE: FAF freight or USGS minerals.
    """

    TRADE_EDGE = "trade_edge"
    DRAIN_EDGE = "drain_edge"
    COMMUTE_OUT = "commute_out"
    COMMUTE_IN = "commute_in"
    PHYSICAL_EXCHANGE = "physical_exchange"
    # P25 U12 (ADR139, L-RECEIPTS — "no flow without a row", §4): the three
    # hops of the social wage's supply chain that had no row vocabulary.
    # EXPLOITATION_FLOW: per-tick rent along an EXPLOITATION edge
    #                    (exploited class → exploiter class), the chain's source.
    # FISCAL_FUNDING:    the Φ slice an enactment actually consumed
    #                    (tribute pool → sovereign fisc).
    # SOCIAL_WAGE:       a delivered per-class social-wage unit
    #                    (sovereign → class).
    EXPLOITATION_FLOW = "exploitation_flow"
    FISCAL_FUNDING = "fiscal_funding"
    SOCIAL_WAGE = "social_wage"


__all__ = ["NodeKind", "BoundaryEdgeKind"]
