# Consent insolvency — the Lawverian trigger for balkanization (research seed)

**Status:** research seed (Director musing, 2026-07-27). NOT chartered. Natural home:
the sovereignty-repair train (the RED_OGV inert-repair, already specced) — not P25/U13,
whose byte-identity gate forbids opening a new six-scenario-DAG coupling mid-unit.

## 1. The determined formula, in c/v/s terms

The state's legitimacy is *funded*. Its consent budget is the slice of measured surplus
it can route back as social wage — which is exactly the L-CEILING identity the engine
already computes per enactment (`formulas/politics.py::sw_deliverable`):

```
SW_deliverable = t + Φ_slice − debt_service
```

- `t` is the state's tax claim on s, from the published Marxian split `s = p + i + r + t`
  (`domain/economics/tick/graph_bridge.py`, the U1 surplus-split publication);
- `Φ_slice` is the imperial-rent cushion (`politics.phi_social_share × phi_inflow`) —
  the periphery mirror (U12-E) is this term measured at ≈ 0;
- `debt_service` is capital's prior claim on `t` (the bond channel; the O'Connor spiral
  the mitterrand golden executes).

`c` enters through accumulation: rising organic composition squeezes the profit rate,
tightening both the endogenous interest claim on `t` and `capital_tolerance` — so
accumulation's needs and legitimation's needs compete for the SAME s. The insolvency
condition:

> **Consent insolvency:** sustained `v + SW_delivered < reproduction floor` over the
> claimed classes — the state can no longer route enough of s to hold
> `P(S|A) > P(S|R)` for the population under its claim.

Operationally this is already integrated: the per-class betrayal integral
`b(c) = Σ gap` (U9 delivery ledger; `policy_delivery` register) is the accumulated
shortfall, and legitimation already decays through the turnout→refresh loop
(`electoral.py::_refresh_legitimation`) when hope dies, with the material floor
recomputed by the lifecycle dual-circuit where hydrated. A candidate normalized form:

```
insolvency(S) = Σ_c pop(c)·b(c) / (t_claim(S) + Φ_slice(S))     — dimensionless,
crossed when insolvency(S) ≥ consent_insolvency_threshold        — a new politics define
AND mean_legitimation(claimed(S)) < legitimacy_backfire_threshold
```

Every term traces to a material flow (Aleksandrov passes); zero new primitives.

## 2. The Lawverian structure

Two partitions of the territory lattice (lattice, not chain — Amendment U / #39):

- **de jure:** the ADMINISTERS DAG + CLAIMS overlays (state/federal definitions);
- **de facto:** effective control (SovereigntySystem's controller resolution,
  `query_territory_claims` rows with live control).

Legitimacy is the map that keeps the de jure partition presenting itself as the real
one. **Balkanization is not an event; it is the sustained failure of that
adjunction** — once consent insolvency crosses, de jure and de facto stop agreeing and
no re-adjunction inside the old jurisdictional lattice restores the isomorphism. This
is a candidate opposition pair for the catalog (`de_jure ⟷ de_facto` sovereignty),
i.e. a W-𝔇 wiring motion under ADR109 — entered through declared data, closed by a
sentinel row, never a bare import-and-call.

## 3. The programmatic trigger, and the player's Rupture

The governance-endgame fork (U12-D, `domain/politics/governance_endgame.py`) already
resolves RUPTURE only when *organs the player built* stand on the terrain
(`dual_power_live`: ≥2 live claimants) AND the party is uncaptured
(`institutional_pull < governance_capture_threshold`); geometry by bridges + Φ-starvation
(`RuptureGeometry.ALLENDE | SYNTHESIS`). The generalization: the SAME
insolvency crossing routes by what topology exists when it crosses —

| topology at crossing | territorial-scale outcome (recognizer axis, never adjudicated — I.11) |
|---|---|
| organs live + bridges + uncaptured party | revolutionary situation: secession-with-organs (RED_OGV axis) |
| no organs | FRAGMENTED_COLLAPSE trajectory |
| no bridges + intact electoral machine | FASCIST_CONSOLIDATION (the weimar golden's route) |

This is the class-scale bifurcation law doing at the territorial scale exactly what it
already does at the class scale. "Moving away from state/federal definitions" becomes
programmatic: CollapseTransition/EdgeTransition become eligible to move CLAIMS only
after the crossing — and the *player-triggered* Rupture is the case where the crossing
meets player-built organs, because organs are the only term the player authors.

## 4. What exists vs. the one missing coupling

LIVE: delivery→betrayal→windows→legitimation decay (U9–U13); dual-power detection
(@17.5); the fork's arm/geometry laws; the s-split publication; the periphery mirror;
consolidation_pressure (shared detector/conjuncture measure).

MISSING (the actual wiring): **consent insolvency feeding CLAIMS/secession
transitions.** `CollapseTransitionSystem` gates on Sovereign/Faction state, not on the
funded-consent formula; the balkanization seed's RED_OGV inertness
(100% SOV_EXTERIOR_NULL) is already specced for repair — that train is the natural
home. Needs: 1–2 new `politics:` defines (III.1), an ADR (no amendment — no new
primitives), a typed wiring motion with its sentinel row, and byte-safety proof that
the six qa scenarios (org-less, claims-static) never cross.

## 5. Non-goals

- No new GameOutcome member; the recognizer recognizes, never adjudicates (I.11).
- Not U13 scope: opening this coupling moves the six-scenario DAG mid-unit.
- Fog discipline: the crossing is MATERIAL state (tick hash); what the *player knows*
  about it is epistemic overlay, never engine input.
