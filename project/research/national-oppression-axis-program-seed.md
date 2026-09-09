# Program seed — The National-Oppression Axis (race as oppressed nations)

**Status: RATIFIED direction (owner, 2026-07-16, AskUserQuestion ruling during Wave-6
close-out). Needs its own ADR + phased spec before code. Estimated 2-3 sessions.**

## The ruling

Wave 6's "race-disaggregated wealth" census deliverable is NOT a display column — the
owner ratified modeling race as the MLM-TW category the corpus treats it as: an
**oppressed-nations axis**, feeding super-exploitation differentials and the
NATIONAL_CHAUVINISM⟷INTERNATIONALISM opposition (the reactionary DT tag renamed by the
owner 2026-07-15 explicitly as "a stand-in for oppressor-nation chauvinism").

## Data reality (verified 2026-07-16)

All four consumer-ready ACS fact tables carry a real `dim_race` dimension:
`fact_census_income` (7.2M rows, bracket × race), `fact_census_housing` (1.35M, tenure ×
race), `fact_census_worker_class` (900k), `fact_census_poverty` (26.6M). The Wave-6 C2/C3
wires (renter share, bracket ratio) deliberately SUM across `race_id` — aggregate-only —
so this program refines them, it does not conflict.

## Design sketch (for the ADR discussion — not binding)

- **Nation composition per class per county**: SocialClass gains a composition vector over
  oppressed-nation categories (derived from ACS race distributions at hydration, county-
  keyed), NOT per-person modeling. Statics derived, motion primitive (Amendment S): the
  composition is a derived static; what MOVES is differential exploitation.
- **Super-exploitation differential**: the wage/value defect (Φ decomposition) split by
  nation composition — oppressed-nation fractions absorb a higher rate of exploitation
  (Emmanuel/Amin unequal exchange INSIDE the core; ties into Program 10 σ-spectrum).
- **Opposition wiring**: NATIONAL_CHAUVINISM⟷INTERNATIONALISM becomes a live
  `domain/dialectics` opposition (catalog + pole_measure per P19 rails) — chauvinism
  accrual already exists mechanically (FascistFactionSystem MEMBERSHIP edges).
- **Constitutional path**: Aleksandrov Test trivially passes (national oppression is a
  material relation); needs an ADR; check whether a new node/edge primitive is required
  (likely NO — composition attrs + existing oppositions suffice, avoiding an amendment).

## Anti-scope (explicit)

- No per-individual race modeling; no "diversity metric" display tourism.
- Aggregate census wires (C2/C3) stay as shipped; this program layers differentials on top.
