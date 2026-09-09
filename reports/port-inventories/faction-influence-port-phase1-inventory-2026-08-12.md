# FactionInfluenceSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `FactionInfluenceSystem` (@14.5, Consequences phase, 256 lines) is
structurally unlike every system this template has covered so far: it writes **zero** graph
attributes — its entire product is (a) four `EventType` emissions and (b) two
`context.persistent_data` dict handoffs consumed same-tick by `CollapseTransitionSystem`
(@20.5). Its core computation (`winning_faction_for_territory`) is an argmax over a per-edge
scalar (`INFLUENCES.influence_level`) that today's BSL substrate cannot read back at all — not
even the substrate's own built-in edge weight is exposed to any query head — and the argmax's
*result* (a Faction reference) has no BSL field type to be stored in even if it could be
computed. One computation (`RED_SETTLER_TRAP_DETECTED`) is portable now; the rest are blocked,
one of them (the contiguous-region BFS) by a control-flow shape no BSL slice — landed, drafted,
or roadmapped — addresses at all. The whole system is also dormant on every canonical
`qa:regression` scenario (no FACTION/SOVEREIGN nodes are seeded there), so no conformance
oracle exists today regardless. **Verdict: BLOCKED — this is a deeper and more varied blocker
set than Territory's, not a subset of it; only the trap diagnostic is portable, and it alone
is not enough content to justify a pack.**

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/faction_influence.py` | 256 | **The target.** `FactionInfluenceSystem`, all five sub-computations. No `update_node`/`update_edge`/`add_node`/`add_edge` calls anywhere in the file (grep-confirmed) — the system writes **no graph state at all**. |
| `src/babylon/formulas/balkanization.py` | 404 | Pure formulas. **Called by this system:** `winning_faction_for_territory` (156-212), `detect_red_settler_trap` (215-252), `contiguous_influence_majority_subregion` (255-296) + its private helpers `_eligible_territories` (299-312), `_faction_influence_in` (315-322), `_largest_contiguous_component` (325-337), `_bfs_component` (340-363). **NOT called by this system** (consumed by `SovereigntySystem`/`Sovereign.metabolic_impact` instead): `calculate_metabolic_impact` (31-74), `derive_extraction_policy_from_stance` (77-102), `derive_default_multipliers_from_stance` (105-153), `extrapolate_habitability` (366-393) — out of scope for this port. |
| `src/babylon/config/defines/balkanization.py` | 176 | `BalkanizationDefines` Pydantic model. Only 5 of its ~20 fields feed this system: `faction_victory_supermajority_threshold`, `secession_influence_threshold`, `secession_hysteresis_ticks`, `min_contiguous_hex_count`, `red_settler_trap_class_reduction_threshold`. |
| `src/babylon/data/defines.yaml` | balkanization block 317-350 | Player-editable values; verified identical to the Pydantic defaults (line-by-line, §2e below). |
| `src/babylon/models/enums/balkanization.py` | 185 | `ColonialStance` (33-51, **RESERVED-LINE**, see below), `SupportType` (139-159, declared but unused by this system's own formulas — see §3). |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.TERRITORY`/`FACTION`/`SOVEREIGN` (61-67), `EdgeType.ADJACENCY`/`CLAIMS`/`INFLUENCES` (107,125-126). |
| `src/babylon/models/enums/events.py` | 234 | The 4 `EventType` members this system emits (149-154). |
| `src/babylon/models/entities/balkanization_faction.py` | 86 | `BalkanizationFaction` Pydantic entity — field types/domains for `colonial_stance`/`class_reduction`. Validated only at seed-construction time, never mid-tick (§3). |
| `src/babylon/models/entities/sovereign.py` | 118 | `Sovereign` Pydantic entity — `ruling_faction_id` field type/domain. |
| `src/babylon/models/entities/relationship.py` | 186 | `Relationship` — the spec-070 edge payload fields `influence_level`/`support_type`/`control_level`/`legal_status` (124-143), all `Optional[float|str]`, `exclude_none=True` at serialization. |
| `src/babylon/models/events/balkanization_payloads.py` | 134 | Typed Pydantic event-payload schemas for the 4 emitted `EventType`s (40-90). **Not constructed by this system** — `faction_influence.py` publishes raw dict-payload `Event` objects (kernel/event_bus.Event); these schemas are the downstream contract. |
| `src/babylon/engine/event_builders.py` | 782 | `EVENT_BUILDERS` bus→pydantic registry (248-277) — converts this system's raw dict payloads into the typed schemas above for replay/observability. Downstream of this system's own computation, ADR174 Python-glue boundary — out of scope for the port itself, noted for cross-system-channel completeness. |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase.resolve_rng` (35-55) — this system's only `SystemBase` helper use. **Not used:** `_write_clamped`/`_read` (this system reads via raw `node.attributes.get(...)`, never `SystemBase._read`, and writes nothing, so `_write_clamped` never fires — a structural contrast with Territory). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol` — the three bespoke accessor signatures this system calls: `query_faction_influence_by_territory` (379-395), `query_sovereign_claims` (397-413), `query_adjacent_territories` (434-448); plus one it does **not** call but that exists for the identical shape it needs: `query_contiguous_component_under_predicate` (473-494, see §6 Row 7). |
| `src/babylon/topology/graph.py` | 1033 | Concrete `BabylonGraph`. The three accessor implementations (902-972); `add_node`/`add_edge` (165-181 and neighboring) are plain-kwargs dict merges — **no Pydantic validation at write time**, same runtime-storage caveat as every other system's inventory. |
| `src/babylon/kernel/tick_partition.py` | — | `TickPartition.CONSEQUENCE` (18-30). |
| `src/babylon/engine/context.py` | 113 | `TickContext.persistent_data`/`.tick` — the two off-graph state channels this system reads and writes (§5). |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` (328-363) — confirms tick position and neighbors. |
| `src/babylon/engine/scenarios/balkanization_seed.py` | 163 | The **only** production path that seeds FACTION/SOVEREIGN/INFLUENCES/CLAIMS. Its own docstring (29-31): "Byte-safety: NOT applied by any of the six qa:regression scenario factories — FACTION/SOVEREIGN nodes land only in the electoral/balkanization scenarios." Direct confirmation of §5's dormancy finding. |
| `src/babylon/data/game/balkanization/__init__.py` + `seed_factions.json`/`seed_sovereigns.json` | 146 + data | The 4 canonical seed factions' `colonial_stance`/`class_reduction` values — **RESERVED-LINE** National Question content (§3, §6 Row 4). |
| `src/babylon/engine/systems/collapse_transition.py` | 313 | The **sole** downstream reader of this system's `persistent_data` writes (`winning_faction_by_territory` at line 71, `secession_eligible` at line 93) — same tick, @20.5. |
| `src/babylon/models/world_state.py` | 1161 (relevant 357-393) | `_reconstruct_relationships` — confirms `influence_level`/`support_type`/`control_level`/`legal_status` all four survive the graph round-trip (`from_graph()`), unlike some of Territory's fields. |
| `tools/regression_scenarios.py` | 2925 (relevant 2761-2768) | `COVERAGE_GAPS_DATA` — the canonical dormancy declaration for this exact system (§5). |
| `reports/bsl-gap-analysis-2026-08-10.md` | 863 | R9 prior-art gap analysis. Names FactionInfluence in **five** of its eighteen numbered gaps (Q1, Q2, Q3, Q4, Q5 — more gap-tags than almost any other system in the 34-system survey), plus the RNG-intrinsic note (615) and the Class-B topology-absent dormancy classification (649-650). Read in full for this inventory. |
| `docs/reference/bsl-language.rst` | 6104 (relevant regions cited inline) | The normative BSL spec. §2.10 (`field-of`/`EdgeRef`, design only — not landed for edges), §3.6 (Q6 graph-scope carrier-node ruling, design only, itself depends on the unserved `the` query head). |

**Rust verification (current dev tree, verified myself, not taken from the report above):**

| File | Role |
|---|---|
| `rust/crates/babylon-bsl/src/evaluator.rs` | `eval_field_of`/`field_of_node` (1185-1218): `field-of` over an `EdgeRef` is **unreachable today** — "no expression form produces one yet." `UNSERVED_EXPRESSION_HEADS` (503-512) lists `edges`/`edge-between`/`the` as Slice 2. `SERVED_QUERY_HEADS` (527) is exactly `["nodes", "neighbors"]`. |
| `rust/crates/babylon-bsl/src/query.rs` | `materialize_neighbors` (155-214): returns `Element::Node(id)` only — **discards the traversed edge's weight entirely** ("neighbors is a set, not a multiset," a design comment confirmed at the call site). |
| `rust/crates/babylon-bsl/src/structural_verbs.rs` | Dispatch table (344-386): `update-edge`/`update-hyperedge` **refuse unconditionally** — "GraphSubstrate keys an edge to one f64 strength and gives a hyperedge no attributes at all... a declared Phase-2/substrate decision." `add_edge` (882-931) accepts exactly one `:strength` operand and **refuses any further `<field-init>`** (902-908): "an edge's state is one f64 strength keyed by (type, from, to)." |
| `rust/crates/babylon-bsl/src/declarations.rs` | `parse_type_name` (646-675): the exhaustive `deffield` type vocabulary — `int`/`bool`/`currency`/`probability`/`intensity`/`coefficient`/`enum` — **no reference type**. |

---

## 2. COMPUTATION CATALOG (execution order, `faction_influence.py:58-81`)

### 2.1 — Winning-faction resolution (`_resolve_winning_factions`, faction_influence.py:83-99; `winning_faction_for_territory`, balkanization.py:156-212)

- **(a)** For every Territory, sum each Faction's `INFLUENCES.influence_level` incident on it; the Faction with the highest total wins; ties break to the incumbent (if tied), else a seeded RNG draw over the sorted tied-ID set.
- **(b)** `totals[faction_id] = totals.get(faction_id, 0.0) + float(influence_level)` (balkanization.py:199); `max_total = max(totals.values())` (204); `tied = sorted(fid for fid, total in totals.items() if (max_total - total) <= 1e-12)` (206) — an epsilon-tolerance tie test; if `len(tied) == 1` that faction wins outright (207-208); else incumbent-in-tied wins (209-210); else `rng.choice(tied)` (212).
- **(c) Reads:** `TERRITORY` node IDs only, via `query_nodes(node_type=TERRITORY)` — **no attribute read** on Territory itself (faction_influence.py:91-93). `INFLUENCES` edges incident on each Territory via `query_faction_influence_by_territory` (returns `(faction_id, influence_level, support_type)` rows, pre-sorted influence-desc/id-asc, graph.py:902-921) — only `row[0]` and `row[1]` are used by the formula; `support_type` (`row[2]`) is read into the tuple but **never used by this computation** (verified: no reference to a third tuple element anywhere in `winning_faction_for_territory`). `persistent["balkanization.previous_winning_faction_by_territory"]` for the incumbent tiebreak (faction_influence.py:89, 95).
- **(d) Writes:** `context.persistent_data["balkanization.winning_faction_by_territory"]` (faction_influence.py:71) and `["balkanization.previous_winning_faction_by_territory"]` (74) — **both off-graph**, not a single `update_node`/`update_edge` call anywhere.
- **(e) Defines:** none directly (the epsilon `1e-12` and the RNG salt `0xBA1AC1A` are hardcoded, not `defines`-sourced — see §4).
- **(f) Events:** none directly (drives 2.2-2.5).

### 2.2 — TERRITORY_TRANSITION emission on flip (`_emit_territory_transitions`, faction_influence.py:101-127)

- **(a)** For every Territory whose winner changed since the previous tick, emit a transition event.
- **(b)** `if old == new: continue` (112-113); else publish `Event(type=TERRITORY_TRANSITION, payload={territory_id, from_sovereign_id: None, to_sovereign_id: None, from_winning_faction_id: old, to_winning_faction_id: new, reason: "influence_flip"})` (114-127). **Note:** `from_sovereign_id`/`to_sovereign_id` are hardcoded `None` from this system every time — `CollapseTransitionSystem` (collapse_transition.py:181-197) is the only other emitter of this same `EventType`, and only it ever populates the sovereign-id fields (reasons `"collapse_partition"`/`"fracture"`/`"conquest"`). This is a shared event type across two systems with disjoint payload-field responsibility, verbatim as found.
- **(c) Reads:** `persistent["balkanization.previous_winning_faction_by_territory"]` (108), the `winning` dict from 2.1.
- **(d) Writes:** event bus only.
- **(e) Defines:** none.
- **(f) Events:** `EventType.TERRITORY_TRANSITION` (faction_influence.py:116).

### 2.3 — FACTION_VICTORY supermajority emission (`_emit_faction_victory`, faction_influence.py:129-157)

- **(a)** If any Faction's aggregate territorial-influence share (fraction of all territories it currently wins) clears a supermajority threshold, emit victory.
- **(b)** `counts[faction_id] = counts.get(faction_id, 0) + 1` per winning territory (139-141); `total = sum(counts.values())` (142); `share = counts[faction_id] / total` (146) — plain int/int true-division; `if share >= threshold: emit` (147-157), threshold = `defines.faction_victory_supermajority_threshold`.
- **(c) Reads:** the `winning` dict from 2.1.
- **(d) Writes:** event bus only.
- **(e) Defines:** `balkanization.faction_victory_supermajority_threshold` — default `0.66`, domain `[0.5, 1.0]` (`BalkanizationDefines` line 127-132; `defines.yaml:345`).
- **(f) Events:** `EventType.FACTION_VICTORY` (faction_influence.py:150).

### 2.4 — RED_SETTLER_TRAP_DETECTED diagnostic (`_emit_red_settler_trap_events`, faction_influence.py:159-190; `detect_red_settler_trap`, balkanization.py:215-252)

- **(a)** A Faction whose `class_reduction` is high (it has substantially defused class contradiction) while its `colonial_stance` is not ABOLISH (it has not dismantled the settler relationship) fires the diagnostic — the canonical RED_OGV trap condition.
- **(b)** `if stance is ABOLISH: return False`; `return class_reduction >= threshold` (balkanization.py:250-252).
- **(c) Reads:** every `FACTION` node's `colonial_stance` (str, cast to `ColonialStance`, faction_influence.py:171-177 — a `ValueError`-caught cast; a Faction with an unparseable `colonial_stance` value silently skips the check, `continue` at 177, no error surfaced) and `class_reduction` (`float(attrs.get("class_reduction", 0.0))`, 178 — bare `0.0` default).
- **(d) Writes:** event bus only.
- **(e) Defines:** `balkanization.red_settler_trap_class_reduction_threshold` — default `0.6`, domain `[0.0, 1.0]` (`BalkanizationDefines` 143-148; `defines.yaml:347`).
- **(f) Events:** `EventType.RED_SETTLER_TRAP_DETECTED` (faction_influence.py:182), payload includes `class_reduction` and `colonial_stance.value` (185-188).
- **RESERVED-LINE.** `colonial_stance` is Constitution I.1's principal-contradiction axis; the three values (`UPHOLD`/`IGNORE`/`ABOLISH`) and this trap's threshold are Director-owned MLM-TW National Question content. The four canonical seed factions (`seed_factions.json`) are: `FAC_RESTORATIONIST` (uphold, class_reduction 0.0), `FAC_WORKERS_CONGRESS` (ignore, 0.7), `FAC_DECOLONIAL` (abolish, 0.5), `FAC_LIBERAL_IMPERIAL` (ignore, 0.7). Two of the four (`FAC_WORKERS_CONGRESS`, `FAC_LIBERAL_IMPERIAL`) are seeded to trip this diagnostic by construction once evaluated — described here as a verified fact, not proposed upon.

### 2.5 — Secession eligibility, hysteresis, SECESSION_DECLARED (`_update_secession_eligibility`, faction_influence.py:192-251; `contiguous_influence_majority_subregion` + helpers, balkanization.py:255-363)

- **(a)** For every non-incumbent (Faction, Sovereign) pair, find the largest contiguous ADJACENCY-connected sub-region of that Sovereign's claimed Territories where the Faction's influence exceeds a threshold; if that region is at least a minimum size, accumulate a per-tick hysteresis counter; once the counter clears a window, declare secession eligible and emit, then reset the counter.
- **(b)** Eligible-territory set: `claimed = {t for t,_,_ in query_sovereign_claims(sovereign_id)}` then `{tid for tid in claimed if _faction_influence_in(graph, faction_id, tid) > threshold}` (balkanization.py:308-312) — **strict `>`**, not `>=`. `_faction_influence_in` sums `influence_level` over every INFLUENCES row matching `faction_id` on that territory (315-322). Largest component: iterate `sorted(eligible)` seeds, BFS each unvisited one via `query_adjacent_territories`, keep the largest (325-363) — a hand-rolled, deterministic-frontier BFS (lex-sorted per level, 354-361). `if len(best) < min_contiguous_hex_count: return frozenset()` (294). Hysteresis: `hysteresis[key] = hysteresis.get(key, 0) + 1` if the region is non-empty, else reset to `0` (faction_influence.py:220-223); `if hysteresis[key] >= secession_hysteresis_ticks: emit SECESSION_DECLARED; hysteresis[key] = 0` (224-244). Stale-key cleanup: any `(faction, sovereign)` key not touched this tick is deleted (247-249).
- **(c) Reads:** `SOVEREIGN.ruling_faction_id` (211, the incumbent-skip test — `for faction_id in faction_ids: if faction_id == incumbent: continue`), `CLAIMS` edges (via `query_sovereign_claims`, only the target-id half of each row used, `control_level`/`legal_status` read into the tuple but unused here), `INFLUENCES` edges (same influence-sum as 2.1/2.4), `ADJACENCY` edges (via `query_adjacent_territories`), `persistent["balkanization.hysteresis_buffer"]` (200).
- **(d) Writes:** `context.persistent_data["balkanization.hysteresis_buffer"]` (250) and `["balkanization.secession_eligible"]` (251) — both off-graph.
- **(e) Defines:** `balkanization.secession_influence_threshold` (default `0.5`, `[0,1]`, defines.yaml:337), `balkanization.min_contiguous_hex_count` (default `12`, `>=1`, defines.yaml:339), `balkanization.secession_hysteresis_ticks` (default `3`, `>=1`, defines.yaml:338).
- **(f) Events:** `EventType.SECESSION_DECLARED` (faction_influence.py:234).
- **Genuine oddity, transcribed verbatim (port-as-is law).** `SOV_EXTERIOR_NULL` (the FR-040b fallback Sovereign, `ruling_faction_id=None`) is iterated by this loop exactly like any real Sovereign — `if faction_id == incumbent` never matches (no real faction ID ever equals `None`), so `contiguous_influence_majority_subregion` runs against it for every Faction every tick. In practice it is a harmless no-op (`SOV_EXTERIOR_NULL` holds no `CLAIMS`, so `_eligible_territories` returns empty), but it is wasted iteration transcribed as found, not corrected.
- **Genuine oddity, transcribed verbatim.** `GraphProtocol` already exposes `query_contiguous_component_under_predicate` (graph_protocol.py:473-494, a single-seed BFS-under-predicate, implemented at graph.py:1003+) — `_largest_contiguous_component`/`_bfs_component` **do not use it**, hand-rolling an equivalent BFS instead. This is a DRY-violation (unused native primitive, duplicated logic), recorded because port-as-is law requires transcribing defects rather than silently repairing them. It does not change the port verdict: the native method's second parameter is an arbitrary Python `Callable[[str], bool]`, which is itself unportable to BSL (no first-class function values) — so even calling the "proper" primitive would not have made this computation any more expressible.
- **Genuine oddity, transcribed verbatim.** `faction_influence.py:80-81`: `with contextlib.suppress(AttributeError): context.persistent_data = persistent`. Since `persistent = context.persistent_data` at line 66 is the same dict object mutated in place throughout `step()`, this reassignment is a no-op except as defense against an alternate `context` shape without a settable `persistent_data` attribute. Pure Python housekeeping with no computational content — noted for completeness, not a port blocker.

**Events emitted by the whole system: 4 distinct `EventType` values** — `TERRITORY_TRANSITION`, `FACTION_VICTORY`, `RED_SETTLER_TRAP_DETECTED`, `SECESSION_DECLARED` (grep-confirmed exhaustive, faction_influence.py). Per the CURRENT BSL surface: `emit` exists as a served effect verb, but TickReport carries no event log — every one of these is a WS1 (#502) ledger row, unpinnable by goldens today, matching Territory/Metabolism precedent.

---

## 3. TYPE INVENTORY

Runtime-storage caveat, carried forward from every prior inventory in this series and
independently re-verified here: `BabylonGraph.add_node`/`add_edge` (topology/graph.py:165+) are
plain-kwargs dict merges with **no Pydantic validation or type coercion**. `BalkanizationFaction`
and `Sovereign`'s field constraints (`Probability` bounds, the `colonial_stance` enum, the
`ruling_faction_id` regex) apply only when those Pydantic models are explicitly instantiated
(seed construction) — never mid-tick. This system reads node attributes via raw
`node.attributes.get(...)`, not `SystemBase._read`, so there is no `required=True` diagnostic
guard on any of its reads either (a further, mild contrast with systems that use `_read`).

| Attribute | Node/Edge | Python model type | Domain | Category |
|---|---|---|---|---|
| `colonial_stance` | FACTION | `ColonialStance` (StrEnum, 3 members) | `{uphold, ignore, abolish}` | **Enum discriminant, RESERVED-LINE** |
| `class_reduction` | FACTION | `Probability` (`Annotated[float, ge=0.0, le=1.0, SnapToGrid]`) | `[0.0, 1.0]` | unit-interval |
| `ruling_faction_id` | SOVEREIGN | `str \| None`, regex `^FAC_[A-Z][A-Z0-9_]*$` | closed FK to a FACTION id, or `None` (SOV_EXTERIOR_NULL only) | **string foreign-key reference** |
| `influence_level` | INFLUENCES edge | `float \| None` (`Relationship`, `ge=0.0, le=1.0`); read back via `query_faction_influence_by_territory` as always-float, `0.0` default | `[0.0, 1.0]` | **edge-scoped unit-interval** |
| `support_type` | INFLUENCES edge | `str \| None` (free-form; `SupportType` StrEnum, 5 members, exists but this field is stored as plain `str`, not validated against it at graph-write time) | `{material, ideological, military, electoral, labor}` per `SupportType` | **declared, unused by this system's own computation** (§2.1) |
| `control_level`, `legal_status` | CLAIMS edge | `float \| None` `[0,1]`; `str \| None` | — | **read into the row tuple, unused by any formula this system calls** (§2.5) |
| `faction_victory_supermajority_threshold`, `secession_influence_threshold`, `red_settler_trap_class_reduction_threshold` (defines) | — | `float` | `[0.5,1.0]`, `[0,1]`, `[0,1]` respectively | unit-interval coefficients |
| `secession_hysteresis_ticks`, `min_contiguous_hex_count` (defines) | — | `int` | `>=1` each | integer coefficients |
| `winning_faction_by_territory`, `previous_winning_faction_by_territory` (persistent_data) | — | `dict[str, str]` (territory_id → faction_id) | — | **off-graph, per-territory reference map** |
| `hysteresis_buffer` (persistent_data) | — | `dict[str, int]` (pair-key string → count) | `>= 0` | **off-graph, pair-keyed integer accumulator** |
| `secession_eligible` (persistent_data) | — | `list[dict]` (composite records: faction id, sovereign id, territory-id tuple) | — | **off-graph, list of composite records** |

**Reference-typed value flag — the load-bearing new finding of this inventory.** `deffield`'s
closed type vocabulary is exactly seven names — `int`, `bool`, `currency`, `probability`,
`intensity`, `coefficient`, `enum` (declarations.rs:646-675, verified against current dev) —
**none of them a reference type.** `winning_faction_for_territory`'s result is a Faction *ID*,
i.e., structurally a `NodeRef`. There is no `deffield` type that can hold one. This is a
different and more severe gap than Territory's enum-storage finding (a `bool`/`int`-ordinal
workaround exists there): here there is no scalar encoding available at all unless the roster of
possible Factions is closed and fixed at content-authoring time (it currently is — no System
ever mints a FACTION node at runtime, grep-confirmed across `src/babylon/engine/systems/`) and a
scenario were willing to declare a bespoke enum naming each seed Faction. No landed or drafted
BSL ruling authorizes a scenario-scoped enum keyed to a specific roster of content IDs, so this
is presented as an open design question, not a solution.

**Off-graph state flag — the second load-bearing new finding.** `winning_faction_by_territory`,
`previous_winning_faction_by_territory`, `hysteresis_buffer` and `secession_eligible` are all
`context.persistent_data` entries, never graph attributes. `docs/reference/bsl-language.rst`
§3.6 (R9 chapter C3) already rules on this shape for **graph-scope scalars** — "ordinary node
state on a declared carrier node type" — but that ruling explicitly does not cover a per-pair or
per-territory *reference* map, nor a growing list of composite records, and it itself depends on
the `the` query head, which is Slice 2 and unserved today (`UNSERVED_EXPRESSION_HEADS`,
evaluator.rs:503-512). `reports/bsl-gap-analysis-2026-08-10.md`'s own Q6 system list (22 systems,
line 245-249) **does not include FactionInfluence at all** — its `hysteresis_buffer` is instead
filed once, in passing, under Q3 ("FactionInfluence (hysteresis on a candidacy edge)," line 170)
as a possible future content-modeling fix (mint a candidacy edge, store the count as its
`:strength`). No report or ruling names `winning_faction_by_territory`/`secession_eligible`'s
shape at all. Flagged here as a genuinely new, previously-unnamed finding.

---

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`/`int`). Grep-confirmed zero
`exp`/`log`/`pow`/`sigmoid`/`math.` calls anywhere in `faction_influence.py` or the functions of
`balkanization.py` this system calls — **zero libm-nondeterminism hazard**, same clean result as
Territory. **Zero `int(...)` truncations anywhere either** — a further contrast with Territory
(two Real→Int demotions) and Metabolism: this system has no floor/trunc hazard at all. Shapes,
in execution order:

