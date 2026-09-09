# Program 19 — The Emergent Class Partition

**Status:** RATIFIED 2026-07-14 (ADR070; owner interrupt + two binding rulings below).
Phase 0 (this document + ADR070) and Phase 1 (Shadow Partition) execute immediately on
`feature/19-emergent-class-partition`; Phases 2/3+ are scheduled as their own sessions.
**Provenance:** three read-only explorer briefs (dialectics machinery, role-adjudication
census, math/determinism contracts) + one design synthesis, all executed 2026-07-14 against
`feature/epochs-wave1-spine`; every file:line cited was verified at that ref. Re-verify seams
if the underlying files have since changed.

______________________________________________________________________

## 1. The theory

Owner's formulation (2026-07-14, verbatim intent): "the {bourg/prole} x {core/periphery}
stuff we encoded **and extended** is itself a particular manifestation of the Lawverian
Dialectics-as-category-theory which we encoded. I think our economic engine becomes both
ideologically more principled AND more accurate."

The 2×2 typology is the k=2 special case of a derivable question: **which side of each
principal opposition does this node sit on?** The sign structure of a node's
EXPLOITATION/WAGES/TRIBUTE net flows answers it per node, per tick. Constitution I.19
already ratifies this verbatim — "All class partitions — including the {Core, Periphery} ×
{Bourgeoisie, Proletariat} schema — emerge from dialectic resolution patterns at specific
scales" — so this program is **code catching up to doctrine**, not an amendment. Amendment S
is not violated: a partition *within* the dialectic (a coarse-graining/measurement of
`OppositionState`) is the sanctioned shape, not a peer abstraction over it.

**The defect being fixed:** today the partition is *seeded, not emergent*. Scenario builders
assign `role=` by fiat; the transition engine (Feature 016) is unwired on every live path;
and the engine adjudicates on the labels (`_STRUGGLING_ROLES` gates uprisings, fascist drift
gates on the petty-bourgeois/labor-aristocracy labels, the imperial-rent circuit walks role
slots). The nouns dictate to the verbs.

**The crux, precisely located:** the dialectics registry measures each opposition as ONE
aggregate gap+balance per tick. `_build_graph_inputs` (`engine/systems/contradiction.py:184-223`)
collects per-edge/per-node pairs, and the aggregation mean is where per-node pole information
dies. Nothing anywhere assigns a node to a pole. The signed per-entity ingredient already
exists (`phi_class`/`phi_hour` sign structure, `value_form.py:193-277`) but is never evaluated
per graph node and written back.

**Relation to Program 10 (σ spectrum, Amendment N, RATIFIED · PENDING CODE):** Program 10
already rules per-node σ is "computed from data only, never assigned," with the binaries kept
as derived σ-band quantizations. This program is that ruling generalized to both principal
axes, landed shadow-first. The two programs are decoupled (§4A); Program 10's data-grounded σ
swaps into the `imperial` axis's pole_measure seam with zero consumer churn.

## 2. Owner rulings (2026-07-14, binding)

1. **Slots-as-positions, not relabeling.** Roles remain pre-seeded persistent graph
   positions — structural addresses in the circuit (Wallerstein: core-ness is a property of
   *production processes*, not the bodies occupying them). Population/wealth flows between
   slots (decomposition's existing 30/70 pattern); the `role` attr is never mutated by the
   derived partition. The derived partition reports *occupancy*: which slots currently
   instantiate each cell (0..N). Constitution I.7-aligned — quantity accumulates, membership
   flows, thresholds flip quality; relabeling would collapse that into "the label just
   changes" and force a golden FORMAT redesign for nothing.
2. **Wave 2 Rounds 2-3 pause now, resume after Phase 1 lands**, then run in parallel with
   Phases 2-3+ (interference confirmed nil: they touch presentation/history, none of the
   seven adjudication sites).

Delegated rulings (recorded): UNPOSITIONED = absence, never a fabricated σ=0.0 (III.11);
σ=0 exact ties resolve by `_lead`'s existing rule (hold previous side, default pole A on
first tick) — no new tie policy is invented anywhere in this program; `derived_class_cell`
values are pole-name pairs (e.g. `"labor:bribed"`), NOT pre-committed historical role names —
the cell→role crosswalk is Phase 1's *evidence*, not an input.

## 3. The census (what adjudicates on labels today)

Only **7 of 30 systems** read `SocialRole`; everything else is confirmed role-agnostic.

