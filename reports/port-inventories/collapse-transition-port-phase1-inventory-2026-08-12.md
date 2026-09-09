# CollapseTransitionSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `CollapseTransitionSystem` (313 lines, tick position 20.5, Consequences
phase) is structurally unlike every other system inventoried in this train: it contains **zero
arithmetic** (no `+`/`-`/`*`/`/`, no libm, no Real→Int demotion, no clamp) — it is a pure
graph-topology mutator that mints/deletes Sovereign nodes and CLAIMS edges. That shape inverts
the usual blocker profile: instead of query-evaluation or libm hazards, this system is blocked by
five *structural* gaps the query-evaluation train never touched — opaque vs. deterministic
node-ID minting, single-scalar edge storage vs. CLAIMS' five fields, no string/NodeRef field
types, no string `emit` payloads, and a `persistent_data` cross-system channel with no
groupby-over-population primitive. Both of its live-data paths are independently, verifiably
dormant on every canonical `qa:regression` scenario — for two distinct, code-confirmed reasons,
not merely the stale "no Sovereign nodes seeded" claim the estate's own coverage-gap ledger
states (5 of 12 canonical scenarios do seed real Sovereign/Faction/CLAIMS/INFLUENCES state).

**Verdict:** BLOCKED — five distinct structural gaps beyond the landed query-evaluation lane
(opaque node-ID minting; single-`f64` edge storage vs. CLAIMS' 5 fields — the named Slice 2/4
edge-attribute gap; no string/NodeRef field types; no string `emit` payloads; a
`persistent_data` channel with no groupby-over-population primitive even under the ruled R9 §3.6
carrier-node pattern) — but zero libm hazard, zero arithmetic hazard, and a strong hand-built
conformance-oracle substrate (`apply_balkanization_seed` + ~1,000 lines of law/integration tests)
already exists for whenever a future train takes this on.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/collapse_transition.py` | 313 | **The target.** `CollapseTransitionSystem`, three phases (`_collapse_sovereign`, `_execute_secession`, `_cleanup_orphaned_sovereigns`) plus a module-level helper `_extraction_policy_for_faction`. |
| `src/babylon/config/defines/balkanization.py` | 176 | `BalkanizationDefines` Pydantic model — this system reads exactly **one** of its ~20 fields: `initial_post_collapse_control_level` (lines 135-140, default 0.8, `[0,1]`). |
| `src/babylon/data/defines.yaml` | (balkanization block: lines 317-353; the one consumed key at 346) | Player-editable coefficient values. |
| `src/babylon/formulas/balkanization.py` | 404 | `derive_extraction_policy_from_stance` (lines 77-102) is the **only** function this system calls out of this ~10-function module (grep-confirmed: no other import from `formulas.balkanization` anywhere in `collapse_transition.py`). |
| `src/babylon/models/entities/sovereign.py` | 118 | `Sovereign` Pydantic entity — field types/domains for every attribute this system writes onto a newly-minted Sovereign node; also carries model-level validators (`_validate_null_ruling_only_with_continue`, `_validate_dissolution_after_founding`) that graph writes bypass entirely (§3). |
| `src/babylon/models/entities/balkanization_faction.py` | 86 | `BalkanizationFaction` entity — `colonial_stance: ColonialStance` (line 66) is the field `_extraction_policy_for_faction` reads off a Faction node. |
| `src/babylon/models/enums/balkanization.py` | 185 | `ColonialStance` (33-51), `ExtractionPolicy` (54-70), `SovereigntyType` (73-87, unused by this system as a symbol but its raw string values ARE written), `ClaimLegalStatus` (115-134, **not imported** by this system despite its `DISPUTED` member being exactly what one code comment claims to write — §2/§4 defect). |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.SOVEREIGN` (66), `NodeType.FACTION` (67), `NodeType.TERRITORY` (61); `EdgeType.CLAIMS` (125), `EdgeType.ADMINISTERS` (127) — ADMINISTERS is named in the class docstring but never written/queried by this file (grep-confirmed zero hits for "administers" in `collapse_transition.py`). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase._wrap_graph` (99-117) — the **only** `SystemBase` helper this system calls. It does **not** call `_write_clamped`, `_read`, or `_publish` (it calls `services.event_bus.publish` directly, bypassing the `_publish` shorthand every other inventoried system used — a minor, verbatim style inconsistency). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.query_nodes` (258), `.get_node` (77), `.add_node` (62), `.add_edge` (115), `.remove_edge` (172), `.update_edge` (152), `.remove_node` (100), `.query_sovereign_claims` (397-413, spec-070-specific), `.bulk_partition_claims` (450-471, spec-070-specific, O(K) contract). |
| `src/babylon/topology/graph.py` | 1033 (relevant: 165-260, 651-720, 923-1001) | Concrete `BabylonGraph` — `add_node`/`update_node`/`add_edge`/`update_edge` are all **raw dict merges, no Pydantic coercion or validation** (§3, same pattern the Territory inventory documented); `query_sovereign_claims` (923-940, sorted control-desc/id-asc); `bulk_partition_claims` (974-1001, delete+`add_edge` per territory, **carrying the OLD edge payload wholesale** — §2 defect). |
| `src/babylon/engine/context.py` | 113 | `TickContext.persistent_data: dict[str, Any]` — the untyped, free-form dict that is the **sole** channel for `balkanization.collapse_triggers`/`.winning_faction_by_territory`/`.secession_eligible` (§5, the system's primary blocker). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` (28-30). |
| `src/babylon/kernel/event_bus.py` | 288 (relevant: 33-56) | `Event` frozen dataclass — `type: str`, `payload: dict[str, Any]`. |
| `src/babylon/models/enums/events.py` | 234 (relevant: 149-153) | `EventType.SOVEREIGN_COLLAPSE` (149), `.TERRITORY_TRANSITION` (150), `.CIVIL_WAR_DECLARED` (153) — the three distinct emissions (§2f). |
| `src/babylon/engine/simulation_engine.py` | 611 (relevant: 328-364) | `_SYSTEM_CLASSES` confirms registry position 20.5, between `FieldDerivativeSystem` (20.0) and `EdgeTransitionSystem` (21.0); `FactionInfluenceSystem` (14.5) is the sole upstream writer of the two live `persistent_data` keys this system reads (§5). |
| `src/babylon/engine/observers/endgame_detector.py` | 812 | **Not a System** — a `SimulationObserver` invoked outside `_SYSTEM_CLASSES` (grep-confirmed zero references to it in `simulation_engine.py`). Four of its five terminal-outcome axes (`_axis_revolutionary_victory`, `_axis_fascist_consolidation`, `_axis_red_ogv`, `_axis_fragmented_collapse`) read Sovereign-node state this system is the sole writer/destroyer of (§5, the SPECIAL NOTES boundary). |
| `src/babylon/engine/scenarios/balkanization_seed.py` | 163 | `apply_balkanization_seed` — the one real hand-built Sovereign/Faction/CLAIMS/INFLUENCES seeding function in the engine tree; self-documents "NOT applied by any of the six qa:regression scenario factories" (lines 26-28) but IS applied, via `electoral_fixture.py`, to 5 of the 12 canonical scenarios (§5, a correction to the naive dormancy read). |
| `src/babylon/data/game/balkanization/seed_sovereigns.json` | 87 | The concrete seed data `apply_balkanization_seed` loads: `SOV_USA_FED` (legitimacy 1.0), `SOV_CAN_FED` (0.85), `SOV_EXTERIOR_NULL` (**0.0** — but code-exempted from the collapse predicate, §5). |
| `tools/regression_scenarios.py` | 2925 (relevant: 37-152 `SCENARIOS`, 2841-2847 the `CollapseTransitionSystem` `COVERAGE_GAPS_DATA` row) | The canonical-scenario registry and its declared coverage-gap ledger — literally correct that no SOVEREIGN nodes are seeded on 7 of 12 scenarios, but **imprecise** for the other 5 (§5). |
| `src/babylon/sentinels/defines_passthrough/registry.py` | 176 | The repo's own "defines-passthrough bug class" sentinel — documents the EXACT anti-pattern this file exhibits twice (bare `BalkanizationDefines()` construction bypassing `services.defines`) but its `WATCHED_FUNCTIONS` registry only covers formulas-layer function calls missing a `defines=` kwarg, not a System's own bare class instantiation — this file's two instances are outside its scope by construction, not by a caught-and-cleared check (§2 defect). |

**Not exercised by `collapse_transition.py` at all:** no `src/babylon/domain/*` module. Two
`formulas`/`config.defines` imports are runtime-local (inside method bodies, lines 120, 218,
300-301), not module-level — avoiding an import cycle the module docstring doesn't explain but
which mirrors the same pattern `models/entities/sovereign.py`'s `metabolic_impact` computed_field
uses for the identical reason (models must not import formulas at module level).

**Reference material read for this inventory (fully read or read in the cited ranges):**
- `docs/reference/bsl-language.rst` §2.8 (Effects — the typed structural verbs, lines 1321-1520),
  §3.6 (Closed vocabulary / the R9 chapter C3 graph-scope-state ruling, lines 2639-2688), the
  no-string-in-the-language ruling (lines 456-489, 2889-2893).
- `rust/crates/babylon-bsl/src/structural_verbs.rs` lines 1-60 (module doc: opaque id-minting,
  single-`f64` edge storage, `update-edge`/`update-hyperedge` "recognised and refused loudly"),
  843-891 (`add_node`'s actual implementation).
- `rust/crates/babylon-graph/src/substrate.rs` lines 1-130 (`GraphSubstrate` trait: `NodeId(u64)`
  opaque identity, `add_edge`'s mandatory single `strength: f64`, `remove_node`'s cascade
  semantics per ADR185 R2, "absence is never success").
- `rust/crates/babylon-bsl/src/declarations.rs` line 110 (`DECLARABLE_INTRINSICS`).
- The sibling `sovereignty-port-phase1-inventory-2026-08-12.md` and
  `territory-port-phase1-inventory-2026-08-11.md` in this same directory (cross-checked for the
  shared `BalkanizationDefines`/canonical-scenario-coverage findings, §5).

## 2. COMPUTATION CATALOG (execution order, `step()`, `collapse_transition.py:59-104`)

### Phase 1 — Collapse-driven detection + partition (`step()` lines 69-90, delegating to `_collapse_sovereign`, lines 106-201)

- **(a)** For every Sovereign (except the exterior-boundary fallback), if legitimacy has hit
  zero or an external trigger names it, the Sovereign collapses: its claimed Territories are
  partitioned among whichever Factions won each one (creating a new successor Sovereign per
  distinct winning Faction), and its old CLAIMS edges are stripped.
- **(b)** Predicate: `legitimacy = float(sov_node.attributes.get("legitimacy", 1.0))` (line 84);
  `trigger = triggers.get(sovereign_id)` (85), `if trigger is None and legitimacy <= 0.0: trigger
  = "legitimacy_zero"` (86-87) — comparison only, no arithmetic. Partition: `by_faction:
  dict[str, list[str]]` built by grouping `claims` (from `query_sovereign_claims`) by
  `winning.get(territory_id)` (141-149) — a pure dict-groupby, no math. New-sovereign ID:
  `new_sov_id = f"SOV_AUTO_T{tick}_F{faction_id.removeprefix('FAC_')}_{counter}"[:64]`
  (153-155) — string interpolation, not arithmetic (`counter` from `enumerate(...,
  start=1)`, an integer, never used in arithmetic). CLAIMS-edge write per assigned territory
  carries `control_level=defines.initial_post_collapse_control_level` (174) — a direct
  define-passthrough, no computation.
- **(c) Reads:** `NodeType.SOVEREIGN` nodes (all, via `query_nodes`); each Sovereign's
  `legitimacy` (default 1.0 if absent); `persistent_data["balkanization.collapse_triggers"]`
  (dict, **provably dead in production**, §5); `persistent_data["balkanization.
  winning_faction_by_territory"]` (dict, live, written by `FactionInfluenceSystem`@14.5, §5);
  `query_sovereign_claims(sovereign_id)` → CLAIMS edges (territory_id, control_level,
  legal_status) of the collapsing Sovereign; for each new successor, the winning Faction node's
  `colonial_stance` (via `_extraction_policy_for_faction`, below).
- **(d) Writes:** new `SOVEREIGN` node(s) via `add_node` — `name`, `sovereignty_type=
  "provisional"`, `legitimacy=0.5`, `color_hex="#7f7f7f"`, `ruling_faction_id`,
  `extraction_policy`, `founded_tick=tick` (157-168); new `CLAIMS` edges from each successor to
  its assigned territories — `control_level`, `legal_status="de_facto"`, `fiscal_status=
  "taxed"`, `recognition_level=0.5`, `claimed_since_tick=tick` (169-179, **all five fields
  freshly stamped**, contrast Phase 2 below); removes the collapsed Sovereign's original CLAIMS
  edges, one per claim, unconditionally (`remove_edge`, 200-201, `contextlib.suppress(KeyError)`
  belt-and-suspenders — the edge was just queried as present, so this never actually fires on
  the live code path).
- **(e) Defines:** `balkanization.initial_post_collapse_control_level` (0.8, `[0,1]`,
  `defines.yaml:346`) — the **only** define this entire system reads.
- **(f) Events:** `SOVEREIGN_COLLAPSE` (once per collapsing Sovereign, 124-135);
  `TERRITORY_TRANSITION` (once per claimed Territory, in `sorted(claims, key=lambda r: r[0])`
  order — i.e., re-sorted by territory ID ascending, ignoring `query_sovereign_claims`'s own
  control-level-desc sort, 182-199).

**Defect, transcribed verbatim (port-as-is law):** the class docstring (lines 6-12) frames the
sequence as "…creates new Sovereigns…, emits `TERRITORY_TRANSITION`…, deletes the collapsed
Sovereign node + outbound edges" as if node deletion happens in this phase. It does not: Phase 1
only strips CLAIMS edges (`remove_edge`); the Sovereign node itself is deleted later, in Phase 3,
conditional on having zero remaining claims (which is always true by the end of Phase 1, since
every claim was just stripped — so the docstring's end-state description is accurate, just
executed one phase later than it reads).

**Determinism-inconsistency, transcribed verbatim:** `step()`'s own Sovereign scan is explicitly
`sorted(node.id for node in wrapped.query_nodes(node_type=NodeType.SOVEREIGN))` (72-74), but
Phase 3's scan (below) is **not** sorted — `[node.id for node in wrapped.query_nodes(...)]`
(280), relying on the concrete `BabylonGraph`'s insertion-order iteration rather than an
explicit lex sort. The class docstring's "Determinism notes" (lines 24-31) name sorted
Sovereign-ID order for the collapse predicate and sorted-ID order for `TERRITORY_TRANSITION`
emission, but say nothing about Phase 3's iteration order — an unstated (though functionally
harmless, since CLAIMS edges are strictly Sovereign→Territory, never Sovereign→Sovereign, so
orphan-removal order cannot change the final graph state) asymmetry worth a D-record if ported,
since a BSL `nodes` query's own iteration-order guarantee may not match either convention
exactly.

### Phase 2 — Active-secession execution (`step()` lines 92-95, delegating to `_execute_secession`, lines 203-273)

- **(a)** A (Faction, parent-Sovereign, contiguous-Territory-set) tuple that has already cleared
  FactionInfluenceSystem's hysteresis window becomes a new breakaway Sovereign: the named
  Territory subregion's CLAIMS edges are rewired parent→breakaway via one O(K) batch operation,
  and a `CIVIL_WAR_DECLARED` event fires.
- **(b)** `territories = {str(tid) for tid in entry["contiguous_territory_ids"]}` (223); if
  empty, early-return (224-225, no-op, no event). New Sovereign ID: `f"SOV_BREAK_T{tick}
  _F{faction_id.removeprefix('FAC_')}"[:64]` (227-228) — same string-interpolation shape as
  Phase 1. `moved = wrapped.bulk_partition_claims(from_sovereign_id=parent_id,
  to_sovereign_id=new_sov_id, territories=territories)` (244-248) — an O(K) protocol-level call,
  not decomposed in this System's own source; the concrete implementation
  (`topology/graph.py:974-1001`) is delete-then-`add_edge` per territory, **carrying the OLD
  edge payload dict wholesale** (`edge_data = dict(self._edge_payload[key]); …
  self.add_edge(to_sovereign_id, territory_id, **edge_data)`, graph.py:995-999) — so the newly
  rewired edge initially retains the PARENT's original `control_level`/`legal_status`/
  `fiscal_status`/`recognition_level`/`claimed_since_tick` verbatim. A follow-up loop then
  `update_edge`s **only two** of those five fields per territory —
  `legal_status="de_facto"`, `control_level=defines.initial_post_collapse_control_level`
  (265-273).
- **(c) Reads:** `persistent_data["balkanization.secession_eligible"]` (list of dicts, live,
  written by `FactionInfluenceSystem`@14.5, §5); the seceding Faction's `colonial_stance` (via
  `_extraction_policy_for_faction`).
