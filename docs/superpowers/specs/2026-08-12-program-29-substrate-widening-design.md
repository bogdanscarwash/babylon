# Program 29 — The Substrate Widening (approved design)

**Status:** Director-approved live in-session, 2026-08-12 ("approved"). The eight structured
rulings behind this design (two structured rounds of four) are recorded verbatim in `ai/decisions/ADR198_program29_substrate_widening_charter.yaml`
(R1–R8); this spec is the program's working design document.

**Evidence base:** `reports/port-estate-survey-2026-08-12.md` (the consolidated verdict over
29 adjudicated Phase-1 inventories, `reports/port-inventories/`), `reports/territory-bsl-surface-facts-2026-08-12.md`,
`ai/decisions/ADR197_bsl_query_evaluation_slice1_handoff.yaml`.

## Problem

Zero un-ported systems grade PORTABLE NOW as a whole; twenty are BLOCKED. The blockers reduce
to five missing lanes, and Checkpoint A of the post-port refactor program (WS3 at MATERIAL
BASE COMPLETE) is unreachable until three of them exist. The dominant blocker is not a query
slice: it is the `GraphSubstrate` edge-attribute storage gap (D35/D65), a Constitution III.7
hash-widening decision, gating eleven systems.

## The rulings (summary — ADR198 is normative)

1. **R1 shape:** full symmetric edge attributes — declared, typed fields per edge type, same
   closed type vocabulary as nodes (enum included; Currency storage still refused).
2. **R2 hash:** a fifth canonical section (edge attributes), **elided when empty** — every
   existing golden/baseline/save byte-identical at landing; the elision rule versioned in the
   hash contract.
3. **R3 writes:** full `update-node` parity at landing (set/add/sub/scale through the same
   collect-then-apply machinery, D104 apply-time accumulation; enum `set` included).
4. **R4:** AG(i) attributed-membership payloads are a **separate, later ceremony**, chartered
   by Community's train (#536).
5. **R5 curves:** one curves dossier (every stipulated curve, its material meaning, a derived
   emergent reformulation per the ADR173 P(S|A)-as-measure pattern) → one Director ruling
   session.
6. **R6:** carrier-node blessed as **the** standard graph-scope idiom (a `:ceiling 1` carrier
   NodeType naming a real aggregate), documented once.
7. **R7:** identity keys are **int-encoded FIPS** (leading-zero trap D-recorded) or node-identity
   where the code named a node; no string type minted.
8. **R8:** Program 29 identity with full GitHub markup (umbrella + train issues + the
   director-gate register on project 8).

## The trains

| # | Train | Kind | Gates it opens |
|---|---|---|---|
| T1 | Carrier-node pattern doc: architecture-standard section + D-record template (R6) | docs | Wave B (~8 systems' graph-scope blockers withdrawn on landed Slice 1) |
| T2 | Query Slice 2 — dyadic edge reads: `edges`, `edge-between`, `field-of` over an EdgeRef (hash-free; strength-only until T3) | engineering | Solidarity outright; the read half of the storage class |
| T3 | Edge-attribute storage per R1–R3 — deffield edge rows, the empty-elided fifth section, `update-edge` verb parity; mutation-verified law tests; the empty-elision proof (existing goldens byte-identical) | engineering (its own landing ADR) | the 11-system storage class (write half) |
| T4 | Curves dossier → Director session → rulings ADR (R5) | research → director-gate | ReserveArmy, TickDynamics wage-pressure, Survival, Consciousness + the register's 8 curve rows |
| T5 | Structural-verb execution surface — §2.8 `add-node`/`remove-node`/`add-edge`/`remove-edge` (+ `update-edge` beyond strength, folded into T3) at tick time; the survey found five of seven refused before/at execution | engineering | CollapseTransition, EdgeTransition, the transition estate |
| T6 | TickDynamics re-read (the survey's one *required* re-read) + charter — the ~28-field ServicesProtocol boundary design, `round()` half-even, county-identity encoding per R7 | research → charter | TickDynamics |

## Port waves (interleaved)

- **Wave A (now, no lane needed):** Territory (in flight — PR A merged when its arc completes,
  PR B next), Production @3.0, Decomposition @11.0 + ControlRatio @12.0 (joint train).
- **Wave B (after T1):** the carrier-node consumers — Substrate's aggregate publish, Policy,
  MarketScissors, FieldDerivative, ContradictionField partials, and the ~8 systems whose only
  blocker was graph-scope state.
- **Wave C (after T2+T3):** Solidarity, Sovereignty, EdgeTransition, and the storage-class
  systems whose remaining blocker was edge attributes.
- **Wave D (after T4 + relevant charters):** the curve-bearing systems — ReserveArmy,
  TickDynamics (also needs T6), Survival, Consciousness, FascistFaction.
- **Wave E (per-charter):** Community (#536, after the AG(i) ceremony it charters), OODA (its
  own train split per its inventory), Doctrine (reserved-line heavy — Director pacing),
  Electoral (clock/schedule design), the shadow dispositions (WealthDistribution,
  EpistemicHorizon — per the register).

## Standing constraints

- The formalism surface stays CLOSED (AE ii): every train above widens storable state or
  documents idioms with existing constructs; nothing mints new mathematics.
- WS1 (events observable) stays at Checkpoint B per the Director's standing ratification; the
  survey's 67-emission ledger load is noted on #502.
- 24 director-gate register rows tracked as one issue: curve rows route to T4's session; the
  12 reserved-line surfaces await async Director rulings; the slice-4/AG(i) escalation is R4;
  the 3 shadow dispositions ride the register.
- Every train follows the standing verification arc (implementation → adversarial verify →
  fix rounds → delta verify → ADR181 merge) and the machine-safety law (cargo single-flight).