1. **Additive accumulation (grouped):** `totals[faction_id] = totals.get(faction_id, 0.0) + float(influence_level)` (balkanization.py:199) — per-faction running sum over an edge set, structurally identical to Territory's "favorable" collect-then-apply pattern, but the operand here is an **edge** attribute, not a node attribute.
2. **Max + epsilon-tolerance tie test:** `max_total = max(totals.values())` (204); `tied = [... if (max_total - total) <= eps ...]` with `eps = 1e-12` (205-206) — a bare scientific-notation literal, hardcoded (not `defines`-sourced). Two implementations comparing floating totals for a tie must agree on this exact epsilon and its associated `<=` vs `<` semantics to reproduce the same tied-set bit-for-bit; this is a determinism-parity concern independent of any BSL language gap.
3. **Integer/count division:** `share = counts[faction_id] / total` (faction_influence.py:146) — Python 3 true division of two `int`s producing a `float`; `if share >= threshold` (147).
4. **Threshold comparisons, no arithmetic:** `class_reduction >= threshold` (balkanization.py:252, RED_SETTLER_TRAP), `_faction_influence_in(...) > threshold` (balkanization.py:312, **strict**, note the asymmetry with #3/#4's `>=`), `len(best) < min_contiguous_hex_count` (294).
5. **Additive accumulation (single-faction, re-summed):** `_faction_influence_in` re-sums the same `influence_level` values (balkanization.py:315-322) independently of #1 above — a second, textually-duplicated accumulation shape over the same edge attribute, for a different (faction, territory) grouping.
6. **Integer counters, no float involved:** `counts[faction_id] = counts.get(faction_id, 0) + 1` (faction_influence.py:141), `hysteresis[key] = hysteresis.get(key, 0) + 1` / reset to `0` (faction_influence.py:220-223).
7. **RNG draw, a determinism hazard distinct from libm:** `rng.choice(tied)` (balkanization.py:212) is CPython's `random.Random.choice`, backed by the Mersenne Twister via `_randbelow`. This is **not** a libm transcendental, but it carries the identical cross-language-reproducibility risk the CLAUDE.md behavioral-contracts principle names for libm: a Rust reimplementation would need to match CPython's exact bit-stream algorithm, not just "a seeded PRNG," to replay historical runs byte-identically. **Verified: RNG has no BSL representation at all today** — grep across `rust/crates/babylon-bsl/src/*.rs` and `rust/crates/babylon-tick/src/*.rs` for `rng`/`Random` returns zero hits. `reports/bsl-gap-analysis-2026-08-10.md:615` records RNG as a sanctioned-but-unbuilt kernel intrinsic ("§2.8 already sanctions RNG as a kernel intrinsic... R10 classes it as a seam") — spec status, not implementation status. This fires only on a genuine unresolved tie with no incumbent present, a narrow but real edge case this system's own hypothesis-based law test (`test_law_faction_influence.py`) deliberately avoids exercising (see §7).
8. **No clamps anywhere.** Unlike Territory (two different `[0,1]` clamp implementations) or Metabolism, this system writes no graph state at all, so there is nothing to clamp — `_write_clamped` is never called.

---

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 14.5** (faction_influence.py:53), confirmed against `_SYSTEM_CLASSES`
  (simulation_engine.py:328-363) and each neighbor's own `position` ClassVar (read directly):
  `OODASystem (14.0) → FactionInfluenceSystem (14.5) → DoctrineSystem (14.7) → SurvivalSystem
  (15.0)`.
- **Reads from a same-tick prior system: none.** `OODASystem` (immediately prior) touches
  neither `colonial_stance`/`class_reduction`/`influence_level`/`ruling_faction_id` nor calls
  `update_node`/`update_edge` at all (grep-confirmed, `ooda.py`). No System anywhere mints or
  removes a `TERRITORY`, `FACTION`, or `SOVEREIGN` node — `TERRITORY` is immutable substrate;
  `FACTION` is never minted at runtime by any System; `SOVEREIGN` is only minted/removed by
  `CollapseTransitionSystem` (@20.5, **after** this system). Every attribute this system reads is
  either its own prior-tick `persistent_data` or scenario-seed state.
- **This system writes zero graph attributes** — the structural fact that shapes everything
  else in this section. Its only writes are:
  - **`context.persistent_data["balkanization.winning_faction_by_territory"]`** and
    **`["balkanization.secession_eligible"]`** — read by exactly one downstream consumer,
    `CollapseTransitionSystem`, same tick, @20.5 (`collapse_transition.py:71,93`, grep-confirmed
    no other reader anywhere in `src/babylon/`).
  - **`persistent["balkanization.previous_winning_faction_by_territory"]`** and
    **`["balkanization.hysteresis_buffer"]`** — read by **no other system**; purely
    self-referential across ticks (grep-confirmed, the two module-level constant strings
    `_PREV_WINNING`/`_HYSTERESIS` appear only in `faction_influence.py`).
  - Four `EventType` emissions (§2), consumed downstream only via `event_builders.py`'s
    replay/observability conversion (Python-glue boundary, out of the port's scope) — no System
    reads any of these event types back as engine-adjudicated state.
- **Shared-readership, not producer-consumer.** `colonial_stance` is read by **six** call sites
  total (`faction_influence.py` itself, `contradiction.py:645`, `collapse_transition.py:306`,
  `electoral.py:434`, `reactionary.py:221` — this last System is **not registered** in
  `_SYSTEM_CLASSES`, dead/unwired code, grep-confirmed absent from `simulation_engine.py` —, and
  `engine/observers/endgame_detector.py:806`) and written by **none** — it is scenario-seed-only,
  immutable for the whole run (grep-confirmed: no `colonial_stance=` kwarg to any
  `update_node`/`add_node` call anywhere in `src/babylon/engine/`). `influence_level` is likewise
  seeded exactly once (`balkanization_seed.py:115`) and never mutated in-tick by any System
  (grep-confirmed no `update_edge` call touching an `INFLUENCES` edge anywhere) — read only by
  this system and by `ContradictionSystem` (@18.0, `contradiction.py:626-629`, its own
  national-axis Balance term). **Practical consequence:** since the underlying INFLUENCES data
  never changes after seeding, `winning_faction_for_territory`'s per-territory result is stable
  from the first tick it is ever computed onward (barring the narrow incumbent/RNG-tiebreak
  path) — the system's own dominant behavior across a run is "compute the same argmax every
  tick," not an evolving contest.
- **Context/service usage with no BSL equivalent:** `resolve_rng(services, tick)`
  (system_base.py:35-55) — falls back to `random.Random(0xBA1AC1A + tick)` when
  `services.rng` is absent (the harness-injection path); see §4 item 7 for the determinism
  hazard this carries independent of any BSL gap.
- **DORMANCY on canonical scenarios — confirmed, not inferred.** `tools/regression_scenarios.py`
  `COVERAGE_GAPS_DATA` (2761-2768): *"no FACTION nodes are seeded; `winning_faction_for_territory`
  returns `None` for every territory every tick, so `balkanization.winning_faction_by_territory`
  stays an empty dict and no TERRITORY_TRANSITION/FACTION_VICTORY/RED_SETTLER_TRAP_DETECTED/
  SECESSION_DECLARED event fires."* Independently corroborated by `balkanization_seed.py`'s own
  docstring (29-31, quoted in full in §1) and by `reports/bsl-gap-analysis-2026-08-10.md`'s
  "Class B — topology-absent" dormancy classification (649-650), which names
  FactionInfluence explicitly among nine systems in that class. **All four downstream
  consequences of this system are therefore unexercised by every one of the six canonical
  `qa:regression` scenarios**, and `CollapseTransitionSystem` — the sole consumer of this
  system's `persistent_data` writes — carries the identical "no SOVEREIGN nodes are seeded" gap
  (`regression_scenarios.py:2842-2846`), so the whole downstream chain from this system is dark
  on canonical scenarios, not just this system's own tick. A port's conformance fixtures would
  need to be hand-built (as Metabolism's `.bscn` fixtures are), invoking
  `apply_balkanization_seed` or an equivalent hand-authored `.bscn` — nothing is harvestable from
  the canonical estate.

---

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface (Slice 1 query lane landed per ADR197; Slices 2-4
NOT built; enum fields landed per ADR195/196; verified myself against `evaluator.rs`/`query.rs`/
`structural_verbs.rs`/`declarations.rs` on current `dev`, not taken on the prior report's word).

| # | Computation | Verdict | Detail |
|---|---|---|---|
| 1 | Winning-faction argmax + persistent handoff (§2.1) | **BLOCKED — three independent lanes** | (a) Needs a per-edge weighted sum grouped by source Faction: `edges`/`edge-between` are Slice 2, unserved (`UNSERVED_EXPRESSION_HEADS`, evaluator.rs:503-512); `field-of` over an `EdgeRef` is unreachable — no expression form produces one (evaluator.rs:1185-1191); even the substrate's own built-in edge "strength" scalar is invisible to every served query head (`neighbors` discards it, query.rs:155-214). (b) `support_type` (INFLUENCES' second declared attribute) has **no storage path at all** — `add-edge` refuses any field beyond `:strength` (structural_verbs.rs:902-908) — though this system's own formula never reads it (dead in the row tuple, §2.1c), so this half doesn't bind *this* computation specifically. (c) The result is a `NodeRef`-shaped value (a Faction ID) and `deffield`'s closed 7-type vocabulary has no reference type (declarations.rs:646-675) — so even a hypothetical future Slice-2 EdgeRef landing would not by itself let a rule persist "the winning faction" anywhere. (d) The consumer is an off-graph `persistent_data` handoff to `CollapseTransitionSystem` — no §2.8 verb writes anything but a node/edge/hyperedge/event, and the one landed/drafted ruling for graph-scope state (bsl-language.rst §3.6, singleton carrier node) does not fit a per-territory reference map, and itself depends on the unserved `the` head. **This is a new finding — no existing Q-item in `bsl-gap-analysis-2026-08-10.md` names the persistent-handoff shape for this system at all (Q6's own system list omits FactionInfluence).** |
| 2 | TERRITORY_TRANSITION emission (§2.2) | **BLOCKED (downstream of Row 1)** | The iteration mechanics (`for-each`) are themselves served in effect position (structural_verbs.rs:370, ADR197 precedent) — the blocker is entirely that the comparison operands (`old`/`new` winning-faction IDs) are Row 1's un-portable output. Even solved, `emit` is a WS1 (#502) ledger item — unpinnable by goldens today. |
| 3 | FACTION_VICTORY emission (§2.3) | **BLOCKED (downstream of Row 1)** | The arithmetic itself (division, threshold compare) is trivial and has no independent language obstacle — it operates on the same un-portable `winning` dict. WS1 applies to the emit regardless. |
| 4 | RED_SETTLER_TRAP_DETECTED diagnostic (§2.4) | **PORTABLE WITH D-RECORD** (+ RESERVED-LINE content, + WS1 on the emit) | Pure per-`FACTION`-node rule: `colonial_stance` is a landed 3-valued enum field (`defenum`/`deffield enum`/`=` comparison, ADR195/196); `class_reduction` is a plain `probability` deffield. No query/fold needed — `(nodes NodeType/FACTION)` + `for-each` + a guard. D-record: pin the enum member declaration order (it is the stored ordinal, hash-bearing). WS1 on the `emit`. The seed values themselves are RESERVED-LINE (§2.4). |
| 5 | Secession-eligibility CLAIMS membership, narrow (`_eligible_territories`'s claimed-set half, balkanization.py:308-310) | **PORTABLE (Slice 1)** | `query_sovereign_claims`'s only functional use here is its target-territory-id set — `(neighbors sovereign-ref EdgeType/CLAIMS :out NodeType/TERRITORY)` is landed and evaluated (ADR197). `control_level`/`legal_status` are read into the row tuple but never used by this call site, so their own Q1 gap doesn't bind here. |
| 6 | Secession-eligibility influence threshold (`_faction_influence_in`, balkanization.py:315-322) | **BLOCKED — same lane as Row 1(a)** | Identical edge-attribute-sum gap as the winning-faction argmax, applied to a single (faction, territory) pair instead of a grouped sum. |
| 7 | Contiguous-component BFS (`_largest_contiguous_component`/`_bfs_component`, balkanization.py:325-363) | **BLOCKED — a different kind, no slice names it** | An unbounded-frontier breadth-first search over ADJACENCY-linked eligible territories (`while frontier:`, no static hop bound). Every served or designed BSL construct (`fold`/`select-max`/`select-min`/`exists`/`forall`/`for-each`) is a single materialize-then-iterate pass over **one** query's result set (§3.7's cost formula: `ceiling(query) × cost(body)`, one query); §2.7 states "Folds are the only iteration construct," and none of them re-query mid-fold. Multi-hop reachability closure has **no BSL analog at any slice named in the query-evaluation plan or the language reference** — not "missing a slice," a control-flow shape the current one-rule-per-position, single-pass execution model does not admit at all. The one native primitive shaped for this (`query_contiguous_component_under_predicate`, unused by this call site — see §2.5) is itself unportable regardless, since its predicate parameter is an arbitrary Python `Callable` and BSL has no first-class function values. Flagged as a structurally open design question, more severe than the edge-attribute gaps above. |
| 8 | Hysteresis counter + SECESSION_DECLARED emission (`_update_secession_eligibility`, faction_influence.py:192-251) | **BLOCKED — compounds Rows 1/6/7, plus its own state-shape gap** | Depends on Rows 6 and 7 to know which pairs are eligible. The hysteresis counter is keyed by a string-concatenated `(faction_id, sovereign_id)` pair in `persistent_data` — off-graph, pair-keyed accumulator state with no graph-node home (not even the §3.6 carrier-node ruling helps; it is not one-per-graph). `bsl-gap-analysis-2026-08-10.md:170` records a proposed fix — "hysteresis on a candidacy edge" (mint an edge carrying the count as `:strength`) — but this is a recorded proposal, not a ruling, and would still need `update-edge` to increment in place, which is refused today (structural_verbs.rs:371-382, the same substrate-widening gap as Row 1). The emitted `secession_eligible` payload is a growing list of composite records (faction id + sovereign id + a variable-length territory-id tuple) — no BSL shape hands off a list of composite records between systems either. |
| — | `TickContext.displacement_mode`-style config override | **N/A** | Unlike Territory, this system reads no such per-run override at all — no analogous item exists here. |

**Summary.** Of eight computations: **one is portable with a D-record** (the trap diagnostic,
Row 4), **one is portable narrowly and only as a sub-clause of a larger blocked computation**
(Row 5), and **six are blocked**, spanning every one of Q1 (edge-attribute reads), Q2 (edge
endpoint/incidence queries), Q3 (`update-edge`/field-init widening), Q4 (element selection), and
Q5 (effect-position cross-node targeting) from `bsl-gap-analysis-2026-08-10.md`, plus two
findings that report does not name at all (the reference-typed field-storage gap, and the
persistent-handoff/pair-keyed-state gap) and one control-flow shape (unbounded BFS) that no
slice or ruling addresses. This is a strictly wider blocker surface than Territory's, which
this template's own prior inventory found already sufficient to defer a whole port train on.

---

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/balkanization/test_faction_influence_system.py` | 219 | **Primary conformance-oracle candidate.** Exercises winning-faction resolution, TERRITORY_TRANSITION on flip / no-flip, FACTION_VICTORY on supermajority, RED_SETTLER_TRAP_DETECTED (fires for UPHOLD/IGNORE, skips ABOLISH), and the unclaimed-territory no-entry case. **Does not exercise `_update_secession_eligibility` at all** — no test in this file seeds a `SOVEREIGN` node or asserts on `SECESSION_DECLARED`/`hysteresis_buffer`/`secession_eligible`. |
| `tests/unit/engine/laws/test_law_faction_influence.py` | 255 | **Property-based (Hypothesis) behavioral-contract laws**, four named: L1 argmax soundness, L2 unchanged-winner inactivity, L3 FACTION_VICTORY soundness + uniqueness, L4 empty-graph inactivity. Explicitly documents its own scope limit in a code comment (16-17): tie-neighborhood cases are avoided by construction, "isolating the argmax property itself" from the incumbent/RNG tiebreak — meaning **the RNG-tiebreak path (§4 item 7) has zero property-based coverage** in this file. **Also does not exercise secession eligibility at all** — confirmed by grep for `SECESSION_DECLARED`/`_update_secession_eligibility` across this file (no hits). |
| `tests/unit/balkanization/test_faction_node_type_query.py` | 117 | **Regression test for a previously-fixed live defect**, cited in CLAUDE.md's own gotchas list: `FactionInfluenceSystem` once queried `node_type="balkanization_faction"` (never matches; canonical stamp is `"faction"`), silently zeroing out RED_SETTLER_TRAP_DETECTED and secession enumeration. Directly relevant conformance history — the port must not reintroduce an equivalent type-string mismatch under BSL's closed `NodeType` enum vocabulary (which would catch this class of bug at load time rather than silently, an improvement worth noting). |
| `tests/unit/balkanization/test_graph_protocol_extensions.py` | 223 | Tests the six `GraphProtocol` extension methods (including the unused `query_contiguous_component_under_predicate`, §2.5) **in isolation** — never through `FactionInfluenceSystem` itself. Useful for pinning the accessor contracts, not a system-level conformance oracle. |
| `tests/unit/engine/scenarios/test_balkanization_seed.py` | 187 | Behavioral contract for `apply_balkanization_seed` — the seed **pipeline**, upstream of this system's tick logic. Establishes the topology this system needs to be non-dormant; not itself a tick-behavior oracle. |
| `tests/unit/balkanization/test_balkanization_defines.py` | 119 | Schema/value-sync test (defines.yaml ↔ Pydantic defaults). Includes `test_secession_and_endgame_thresholds_match_schema` (62-68) — confirms `secession_influence_threshold`/`secession_hysteresis_ticks` values, but does not exercise the secession *behavior*. |
| `tests/unit/balkanization/test_enums.py` | 139 | `ColonialStance`/other enum member/value tests — schema-level. |
| `tests/unit/balkanization/test_event_payloads.py` | 180 | Pure Pydantic payload-schema validation for all 9 spec-070 event types (not just this system's 4) against `contracts/balkanization_events.json` — schema tests, not System behavior. |
| `tests/unit/balkanization/test_seed_influences.py` | 348 | Seed-data **provenance/computation-pipeline** tests (MIT Election Lab vote-share sourcing, byte-identical determinism of the seed computation itself, schema validation) — data-pipeline testing, not this system's tick behavior. |
| `tests/unit/formulas/test_balkanization_import.py` | 30 | Import-surface smoke test. |
| `tests/integration/balkanization/test_us4_secession_fracture.py` | 263 | Tests `CollapseTransitionSystem`'s consumption of a **hand-written** `persistent["balkanization.secession_eligible"]` fixture — **does not call `FactionInfluenceSystem._update_secession_eligibility` to produce that list**; the producer side (Row 8, §6) is exercised nowhere in this file despite its name. |
| `tests/integration/balkanization/test_us1_extraction_trajectory.py` | 223 | Sovereign/extraction-policy trajectory integration test — does not exercise `FactionInfluenceSystem`. |
| `tests/integration/balkanization/test_determinism_replay.py` | 236 | Determinism/replay integration coverage for the broader balkanization estate. |
| `tests/integration/balkanization/test_audit_round_trip.py` | 158 | Audit-log round-trip for balkanization mutations — does not itself touch `FactionInfluenceSystem`'s computation. |
| `tests/integration/balkanization/test_seed_coverage_invariant.py` | 126 | Seed-coverage invariant (every Territory carries exactly one CLAIMS edge) — seed-pipeline testing. |
| `tests/integration/balkanization/test_postgres_persistence.py` | 63 | Postgres persistence round-trip — infrastructure, not System behavior. |
| `tests/unit/projection/test_faction.py` | 307 | `observe()`-page projection/rendering for Faction state — presentation layer, not engine math. |

**Coverage gap, confirmed by direct search (not inferred from a table).** `_update_secession_eligibility`
— roughly a third of this system's own code (faction_influence.py:192-251) — together with
`contiguous_influence_majority_subregion` and its BFS helpers, has **zero direct test coverage
anywhere in the repository**. Grep for `_update_secession_eligibility`, `_bfs_component`,
`_largest_contiguous_component`, `_eligible_territories` across `tests/` returns no hits; grep
for `SECESSION_DECLARED`/`contiguous_influence_majority_subregion` returns only schema/defines/
event-conversion tests, never a call through the System. This is independent of, and in addition
to, the canonical-scenario dormancy finding in §5 — even a hand-built unit-test graph never
exercises this code path today.

**qa:regression byte-gate coverage.** As established in §5, no canonical scenario seeds
`FACTION`/`SOVEREIGN` nodes, so `graph_content_hash` (`tools/regression_test.py:924-964`) never
observes a non-empty output from this system on any of the six scenarios — the byte-identical
hash gate provides **zero real coverage** for this system today, a stronger statement than
Territory's partial-coverage finding (Territory's Phase 1 heat dynamics at least fires on every
scenario). A port's conformance fixtures would need to be entirely hand-built.

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`). The Rust-side verification in §1 is
excellent and every anchor in it re-verified clean — this inventory did its own reading rather
than inheriting a prior report's, exactly as the house standard asks. Its **BLOCKED verdict on
the language gaps survives intact**. Its **dormancy finding does not**: the system has a live,
byte-gated conformance oracle on five canonical scenarios today, and the source it trusted for
the contrary claim is stale.

1. **CORRECTION — the system is NOT dormant on every canonical scenario. Five of the twelve
   registered scenarios seed FACTION and SOVEREIGN nodes.** §5 and §7 rest on
   `balkanization_seed.py`'s own docstring ("Byte-safety: NOT applied by any of the six
   qa:regression scenario factories — FACTION/SOVEREIGN nodes land only in the
   electoral/balkanization scenarios," `src/babylon/engine/scenarios/balkanization_seed.py:30-32`)
   and on the matching `COVERAGE_GAPS_DATA` row. Both are stale, because **the electoral
   scenarios ARE canonical qa:regression scenarios now** (P25 U13/ADR140 made the electoral
   goldens qa scenarios 7-11). The chain, verified end to end: `create_mitterrand_scenario`,
   `create_syriza_scenario`, `create_weimar_scenario`, `create_debs_scenario`,
   `create_bernie_valve_scenario` (`src/babylon/engine/scenarios/electoral_goldens.py:209,253,
   310,410,481`) each call `apply_political_terrain` (`:226,270,334,428,504`), which calls
   `apply_balkanization_seed(state)` at `electoral_fixture.py:204` — the function whose own
   docstring says nobody canonical calls it. All five are registered in `SCENARIOS`
   (`tools/regression_scenarios.py:77,88,97,106,115`), all five have committed baselines
   (`tests/baselines/{mitterrand,syriza,weimar,debs,bernie_valve}.json` + `dense/*.csv`),
   `PENDING_CEREMONY` is empty (`:143`), and `qa:regression compare` iterates the whole registry
   (`tools/regression_test.py:1424`). `WorldState` carries `sovereigns`/`factions` as first-class
   dicts that round-trip through `to_graph`/`from_graph` (`models/world_state.py:643-649,
   785-787, 981-1000, 1042-1043`), so `graph_content_hash` — which hashes every node/edge
   attribute of the projection — **does** observe this system's inputs and (via
   `CollapseTransitionSystem`'s consumption) its downstream effects. §7's "the byte-identical hash
   gate provides **zero real coverage** for this system today" is withdrawn.

2. **CORRECTION — "the six canonical `qa:regression` scenarios" is stale by six.** The phrase
   recurs in §5, §6 and §7. `SCENARIOS` has **twelve** keys — `imperial_circuit`, `two_node`,
   `starvation`, `glut`, `fascist_bifurcation`, `single_county`, `mitterrand`, `syriza`, `weimar`,
   `debs`, `bernie_valve`, `org_probe` (`tools/regression_scenarios.py:38-128`) — and
   `compare_all_baselines` walks all of them. Every dormancy conclusion in this report that
   quotes a "six"/"five" count from a source comment inherits that staleness and must be
   re-derived over twelve.

3. **CORRECTION — `reactionary.py` is NOT dead/unwired code.** §5's shared-readership bullet
   states of the `colonial_stance` reader at `reactionary.py:221`: "this last System is **not
   registered** in `_SYSTEM_CLASSES`, dead/unwired code, grep-confirmed absent from
   `simulation_engine.py`." The module hosts `FascistFactionSystem`
   (`src/babylon/engine/systems/reactionary.py:74`, `position: ClassVar[float] = 17.4`, `:78`),
   which is imported at `simulation_engine.py:67`
   (`from babylon.engine.systems.reactionary import FascistFactionSystem`) and registered in
   `_SYSTEM_CLASSES` (`:349`). The grep missed it because the class name and the module name
   differ. So `colonial_stance` has **six** live System readers, not five, and one of them runs
   at 17.4 — after this system, same tick.

4. **CORRECTION (strengthening) — Row 8's proposed hysteresis-on-a-candidacy-edge fix is blocked
   one layer earlier than stated.** The row correctly notes the proposal would "still need
   `update-edge` to increment in place, which is refused today." It would also need to **mint**
   the candidacy edge from a rule, and `add-edge` is one of the six `DEFERRED_SHAPE_VERBS`
   (`structural_verbs.rs:1352-1359`) refused **at content load** by `check_no_deferred_shape_verbs`
   (`:1388-1406`), wired unconditionally into `rule_pipeline::load_rule_form`
   (`rule_pipeline.rs:268`) — Task 12's collect-then-apply split "does not serve the graph-shape
   verbs, only update-node/emit/guard/for-each" (`structural_verbs.rs:691-694`). A rule containing
   `add-edge` anywhere in `<when>`/`<effects>` does not load at all. Same gate applies to any
   future proposal that mints a node or edge to carry off-graph state.

5. **CONFIRMATION — the reference-typed-field-storage finding is real and correctly the
   load-bearing one.** `parse_type_name` (`declarations.rs:646-675`) is exhaustive over exactly
   `int`/`bool`/`currency`/`probability`/`intensity`/`coefficient`/`enum`, and even the `enum`
   arm refuses without a companion `:enum-type` (`:664-669`). No reference type exists. A Faction
   ID is structurally a `NodeRef` and has nowhere to be stored.

6. **CONFIRMATION — every edge-lane anchor.** `field-of` serves `NodeRef` referents only; the doc
   states verbatim "an `EdgeRef` referent is unreachable today (no expression form produces one
   yet; slice 2 mints `EdgeKey`)" (`evaluator.rs:1190-1192`) and the runtime arm refuses
   non-reference operands with "edge referents ride slice 2" (`:1219-1223`).
   `materialize_neighbors` returns `Element::Node(id)` values only, discarding the traversed
   edge entirely (`query.rs:186-215`). `UNSERVED_EXPRESSION_HEADS` (`evaluator.rs:503-512`) lists
   `edges`/`edge-between`/`the` as slice 2. `SERVED_QUERY_HEADS` is exactly `["nodes","neighbors"]`
   (`:527`).