| Site | Label stands in for | Derived predicate | Phase |
|---|---|---|---|
| Reactionary LA-defection filter (reactionary.py:245) | superwaged member | existing `_is_superwaged` (:311-318) — pure WAGES-edge check | **2** |
| Production producer routing (production.py:140-193) | direct-vs-employed | `_find_employer is not None` (routing provably redundant; eligibility needs the TENANCY sweep proof) | **2** |
| ImperialRent sink (economic.py:311-388) | circuit terminus | `is_extraction_sink`: incoming EXPLOITATION/TRIBUTE ∧ no outgoing of those | 3.1 |
| Struggle `_STRUGGLING_ROLES` (struggle.py:48,303) | exploited + poor | `is_net_exploited ∧ coverage_ratio < 1` | 3.2 |
| Struggle lumpen riot gate (:459) | declassed/atomized | `has_no_productive_relation` | 3.2 |
| Struggle George-Jackson walk (:512-667) | circuit positions | topology walk + §6 multiplicity policy | 3.2 |
| Reactionary `_ENTITLED_ROLES` (:53,120) | imperial stake | `net_transfer_in > 0 ∧ solidarity_incidence < ε` (near-rename) | 3.3 |
| Community `_ROLE_TO_CLASS_POSITION` (community.py:36-44,513) | class distance | edge-topology distance | 3.4 |
| ControlRatio `_PRISONER_ROLES`/enforcer (control_ratio.py:31-59) | coerced outside wage relation / repression-paid | **highest stakes: revolution-vs-genocide branch** | 3.5 LAST |
| Decomposition slot find/create (decomposition.py:142-316) | stratum → slots | already flow-between-slots (the slots ruling's own precedent); shares the §6 helper | 3.2/3.5 |
| SocialClass role-seeded defaults (social_class.py:29-49,444-470) | reproduction cost / stake / volatility | **deferred follow-on program** — bootstrap problem; construction defaults ARE the permanent seed vocabulary | — |

Structural facts: role is never mutated at runtime (exhaustive grep); the roundtrip sentinel
pins `role` byte-for-byte (stays true forever under slots); wayne reuses C-ids under
different roles, so resolution-by-role is already mandatory and nothing depends on id→role
invariance.

## 4. Formalization (Phase 1 mechanism)

### 4A. The per-node primitive

In `domain/dialectics/core/opposition.py`, mirroring `GapReading`/`GapMeasure` (PEP 695
idiom; no collision with existing `PoleBinding`):

- **`PoleSample`** (frozen, extra=forbid): `entity_id: str`, `sigma: Balance` — one node's
  raw signed position on one axis. Emitted only for nodes with ≥1 contributing edge/attr on
  that axis.
- **`PoleMeasure[I](Protocol)`**: `(inputs: I) -> tuple[PoleSample, ...]`.
- **`PoleReading`** (frozen): `opposition_key`, `entity_id`, `side: Literal["a","b"]`, `sigma`.
- `BoundOpposition.pole_measure: PoleMeasure[I] | None = None` — inert by default.
- **`OppositionRegistry.read_poles(inputs, previous)`** — NEW sibling of `step()` (same
  inputs, same tick; `step()` untouched). Derives `side` centrally from sigma sign with
  `_lead`'s exact tie rule; output sorted by (opposition_key, entity_id).

`GraphInputs` (`instances/catalog.py`) gains additive default-`()` id-carrying fields
(`exploitation_id_pairs`, `wage_value_id_pairs`, `tenancy_id_pairs`), built inside the SAME
loops in `_build_graph_inputs` — zero extra traversal.

Catalog pole_measures: `_capital_labor_poles` (mean signed balance over all EXPLOITATION
participations, source→labor-signed / target→capital-signed); `_wage_poles` (own
wage⇄value defect sign, `_wage_value_reading` ordering); **`_imperial_poles = _wage_poles`**
— a NAMED PROXY (D5 precedent: same defect, different pole names). **Program 10 lands here**:
its data-grounded σ (OCC / capital intensity / vertically-integrated labor content) swaps in
as a new pole_measure on the `imperial` binding; `PoleReading` and every consumer are
unchanged. `tenancy`/`atomization` get no pole_measure in Phase 1.

The **class cell** = product over the two principal axes, computed only when BOTH readings
exist; a node positioned on one axis only is honestly reported as partially positioned.

### 4B. Shadow write

`ContradictionSystem._step_registry` (contradiction.py:136-163) extended — NOT a new pipeline
system; it already owns the GraphInputs/previous/registry lifecycle and stashes
`opposition_states` on the graph the same way. New node attrs (social_class only, present
only when positioned): `sigma_capital_labor`, `sigma_wage`, `derived_class_cell`. New graph
attr `pole_readings` = this tick's snapshot + next tick's `previous` (tie-break + flip
tracking). All three node attrs registered in `SOCIAL_CLASS_COMPUTED_FIELDS`
(models/world_state.py:59-80) in the same commit (`threat_score` precedent; C.1 gate).

**Byte-identical safe by construction:** dense goldens pin a fixed getter allowlist
(`tools/regression_test.py` `_DENSE_ENTITY_FIELDS`); new attributes are invisible to both
dense CSVs and sparse JSONs. qa:regression with UNMODIFIED baselines is the Phase 1 gate.

### 4C. The partition sentinel

`babylon/sentinels/partition/` — sixth sentinel, exact seam/conservation sibling shape,
`tools/sentinel_check.py partition` subcommand, **advisory tier only** in Phase 1. Reports:

- `agreement_rate` — derived cell vs an explicit `_DERIVED_CELL_TO_SEEDED_ROLES` crosswalk.
- `divergent_nodes` — `[(node_id, seeded_role, derived_cell)]`.
- `unpositioned_count` — per axis, per node-type (territories always unpositioned = expected).
- `multi_occupancy` — `{derived_cell: node_count}`; N>1 is the §6 signal, surfaced not acted on.
- `side_flip_count` — per (node_id, axis) over a run; the chattering instrument.

Optional (approved): one `SeamEntry` for `derived_class_cell` riding the
`_dominant_class_by_territory` categorical rails.

## 5. Phases, DoDs, and decision rules

**Phase 0 — this doc + ADR070.** DoD: committed. No code.

**Phase 1 — Shadow Partition** (~3 sessions). DoD: qa:regression 5/5 byte-identical,
baselines UNMODIFIED; roundtrip green; unit tests (hand-computed sigmas on a synthetic
graph; UNPOSITIONED absence; σ=0 tie-hold); `sentinel_check partition` clean on all 5
scenarios + wayne_county with the real numbers captured into §8 of this doc.

**Decision rules the Phase 1 data feeds (binding):**
1. `agreement_rate` materially lower on wayne_county than on the synthetics ⇒ the wage/value
   proxy does not generalize ⇒ BLOCKING owner escalation: pull Program 10's real σ forward
   ahead of Phase 3.1.
2. High `side_flip_count` ⇒ gates ALL of Phase 3+ until hysteresis lands or the chattering
   axis stays shadow-only. ControlRatio never ships on a flickering axis.

**Phase 2 — Redundant cutover** (1-2 sessions): the two census sites marked Phase 2, proven
by a throwaway side-by-side decision-diff harness across 5 scenarios + wayne_county. Empty
diff ⇒ cut over, baselines untouched. Non-empty ⇒ the site wasn't redundant; falls to
Phase-3 treatment. Production caveat: the eligibility claim additionally requires proving no
non-producer role ever holds a TENANCY edge in any shipped scenario. DoD: byte-identical +
both diffs attached as evidence.

**Phase 3+ — Deep cutover** (risk order per §3 table; ~10-12 sessions). Each: derive
predicate → migrate call site → regenerate dense+sparse baselines → **R-PROOF divergence
doc** (the `specs/*/proof.md` convention — regeneration is never silent) → update pinned
tests. 3.1 WILL move the golden-pinned `economy_current_super_wage_rate` — expected and
documented. Implementation is file-disjoint and parallelizable; **baseline regeneration is
single-writer, serialized at integration**. ControlRatio last, dedicated high-effort review,
never parallelized with anything touching prisoner/enforcer counts. Also here: extract the
duplicated `_find_entity_by_role` (struggle.py:196-230 ≡ decomposition.py:52-83) into one
shared multiplicity-aware helper.

**Deferred (follow-on program):** seed-default derivation (subsistence/entitlement/
volatility from derived position) — the partition needs edges, construction has none;
construction-time defaults are the intended permanent seed vocabulary.

## 6. Multiplicity policy (active Phase 3+ ONLY)

- N=0: unchanged (callers already handle None).
- N≥1 reads: population-weighted aggregate over cell occupants.
- N≥1 writes: broadcast to all occupants (strict generalization of today's N=1).
- Must-name-one (event payloads): existing `min(candidates)` lexicographic idiom verbatim
  (`_find_fascist_faction`, `_circuit_role_to_node_id` precedents).
- Phase 1 only *reports* `multi_occupancy`; the engine never acts on N>1 before the owning
  site's Phase-3 cutover.

## 7. Presentation re-keying map (post-Phase-3; do not build parallel lenses)

`_CIRCUIT_ROLES`/`_build_circuit_flows` (engine_bridge.py:876-960) and
`_dominant_class_by_territory` already resolve dynamically by role over fixed ordered tuples
with honest omission — the first re-keying targets onto `derived_class_cell`, as a contained
swap at those call sites. `_CLASS_ID_NAMES` (narrator) is pure display flavor and never needs
to change. Wave 2 may extend these tables freely in the meantime; extensions are additive.

## 8. Phase-1 findings (first probe run, 2026-07-14, 52 ticks/scenario)

`poetry run python tools/sentinel_check.py partition` (synthetics) — 9 advisory findings:

| Scenario | agreement | cells formed | unpositioned (cl / wage) | flips |
|---|---|---|---|---|
| imperial_circuit | n/a | none at final tick | 4/6 · 5/6 | C001×1, C002×1 |
| glut | n/a | none | 4/6 · 5/6 | C001×1, C002×1 |
| fascist_bifurcation | n/a | none | 4/6 · 5/6 | C001×1, C002×1 |
| starvation | n/a | none | 6/6 (all starved) · 5/6 | C001×1, C002×1 |
| two_node | **0.000** (0/1) | `capital:bribed`×1 | 0/2 · 1/2 | — |
| wayne_county (bare factory) | n/a | none | 5/5 · 5/5 @t52; 2/4 · 3/4 @t5 | — |

**Reading of the evidence:**

1. **The two_node divergence is the thesis made visible.** C001 is seeded
   `periphery_proletariat` but derives `capital:bribed` at tick 52 — the documented
   worker-overtakes-owner wealth crossover (~tick 8) means the seeded label has rotted while
   the flows moved on. This is precisely the "nouns dictating to the verbs" defect the
   program exists to fix, now measurable.
2. **Final-tick cells are sparse in the synthetics.** Only 1-2 of 6 class nodes hold
   EXPLOITATION edges, and by tick 52 the wages phase pays almost nobody (w_paid/v_produced
   presence is per-payment, so a collapsed wage relation honestly de-positions the wage
   axis). Cells likely form mid-run while wages still flow — a **windowed/any-tick cell
   report** is a natural analyzer refinement before Phase 3 uses this evidence; noted, not
   yet built.
3. **No chattering.** Flip counts are exactly 1 per positioned node over 52 ticks — the
   single genuine regime transition (the crossover), not oscillation. The §5 decision rule 2
   gate is CLEAR on synthetic evidence so far.
4. **wayne_county (bare factory) never forms a cell, at any tick.** The seed graph has 2
   EXPLOITATION edges + 1 WAGES edge over 4 classes (verified directly); at tick 5 the
   exploitation-positioned nodes and the wage-paid node are *different nodes*, so the
   both-axes cell never exists; by tick 52 the bare run has deactivated every class
   (decomposition adds a 5th node mid-run). Two consequences, feeding §5:
   - **Decision rule 1 fires in a sharper form than anticipated**: it is not that the
     wage/value proxy fails to *generalize* — the final-tick both-axes cell construction is
     too strict to produce evidence on real scenarios at all. Escalation to owner (Phase 3
     precondition): (a) add the **windowed/any-tick cell** analyzer refinement, and (b) run
     the probe at the **bridged altitude** (the real wayne game runs through the web bridge
     with DB hydration — same altitude-mismatch lesson as Seam Sensor-2); consider pulling
     Program 10's data-grounded σ forward if the wage axis stays sparse even then.
   - The 5/5-unpositioned report is itself the sentinel working as designed: absence over
     fabrication — nothing pretended to be classified.

## 9. Next-actions checklist

- [x] ADR070 + this document (Phase 0). `a38771d5`
- [x] P19.1a — types + `read_poles` + GraphInputs ids + 2 pole_measures (TDD red first). `1da23e94`
- [x] P19.1b — ContradictionSystem shadow write + `SOCIAL_CLASS_COMPUTED_FIELDS` (same commit). `938396d8`
- [x] P19.1c — partition sentinel + CLI subcommand. `7af30c74` (the optional
      `derived_class_cell` seam/inspector row is DEFERRED to the Wave 2 Round 2/3 resumption:
      with cells honestly sparse on every shipped scenario — §8.4 — an inspector row today
      would be a dead payload the seam sweep itself would flag; it rides with the categorical
      lens work where it renders something)
- [x] P19.1d — gates (full check 9767 green; qa:regression 5/5 UNMODIFIED); §8 findings;
      state.yaml + memory.
- [ ] **NEXT (Phase 3 precondition, owner-visible):** windowed/any-tick cell analyzer +
      bridged-altitude probe (§8.4); reassess Program-10 pull-forward after those.
- [ ] Resume Wave 2 Rounds 2-3 (owner ruling 2 — Phase 1 has landed).
- [ ] Schedule Phase 2 session (redundant cutover, diff harness).
