# Constitutional Amendment: The Consciousness Coupling Law (CCL)

**Date**: 2026-07-15
**Status**: Draft — for constitution check at plan gate
**Version impact**: MINOR (additive against v1.x). No existing principle is superseded.
**Amends**: Article I (Theoretical Commitments), Article II (Architecture Principles), Article V (Action Vocabulary)
**Adds**: New Article I principles; Article II state-class registry entries; Article III falsifiability entries and approved data sources; Article VIII anti-patterns
**Authorizes**: spec `consciousness-coupling` through the spec-kit pipeline (Research → Plan → Data Model → Contracts → Tasks → Implementation)
**Related**: Tri-County Amendment (Macomb bellwether, electoral validation tiers); state-repression research (backfire, consciousness routing disruption); pending v2 dialectics-primitive series (see §10)

---

## 0. Purpose

The constitution commits to three consciousness tendencies, membership-gated ideological effects, repression backfire, and the George Jackson bifurcation — but does not specify the mechanism by which consciousness derives from the material base. The state-repression research document lists "value flows → class positions → consciousness" as an unimplemented dependency. This amendment supplies the missing law.

It introduces **exactly two stored state variables** and a **routing discipline**: consciousness position evolves only through forces that are functions of the value ledger decomposed by edge mode. No verb, event, or narrative process writes consciousness directly. This makes the social layer a derived expression of c/v/s, Φ, OCC, and g₃₃ rather than a parallel system, and makes every consciousness trajectory auditable back to ledger queries.

---

## 1. Definitions

**Tendency simplex Δ².** Barycentric coordinates over vertices e_L (assimilationist-liberal), e_F (assimilationist-fascist), e_R (revolutionary). A point p ∈ Δ² satisfies p_L + p_F + p_R = 1, p ≥ 0.

**Scope of attachment.** Tendency state attaches to community hyperedges of Category 1 (contradiction pairs, both sides — SETTLER carries a position exactly as NEW_AFRIKAN does) and Category 2 (institutional exclusion). Category 3 (lifecycle phases) carries no position; whether lifecycle phase modulates mobility is deferred (OQ-5).

| Symbol | Name | Class | Stored? |
|---|---|---|---|
| P(C) ∈ Δ² | Ideological position of hyperedge C | Coefficient | **Yes** |
| Q(e) ≥ 0 | Suppressed-contradiction charge on CO-OPTIVE edge e | Accumulator quantity | **Yes** |
| A(C) ∈ Δ² | Material anchor | Derived (ledger query) | Never |
| G(C) = P − A | Tether (false-consciousness vector) | Derived | Never |
| ρ_m(C) | Reproduction share arriving via edge mode m | Derived (Volume II query) | Never |
| T(C) ≥ 0 | Temperature (mobility scalar) | Derived | Never |
| H, h | Hegemony budget and direction | Derived (state ledger) | Never |
| D(C) ∈ {L, F, R, CONTESTED} | Dominant tendency | Quality (enum) | Derived from P with hysteresis |

All pull and force terms are denominated per member-hour in MELT-basket labor-hours (dimensionless intensities after normalization), consistent with existing tensor conventions.

---

## 2. The Law

### CCL-1 — Ledger Sovereignty (routing law)

No player verb, state verb, event, or narrative process writes P or Q directly. Verbs mutate only (a) the ledger — labor time, use-values, flows — or (b) topology — edges, edge modes, membership. P evolves solely by CCL-4; Q solely by CCL-5. The AI narrative layer reads consciousness state and never writes it (conformant with **AI Observes, Never Controls**).

### CCL-2 — Anchor Law (material determination)

The anchor A(C) is a pure function of the current ledger, recomputed each Layer 0, never stored. Three pulls per member-hour:

```
a_R(C) = w₁·(s/v)_borne(C)
       + w₂·max(0, R_cost(C) − v_recv(C))          [reproduction deficit]
       + w₃·Φ_net_extracted(C)
       + w₄·g₃₃·D3_shadow_borne(C)                  [visibility-gated]

a_L(C) = w₅·Φ_net_received_stable(C)
       + w₆·max(0, v_recv(C) − R_cost(C))          [labor-aristocracy premium]

a_F(C) = w₇·Φ_net_received(C) · max(0, −d/dt Φ_share(C))   [threatened rent]
```

