# EpistemicHorizonSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** `EpistemicHorizonSystem` (`src/babylon/engine/systems/epistemic_horizon.py`,
245 lines) is a Phase-1 SHADOW-ONLY system: it runs last in the tick (position 22.0, last of 34
Systems), reads already-mutated same-tick state (`p_acquiescence`, `ideology.class_consciousness`,
`role`, `population` off TENANCY-linked SocialClass tenants, plus PRESENCE-edge topology) and writes
three derived territory attrs (`mass_receptivity`, `intel_confidence`, `vision_state`) that model
"fog of war" — how well the player org can perceive a territory's true state. Every one of those
three writes is explicitly excluded from `WorldState.from_graph()` (`TERRITORY_EXCLUDED_FIELDS`,
`world_state.py:135-137`) and therefore never reaches `graph_content_hash`/`qa:regression`'s byte
gate — a fact proven by a dedicated unit test, not inferred. The formulas themselves ARE largely
expressible against the current landed BSL query/fold/enum surface (the population-weighted M_r
mean maps almost exactly onto a landed, evaluated `fold mean … :weight …`), but the system's
entire purpose is fog/epistemic display data, and the codebase's own governing standard already
rules (S-23, `ai/bsl-architecture-standard.md:705`, citing Amendment V/II.8) that "fog is epistemic
and stays out of the tick hash" — the `Obs: 𝒮 → Proj` morphism is one-way, with no morphism back
into 𝒮. **Verdict: NOT-A-PACK.** This is not a `.bsl` tick-content port candidate at all; it belongs
on the Rust projection/`Obs` lane (a post-tick recompute over already-hashed state, exactly the
pattern the legacy web bridge's `_carry_epistemic_horizon` already implements), never fed through
`run_once_into`.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/epistemic_horizon.py` | 245 | **The target.** `EpistemicHorizonSystem` (thin `SystemBase` adapter, lines 212-245) + the two pure functions it delegates to: `mass_receptivity_of` (52-110) and `compute_epistemic_horizon` (113-209). Self-contained — imports only `kernel.node_access`, `kernel.tick_partition`, `models.enums`, `kernel.system_base`, `kernel.system_protocol` (lines 40-49); zero `formulas/`/`domain/` imports (grep-confirmed). |
| `src/babylon/kernel/node_access.py` | 37 | `class_consciousness_from_node` (lines 15-37) — shared reader for the `ideology.class_consciousness` sub-dict, called from `mass_receptivity_of:96`. Also used by SolidaritySystem/StruggleSystem/ImperialRentSystem (spec-116 Phase 3 dedup). |
| `src/babylon/kernel/tick_partition.py` | 30 | `TickPartition.CONSEQUENCE` — the enum member `EpistemicHorizonSystem.partition` is declared as (epistemic_horizon.py:219). |
| `src/babylon/config/defines/epistemic_horizon.py` | 172 | `EpistemicHorizonDefines` Pydantic model — the coefficient source. |
| `src/babylon/data/defines.yaml` | (block: lines 952-967) | Player-editable `epistemic_horizon:` section, values matching the Pydantic defaults. |
| `src/babylon/models/entities/territory.py` | 352 | `Territory` Pydantic entity. `investigation_intel` (a real, declared field, lines 289-299) is read by `compute_epistemic_horizon:187`. `mass_receptivity`/`intel_confidence`/`vision_state` are **not** declared fields here — deliberately (see §5). |
| `src/babylon/models/entities/social_class.py` | 522 | `SocialClass` Pydantic entity — `role: SocialRole` (line 296, required), `population: int` (line 406, default 1, `ge=0`), `p_acquiescence: Probability` (line 341-ish, `[0,1]`) are all real declared fields `mass_receptivity_of` reads off TENANCY-linked tenants. |
| `src/babylon/models/enums/social.py` | 211 | `SocialRole` (StrEnum, 8 members, lines 12-64) — 4 of 8 members are named in `mass_receptivity_of`'s `role_factor` table (74-77); the other 4 fall through to `class_factor_default`. `SocialRole.coerce` (45-64) is the defensive live-enum-or-string coercion `mass_receptivity_of:97` calls. |
| `src/babylon/models/enums/topology.py` | 253 | `NodeType.TERRITORY`/`.SOCIAL_CLASS` (lines 61, 62), `EdgeType.TENANCY`/`.PRESENCE` (lines 106, 113). |
| `src/babylon/kernel/system_base.py` | 202 | `SystemBase` ABC — `EpistemicHorizonSystem` inherits the ClassVar/`step()` contract only; it calls **none** of `SystemBase`'s helper methods (`_write_clamped`, `_publish`, `_get_persistent_data`) — the `intel_confidence` clamp is hand-rolled inline instead (see §4 item 7). |
| `src/babylon/kernel/system_protocol.py` | 41 | `ContextType = "TickContext"` (line 16) — `EpistemicHorizonSystem.step`'s third parameter is named `_context` (underscore-prefixed, unused) — confirms zero `TickContext` dependency (see §5). |
| `src/babylon/kernel/graph_protocol.py` | 494 | `GraphProtocol.query_nodes`/`.query_edges`/`.get_node`/`.update_node` signatures (lines 77-98, 258-298) — note `query_edges` has **no target-node filter parameter**; both `mass_receptivity_of` and the PRESENCE loop do a full-type scan then filter by `edge.target_id == territory_id` in Python (§4 performance note). |
| `src/babylon/topology/graph.py` | 1033 (relevant: `update_node` at 660-670) | Concrete `BabylonGraph.update_node` — plain dict merge, **no type coercion or quantization at tick time**, same as every other System (Territory's inventory precedent). |
| `src/babylon/models/world_state.py` | 1161 | `TERRITORY_EXCLUDED_FIELDS` (lines 94-150ish) — the frozenset that drops `mass_receptivity`/`intel_confidence`/`vision_state` (135-137) on every `from_graph()` reconstruction. **Load-bearing for the whole verdict** — see §5/§6. |
| `src/babylon/engine/actions/investigate.py` | 122 | `resolve_investigate` — an **independent production consumer** of `mass_receptivity_of` (line 94), called at OODA/action-resolution time (tick position 14, **before** `EpistemicHorizonSystem` runs at 22.0 this same tick) to gate the player's INVESTIGATE verb. Not called by, and does not call, `EpistemicHorizonSystem`. |
| `src/babylon/engine/simulation_engine.py` | 611 | `_SYSTEM_CLASSES` tuple (328-363) — confirms `EpistemicHorizonSystem` is the **last** entry (line 362) of 34 systems. |
| `web/game/engine_bridge.py` | (relevant: `_carry_epistemic_horizon`, lines 8294-8347) | **Legacy, non-gating** (web = legacy per Amendment V/II.8 ruling). Re-invokes `compute_epistemic_horizon` against the post-round-trip `new_graph` before persistence, to re-derive the three dropped attrs for display — living precedent for the "recompute in the projection lane, never re-enter the hash-bearing state" pattern this inventory's verdict recommends for the Rust side. Its own docstring states the same `TERRITORY_EXCLUDED_FIELDS` mechanism this inventory relies on (8299-8306). Not fully read line-by-line (out of scope — legacy, doesn't gate); the one function cited was read in full. |
| `src/babylon/kernel/services.py` | (relevant: `ServicesProtocol`, lines 23-38) | `services.defines.epistemic_horizon` — the DI seam `EpistemicHorizonSystem.step:243` reads the `EpistemicHorizonDefines` instance from. |
| `src/babylon/models/types.py` | 337 | `Probability`/`Intensity`/`Coefficient`/`Currency` — all `Annotated[float, Field(ge=..., le=...), SnapToGrid]`. Confirms `p_acquiescence` (Probability) and `class_consciousness` (Intensity, by the writer's own field name and domain) are `[0,1]`-quantized-on-instantiation types, **not** quantized mid-tick (same non-quantized-in-tick precedent as every other System). |
| `ai/bsl-architecture-standard.md` | (relevant: row S-23, line 705; §5.4, lines 639-673) | The governing BSL port standard. **S-23 is the load-bearing citation for this inventory's NOT-A-PACK verdict** (see §6). |
| `docs/reference/bsl-language.rst` | (relevant: §3.1 types, 2293-2382; §3.3 numeric lanes, 2513-2530; §3.4 kind/aggregation law, 2532-2617; `eval_fold`/`fold_mean`, referenced via the Rust source below) | The BSL language reference — types table, kind-propagation/aggregation-law table, store-boundary clamp behavior. |
| `rust/crates/babylon-bsl/src/evaluator.rs` | (relevant: `eval_fold` 773-874, `fold_mean` 1074-1140ish, weighted-mean tests ~2700-2890) | Confirms `fold mean … :weight …` is **evaluated**, not just typechecked — the direct target shape for `mass_receptivity_of`'s population-weighted mean. |

**Reference material read for BSL-surface grounding (not the target file, cited by anchor):**
`docs/reference/bsl-language.rst` §§3.1, 3.3, 3.4 (types, numeric lanes, kind/aggregation law);
`rust/crates/babylon-bsl/src/evaluator.rs` (`eval_fold`, `fold_mean`, the weighted-mean unit tests);
`ai/bsl-architecture-standard.md` (row S-23 and its citations, §5.4 "Defects not to transcribe").

---

## 2. COMPUTATION CATALOG (execution order, `epistemic_horizon.py:164-209`)

There is exactly **one** `step()` call, delegating to `compute_epistemic_horizon`, which loops over
every TERRITORY node and performs the same five-part computation per territory. There are no
"phases" in Territory's sense — one pass, one formula family, honest-null-gated.

### Computation 1 — Mass Receptivity, M_r (`mass_receptivity_of`, lines 52-110)

- **(a)** For one territory, M_r is the population-weighted mean, over its TENANCY-linked tenant
  SocialClass nodes, of `(1 - P(S|A)) × class_consciousness × C_f` — "how ready are the masses who
  live here to talk to you," per class role.
- **(b) Formula, in execution order:**
  - Per-tenant role→coefficient lookup (lines 73-78, 97-102):
    `class_factor = role_factor.get(role, defines.class_factor_default)` where `role_factor` maps
    exactly 4 of `SocialRole`'s 8 members (`PERIPHERY_PROLETARIAT`, `LUMPENPROLETARIAT`,
    `PETTY_BOURGEOISIE`, `LABOR_ARISTOCRACY`) to their `class_factor_*` define; every other role
    (`CORE_BOURGEOISIE`, `COMPRADOR_BOURGEOISIE`, `INTERNAL_PROLETARIAT`, `CARCERAL_ENFORCER`, or an
    unrecognized/absent role) falls to `class_factor_default` (0.0).
  - Per-tenant zero-population skip (lines 91-93): `population = float(attrs.get("population", 0)
    or 0); if population <= 0.0: continue` — a tenant contributing zero population is excluded
    from both the numerator and the weight sum entirely (not merely weighted to zero).
  - Per-tenant M_r term (line 104): `class_m_r = (1.0 - p_acquiescence) * ideological_alignment *
    class_factor`.
  - Accumulation (lines 105-106): `weighted_sum += class_m_r * population; total_population +=
    population`.
  - Honest-null gate (lines 108-109): `if total_population <= 0.0: return None`.
  - Result (line 110): `return weighted_sum / total_population`.
- **(c) Reads:** `EdgeType.TENANCY` edges (full-type scan, filtered to `target_id == territory_id`,
  lines 83-85); tenant node's `node_type` (must be `"social_class"` — a **bare string literal
  comparison**, line 87, not `NodeType.SOCIAL_CLASS`, though the two are value-identical today —
  see §4 for the transcription note); tenant's `population` (default 0, line 91), `p_acquiescence`
  (default 0.0, line 95), `ideology.class_consciousness` (via `class_consciousness_from_node`,
  default 0.0 on missing/malformed `ideology`, `node_access.py:29-36`), `role` (via
  `SocialRole.coerce`, `None` on absent/unrecognized, line 97).
- **(d) Writes:** none — pure function, returns `float | None`.
- **(e) Defines:** `epistemic_horizon.class_factor_periphery_proletariat` (1.0, `[0,1]`),
  `.class_factor_lumpenproletariat` (1.0, `[0,1]`), `.class_factor_petty_bourgeoisie` (0.3, `[0,1]`),
  `.class_factor_labor_aristocracy` (0.2, `[0,1]`), `.class_factor_default` (0.0, `[0,1]`) —
  `defines.yaml:958-962`, `EpistemicHorizonDefines` `config/defines/epistemic_horizon.py:58-87`.
- **(f) Events:** none.

**RESERVED-LINE flag:** the four named `class_factor_*` values (and the choice of which 4 of 8
`SocialRole` members carry a named coefficient at all) are theory-corpus-derived MLM-TW class
content — "who informs on the movement to the state, and how much" — sourced verbatim from
`ai/epochs/epoch3/fog-of-war.yaml:195-236` per the defines module's own docstring
(`config/defines/epistemic_horizon.py:27-36`). This is Director-reserved ideological content in
the same register as doctrine-tree tag content and National Question parameters. **Described here,
never proposed for change.**

### Computation 2 — Cadre Presence, C_p (inline in `compute_epistemic_horizon`, lines 173-183)

- **(a)** Binary flag: does the player org (or a legacy-tagged `is_player` org) have a PRESENCE
  edge into this territory?
- **(b)** `cadre_presence = 0.0`; loop over `EdgeType.PRESENCE` edges with `target_id ==
  territory_id`; set `cadre_presence = 1.0` and `break` on the **first** match satisfying either
  `player_org_id is not None and edge.source_id == player_org_id` (the primary, "EH ruling 6" path)
  or `org.attributes.get("is_player", False)` (the `PoliticalFaction`-subtype legacy fallback path,
  lines 176-183).
- **(c) Reads:** `EdgeType.PRESENCE` edges (full-type scan filtered by `target_id`); the acting
  org's node via `graph.get_node(edge.source_id)`; `org.attributes["is_player"]` (a real field only
  on the `PoliticalFaction` `Organization` subtype, `models/entities/organization.py:399-402`;
  absent/`False` on every other org type, defaulted via `.get(..., False)`); `player_org_id` —
  **not a graph attribute at all**, read from `graph.graph["player_org_id"]` graph-level metadata
  one level up in `EpistemicHorizonSystem.step` (lines 239-240) and passed in as a plain function
  argument.
- **(d) Writes:** none directly (feeds Computation 4).
- **(e) Defines:** none.
- **(f) Events:** none.

### Computation 3 — Investigation Intel carry (inline, line 187)

- **(a)** Reads the player's already-accumulated, event-sourced INVESTIGATE intel for this
  territory (a real, round-trip-surviving `Territory` field, written by a **different** module —
  `engine/actions/investigate.py`'s `resolve_investigate`, not by this system).
- **(b)** `investigation_intel = float(territory.attributes.get("investigation_intel", 0.0))`.
- **(c) Reads:** `TERRITORY.investigation_intel` (default 0.0).
- **(d) Writes:** none directly (feeds Computation 4).
- **(e) Defines:** none (the `investigate_intel_boost`/`investigate_min_receptivity` defines that
  *produce* this value live entirely in `investigate.py`, not here).
- **(f) Events:** none.

### Computation 4 — Intel Confidence, I_c (lines 189-195)

- **(a)** A baseline public-observation floor, plus cadre-presence-gated M_r, plus earned
  investigation intel, clamped to `[0,1]`.
- **(b)** `intel_confidence = max(0.0, min(1.0, defines.base_observation + cadre_presence *
  mass_receptivity + investigation_intel))` — a **hand-rolled** `max(lo, min(hi, value))` clamp,
  structurally identical to `SystemBase._write_clamped`'s shape (`system_base.py:189`) but **not**
  calling that shared helper (this System calls none of `SystemBase`'s methods).
- **(c) Reads:** `defines.base_observation`; Computation 2's `cadre_presence`; Computation 1's
  `mass_receptivity` (guaranteed non-`None` at this point — the honest-null `continue` at line
  168-171 already skipped this territory otherwise); Computation 3's `investigation_intel`.
- **(d) Writes:** none directly (feeds Computation 5 and the final `update_node` at line 204).
- **(e) Defines:** `epistemic_horizon.base_observation` (0.1, `[0,1]`) — `defines.yaml:955`.
- **(f) Events:** none.

### Computation 5 — Vision State classification (lines 197-202)

- **(a)** Three-way threshold classification of M_r into a display-tier label.
- **(b)** `if mass_receptivity < defines.desert_threshold: "desert" elif mass_receptivity >=
  defines.water_threshold: "water" else: "mud"` — plain comparisons, no arithmetic.
- **(c) Reads:** Computation 1's `mass_receptivity`; `defines.desert_threshold`,
  `defines.water_threshold`.
- **(d) Writes:** none directly (feeds the final `update_node`).
- **(e) Defines:** `epistemic_horizon.desert_threshold` (0.2, `[0,1]`), `.water_threshold` (0.8,
  `[0,1]`) — `defines.yaml:956-957`.
- **(f) Events:** none.

**⚠ `vision_state` is NOT backed by any Python enum.** `"desert"`/`"mud"`/`"water"` are bare string
literals (lines 198, 200, 202) — grep-confirmed no `VisionState` (or equivalent) StrEnum exists
anywhere in `src/babylon/models/enums/`. (`TerritoryType`'s `WATER = "water"` in
`models/enums/territory.py:131` is an unrelated hex-terrain classification — coincidental name
overlap, not shared vocabulary.) This is a genuine looseness in the frozen reference itself,
recorded verbatim per port-as-is law (§6).

### Final write (line 204-209)

`graph.update_node(territory_id, mass_receptivity=mass_receptivity, intel_confidence=
intel_confidence, vision_state=vision_state)` — all three attrs written together, unconditionally,
once the honest-null gate is passed. **Events emitted by the whole system: zero** (grep-confirmed:
no `EventType`/`.publish(`/`emit` reference anywhere in `epistemic_horizon.py`).

---

## 3. TYPE INVENTORY

| Attribute | Node type | Python model type | Domain | Category |
|---|---|---|---|---|
| `role` | SOCIAL_CLASS | `SocialRole` (StrEnum, 8 members) | closed set | **Enum discriminant** — read-only here; written by scenario seeding elsewhere |
| `population` | SOCIAL_CLASS | `int` | `≥ 0`, default 1 | integer, extensive |
| `p_acquiescence` | SOCIAL_CLASS | `Probability` (`Annotated[float, ge=0.0, le=1.0]`) | `[0,1]` | unit-interval, intensive (rate) |
| `ideology.class_consciousness` | SOCIAL_CLASS | plain `float` inside a `dict`-valued `ideology` attr | `[0,1]` (by convention; the writer only ceils at `min(1.0, …)`, no explicit floor — an **upstream** ConsciousnessSystem characteristic, not this system's) | unit-interval, intensive — **nested/structured attribute, not a flat scalar field** |
| `is_player` | ORGANIZATION (`PoliticalFaction` subtype only) | `bool` | `{T,F}`, default `False` | boolean discriminant, subtype-scoped (absent on `STATE_APPARATUS`/`BUSINESS`/`CIVIL_SOCIETY` orgs) |
| `player_org_id` | — (graph-level metadata, `graph.graph["player_org_id"]`) | `str \| None` | — | **not a node/edge attribute at all** — WorldState-scoped identity metadata, read one level above the graph substrate |
| `investigation_intel` | TERRITORY | `float` (real declared `Territory` field) | `[0,1]`, default 0.0 | unit-interval, **survives WorldState round-trip** (unlike the three below) |
| `mass_receptivity` | TERRITORY | plain `float` (transient graph attr — **not** a `Territory` model field) | `[0,1]` (unclamped by construction — see §4 item 3) | unit-interval, intensive, **write-only, dropped every round-trip** |
| `intel_confidence` | TERRITORY | plain `float` (transient graph attr — **not** a `Territory` model field) | `[0,1]` (explicitly clamped) | unit-interval, intensive, **write-only, dropped every round-trip** |
| `vision_state` | TERRITORY | plain `str` (transient graph attr — **not** backed by any enum, Python or otherwise) | `{"desert","mud","water"}` | **categorical discriminant with no type-system backing whatsoever** — write-only, dropped every round-trip |
| `base_observation`, `desert_threshold`, `water_threshold`, `class_factor_periphery_proletariat`, `class_factor_lumpenproletariat`, `class_factor_petty_bourgeoisie`, `class_factor_labor_aristocracy`, `class_factor_default` (defines) | — | `float` | `[0.0, 1.0]` each | unit-interval coefficients |

**The nested-dict-attribute flag — the sharpest gap in this inventory.** `ideology` is stored as a
`dict[str, float]` graph attribute (`{"class_consciousness": …, "national_identity": …,
"agitation": …}`, written by `ConsciousnessSystem`, `ideology.py:418-424`), and
`class_consciousness_from_node` reads one key out of it (`node_access.py:29-36`). BSL's `deffield`
type vocabulary (`bsl-language.rst` §3.1, verified against the current table: `int`, `bool`,
`currency`, `probability`, `intensity`, `coefficient`, `enum` — seven rows, closed: "no
type variables, no subtyping, no coercions, and no user-defined types," line 2367) has **no
struct/record/nested type**. A `deffield` can only ever store one of those seven flat scalar/enum
shapes. `class_consciousness` cannot be read via any `:field`/`field-of` accessor as a sub-path of
a larger `ideology` object — it would need its **own** top-level `deffield class-consciousness
:type probability` (a decision that belongs to whoever ports `ConsciousnessSystem`, not to this
system's own port). This is a genuine cross-pack content-modeling dependency, not a language-level
blocker (see §6).

**The enum-discriminant flags — landed differently than Territory's port found.** `SocialRole` (8
members) is read (not written) by this system for the `role_factor` lookup. Per the CURRENT BSL
surface (ADR195/ADR196, org-foundation train, landed on dev after Territory's port inventory was
written): `defenum`/`deffield … enum …` field storage and `=`/`!=` comparison against an
`<enum-ref>` **are now landed and evaluated** — this supersedes Territory's earlier "no enum row"
finding. The 8-member `role_factor.get(role, default)` lookup is expressible as a 5-way `if`/`elif`
chain of `=` guards against `SocialRole/PERIPHERY_PROLETARIAT` etc. (nested-`if`, matching every
landed-pack precedent — no `dict.get`-equivalent construct exists, nor is one needed). The `role`
field itself would need to be declared `deffield social-class/role :type enum :of SocialRole` by
whoever ports the SocialClass-seeding content — again a cross-pack dependency, not a blocker to
this system specifically.

`vision_state`'s three string literals have no Python-side enum to even transcribe port-as-is —
the port would need to **mint** a new `defenum VisionState {DESERT, MUD, WATER}`, which the frozen
reference itself never declared. This is a genuine improvement-on-transcription case (the frozen
source is looser than what BSL requires), not a regression to fix — recorded as a D-record
decision, not a defect repair (nothing computational changes; only the storage *representation*
tightens from an untyped string to a proper enum).

---

## 4. FLOAT-OP INVENTORY

All arithmetic is binary64 (Python native `float`), grep-confirmed **zero** `exp`/`log`/`pow`/
`sigmoid`/`math.` calls anywhere in `epistemic_horizon.py` or `node_access.py`. **Zero
libm-nondeterminism hazard.** Shapes, in execution order:

1. **Coercion + bare-literal defaulting (×5):** `float(attrs.get("population", 0) or 0)` (line 91),
   `float(attrs.get("p_acquiescence", 0.0))` (95), `float(ideology.get("class_consciousness", 0.0))`
   (`node_access.py:35`), `float(territory.attributes.get("investigation_intel", 0.0))` (187) —
   each a bare `0`/`0.0` default literal. A BSL port's equivalent is the `:default` mechanism on an
   `:optional` binding (§3.5) — every `:default` must appear in the migration corpus's allowlist,
   which is a load-time obligation, not a language gap.
2. **Threshold comparisons (×4):** `population <= 0.0` (92), `total_population <= 0.0` (108),
   `mass_receptivity < defines.desert_threshold` (197), `mass_receptivity >=
   defines.water_threshold` (199) — plain `<`/`<=`/`>=`, no hazard.
3. **The M_r per-tenant term, UNCLAMPED:** `(1.0 - p_acquiescence) * ideological_alignment *
   class_factor` (line 104) — one subtract-from-`1.0`, two multiplies. **No internal clamp.** The
   frozen system's own property-law test documents this explicitly as a deliberate design fact, not
   an oversight (`tests/unit/engine/laws/test_law_epistemic_horizon.py` L2, lines 17-26: "the reader
   itself does NOT clamp … this law only holds for validated-domain inputs, not arbitrary graph
   state"). Contrast with item 7 below (`intel_confidence` IS explicitly clamped) — this asymmetry
   must be transcribed faithfully (port-as-is), not "fixed" by adding a defensive clamp `M_r` never
   had.
4. **Weighted accumulation (running total, ×2):** `weighted_sum += class_m_r * population;
   total_population += population` (lines 105-106) — a **population-weighted mean**, computed via
   sequential `+=` in the iterator's own order (BabylonGraph's insertion-order `query_edges`
   iterator — not itself an ordered/sorted traversal). Floating-point addition is not associative,
   so this is order-sensitive at the bit level, the same class of concern
   `docs/reference/bsl-language.rst` names for BSL's own fold reductions (S-19 "order is structure";
   `evaluator.rs`'s own `fold_mean_sum_wx_is_the_left_fold_and_is_not_partition_invariant` test,
   lines 2706-2733, proves the Rust evaluator treats this identically — sequential left-fold, no
   reassociation).
5. **Division (guarded):** `weighted_sum / total_population` (line 110) — the honest-null gate at
   line 108 (`if total_population <= 0.0: return None`) makes this division-by-zero-safe by
   construction; no hazard.
6. **Boolean-as-float encoding:** `cadre_presence = 0.0` / `= 1.0` (lines 173, 178, 182) — a
   conceptually boolean flag stored and later multiplied as a Python `float`, not a Python `bool`.
   Feeds directly into item 8's multiply.
7. **The intel_confidence sum + explicit clamp:** `defines.base_observation + cadre_presence *
   mass_receptivity + investigation_intel` (line 193, one multiply then two adds, left-to-right per
   Python precedence: `base_observation + (cadre_presence * mass_receptivity)` then `+
   investigation_intel`), wrapped in `max(0.0, min(1.0, …))` (lines 189-195) — **hand-written**, the
   identical `max(lo, min(hi, value))` shape as `SystemBase._write_clamped`
   (`system_base.py:162-189`) but not calling it. **This clamp is not optional for a port**: BSL's
   store boundary does **not** clamp on write — an `update-node` whose resulting value falls outside
   the target field's declared range is `E-EVAL-020`, "a loud runtime failure, never a clamp"
   (`bsl-language.rst:2522-2528`). The unclamped sum genuinely can exceed `1.0` (e.g.
   `base_observation=0.1`, `cadre_presence=1.0`, `mass_receptivity=1.0`, `investigation_intel=1.0`
   → raw sum `2.1`), so a port that skipped the nested-`if` clamp (the only clamp idiom the grammar
   has — no scalar `min`/`max`, per Territory's precedent finding) would **crash the tick** on
   reachable inputs rather than silently misbehave. This is a mandatory transcription requirement,
   not a style choice.
8. **No Real→Int demotion anywhere** in this system (contrast Territory's `int(...)` truncation
   sites) — every quantity here stays in the binary64/Real lane end to end.

**Performance note (not a correctness hazard):** `GraphProtocol.query_edges` takes no target-node
filter (`graph_protocol.py:278-298`), so both the TENANCY loop (`mass_receptivity_of:83-85`) and the
PRESENCE loop (`compute_epistemic_horizon:174-179`) perform a full scan of every edge of that type,
filtered in Python by `target_id`. This is an artifact of `GraphProtocol`'s interface, not of the
formula's content — a BSL port using a typed-neighbor/incidence query would not inherit this
O(territories × edges) shape, but that is a legitimate structural improvement at the interface
level, not a "port-as-is" concern (nothing about the *arithmetic* changes).

---

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 22.0, LAST of 34 Systems** (`epistemic_horizon.py:220`), confirmed against
  `_SYSTEM_CLASSES` (`simulation_engine.py:328-363` — `EpistemicHorizonSystem` is the final tuple
  entry, line 362). Partition `CONSEQUENCE`.
- **Reads from same-tick prior systems (both confirmed live, both run before position 22.0):**
  - `SOCIAL_CLASS.p_acquiescence`, written by `SurvivalSystem` (**position 15.0**,
    `survival.py:78,165`: `graph.update_node(node.id, p_acquiescence=p_acq, p_revolution=p_rev)`).
  - `SOCIAL_CLASS.ideology.class_consciousness`, written by `ConsciousnessSystem` (**position
    17.0**, `ideology.py:109,418-424`).
  - Both writers run strictly before position 22.0, so `EpistemicHorizonSystem` genuinely observes
    this tick's freshly-mutated values, not last tick's stale ones (the docstring's own claim,
    verified).
- **Writes consumed downstream:** **none, structurally, by design.**
  - `mass_receptivity`/`intel_confidence`/`vision_state` are in `TERRITORY_EXCLUDED_FIELDS`
    (`world_state.py:135-137`) — every `WorldState.from_graph()` call **drops all three**,
    confirmed by a dedicated unit test
    (`tests/unit/models/test_graph_roundtrip.py::test_from_graph_drops_epistemic_horizon_shadow_attrs`,
    lines 751-773: writes all three onto a graph node, reconstructs a `Territory`, asserts
    `not hasattr(restored_territory, "mass_receptivity")` etc.).
  - No `src/babylon/engine/systems/*.py` reads these three attribute names off a live graph node
    (grep-confirmed across the whole `engine/systems/` tree). Since `EpistemicHorizonSystem` runs
    last, there is no same-tick downstream System to read them anyway; since they're dropped on
    round-trip, there is no next-tick read path either.
  - **A genuinely independent production consumer exists, but it does not read the stored attr:**
    `engine/actions/investigate.py::resolve_investigate` (line 94) calls `mass_receptivity_of`
    directly — the same pure helper this system calls — **re-computing** M_r fresh against live
    graph state at OODA/action-resolution time (**position 14**, i.e. *before* this tick's
    `EpistemicHorizonSystem` run at 22.0), to gate the player's INVESTIGATE verb
    (`investigate_min_receptivity`, fog-of-war.yaml's SOCIAL_INVESTIGATION rule). This is a shared
    *formula*, not a shared *graph attribute* — the two call sites never observe each other's
    output.
  - `web/game/engine_bridge.py::_carry_epistemic_horizon` (legacy, non-gating) re-invokes
    `compute_epistemic_horizon` itself against the post-round-trip graph, purely to re-derive the
    display attrs for the (legacy) web UI. Confirms the round-trip-drop mechanism is a **known,
    deliberate** design, not an accidental gap — its own docstring cites the identical
    `TERRITORY_EXCLUDED_FIELDS` mechanism (8299-8306).
  - `src/babylon/sentinels/seam/registry.py` registers `mass_receptivity`/`intel_confidence`/
    `vision_state` as `SeamScope.MAP` observables (the `observe()`/vault-projection page, III.13
    estate) at `LivenessClass.DECLARED_CONDITIONAL` (not `MUST_BE_LIVE`) — confirmed by
    `tests/unit/sentinels/test_seam_liveness.py:222-230`. This is the **projection lane**, a
    separate estate from the engine tick hash (see §6) — another confirming data point that these
    three attrs are architecturally scoped to display, not simulation state.
- **Context/service usage with no BSL equivalent:**
  - `_context: ContextType` (the `TickContext`) is accepted but **never read** — the parameter name
    is underscore-prefixed (`epistemic_horizon.py:230`). Zero `TickContext` dependency, unlike
    Territory's `displacement_mode` — nothing to port here.
  - `player_org_id` is read from `graph.graph["player_org_id"]` — **graph-level metadata**, not a
    node/edge attribute (`epistemic_horizon.py:239-240`). This is architecturally distinct from
    Territory's `TickContext.displacement_mode` (a per-run harness override that's provably always
    one value in production): `player_org_id` is core WorldState identity data ("which node is the
    player"), set legitimately by two scenario factories that DO seed it
    (`create_us_scenario`/`_legacy.py:1050`, `create_wayne_county_scenario`/`_legacy_wayne.py:743`)
    — but **neither factory is wired into the canonical `SCENARIOS` dict**
    (`tools/regression_scenarios.py:37-133` lists only `create_imperial_circuit_scenario`,
    `create_two_node_scenario`, `create_single_county_scenario`, and the five
    `electoral_goldens.py`/`org_probe.py` factories — none of which call `player_org_id=` or
    `is_player=`, grep-confirmed zero hits for both across every canonical-scenario factory file).
    See dormancy note below.
- **DORMANCY on canonical scenarios — genuinely partial, not total, and confirmed by direct grep
  of every canonical factory:**
  - **Computation 1 (M_r) is LIVE on every canonical scenario.** All 9 canonical `SCENARIOS`
    entries seed `EdgeType.TENANCY` edges with positive-population tenant classes (confirmed:
    `create_imperial_circuit_scenario` — and hence `imperial_circuit`/`two_node`/`starvation`/
    `glut`/`fascist_bifurcation`, all built from it — has TENANCY edges at `_legacy.py:477-491`
    ["Workers occupy territories"]; `single_county.py:125`; `org_probe.py:106`; the four
    `electoral_goldens.py` factories build off the same Wayne-seeded terrain). So `mass_receptivity`
    genuinely computes a real, varying, non-`None` value on canonical runs — this is **not** a
    Territory-Phase-2/3/4-style "no vector exercises this at all" gap.
  - **Computation 2 (C_p / cadre_presence) is PROVABLY 0.0 on every canonical scenario, both
    branches.** No canonical `SCENARIOS` factory ever sets `player_org_id` on the returned
    `WorldState`, and no canonical factory ever sets `is_player=True` on any organization
    (grep-confirmed zero hits for `player_org_id\s*=` and `is_player` across every file under
    `src/babylon/engine/scenarios/` reachable from the canonical dict). `to_graph()` does synthesize
    real PRESENCE edges from `org.territory_ids`/`inst.territory_ids` (`world_state.py:748-762`), so
    the loop at `compute_epistemic_horizon:174` may iterate real edges — but neither match condition
    can ever fire, so `cadre_presence` is provably, structurally `0.0` on every byte-gated tick
    today. This is the same "provably uniform on every live path" shape Territory's
    `TickContext.displacement_mode` finding used — a `:const`-style D-record candidate for a
    hypothetical port, **not** a scenario-coverage gap to close (the two scenario factories that DO
    exercise the live path, `create_us_scenario` and `create_wayne_county_scenario`, exist and are
    read, but are simply not wired into `qa:regression`).
  - **Computation 3 (`investigation_intel`) is provably 0.0 on every canonical scenario.**
    `tools/regression_test.py`'s `compute_tick_hash` call is hardcoded `actions=[]`
    (`regression_test.py:964`, the only `actions=` reference in the whole file) — the qa:regression
    harness never dispatches a player action, so `resolve_investigate` never runs, so
    `investigation_intel` never leaves its `Territory` model default of `0.0`
    (`territory.py:289-291`).
  - **Net effect:** on every canonical scenario, `intel_confidence` collapses to the constant
    `defines.base_observation` (`0.1`) whenever `mass_receptivity` is non-`None`, and `vision_state`
    is driven purely by the live-but-`base_observation`-independent `mass_receptivity` value against
    the two fixed thresholds. The formula's *interesting* cross-term
    (`cadre_presence * mass_receptivity`) and the earned-intel term never fire on any byte-gated
    vector — a genuine, precise dormancy finding, distinct in shape from Territory's (there,
    whole *phases* were dormant; here, one *term* of one *sub-formula* is dormant, and the
    headline M_r computation is not).

---

## 6. BLOCKER ASSESSMENT

Adjudicated against the CURRENT BSL surface stated in this task (Slice 1 query/fold/enum lane
landed on dev; Slices 2-4 not built; `exp`/`log`/`floor` intrinsics declarable; no imposed
functional forms; events unpinnable via WS1). One row per computation, plus the overall system
verdict, which supersedes the per-row detail per the SPECIAL NOTES' invited disposition.

| Computation | Verdict | Detail |
|---|---|---|
| **Whole system** (all five computations + the final write) | **NOT-A-PACK** | Every write this system makes is architecturally scoped to the `Obs: 𝒮 → Proj` projection lane, not the tick-hash-bearing `𝒮` (State) lane a `.bsl` content pack feeds through `run_once_into`. Grounded three ways: (1) `TERRITORY_EXCLUDED_FIELDS` (`world_state.py:135-137`) drops all three outputs on every `WorldState.from_graph()` call, proven by a dedicated test (`test_graph_roundtrip.py:751-773`); (2) `graph_content_hash` (`regression_test.py:924-964`) hashes `state.to_graph()` where `state` is the *post-round-trip* `WorldState` — these three attrs structurally can never reach the byte-gate, on any scenario, forever, not merely "dormant today"; (3) the governing standard already states the general law: **S-23** (`ai/bsl-architecture-standard.md:705`, citing II.8/Amendment V, `THE_FORMALISM.md:717-723`): *"`Obs: 𝒮 → Proj` is one-way. The algebra contains no morphism `Proj → 𝒮`. Fog is epistemic and stays out of the tick hash,"* proved by `mise run lint:imports` and the standing fact that "the tick hash is blind to the projection lane." No mechanism was found anywhere in `bsl-language.rst` for declaring a field that is written by a rule but excluded from `rules_hash`/the tick hash (§2.11's "what enters which hash" passage and §2.12's hashing clause both state the positive claim — every declared field's state is hashed, no exemption). Building this as `.bscn`/`.bsl` content evaluated via `run_once_into` would, by construction, put epistemic/fog data into the deterministic tick hash — the exact thing S-23 forbids. The correct target is a Rust projection-layer function operating on already-hashed post-tick state, structurally identical to the legacy `_carry_epistemic_horizon` pattern (`engine_bridge.py:8294-8347`) already living in this codebase. |
| Computation 1 — Mass Receptivity, M_r (`mass_receptivity_of`, lines 52-110) | **Formula PORTABLE (if the container question above is set aside)**, gated by two D-records | The population-weighted-mean shape maps almost exactly onto `(fold mean (typed-neighbors …) <body> :weight population)` — **confirmed evaluated, not merely typechecked**, at `rust/crates/babylon-bsl/src/evaluator.rs:844-852,1074-1140` (`fold_mean`), including a landed test using `:weight (field-of it social-class/head-count)` (evaluator.rs:2719, 2843) — structurally the same shape as `:weight population`. Two required D-records: (a) BSL's `fold mean` over an **empty** query errors loudly (`EvalCode::EmptyAggregate`, evaluator.rs:1083-1088) rather than skipping, unlike Python's honest-`None`-then-`continue` — the rule would need an `exists`-guard wrapping the fold to reproduce the skip; (b) `ideology.class_consciousness` is a nested-dict Python attribute with no flat-scalar BSL `deffield` equivalent (§3.1's seven-row closed type vocabulary has no struct type) — porting requires `ConsciousnessSystem`'s own (separate) port to first flatten it into a top-level `deffield`, a cross-pack dependency this system's port cannot resolve alone. The `class_m_r` term's kind (intensive × intensive × kind-neutral-const) is not unambiguously resolved by §3.4's stated table (only "exactly one intensive" and "both extensive" are named for `*`/`/`) — flagged UNVERIFIED, needs confirmation from the language spec's own maintainers before a port relies on it. |
| Computation 2 — Cadre Presence, C_p (lines 173-183) | **PORTABLE WITH D-RECORD via the `is_player` attribute path only; the `player_org_id`-metadata path is BLOCKED** | The `is_player`-attribute branch (`org.attributes.get("is_player", False)`) is expressible as `exists (typed incidence of PRESENCE into this territory) org (= (field-of org organization/is-player) #t)`, once `organization/is-player :type bool` is declared — pure node-attribute reading, fully within the landed query lane. The `player_org_id` branch reads **WorldState-scoped graph metadata**, not a node/edge attribute — there is no lane in the current BSL surface for a rule to read run-scoped identity metadata analogous to `graph.graph[...]`; naming this precisely: **missing lane = run/world-scoped metadata access from within a rule** (distinct from, and not served by, node/edge attribute storage or the `:const` mechanism, which is content-time not run-time). Since the frozen system's own architecture already treats `is_player` as a legitimate (if "legacy") alternate path, routing the port through it exclusively is a documented narrowing, not an invention — worth its own D-record precisely because it silently changes nothing on any canonical scenario today (both paths are provably dead there, §5) but is a real semantic narrowing relative to `create_us_scenario`/`create_wayne_county_scenario`'s (non-canonical) player-identified worlds. |
| Computation 3 — Investigation Intel carry (line 187) | **PORTABLE NOW** | `investigation_intel` is a real, declared `Territory` field (`territory.py:289-299`, `Probability`-shaped `[0,1]`) that survives the WorldState round trip — a plain `deffield territory/investigation-intel :type probability`, `:field`-read. No gap. (Whether the *writer*, `resolve_investigate`, is itself portable is out of scope for this system's inventory.) |
| Computation 4 — Intel Confidence, I_c (lines 189-195) | **PORTABLE WITH D-RECORD (clamp is mandatory, not optional)** | The arithmetic (`base_observation + cadre_presence * mass_receptivity + investigation_intel`) is plain binary64 addition/multiplication, fully within the landed surface. The clamp is the load-bearing item: BSL's store boundary does **not** clamp on `update-node` — an out-of-range write is `E-EVAL-020`, a loud runtime failure (`bsl-language.rst:2522-2528`) — and the unclamped sum is reachable above `1.0` on real inputs (§4 item 7). The port MUST express `max(0.0, min(1.0, …))` as nested `if` (no scalar `min`/`max` exists in the grammar, per Territory's own precedent finding) *before* the `update-node` call. Omitting it is not a simplification — it is a correctness defect that would crash ticks the frozen Python system handles silently. |
| Computation 5 — Vision State (lines 197-202) | **PORTABLE WITH D-RECORD (mint a new enum)** | The three-way threshold classification itself is plain comparisons, no gap. But BSL's `Str` type is explicitly restricted ("Only `:material-basis` and vector ids. No operations," `bsl-language.rst:2363-2365`) — a bare string attribute of the Python system's shape is **flatly unrepresentable**. The port must mint a `defenum VisionState {DESERT, MUD, WATER}` the frozen Python reference never declared (transcribing the three *values* faithfully; tightening the *representation*, per §3's finding — not a computational change, so not a deviation from port-as-is in the load-bearing sense). |
| Events (none emitted) | **N/A — nothing to port** | Zero `EventType` emissions anywhere in the system (grep-confirmed); no WS1 ledger row needed. |

---

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_epistemic_horizon.py` | 620 | **Primary conformance-oracle candidate.** Direct-system unit coverage using hand-built `BabylonGraph` fixtures (`_territory`/`_tenant`/`_player_org_presence` helpers, lines 30-68) mirroring `TerritorySystem`'s TENANCY-resolution precedent. Covers defines defaults, honest-null (no tenants / zero-population tenants), the role→coefficient table (all 4 named roles + the default fallback), both cadre-presence paths, the desert/mud/water threshold boundaries, and the clamp. The direct behavioral analogue of Territory's own primary conformance file. |
| `tests/unit/engine/laws/test_law_epistemic_horizon.py` | 240 | **Property-based invariant contract** (Hypothesis). Four named laws: L1 honest inactivity (III.11), L2 M_r boundedness **with the explicit unclamped-reader caveat** (lines 17-26 — the direct source for §4 item 3's finding), L3 vision_state as a pure threshold function, L4 intel_confidence's unconditional `[0,1]` clamp. Exactly the kind of behavioral-contract law a rewrite-test regime should re-prove independent of bit-exactness. |
| `tests/unit/models/test_graph_roundtrip.py` | 1182 (relevant: `test_from_graph_drops_epistemic_horizon_shadow_attrs`, lines 751-773) | **Schema/contract test — the single most load-bearing test in this whole inventory.** Directly proves the round-trip-drop mechanism §5/§6's NOT-A-PACK verdict rests on. Not a conformance oracle for the *formula*; it is the oracle for the *architectural claim*. |
| `tests/contract/verbs/test_roundtrip.py` | 136 (relevant: line 117, `investigate_intel_boost` reference within an INVESTIGATE-verb round-trip case) | Contract-level: proves every resolver write is either a surviving model field or a documented transient. One case touches this system's defines category indirectly (via `resolve_investigate`, not `EpistemicHorizonSystem` itself). |
| `tests/unit/engine/test_system_order.py` | 300 (relevant: lines 97, 192, 200-213, 266) | Schema/ordering test — pins `EpistemicHorizonSystem` as literally the last system in `_DEFAULT_SYSTEMS` (`test_epistemic_horizon_runs_last`, lines 200-213). Confirms tick position; not a formula conformance oracle. |
| `tests/unit/kernel/test_node_access.py` | 70 | Unit test for the **shared** `class_consciousness_from_node` helper (also exercised by SolidaritySystem/StruggleSystem/ImperialRentSystem) — schema-level for a dependency, not this system's own conformance surface. |
| `tests/unit/projection/fog/test_investigate_wiring.py` | 140 | Contract test for the INVESTIGATE verb's fog-ledger wiring (a different module, `resolve_investigate`) — adjacent, not this system. |
| `tests/unit/projection/fog/test_reach.py` | 258 (relevant: lines 162, 196, `defines.epistemic_horizon.organizing_reach_radius`) | Tests the `organizing_reach` primitive (a different module) — touches this system's defines *category* for an unrelated define (`organizing_reach_radius`, not read by `EpistemicHorizonSystem` itself). |
| `tests/unit/projection/test_veil_gating.py` | 143 | Fog+vision-gate composition at the projection layer — narrative/display, not engine math; further confirming evidence for the projection-lane disposition (§6). |
| `tests/unit/sentinels/test_seam_liveness.py` | 398 (relevant: lines 222-244) | Sentinel-2 liveness check registering `mass_receptivity`/`vision_state` as `SeamScope.MAP`/`DECLARED_CONDITIONAL` observables — projection-layer plumbing, cited in §5/§6 as confirming evidence, not a conformance oracle for the tick formula. |
| `tests/unit/test_public_import_surface.py` | 309 (relevant: lines 24, 48) | Pins `EpistemicHorizonDefines`'s presence in the `config.defines` package's public `__all__` — schema-level, not behavioral. |

**qa:regression byte-gate coverage: NONE, structurally, by design.** `graph_content_hash`
(`tools/regression_test.py:924-964`) hashes `state.to_graph()` off the **post-`from_graph()`**
`WorldState` — since `mass_receptivity`/`intel_confidence`/`vision_state` are unconditionally
dropped by `TERRITORY_EXCLUDED_FIELDS` before that point, no value this system ever computes can
appear in the byte-gate on any scenario, canonical or otherwise. This is the one respect in which
this system's test/baseline surface differs categorically from every prior port-estate inventory in
this series (Territory, Metabolism, etc.): there the byte-gate covers the live surface but the
*canonical scenarios* don't exercise every phase; here the byte-gate structurally **cannot** cover
this system's output at all, regardless of scenario content. Any future conformance oracle for a
projection-lane implementation of this formula would need to be a **new**, dedicated fixture/golden
built against the live in-tick graph (or the `.bscn`-fixture pattern the query-evaluation train
already established), never the `qa:regression` dense/hash goldens.

---

## Adjudication (2026-08-12)

Adjudicated against the current dev tree (`9324482f`). The NOT-A-PACK verdict is **right**, and
its every load-bearing anchor checks out verbatim — but it rests on a three-legged argument of
which only one leg actually bears weight, its Computation-1 row misses the same D102 refusal the
sibling wealth-distribution inventory correctly catches for the identical `SocialRole` fold, and
its §5 canonical-scenario arithmetic is wrong twice. Three corrections and two confirmations.

1. **CORRECTION — Computation 1's `class_factor` lookup hits D102; the row names two required
   D-records and misses the hard third.** The per-tenant `role_factor.get(role,
   class_factor_default)` (`epistemic_horizon.py:73-78,97-102`) reads the role of the **iterated
   tenant**, not of the rule's subject — the subject here is the TERRITORY, and the tenants are
   the fold elements. Under §3's own recommendation to declare `deffield social-class/role :type
   enum :of SocialRole`, that read is `(field-of it social-class/role)`, refused at content LOAD
   citing D102 by name (`docs/reference/bsl-language.rst:2274-2284`;
   `rust/crates/babylon-bsl/src/typecheck.rs:246-280`; wired at `rule_pipeline.rs:51,293-301`;
   vectored at `tick.rs:1304-1335`). The landed weighted-mean test this row leans on weights by
   `(field-of it social-class/head-count)` (`evaluator.rs:2719,2843`) — a **non**-enum field,
   which is exactly why it evaluates. So "Formula PORTABLE (if the container question above is
   set aside)" is wrong as written: with the enum declaration §3 recommends it is BLOCKED, and
   without it (role stored as an int ordinal) it is portable only under a third D-record that
   contradicts §3's own recommendation. The sibling `wealth-distribution-port-phase1-inventory`
   identifies this exact refusal for the same enum on the same node type; this row should reach
   the same finding.
2. **CORRECTION — leg (1) of the three-way NOT-A-PACK grounding does not carry the weight
   assigned to it.** `TERRITORY_EXCLUDED_FIELDS` (`src/babylon/models/world_state.py:94-150`) is
   an `extra="forbid"` transient-attribute list, not an epistemic register. It also drops
   `p_acquiescence` and `p_revolution` (`:96-97`) — `SurvivalSystem` @15.0's core material
   outputs, which **this very system reads as material state** — plus `wage_pressure` /
   `dispossession_intensity` (`:120-121`), `habitability` (`:114`), `infrastructure` (`:129`) and
   `price_divergence` (`:143`). Membership therefore proves per-tick transience, not
   projection-lane status; applied as a criterion it would rule `SurvivalSystem` NOT-A-PACK too,
   which is plainly wrong. Leg (2) is leg (1) restated through `graph_content_hash` (independently
   confirmed: `tools/regression_test.py:924-964` hashes `state.to_graph()` off the
   post-`from_graph()` `WorldState`, and excludes `g.graph` metadata besides, `:934-941`). **Leg
   (3) — S-23 — is the only leg that distinguishes fog from transient material state, and it does
   carry the verdict by itself:** `ai/bsl-architecture-standard.md:705` reads verbatim
   "`Obs: 𝒮 → Proj` is **one-way**. The algebra contains no morphism `Proj → 𝒮`. Fog is epistemic
   and stays out of the tick hash," sourced to II.8 / Amendment V.
3. **CORRECTION — §5's canonical-scenario arithmetic is wrong twice, though its conclusion
   survives re-derivation.** `SCENARIOS` (`tools/regression_scenarios.py:37-129`) holds **12**
   entries, not 9 — `imperial_circuit`, `two_node`, `starvation`, `glut`, `fascist_bifurcation`,
   `single_county`, `mitterrand`, `syriza`, `weimar`, `debs`, `bernie_valve`, `org_probe` — and
   `two_node` is **not** "built from `create_imperial_circuit_scenario`": it has its own factory
   (`SCENARIOS` entry at `:43-46` → `scenarios/two_node.py:26-29` →
   `_legacy.create_two_node_scenario:46`). The substantive claim survives on re-check —
   `create_two_node_scenario` seeds its own TENANCY edge (`_legacy.py:155-162`, "TENANCY edge:
   Worker occupies territory"), as do `single_county.py:125` and `org_probe.py:106` — so M_r is
   live on all 12, and the C_p / `investigation_intel` dormancy findings are unaffected.
4. **CONFIRMATION — every load-bearing anchor of the verdict, verified verbatim.**
   `mass_receptivity` / `intel_confidence` / `vision_state` sit at `world_state.py:135,136,137`
   exactly as cited; `test_from_graph_drops_epistemic_horizon_shadow_attrs`
   (`tests/unit/models/test_graph_roundtrip.py:751-773`) proves the drop with the three
   `not hasattr` assertions described. `fold mean … :weight` IS evaluated
   (`evaluator.rs:1074-1140`) and an empty aggregate IS `EvalCode::EmptyAggregate` = `E-EVAL-021`
   (`:141,201,1085`), so the `exists`-guard D-record is correctly derived. The store boundary
   genuinely does not clamp — "a written value outside the target field's declared `[0,1]` domain
   is `E-EVAL-020`, a loud failure, never a clamp"
   (`rust/crates/babylon-bsl/src/structural_verbs.rs:47-49`) — so the "clamp is mandatory, not
   optional" finding for I_c is correct and load-bearing. `Str` is "Only `:material-basis` and
   vector ids. No operations" (`bsl-language.rst:2361-2363`) and a string literal in expression
   position is `E-PARSE-010` (`:487-489`), so the mint-a-`defenum VisionState` finding is right.
   Position 22.0, last of 34, confirmed against `_SYSTEM_CLASSES` (34 entries,
   `simulation_engine.py:328-359`). The RESERVED-LINE flag on the four `class_factor_*` values is
   correctly raised and correctly not acted on.
5. **CONFIRMATION with a scope note — the M_r formula is DUAL-HOMED, and NOT-A-PACK for the
   SYSTEM must not be read as NOT-A-PACK for the FORMULA.** §5 correctly identifies
   `resolve_investigate` as an independent consumer (`src/babylon/engine/actions/investigate.py:94`)
   but understates the consequence. That call site runs **inside the tick** — the Action phase's
   resolvers are dispatched by `OODASystem` @14.0 (`engine/systems/ooda.py:317-326`,
   `resolve_player_action`) — and its output is written straight onto the territory node
   (`investigate.py:105-109`, `investigation_intel=min(1.0, existing + boost)`) as a real declared
   `Territory` field that **survives** `from_graph` (`models/entities/territory.py:289-299`; it is
   absent from `TERRITORY_EXCLUDED_FIELDS`) and is therefore hash-bearing. M_r thus sits on the
   hash-bearing side of `Obs` at one of its two call sites. Whatever lane this System lands on,
   the verb lane owes a port of this same formula — carrying correction 1's D102 finding with it.

**FINAL VERDICT: NOT-A-PACK — sustained for the SYSTEM, but on S-23 alone
(`ai/bsl-architecture-standard.md:705`); the round-trip-exclusion argument does not distinguish
fog from transient material state and must not be re-used as a projection-lane test. Two scoped
amendments the eventual disposition inherits: (i) the M_r formula is dual-homed and IS
hash-bearing via `resolve_investigate`→`investigation_intel`, so it owes a port on the verb lane
even though this System does not; (ii) Computation 1's per-tenant role read is `field-of`-on-enum
and refused at load under D102 given §3's own recommended declaration — a third, unnamed D-record
binding on whichever lane the formula eventually lands.**
