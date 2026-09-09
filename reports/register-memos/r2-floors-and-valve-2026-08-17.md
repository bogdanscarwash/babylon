# Decision memos — ADR208 R2 (D-02/D-03, #564 rows 13 and 20)

Charter: ADR208 R2 (docket sitting 2026-08-17): "ADR171 SCOPE — NARROW. ADR171 ruled the
National Question line, NOT downstream mechanism transcription. The Community
SUBSTRATE_FLOOR_DEFAULTS floors … and the ReserveArmy border-valve wage throttle EACH owe a
dedicated ruling; both are queued for the next sitting with their full content presented (the
floor table with provenance; the valve mechanism spec). Neither transcription may proceed on
ADR171's authority alone."

Both memos below present code, values, and option space only. **No recommendation is offered in
either memo** — both sit on the reserved ideological line (Constitution IX.5) and the workforce
does not pick a lane. Prior workforce analysis exists for both rows
(`reports/register-memos/rows-13-16.md` row 13, `rows-17-20.md` row 20) and is cited where it
adds verified fact; this memo is self-contained and does not require reading those files.

---

# MEMO 1 — Community `SUBSTRATE_FLOOR_DEFAULTS`

**Location:** `src/babylon/models/entities/consciousness.py:356-455` (dict), consumed at
`src/babylon/engine/systems/community.py:453` via `compute_ternary_consciousness`
(`src/babylon/formulas/consciousness.py:29-109`).

## 1. The models

```python
# consciousness.py:35-48
class ProvenanceLevel(StrEnum):
    """Data quality indicator for substrate floor computation.

    Values:
        HIGH: Derived from 2+ independent proxy data sources.
        MEDIUM: Derived from 1 proxy data source.
        LOW: Estimated from related data, not direct proxy.
        SYNTHETIC: Stipulated placeholder with no data path.
    """
    HIGH = "high"
    MEDIUM = "medium"
    LOW = "low"
    SYNTHETIC = "synthetic"
```

```python
# consciousness.py:294-327
class SubstrateFloor(BaseModel):
    """Per-community-type minimum revolutionary consciousness with provenance.

    The substrate floor is consciousness that persists even when all
    organizations are destroyed — the grandmother teaching not to talk
    to cops, survival knowledge transmitted through socialization.
    """
    model_config = ConfigDict(frozen=True)

    community_type: CommunityType
    floor_value: Probability = Field(default=Probability(0.0),
        description="Minimum r regardless of org landscape")
    confidence: ProvenanceLevel = Field(default=ProvenanceLevel.SYNTHETIC,
        description="Data quality indicator")
    data_sources: list[str] = Field(default_factory=list,
        description="Named data sources used")
    computation_method: str = Field(default="",
        description="How floor was derived from proxies")
```

## 2. The complete value table (verbatim, `consciousness.py:356-455`)

All 14 `CommunityType` members have a row — this is exhaustive coverage, not a subset (verified:
`CommunityType` in `src/babylon/models/enums/community.py:39-55` enumerates exactly these 14
values, no more).

| # | CommunityType | floor_value | confidence | data_sources | computation_method |
|---|---|---|---|---|---|
| 1 | `NEW_AFRIKAN` | 0.12 | MEDIUM | `["Vera incarceration rates", "Chetty mobility atlas"]` | "midpoint of incarceration + mobility proxy range" |
| 2 | `FIRST_NATIONS` | 0.12 | MEDIUM | `["Vera incarceration rates", "Chetty mobility atlas"]` | "midpoint of incarceration + mobility proxy range" |
| 3 | `INCARCERATED` | 0.18 | MEDIUM | `["Vera incarceration rates"]` | "incarceration density proxy midpoint" |
| 4 | `CHICANO` | 0.08 | LOW | `["Chetty mobility atlas"]` | "mobility proxy estimate" |
| 5 | `WOMEN` | 0.04 | LOW | `["estimated"]` | "estimated from related community data" |
| 6 | `TRANS` | 0.06 | LOW | `["estimated"]` | "estimated from related community data" |
| 7 | `DISABLED` | 0.03 | LOW | `["estimated"]` | "estimated from related community data" |
| 8 | `QUEER` | 0.04 | LOW | `["estimated"]` | "estimated from related community data" |
| 9 | `UNDOCUMENTED` | 0.10 | LOW | `["estimated"]` | "estimated from related community data" |
| 10 | `SETTLER` | 0.0 | HIGH | `["structural (hegemonic default)"]` | "hegemonic default: no substrate revolutionary consciousness" |
| 11 | `PATRIARCHAL` | 0.0 | HIGH | `["structural (hegemonic default)"]` | "hegemonic default: no substrate revolutionary consciousness" |
| 12 | `YOUTH` | 0.0 | HIGH | `["structural (lifecycle phase)"]` | "lifecycle phase: no accumulated substrate" |
| 13 | `ADULT` | 0.0 | HIGH | `["structural (lifecycle phase)"]` | "lifecycle phase: no accumulated substrate" |
| 14 | `ELDER` | 0.02 | LOW | `["estimated (generational memory)"]` | "estimated from generational memory transmission" |

