#!/usr/bin/env python3
"""Community port train, Task 7 Step 6 — the frozen-corroboration driver.

Drives the REAL frozen `CommunitySystem.step()` (the Python reference
estate, `src/babylon/engine/systems/community.py`) over a hand-seeded
`BabylonGraph` mirroring conformance world 1, and prints the evidence
`reports/community-frozen-corroboration-2026-08-18.md` embeds verbatim.

LEGAL because `_extract_memberships_from_node` accepts plain dicts
(community.py:288-293) and `services.community_hypergraph` is a plain dict
(:296-306) — the frozen path that `sentinels/seam/registry.py:2171` rules
STRUCTURALLY_IMPOSSIBLE in production is runnable from a script. This is
EVIDENCE for the plan §1 archaeology, NEVER the conformance oracle (the
oracle is `content/scenarios/community_conformance.py`, the mirror).

THE C4 OBLIGATION: world 1's n5 (`inactive-member`) is seeded here
INACTIVE and holding a real NEW_AFRIKAN membership; frozen's
`_collect_memberships` (community.py:472-474) must skip it, so
`community_cost_modifier` must be ABSENT from its post-step attribute dict
— printed verbatim below. If it is present, c09/c10's `active` guard is
wrong and Task 10 STOPs.

Run: `uv run python reports/community_frozen_corroboration_2026_08_18.py`
"""

from __future__ import annotations

from babylon.engine.services import ServiceContainer
from babylon.engine.systems.community import CommunitySystem
from babylon.models.entities.community import CommunityState
from babylon.models.entities.consciousness import TernaryConsciousness
from babylon.models.enums import CommunityType
from babylon.models.graph import GraphEdge
from babylon.topology.graph import BabylonGraph


class FrozenCompatGraph(BabylonGraph):
    """The 2026-08-22 graph-protocol drift, made concrete as a shim.

    The frozen `community.py` calls `query_edges(source_id=…, edge_type=…)`
    (community.py:418) — the CURRENT `QueryMixin.query_edges`
    (topology/adapters/query_mixin.py:70) accepts only
    `edge_type`/`predicate`/`min_weight`/`max_weight`; the source-scoped
    parameter is gone. That is precisely why
    `sentinels/seam/registry.py:2171` rules the frozen path
    STRUCTURALLY_IMPOSSIBLE in production — the engine's own seam no longer
    admits it. This subclass restores the called signature by translating
    to the landed one (filter the type query by source). It changes NO
    frozen source and NO topology source — it is script scaffolding, and
    its necessity is itself part of the evidence the report records.
    """

    def query_edges(  # type: ignore[override]
        self,
        edge_type=None,
        source_id=None,
        **kwargs,
    ):
        edges = super().query_edges(edge_type=edge_type, **kwargs)
        if source_id is None:
            yield from edges
            return
        for edge in edges:
            if isinstance(edge, GraphEdge) and edge.source_id == source_id:
                yield edge


# ---- The world, hand-seeded node-for-node against the .bscn ----

graph = FrozenCompatGraph()

# Social classes: (name, active, memberships as plain dicts — the :288-293
# lane). Membership community_type is the StrEnum's VALUE.
graph.add_node("na-worker", _node_type="social_class", active=True)
graph.add_node("na-organizer", _node_type="social_class", active=True)
graph.add_node("settler-la", _node_type="social_class", active=True)
graph.add_node("unaffiliated", _node_type="social_class", active=True)
graph.add_node("inactive-member", _node_type="social_class", active=False)

graph.nodes["na-worker"]["community_memberships"] = [
    {"agent_id": "na-worker", "community_type": "new_afrikan"},
    {"agent_id": "na-worker", "community_type": "queer"},
]
graph.nodes["na-organizer"]["community_memberships"] = [
    {"agent_id": "na-organizer", "community_type": "new_afrikan"},
]
graph.nodes["settler-la"]["community_memberships"] = [
    {"agent_id": "settler-la", "community_type": "settler"},
]
graph.nodes["unaffiliated"]["community_memberships"] = []
# C4: the membership is REAL; the node is inactive.
graph.nodes["inactive-member"]["community_memberships"] = [
    {"agent_id": "inactive-member", "community_type": "new_afrikan"},
]

