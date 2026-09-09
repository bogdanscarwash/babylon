# Program Proposal — The Epistemic Horizon (fog of war)

**Status: RATIFIED by owner 2026-07-15** (drafted 2026-07-15 from a full corpus
mine; audit Wave 5's "biggest net-new mechanic"). **Historical grounding:** 50-agent
Marxists.org/ProleWiki sweep, owner-directed at ratification —
`project/research/history-sweep/` (`brief-eh-{mass-line,infiltration,receptivity}.md` +
`addendum-eh-rulings.md` closes rulings 3+5 with recommendations; coefficient tables inside). Sources: `ai/epochs/epoch3/fog-of-war.yaml`
(873 lines, SPEC_COMPLETE — the primary spec), `ai/brainstorms/network/percynotes2.md:190-198`
(the OODA `intelligence`/`sensor_latency` articulation), CONSTITUTION.md Article V (the
ratified Investigate sub-verbs), `reports/epochs-vision-gap-audit.md` §3b/§5 Wave 5.
Never-authored spec slot: spec-076 (`project/programs/05-catalog-execution.md:52-54`).

## Thesis (the corpus's, verbatim where it matters)

"Intelligence is not about PRESENCE but about RELATIONSHIP. You know what the masses tell
you." The State sees everywhere but cannot understand; the Revolution understands only where
it has built relationships. Mechanically:

- **Intel Confidence** `I_c = B_o + (C_p × M_r)` — public baseline + cadre-presence ×
  mass-receptivity.
- **Mass Receptivity** `M_r = (1 − P(S|A)) × I_a × C_f` — desperation (survival calculus,
  ALREADY LIVE per class) × ideological alignment × class factor (Reactionary Subject).
- **Three Vision States**: Desert (`M_r < 0.2`) — attributes not just hidden but
  **FALSIFIED** ("Masked data looks NORMAL" — the signature commitment); Mud — approximate
  values with `~`/`±` markers; Water (`M_r ≥ 0.8`) — true state, cadre protected.
- Per-tick decay (0.20/0.05/0.01 by state); three mass-line actions raise it
  (AGITATE_LOCALLY, SOCIAL_INVESTIGATION, ESTABLISH_CONTACT_NETWORK) — "You cannot SCOUT
  your way to intelligence."

## Verified engine reality (2026-07-15)

- `investigate.py` is the ONLY real code: a flat `_REVEAL_BY_NODE_TYPE` attr-name lookup
  whose `revealed` payload is **write-only telemetry** — nothing ever reads it back to gate
  what a snapshot shows (zero hits for readers repo-wide). No `mass_receptivity`,
  `intel_confidence`, masking, or decay exists anywhere.
- `narrator.py`'s `visibility` param is a documented no-op reserved for the STATE-side
  hegemony hook (spec-077 Panopticon) — a different system; do not conflate.
- All of `M_r`'s inputs except `I_a` already exist: `p_acquiescence` per class is live
  (W2 R3 exposed it); class composition is the Reactionary Subject layer (spec-071 exists).

## Owner rulings required BEFORE spec authoring (the crown of this proposal)

1. **Lineage reconciliation.** Two independent designs coexist: the epoch3
   relationship-gated continuous model vs. Constitution V's RATIFIED atomic
   Investigate(Territory|Org|Edge) sub-verbs ("one node per tick"). Proposed resolution:
   the sub-verbs stay the ATOMIC ACTION surface; the Epistemic Horizon is the CONTINUOUS
   SUBSTRATE they act against (Investigate = the corpus's SOCIAL_INVESTIGATION, gated on
   `M_r ≥ 0.3`). Needs your explicit ratification since it reinterprets a P1 load-bearing
   article.
2. **Falsification vs. III.11 Loud Failure.** Desert-state masking intentionally serves
   plausible-but-false values with no UI tell — a naive III.11 reading bans exactly that.
   Proposed resolution: a constitutional note (amendment-level) distinguishing SUBSYSTEM
   failure (loud, always) from DESIGNED in-fiction deception (deterministically derived,
   engine-adjudicated, server-side). Without this ruling the mechanic is unbuildable.
3. **Scope of `M_r`**: per-territory scalar vs. per-(org, territory)? The corpus formula is
   territory/class-scoped; multi-org implications unaddressed in the source.
4. **Storage owner** (II.11): `masked_attributes` (the TRUE hidden values) is new persisted
   state; II.8 requires the client NEVER receives true values for masked territories — the
   masked value must be computed server-side and sent AS the value.
5. **Decay re-derivation**: the corpus's per-tick rates were authored against an unspecified
   tick length; re-derive against the weekly tick.

## Constitutional guardrails (non-negotiable, already clear)

- III.7: masking/false values must be deterministic functions of committed state + seeded
  RNG — never display-time randomness. The corpus's decay formula is already tick-pure.
- II.5: the IntelligenceSystem lives in the ENGINE (adjudicates masking); the narrator only
  narrates. The Ambush Trap is engine state, not prose invention.
- III.12: the corpus's worked numeric examples (three `M_r` cases + the full Ambush Trap
  trace) seed golden baselines directly.