**Row count: 14. Weakest provenance: a five-way tie** — `WOMEN`, `TRANS`, `DISABLED`, `QUEER`,
`UNDOCUMENTED` all carry `confidence=LOW`, the literal `data_sources=["estimated"]` (no named
dataset at all), and the identical boilerplate `computation_method="estimated from related
community data"` (which does not name what the "related" data is). `ELDER` is adjacent but at
least names a mechanism ("generational memory transmission") rather than pure boilerplate.
`CHICANO` is also LOW but is measurably better grounded than the five-way tie — it cites a real
proxy dataset (Chetty mobility atlas) rather than "estimated". Of the five-way tie, `DISABLED`
carries the lowest floor value (0.03) if a single representative is wanted.

**Provenance distribution:** HIGH — 4 rows (SETTLER, PATRIARCHAL, YOUTH, ADULT — all zero-valued
structural claims, no proxy data at all, confidence is about the *certainty of the theoretical
claim* "no substrate," not about empirical measurement). MEDIUM — 3 rows (NEW_AFRIKAN,
FIRST_NATIONS, INCARCERATED — Vera Institute incarceration data ± Chetty Opportunity Atlas).
LOW — 7 rows (CHICANO, WOMEN, TRANS, DISABLED, QUEER, UNDOCUMENTED, ELDER — one real proxy source
at best, five of the seven cite no dataset at all).

## 3. Application mechanism (not just the values)

The floor is a parameter into `compute_ternary_consciousness`, called from
`community.py:452-459`:

```python
# community.py:451-462
# Only recompute if we have org data; otherwise keep existing
if org_landscape:
    floor_entry = SUBSTRATE_FLOOR_DEFAULTS.get(comm_type)
    floor_value = float(floor_entry.floor_value) if floor_entry else 0.0
    new_consciousness = compute_ternary_consciousness(
        community_type=comm_type,
        org_landscape=org_landscape,
        substrate_floor=floor_value,
    )
    community_states[comm_type] = state.model_copy(
        update={"consciousness": new_consciousness},
    )
```

Note the gate: the floor only fires **if `org_landscape` is non-empty** — i.e. if at least one
organization overlaps the community's member agents this tick (`community.py:437-449`, overlap
computed from `MEMBERSHIP` edges intersected with the community's agent set). If no organization
touches the community at all this tick, the prior `state.consciousness` is left untouched and the
floor is not (re-)applied that tick.

`compute_ternary_consciousness` (`formulas/consciousness.py:29-109`), four-step algorithm:

1. **Weighted sum per tendency.** For each org in `org_landscape`: `weight =
   membership_density * cadre_level * cohesion`, accumulated into `r_raw`/`l_raw`/`f_raw` by the
   org's `ConsciousnessTendency`. `total_density` accumulates the raw membership densities
   (uncapped by weight).
2. **Unorganized fraction defaults to liberal.** `unorganized = max(0, 1 - total_density)`, added
   to `l_raw` — Jackson's insight that passive non-organization is not neutral, it is liberal
   hegemony.
3. **Normalize to the simplex.** `r/l/f _norm = {r,l,f}_raw / total` (or, in the degenerate
   zero-total case, `r_norm = substrate_floor, l_norm = 1 - substrate_floor, f_norm = 0`).
4. **Apply the substrate floor, post-normalization** (`formulas/consciousness.py:97-107`):
   ```python
   if r_norm < substrate_floor:
       remaining = 1.0 - substrate_floor
       lf_sum = l_norm + f_norm
       if lf_sum > 1e-10:
           l_norm = l_norm * remaining / lf_sum
           f_norm = f_norm * remaining / lf_sum
       else:
           l_norm = remaining
           f_norm = 0.0
       r_norm = substrate_floor
   ```
   In words: if the organizationally-derived revolutionary share `r` falls below the community's
   floor, `r` is **clamped up to the floor** and the freed-up mass is **redistributed
   proportionally** between `l` and `f` so the simplex still sums to 1.0. The floor is a hard
   floor on the *r* component only — it never suppresses `r` when orgs already exceed it, and it
   never directly touches the `l`/`f` split beyond renormalizing them to fit under `1 - floor`.

