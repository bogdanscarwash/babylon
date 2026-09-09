# Kind-straddle repair options — `consciousness/p5-agitation`, `solidarity/p0-transmit`

Director ruling (popup, 2026-08-18): REPAIR NOW + CEREMONY. Both formulas get restructured
kind-coherently on `feature/491-rung-ladder`; goldens re-pin via a declared ceremony; the Director
rules the new shapes first from an options popup built off this dossier. **No code changes made
producing this document** — pure paper derivation, no `cargo` runs. Starting point:
`.superpowers/sdd/2026-08-17-491-rung-ladder/task-1-report.md`'s second-pass findings (the arm
that enforces both rejections lives on THIS branch — commit `1aa8dedc`, `rust/crates/babylon-bsl/
src/typecheck.rs`; the `previous-wealth` companion content fix is commit `18ad059a`).

Numbers below are hand-derived from the `.bscn` seeds and the rule arithmetic, cross-checked
against `consciousness_ternary_conformance.rs`'s own pinned assertions where they overlap (they
matched exactly — see §1.0), and against a literal Python (IEEE-754 double, same semantics as
Rust `f64`) re-execution for the floating-point bit-identity claims in §2. Anything I could not
derive this way is marked **UNKNOWN-UNTIL-RUN**, not estimated.

---

## 1. `consciousness/p5-agitation`

### 1.0 Current landed expression, frozen-Python source, defect

Verbatim (`rust/crates/babylon-tick/content/rules/consciousness.bsl:281-289`):

```lisp
(binding wage-change :expr (- wages-in prev-wages))
(binding exploit-delta :expr (if (< wage-change 0) (- 0 wage-change) 0))
(binding wealth-change :expr (- wealth prev-wealth))
(binding increment :expr
  (+ (* (if (> exploit-delta 0) exploit-delta 0) exploit-sens)
     (+ (* (if (> (- 0 wealth-change) 0) (- 0 wealth-change) 0) rent-sens)
        (+ (* 0.0c vis-coeff)
           (* (if (> (- rf rep-base) 0) (- rf rep-base) 0) rep-sens)))))
(binding new-agitation :expr (+ agitation (+ increment wd-stub))))
```

Frozen-Python source: `src/babylon/formulas/consciousness_routing.py:124-190`
(`compute_agitation_delta`, formula block `:126-130`, `rent_component = max(0.0,
-imperial_rent_delta) * d.rent_decline_sensitivity` at `:160`); call site
`src/babylon/engine/systems/ideology.py:372-380` (`imperial_rent_delta=wealth_change, # Wealth
decline ~ rent decline`) and `:380` (`new_agitation = current_profile["agitation"] +
agitation_increment + wage_deterioration`).

