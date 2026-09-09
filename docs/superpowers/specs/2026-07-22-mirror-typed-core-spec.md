# The Mirror & the Typed Core — preparatory spec (POST-v1.0, TABLED)

*Status: **preparation, not ratification.** BD-directed 2026-07-22: "table this,
but make a spec to prepare it." Everything here is post-v1.0 work, deliberately
off the Gate-3 critical path, consistent with the 2026-07-21 ruling that the
formalism surface is **closed for v1.0** — this spec prepares the constitutional
amendment that would reopen it, and does nothing else. The two §4 sentinel
motions are the only items that may be scheduled pre-1.0, at owner discretion,
because they add gates, not formalism.*

*Companions: the BD's Haskell algebra
[`2026-07-22-babylon-core-typed-haskell-draft0.md`](2026-07-22-babylon-core-typed-haskell-draft0.md)
(committed verbatim; its module `BabylonCoreDraft.hs` lands beside it when the
BD drops the copy) and the Lawvere-archive evidence base
[`../research/2026-07-22-lawvere-archive-readings.md`](../research/2026-07-22-lawvere-archive-readings.md)
(25 papers read in full from `/media/user/data/babylon-data/lawvere-master`;
every claim below marked **[L]** is page-cited there).*

---

## 1. What this prepares, in one paragraph

Two programs share one foundation. **The Typed Core** ports the functional core
(kernel, models, formulas, topology, dialectics — the tick as the sole arrow)
to a Haskell algebra where a class of constitutional violations becomes
unrepresentable rather than reviewable. **The Mirror** (NORTH_STAR §7, Round 2)
turns Stratum 2's implicit self-model into explicit data: the construct graph
as a first-class, queryable object, audited with the same boundary mathematics
the game applies to its world. The shared foundation is Lawvere's: the archive
reading confirms most of Babylon's constructs are *real* instances of his
mathematics — and identifies, precisely, the five places where our current
phrasing borrows rigor it has not earned. This spec's job is to make the
eventual ratification honest: every construct below is tagged **EARNED**
(matches Lawvere as stated), **CONDITIONAL** (matches if a named check passes),
or **UNEARNED AS PHRASED** (rename or strengthen at ratification).

## 2. Ground truth: what the codebase already proves (2026-07-22 audit)

The kernel/system/engine audit (verified against dev) found the ontology more
formal than feared:

- **Kernel** = the initial segment of the import poset — machine-enforced
  (import-linter "kernel is layer 0"). It is the *signature* Σ in which the
  layers above are typed.
- **System** = a registered, position-ordered, partition-classified,
  deterministic endomorphism on the world graph. Membership (`_SYSTEM_CLASSES`,
  30 entries), total order (duplicate positions = import-time `RuntimeError`),
  and partition coverage (MATERIAL_BASE / ACTION / CONSEQUENCE, ADR081) are all
  **proven at import time** in `simulation_engine.py`.
- **Engine** = the terminal object: evaluator of the one canonical word
  `T = S₃₀ ∘ … ∘ S₁` plus the hash seal; five import-linter contracts make
  engine-ness terminal.

Four audited gaps, which §4 turns into motions: (1) system **footprints**
(reads/writes) are prose, not data — the materialist-causality rationale is
unauditable; (2) **registration is unguarded** — an unregistered `SystemBase`
subclass is silent dead superstructure; (3) determinism is proven only in
aggregate (tick hash), never attributed per-system; (4) doc rot:
`system_protocol.py` still says "Mutable NetworkX graph" (Amendment L removed
NetworkX).

## 3. The constructs, adjudicated against the archive

Draft0's architecture is endorsed: two planes, kernel newtypes, GADT verb
grammar, phase-indexed registry, witness capabilities, `observe` as the sole
read surface. The table below is what ratification must additionally fix.
(Draft0 sections in parentheses; **[L]** = evidence base.)