- **(d) Writes:** one new `SOVEREIGN` node (`sovereignty_type="secessionist"`, `legitimacy=0.5`,
  `color_hex="#ff7f00"`, `ruling_faction_id`, `extraction_policy`, `founded_tick=tick`, 229-239);
  CLAIMS edges rewired parent→new via `bulk_partition_claims`, then `update_edge`-refreshed on
  exactly `legal_status`/`control_level`.
- **(e) Defines:** `balkanization.initial_post_collapse_control_level` (same single define as
  Phase 1, second use site, line 272).
- **(f) Events:** `CIVIL_WAR_DECLARED` (once, 250-261).

**Defect, transcribed verbatim (port-as-is law), high-confidence:**

1. **Stale-attribute carryover.** `fiscal_status`, `recognition_level`, and `claimed_since_tick`
   on the rewired CLAIMS edge are the **parent Sovereign's pre-secession values**, never reset —
   contrast Phase 1's new-successor CLAIMS edges, which stamp all five fields fresh
   (`recognition_level=0.5`, `claimed_since_tick=tick`, explicitly). No test in the estate
   (`tests/integration/balkanization/test_us4_secession_fracture.py`, read in full, §7) asserts
   on any of these three post-secession values — the discrepancy is untested and would be
   silently transcribed by a byte-faithful port unless flagged.