**Defect, one sentence:** the formula sums a Currency-extensive wealth delta (the
`rent_component`, standing in for a dimensionless imperial-rent ratio Φ via a documented "wealth
decline ~ rent decline" approximation) against genuinely intensive/dimensionless terms
(`exploit_component`, `repression_component`, and `agitation` itself, `real intensive`,
`consciousness-ternary-conformance.bscn:240`) inside one undifferentiated `+` chain.

**Tick-1 values this repair must reproduce or knowingly diverge from** (pinned,
`consciousness_ternary_conformance.rs:519-558`; `new-agitation` = p5's own UNDECAYED write, before
p6's ×0.9 decay):

| class | wealth | prev-wealth | Δwealth | wages-in | prev-wages | Δwage | rf−rep-base | `increment` today | `new-agitation` today |
|---|---|---|---|---|---|---|---|---|---|
| class-exploited | 50 | 50 | 0 | 9 | 10 | −1 | 0 (unseeded, 0.5−0.5) | 0.15 | 0.15 |
| class-bribed | 90 | 95 | −5 | 12 | 12 | 0 | 0 | **1.0** | **1.0** |
| class-emergent | 30 | 30 | 0 | 8 | 9 | −1 | 0 | 0.15 | 0.15 |

Only `class-bribed` has a nonzero wealth delta — the sole node any wealth-term restructure can
move.

### 1.1 Candidate C1 — proportional wealth-decline rate (÷ `previous-wealth`) — VALUE-CHANGING

```lisp
(binding wealth-rate :expr (if (> prev-wealth 0) (/ wealth-change prev-wealth) 0c))
...
  (+ (* (if (> (- 0 wealth-rate) 0) (- 0 wealth-rate) 0) rent-sens) ...)
```

**Dimensional check:** `wealth-change` = `wealth`(Extensive) − `prev-wealth`(Extensive) =
Extensive (same-kind, `−`). `wealth-rate` = `wealth-change`(Extensive) ÷ `prev-wealth`(Extensive)
→ **Intensive** (the licensed T1 case, `typecheck.rs`'s `mul_div_kind`, D181). `(- 0
wealth-rate)`: Neutral−Intensive → absorbs → Intensive. `* rent-sens`(`:const`, Neutral) →
absorbs → Intensive. Now every term of `increment`'s `+` chain is Intensive (exploit-term
Intensive, this term Intensive, vis-term Neutral, repression-term Intensive) — no more mixing;
`new-agitation = agitation(Intensive) + increment(Intensive) + wd-stub(Neutral)` also clears.
Needs a `prev-wealth > 0` guard (not exercised by this fixture — no seeded class has zero baseline
wealth — but required for correctness against `E-EVAL-012`, division by zero, on any future
content that seeds one).

**Values:** class-exploited & class-emergent unchanged (Δwealth=0 ⟹ rate=0 either way).
class-bribed: rate = −5/95 = **−0.05263157894736842**; rent term = 0.010526315789473684 (was
1.0); `increment` = **0.010526315789473684** (was 1.0); `new-agitation` = **0.010526315789473684**
(was 1.0) — a ~99% reduction for this one node. Downstream `p6-route`'s `(* agitation
consumption)`, chauvinist-pressure gate, and the routed (r, l, f)/dominant-worldview for
class-bribed all change as a consequence — **UNKNOWN-UNTIL-RUN** past this point: `p6-route`
composes `min(1.0, …)` writes, a chauvinist-pressure clamp, and a popular-front suppression
multiplier (`consciousness.bsl:310-338`) too nonlinear to hand-derive reliably, and this
scenario's own tick-1 dominant-worldview flip for class-bribed (LIBERAL→FASCIST land at tick 2,
per the file's own "Ruling A" comment at `consciousness_ternary_conformance.rs:542-544`) is
already a near-boundary result I will not re-derive by hand and risk asserting wrong.

**Material meaning (Aleksandrov test):** the term now measures *the fraction of a class's wealth
lost this tick* — a scale-invariant proxy for imperial-rent decline, consistent with how Φ is
treated as a **ratio** everywhere else in the engine (MarketScissors's `price_value` opposition,
wage/value ratios) rather than an absolute currency figure.

**Gameplay/pedagogy:** teaches that *relative*, not absolute, material loss drives radicalization —
a $5 loss devastates a poor class and barely registers for a rich one; this is a materially TRUE
MLM-TW point (differential class impact of the same nominal shock) the current formula gets wrong
by construction.

### 1.2 Candidate C2 — per-capita wealth-decline rate (÷ `population`) — VALUE-CHANGING

```lisp
(binding wealth-per-capita :expr (if (> population 0) (/ wealth-change population) 0c))
```

**Dimensional check:** identical shape to C1 — `wealth-change`(Extensive) ÷ `population`
(Extensive, `social-class/population int extensive`) → Intensive, licensed; same downstream
absorption into `increment`/`new-agitation`. Needs a `population > 0` guard (also unexercised
here).

**Values:** class-exploited & class-emergent unchanged (0). class-bribed: rate = −5/800 =
**−0.00625**; rent term = 0.00625 × 0.2 = **0.00125**; `increment` = **0.00125** (was 1.0) —
smaller than C1's result (population 800 dwarfs wealth 95 for this class). Downstream (r, l, f) /
dominant-worldview: **UNKNOWN-UNTIL-RUN**, same reasoning as C1.

**Material meaning:** *currency lost per class member* — an intensive quantity in the SAME family
as `w̄ = wealth ÷ population` (T1's own headline `E-TYPE-040` example, D181), reusing that exact
idiom rather than minting a new one.

**Gameplay/pedagogy:** frames the shock as "material loss per worker," echoing a declining-wages
narrative more than a rent-decline one — defensible, but a *weaker* semantic match to
`imperial_rent_delta`'s own name than C1's percentage framing.

### 1.3 Candidate C3 — re-kind the accumulator family to Extensive — VALUE-PRESERVING (conditionally)

Reclassify `social-class/wages-inbox`, `social-class/previous-wages`, `wages/value-flow`
(`consciousness-ternary-conformance.bscn:243-244,246`) from `intensive` to `extensive` — the
**exact same Aleksandrov-test move the Director already ratified for `previous-wealth`**:
`wages-inbox` accumulates raw wage-flow AMOUNTS pushed from `wages/value-flow`
(`consciousness.bsl:232`, values 9/12/8 in this scenario), the same scale/kind as
`social-class/wages-paid`, which is ALREADY declared `int extensive`
(`consciousness-ternary-conformance.bscn:229`) — an accumulator of an extensive flow is itself
extensive, by the identical logic already landed.

**This closes 3 of 4 mismatched terms, not all 4.** After the reclassification: `wage-change` =
Extensive − Extensive = Extensive; `exploit_component` = Extensive; `rent_component` already
Extensive (untouched, no fix needed there — never the odd one out among these three, always
extensive). But `repression_component` (from `social-class/repression-faced`,
`intensity intensive`, `:245`) is **still Intensive** and still mismatches the now-all-Extensive
exploit+rent terms in the same `+` chain — UNLESS `repression-faced` is ALSO reclassified to
extensive. That leg is **weakly grounded**: `repression-faced` is not a snapshot or accumulator of
any extensive source (`consciousness_routing.py:80-96`'s own doc: "a continuous LEVEL in [0, 1]",
"declared input only; nothing in Rust writes it yet") — a bounded [0,1] level reads more naturally
as intensive (matching `probability`/`coefficient` fields elsewhere), and reclassifying it has no
"snapshot carries source kind" argument to stand on. **Flagging prominently, per the brief:** IF
the Director accepts reclassifying `repression-faced` too (on no stronger ground than "it's the
one term left to move"), **the entire candidate is value-preserving — zero arithmetic changes,
every number in §1.0's table stays bit-for-bit identical** — because nothing about the FORMULA
changes, only 4 deffield `:kind` tokens do. Also needs `agitation` itself
(`real intensive`, `:240`) to move to extensive for the OUTER `new-agitation = agitation +
increment + wd-stub` sum to close, which is the more defensible half of this candidate: `agitation`
is described as "[0,∞) raw f64 unbounded... accumulator" (`consciousness_routing.py`
module doc; `consciousness-ternary-conformance.bscn:240`) — an unbounded accumulating stock reads
far more naturally as extensive than a probability-like intensive level.

**Values:** unchanged — 0.15, 1.0, 0.15, exactly as §1.0's table, for every class, forever (no
formula touched).

**Material meaning:** none changes — this candidate doesn't reinterpret what any term MEASURES,
it only corrects a mislabeling, the same class of fix as `previous-wealth`.

**Gameplay/pedagogy:** zero player-visible effect. From the engagement+pedagogy compass this
candidate teaches nothing new and fixes nothing about how the simulation reasons — a type-hygiene
repair, not a modeling improvement, and its weakest leg (`repression-faced`) asks the
Director to accept a reclassification on strictly weaker grounds than the one already ratified.

### 1.4 Recommendation: **C1**

Argued from engagement+pedagogy, not engineering convenience: C1 is the only candidate that makes
the simulation say something *more correct* about material conditions (proportional, not
absolute, loss drives consciousness — matching how Φ behaves as a ratio everywhere else in this
engine) while touching exactly one term and needing no companion reclassification with weak
grounding. C3 is available as the zero-risk fallback if the Director wants NO behavior change
tonight, at the cost of accepting `repression-faced`'s weaker justification. C2 is a reasonable
middle option but is a worse semantic fit for a term literally named `rent_component`.

---

## 2. `solidarity/p0-transmit`

### 2.0 Current landed expression, frozen-Python source, defect

The mixing sub-expression (appears verbatim **4 times** in the rule — the negligible-floor
threshold check, both nested clamp branches, and the `CONSCIOUSNESS_TRANSMISSION` emit's
`new-target-consciousness` payload field; no per-iteration binding form exists to name it once,
`solidarity.bsl`'s own material-basis note, plan §4.3):

```lisp
(+ (field-of it social-class/revolutionary)
   (* (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength)
      (- r (field-of it social-class/revolutionary))))
```

(`rust/crates/babylon-tick/content/rules/solidarity.bsl:212-218`, repeated `:220-226`,
`:231-237`, `:266-` for the emit payload.)

Frozen-Python source: `src/babylon/formulas/solidarity.py:36` (`return solidarity_strength *
(source_consciousness - target_consciousness)`); applied `src/babylon/engine/systems/
solidarity.py:164` (`new_consciousness = target_consciousness + delta`), clamped `:165`
(`max(0.0, min(1.0, new_consciousness))`).

**Defect, one sentence:** `social-class/revolutionary` (probability, intensive) is added to a
product of the **implicit**, language-mandated-extensive `<edge-type>/strength` field (§2.9;
`solidarity-conformance.bscn:41-45` — content is FORBIDDEN from even declaring it, `E-LOAD-001`)
and an intensive consciousness-level difference, which T1's own new licensed rule (extensive ×
intensive → extensive) correctly resolves to Extensive, so the outer `+` mixes it against the
Intensive level it's being added to.

**Note on scope:** reclassifying `solidarity/strength` itself is **not an available candidate** —
it names a language-level default (§2.9), not a per-content `deffield`; changing it demands a
language amendment, out of scope for "restructure the rule."

### 2.1 Candidate S1 — convex-combination reformulation — VALUE-PRESERVING (algebraically; bit-exact for 10/12 witnesses, confirmed by direct computation)

```lisp
(binding s :expr (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength))
(binding new-r :expr
  (+ (* (- 1 s) (field-of it social-class/revolutionary))
     (* s r)))
```

(replacing all 4 occurrences of the mixing sub-expression above with `new-r`, or its inline
expansion at each site).

**Dimensional check, line by line:** `(- 1 s)`: `1`(Neutral) − `s`(Extensive) → absorbs → Extensive.
`(* (- 1 s) target-r)`: Extensive × Intensive → **licensed** (T1) → Extensive. `(* s r)`:
Extensive × Intensive → Extensive. `(+ Extensive Extensive)`: same-kind → **legal** (the `+`/`-`
rule never refuses same-kind, only mixed). Zero mixing anywhere — this typechecks under the
**already-landed** arm with no further engine change.

**Is it the same VALUE?** Algebraically yes: `target + s×(source − target) ≡ (1−s)×target +
s×source` (expand and cancel). Floating-point-wise: **verified by direct IEEE-754 double
computation against all 12 witnesses** (Python's `float`, same double semantics as Rust `f64`):

| witness | strength | current | reformulated | bit-identical |
|---|---|---|---|---|
| plain (W1) | 0.5c | 0.375 | 0.375 | **yes** |
| mass-awaken-cross (2a) | 0.5c | 0.71875 | 0.71875 | **yes** |
| mass-awaken-stays (2b) | 0.125c | 0.546875 | 0.546875 | **yes** |
| mass-awaken-exact (2c) | 1c | 0.6 | 0.6 | **yes** |
| zero-strength (3a, gated, never computed) | 0c | — | — | n/a |
| at-threshold (3b, gated, never computed) | 0.5c | — | — | n/a |
| negligible (3c) | 0.125c | 0.4453125 | 0.4453125 | **yes** |
| multi-source-a→multi-target | 0.3c | 0.33999999999999997 | 0.34 | **no** |
| multi-source-b→multi-target (survives, last-write-wins) | 0.3c | **0.31000000000000005** | **0.31** | **NO** |
| inactive-source/-target pair (both gated, never computed) | 0.5c | — | — | n/a |
| clamp (int strength=2) | 2 | 1.125 (pre-clamp) | 1.125 (pre-clamp) | **yes** |

Only the two witnesses using the decimal (non-power-of-2) literal `0.3c` diverge, and only in the
last bit (`...05` vs exact `.31`) — an IEEE-754 rounding-ORDER artifact, not a semantic drift: `0.3`
is not exactly representable in binary64, so `a+b*(c-a)` and `(1-b)*a+b*c` round differently at the
last step even though they're the same real number. Every dyadic-literal witness (powers of
1/2 — 0.5, 0.25, 0.125, 0.875, 0.5625, 0.6-exact-trick, 1.0, and the bare-int `2`) is **provably**
bit-identical. `multi-target`'s FINAL stored value is `multi-source-b`'s write (declaration-order,
last-write-wins, `solidarity.bsl`'s own D-record 2) — so exactly ONE node's ONE field, in the
whole 22-node world, differs by one ULP under this candidate.

**Material meaning:** unchanged — same relation, `target moves toward source, scaled by edge
strength`, just expressed as a weighted average of the two consciousness levels instead of a
level-plus-scaled-delta. Aleksandrov test: still measures proletarian-internationalist
consciousness transmission along a SOLIDARITY edge; nothing new is claimed.

**Gameplay/pedagogy:** no player-visible behavior change (mod one ULP on one witness this
conformance world doesn't narratively distinguish) — the safest possible repair, since it doesn't
touch tuned game balance at all.

### 2.2 Candidate S2 — declared-intensive coefficient duplicate — VALUE-PRESERVING arithmetic, but needs a companion engine change + new content

```lisp
; new deffield (edge-attr, mirroring the wages/value-flow precedent):
(deffield solidarity/coefficient coefficient intensive)
; seeded per edge, identical to the existing implicit strength value:
(edge-attr EdgeType/SOLIDARITY plain-source plain-target solidarity/coefficient 0.5c)
; … 11 more, one per edge …

; rule reads the new field for the ARITHMETIC (keeps reading `strength` for the >0 gate,
; a comparison, which is kind-blind):
(* (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/coefficient)
   (- r (field-of it social-class/revolutionary)))
```

**Dimensional check:** `coefficient`(Intensive, content-declared) × `(- r target-r)`(Intensive) =
**Intensive × Intensive**, which the arm AS LANDED TODAY conservatively **refuses**
(`typecheck.rs`'s `mul_div_kind` `_ => Err(...)` catch-all — I deliberately left this combination
unlicensed in T1, having no real-content case forcing a decision either way). **This candidate
does not typecheck today.** Landing it requires a companion, small extension — license
Intensive × Intensive → Intensive (the same "scale by a dimensionless factor" logic already
licensed for Extensive × Intensive, applied to the other operand pairing: coefficient ×
probability-delta = probability-delta) — a decision this dossier surfaces but does not make;
resolving it sits outside a single rule's restructure and needs its own sign-off.

**Values:** IF the companion extension lands and `coefficient` is seeded identically to `strength`
on every edge, the arithmetic is byte-for-byte the CURRENT formula (same operands, same operation
order) — genuinely value-preserving, no rounding-order question at all (unlike S1).

**Material meaning:** none changes; this candidate's whole point is giving `strength`'s VALUE an
intensively-kinded name for the arithmetic while the implicit field keeps its language-mandated
extensive kind for whatever else reads it (nothing else in this pack currently does).

**Gameplay/pedagogy:** identical to S1 — zero player-visible change — but costs a companion arm
extension AND new seeded content (12 edge-attr rows + 1 deffield), which S1 doesn't need.

### 2.3 Recommendation: **S1**

From engagement+pedagogy: both candidates are behavior-neutral, so the deciding factor is risk and
scope, not gameplay. S1 needs zero further engine change (lands cleanly under the arm the
Director already ratified) and is bit-identical for 10 of 12 witnesses (the 11th and 12th differ
by one ULP, in a node this conformance world's own narrative doesn't hinge on). S2 reopens the
typecheck arm itself for a SECOND time in one night, on a case this dossier cannot resolve
unilaterally (whether Intensive × Intensive should ever be licensed is a real design question,
not a rubber stamp) — recommend deferring S2 unless the Director specifically wants the
`solidarity/coefficient` field to exist as a reusable, explicitly-intensive edge attribute for
OTHER future rules, which S1 gives no path to.

---

## 3. Ceremony surface

Both `report.before` (`hex(&report.before)`, the pre-tick SUBSTRATE load) and `report.after`
(post-tick state) in `tick_goldens.rs` are **graph-state** hashes — node/edge ATTRIBUTE VALUES,
not `deffield :kind` metadata or rule text (corroborated by T1's own observed behavior: the
`previous-wealth` kind flip, commit `18ad059a`, moved no pin anywhere in the 35 unaffected test
binaries). On that basis:

| Rule | Candidate | `report.before` | `report.after` | Why |
|---|---|---|---|---|
| consciousness | C1 (rate) | **unmoved** | **moves** | no new `.bscn` attrs; `new-agitation`/downstream values change for class-bribed |
| consciousness | C2 (per-capita) | **unmoved** | **moves** | same reasoning, different scale |
| consciousness | C3 (re-kind, value-preserving) | **unmoved** | **unmoved (high confidence)** | pure `:kind` metadata edit, zero attribute-value change — **needs no pin ceremony** if this holds, per the brief's own framing; confirm on the actual gate run before treating as certain |
| solidarity | S1 (convex combo) | **unmoved** | **moves** | `multi-target`'s stored value shifts by 1 ULP (0.31 vs 0.31000000000000005); everything else bit-identical, but ONE differing byte anywhere moves the whole hash |
| solidarity | S2 (coefficient dup.) | **moves** | **moves** | 12 new edge-attr rows + 1 new deffield widen the loaded graph's attribute set — mirrors this SAME file's own recorded precedent language for the Train B item 3 re-pin: "attribute-set change only, zero value drift" |

**Pins affected, by name** (`rust/crates/babylon-tick/tests/tick_goldens.rs`):
`consciousness_ternary_foundation_hashes_are_pinned` (:366, both hashes) for any consciousness
candidate except C3-if-verified-unmoved; `solidarity_conformance_hashes_are_pinned` (:458, at
least `report.after`, `report.before` only under S2) for either solidarity candidate except
none (S1 always moves `report.after` because of the multi-inbound ULP). Downstream value-level
assertions also move: `consciousness_ternary_conformance.rs`'s tick-1 table (:519-558, `agitation`
column and — **UNKNOWN-UNTIL-RUN** — the routed r/l/f/dominant columns for class-bribed under C1/
C2) and its tick-2 table (:704-721, likely UNCHANGED under C1/C2 since tick-2's own increments are
independently zero, but class-bribed's tick-2 STARTING state depends on tick-1's now-different
agitation — UNKNOWN-UNTIL-RUN whether that cascades into a different tick-2 dominant/ternary
read); `solidarity_conformance.rs`'s per-witness assertions (only the multi-inbound witness's
exact literal under S1).

The repair commits ride **this branch** (`feature/491-rung-ladder`) — the arm that turns these
formulas into rejections in the first place (`1aa8dedc`) lives here, and neither rule has loaded
successfully on this branch since. Whatever candidates the Director selects should land as their
own commits here, followed by the ceremony (`tools/generate_ceremony_message.py`, `Baselines:
blessed(<slug>)` trailer, §6.5) for whichever pins actually move once the real gate runs.