| Construct (draft0) | Verdict | Ratification requirement |
|---|---|---|
| Adjoint cylinder L ⊣ U ⊣ R (§3) | **CONDITIONAL** | Legitimate UIAO **iff** L, R are full+faithful and U is a genuine two-sided retraction (U∘L = id = U∘R) — otherwise it is a mere adjoint triple and the "unity of opposites" label is unearned. Ship the retraction and both adjunction laws as QuickCheck properties. **[L1, L2]** |
| Tension w as "adjunction defect" (§3) | **UNEARNED AS PHRASED** | Two fixes: (a) the current defect measures only the **unit** side — a half Galois connection; track the counit defect or scope the claim as unit-only. (b) w has no compositional law today. Either build Tension as the free [0,∞]-category on the weighted verb-graph (min-plus shortest-path closure — then the triangle inequality *is* the composition law, and derived relations compose by Bellman–Fenchel convolution), or declare in the spec that tension is non-compositional. Law to ship either way: `w(𝔇) = 0 ⟺ the canonical map L(X) → R(X) is iso`. **[L4]** |
| σ sublation (§3) | **UNEARNED AS PHRASED** → derivable | Replace the ad hoc map with a derived construction carrying laws for free: the image-factorization of the canonical 2-map between the opposite inclusions of the UIAO (Lawvere's "third section", van der Waals model), or Isbell double-conjugation F ↦ F\*# (idempotent, extensive, monotone). **[L1]** |
| Level lattices + aufhebung (§3) | **UNEARNED AS PHRASED** — the weakest link | "Least strictly-higher level where a resolution predicate holds" drops the mathematics. Lawvere: a *level* is a downward functor with both adjoints full inclusions; T is the Aufhebung of S iff **R_T L_S = L_S** (the higher coskeleton fixes the lower skeleton — the factoring condition); "smallest such T exists" is a **theorem** (the lattice of levels ≅ idempotent two-sided ideals; left ideals, for a graphic monoid). Implement Aufhebung as this closure operator over the finite ideal lattice, with laws (extensive, idempotent, monotone) and **the Hegelian Taco as the golden fixture**: the 8-element graphic monoid with known dimension 3 and known ideal chain ∅ ⊂ {0,1} ⊂ [ℓ,q,r] ⊂ [L,R] ⊂ [1]. A rewrite in any language must reproduce the taco — that is the behavioral contract. **[L1]** |
| Extensive/Intensive (§2) | **EARNED**, one rename, one fork | The Num-less `Intensive` is exactly right. But: (a) the conservation property is **not a "sheaf law"** (sheaves glue over *overlapping* covers — a limit condition); it is the **extensive-quantity coproduct law** over a partition (Lawvere 1992's defining clause). Rename it; reserve "sheaf" for a genuine overlap-gluing law if border-sharing regions ever demand one. (b) `allocate ⊣ aggregate`: as drafted (proportional section with aggregate∘allocate = id) it is a **section of a split epi, not an adjunction**. Either state and test the extremal universal property, or frame the level lattice as a genuine site with aggregate as direct image (then p_! ⊣ p\* is real), or rename honestly. **[L2]** |
| Cohesion over the solidarity graph | **CONDITIONAL** — one functor short | Babylon has the 3-string; Lawvere's category of cohesion needs the 4-string **p_! ⊣ p\* ⊣ p_\* ⊣ p^!** — the missing piece is `pieces` (connected components), which must preserve finite products (axiom a). With finite hom-sets, continuity (b) then comes free, and the Nullstellensatz (c) — "every nonempty piece contains a point" — is trivially checkable. Bonus: Lawvere already proved the reflexive-graph topos sufficiently cohesive and infinitesimally generated (2007 Props 5-6) — adopt his result rather than re-derive. Ground "atomization" in the canonical map X → codiscrete(points X) classified by the 5-element Ω (in A / entering / leaving / excursion / outside) — the scalar quantification is our own addition and must cite the enrichment fork above for its justification. **[L2]** |
| ∂L boundary / seam space | **CONDITIONAL** — currently conflates two operators | (a) The **subobject boundary** ∂A = A ∧ non A is unconditionally lawful in any presheaf topos, and for directed graphs is *literally* the seam: nodes touched by arrows crossing out of A. (b) The **subtopos boundary/Aufhebung** on the lattice of essential subtoposes is the better match for "audit that declared subsystems are wired" (Lawvere's own use case: hypercard, library catalogues — structurally the Mirror) but is finite/computable **iff the schema is graphic** (endomorphism identity `aba = ab` — then subtoposes ≅ ideals and Ω is a cogenerator). Ratification must: pick (a) or (b) per use; **check the graphic identity** on the actual schema (falsifiable, currently unchecked); decide **reflexive vs irreflexive** edges (recommend reflexive/Δ₁ — the gros topos with the 5-element Ω; irreflexive is an étendue with different Ω and negation, and every boundary formula changes); assert the Leibniz rule ∂(A×B) = (∂A×B) ∨ (A×∂B) **only as a property test** against the enumerated finite schema (its hypotheses — nodal factorization or nonempty homs + products — generically fail for our multi-sorted schema); and never claim ∂ commutes with refactoring maps (substitution stability is lax: non(Af) ⊆ (non A)f, equality iff groupoid). **[L5]** |
| The tick (§8) | **EARNED**, three sharpenings | It *is* Lawvere's discrete dynamical system (the 1978 example verbatim). Sharpen: (a) distinguish the cyclic monoid ⟨T⟩ ≅ ℕ from the generated systems monoid on {S₁..S₃₀} — his structural theorems bind differently to each (the no-cycles cancellation theorem does NOT apply to ⟨T⟩); (b) name determinism as the **discrete-op-fibration unique-lifting law** (fibers discrete; `Left Violation` = failed unique lift) and property-test it; (c) "statics are fixed points of T" must say whether it means the trivial fixed point (T = id, empty report) or Lawvere's **freeze** (state fixed, cost/report nonzero) — his own marginal note flags exactly this ambiguity. **[L3]** |
| observe :: World → Chronicle (§10) | **CONDITIONAL** | As a snapshot it is Lawvere's weak "output" case. Upgrade to a **response functor**: per-system fragments with a monoid instance and the law `observe(tick) = mconcat [observeStep i wᵢ wᵢ₊₁]` — the Volterra ∫θ ds accumulation. This is also what makes per-system determinism attribution (gap 3 of §2) checkable. **[L3]** |
| "Chaos" | **DO NOT USE** | Lawvere's chaos = symbol-sequence surjectivity of X → Y^T — unrelated to I.12's fold catastrophes ("normal, reasonable behavior" qualifies, his words). For thresholds/discontinuities, the usable toolkit is the entropy paper's Lawvere-metric state category with upper-semicontinuous regularization. **[L3]** |
| Verb grammar GADT (§4) | **EARNED**, one naming fix | It is the **generating 𝒱-graph of the base category of types** — not a hyperdoctrine (which needs per-type attribute CCCs, substitution, Σf ⊣ f· ⊣ Πf with Frobenius + Beck). Name it honestly; heterogeneous verbs (class ⊢ territory) are naturally **bimodules/profunctors**, which buys derived indirect relations by convolution. **[L4]** |
| Capability witnesses (§9) | **EARNED** | `requireMembership` as sole mint of `MembershipW` is precisely the **Comprehension Schema** ({B:ψ} with structural map). Recorded caveat: if node kinds ever gain internal morphisms, Frobenius fails and equality must become the hom profunctor. **[L4]** |
| Mode category (§6), two planes (§1), formulas (§7), boundaries as physics (§10) | **EARNED** | As drafted. The V₀ ⊂ [0,∞] adjoint triple gives the two-planes discipline its formal home (`reachable :: Tension → Bool` as the collapse pair) if the enrichment fork is taken. **[L4]** |

New constructions the archive offers that draft0 does not yet use (candidates,
not commitments): the kinship method (generators → free-monoid topos → graded
Ω "how Irish is y?" → coarsening as epimorphic quotients with p_! ⊣ p\* ⊣ p_\*
as objective ∃/∀) as the template for class/solidarity queries; the `X mod A`
pushout for collapsing extinct-faction history; Cauchy completeness as
"solidarity-network saturation"; the trace Tr(f_\*) = inf_a A(a, fa) fixed-point
criterion tied to the Survival Calculus equilibrium question; the universal
state F_C(Y)(A) = Cat(A/C, Y) as the "is World expressive enough" ceiling test.

## 4. Near-term motions (the only pre-1.0 items; owner-schedulable)

Both are Stratum-2 gates inside existing machinery — no new formalism, so the
closed-surface ruling is respected:

1. **Footprint ClassVars** on `SystemBase` — `reads`/`writes` frozensets of
   graph-attribute names, audited by the `check:vocabulary` family (which
   already audits stamped attributes). Prize: position order becomes a proven
   **linear extension of the writer-before-reader DAG** — the 60-line ordering
   comment becomes a theorem the build re-proves. This is also the data
   substrate the Mirror's construct graph will read.
2. **Registration sentinel** — every `SystemBase` subclass under `src/` is in
   `_SYSTEM_CLASSES` or a cited exemption. Closes the silent-dead-institution
   hole at the system level.

(Plus the one-line `system_protocol.py` NetworkX docstring fix, folded into
whichever commit lands first.)

## 5. Toolchain: GHC in Nix (post-1.0, specified here, not implemented)

Draft0's toolchain note predates ADR102 and is **superseded on location**: the
infra submodule is unmounted and babylon's own vendored `flake.nix` is the
canonical toolchain — the Haskell lane lands **in babylon's flake**, not
babylon-infra's. Shape at adoption time:

- A third devshell `haskell` (beside `default` and `dataBuild`): pinned GHC
  9.4+ line, `cabal-install`, HLS — all from the same rev-pinned nixpkgs so
  the GHC pin doubles as float-determinism infrastructure (the III.12(b)
  cross-implementation story needs a byte-stable compiler at least as much as
  Python needs the sqlite pin).
- `packages.babylon-core` builds the core library + a `babylon-core-check`
  executable running the property/law suite; `checks.core-laws` wires it into
  `nix flake check`.
- The Python⟷Haskell seam ships as data (draft0 §Mermaid): hydration hands
  `Material (NodeData k)` in, `TickReport` + deltas come out; shadow-mode
  validation runs both engines against the dense goldens under III.12(b)
  tolerances until cutover.
- If the core binary ever deploys to the production host: ship the ELF, not a
  closure (babylon-infra Article X.1 stays untouched).
- Machine safety: GHC builds are heavy; they join cargo in the single-flight
  discipline, never fanned out.

## 6. Sequencing (unchanged from draft0 §13, made explicit)

1. **Program 13 first** (float honesty: hash byte-recipe, sigmoid polynomial +
   error bound, tolerance policy) — draft0 depends on its artifacts at three
   named points.
2. **Ratification spec** resolving draft0 §12's seven rulings **plus** this
   spec's adjudications (§3 table) and new open rulings (§7). That spec is the
   constitutional amendment reopening the formalism surface.
3. **Typed core in shadow mode** against dense goldens; cutover only on
   byte-agreement under the declared tolerances; fixed-point Currency triggers
   the III.12(b) re-baseline ceremony.
4. **The Mirror** opens only after the current trains merge (NORTH_STAR §7):
   its opening brief is §3's ∂L adjudication — construct graph as data,
   graphic-topos target, footprints (§4.1) as its first real dataset.

## 7. Open rulings for the ratification spec (beyond draft0 §12)

1. **Reflexive or irreflexive** world/construct graphs (changes Ω, negation,
   and every boundary formula; recommendation: reflexive/Δ₁).
2. **Graphic identity check**: does the schema's endomorphism monoid satisfy
   `aba = ab`? (Determines whether the finite ideal-lattice ∂L machinery is
   available or the general co-Heyting fallback applies.)
3. **Tension enrichment fork**: free-[0,∞]-category (compositional, triangle
   inequality by construction) vs declared non-compositional scalar.
4. **allocate/aggregate**: universal-property adjunction, site-theoretic
   framing, or honest rename.
5. **pieces functor**: add p_! and claim full cohesion, or stay at the
   adjoint-cylinder level and say so.
6. **observe**: upgrade to response functor now or record the snapshot
   limitation.
7. **Which golden fixtures are constitutional**: the Hegelian Taco (Aufhebung),
   the nx differential oracle port (III.7), the conservation/coproduct law,
   the response-functor mconcat law — each is a rewrite-test artifact in the
   Tests-as-Behavioral-Contracts sense.

## 8. What this buys (unchanged) and what it must not cost

Draft0 §13 stands: determinism as a type; grammar and mode transitions as
compile-time law; tension only by measurement; the intelligence boundary as
linkage fact. The archive reading *adds*: our dialectical vocabulary stops
being decoration — where it survives adjudication it is Lawvere's mathematics
with page numbers, and where it does not, we rename before we ratify. What it
must not cost: the game. Nothing in this document schedules work before
v1.0.0 ships except §4, and §4 only if the owner says so.
