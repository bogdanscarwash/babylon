# Community frozen corroboration — world 1 (Task 7 Step 6)

**Date:** 2026-08-22 (the artifact keeps the plan's dated filename).
**Plan:** `docs/superpowers/plans/2026-08-18-community-port.md` §9 — "evidence, not the oracle".
**Driver:** `reports/community_frozen_corroboration_2026_08_18.py` (preserved beside this file; run
`uv run python reports/community_frozen_corroboration_2026_08_18.py`).

This artifact drives the **real** frozen `CommunitySystem.step()`
(`src/babylon/engine/systems/community.py`) over a hand-seeded `BabylonGraph` mirroring conformance
world 1 (`rust/crates/babylon-tick/content/scenarios/community-conformance.bscn`) node-for-node.
It is **evidence for the plan §1 archaeology, never the conformance oracle** — the oracle is the
mirror (`rust/crates/babylon-tick/content/scenarios/community_conformance.py`), which transcribes
the *rules*; this file records what the *frozen engine* actually does to the same world.

## The drift the driver had to shim (evidence item 1)

The frozen system could not be driven against the *current* topology unaided:
`community.py:418` calls `graph.query_edges(source_id=…, edge_type=…)`, and the current
`QueryMixin.query_edges` (`src/babylon/topology/adapters/query_mixin.py:70`) accepts only
`edge_type`/`predicate`/`min_weight`/`max_weight` — the source-scoped parameter is gone. This is
exactly the seam `src/babylon/sentinels/seam/registry.py:2171` rules `STRUCTURALLY_IMPOSSIBLE` in
production, now measured from the driver side. The script's `FrozenCompatGraph` subclass restores
the called signature by translating to the landed one (filter the type query by source); it touches
no frozen source and no topology source, and its necessity is itself the evidence.

## The C4 obligation — verdict: PASS

World 1's n5 (`inactive-member`) was seeded **inactive and holding a real NEW_AFRIKAN membership**.
Frozen's `_collect_memberships` (`community.py:472-474`) skips it, so `community_cost_modifier`
must be **absent** from its post-step attribute dict. The verbatim dict, post-`step()`:

```text
{'_node_type': 'social_class', 'active': False, 'community_memberships': [{'agent_id': 'inactive-member', 'community_type': 'new_afrikan'}]}
```

**`community_cost_modifier` is ABSENT.** The mirror models the same field as a missing key (never
`1.0`), and the Rust assertion in Tasks 10-11 reads the substrate's honest-null error. The plan's
§1.4/C4 reading of frozen `:472-474` is confirmed; `c09`/`c10`'s `active` guard stands.

## Frozen's outputs on world 1 (verbatim driver stdout)

```text
== post-step community states (frozen, real step()) ==
new_afrikan: r=0.864865 l=0.135135 f=0.0
  heat=0.475 cohesion=0.7275 education_pressure=0.225
settler: r=0.0 l=0.5 f=0.5
  heat=0.2375 cohesion=0.485 education_pressure=0.1125
queer: r=0.761905 l=0.238095 f=0.0
  heat=0.7124999999999999 cohesion=0.60625 education_pressure=0.45

== per-node community_cost_modifier (frozen writes) ==
na-worker: 1.09375
na-organizer: 0.875
settler-la: 1.0
unaffiliated: 1.0
```

## Named divergence found BY this artifact: the SnapToGrid boundary (evidence item 2)

Frozen's stored ternary is **not** the mirror's full-precision chain:

| community | mirror (the oracle) | frozen stores |
|---|---|---|
| new_afrikan r | `0.8648648648648649` | `0.864865` |
| new_afrikan l | `0.13513513513513511` | `0.135135` |
| queer r | `0.7619047619047619` | `0.761905` |
| queer l | `0.23809523809523808` | `0.238095` |

Cause, pinned to code: `TernaryConsciousness`'s fields are `Probability`
(`src/babylon/models/entities/consciousness.py:76-78`), and `Probability` carries the `SnapToGrid`
validator (`src/babylon/models/types.py:50-58`), which quantizes to a **10⁻⁶ grid, ROUND_HALF_UP**
(`src/babylon/kernel/math.py:41`, `_PRECISION = 6` at `:16`; the type's own docstring says 10⁻⁵ —
stale, the constant is 6). Frozen therefore stores the grid-snapped value at the model boundary;
the port's BSL lane stores raw f64. The mirror is deliberately the **unquantized** oracle — the
rules compute the same operation chain frozen computes *before* the boundary snaps it. Every other
value in world 1 (the decay triples, the cost modifiers) is already on-grid, which is why only the
two divided ternaries show the seam. The pack's D-row register records this at its landing; Tasks
9's assertions pin the **unquantized** values, and the divergence is stated in the pack header.

The decay triples and the four cost modifiers agree to the bit between frozen and the mirror
(`0.475`/`0.7275`/`0.225`, `0.2375`/`0.485`/`0.1125`, `0.7124999999999999`/`0.60625`/`0.45`;
`1.09375`, `0.875`, `1.0`, `1.0`).

## What this artifact does NOT show

- The blocked half (threat scoring, solidarity amplification, CORE_ORGANIZER infrastructure
  maintenance) ran in frozen — world 1 seeds no SOLIDARITY edges and no CORE_ORGANIZER roles, so
  those paths exercised trivially. They remain #653-gated in the port regardless.
- Bit-parity of the ternary **operation order**: frozen sums per-org (`org_landscape` order), the
  port sums per-class (D-NF+3). In world 1 the dyadic seed values make both orders agree; the
  divergence class is recorded in the plan, not reproduced here.
