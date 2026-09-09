---
type: Reference
title: "Glossary — Core Concepts"
description: "Stable definitions of the MLM-TW theoretical primitives that drive the Babylon simulation: the dialectic D=(A,Ā,w,T,σ), Imperial Rent Φ, Survival Calculus P(S|A)/P(S|R), Lawverian algebra, hyperedge, fuel, ceremony. Each definition cites its normative source."
tags: [glossary, theory, mlm-tw, formalism]
timestamp: "2026-07-30T00:00:00Z"
---

# Glossary — Core Concepts

> **Namespace:** `Glossary` — Stable concept definitions. Updated only when a ruling/ADR moves a definition.
> **Source of truth:** `ai/THE_FORMALISM.md`, `CONSTITUTION.md` Articles II–III, Amendments D/K/AE, `ai/theory.yaml`, `ai/ontology.yaml`.

---

## The Dialectic — 𝔇 = (A, Ā, w, T, σ)

> **Definition:** The primitive generator of all structure in Babylon. A **bound opposition** between a thesis `A` and antithesis `Ā`, measured by a **weight** `w ∈ [0,1]`, evaluated at a **level** `T` in the level lattice, with a **severity** `σ` derived from kind × terminal proximity (never hand-tiered).
>
> **Constitutional status:** Primitive (Constitution II.1). All partitions emerge from it.
> **Source:** `CONSTITUTION.md` II.1; `ai/THE_FORMALISM.md` §1–§3; `ai/theory.yaml` `dialectic`.

### Components
| Symbol | Name | Type | Role |
|--------|------|------|------|
| `A` | Thesis | Opposition term | The affirmative pole (e.g., "Core", "Capital", "Repression") |
| `Ā` | Antithesis | Opposition term | The negative pole (e.g., "Periphery", "Labor", "Revolution") |
| `w` | Weight | `Probability` [0,1] | Relative force of `A` in the opposition; measured fresh each tick |
| `T` | Level | `LevelLattice` element | Scale at which the opposition is evaluated (e.g., `county`, `state`, `national`, `global`) |
| `σ` | Severity | `Severity` (derived) | `kind × terminal_proximity` — **single-sourced**, never summed |

### Constructor Families (Closed Algebra)
| Family | Symbol | Operation | Constitutional Basis |
|--------|--------|-----------|---------------------|
| **Composition** | `C` | `𝔇 × 𝔇 → 𝔇` | Combine oppositions at same level |
| **Coarse-Graining** | `G` | `𝔇 → 𝔇` | Aggregate across level lattice (county → state → national) |
| **Projection** | `P` | `𝔇 → Shadow` | One-way epistemic projection; no morphism back (Amendment S) |

> **Key law:** Every formal construct carries its **derivation chain** back to registered oppositions — that chain is its "constitutional rent record" (`NORTH_STAR.md` §3).

---

## Imperial Rent — Φ (Phi)

> **Definition:** The surplus value extracted from the Periphery (and Semi-Periphery) to the Core via unequal exchange. In the Fundamental Theorem: revolution in the Core is impossible while `W_c > V_c` (core wages > value produced by core labor). The gap `W_c - V_c = Φ` is Imperial Rent.
>
> **Mathematical form:** Tensor computation over the Leontief inverse `(I - A)⁻¹` with gamma visibility `γ` and unequal exchange ratios `σ` (Amin/Emmanuel spectrum).
>
> **Source:** `ai/imperial-rent-spec.yaml`; `src/babylon/formulas/unequal_exchange.py`; `src/babylon/domain/economics/tensor/`; `docs/concepts/imperial-rent.rst`; `docs/concepts/volume-i-theory.rst`.

### Key Properties
- **Three-tier rule** (Program 26 U5a/ADR167): Core = 0 (zero CORE OUTFLOW rows in Ricci); Semi damped by data-derived `w_semi = 0.7395`; Periphery undamped.
- **Sigma composition**: `Φ = σ_core ⊕ σ_semi ⊕ σ_periphery` — attribution lives in session bootstrap, not per-tick.
- **Negative rents**: Structural negative industry rents in Leontief are a known theory question (reserved, un-ruled per ADR172).

---

## Survival Calculus — P(S│A) / P(S│R)

> **Definition:** The two survival probabilities that drive the revolutionary rupture condition.
>
> - **P(S│A)** — Survival by **Acquiescence**: `Sigmoid(Wealth − Subsistence)`. Probability a class member survives by accepting conditions.
> - **P(S│R)** — Survival by **Revolution**: `Organization / Repression`. Probability a class member survives by revolting.
>
> **Rupture condition:** `P(S│R) > P(S│A)` → revolutionary consciousness becomes rational.
>
> **Critical ruling (ADR172 ruling 5, NORTH_STAR.md §0):** **No functional form may be imposed.** The logistic/Sigmoid shape in the Python reference is the **frozen reference's implementation**, not the going-forward law. In the Rust/BSL engine, `P(S│A)` is the **measure of class members whose wealth clears subsistence** — the S-curve **emerges** from within-class wealth dispersion; no stipulated sigmoid.
>
> **Source:** `src/babylon/formulas/survival_calculus.py`; `docs/concepts/survival-calculus.rst`; `NORTH_STAR.md` §0; `CONSTITUTION.md` II.2–II.3; `ai/theory.yaml` `survival_calculus`.