2. **Comment/code mismatch on `legal_status`.** The comment at lines 262-264 reads: "Promote the
   new edges' legal_status to **disputed** for FR-028's contested-boundary semantics." The code
   at line 271 writes the literal string `"de_facto"` — the *same* value Phase 1 uses for an
   uncontested partition, not `"disputed"`. `ClaimLegalStatus.DISPUTED = "disputed"`
   (`models/enums/balkanization.py:134`) is a real, declared, five-member-enum value (`DE_JURE`,
   `DE_FACTO`, `DISPUTED`, `OCCUPIED`, `CEDED`; also the exact CHECK-constraint domain in
   `persistence/migrations/0025_balkanization.sql:92-94`) that this code never imports or
   writes. Either the comment is stale documentation or the code under-implements FR-028's
   stated "contested-boundary semantics" — either way, port-as-is law requires transcribing the
   **code's actual behavior** (`"de_facto"`), not the comment's stated intent, with a D-record
   naming the discrepancy rather than silently "fixing" it to `"disputed"` during the port.

### Phase 3 — Orphaned-Sovereign cleanup (`step()` lines 97-98, delegating to `_cleanup_orphaned_sovereigns`, lines 276-290)

- **(a)** Any Sovereign left with zero outgoing CLAIMS edges (other than the exterior-boundary
  fallback) is deleted outright.
- **(b)** `sovereign_ids = [node.id for node in wrapped.query_nodes(node_type=NodeType.
  SOVEREIGN)]` (280, **not sorted**, see the Phase-1 determinism note above); for each,
  `claims = wrapped.query_sovereign_claims(sovereign_id); if claims: continue` (282-284); the
  `SOV_EXTERIOR_NULL` exemption repeats here (287-288); `wrapped.remove_node(sovereign_id)`
  (290, `contextlib.suppress(KeyError)`).
- **(c) Reads:** `NodeType.SOVEREIGN` nodes; each one's outgoing CLAIMS-edge count (via
  `query_sovereign_claims`).
- **(d) Writes:** deletes zero-claims Sovereign nodes. The class docstring (277-278) says this
  deletes "the corresponding ADMINISTERS edges" too, but the **code itself never names
  ADMINISTERS** — that behavior, if it occurs, is entirely inherited from the underlying
  `BabylonGraph.remove_node`'s generic incident-edge cascade (`topology/graph.py:192-217`, which
  removes every edge touching the node regardless of type), not from any explicit logic in this
  method. A favorable structural note for the port: BSL's own `GraphSubstrate::remove_node` is
  independently documented as cascading (`substrate.rs:82-99`, ADR185 R2) — the two systems
  agree on this specific point even though this Python method's docstring over-states its own
  role in producing it.
- **(e) Defines:** none.
- **(f) Events:** none — this is the only one of the three phases that emits nothing.

### Housekeeping — single-shot input clearing (`step()` lines 100-104)

`persistent["balkanization.collapse_triggers"] = {}`; `persistent["balkanization.
secession_eligible"] = []`. Pure Python-dict bookkeeping with **no graph interaction whatsoever**
— guards its own idempotency for next tick (so a trigger fired once is not re-processed). Has no
BSL analog because the mechanism it clears (`persistent_data`) itself has none (§5/§6).

**Events emitted by the whole system: three distinct `EventType` values**
(`SOVEREIGN_COLLAPSE`, `TERRITORY_TRANSITION`, `CIVIL_WAR_DECLARED`) — confirmed by grep, zero
other `EventType`/`.publish(` sites in the file.

## 3. TYPE INVENTORY