A(C) = normalize(a_L, a_F, a_R); normalization functional form is a GameDefines choice (OQ-4). Degenerate case (all pulls ≈ 0) resolves to the barycenter.

**Commentary.** The F-term is the load-bearing theoretical claim: the material basis of the fascist tendency is not deprivation but *rent still held and declining* — Du Bois's public and psychological wage under threat, Emmanuel's settler labor aristocracy in contraction. It predicts Macomb and it predicts that Wayne's Black proletariat, under worse deprivation with no rent share, does not trend fascist. w₂ and w₆ are the two sides of one quantity (deficit vs premium against reproduction cost), preserving conservation.

### CCL-3 — Drive Decomposition (reproduction routes identity)

The endogenous force on P is the mode-decomposition of how C's reproduction basket actually arrives, plus exposure backfire:

```
F_drive(C) = ρ_T(C)·(d_T(C) − P)                       [wage circuit]
           + ρ_S,intra(C)·(A(C) − P)                   [in-group solidarity → class-for-itself]
           + ρ_S,cross(C)·(e_R − P)                    [solidarity across the principal contradiction]
           + ρ_Coopt(C)·(e_L − P)                      [managed provision]
           + b·X_exposure(C)·(A(C) − P)                [EXTRACTIVE/ANTAGONISTIC backfire]
```

where d_T(C) sits on the L–F edge of the simplex, positioned toward e_F in proportion to the same threatened-rent trigger as a_F, toward e_L otherwise; ρ_S is split into intra-hyperedge and cross-principal-contradiction shares; X_exposure is repression events plus extractive intensity borne per member-hour.

**Commentary.** Solidarity is **not intrinsically revolutionary**. Intra-group solidaristic reproduction drives P toward the *anchor* — which routes R for exploited communities and F for rent-threatened ones (fraternal-order fascism; consistent literature: Satyanath–Voigtländer–Voth on associational density and Nazi Party entry — verify at Research gate). Only solidaristic circuits crossing the principal contradiction drive toward e_R. This is the George Jackson bifurcation stated as a force law. The backfire term implements the existing Article I commitment (repression increases collective identity) as de-hegemonization: exposure pushes P toward A.

### CCL-4 — Update and Reset Law

**Between crises** (coefficient dynamics):

```
P_{t+1}(C) = Π_Δ [ P_t(C) + η·T(C)·( F_drive(C) + F_heg(C) ) ]
F_heg(C)   = λ_H·H_alloc(C)·( h − P_t(C) )
T(C)       = τ₁·displacement_rate + τ₂·immiseration_rate + τ₃·max(0, −dΦ_realized/dt)
```

Π_Δ is projection onto the simplex; η·T is the α-smoothing analog for this coefficient. Temperature is a pure ledger derivative: the falling rate of profit and the shedding of v literally heat the phase space.

**Quality layer.** The dominant tendency D(C) = argmax P if max ≥ θ_dom, retained until a challenger exceeds the incumbent by θ_hys; otherwise CONTESTED. **All qualitative gameplay effects key on D, never on raw P.** P is a coefficient (continuous, slow, permitted); D is the quality (discrete, thresholded, hysteretic) — conformant with **Quantitative → Qualitative**.

**At crisis** (coefficient reset): the hegemony budget H collapses with Φ (CCL-6), capacitors discharge (CCL-5), and P re-equilibrates under F_drive alone over the reset window at elevated T. The direction each community lands — the bifurcation — is therefore gated by the topology term ρ_S,cross at reset time. Warsaw Ghetto corollary is unaffected: P(S|A) → 0 triggers revolt through the action layer directly, bypassing tendency dynamics by design.

### CCL-5 — Capacitor Law

Per CO-OPTIVE edge e provisioning C:

```
dQ(e)/dt = κ_Q · ρ_Coopt,e(C) · a_R(C)  −  δ_Q · Q(e)
```