**Net effect in the tick:** the floor is a per-community-type ratchet stating "revolutionary
consciousness cannot be organized away entirely, even if every revolutionary organization in this
community is destroyed" — it survives org collapse. For SETTLER/PATRIARCHAL/YOUTH/ADULT (floor
0.0) there is no ratchet: `r` can be organized to zero. For the other 10 types, `r` has a
type-specific non-zero minimum ranging 0.02–0.18 that no amount of repression against
organizations can reduce below.

## 4. Option space for the ruling (transcribed from the task; not evaluated here)

- **(a) Transcribe verbatim into BSL content at the Community port.** All 14 rows, byte-identical
  floor values, ported as declared content (defines/`.bscn` deffield or equivalent).
- **(b) Transcribe with specific rows flagged for revision.** Same port, but naming which rows
  carry provenance too weak to ratify as-is — candidates by the provenance table above: the
  five-way LOW tie (WOMEN/TRANS/DISABLED/QUEER/UNDOCUMENTED, `data_sources=["estimated"]`,
  zero named datasets) and/or ELDER (LOW, "generational memory" un-cited).
- **(c) Re-derive from the #334 incidence artifact when it lands.** #334 is open, chartered,
  unbuilt (~6 Mtok scope per the docket record); this option defers the whole table until that
  artifact exists rather than ratifying the current estimates now.

No recommendation given — reserved line (Constitution IX.5; ADR171 OQ5 already named "any national
input [to consciousness] is a future Director escalation, baseline-moving").

---

# MEMO 2 — ReserveArmy border-valve throttle ("the settler-wing wage bargain")

**Location:** `src/babylon/engine/systems/reserve_army.py` (147 lines total). Register: issue
#564 row 20.

## 1. The mechanism, verbatim

```python
# reserve_army.py:57-91 (step(), relevant excerpt)
protocol = self._wrap_graph(graph)
tick = context.tick
defines = services.defines.reserve_army
calculator = DefaultWagePressureCalculator(defines)

# P25 U9 (ADR135): the border_regime overlay is the reserve-army
# inflow valve (§2.4). PolicySystem @17.47 wrote it LAST tick
# (17.47 > 5.0 — the base reads the prior tick's superstructure by
# pipeline position, the I-ORD grain). Absent register ⟹ identical
# math — the qa six never carry it.
overlays = protocol.get_graph_attr(POLICY_OVERLAYS_ATTR, None)

for node in list(protocol.query_nodes(node_type=NodeType.TERRITORY)):
    data = node.attributes
    reserve_ratio = data.get("reserve_ratio", 0.0)
    if not isinstance(reserve_ratio, (int, float)):
        continue
    reserve_ratio = float(reserve_ratio)
    if reserve_ratio <= 0.0:
        continue

    if overlays:
        border = self._border_valve(protocol, node.id, overlays)
        if border > 0.0:
            # A tighter border throttles the reserve army's
            # replenishment: the effective ratio shrinks, wage
            # pressure eases — the settler-wing wage bargain.
            reserve_ratio *= 1.0 - border
            if reserve_ratio <= 0.0:
                continue

    wage_pressure = calculator.compute_wage_pressure(reserve_ratio)
    ...
```

```python
# reserve_army.py:123-147 (_border_valve, in full)
@staticmethod
def _border_valve(
    graph: GraphProtocol,
    territory_id: str,
    overlays: dict[str, Any],
) -> float:
    """The territory's effective border_regime magnitude, [0, 1].

    Read from the territory's TOP claims-holder's overlay row
    (PolicySystem @17.47, prior tick — P25 U9, ADR135). No claims or
    no overlay ⟹ 0.0 (an open valve, the pre-U9 behavior).
    """
    rows = graph.query_territory_claims(territory_id)
    if not rows:
        return 0.0
    sovereign_axes = overlays.get(rows[0][0])
    if not isinstance(sovereign_axes, dict):
        return 0.0
    border = sovereign_axes.get("border_regime")
    if not isinstance(border, dict):
        return 0.0
    magnitude = border.get("magnitude")
    if not isinstance(magnitude, (int, float)):
        return 0.0
    return min(1.0, max(0.0, float(magnitude)))
```

**Formula shape:** `reserve_ratio *= (1.0 − border)`, where `border ∈ [0,1]` is read from the
top CLAIMS-holder's `border_regime` policy overlay magnitude for that territory (0.0 if no
overlay/claims exist — an "open valve"). A tighter border (`border → 1`) shrinks the effective
reserve-army ratio toward 0, which then feeds `DefaultWagePressureCalculator.compute_wage_pressure`
— a smaller `reserve_ratio` produces less downward `wage_pressure`, i.e. wages hold up better.