Runtime storage note (load-bearing, same finding as the Territory/Sovereignty inventories):
`BabylonGraph.add_node`/`.update_node`/`.add_edge`/`.update_edge` (`topology/graph.py:165-260,
660-670, 690-720`) are **plain dict merges with no type coercion or Pydantic validation**. The
`Sovereign` Pydantic model's own validators — `_validate_null_ruling_only_with_continue`
(`ruling_faction_id is None` requires `extraction_policy == CONTINUE`) and
`_validate_dissolution_after_founding` — are therefore **never actually checked** against what
this System writes to the graph; they would only fire if a `Sovereign(**attrs)` were
re-instantiated from the raw dict, which no code path here does.

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `legitimacy` | SOVEREIGN | `Probability` (`sovereign.py:63`) | `[0.0, 1.0]` | unit-interval — **read-only by this system in production** (§5: nothing writes it after seed time) |
| `sovereignty_type` | SOVEREIGN | `SovereigntyType` (StrEnum, 6 members) | closed set | **Enum discriminant** — written as a bare string literal (`"provisional"`/`"secessionist"`), never via the enum symbol |
| `color_hex` | SOVEREIGN | `str`, pattern `^#[0-9A-Fa-f]{6}$` | fixed-format string | **string, not a numeric/enum field** — no `deffield` type covers this |
| `ruling_faction_id` | SOVEREIGN | `str \| None`, pattern `^FAC_[A-Z][A-Z0-9_]*$` | a foreign-key-shaped node-ID string | **NodeRef-as-stored-attribute** — no `deffield` type covers this either |
| `extraction_policy` | SOVEREIGN | `ExtractionPolicy` (StrEnum, 3 members) | closed set | **Enum discriminant**, deterministically derived from `colonial_stance` |
| `founded_tick` | SOVEREIGN | `int`, `ge=0` | `[0, ∞)` | integer, tick-valued |
| `dissolved_tick` | SOVEREIGN | `int \| None`, `ge=0` | `[0, ∞)` or absent | integer — **declared on the model, never written by this system's own deletion path** (§2 Phase 3 note; a hard `remove_node` erases the node before any dissolution tick could be stamped, so the field is permanently unpopulated for any Sovereign that goes through orphan cleanup — the projection-layer renderer already self-flags this as a gap, `projection/vault/render.py:156`: `"Investigate(Sovereign) to attribute a dissolution tick"`) |
| `colonial_stance` | FACTION | `ColonialStance` (StrEnum, 3 members) | closed set | **Enum discriminant, RESERVED-LINE** (the settler-colonialism principal axis — see below) — read-only here |
| `control_level` | CLAIMS edge | `Relationship.control_level`-shaped float, per migration `CHECK (control_level BETWEEN 0 AND 1)` | `[0,1]` | unit-interval, edge-scoped |
| `legal_status` | CLAIMS edge | `ClaimLegalStatus` (5-member StrEnum) per migration CHECK, but the Python `Relationship` model types it as plain `str \| None` (`relationship.py:140`) — **the enum is declared but not wired to the field's type**, matching the Sovereignty inventory's identical finding | closed 5-set at the DB layer, unenforced at the Python model layer | enum-shaped string, written as bare literals here |
| `fiscal_status` | CLAIMS edge | same untyped-string pattern | closed 5-set at DB layer (`taxed`/`revolt`/`blockade`/`liberated`/`occupied`) | enum-shaped string, only ever written `"taxed"` by this system |
| `recognition_level` | CLAIMS edge | float, DB `CHECK BETWEEN 0 AND 1`, DB default `1.0` | `[0,1]` | unit-interval, edge-scoped |
| `claimed_since_tick` | CLAIMS edge | int, DB `CHECK >= 0` | `[0, ∞)` | integer, tick-valued, edge-scoped |
| `initial_post_collapse_control_level` (define) | — | `float`, `ge=0.0, le=1.0` | `[0,1]` | unit-interval coefficient |

**RESERVED-LINE flag.** `ColonialStance` (`UPHOLD`/`IGNORE`/`ABOLISH`) and its deterministic
mapping to `ExtractionPolicy` (`INTENSIFY`/`CONTINUE`/`CEASE`, via
`derive_extraction_policy_from_stance`) are the settler-colonialism principal-contradiction axis
this codebase's Constitution names explicitly (`ColonialStance`'s own docstring: "The principal
contradiction in MLM-TW analysis... settler colonialism vs anti-settler liberation";
`babylon/CLAUDE.md`'s "National Question RULED — ADR171"). This system does not compute or
choose a stance — it only *reads* an already-seeded Faction's stance to derive a new Sovereign's
policy — but the mapping itself, and every successor-Sovereign field this system stamps from it,
is ideological content under Director authority. Described here, not touched, and not proposed
for change.

**Enum-discriminant flags (four distinct fields, all landed as expressible per ADR195/196):**
`sovereignty_type`, `extraction_policy`, `colonial_stance` are all closed StrEnums with the
BSL `enum` `deffield` type now available (landed, unlike the Territory inventory's pre-landing
finding). `legal_status`/`fiscal_status` are enum-**shaped** strings whose Python model never
actually types them as the declared enum (a pre-existing, independently-discovered defect this
inventory shares with the Sovereignty inventory, not new here).

**Unbounded-real / bool flags:** none found — every numeric field this system touches is
domain-bounded `[0,1]` or a non-negative integer (`founded_tick`/`claimed_since_tick`/
`dissolved_tick`). No bool-typed attribute is read or written anywhere in this file.

**String / NodeRef-as-attribute flags (two, both hard, both new relative to every prior
inventory in this train):** `color_hex` (a fixed-format string, no numeric encoding attempted
anywhere in the estate) and `ruling_faction_id` (a stored cross-reference to another node's ID,
not a transient query result) — see §6.

## 4. FLOAT-OP INVENTORY

**Zero arithmetic operators anywhere in this file** — grep-confirmed no `+`, `-`, `*`, `/`
outside string f-strings, and zero `exp`/`log`/`pow`/`sigmoid`/`math.` calls. This is a
qualitatively different shape from every other system inventoried in this train (Territory,
Sovereignty, Metabolism, …): `CollapseTransitionSystem` is a pure graph-topology mutator, not a
numeric formula evaluator. The only numeric operations present:

1. **Type cast:** `float(sov_node.attributes.get("legitimacy", 1.0))` (line 84) — a coercion,
   not a computation.
2. **Threshold comparison:** `legitimacy <= 0.0` (line 86) — one comparison, bare `0.0` literal.
3. **Bare non-integer literals (four sites):** `0.0` (line 86, comparison); `legitimacy=0.5`
   (lines 163, 235, both bare-literal node-field writes — not computed); `recognition_level=0.5`
   (line 177, bare-literal edge-field write). Each of these would need the same `c`-suffixed
   `0.5c`/`0.0c` treatment (or the Real-zero-promotion idiom) the Territory/Metabolism inventories
   already flagged for BSL's "no bare non-integer literal" parser rule — trivial individually,
   listed for completeness.
4. **Define passthrough, not computation:** `control_level=defines.
   initial_post_collapse_control_level` (lines 174, 272) — a direct read-and-stamp, no
   arithmetic performed on it.
5. **Integer counter, non-arithmetic use:** `enumerate(sorted(by_faction.items()), start=1)`
   (line 152) — `counter` is used only inside an f-string (ID construction), never in a
   numeric expression.
6. **String slicing (not float-op, noted for completeness):** `new_sov_id[:64]` (lines 155, 228)
   — a length cap on the constructed ID, integer index, no numeric domain relevance.

**No clamps at all** — unlike Territory (`_write_clamped` / hand-written `min(1.0, …)`) or
Metabolism, this system never calls `SystemBase._write_clamped` and has no hand-written
`max`/`min` clamp anywhere; every numeric field it writes is either a bare literal already inside
its declared domain (`0.5` for `legitimacy`/`recognition_level`, both `[0,1]`) or a
define-passthrough already validated at the Pydantic-model level (`ge=0.0, le=1.0` on
`initial_post_collapse_control_level`). **No Real→Int demotion anywhere** (no `int(...)` cast on
a float anywhere in the file — the only integer values, `tick`/`counter`, are already integers at
their source).

