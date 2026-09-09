# Lawvere Archive Readings — evidence base for the Mirror/typed-core spec

*Five parallel full-text readings of 25 papers from the collected-works archive at
`/media/user/data/babylon-data/lawvere-master/pdfs/` (Earnshaw collection, 92 PDFs),
2026-07-22, commissioned for `2026-07-22-mirror-typed-core-spec.md`. Each section is
the reader's verbatim deliverable: per-paper core definitions with page-referenced
quotes, adversarial mappings onto Babylon's constructs, and unused constructions
with Haskell shapes. Read the spec first; come here for the receipts.*

---

## Reading 1 — Dialectics / UIAO / Aufhebung

Papers: Taking Categories Seriously (1986); Some Thoughts on the Future of Category
Theory (1991); Unity and Identity of Opposites in Calculus and Physics (1996);
Grassmann's Dialectics and Category Theory (1996); Display of Graphics … the
Hegelian Taco (1989).

Key findings (full detail in the reader deliverable, summarized here verbatim
where load-bearing):

- **Level (1991, p.6):** "By a level in a category of Being, I mean a ('downward')
  functor from it to a smaller category which has both left and right adjoints
  which are full inclusions." A UIO is such a triple; objects lie on "threads";
  skeleton/coskeleton are the idempotents L·down and R·down.
- **Aufhebung (1991, p.8, exact):** "…the longer left adjoint inclusion factors
  across the shorter right adjoint inclusion; equivalently, the higher coskeleton
  functor fixes both the skeleta and the coskeleta in the sense of the lower
  level." And: "every level has a smallest aufgehobenen level over it, which could
  reasonably be called 'the' Aufhebung of it" — a closure operator on the poset of
  levels, NOT a linear scan for a predicate.
- **Babylon's current phrasing ("aufhebung = least strictly-higher level where a
  resolution predicate holds") is a weak order-theoretic gloss** that drops the
  factoring condition; existence-of-smallest is a theorem (complete lattice of
  UIOs — Hegelian Taco Prop 16: UIOs of a presheaf category ≅ idempotent
  two-sided ideals; for a graphic monoid, ≅ left ideals), not a definition.
- **Operational Aufhebung (Taco, p.68):** "T resolves the opposites of S … iff
  every S-skeletal application is a T-sheaf, i.e. R_T L_S = L_S. The Aufhebung S'
  of S = the smallest T resolving the opposites of S." Dimension = length of the
  fixed-point iteration T_{n+1} = T'_n. Prop 17: 0' = 1.
- **The Hegelian Taco itself (Prop 19, p.70):** an 8-element, dimension-3 graphic
  monoid with known ideal chain ∅ ⊂ {0,1} ⊂ [ℓ,q,r] ⊂ [L,R] ⊂ [1] — a ready-made
  golden test fixture for any level-lattice + Aufhebung implementation.
- **Graphic identity:** a graphic category has endomorphism monoids satisfying
  xyx = xy (Schützenberger–Kimura). This identity is the finite-computability
  backbone (every left ideal two-sided, ideals idempotent, principal ideals
  faithfully represented, Mab = Ma ∩ Mb). Whether Babylon's transition monoid is
  graphic is an UNCHECKED, falsifiable design constraint.
- **UIAO (1996 Unity/Identity, p.168):** AO = parallel f,g with a single h,
  g ⊣ h ⊣ f; UIAO = AO where both are full+faithful and h is a common retraction
  ("essential localization"). "discrete/codiscrete are AO" — Babylon's L ⊣ U ⊣ R
  is a legitimate instance PROVIDED U is a genuine two-sided retraction; otherwise
  it is only an adjoint triple and the label is unearned.
- **σ (sublation) candidates with laws for free:** the image-factorization of the
  canonical 2-map between the opposite inclusions of a UIAO (1996, p.170 — the
  van der Waals "third section" model); or Isbell double-conjugation F ↦ F*#
  (1986 §7: idempotent, extensive, monotone closure; "the conjugacies are the
  first step toward expressing the duality between space and quantity").
- **w (tension) law:** w(𝔇) = 0 ⟺ the canonical map L(X) → R(X) is iso.
- **T (motion) candidate model:** Grassmann's boundary operator ∂ on a graded
  algebra — linear, graded Leibniz, degree −1, ∂² = 0 (1996 Grassmann, p.260);
  "the precise mathematical form of their relationship is not that of inverse,
  but rather adjointness" (p.257).

## Reading 2 — Space & Quantity / Cohesion

Papers: Categories of Space and Quantity (1992); Axiomatic Cohesion (2007);
Cohesive Toposes and Cantor's lauter Einsen (1994); Cohesive Toposes:
combinatorial and infinitesimal cases (2008); Volterra's Functionals (1997).

- **Extensive quantity-type (1992, p.19):** "a covariant coproduct-preserving
  functor from a distributive category to a linear category." Intensive: "a
  contravariant functor, taking coproducts to products … whose values have a
  multiplicative structure as well" (p.20). Their pairing satisfies the
  projection formula / Frobenius reciprocity / CCR (pp.21-22).
- **VERDICT — Babylon's `allocate ⊣ aggregate`:** as implemented (proportional
  split section of a split epi with aggregate ∘ allocate = id) it is NOT an
  adjunction; a genuine one needs the extremal universal property or the
  direct-image/inverse-image framing over the level-lattice-as-site. The
  conservation law is NOT a "sheaf law" (that is a limit/gluing condition over
  overlapping covers); it is precisely the **extensive-quantity coproduct law**
  over a partition. Rename; reserve "sheaf" for a real overlapping-cover gluing
  law if one is ever added.