## 2. Coefficients and defines

**The valve itself carries no defines-file coefficient.** It is a direct multiplicative read of
an externally-written overlay magnitude (already `[0,1]`-constrained upstream by
`PolicyAgendaItem.magnitude`, `policy.py:98`, and defensively re-clamped at
`reserve_army.py:147`). There is no `sigmoid_k`/threshold/weight specific to the valve in
`defines.yaml`.

The **downstream wage-pressure curve** the throttled ratio feeds (a separate mechanism, not the
valve) is parametrized in `src/babylon/data/defines.yaml:413-419`
(`ReserveArmyDefines`, consumed by `DefaultWagePressureCalculator.compute_wage_pressure`,
`src/babylon/domain/economics/reserve_army/calculator.py`):

| define | value | role |
|---|---|---|
| `sigmoid_k` | 20.0 | sigmoid steepness, reserve_ratio → wage_pressure |
| `sigmoid_r0` | 0.08 | reserve ratio at sigmoid midpoint |
| `wage_pressure_ceiling` | 0.5 | max wage-pressure coefficient (prevents total wage elimination) |

Formula (`calculator.py:29-65`):
```python
raw = 1 / (1 + exp(-k * (reserve_ratio - r0)))
baseline = 1 / (1 + exp(-k * (0 - r0)))   # sigmoid(0) baseline, subtracted so pressure ~0 at ratio~0
normalized = (raw - baseline) / (1 - baseline)
wage_pressure = ceiling * clamp(normalized, 0, 1)
```
These three coefficients govern the wage-pressure *curve*, not the border valve; they are cited
here only because the valve's output (`reserve_ratio`, post-throttle) is this curve's direct
input, and ADR202 R5 (below) rules specifically on this curve.

## 3. Theoretical model

The comment at `reserve_army.py:86-88` states the model directly: **"A tighter border throttles
the reserve army's replenishment: the effective ratio shrinks, wage pressure eases — the
settler-wing wage bargain."** This models the labor-aristocracy thesis mechanically: border
enforcement (immigration restriction) is read as reducing the inflow that replenishes the reserve
army of labor in a territory; a smaller reserve army exerts less downward wage pressure; the
beneficiary of that eased pressure is left unnamed in code but the comment names it politically as
"the settler wing."

Provenance of the mechanism itself: `PolicyAxis.BORDER_REGIME`
(`src/babylon/models/enums/politics.py:27-28,46`) is documented as "reserve-army inflow valve
(read-side: ReserveArmySystem — landed U9)"; the overlay is written by `PolicySystem` at position
17.47 (`policy.py`), one tick before `ReserveArmySystem` (position 5.0) reads it the *following*
tick — the register's "I-ORD grain" note. This is P25 Unit 9, ratified by **ADR135**. The
adjacent national-question framing (which wing benefits, at what magnitude) is **ADR171** —
MIM+MLP line, the B+C+I named-nations partition, bribe:deprivation ratio = 1.55 — a ruling this
memo does not re-litigate; it is cited only as the reserved-line context the Director already
holds.