## Relation to the Divergence Channel (ADR072)

ADR072 (the Divergence Channel — the general "authored pole vs. material pole, shadow-first"
facility) examined whether Epistemic Horizon is an instance. **It is not — and forcing it to
be would misdescribe it.** EH is a **one-sided (material-only) partial instance**, scoped
**OUT** of the Divergence Channel's Phase 1, honestly:

- `M_r` / `I_c` / `vision_state` are a derived aggregate **VIEW** of already-material state
  (`p_acquiescence`, `class_consciousness` — those two *are* the substrate). There is no
  second, independently-tracked ground truth to audit a claim against, and no in-fiction
  "declared belief" object to serve as the authored pole. A `sigma_authored` on EH would be a
  costume, not a claim.
- EH shares only the **honest-null shadow-attr writer discipline** the Divergence Channel
  formalizes (III.11: absence is UNPOSITIONED, never a fabricated 0.0) — and it already writes
  its shadow attrs that way. No behavioral change, no dependency.

**Named forward seam (Phase 3, display layer — NOT this facility's authored pole).** EH's
"Desert falsification" (`M_r < 0.2` ⇒ the server serves a *falsified* display `M_r` that
diverges from the engine's *true* `M_r`) **is** a genuine authored↔material divergence — but it
lives on the **display/render layer (server→client)**, not the dialectical-opposition layer.
It is modeled as its own web-bridge mechanism, and its deliberate III.11 carve-out
("plausible-but-false values with no UI tell" — ruling #2 above) is a **recorded exception**,
folded into the divergence sentinel's *recorded-but-not-fixed* category, explicitly **not**
retrofitted as a `sigma_authored` pole. Scoped out *now*, named as a seam *later*.

## Proposed phasing (post-ratification)

- **Phase 0** — ADR + constitutional note (ruling 2) + this doc promoted to
  `project/programs/`.
- **Phase 1** — substrate shadow: `M_r`/`I_c` computed per tick from live inputs, written as
  honest node/graph attrs + seam rows, NO masking yet (byte-identical-safe by the P19
  shadow-attr precedent); probe CLI reports the three-state partition across scenarios —
  the data, not a threshold, is the deliverable.
  **✅ EXECUTED `483c4265` (2026-07-15, pre-ratification — ruling-free by construction).**
  `EpistemicHorizonSystem` (position 27, last), `EpistemicHorizonDefines` (8 fields),
  transient territory attrs `mass_receptivity`/`intel_confidence`/`vision_state`.
  qa:regression: all tick values byte-identical; baselines regenerated for the
  `defines_hash` metadata only (+ a harness warning-prefix bug fixed — an advisory
  hash-change was mis-counted as a hard diff). **Phase-1 findings:** wayne_county's 81
  territories = 65 desert / 16 mud / 0 water, `I_c` uniformly 0.1 — because `C_p`=0
  everywhere: NO org model outside PoliticalFaction carries a player marker (CivilSocietyOrg
  ORG001 has 2 real PRESENCE edges but no `is_player`) → **new ruling 6: promote `is_player`
  to the base Organization model or add `WorldState.player_org_id`** (prereq for Phase 2).
  Also flagged: the corpus's base_area example (M_r=0.665) says "Water" in prose but "Mud"
  by its own ≥0.8 threshold table — the system implements the table; I_a is proxied by
  `class_consciousness` (documented); 4 corpus-unnamed roles fall to an explicit
  `class_factor_default=0.0` (Phase-2 judgment call, esp. CARCERAL_ENFORCER).
  **Honest-display lenses landed (Wave-5 receptivity pair, 2026-07-15):** `mass_receptivity`
  (numeric, dedicated desert→mud→water receptivity ramp) + `vision_state` (categorical) as
  `/map/` lenses; `intel_confidence` on the territory-serializer/inspector surfaces only
  (uniformly 0.1 today, a lens would be decorative); bridge-side `_carry_epistemic_horizon`
  recompute closes the round-trip altitude gap — still ZERO masking, pure display of true
  computed values.
- **Phase 2** — reveal gating: bridge serialization filters by vision state (Mud
  approximation with markers); Investigate wired to raise `I_c` (finally reading its own
  `revealed` channel). **EXECUTED 2026-07-16** (`feature/eh-phase2-reveal-gating`):
  `WorldState.player_org_id` (ruling 6, owner-chosen option B) rides graph metadata;
  class-inspector gate (`_apply_class_vision_gate` — Desert withholds + `vision_masked`,
  Mud quantizes to a 0.4 grid = the corpus's ±0.2 margin with documented round-half-up,
  Water exact); `Territory.investigation_intel` as a real model field (accumulated
  event-sourced state; the shadow-attr version crashed `from_graph`, caught by the
  27-agent adversarial verify); Investigate gated by the corpus's SOCIAL_INVESTIGATION
  receptivity requirement (`M_r >= investigate_min_receptivity`, auto-fail below —
  "the people do not trust you").
- **Phase 3** — Desert falsification (needs ruling 2 executed) + contact networks +
  decay + the mass-line action wiring; frontend overlay (Water/Mud/Desert map treatment,
  DESIGN_BIBLE §11 laws apply — vision-state changes are hard cuts).
  **Added scope from the 2026-07-16 adversarial verify (evidence on file):**
  - **Territory/map-surface gating.** The corpus visibility table is per-TERRITORY, but
    Phase 2 (per the "inspector boundary only" scoping ruling) gates only the derived
    class-inspector surface — `_serialize_territory` and `_build_hex_state_attributes`
    still ship true `heat`/`agitation`/`crisis_phase`/`unemployment_rate` alongside the
    territory's own `vision_state`. Gating the primary map surface is a product-shaping
    change (fog on the live map lenses) that belongs with Phase 3's frontend overlay —
    NOT silently, and not before an owner look at the UX.
  - **Cadre-presence gate on Investigate.** The corpus requires "Cadre present in
    territory" alongside M_r ≥ 0.3; Phase 2 ships the M_r gate only, because NO verb or
    scenario can create PRESENCE edges today — a hard cadre gate would permanently dead
    the feature. Phase 3's contact-network/mass-line work must add the presence-
    establishing verb and then arm the second gate.
  - Naming note: the class-inspector marker is `class_vision` (a max-fold DERIVED view
    over the class's territories), deliberately distinct from the territory's own
    `vision_state` — two names, two semantics.
- qa:regression gates every phase; masking is bridge/presentation-side until Phase 3's
  engine system, which regenerates no baselines by construction (new attrs invisible to
  dense goldens) — verified pattern from P19/W3.