7. **CONFIRMATION — Row 7's BFS finding is the strongest claim in the report and it holds.**
   `query::materialize` (`query.rs:95-124`) serves exactly two heads, and **every** iterating form
   —`fold`, `exists`/`forall`, `select-max`/`select-min`, `for-each` — takes a single §2.6 query
   form as its operand and materializes it once before iterating (`evaluator.rs:968`,
   `structural_verbs.rs`'s `for-each` arm). There is no construct that re-queries mid-iteration
   and no first-class function value. Multi-hop reachability closure genuinely has no analog at
   any slice named in the plan. "A control-flow shape the current execution model does not admit
   at all" is the correct framing, and it is a stronger blocker than any of the storage gaps
   above it.

8. **CONFIRMATION — tick position and the persistent-handoff channel.** `position: ClassVar[float]
   = 14.5` (`faction_influence.py:53`), between `OODASystem` (14.0, `ooda.py:101`) and
   `DoctrineSystem` (14.7, `doctrine.py:626`) in `_SYSTEM_CLASSES` (`simulation_engine.py:328-364`).
   The `CollapseTransitionSystem`-only readership of `winning_faction_by_territory` /
   `secession_eligible` re-greps clean.

9. **CONFIRMATION — the RESERVED-LINE handling is correct and complete.** `colonial_stance` as
   Constitution I.1's principal-contradiction axis, the trap threshold, and the four seed
   factions' stance/`class_reduction` values are described as verified fact and not proposed
   upon. Independently checked against `src/babylon/data/game/balkanization/seed_factions.json`:
   `FAC_RESTORATIONIST` (uphold, 0.0), `FAC_WORKERS_CONGRESS` (ignore, 0.7), `FAC_DECOLONIAL`
   (abolish, 0.5), `FAC_LIBERAL_IMPERIAL` (ignore, 0.7) — the two-of-four "seeded to trip the
   diagnostic by construction" reading is exact. **Consequence of correction 1:** since these
   factions are seeded on five canonical scenarios, `RED_SETTLER_TRAP_DETECTED` (Row 4, the one
   PORTABLE row) has a live oracle — which makes Row 4 more, not less, worth landing.

**FINAL VERDICT: BLOCKED — the language-gap verdict stands unchanged and is if anything
understated (add the Task-12 deferred-shape-verb load gate to Row 8), but the "no conformance
oracle exists today regardless" leg is WITHDRAWN.** Only the RED_SETTLER_TRAP diagnostic is
portable; the argmax, the secession pipeline and the contiguous-region BFS remain blocked across
Q1/Q2/Q3/Q4/Q5 plus the two new findings (no `deffield` reference type; no BSL verb for the
`persistent_data` handoff), and the BFS still has no analog at any slice or ruling. What changes
is the *sequencing argument*: the five electoral goldens seed FACTION/SOVEREIGN/INFLUENCES/CLAIMS
on the byte gate today, so a port of Row 4 can be conformance-checked against the canonical estate
rather than a hand-built fixture, and the eventual full pack inherits a real oracle instead of
starting from zero.

**INADEQUATE-COVERAGE NOTE.** A re-read must add: (i) a per-computation liveness pass over the
five electoral goldens — which of the eight §6 computations actually fire there (the argmax
certainly does; `SECESSION_DECLARED` depends on `min_contiguous_hex_count=12` against those
scenarios' territory counts and on ADJACENCY edges the canonical factories declare they do not
emit, so Rows 5-8 may still be dark for a *different*, still-verifiable reason than the one
given); (ii) `apply_political_terrain` (`electoral_fixture.py:113-204`) to the FILE MAP as the
production seeding path this system's liveness actually depends on; (iii) a correction pass over
every "six scenarios" count in §5-§7 against the twelve-key `SCENARIOS` registry.
