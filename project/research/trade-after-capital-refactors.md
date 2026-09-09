# International Trade System — deferred until the Capital Vol I + Vol II refactors land

**Status:** DEFERRED by owner ruling 2026-07-20 ("defer the international trade system until
Capital volumes 1 and 2 refactors are completed similar to how we did volume 3"). Supersedes
the 2026-07-19 ruling (deferred until the queue clears, kickoff-proposal-only). This document
is the future plan; the former task-list entry (#41) is retired in favor of it.

## Why this sequencing

Trade — blocs, resource flows, alignment — is **international value transfer**. Its theory
layer (Emmanuel/Amin unequal exchange, the ratified σ-gradient of Program 10 / spec-107)
presupposes a complete domestic value architecture underneath:

- **Vol III (DONE, merged 2026-07-19, PR #216 / ADR089):** money, credit, the price⟷value
  scissors, endogenous interest, the distribution identity `s = p + i + r + t`. This program
  established the pattern the remaining volumes follow.
- **Vol I (NOT YET PROGRAMMED):** production of value and surplus value — the magnitudes the
  other volumes circulate and distribute. No program prompt exists yet; its
  brainstorm → spec → plan cycle is the first step of this roadmap.
- **Vol II (STAGED):** circulation and the reproduction schemas — prompt at
  `ai/_inbox/vol2-circulation-engine-program-prompt.md`, with recorded findings (dormant
  `vol2_step` two levels deep, stub-fed reproduction, two owed sentinel classes) and
  pre-reserved coupling slots (circulation→realization, reproduction→disproportionality).

Modeling cross-border transfer before domestic circulation exists would recreate exactly the
"shadow value system" defect class Vol III existed to eliminate: trade prices with no
value-grounded counterpart to diverge from. The Fundamental Theorem's imperial rent Φ — the
quantity trade ultimately moves — must be produced (Vol I), circulated (Vol II), and financed
(Vol III) before it can be transferred internationally.

## The program pattern to replicate (the Vol III template)

Each volume refactor runs as its own program on a dedicated branch
(`refactor/vol1-<name>`, `refactor/vol2-circulation`), following the Vol III shape:

1. Theory-grounded unit decomposition (U-numbered), primary sources cited per unit.
2. Shadow-first phases for anything touching live math (ADR077/078 shadow→promote
   discipline), with observe-only verification before promotion.
3. Per-unit implementer + adversarial reviewer; mutation-validated sentinels for every error
   class the program discovers (standing owner rule).
4. Byte-identical qa:regression throughout, intentional movement only via declared
   `test(baselines):` ceremonies with per-scenario drift tables.
5. One honest PR to dev; post-merge ledger/memory close-out.

## Preconditions before trade kickoff (in order)

1. Current queue branch (`feature/queue-2026-07-19`: #39 hex/scale + #42 wiring) merged.
2. qa:regression modernization merged (`refactor/qa-regression-modernization`) — trade flows
   are county/national financial quantities, exactly the coverage class the old gate was
   blind to; trade must never develop under the blind gate.
3. Parquet Phase 6 cutover complete (reference data pipeline stable; IMPORT_USE rows are
   themselves trade-adjacent BEA data).
4. **Vol I program**: brainstorm → owner-reviewed spec → plan → execution → merge.
5. **Vol II program**: execute the staged prompt (its own gates are already recorded) → merge.
6. THEN: a trade **kickoff proposal** to the owner — never auto-start (standing rule). The
   proposal inventories what exists at that point and proposes the program decomposition.

## Assets preserved for the eventual program

- Branch `101-trade-activation` (spared, untouched — the prior activation attempt; treat as
  reference material, not a base: it predates Vol III, the county lattice, and the sentinel
  families).
- Ratified specs: Program 10 / spec-107 (Spectrum of Unequal Exchange — the Amin/Emmanuel
  σ-gradient), Program 11 / spec-108 (Transport Substrate, Constitution II.13 — AIR_LINK /
  SHIPPING_LANE / ROAD / RAIL edge types).
- Amendment U's scale lattice (county atom; CZ/MSA/state parallel; state<nation) — the
  spatial frame trade flows aggregate through — plus SubstrateSystem's raw-material stocks
  (P22 minerals, criticality flags on `dim_commodity`) as the resource base.
- The seam/vocabulary/coverage sentinel families — every trade-facing bridge gets the same
  gate discipline.
- Research seeds that intersect: `energy-labor-money-simplex.md` (deferred),
  `national-oppression-axis-program-seed.md`, and the #42-C national axis shadow opposition
  (NATIONAL_CHAUVINISM ⟷ INTERNATIONALISM), whose promotion decision will likely ride the
  trade program (chauvinism is the ideological form of imperial-rent participation).

## What "done" will mean for the trade program (sketch, to be superseded by its real spec)

Blocs as graph-level alignments over sovereigns; resource flows over the transport substrate
priced by the Vol III money layer; unequal exchange as the σ-gradient acting on Vol I/II value
flows; imperial rent Φ becoming an actual inter-national transfer rather than a domestic
parameter. All of it deterministic, byte-gated, and sentinel-covered. The real definition
comes from the kickoff proposal's brainstorm — this sketch only records intent.