**Conclusion: zero libm-nondeterminism hazard, zero clamp-inconsistency hazard, zero
Real→Int-demotion hazard.** The entire float-op surface of this system is four bare literals and
one comparison. Every real blocker for this system lives in the STRUCTURAL verb/storage surface
(§6), not the arithmetic surface.

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 20.5** (`collapse_transition.py:54`), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328-364`): `... → ContradictionFieldSystem → FieldDerivativeSystem
  (20.0) → CollapseTransitionSystem (20.5) → EdgeTransitionSystem (21.0) → WealthDistribution
  System → EpistemicHorizonSystem`.
- **Reads from a same-tick prior system: two live persistent-data channels, one dead one.**
  `FactionInfluenceSystem` (14.5) is the **sole writer**, this same tick, of
  `persistent_data["balkanization.winning_faction_by_territory"]`
  (`faction_influence.py:71`) and `persistent_data["balkanization.secession_eligible"]`
  (`faction_influence.py:251`) — both grep-confirmed written nowhere else in `src/babylon/`.
  `persistent_data["balkanization.collapse_triggers"]` is read (and cleared) by this system but
  **written nowhere in production `src/babylon/`** at all — grep across the entire source tree
  finds it only in this file (read + clear) and in the unit-test harness
  (`tests/unit/balkanization/test_collapse_transition_system.py:134,166`). This is the same
  "provably-uniform, test-only override" shape the Territory inventory documented for
  `TickContext.displacement_mode` — the honest port declares it dead and drops the override
  machinery, recording why. No graph-node/edge read from `VitalitySystem` through
  `FieldDerivativeSystem` (positions 1.0-20.0) overlaps this system's reads at all — its only
  upstream dependency is the two `FactionInfluenceSystem` persistent-data keys.
- **Writes consumed downstream: cross-tick only, plus one same-tick post-tick observer.**
  Grep across every `src/babylon/engine/systems/*.py` for `NodeType.SOVEREIGN`/`"sovereign"`
  finds exactly four files: this one, `sovereignty.py`, `faction_influence.py`, `electoral.py`
  — and all three of the others run **earlier** in the tick (positions well before 20.5), so
  within a single tick nothing downstream (`EdgeTransitionSystem`@21, `WealthDistributionSystem`,
  `EpistemicHorizonSystem`) ever reads a Sovereign-type node (grep-confirmed zero hits in
  `edge_transition/`, `wealth_distribution.py`, `epistemic_horizon.py`). The Sovereign
  population and CLAIMS-edge graph this system mutates therefore only becomes visible to other
  Systems on the **next tick**, when `SovereigntySystem`/`FactionInfluenceSystem`/
  `ElectoralSystem`/this-system-itself run again. The one **same-tick** consumer is the
  `EndgameDetector` observer (below).
- **The system/observer boundary (SPECIAL NOTES).** `EndgameDetector`
  (`engine/observers/endgame_detector.py`) is a `SimulationObserver`, not a `System` — it is
  never listed in `_SYSTEM_CLASSES`/`_DEFAULT_SYSTEMS` (grep-confirmed) and its own docstring
  states it "receives state change notifications but cannot modify simulation state." It runs
  post-tick, reading the tick's final `WorldState`/`BabylonGraph`. Four of its five terminal
  outcomes read state this system is the sole writer or destroyer of:
  - **FRAGMENTED_COLLAPSE** (`_axis_fragmented_collapse`, lines 579-640): counts surviving
    `sovereign` nodes (`survivor_count`, 600-605) and checks whether any carries
    `sovereignty_type` in `{insurgent, occupation, emergency}` (`has_crisis`, 610-611) — this
    system is the only writer of new `sovereignty_type` values (`"provisional"`,
    `"secessionist"`) and the only destroyer of Sovereign nodes (orphan cleanup), so it directly
    drives both this axis's survivor-count and crisis-type gates.
  - **REVOLUTIONARY_VICTORY** (lines ~380-447), **FASCIST_CONSOLIDATION**'s political-violence
    route (lines ~500-549), and **RED_OGV** (lines 551-577) all gate on `_has_stance_majority`/
    `_aggregate_extraction_policy_is`, which resolve each Sovereign's `ruling_faction_id` →
    Faction's `colonial_stance` (`_lookup_sovereign_stance`, lines 794-812) and each Sovereign's
    own `extraction_policy` — both fields this system is the sole stamper of on every new
    successor/breakaway Sovereign.
  - **ECOLOGICAL_COLLAPSE** is the one axis independent of Sovereign state (habitability/
    biocapacity only).
  This means `CollapseTransitionSystem`, despite never itself computing an outcome, is a direct
  upstream determinant of 4 of the 5 terminal-outcome axes via the Sovereign-node population and
  field values it alone mutates.
- **Context/service usage with no BSL equivalent:** `context.persistent_data` (untyped
  `dict[str, Any]`, `engine/context.py:49`) is the **entire** mechanism by which this system
  learns which Sovereign to collapse-with-what-trigger and which Faction/Territory sets are
  secession-eligible. There is no query-lane or field-storage equivalent for "a dict computed by
  a different System earlier this tick" in the current BSL surface (§6).
- **DORMANCY on canonical scenarios — a nuanced, verified finding, not the ledger's literal
  claim.** `tools/regression_scenarios.py:2841-2847`'s own `COVERAGE_GAPS_DATA` entry for
  `CollapseTransitionSystem` states "no SOVEREIGN nodes are seeded" — **this is imprecise**: 5 of
  the 12 canonical `SCENARIOS` (`mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`, all via
  `create_*_scenario` → `apply_political_terrain` → `apply_balkanization_seed`,
  `electoral_fixture.py:204`) DO seed three real Sovereign nodes with real CLAIMS edges from
  `seed_sovereigns.json` (§1) — the same correction the sibling `SovereigntySystem` inventory
  makes for its own system. **The functional conclusion nonetheless holds, for two independently
  verified, more precise reasons:**
  1. **Collapse-driven path:** `legitimacy` is stamped only at node-creation time (scenario seed,
     or by this system itself on a successor Sovereign) and **never decremented by any System in
     production** (grep across every `engine/systems/*.py` for a `legitimacy=` write finds only
     scenario/fixture files and this system's own two node-creation sites — never an
     `update_node(..., legitimacy=...)` call anywhere). The only seeded Sovereign at `legitimacy
     0.0` is `SOV_EXTERIOR_NULL` (`seed_sovereigns.json`), which this system's own code
     explicitly exempts from the collapse predicate (line 79-80). So even on the 5 scenarios that
     seed real Sovereigns, `legitimacy <= 0.0` can never fire.
  2. **Active-secession path:** `secession_eligible` population ultimately depends on
     `contiguous_influence_majority_subregion` (`formulas/balkanization.py:255-296`), whose
     `_largest_contiguous_component` (299-337) is an explicit "BFS over ADJACENCY edges" that can
     never exceed a 1-node component if zero ADJACENCY edges exist, and must reach
     `min_contiguous_hex_count` (12) to return non-empty. Grep across
     `single_county.py`/`electoral_fixture.py`/`balkanization_seed.py` finds **zero** ADJACENCY
     references — confirming, independently, the sibling Territory inventory's own finding
     ("the canonical `SCENARIOS` factories… emit NO ADJACENCY edges… no real county-adjacency
     reference source exists at seeding time"). So `secession_eligible` structurally cannot
     populate on any canonical scenario, regardless of Faction/INFLUENCES seeding.
  **Net result:** `SOVEREIGN_COLLAPSE`/`CIVIL_WAR_DECLARED`/`TERRITORY_TRANSITION` never fire on
  any of the 12 canonical scenarios — the ledger's bottom-line claim is correct, its stated
  reason ("no SOVEREIGN nodes are seeded") is only true for 7 of the 12. Phase 3 (orphan
  cleanup) is the one sub-computation that DOES get real, non-trivial exercise on the 5
  balkanization-seeded scenarios: its query loop runs against three real Sovereigns every tick
  and always finds non-empty claims (a genuine no-op outcome, not an unreached code path — a
  meaningfully different dormancy shape than Phase 1/2's total inertness).
  A port's conformance fixtures will need to be hand-built regardless (as the estate's own
  `apply_balkanization_seed` + `tests/unit/balkanization/`/`tests/integration/balkanization/`
  suite already demonstrates, §7) — nothing is lost, and much is already in hand, for a future
  train.

## 6. BLOCKER ASSESSMENT

| Computation | Verdict | Detail |
|---|---|---|
| Phase 1 — collapse predicate scan (`legitimacy <= 0.0` / external trigger, lines 69-90) | **PORTABLE WITH D-RECORD** | `nodes` query over SOVEREIGN + `field-of` for `legitimacy` are both landed (Slice 1). The `SOV_EXTERIOR_NULL` string-ID exemption has no BSL equivalent (opaque `NodeId(u64)` minting means no rule can compare a node's identity against a fixed human-readable string) — D-record: replace with a dedicated `is_exterior_fallback bool` field, stamped once at seed time. `collapse_triggers` itself is additionally provably dead in production (§5) — D-record: drop the override, declare `:const`-away, Territory-`displacement_mode`-style. |
| Phase 1 — CLAIMS partition + successor-Sovereign minting (`by_faction` grouping, `add_node`, `add_edge`, lines 106-179) | **BLOCKED — three compounding structural gaps** | (1) **Opaque node-ID minting**: `add-node`'s id operand is an effect-list-scoped local symbol, not a caller-chosen persistent string (`structural_verbs.rs:28-32,843-891`; `GraphSubstrate::add_node` returns `NodeId(u64)`, `substrate.rs:30-33,80`) — the frozen system's deterministic `SOV_AUTO_T{tick}_F{faction}_{counter}` naming law (docstring line 29) has no BSL representation at all, and no string-concatenation primitive exists to build one even if it did (`bsl-language.rst:459`: "no string concatenation, comparison, or interpolation in the language"). (2) **String/NodeRef field types**: `color_hex` (a hex-string) and `ruling_faction_id` (a stored cross-reference) are both outside `deffield`'s closed vocabulary (`int/bool/currency/probability/intensity/coefficient/enum`). (3) **Grouping over `persistent_data`**: `winning_faction_by_territory` is a Python dict from a different System's `persistent_data` write, not graph state — R9 chapter C3 (`bsl-language.rst:2650-2688`) rules a carrier-node/ordinary-field representation exists *in principle*, but the "one new Sovereign per distinct winning Faction" computation additionally needs a groupby/distinct-values-of-a-field primitive that no landed query head (`fold`/`select-max`/`select-min`/`exists`/`forall` over typed neighbors) provides, and requires re-modeling the open `NodeType.FACTION` population as a closed, statically-declared enum first — a real content-modeling redesign, not a passthrough. |
| Phase 1 — SOVEREIGN_COLLAPSE / TERRITORY_TRANSITION emission (lines 124-135, 182-199) | **BLOCKED — string emit payloads** | `emit` is a landed verb, but every payload field here (`trigger`, `sovereign_id`, `from_sovereign_id`, `to_sovereign_id`, `reason`) is a string; `bsl-language.rst:2889-2893` rules "no string payloads on emit" as closed — `Str` has no operations and `<expr>` has no string literal at all. `trigger` is additionally an *open-ended* string (arbitrary external-trigger names per the docstring), not reducible to a small closed enum without its own content-modeling ruling. Independent of the payload problem, `TickReport` itself carries no event log yet (WS1 #502) — every emission here is unpinnable by goldens regardless. |
| Phase 1 — old-Sovereign CLAIMS stripping (`remove_edge` loop, lines 200-201) | **PORTABLE NOW** | `remove-edge` is a landed verb. The frozen code's `contextlib.suppress(KeyError)` never actually fires on this path (the edges were just queried as present), so BSL's stricter "absence is never success" (`E-EVAL-031`) semantics are functionally equivalent here, not a behavior change — the one clean, fully-portable computation in this system. |
| Phase 2 — active secession, new-Sovereign minting + `bulk_partition_claims` (lines 203-261) | **BLOCKED — every Phase-1-minting gap, plus rich edge-attribute storage** | All three Phase-1-partition blockers apply again (opaque ID minting, `color_hex`/`ruling_faction_id` field types, `secession_eligible` sourced from `persistent_data`). Additionally: `bulk_partition_claims` moves CLAIMS edges that carry **five** attributes (`control_level`, `legal_status`, `fiscal_status`, `recognition_level`, `claimed_since_tick`); `GraphSubstrate`'s edge state is one `f64` "strength" keyed by `(type, from, to)` (`structural_verbs.rs:16-21`; `substrate.rs`'s `add_edge` takes a single mandatory `strength: f64`) — the named Slice 2 (edge-attribute reads/`EdgeRef`) and Slice 4 (attribute-storage widening, Director-escalation-gated per the task brief) gaps both bind directly here, and there is no O(K) bulk-rewire primitive of any kind in the landed verb set (`update-edge` is explicitly "recognised here and refused loudly" per the same module, since the substrate has no field storage for it to write). |
| Phase 2 — CIVIL_WAR_DECLARED emission (lines 250-261) | **BLOCKED — same string-payload gap as Phase 1's emissions** | `parent_sovereign_id`/`secessionist_faction_id` are string payload fields; same `bsl-language.rst:2889-2893` ruling applies. |
| Phase 2 — stale-attribute-carryover / comment-code `legal_status` mismatch (§2 defects) | **NOT-A-PACK today, PORT-QUESTION at build time** | Both are genuine frozen-code defects to transcribe verbatim per port-as-is law, not to silently fix — but they are moot until the surrounding computation (blocked above) is buildable at all; flagged here so the eventual D-record inherits them rather than rediscovering them. |
| Phase 3 — orphan-Sovereign scan + cleanup (lines 276-290) | **PORTABLE WITH D-RECORD** | The existence check ("does this Sovereign have zero outgoing CLAIMS edges") is expressible via Slice-1 `exists`/`neighbors` over typed edges (no edge *attribute* read needed, only adjacency — a favorable, narrower case than Phases 1-2's CLAIMS-payload dependence). `remove-node` is landed and, favorably, BSL's own `GraphSubstrate::remove_node` cascades to incident edges exactly as the concrete Python `BabylonGraph.remove_node` does (`substrate.rs:82-99`, ADR185 R2) — a genuine structural match, not a deviation. Blocked only by the same `SOV_EXTERIOR_NULL` ID-string-exemption gap as Phase 1 (shared D-record: the `is_exterior_fallback` bool field). |
| `_extraction_policy_for_faction` — `colonial_stance` → `extraction_policy` derivation (lines 293-313) | **PORTABLE WITH D-RECORD (enum mapping) / BLOCKED (its invocation)** | The 3-way enum-to-enum mapping itself (`UPHOLD→INTENSIFY`/`IGNORE→CONTINUE`/`ABOLISH→CEASE`) is trivially expressible via landed enum-field comparison (`field-of`/`:field` + `=` + nested `if`) — no arithmetic, no libm, RESERVED-LINE content (§3) but structurally simple. Its actual invocation here, however, is gated on a `faction_id` string pulled from `persistent_data` and used to `get_node(faction_id)` by an externally-supplied ID — not one of the landed query heads (which operate over already-queried `NodeRef`s, not "look up by an arbitrary string handed in from outside the graph") — so it inherits the same `persistent_data` blocker as the computations that call it. |
| Housekeeping — single-shot `persistent_data` clearing (lines 100-104) | **NOT-A-PACK** | Pure Python-dict bookkeeping with zero graph interaction; moot once the `persistent_data` channel itself is redesigned away (per the Phase-1/2 grouping blocker) rather than needing its own BSL representation. |

**Summary count:** 1 fully portable-now computation (old-CLAIMS stripping), 3 portable-with-a-named-D-record
(collapse predicate scan minus minting, orphan cleanup, the enum-mapping helper in isolation),
5 blocked computations naming 5 distinct structural gaps (opaque node-ID minting; string/NodeRef
field types; rich multi-field CLAIMS-edge storage — the named Slice 2/4 gap; string `emit`
payloads; the `persistent_data` cross-system channel and its missing groupby-over-population
primitive), 1 not-a-pack (pure bookkeeping).

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/balkanization/test_collapse_transition_system.py` | 226 | **Primary unit conformance oracle.** Covers `legitimacy<=0.0` collapse (`test_legitimacy_zero_triggers_sovereign_collapse`), per-Territory `TERRITORY_TRANSITION` emission, CLAIMS-edge stripping, the external-trigger override path (`test_external_trigger_via_persistent_data` — a **test-only** exercise of the production-dead `collapse_triggers` channel, mirroring Territory's `test_context_displacement_mode_is_respected` pattern exactly), single-shot trigger clearing, `SOV_EXTERIOR_NULL` exemption. **Does not** exercise the `by_faction`/successor-Sovereign-minting branch at all — every test here leaves `winning_faction_by_territory` empty, so `add_node`/`add_edge` for a new successor Sovereign never actually runs in this file. |
| `tests/unit/engine/laws/test_law_collapse_transition.py` | 243 | **Strongest property-based invariant oracle** (hypothesis-driven, `@given`). Four named laws (L1-L4, docstring lines 18-60): Territory-node conservation, exact/exclusive post-collapse CLAIMS partition (`_CONTROL = BalkanizationDefines().initial_post_collapse_control_level`, cross-checks the exact define value), old-Sovereign CLAIMS stripping, and full no-op inactivity. **Does** exercise the successor-Sovereign-minting branch (`test_partition_is_exact_and_exclusive`, faction IDs `"FAC_A"`/`"FAC_B"`) — but those Faction IDs are never seeded as real graph nodes, so `_extraction_policy_for_faction`'s `get_node(faction_id) is None → "continue"` fallback branch is exercised, never the live `colonial_stance` lookup. |
| `tests/integration/balkanization/test_us4_secession_fracture.py` | 263 | **Strongest integration conformance oracle — the only file exercising both the live extraction-policy derivation (real Faction nodes with real `colonial_stance`) and the full secession/`bulk_partition_claims` path.** `test_secession_creates_new_sovereign_via_bulk_partition`, `test_civil_war_declared_event_emitted_with_contested_count`, `test_orphaned_sovereign_pruned_when_all_territories_secede` (the parent-orphaned-in-the-same-tick edge case), `test_sov_exterior_null_never_orphan_pruned`, `test_collapse_with_winning_factions_creates_successor_sovereigns` (the ONLY test in the whole estate that exercises Phase-1 successor-minting with real Faction/`colonial_stance` nodes present). **None** of these tests assert on `fiscal_status`/`recognition_level`/`claimed_since_tick` post-secession (the stale-carryover defect, §2) or on the literal `legal_status` value written (the `"de_facto"` vs. comment-claimed `"disputed"` defect, §2) — both defects are present but silently untested here. |
| `tests/integration/balkanization/test_determinism_replay.py` | 236 | **Determinism conformance oracle.** Drives a real 4-system pipeline (`FactionInfluenceSystem`, `SovereigntySystem`, `MetabolismSystem`, `CollapseTransitionSystem`) twice from the same seed, asserting byte-identical state mutations and event sequences (`test_determinism_byte_identical_state_replay`, `test_determinism_event_stream_byte_identical`, `test_determinism_distinct_seeds_can_diverge`) — a genuine cross-system, not just single-system, conformance candidate. |
| `tests/integration/balkanization/test_seed_coverage_invariant.py` | 126 | Schema/seed invariant test (SC-017: every in-scope Territory has either an active INFLUENCES row or a `SOV_EXTERIOR_NULL` CLAIMS row) — a seeding-layer contract, not a `CollapseTransitionSystem.step()` behavior test; does not import `CollapseTransitionSystem`. |
| `tests/integration/balkanization/test_audit_round_trip.py` | 158 | Persistence/audit-table round-trip test; does not import `CollapseTransitionSystem` — out of scope for this system's conformance oracle. |
| `tests/integration/balkanization/test_postgres_persistence.py` | 63 | Migration-0025 schema test (`requires_postgres` marker); does not import `CollapseTransitionSystem` — schema-level, not behavior-level. |
| `tests/integration/balkanization/test_us1_extraction_trajectory.py` | 223 | Drives `SovereigntySystem` + `MetabolismSystem` only; does not import `CollapseTransitionSystem` — narrative-adjacent to this system's domain but not a conformance candidate for it. |
| `tests/unit/engine/scenarios/test_balkanization_seed.py` | 187 | Behavioral contract for `apply_balkanization_seed` itself (the seeding function, §1/§5) — schema/seed-shape test, not a `CollapseTransitionSystem.step()` test. |
| `tests/unit/balkanization/test_balkanization_defines.py` | 119 | `BalkanizationDefines` field-presence-vs.-schema-JSON test — pure schema conformance, not behavior. |
| `tests/unit/formulas/test_balkanization_import.py` | 30 | Import-cycle smoke test for `formulas/balkanization.py` — infrastructure, not behavior. |

**qa:regression byte-gate coverage.** `tools/regression_test.py::graph_content_hash` hashes every
node/edge attribute of the `WorldState→graph` projection on all 12 canonical scenarios, so any
change to this system's outputs is caught by the byte-identical hash gate **on the 5
balkanization-seeded scenarios' Phase-3 no-op path only** (§5) — Phase 1/2's real mutation logic
(successor-Sovereign minting, CLAIMS partition, secession) has **zero** canonical-scenario
coverage today. A port's conformance oracle should lean heavily on
`test_us4_secession_fracture.py` (the one file that already exercises the live-`colonial_stance`
+ full-secession path end to end) and `test_law_collapse_transition.py`'s hypothesis-driven
invariants, both of which are strong, ready-to-adapt candidates rather than needing to be built
from nothing — a materially better starting position than Territory's port faced.

---

## Adjudication (2026-08-12)

Adjudicated against the current dev tree (`9324482f`). The inventory is thorough on the frozen
Python side — its two Phase-2 defects, its dormancy re-derivation, and its enum/string/NodeRef
type findings all survive fresh checking verbatim. Its **blocker table does not**, because it
read only the retired half of the Rust effect executor. Four corrections and three
confirmations.

1. **CORRECTION — Phase-1 CLAIMS stripping is NOT "PORTABLE NOW"; a rule naming `remove-edge`
   is refused at content LOAD.** §6's row rates it PORTABLE NOW because "`remove-edge` is a
   landed verb," and §1 cites `structural_verbs.rs` lines 1-60 / 843-891 as its evidence base.
   Those lines belong to `EffectExecutor::execute_effects`, which the module itself says is
   **"retired from production (Task 12) and stays only as a test/corpus harness"**
   (`rust/crates/babylon-bsl/src/structural_verbs.rs:704-708`). The production seam is
   `collect_effects`/`collect_item`, whose match arms serve exactly `guard`, `update-node`,
   `emit` and `for-each` (`structural_verbs.rs:661-687`) and refuse the six graph-shape verbs
   (`:691-707`). That runtime refusal is itself only defence in depth: **`check_no_deferred_shape_verbs`**
   (`structural_verbs.rs:1388-1405`) walks every rule form at load and rejects any occurrence of
   `add-node`/`remove-node`/`add-edge`/`remove-edge`/`add-hyperedge`/`remove-hyperedge` —
   "deferring a MINTING verb needs a placeholder-id scheme this repair does not specify, so
   run_tick's two-pass split cannot defer {verb} the way it defers update-node." It is wired
   into the load pipeline at `rule_pipeline.rs:49` and surfaces as
   `LoadError::DeferredShapeVerb` (`rule_pipeline.rs:143-150`). **The inventory's only
   fully-portable-now computation therefore does not exist.**
2. **CORRECTION — Phase-3 orphan cleanup is BLOCKED, not "PORTABLE WITH D-RECORD".** `remove-node`
   is one of the same six deferred shape verbs (`structural_verbs.rs:1354` in `DEFERRED_SHAPE_VERBS`,
   refused at `:1388-1405`), so no rule expressing it loads today — the `SOV_EXTERIOR_NULL`
   ID-string exemption is not this row's binding constraint. The row's *favourable* half is
   independently CONFIRMED and worth keeping: `GraphSubstrate::remove_node` genuinely cascades to
   incident structure, and says so (`rust/crates/babylon-graph/src/substrate.rs:82-99`, ADR185 R2),
   matching `BabylonGraph.remove_node`. That is a substrate fact, not an executable-verb fact.
3. **CORRECTION — the minting rows name the right gap for the wrong reason, and miss a sixth gap
   that outranks all five.** §6's "opaque node-ID minting" framing is accurate about what
   `add-node`'s id operand *means* (`structural_verbs.rs:29-33` and `:857-861`:
   `fresh_declared_name` + `graph.add_node(node_type)` returning an opaque
   `NodeId(u64)`, `substrate.rs:30-33,80`) — but that description is of a code path production
   never reaches. The decisive gap is prior and blunter: **the verb does not load.** The
   "placeholder-id design" named in the load-time refusal message is the follow-on train that
   must land before any of Phase 1's or Phase 2's minting is even expressible. The verdict's
   gap count of five should be six, and the sixth is first in line.
4. **CORRECTION — an unflagged determinism hazard inside the very method §2 dissects.**
   `BabylonGraph.bulk_partition_claims` iterates `for territory_id in territories` where
   `territories` is a **`set[str]`** (`src/babylon/topology/graph.py:992`, called with the set
   built at `collapse_transition.py:223`); string-set iteration order is `PYTHONHASHSEED`-
   dependent, and the resulting `add_edge` insertion order is observable structure (S-19) that
   `graph_content_hash` walks. The estate's own two-process determinism leg deliberately
   **strips** `PYTHONHASHSEED` to catch exactly this class (`tools/regression_test.py:1517`).
   The frozen author was alive to the issue one line later — `_execute_secession` sorts the same
   set for its follow-up `update_edge` loop (`collapse_transition.py:265`) while handing it
   unsorted to `bulk_partition_claims` (`:244-248`). §2's "determinism-inconsistency" note flags
   the Phase-3 unsorted scan (which it correctly reasons is order-*independent* in final state)
   and misses this one, which is not. Dormant today — it needs ≥2 seceding territories, provably
   unreachable on canonical — so no gate would catch it either.
5. **CONFIRMATION — the dormancy finding, both legs, verified independently.** (a) `legitimacy`
   is written by no System: a grep over `src/babylon/engine/systems/*.py` finds only this file's
   two node-creation stamps (`collapse_transition.py:163,235`) and its read at `:84`;
   `electoral.py:1059` and `:1124` are `FactionBalance.legitimacy` and an
   `update_internal_balance(legitimacy=…)` operand, neither a Sovereign node attribute. (b)
   `apply_balkanization_seed` has exactly ONE call site (`electoral_fixture.py:204`), reached only
   through `apply_political_terrain`, called by exactly five factories
   (`electoral_goldens.py:226,270,334,428,504`) — so "**5 of the 12**" is exact: `SCENARIOS`
   (`tools/regression_scenarios.py:37-129`) holds 12 entries, and the `COVERAGE_GAPS_DATA` row at
   `:2841-2847` reads verbatim as quoted. The correction to the ledger's stated reason stands.
6. **CONFIRMATION — both §2 Phase-2 defects, transcribed exactly right.** `bulk_partition_claims`
   carries the parent payload wholesale (`topology/graph.py:995-999`:
   `edge_data = dict(self._edge_payload[key]); … self.add_edge(to_sovereign_id, territory_id,
   **edge_data)`), and the follow-up refreshes only two of five fields
   (`collapse_transition.py:265-273`). The comment at `:262-264` does say "Promote the new edges'
   legal_status to disputed" while `:271` writes `legal_status="de_facto"`. Both stand as
   port-as-is transcription obligations.
7. **CONFIRMATION — the string/NodeRef/emit gaps, verified, and one of them UNDERSTATED.**
   `deffield`'s `<type-name>` vocabulary is exactly seven rows — `int`/`bool`/`currency`/
   `probability`/`intensity`/`coefficient`/`enum` — with no string and no reference row
   (`docs/reference/bsl-language.rst:2293-2382`); "there is no string concatenation, comparison,
   or interpolation in the language" (`:458-460`); a string literal in expression position is
   `E-PARSE-010` (`:487-489`). The `emit` ruling is **stronger** than §6 states: "every
   `<payload-item>` expression is a number, a bool or an enum-ref"
   (`bsl-language.rst:2889-2897`), so `sovereign_id`/`from_sovereign_id`/`to_sovereign_id` are
   excluded as *references*, not merely as strings — there is no NodeRef-payload escape route to
   look for. The RESERVED-LINE flag on `ColonialStance`→`ExtractionPolicy` is correctly raised
   and correctly not acted on.

**FINAL VERDICT: BLOCKED — sustained and hardened. SIX structural gaps, not five, and the new
one leads: five of the seven §2.8 verbs this system needs (`add-node`, `add-edge`, `remove-edge`,
`remove-node`, plus `update-edge`) are refused before or at execution — the four shape verbs at
content LOAD by `check_no_deferred_shape_verbs`, `update-edge` at every execution path for want
of substrate storage — so ZERO of this system's computations are portable today. The
inventory's one PORTABLE-NOW row and one of its three PORTABLE-WITH-D-RECORD rows both fall.
Everything else in the report (zero arithmetic/libm hazard, the two Phase-2 defects, the
two-reason dormancy re-derivation, the strong `apply_balkanization_seed` + law/integration
conformance substrate) stands as written.**

**INADEQUATE-COVERAGE (scoped, verdict-consequential).** §1's reference list reads
`structural_verbs.rs` lines 1-60 and 843-891 only, and reaches a portability verdict from them.
A re-read MUST add: (i) `structural_verbs.rs:640-720` — the `collect_item` production effect
path and its refusal arms; (ii) `structural_verbs.rs:1340-1405` — `DEFERRED_SHAPE_VERBS` and
`check_no_deferred_shape_verbs`; (iii) `rust/crates/babylon-bsl/src/rule_pipeline.rs` — the
load-gate pipeline and `LoadError::DeferredShapeVerb` (`:49,143-150`); (iv)
`rust/crates/babylon-tick/src/lib.rs::run_once_into` (`:273-280`), the actual production seam
every portability claim in this survey is measured against. Without (i)-(iv) the report cannot
tell a landed verb from a retired one, which is exactly what happened.
