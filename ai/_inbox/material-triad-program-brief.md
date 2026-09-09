# The Material Triad — program brief (metabolic calculus → the territory system)

> STATUS: CONTROLLER DRAFT 2026-07-21 for BD review. Charters the execution of
> `ai/_inbox/math/metabolic-calculus.md` (v0.2, DRAFT — confers no authority until the
> amendment sitting). Triggered by BD directive 2026-07-21: "that gets into the territory
> system and we need to expand on that." Program number assigned by BD at kickoff.

## Why now — the U6 incident is the symptom

Vol I's U6 stamped `subsistence_threshold` on a `territory` fixture node (caught by the
vocabulary sentinel, fixed in-lane `c435d3a6`). The instinct was natural and the type system
said no: **subsistence/consumption is class-side** (`s_bio + s_class` × population on
`social_class`), **capacity is territory-side** (`biocapacity`, `max_biocapacity`,
`regeneration_rate`, `extraction_intensity`). That split is exactly the matter book of
§VI.1 — currently enforced only by the sentinel's allowlist, not by a typed sort. The paper
turns this folk knowledge into schema: the `Matter` sort + the typed triad make the U6 bug
class *inexpressible* instead of merely *detectable*.

## Ground truth (inventory 2026-07-21, main checkout, anchors verified)

- **Matter-book kernels are SHIPPED**: `MetabolismSystem` @13 implements ΔB = R − (E·η)
  (`formulas/metabolic_rift.py:9`), the hysteresis ratchet (`:56`, `new_max = max(0, cap −
  damage)` at `metabolism.py:118`), and O = C/B (`:90`). The `metabolic` opposition's measure
  is a pure function of these — **shippable immediately**, as the paper claims (§VI.2).
- **The matter fields are untyped**: `Territory.{biocapacity,max_biocapacity,
  regeneration_rate,extraction_intensity}` are plain floats/Currency
  (`models/entities/territory.py:153-175`); `habitability` isn't a model field at all —
  graph-only via `EXTRA_STAMPABLE_ATTRIBUTES` (`sentinels/vocabulary/registry.py:205-228`).
- **No `metabolic`/`somatic` opposition exists or is reserved** — catalog has 11 + the seven
  Vol I/II reserved keys only (`instances/catalog.py:459-666`).
- **DEFECT A — consumption is seed-static, so the overshoot signal has no dynamics.**
  CORRECTED 2026-07-21 (verified survey): `s_bio`/`s_class` ARE declared `SocialClass`
  fields (`social_class.py:374-379`), seeded at scenario build — but **no system ever
  evolves them**. Vitality (`vitality.py:154,230`), Metabolism, and EndgameDetector
  (`endgame_detector.py:466`) all consume a constant, so C in O = C/B only moves with
  population. The never-fires glut finding (2026-07-19) traces to static magnitudes plus
  DEFECT B's twin thresholds. W1 gives C real dynamics; the durable fix is W5's J_soma
  somatic floor (consumption as a produced, demographically-scaled matter-book quantity).
- **DEFECT B — two independent overshoot thresholds.** `MetabolismDefines.overshoot_threshold`
  (drives the event) vs `EndgameDefines.ecological_overshoot_threshold` (drives the
  ECOLOGICAL_COLLAPSE axis, recomputed from WorldState at `endgame_detector.py:448-487`).
  They can disagree; nothing reconciles them.
- **DEFECT C — the legitimation trajectory is memoryless** (survey 2026-07-21):
  `LifecycleSystem` reads `legitimation_state` every tick (`lifecycle.py:266`) but never
  writes it back (its `update_node` calls write `dpd_state`/`legitimation_index`/etc.
  only), and `legitimation_state` sits in `TERRITORY_EXCLUDED_FIELDS`
  (`world_state.py:96`) so it can't round-trip either — re-defaulted every tick, a broken
  accumulator. W4's L-MAT-7 is the principled fix: legitimation becomes a *legally*
  derived-fresh-per-tick quantity bounded by the s = p+i+r+t serviceability envelope,
  instead of a stateful trajectory that was never actually stateful.
- **DEAD ESTATE — hex-grain biocapacity is schema/seed-only**: `DynamicHexState.
  biocapacity_stock`, `BiocapacityType` (6 categories), `InfraTerrainDefines` init/depletion
  rates all have ZERO live per-tick consumers; `aggregate_hexes_by_county` iterates an empty
  set (no production hex nodes on the engine graph — same class as the Vol II hex defect).
  Vol II U6a (d694928d) found a THIRD live hex-query citation:
  `engine/simulation_engine.py:235`, the determinism-hash hex-row collector — also an
  empty-set fold (deterministic but dead). W1 item 5's disposition covers all three.