- **Cohesion axioms (2007, Def. 2):** the four-functor string p_! ⊣ p* ⊣ p_* ⊣ p^!
  with (a) p_! preserves finite products, p^! full+faithful; (b) continuity;
  (c) Nullstellensatz "the canonical map p_* → p_! is epimorphic". Babylon's
  3-string L ⊣ U ⊣ R matches the 1994 adjoint cylinder ("Such adjoint cylinders I
  propose as the mathematical models for many instances of the Unity and Identity
  of Opposites", 1994 p.11) but LACKS the pieces functor p_! — the single largest
  structural gap; with finite hom-sets, continuity then comes free, and the
  Nullstellensatz is plausibly trivial for inhabited finite graphs.
- **Prop 5/6 (2007):** the topos of reversible reflexive graphs is sufficiently
  cohesive and infinitesimally generated — Lawvere already proved the strong
  properties for the graph-topos shape Babylon would use.
- **Atomization:** Lawvere's own invariant is the canonical map
  X → codiscrete(points(X)) being iso-or-not, classified by the five-element
  Ω of the reflexive-graph topos (1994 pp.12-13: totally in A / entering /
  leaving / excursion / outside); the scalar quantification is Babylon's own
  addition and must be justified (e.g. via [0,∞]-enrichment), not inherited.
- **Substance machinery (2007 Thm 2, 2008):** rarefied s_*X vs condensed s_!X
  with the cooling map — two dual intensive-quality constructions Babylon could
  use for faction self-interaction vs cross-tie structure.

## Reading 3 — Dynamics / State Categories / Chaos

Papers: Categorical Dynamics (1978); State Categories, Closed Categories and
Entropy (1984); State Categories and Response Functors (1986); Functorial Remarks
on the General Concept of Chaos (1984); Toposes of Laws of Motion (1997).

- **Babylon's tick IS Lawvere's primary discrete example** (1978, p.7: a single
  S-endomorphism τ, "a 'discrete time dynamical system'"); but distinguish the
  cyclic monoid ⟨T⟩ ≅ ℕ from the finitely-generated systems monoid on {S1..S30}
  — Lawvere's structural theorems bind differently to each. The 1986 cancellation
  theorem (no nontrivial cyclic/idempotent processes) needs a cancellative,
  divisible duration monoid — it does NOT apply to ⟨T⟩ ≅ ℕ.
- **Determinism has a name (1986, pp.17-18):** the discrete-op-fibration /
  unique-lifting condition; fibers are discrete; discrete-op-fibrations over C ≅
  S^C. Babylon's `advance` purity should be stated as this law with `Left
  Violation` = failed unique lift.
- **`observe` vs response functors (1986, pp.18-20):** Babylon's end-of-tick
  snapshot is the weak "output" case; a genuine response functor is compositional
  (Δ(P̄·P) = Δ(P̄)+Δ(P)) and path-dependent responses are Volterra integrals
  Q = ∫θ ds accumulated over sub-steps — the Chronicle should be built as the
  per-system mconcat with a stated law, not a snapshot.
- **"Chaos" (1984, p.4):** an observable is T-chaotic iff X → Y^T is epi —
  symbol-sequence surjectivity, NOT sensitivity/Lyapunov anything; Lawvere
  himself: several instances are "normal, reasonable behavior." DO NOT use it to
  back I.12 fold catastrophes; the entropy paper's upper-semicontinuous
  regularization m(x1,x2) = inf sup sup Δ(P) is the closer toolkit for
  threshold/discontinuity reasoning, and entropy functions ≡ V-functors on the
  Lawvere-metric state category (smallest = representable M(x0,−); "Cayley-
  Dedekind-Grothendieck-Yoneda").
- **Freeze vs identity (1986, pp.10-11):** a freeze is a possibly-costly process
  with every factorization endo — "statics as fixed points of T" must say which
  of trivial-fixed-point / frozen-fixed-point it means.
- **Universal state (1986, pp.28-30):** F_C(Y)(A) = Cat(A/C, Y) — the co-Yoneda
  continuation ceiling; "is World expressive enough" = injectivity into it.
- **Abstract/Concrete General (1997):** "a given model of Time … serves as an
  Abstract General which is accompanied by the Concrete General which is the
  category of all dynamical systems."

## Reading 4 — Logic / Hyperdoctrines / Enriched Tension

Papers: Adjointness in Foundations (1969); Equality in Hyperdoctrines (1970);
Quantifiers and Sheaves (1970); Metric Spaces, Generalized Logic, and Closed
Categories (1973); Left and Right Adjoint Operations on Spaces and Data Types
(2004).

- **The verb-grammar GADT is the generating 𝒱-graph of the base category, NOT a
  hyperdoctrine** (which needs per-type attribute CCCs, substitution, Σf ⊣ f· ⊣ Πf
  with Frobenius + Beck). Name it honestly.
- **Edges are better modeled as bimodules/profunctors** (1973 §3): heterogeneous
  sorts compose by Bellman–Fenchel convolution (ψ∘φ)(z,x) = inf_y[ψ(z,y)+φ(y,x)]
  — derived indirect relations for free, min-plus/tropical algebra, Floyd–
  Warshall as the concrete algorithm.
- **Tension gets a triangle inequality only by construction:** build Tension as
  the free [0,∞]-category on the weighted verb-graph (shortest-path closure,
  1973 pp.164-165) and "the triangle inequality IS the composition law" (table
  p.138, def p.146); if tension is computed any other way the inequality must be
  separately verified — the key adversarial fork.
- **"Unit defect as tension" is HALF a Galois connection** (1969 §4, p.14): track
  the counit defect too, or scope the claim as unit-only; better, globalize the
  poset connection into a functor-level adjunction ("basic examples of Galois
  connections are really just fragments of more global adjoints").
- **requireMembership/witnesses ≅ the Comprehension Schema** (1970, pp.12-13):
  {B:ψ} with structural map p_ψ — exact match; caveat: Frobenius fails once types
  carry internal automorphisms (groupoid counterexample, p.10-11; fix = hom
  profunctor).
- **V₀ ⊂ V adjoint triple** (1973 commentary p.3): truth-values into [0,∞] with
  both adjoints — the principled home for continuous-control/discrete-state
  (reachable :: Tension -> Bool as the π₀/right-adjoint collapse pair).
- **Import-layer poset:** a real but nearly-vacuous categorical object (a
  2-enriched category); to be non-trivial it needs enriched hom-objects or a
  comprehension adjoint — flag honestly, do not oversell.
- **Cauchy completeness** (1973, pp.163-164) as "solidarity-network saturation";
  **trace** Tr(f_*) = inf_a A(a, fa) vanishing ⟺ fixed point — ties to the
  Survival Calculus equilibrium question.

## Reading 5 — Boundaries / Graphic Toposes / Kinship

Papers: Intrinsic Co-Heyting Boundaries and the Leibniz Rule (1991); Categories
of Spaces May Not Be Generalized Spaces (1986); Qualitative Distinctions Between
Some Toposes of Generalized Graphs (1989); More on Graphic Toposes (1991);
Kinship and Mathematical Categories (1999).

- **∂ is real and unconditional on subobject lattices:** "In any presheaf topos …
  the lattice of all subobjects of any given object is another example of a
  co-Heyting algebra"; boundary ∂A = A ∧ non A; for directed graphs, "the
  boundary of a subgraph A consists of all its nodes which are either sources or
  targets of arrows (in the ambient graph) which are not in the subgraph A" —
  EXACTLY the intended seam reading of ∂L.
- **The Leibniz rule ∂(A×B) = (∂A×B) ∨ (A×∂B) is NOT free:** needs split-mono∘
  split-epi factorization in the schema (checkable via the nodal-category
  criterion, 1989 pp.296-297) or nonempty hom-sets + binary (co)products —
  Babylon's multi-sorted, largely-disconnected schema generically FAILS the
  corollary's hypothesis; assert Leibniz only as a property test against the
  enumerated finite schema.
- **∂L conflates two constructions:** (a) subobject boundary (above) vs (b)
  boundary/Aufhebung on the lattice of essential subtoposes — (b) is the better
  match for "audit that declared subsystems are wired" and is finite/computable
  exactly when the schema is GRAPHIC (aba = ab ⟹ subtoposes ≅ two-sided ideals;
  "In any graphic topos, Ω is a cogenerator" — 1991 Prop 2). Lawvere's own
  stated applications: "hierarchical structures such as 'hypercard', library
  catalogues" — structurally the Mirror.
- **Reflexive vs irreflexive is a foundational decision** (1986): reflexive
  graphs S^(Δ1^op) satisfy the topos-of-spaces axioms (gros; Ω has 2 points, 5
  edges — the fifth element IS the boundary class); irreflexive graphs S^⇉ are
  an étendue ("locally localic"), fail axioms 0/1, Ω has 3 elements with an
  involutory negation. Every downstream Ω/boundary formula depends on the choice.
- **Substitution is only laxly stable** (non(Af) ⊆ (non A)f; equality iff
  groupoid): the boundary does NOT commute with refactoring maps — any "∂L is
  stable under renaming" claim is generically false.
- **Kinship (1999) is the method template:** primitive generators as structural
  self-maps (m, f) → presheaves on the free monoid → graded co-Heyting Ω
  ("Precisely how Irish is y?") → coarsening as epimorphic monoid quotients with
  p_! ⊣ p* ⊣ p_* as objective ∃/∀ — port for solidarity/class relations; plus
  the "X mod A" pushout ("rationally neglecting the remote past") for collapsing
  extinct-faction history.