# Organizations: tendency as the StrEnum value, cadre_level, cohesion.
graph.add_node(
    "rev-org",
    _node_type="organization",
    consciousness_tendency="revolutionary",
    cadre_level=0.5,
    cohesion=0.8,
)
graph.add_node(
    "lib-org",
    _node_type="organization",
    consciousness_tendency="liberal",
    cadre_level=0.25,
    cohesion=0.5,
)
graph.add_node(
    "fash-org",
    _node_type="organization",
    consciousness_tendency="fascist",
    cadre_level=0.5,
    cohesion=0.25,
)
graph.add_node(
    "no-member-org",
    _node_type="organization",
    consciousness_tendency="revolutionary",
    cadre_level=1.0,
    cohesion=1.0,
)

for src, tgt in [
    ("rev-org", "na-worker"),
    ("rev-org", "na-organizer"),
    ("lib-org", "na-worker"),
    ("lib-org", "settler-la"),
    ("fash-org", "settler-la"),
]:
    graph.add_edge(src, tgt, _edge_type="membership")

# Community states, world-1 seeds (the prior-tick ternary included).
community_states = {
    CommunityType.NEW_AFRIKAN: CommunityState(
        community_type=CommunityType.NEW_AFRIKAN,
        heat=0.5,  # type: ignore[arg-type]
        cohesion=0.75,  # type: ignore[arg-type]
        education_pressure=0.25,  # type: ignore[arg-type]
        reproduction_cost_modifier=0.875,
        consciousness=TernaryConsciousness(r=0.5, l=0.25, f=0.25),  # noqa: E741
    ),
    CommunityType.SETTLER: CommunityState(
        community_type=CommunityType.SETTLER,
        heat=0.25,  # type: ignore[arg-type]
        cohesion=0.5,  # type: ignore[arg-type]
        education_pressure=0.125,  # type: ignore[arg-type]
        reproduction_cost_modifier=1.0,
        consciousness=TernaryConsciousness(r=0.0, l=0.75, f=0.25),  # noqa: E741
    ),
    CommunityType.QUEER: CommunityState(
        community_type=CommunityType.QUEER,
        heat=0.75,  # type: ignore[arg-type]
        cohesion=0.625,  # type: ignore[arg-type]
        education_pressure=0.5,  # type: ignore[arg-type]
        reproduction_cost_modifier=1.25,
        consciousness=TernaryConsciousness(r=0.25, l=0.5, f=0.25),  # noqa: E741
    ),
}

services = ServiceContainer.create(
    community_hypergraph={"community_states": community_states},
)

# ---- Drive the real system ----
CommunitySystem().step(graph, services, None)

# ---- The evidence ----
out = services.community_hypergraph["community_states"]
print("== post-step community states (frozen, real step()) ==")
for ct in (CommunityType.NEW_AFRIKAN, CommunityType.SETTLER, CommunityType.QUEER):
    s = out[ct]
    print(
        f"{ct.value}: r={float(s.consciousness.r)!r} "
        f"l={float(s.consciousness.l)!r} f={float(s.consciousness.f)!r}"
    )
    print(
        f"  heat={float(s.heat)!r} cohesion={float(s.cohesion)!r} "
        f"education_pressure={float(s.education_pressure)!r}"
    )

print()
print("== per-node community_cost_modifier (frozen writes) ==")
for node_id in ("na-worker", "na-organizer", "settler-la", "unaffiliated"):
    attrs = graph.nodes[node_id]
    print(f"{node_id}: {attrs.get('community_cost_modifier', 'ABSENT')!r}")

print()
print("== C4: inactive-member's post-step attribute dict, verbatim ==")
print(repr(dict(graph.nodes["inactive-member"])))
print()
if "community_cost_modifier" in graph.nodes["inactive-member"]:
    print("C4 VERDICT: FAIL — the field is PRESENT; c09/c10's active guard is wrong.")
else:
    print("C4 VERDICT: PASS — community_cost_modifier is ABSENT (frozen skips the inactive).")