- **Data**: `fact_hickel_erdi_annual` already ingested (L-MAT-6's bootstrap is live);
  EPA GHGRP 2010–2023 snapshot **DONE 2026-07-21** — 9 files ~49 MB at
  `/media/user/data/babylon-data/epa_ghgrp/` (all 14 per-year workbooks RY2010–RY2023, no
  gaps; sha256-pinned `MANIFEST.yaml`; parent-company xlsb, C/D/AA unit-fuel zip, ORIS
  crosswalk, L/O + I + E/S/BB/CC/LL subpart files, page snapshots; the ~60 superseded
  revision-vintage files on the archive page were deliberately NOT captured — flagged in
  the manifest if full revision history is ever wanted). EPA's current "Data Summary
  Spreadsheets" release IS the full official bulk data — spreadsheet-sized, not multi-GB.
  The rest of §VI.11's estate is keyless bulk, unqueued.

## Waves (staged by readiness; none gate Gate 3)

**W0 — underway.** GHGRP snapshot to `/media/user/data/babylon-data/epa_ghgrp/` with
pin manifest. Remaining §VI.11 keyless ingests queue as idle [P] tasks, NOT now (box busy).

**W1 — territory-system repair + the metabolic opposition (post-cascade; small; first).**
1. Settle DEFECT A: wire a real consumption producer over the seeded `s_bio`/`s_class`
   fields (W-C motion per the wiring doctrine — a system whose declared write-set evolves
   subsistence needs with conditions) and make `ECOLOGICAL_OVERSHOOT` genuinely fireable;
   regression-test the glut scenario fires it. Ceremony expected (real drift).
2. Settle DEFECT B: one derived overshoot signal, two declared consumers (or a recorded
   ruling for why event-tier and endgame-tier thresholds differ). No silent twins — the
   severity-map lesson applies verbatim.
3. Register `metabolic` (poles regeneration⇄extraction, county, antagonistic=True,
   `shadow=True` per the ADR077 promotion discipline Vol I just reused): measure
   b = (Xη−R)/(Xη+R), g = |b|, R = regeneration_rate·M̄, Xη = extraction_intensity·B·η;
   UNPOSITIONED when Xη+R = 0 (dead territory is absent, never "balanced"). Property tests.
   Coupling row `metabolic → tenancy (constrains)`.
4. L-MAT-2 property test (ratchet monotonicity — pin the shipped clamp) and the L-MAT-1
   matter-budget row in the conservation registry (residual = alarm, III.11).
5. Disposition for the dead hex-biocapacity path: sentinel it or retire-with-record; the
   empty-set county aggregation must stop looking alive.

**W2 — Amendment AB sitting (BD wordsmiths; MINOR class).** The `Matter` sort
(𝔾 ∩ [0,∞)); type the five matter fields + promote `habitability` to a declared field;
I.9 Material-Triad extension; energy-accounting principle; demographic anchor; III.4
catalog additions. **Letter corrections vs the paper**: it drafts "Amendment W" — stale
(W = III.13 ratified); AA = Windows; this takes **AB+**. Its "Part VI insert" collides
with formalism Part VI (Episteme) — BD renumbers at ratification. Modulus-2 items in W1
deliberately do NOT wait for this sitting.

**W3 — the β chart (post-T3, rides the dossier/map).** β_T/β_L from shipped data (REIS
rent share, QCEW wage share); β_J = UNPOSITIONED honest absence until W5; L-MAT-4
well-formedness test; one DeclaredView + county-dossier fence + a **MapView lens**
(the interface-shell MapView lens selector is the natural surface). Strictly G∘P —
recognizer/Archive input, never a control input (Amendment-S tripwire; the paper's own
failed-rent cut of replicator dynamics is binding).

**W4 — DPD cross-wires (engine-touching; ceremonies).** L-MAT-9 (φ_repro = Σ shadow
subsidy over P–D–P′, replacing the Meillassoux proxy; conservation-checked vs L-VAL-5);
L-MAT-7 (legitimation scissors vs the s = p+i+r+t split); A3 magic-constant rehoming
(`dual_circuit.py` 0.6/0.4/0.2 + 0.5 fertility cap + 0.1 ideology, `cohort_dynamics.py`
0.1, `inheritance.py` `_CLASS_INHERITANCE_SCALE` → GameDefines); F-3
population-conservation warn→alarm **if** T1.1's F-* closeout didn't already take it
(verify at merge).

**W5 — the energy split + county-B construction (post-1.0 unless BD pulls in).**
J_soma/J_flow/J_stock; atmospheric carbon register (graph register, no new node type);
climate closure into `regeneration_rate`/`habitability` (the balkanization write-path
template at `metabolism.py:78-88` is the precedent); EROI/ρ_pd validator envelopes
(L-MAT-8); `somatic` opposition; county-B construction (NLCD × NASS × FIA × gSSURGO zonal
fixtures reconciled to FoDaFo national series as an A4 row); the hex `BiocapacityType`
estate finally gets its consumer here or is retired; USGS MCS county apportionment
(Program 22 Wave-2) is the first implementing task per §VI.11.1. v1.0 ships the honest
absence (already the T3 dossier prescription).

**W6 — research tier.** Conversion quasi-metric d + χ Hodge split (L-MAT-5 curvature law,
L-MAT-6 EUE falsifier vs the live ERDI series); T-8's forward-completion obligation as a
property law over scenario orbits + a dedicated overshoot golden once W5 lands.

## Slotting

W1 forks after the merge cascade + its ceremonies settle (it needs the merged catalog —
Vol I's three new oppositions land first — and T1.1's stub-vs-calculator check to settle
DEFECT A cheaply). Runs parallel to T3/T5 as a small train; W3 explicitly follows T3.
Nothing here gates Gate 3 / #241 / T7 / T8. Heavy gates single-flight as always.

## BD-open questions

1. Amendment AB text + Part-numbering resolution (W2) — a sitting, not lane work.
2. W5 timing: post-1.0 (recommended) or pulled into the v1.0 tail?
3. EXIOBASE-only vs adding Eora26 for periphery resolution (§VI.11.7 item 5) — default
   stands unless the 5-RoW blocs prove too coarse.
4. DEFECT B disposition if the two thresholds turn out to be intentional tiers.