---

## Lawverian Algebra (The Closed Formalism)

> **Definition:** The topological-algebraic machinery expressing the dialectic as computable field operations on the simulation graph. Closed for v1.0 — **no new primitives without constitutional amendment** (Amendment AE re-opened for exactly one additive construct: BSL).
>
> **Core structures:**
> - **Level lattices** — county ⊂ state ⊂ national ⊂ global
> - **Opposition catalog** — finite, registered set of `(A, Ā)` pairs
> - **Adjunctions in production** — `C ⊣ G ⊣ P` (Composition ⊣ Coarse-Graining ⊣ Projection)
> - **Boundary operator ∂L** — seam space over the construct graph (finite, computable)
> - **Derived severity** — `σ = kind × terminal_proximity` (single-sourced)
>
> **Source:** `ai/THE_FORMALISM.md`; `ai/bsl-architecture-standard.md` §3; `CONSTITUTION.md` Articles II–III; `docs/concepts/dialectical-field-theory.rst`.

---

## Hyperedge (Native — Amendment D)

> **Definition:** A first-class n-ary relation in `babylon-graph`'s exposed model and type system. **Not** a clique expansion. Levi/incidence matrices are **internal storage only**.
>
> **Rulings D-1..D-7 (PR #353, ADR172):**
> - D-1: Native hyperedge in the type system (not dyadic reduction)
> - D-2: XGI compatibility layer for algorithms
> - D-3: `CommunityHypergraph` layer uses hyperedges for n-ary identity memberships
> - D-4: Incidence/Levi are private implementation details
> - D-5: Hyperedge-weighted formulas (e.g., solidarity transmission) are first-class
> - D-6: Serialization/ceremony format preserves hyperedge structure
> - D-7: II.3 transition-state marker closes
>
> **Source:** `ai/decisions/ADR172_amendment_ae_refoundation_ratified.yaml`; `docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`; `docs/reference/bsl-language.rst` §2.6, §2.8, §3.7.

---

## Fuel & Ceremony

> **Fuel:** The resource consumed to perform **ceremonies** — intentional, declared changes of meaning (e.g., adding a system, promoting a construct, changing a define). Fuel makes change **costly and auditable**.
>
> **Ceremony:** A structured, recorded event that:
> 1. Declares the change
> 2. Cites the ruling/ADR authorizing it
> 3. Runs the affected sentinel suite
> 4. Publishes the new golden baselines
> 5. Updates the construct's rent record
>
> **Constitutional basis:** Constitution §6 (Ceremonies), Amendment AE clause (viii) (save-compat reset + client-stops-carry).
> **Source:** `NORTH_STAR.md` §6; `CONSTITUTION.md` §6; `ai/architecture.yaml` `ceremony` (if present).

---

## Five Canonical Outcomes (Terminal States)

| Outcome | Trigger | Constitutional Status |
|---------|---------|----------------------|
| **REVOLUTIONARY_VICTORY** | Revolutionary consciousness percolates to national sovereignty | Canonical |
| **ECOLOGICAL_COLLAPSE** | Metabolic rift overshoot `O = C/B > 1` sustained | Canonical |
| **FASCIST_CONSOLIDATION** | Fascist faction captures state apparatus via bifurcation | Canonical |
| **RED_OGV** (Old Guard Victory) | Reformist electoral path liquidates revolutionary potential | Canonical |
| **FRAGMENTED_COLLAPSE** | Balkanization → warlord trajectory → no coherent successor | Canonical |

> **Source:** `src/babylon/engine/observers/endgame_detector.py`; `docs/concepts/terminal-crisis.rst`; `docs/concepts/warlord-trajectory.rst`; `CONSTITUTION.md` II.4.

---

## Cross-References

| Concept | Glossary Page | Flavor Page | State Page |
|---------|---------------|-------------|------------|
| Dialectic 𝔇 | This page | [Theory Line](/openwiki/flavor/theory-line.md) | — |
| Imperial Rent Φ | This page | — | [Architecture](/openwiki/state/architecture.md#imperial-rent-system) |
| Survival Calculus | This page | [Theory Line](/openwiki/flavor/theory-line.md#survival-calculus) | [Systems Order](/openwiki/state/systems-order.md#survival-system) |
| Hyperedge | This page | — | [Architecture](/openwiki/state/architecture.md#topology) |
| Five Outcomes | This page | [Five Outcomes](/openwiki/flavor/five-outcomes.md) | [Architecture](/openwiki/state/architecture.md#endgame-detector) |

---

*Generated against commit `786893fc9f25954312fb654bcb67fb650bbc3820`. Sources: `CONSTITUTION.md` v3.0.0, `ai/THE_FORMALISM.md`, `ai/imperial-rent-spec.yaml`, `ai/theory.yaml`, `NORTH_STAR.md`, `src/babylon/formulas/survival_calculus.py`, `src/babylon/engine/observers/endgame_detector.py`.*