**Byte-safety note:** the throttle is a no-op on every current canonical `qa:regression` scenario
— no scenario seeds a `policy_overlays` register, so `overlays` is falsy and the `if overlays:`
branch never executes (`reserve_army.py:65-66`: "Absent register ⟹ identical math — the qa six
never carry it"). A verbatim port is byte-safe by construction regardless of which option is
chosen.

## 4. Coupling to ADR202 R5

**ADR202 R5** (T4 curves ruling session, 2026-08-14) ruled on "CURVE 8, RESERVEARMY /
TICKDYNAMICS WAGE PRESSURE": **Option A — the full #491 (audit Q3) rung-ladder measure, now**,
overriding the workforce's staged-stub recommendation. Verbatim: *"The wage pressure curve is
re-derived against the stratum ladder WITH the absorption-flow producer and the organizational
arm, in one landing; the port BLOCKS behind the #491 (audit Q3) artifact rather than transcribing
an interim form."* R5 also records: the frozen `expansion_absorption` is hardcoded 0 at
`src/babylon/domain/economics/reserve_army/accumulation.py:133`; no wage floor was ruled
(super-exploitation stays expressible); ownership is the ReserveArmy port train.

**Is the valve inside or outside R5's blocked surface?** Both, in different senses — precision
matters here:

- **R5's re-derivation TARGET is `DefaultWagePressureCalculator.compute_wage_pressure`
  (`reserve_army/calculator.py`) — the sigmoid curve, NOT the border-valve overlay coupling.**
  The prior workforce memo on this exact row (`reports/register-memos/rows-17-20.md`, row 20)
  states this explicitly: *"ADR202 R5 (C8): the wage-pressure CURVE is ruled Option A … That
  ruling covers `reserve_army/calculator.py`, not this overlay coupling; both belong to the same
  ReserveArmy port train."* The valve does not feed into or out of the rung-ladder measure
  itself — it only pre-shrinks the `reserve_ratio` value that measure will eventually consume.
  In that narrow sense the valve is **outside** what R5 re-derives.
- **The valve is nonetheless part of the SAME System (`ReserveArmySystem`) and the SAME port
  train that R5 blocks behind #491.** Since the whole system's landing is gated on the #491
  artifact per R5, the valve cannot land ahead of or separately from the wage-pressure curve
  under R5's ruling as written — it ships (or doesn't) with the same landing.
- **The valve additionally carries its OWN two independent blockers**, documented in the
  phase-1 port inventory (`reports/port-inventories/reserve-army-port-phase1-inventory-2026-08-12.md`,
  §"Step 2"/finding 4): (a) ranking CLAIMS rows by the edge attribute `control_level`
  (`BabylonGraph.query_territory_claims`) needs BSL's edge-attribute read lane (Slice 2, not yet
  landed — `edges`/`edge-between`/`the` are all in `UNSERVED_EXPRESSION_HEADS`); (b)
  `policy_overlays` is an untyped, arbitrarily-nested graph-level Python dict with **no BSL
  representation at all** — not even via the carrier-node escape hatch (`bsl-language.rst`
  §"field-of the …"), since that hatch itself requires the same slice-2 `the` construct. These
  are blockers independent of #491 — they would block the valve even if the rung-ladder artifact
  existed today.

**Summary answer:** the valve is outside R5's specific re-derivation scope (it names the
sigmoid/calculator, not the overlay throttle), but it is inside the same #491-blocked
`ReserveArmySystem` port train R5 gates, and on top of that it carries two further, independent
blockers (edge-attribute reads + no graph-side-channel construct) that would hold it back even
absent the #491 gate.

## 5. Option space for the ruling (transcribed from the task; not evaluated here)

- **(a) Transcribe as-is.** Verbatim mechanism and magnitude (direction: tighter border ⟹
  smaller effective reserve ratio ⟹ less wage pressure); inert on every current qa scenario, so
  byte-safe regardless.
- **(b) Transcribe with the coefficients re-presented as a separate defines ruling.** Note: the
  valve itself has no dedicated coefficient to re-present (see §2) — its only "coefficient" is the
  externally-written `border_regime` magnitude, which is policy *state*, not a `defines.yaml`
  physics constant. This option would most directly apply to the adjacent
  `sigmoid_k`/`sigmoid_r0`/`wage_pressure_ceiling` triple that ADR202 R5 already rules on
  separately (full ladder replaces that curve), or to formally declaring the valve's political
  framing (which wing benefits) as its own ADR171-adjacent ruling artifact rather than a code
  comment.
- **(c) Hold the valve out of the first ReserveArmy landing under a D-record.** Ships the rest of
  `ReserveArmySystem` (territory iteration, the multiplicative wage write, event emission — all
  independently portable once R5's measure lands) without the border-valve coupling; the D-record
  would need to name both the ADR171 reserved-line reason and the two independent BSL-grammar
  blockers (§4) as why the valve specifically is held back.

No recommendation given — reserved line (ADR171 national-question terrain; ADR208 R2 explicitly
routes this row to a dedicated ruling rather than transcription-by-inheritance from ADR171).
