# Metabolism BSL Port — Assessment (Phase 1 gate)

**Date.** 2026-08-11. **Author.** Implementation engineer, Metabolism port train
(worktree `wt-metabolism-port`, branch `feat/metabolism-bsl-port`, cut from dev tip
`a0caab02`, PR #500 Currency×Ratio `:floor`/`:cap` included).

**Purpose.** The Phase 1 assessment gate required before any `.bsl` is written
(`docs/superpowers/plans/` port-train convention, mirrored from the Dispossession port,
PR #498). Every constant, clamp, branch, default and event in `MetabolismSystem`
(`src/babylon/engine/systems/metabolism.py`, position 13.0, `MATERIAL_BASE`,
`creates_value=False`) is inventoried below, with a PORT NOW / D-record / BLOCKED
disposition per block, grounded in direct reads of the frozen Python, the three
formulas it calls, `MetabolismDefines`, the Dispossession/Lifecycle/fundamental-theorem
BSL packs, the BSL spec (`docs/reference/bsl-language.rst`), and the actual Rust engine
source (`rust/crates/babylon-bsl/src/{tick,domain,scenario,evaluator,metrics}.rs`).

**Headline finding the task brief did not anticipate** (reported per this train's
standing instruction to surface contradictions loudly): **`entropy_factor` cannot
legally be declared as a `Ratio` (`r`-suffixed) literal for this formula**, despite
`bsl-language.rst`'s own Draft-Ruling Register row D99 naming `entropy_factor` as the
Currency×Ratio addendum's own worked example. The reason, verified at the code level,
is in §3 below. A workaround is used to still deliver a working port; the gap is
recorded as a genuine open finding, not silently routed around.

---

## 1. The frozen system, verbatim math

`MetabolismSystem.step()` (`metabolism.py:56-153`) has four blocks, in source order:

1. **Spec-070 sovereign channel** (`:78-88`): reads
   `context.persistent_data["balkanization.metabolic_impact_by_territory"]`
   (a `dict[territory_id, float]` written by `SovereigntySystem`, one tick earlier —
   Sovereignty runs @17.5, AFTER Metabolism @13.0, so this is genuinely a
   **previous-tick** handoff, read at the START of Metabolism's own tick) and adds it to
   `territory.habitability`, clamped (`_write_clamped`, bounds not shown in the excerpt
   but implied `[0,1]` by the field's own declared type).
2. **Phase 1 — per-territory biocapacity update** (`:90-118`): for every `TERRITORY`
   node, computes `calculate_biocapacity_delta` and `calculate_hysteresis_damage`, then
   writes `biocapacity`/`max_biocapacity`.
3. **Phase 2 — global aggregates** (`:120-132`): sums `biocapacity` over every
   `TERRITORY` node; sums `(s_bio + s_class) * population` over every **active**
   `SOCIAL_CLASS` node (Mass Line: inactive/dead classes excluded).
4. **Phase 3 — overshoot check** (`:134-153`): `calculate_overshoot_ratio`, then
   `if ratio > overshoot_threshold: emit ECOLOGICAL_OVERSHOOT`.

### The three formulas (`src/babylon/formulas/metabolic_rift.py`)

```text
calculate_biocapacity_delta(regeneration_rate, max_biocapacity, extraction_intensity,
                             current_biocapacity, entropy_factor=1.2) -> float:
    regeneration = regeneration_rate * max_biocapacity
    if current_biocapacity >= max_biocapacity: regeneration = 0.0
    raw_extraction = extraction_intensity * current_biocapacity
    ecological_cost = raw_extraction * entropy_factor
    return regeneration - ecological_cost

calculate_hysteresis_damage(extraction_intensity, current_biocapacity,
                             hysteresis_rate) -> float:
    raw_extraction = extraction_intensity * current_biocapacity   # SAME product
    return raw_extraction * hysteresis_rate

calculate_overshoot_ratio(total_consumption, total_biocapacity, max_ratio=999.0) -> float:
    if total_biocapacity <= 0: return max_ratio
    return total_consumption / total_biocapacity
```

`raw_extraction = extraction_intensity * current_biocapacity` is computed **identically**
in both `calculate_biocapacity_delta` (line 47) and `calculate_hysteresis_damage`
(line 86), called from `metabolism.py` with the *same* `extraction_intensity` and
`current_biocapacity` values in both cases (`:98`/`:109`, `:95`/`:99`/`:103`). Sharing one
binding for it in the BSL port is a value-preserving consolidation, not a transcription
deviation — noted inline in the rule, not filed as a D-record.

`metabolism.py`'s own step, lines 90-118:

```python
delta = calculate_biocapacity_delta(
    regeneration_rate=attrs.get("regeneration_rate", 0.02),
    max_biocapacity=attrs.get("max_biocapacity", 100.0),
    extraction_intensity=attrs.get("extraction_intensity", 0.0),
    current_biocapacity=attrs.get("biocapacity", 100.0),
    entropy_factor=entropy_factor,          # GameDefines
)
current = attrs.get("biocapacity", 100.0)
max_cap = attrs.get("max_biocapacity", 100.0)
damage = calculate_hysteresis_damage(
    extraction_intensity=attrs.get("extraction_intensity", 0.0),
    current_biocapacity=current,
    hysteresis_rate=hysteresis_rate,        # GameDefines
)
new_max = max(0.0, max_cap - damage)
new_biocapacity = max(0.0, min(new_max, current + delta))
graph.update_node(node.id, biocapacity=new_biocapacity, max_biocapacity=new_max)
```

### `MetabolismDefines` (`src/babylon/config/defines/territory.py:209-245`)

| field | default | bounds | used in |
|---|---|---|---|
| `entropy_factor` | 1.2 | `gt=1.0, le=3.0` | Phase 1 (ecological cost) |
| `overshoot_threshold` | 1.0 | `gt=0.0, le=2.0` | Phase 3 (BLOCKED) |
| `max_overshoot_ratio` | 999.0 | `gt=0.0` (unbounded above) | Phase 3 (BLOCKED) |
| `hysteresis_rate` | 0.005 | `ge=0.001, le=0.01` | Phase 1 (hysteresis damage) |

---

## 2. Per-block disposition

| Block | Disposition | Notes |
|---|---|---|
| Spec-070 sovereign pre-pass (`:78-88`) | **BLOCKED** | §4(a) below |
| Phase 1 — per-territory biocapacity/hysteresis (`:90-118`) | **PORT NOW**, with one workaround-plus-D-record for `entropy_factor` | §3, §5 |
| Phase 2 — global aggregates (`:120-132`) | **BLOCKED** | §4(b) below |
| Phase 3 — overshoot check + `ECOLOGICAL_OVERSHOOT` emit (`:134-153`) | **BLOCKED** | §4(b) below (same root cause as Phase 2 — the two are one rule in the frozen source and cannot be split without the aggregates Phase 2 produces) |

This matches `reports/bsl-gap-analysis-2026-08-10.md` row 13.0 exactly: *"Per-territory
biocapacity/hysteresis rule plus one graph-scoped overshoot rule; blocked on
graph-scoped evaluation only."* The per-territory block is genuinely portable; the
global check is not — confirmed below at the source level, not merely cited.

---

## 3. New finding: `entropy_factor` has no legal BSL representation in the actual formula

The task brief's working assumption was: *"`entropy_factor` becomes a defconst with
`1.2r :floor 1r :cap 3r`"* — citing `bsl-language.rst`'s Draft-Ruling Register row **D99**
(the Currency×Ratio scale operation, PR #500/ADR194), whose own worked example is,
verbatim: `(defconst metabolism/entropy-factor 1.5r :floor 1r :cap 3r)`, proved through
`rust/crates/babylon-tick/tests/currency_scale_op_e2e.rs`.

**This does not work for `MetabolismSystem`'s actual arithmetic**, and the reason is
precise, not a matter of style:

1. `entropy_factor`'s declared domain is `(1.0, 3.0]` — above what any `p`/`i`/`c`
   scaled literal can hold (`[0.0, 1.0]` closed, `bsl-language.rst` §1.5,
   `E-LEX-024`). The **only** literal suffix that can hold `1.2` at all is `r`
   (`Ratio`) — confirmed at the lexer: a bare, unsuffixed decimal literal (`1.2` with
   no letter) is unconditionally `E-LEX-021` (`reader.rs:159`, `BareFloat`); there is
   no "plain Real literal" escape hatch the way there is a "plain Int literal" one
   (dispossession's own `:const`s use exactly that Int escape hatch, D-2/D-4 below).
2. **`Ratio` has exactly one legal operator: `Currency × Ratio → Currency`**
   (`bsl-language.rst` D99: *"`Ratio` gets exactly this one operator and no other (no
   `+`, no `Ratio × Ratio`, no ordering)"*), confirmed in the evaluator:
   `apply_arith`'s only `Value::Ratio`-matching arm is the `(Currency, Ratio) | (Ratio,
   Currency) if op == "*"` pair (`rust/crates/babylon-bsl/src/evaluator.rs:560-565`).
   Every other combination involving a `Value::Ratio` falls through to the generic
   `_ => match (real_lane(lhs), real_lane(rhs))` arm (`:590-595`), and `real_lane`
   (`:535-544`) only recognizes `Value::Real`/`Value::Int` — a `Value::Ratio` operand
   there is a hard `EvalError` ("no arithmetic is defined on ...").
3. The formula's actual multiplicand, `raw_extraction = extraction_intensity *
   current_biocapacity`, is **never `Currency`**. Both `extraction_intensity` and
   `biocapacity` must be `:field` reads (per §5 below — they genuinely vary
   per-territory), and slice-1's scenario/attribute layer refuses to store **any**
   field as anything but `int`-declared: `attribute_value` (`scenario.rs:653-667`)
   unconditionally rejects a non-`Int`-typed `deffield`, with the comment *"slice 1
   stores only `int`-declared fields — the scaled and Currency lanes need typed
   attribute storage (a declared Phase-2 trait revision)"*. So `raw_extraction` is
   `Value::Real` (via `Int → Real` promotion), by construction, always — there is no
   way to make it `Value::Currency`, because `Currency` values can only enter a BSL
   rule as an **inline literal in the rule body** (`currency_scale_op_e2e.rs`'s own
   module doc: *"A `Currency` value can currently enter a rule's evaluation ONLY as a
   literal written directly in the rule body"*), never as the result of a field read.

So `ecological_cost = raw_extraction * entropy_factor` needs `Real × Ratio`, which does
not exist. `entropy_factor` declared as `1.2r :floor 1r :cap 3r` would **load** cleanly
(the scenario-level check has no idea what the rule body will do with it) and then
**fail at evaluation**, the first time any subject's guard passes far enough to reach
the multiply.

**This is not a new gap invented by this port — it is the exact residual gap
`reports/bsl-gap-analysis-2026-08-10.md`'s own Appendix item 3 already named, one day
before D99 landed**: *"The residual gap is real only for a runtime-valued `:const`
outside `[0,1]` (`entropy_factor`, domain `(1.0, 3.0]`), which authors cannot split at
load time."* D99/ADR194's ruling closes director-gate #492 for the shape its own worked
example tests — `Currency × Ratio`, e.g. `1000$ × factor` — but `entropy_factor`'s real
consumer needs `Real × Ratio`, a shape D99 never added an operator for. The gap-analysis
report's framing ("Currency × unbounded coefficient") appears to have assumed
`biocapacity` would carry a `Currency`-typed BSL field, matching its `Currency` type
annotation in the Python Pydantic model (`territory.py:155-163`) — but slice-1 BSL has
no `Currency`-typed field storage at all (point 3 above), so that assumption does not
hold for the actual port.

**Workaround used, D-recorded in the rule pack (see `metabolism.bsl`'s own D-1):** a
scaled bare-`Int` `:const`, the same escape hatch Dispossession's own D-2/D-4 already
use and document (`dispossession.bsl:118-139`: *"`defconst` also accepts a BARE
`Atom::Int`... a bare Int carries NO domain check at all"*) — `entropy_factor` is
declared `(defconst metabolism/entropy-factor-x1e6 1200000)` (scaled ×1,000,000) and
divided back out inline: `(/ (* raw-extraction entropy-factor-x1e6) 1000000)`.

**Correction, round 1 (F1 fix round, adversarial review of PR #501): this workaround is
NOT bit-exact against the frozen engine in general.** An earlier revision of this
document and of `metabolism.bsl`'s own D-1 claimed the workaround "preserves the
formula's exact value ... for ANY legal `(1.0, 3.0]` modded value" and "carries the
coefficient's full fractional precision as an exact integer" — both FALSE, disproved by
execution.

**Correction, round 2 (independent re-verification of round 1's own restatement): round
1's fix bounded the WRONG QUANTITY, and introduced two further false claims in the
process of fixing the first one.** Corrected findings, all verified by direct execution
against both engines:

1. **The frozen engine computes `raw_extraction * entropy_factor` as ONE binary64
   multiply; this pack computes `(raw_extraction * entropy_factor_x1e6) / 1000000`** —
   the same real-valued function, different floating-point PROGRAMS.
2. **The `<= 2 ULP` derivable / `<= 1 ULP` measured bound applies to the
   `ecological-cost` TERM, not to this rule's OUTPUT** — round 1 derived its bound over
   `ecological-cost` correctly, then measured `biocapacity` (the output) and reported it
   under the same bound, which does not hold: `ecological-cost` is subtracted from a
   comparable-magnitude quantity (`current + delta`) under CANCELLATION and then
   clamped, and cancellation is exactly the operation that turns a small relative input
   error into an arbitrarily large one. Measured counterexamples, both engines:
   - A: `biocapacity=149, max_biocapacity=150, extraction_intensity=1,
     regeneration_rate=0.005, entropy_factor=1.005` — frozen `0.005000000000023874`
     (`0x3f747ae147ae8000`) vs this pack `0.0049999999999954525`
     (`0x3f747ae147ae0000`) — **32768 ULP (2^15) apart**, relative difference `5.7e-12`.
   - B: `biocapacity=1, max_biocapacity=10, extraction_intensity=3,
     regeneration_rate=0.26, entropy_factor=1.2` (the SHIPPED DEFAULT) — frozen
     `4.440892098500626e-16` vs this pack `0.0`: the two engines land on OPPOSITE SIDES
     of the `max(0.0, ...)` floor. A broader round-value sweep found 305 such
     floor-branch splits — this is not a cherry-picked edge case.
3. **"An EXACT integer multiply (both operands fit well inside 2^53)" named the wrong
   condition** — exactness of `raw_extraction * entropy_factor_x1e6` depends on the
   PRODUCT's significand fitting 53 bits, not on either operand's magnitude
   individually; re-derived correctly in `metabolism.bsl`'s own D-1.
4. **Tick-1 exactness is a special case, not the general rule** — `raw_extraction` is
   an exact integer only when every field is freshly int-seeded (true of every
   conformance fixture in this pack, on tick 1). From tick 2 onward `biocapacity` feeds
   back as a computed float, and in live gameplay `extraction_intensity` is a genuine
   float written by `ProductionSystem`, never an integer. Verified by multi-tick replay
   of both engines: for a `biocapacity=5` seed at the production defaults, the two
   engines already disagree by tick 3.

The `ecological-cost` term's own bound is still real and still measured: an exhaustive
sweep (int `raw_extraction` 1-2000 x every point of the 6-digit `entropy_factor` grid in
`(1.0, 3.0]` — 2,000,000 grid points, 4,000,000,000 combinations total) found a worst
deviation of exactly 1 ULP for that term alone, zero exceeding it. What changed is the
claim about what that bound implies for the rule's actual output — nothing, once the
term is subtracted under cancellation and clamped.

**One more undisclosed consequence of the bare-Int bypass itself, beyond the numeric
deviation**: the same `E-LEX-024`-exempt unsuffixed-`Int` `:const` that makes the
workaround possible also means nothing on the BSL surface refuses an
`entropy-factor-x1e6` outside `MetabolismDefines.entropy_factor`'s declared `(1.0, 3.0]`
domain, including negative — verified directly (a scenario declaring
`entropy-factor-x1e6` as `-500000` loads and runs through the real engine without
refusal). Only this rule's own in-body clamps contain the resulting value; nothing
enforces the coefficient's own domain.

**Why the (unbounded) numeric deviation is acceptable anyway (ADR183 §5.4): the frozen
Python engine is the contract source for STRUCTURE and ORDERING, never a bit-exact
correctness oracle.** What is constitutional (III.7) is the Rust engine's OWN
determinism — reproducible, not Python-identical — which integer-scaled arithmetic
satisfies exactly regardless of how far any single tick's output deviates from the
frozen engine's own number. The en-masse retirement of this whole workaround class (a
genuine `Real × Ratio` operator, or `Ratio`-typed field storage) is chartered as
**workstream 3 of the post-port refactor program** (Director directive 2026-08-11,
tracked at GitHub issue #502), not attempted by this port. `metabolism-entropy-low-
conformance.bscn`/`metabolism-entropy-high-conformance.bscn` still mutation-verify the
workaround carries the coefficient's EFFECT (a magnitude check); `metabolism-rounding-
divergence-conformance.bscn` pins ONE concrete, reproducible deviation (`biocapacity=3`,
2 ULP) — not the general (unbounded) case — and a PASS there means "matches this pack's
own prior deterministic output", never "matches the frozen engine".

**This port's conformance suite is NOT guaranteed bit-exact against the frozen engine in
general — it is bit-IDENTICAL for several of its own vectors (including `nominal-county`
at the shipped default `entropy_factor=1.2`, and `low-entropy-county`/three others) and
DIVERGENT for others** (the dedicated `metabolism-rounding-divergence-conformance.bscn`
vector, and, per counterexamples A/B above, plausibly others this pack does not
exercise). Every vector this pack actually pins was independently verified against the
frozen engine at authoring time; which specific seeds land on which side is a fact about
each seed's own cancellation pattern, not a property this port controls or claims to.

---

## 4. The two flagged structural wrinkles

### (a) Spec-070 sovereign channel — BLOCKED, confirmed

`context.persistent_data["balkanization.metabolic_impact_by_territory"]` has no BSL
read path. `bsl-language.rst` §2.5 closes `<bind-src>` at exactly four forms —
`:field`/`:const`/`:metric`/`:tick` — none of which can name `context.persistent_data`.
`reports/bsl-gap-analysis-2026-08-10.md`'s **Q6** section ("Graph-scope state, read and
write... the single most pervasive gap in the estate") lists **Metabolism** by name
among its 22 affected systems (line 246) and names this exact handoff explicitly:
*"three of these values are one-tick-lagged handoffs (**Sovereignty → Metabolism**,
MarketScissors → WealthDistribution, Production → ImperialRent)... the Metabolism survey
proved this by construction"* (lines 277-281). Q6's own recommended fix (route (a):
model the channel as an ordinary singleton-carrier `NodeType` field) is spec-level
content work this port does not perform — it is exactly the kind of new-content-not-a-
retrofit decision Lifecycle's own D-1 declined to make unilaterally. **Disposition
confirmed BLOCKED, not assumed**: dropped whole, matching the pattern Lifecycle's own
header uses for its two dropped rules, D-4 below records it.

### (b) Phases 2-3 (global overshoot) — BLOCKED, confirmed at the execution-engine level

This is not simply "no cross-namespace fold exists" — it is that **the mechanism the
BSL spec and its load-time validator already support for this exact case
(`(domain :graph)`) is not wired into the tick-execution engine at all**:

- `rust/crates/babylon-bsl/src/domain.rs` fully implements `(domain :graph)` resolution
  at **load time** — `RuleDomain::Graph`, `resolve_domain`, `check_graph_domain` — and
  its own test suite proves a `(domain :graph)` rule with a `fold sum` over
  `(nodes NodeType/ORGANIZATION)` resolves cleanly (`domain.rs:401-414`,
  `442-453`).
- But `rust/crates/babylon-bsl/src/tick.rs::run_tick` (the function every entry point —
  `run_once`/`run_once_into`, hence the CLI and `babylon-client`'s engine link — actually
  calls) **never reads `loaded.domain` at all**: it unconditionally calls
  `subject_type_of(&loaded.bindings)` (`tick.rs:367`), which derives a subject type
  purely from `:field` binding namespaces and knows nothing about `RuleDomain::Graph`.
  A `(domain :graph)` rule would either fail outright at `subject_type_of` ("the rule
  declares no `:field` binding, so it names no subject type" — if it has none at rule
  scope) or, worse, be silently misinterpreted as an ordinary per-node rule over
  whatever type a `fold`'s own `:field` binding happens to name (since
  `subject_type_of` does not distinguish a binding referenced only inside a fold body
  from one referenced at rule scope, unlike `domain.rs`'s own `self_scoped_segments`,
  which correctly excludes fold-body-only references).
- Confirmed independently: `rust/crates/babylon-tick/src/lib.rs` constructs its
  `DefinesEnv`/driver state with `metrics: HashSet::new()` (line 122) — **zero graph-level
  metrics are registered anywhere in the actual driver** — and (corrected citation: an
  earlier revision of this row ran `rg -rn "domain :graph|domain.*graph" rust/crates/
  babylon-tick/`, which — per the repo's own known gotcha, `-r` is ripgrep's REPLACE flag,
  not "recursive", and `rg -rn` silently rewrites matches to the literal string `n` rather
  than searching — actually printed a mangled two-line excerpt of this very assessment's
  own `metabolism.bsl`, not a real crate-wide search; the conclusion below happened to
  still be true, but that command never proved it) `rg -n '\(fold ' rust/crates/babylon-tick/`
  and `rg -n 'domain :graph' rust/crates/babylon-tick/ -g'*.bscn' -g'*.rs'` (scoped to
  scenario and Rust-test files, excluding this pack's own prose commentary describing the
  gap) both return **zero hits**: no scenario, rule, or test in the crate that actually
  runs a tick has ever exercised `(domain :graph)` or `fold` end to end.
- `reports/bsl-gap-analysis-2026-08-10.md`'s **Q12** section states this precisely:
  *"Three of them perform exactly one graph-level check per tick — ControlRatio's
  four-phase state machine, **Metabolism's overshoot check**, FieldDerivative's
  principal-contradiction pick. Under per-node inference those would emit once per
  node"* (lines 386-389) — i.e. even the SPEC's own inference rule would fire this
  check once per `TERRITORY` (or per `SOCIAL_CLASS`, depending which namespace won),
  never once per tick, which is why an explicit `(domain :graph)` declaration is
  required and why that declaration's absence from the execution engine is fatal to
  this block specifically.

Even setting the engine gap aside, the aggregation itself spans **two** node-type
namespaces in one rule — `sum(TERRITORY.biocapacity)` and
`sum(SOCIAL_CLASS.(s_bio+s_class)*population) filtered by active` — which would need
**two** `fold`s over two different `(nodes NodeType/...)` queries inside one
`(domain :graph)` rule. Nothing in `fundamental-theorem.bsl` (single-namespace,
`social-class` only) or any other landed pack exercises a two-namespace fold, so this
compounding question is recorded but is moot while the driver cannot run a
`(domain :graph)` rule at all.

**Disposition: BLOCKED, confirmed rather than assumed**, on two independent, precisely
cited grounds (Q6/Q12's own spec-level gap, AND the deeper execution-engine gap this
assessment found by reading `tick.rs` directly). Phases 2 and 3 are dropped from this
port whole; recorded as **D-4** in the rule pack.

---

## 5. Defaulted-attribute trap sweep (`.get(field, default)` fallbacks)

Every `attrs.get(...)` fallback in `metabolism.py`'s Phase 1 block was checked against
who else in the live engine writes that attribute onto `TERRITORY` nodes — the same
question Dispossession's D-1 answered for its five rate inputs, and the SAME reasoning
does not give the same answer for every field here:

| Field | `:const` or `:field`? | Evidence |
|---|---|---|
| `extraction_intensity` | **`:field`** | Written live, per-territory, by `ProductionSystem` (`production.py:268`, `graph.update_node(node.id, extraction_intensity=intensity)`), itself derived from `total_production / max_biocapacity` (`:251-268`) — genuinely varying data, confirmed by `reports/bsl-gap-analysis-2026-08-10.md` row 13.0 marking Metabolism **"No"** under "Dormant on canonical" (unlike Dispossession's "Yes (zero-rate inputs)"). Treating this as `:const` would misrepresent a live production channel, the opposite of Dispossession's D-1 justification. |
| `biocapacity` / `max_biocapacity` | **`:field`** | Self-evidently per-node evolving state (this system's own primary output) AND seeded with genuinely different values per territory at scenario-build time — `src/babylon/engine/scenarios/_legacy.py:673-677`: `biocap = 150.0` / `40.0` / `100.0` depending on sector classification, not a uniform constant. |
| `regeneration_rate` | **`:const`** (D-record) | Grep-verified: no scenario builder anywhere in `src/babylon/engine/scenarios/` ever assigns a `regeneration_rate=` distinct from the `Territory` Pydantic model's own default (`models/entities/territory.py:165-170`, `default=0.02`), and nothing else in the engine writes it onto a `TERRITORY` node (only `metabolism.py`'s own `.get()` read and `SubstrateSystem`'s **unrelated** `raw_material_stock`-side `regeneration_rate` — a different `SubstrateDefines` coefficient feeding a different formula call on a different attribute; `substrate.py`'s own module doc states explicitly *"Does NOT touch `Territory.biocapacity`/`MetabolismSystem`"*). Every territory in the shipped engine reads the same 0.02 — the exact "per-node storage never observably diverges from the global constant" shape Dispossession's D-1 and Lifecycle's D-1 both name. |
| `entropy_factor` | **`:const`** (GameDefines global coefficient, not per-node at all) | See §3 for the representation problem; not a defaulted-attribute question — it is read from `services.defines.metabolism.entropy_factor` in the frozen Python, never off a node. |
| `hysteresis_rate` | **`:const`** (GameDefines global coefficient) | Same — `services.defines.metabolism.hysteresis_rate`, `[0.001, 0.01]`, fits an ordinary `c`-suffixed Coefficient literal with no representation problem (well under the `[0,1]` cap). |

---

## 6. D-record numbering — a second contradiction of the task brief, flagged

The task brief's instruction was to find "the register" (bsl-language.rst's
Draft-Ruling Register, currently D1-D99) and "use the next free numbers" for this
port's D-records, citing the sync-guard test that "enforces register uniqueness."
**Read directly, that sync-guard test does not cover what the brief assumed.**
`tests/unit/reference/test_bsl_grammar_sync.py::TestTheDraftRulingRegisterHasNoDuplicateRowNumbers`
(landed in commit `723a4c23`, read in full) scans **only** the text strictly between
`"\nDraft-Ruling Register\n"` and `"\nSee Also\n"` inside `docs/reference/bsl-language.rst`
itself, for rows matching the literal pattern `"   * - D<n>"`. That register is for
**language-level** draft rulings about BSL's own grammar/semantics (D97 = the `floor`
intrinsic, D98 = the `real` intrinsic-type-name, D99 = Currency×Ratio) — it has nothing
to do with, and does not enumerate, the **port-content** `D-1`, `D-2`, ... convention
used *inside* individual `.bsl` rule-pack files. Read directly: `dispossession.bsl` uses
its own `D-1` through `D-5`, and `lifecycle.bsl` uses its own `D-1` through `D-6`, each
**restarting at D-1**, entirely independent of the spec register — neither file's
D-records appear as `"   * - D<n>"` rows in `bsl-language.rst` at all (grep-confirmed:
zero hits for `dispossession` or `lifecycle` inside the Draft-Ruling Register section).

**This port follows the actual precedent** (file-local numbering, mirroring
Dispossession and Lifecycle exactly) rather than the brief's assumption of continuing
the global spec register at D100+. `metabolism.bsl`'s own D-1 through D-5 are local to
that file (D-5 added in the F1-F5 fix round: the four dropped `.get(attr, default)`
fallbacks now have an explicit disposition). No new row is added to `bsl-language.rst`'s
Draft-Ruling Register by this port — the §3 finding (`Real × Ratio` has no operator) is
a genuine candidate for a FUTURE such row, but minting one is a language-surface ruling
this port's scope does not license; it is recorded here and in the rule pack's own D-1
for the spec owners, alongside its scheduled retirement as workstream 3 of the
post-port refactor program (Director directive 2026-08-11, GitHub issue #502).

---

## 7. Plan for the BSL rule pack

One rule, `metabolism/biocapacity-update`, domain `TERRITORY` (inferred from its
`:field` bindings, matching the fundamental-theorem/dispossession precedent — no
explicit `(domain ...)` needed since every reference is self-scoped to one namespace).

Bindings: `current` (`:field territory/biocapacity`), `max-cap`
(`:field territory/max-biocapacity`), `extraction-intensity`
(`:field territory/extraction-intensity`), `regeneration-rate`
(`:const metabolism/regeneration-rate`, D-2), `entropy-factor-x1e6`
(`:const metabolism/entropy-factor-x1e6`, D-1's workaround), `hysteresis-rate`
(`:const metabolism/hysteresis-rate`).

No `(when ...)` guard in the frozen source (the Phase 1 loop has no `continue`) — every
`TERRITORY` node gets its effects unconditionally, matching `metabolism.py:91-118`
exactly (no `when` form needed, mirroring `lifecycle.bsl`'s unconditional Block 1).

Effects: two unconditional `update-node` writes (`biocapacity`, `max-biocapacity`) — no
`emit`, since Phase 1 publishes no event in the frozen source.

Conformance fixtures, as actually delivered across Phase 3 and the F1-F5 fix round
(eight scenarios, 28 tests across 8 Rust test files) — the original plan's four vectors
undersold what turned out to be necessary once adversarial review checked each clamp's
operand choice for mutation-liveness, not just its presence:

- **Nominal trajectory** (`metabolism-conformance.bscn`): regen > extraction, neither
  clamp binds.
- **Hysteresis inert** (`metabolism-conformance.bscn`): `extraction_intensity = 0`, pure
  regeneration, `max_biocapacity` bit-identical to its seed.
- **Zero floor + ratchet** (`metabolism-conformance.bscn`): heavy extraction at the
  ceiling floors `biocapacity` to exactly `0` AND strictly ratchets `max_biocapacity`.
- **Ceiling clamp binds, unratcheted** (`metabolism-ceiling-conformance.bscn`): boosted
  `regeneration_rate`, `extraction_intensity=0`.
- **Ceiling clamp binds against the RATCHETED value**
  (`metabolism-ratcheted-ceiling-conformance.bscn`, added F2): the operand the clamp
  compares against (`new-max`, not `max-cap`) was mutation-dead until this vector —
  needs `entropy_factor`/`hysteresis_rate` ALSO pushed off their production defaults,
  not just `regeneration_rate`.
- **Regeneration suppressed exactly at the ceiling boundary**
  (`metabolism-ceiling-suppression-conformance.bscn`, added F3): `current == max_cap`
  chosen so the `>=`-vs-`>` guard's two branches land on different sides of the `max(0,
  ...)` floor — the earlier `zero-floor-county` vector floored both branches to the
  identical `0.0`, hiding the guard's boundary entirely.
- **`new-max` floor clamp binds** (`metabolism-extreme-damage-conformance.bscn`, added
  F4): an unbounded `biocapacity` seed pushes `damage` past `max_biocapacity`, legal
  because neither field carries a Pydantic upper bound.
- **Entropy-factor magnitude pair** (`metabolism-entropy-{low,high}-conformance.bscn`):
  the identical territory at `entropy_factor=1.01` vs `3.0`, mutation-verifying the D-1
  workaround carries the coefficient's EFFECT.
- **Rounding-divergence pin** (`metabolism-rounding-divergence-conformance.bscn`, added
  F1): PINS the D-1 workaround's bit-level deviation from the frozen engine rather than
  claiming exactness — see §3's correction.

Every clamp/guard/branch load-bearing enough to change the port's OUTPUT is now backed
by at least one vector where mutating it flips a specific, named test — verified by
hand for each (§3, and the F2/F3/F4 commits' own mutation records).