Co-optive provision accumulates the revolutionary pull it suppresses, minus a slow leak (genuine absorption; δ_Q contested, see falsifiability row 3). On edge destruction — Withdraw, Attack, budget collapse — a discrete discharge event applies impulse ΔP = η·T·μ_Q·Q·(e_R − P) and sets Q → 0. Pacification stores what it suppresses.

### CCL-6 — Hegemony Budget Law

```
H_total = Σ over StateApparatus orgs of Φ_realized routed through Administer/Develop this tick
h       = faction-weighted point on the L–F edge (finance-capital → e_L, settler-populist → e_F)
```

Security-state faction allocation flows to Repress instead — converting hegemony spending into backfire risk (the coalition's internal contradiction). Allocation across communities is a state-AI strategic choice, not a constant. Consequence: tether magnitude |G(C)| is sustainable only while allocated hegemonic force balances drive; Φ contraction under TRPF mechanically snaps tethers toward anchors. Crisis ideology is a consequence of value dynamics, not a scripted event.

### CCL-7 — Coupling Gates

**Membership gating.** Any ideological force originating from a verb targeted at hyperedge C affects a node or organization n with strength scaled by overlap(n, C); zero overlap yields near-zero effect (existing Article I commitment; functional form OQ-1).

**Visibility gating.** Every Department III term in CCL-2 and CCL-3 enters multiplied by g₃₃. Rising visibility re-couples reproductive extraction to the anchors of the hyperedges that bear it (WOMEN anchor shifts R-ward as g₃₃ rises), independent of extraction magnitude.

---

## 3. State Classification (Article II conformance)

**Stored**: P (coefficient class — hyperedge-level, slow, crisis-reset), Q (accumulator quantity — edge-level, flux, discrete discharge). Neither is a primitive: they are model state, and both require **data-sourced initialization** at t₀ — P(t₀) mapped from county/tract electoral returns and survey priors at simulation start (see §6), Q(t₀) = 0.

**Derived, never stored**: A, G, ρ_m, T, H, h, D. Storing any of these violates Primitives vs Derived.

**Variance discipline.** P is intensive: it restricts, never sums. Display aggregation across H3 resolutions or hyperedge unions uses the member-hour-weighted barycentric mean (the extensive/intensive pairing), and no aggregation path may add positions.

---

## 4. Verb Channel Map (Article V conformance)

Informative — the normative content is CCL-1. Each verb reaches consciousness only through ledger or topology:

| Verb | Operation | CCL channel |
|---|---|---|
| Aid | Reroute reproduction use-values via solidaristic edges | raises ρ_S (intra or cross) |
| Mobilize | Create/upgrade edges | future ρ composition; Π₀; bifurcation gate |
| Educate | Apply force η_edu·(1 + σ·ρ_S(C))·(A − P) to gated audience | toward **anchor only**; effectiveness scales with lived solidaristic share |
| Campaign | Educate-like force, broad gating, low intensity | breadth over depth |
| Negotiate | Create CO-OPTIVE edges | capacitor formation (CCL-5) |
| Attack | Destroy c, interrupt s; provokes ANTAGONISTIC exposure | anchor shift + backfire channel |
| Investigate | Reveal state topology/heat | epistemic only; no P force |
| Move / Reproduce | Spatial topology / org Dept III allocation | indirect via ρ and ledger |
| Administer / Develop (state) | Route Φ to provision | H budget (CCL-6) |
| Co-opt (state) | Create CO-OPTIVE edges + e_L force | capacitor + hegemony |
| Repress (state) | Cut edges, remove nodes | ANTAGONISTIC exposure → backfire; topology |
| Withdraw (state) | Terminate provision | capacitor discharge |
| Research (state) | Raise OODA/targeting fidelity | no direct P force |

**Non-commutativity remark.** Because Educate's coefficient depends on ρ_S, which Aid raises, Aid∘Educate ≠ Educate∘Aid. Propaganda follows practice as a theorem of two coupled coefficients. This is a testable simulation invariant.

**Hard rule restated**: Educate moves P toward A(C), never toward e_R. A community cannot be educated toward a position its material reproduction does not support; changing the destination requires changing the ledger.

---

## 5. Falsifiability Table (Article III requirement)

| # | Law | Prediction | Null hypothesis | Distinguishing observable | Falsifying data |
|---|---|---|---|---|---|
| 1 | CCL-2 F-term | F-tendency growth concentrates where Φ share > 0 and dΦ_share/dt < 0 | F tracks deprivation level regardless of rent | Wayne Black proletariat (severe deprivation, ~zero rent) vs Macomb (declining rent) diverge in F-trajectory under similar deprivation trends | County/tract presidential swings (MIT Election Lab); ANES/CES authoritarianism panels vs QCEW wage-share and BEA transfer trajectories. Falsified if fit requires deprivation-only coupling |
| 2 | CCL-3 solidarity split | Intra-group associational density in rent-threatened communities raises F; cross-divide solidaristic reproduction raises R and suppresses F | All social capital suppresses extremism uniformly | Fraternal/militia density vs integrated-union density as divergent predictors of F-swing in matched counties | Union integration records (UAW locals, CPS supplements); civic-org rosters vs vote swings. Weimar analog: associational density → NSDAP entry (verify at Research gate) |
| 3 | CCL-5 capacitor | Terminating co-optive provision produces mobilization exceeding never-provisioned baseline | Cuts demobilize (pure resource loss) or match never-covered baseline | Post-termination protest/organizing incidence vs matched never-covered populations | Welfare-reform natural experiments; austerity–unrest panels (Ponticelli–Voth — verify); protest event data (CCC / Dynamics of Collective Action). Falsified if cuts uniformly demobilize |
| 4 | CCL-6 budget law | Sustainable |G| is bounded by Φ-funded provision; polarization *follows* budget contraction with lag | Polarization is elite-cue/media-driven, independent of fiscal capacity | Lead–lag structure of attitude shifts vs social-spending contraction across regions with different Φ exposure | BEA transfers, state/local budget series vs panel attitude data. Falsified if polarization systematically leads contraction |
| 5 | CCL-7 g₃₃ gate | WOMEN-hyperedge R-tendency co-moves with reproductive-labor *visibility*, not extraction *magnitude* | Consciousness tracks extraction level | ATUS unpaid-labor totals roughly flat while visibility proxies vary; consciousness should track the latter | ATUS time series + GSS gender-attitude panels vs care-policy salience measures |
| 6 | CCL-4 hysteresis | Tendency flips are punctuated, lagged, sticky (hysteresis loops in realignment) | Continuous proportional tracking of conditions | Punctuated vs smooth realignment in the bellwether series | Macomb presidential series 1960–2024 (already the constitutional bellwether target) |

Row 1 is the amendment's primary falsification target and doubles as the calibration benchmark.

---

## 6. Data Source Additions (Article III — approved source list)

Proposed additions, each requiring explicit ratification: **county/precinct election returns** (MIT Election Lab; Leip Atlas) — P(t₀) initialization and rows 1/6 validation; **ANES / CES / GSS** panels — tendency priors and rows 1/4/5; **CPS union supplement + historical UAW local records** — row 2; **protest event data** (Crowd Counting Consortium; Dynamics of Collective Action) — row 3. Existing approved sources (QCEW, BEA, Census/ACS, FRED, ATUS, LODES) cover all ledger-side terms. Φ calibration inherits the Hickel drain methodology already in project data (`babylon_hickel_final.csv`); the downscaling of national unequal-exchange drain to metro-scale Φ_share(C) is an open methods question (OQ-2).

---

## 7. GameDefines Registry Additions

All uncalibrated at proposal; provenance status PROVISIONAL, entered into the constants audit.

| Symbol | Meaning | Prior range | Calibration target |
|---|---|---|---|
| w₁–w₇ | Anchor pull weights | [0, 1] normalized | Row 1: Wayne/Macomb divergence, fit 2000–2016 |
| η | Base mobility | (0, 0.2] | Row 6: realignment lag |
| λ_H | Hegemony coupling | (0, 1] | Row 4: lead–lag structure |
| τ₁–τ₃ | Temperature weights | [0, 1] | Crisis-window volatility |
| θ_dom, θ_hys | Dominance threshold, hysteresis | (⅓, ⅔), (0, 0.2] | Row 6: punctuation vs smoothness |
| κ_Q, δ_Q, μ_Q | Charge rate, leak, discharge gain | ≥ 0 | Row 3 |
| b | Backfire coefficient | ≥ 0 | Repression-response episodes (repression research doc) |
| η_edu, σ | Educate gain, practice-coupling | ≥ 0 | Simulation-internal; no direct historical target — flag |

**Calibration protocol**: fit on tri-county 2000–2016; directional out-of-sample validation 2016–2024 (bellwether F-swing sign and timing; Wayne non-F under deprivation), mirroring the existing Wayne/Oakland capital-share directional test.

---

## 8. Anti-Pattern Additions (Article VIII)

- **AP-CCL-1**: Writing P or Q from any verb, event handler, or narrative process. All consciousness change routes through CCL-2..7.
- **AP-CCL-2**: Storing A, G, ρ, T, H, or D. Derived means derived.
- **AP-CCL-3**: Treating solidarity as intrinsically revolutionary. Solidaristic flows must be decomposed intra vs cross-principal-contradiction before entering any force term.
- **AP-CCL-4**: Applying ideological effects without membership gating.
- **AP-CCL-5**: Keying qualitative behavior to raw P instead of the dominance enum D — continuous quality gradients smuggled through the consciousness layer.
- **AP-CCL-6**: Summing positions across scales or communities. P is intensive; aggregate by member-hour-weighted mean for display only.

---

## 9. Consistency With Existing Commitments

**George Jackson Bifurcation**: operationalized, not modified — CCL-3's cross/intra split plus CCL-4's reset dynamics make "solidarity across the colonial divide" the literal gate variable. **Warsaw Ghetto corollary**: preserved as an action-layer override outside CCL. **Empirical/Strategic Separation**: anchors, shares, temperature, and budget are empirical (ledger); verb choices and state allocation are strategic; no strategic quantity is data-initialized except P(t₀), which initializes *state*, not *choices*. **AI Observes**: CCL-1. **Department III / g₃₃**: CCL-7 makes visibility a live political variable, per the existing profit-compression commitment. **Organization vs Institution**: organizational lines vs base positions deferred (OQ-3) — the amendment binds hyperedges only. **Repression research alignment**: backfire (CCL-3), consciousness-routing disruption (state cutting ρ_S,cross), and inter-cluster hostility (a state verb manufacturing ANTAGONISTIC edges across the divide — future mechanic) all land in existing terms.

---

## 10. Forward Compatibility — v2 Dialectics Series

The pending v2 series makes the dialectic the primitive with the tensor derived. CCL is drafted against v1.x but survives the inversion: every CCL input is a ledger query, and v2 re-derives the ledger from dialectic state, so CCL-2..7 compose unchanged. Under v2, A(C) reads naturally as the material pole of C's dialectic and P(C) as its ideological pole, with G the internal contradiction between them — the amendment's tether *is* a dialectic in v2 vocabulary. If the v2 series lands first, this amendment should be re-expressed as forces on dialectic intensities; no substantive change is expected. Flagged so neither series blocks the other.

---

## 11. Open Questions (deferred to spec Research gate)

- **OQ-1**: Overlap-gating functional form (binary membership vs Jaccard over shared hyperedges).
- **OQ-2**: Metro-scale Φ_share attribution — downscaling Hickel-style national drain to tri-county communities.
- **OQ-3**: Organizational lines — do orgs carry P distinct from their base (line struggle), or inherit the weighted mean?
- **OQ-4**: Anchor normalization form (linear share vs softmax with sharpness β) and its effect on CONTESTED prevalence.
- **OQ-5**: Lifecycle-phase modulation of mobility η (youth volatility) — requires a data source before inclusion.
- **OQ-6**: δ_Q (capacitor leak) sign and magnitude — genuine absorption vs pure suppression is an empirical question row 3 partially resolves.

---

## 12. Adoption

Per Governance: this draft enters review as a single additive amendment; ratification assigns the amendment number, bumps the constitution MINOR version, appends §6 sources to the approved list, and unlocks the `consciousness-coupling` spec at the pipeline's Research gate. External citations marked "verify at Research gate" are supporting literature, not load-bearing: the falsifiability table stands on the named federal and electoral datasets.
