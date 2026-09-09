# The subsistence-unit reconciliation — units, the household crossing, τ's option space

Deliverable 2 of the #491 rung-ladder train (`docs/superpowers/plans/2026-08-17-491-rung-ladder.md`
§3, ADR202 R1a). **No code changes made producing this document** — pure paper derivation plus
in-repo verification (`sed`/`grep`/`sqlite3` reads only, no `cargo`, no `uv run pytest`). Starting
point: BASE `1b6c5e10` (#491 T1's final commit — the kind-straddle repair ceremony, ADR216).

This record re-opens and re-verifies every `file:line` citation below against this worktree, rather
than carrying it over from the plan text. Citations into `docs/reference/bsl-language.rst` and
`rust/crates/babylon-bsl/src/evaluator.rs` drifted: the plan predates ADR211/ADR212/ADR213/ADR216,
whose D158–D184 insertions shifted every downstream line number by tens of lines. This record uses
the corrected citations throughout and calls out each stale plan-cited number explicitly, so the
drift itself joins the record instead of disappearing into a silent fix.

**This plan's task list (§9, T2) posed τ (DP-5) and the household→person crossing (DP-7) as
reserved-not-decided questions.** The DP sitting has since ruled both: it ran and posted its verdict
to issue #491 at 2026-08-18T02:42:38Z, with the formal panel record following at 02:53:02Z. §3 and
§4 below record the corrected analysis exactly as the plan specifies (the analysis is what forced
the questions to the Director in the first place, and stands regardless of how the sitting ruled
them), and then record the actual dispositions as **facts already established by the legitimate
delegated process**, cited to their source — this document does not decide either question; it
reports a decision already made and provides the derivation that motivated it.

---

## 1. The problem, in units

Three quantities in the tree, not commensurable until this record:

| Quantity | Site | Declared type | Actual dimension |
|---|---|---|---|
| `SurvivalDefines.default_subsistence = 0.3` | `src/babylon/config/defines/survival.py:23-28` (`ge=0.0, le=1.0`) | dimensionless `[0,1]` | compared against `wealth/population`, an **unbounded Currency-per-member** (`src/babylon/engine/systems/survival.py:143,154-158`) — a category error, the same species as the retired `steepness_k` |
| `EconomyDefines.base_subsistence = 0.0005` | `src/babylon/config/defines/economy_basic.py:267-275` | `[0,0.5]` float | a **per-member-per-tick rate**: `cost = (base_subsistence × population) × multiplier`, `src/babylon/engine/systems/vitality.py:118-120` |
| `s_bio + s_class` | `rust/crates/babylon-tick/content/rules/vitality.bsl:49-50,74`; frozen `src/babylon/formulas/vitality.py:29` (docstring) | `int intensive` today, `currency intensive` after T3 | **Currency per member per tick** — the value of labour power's two components |

Re-verified this pass:

- `survival.py:23-28` — `default_subsistence: float = Field(default=0.3, ge=0.0, le=1.0,
  description="Game design: minimum wealth for survival through compliance.")`. Exact match, no
  drift.
- `economy_basic.py:267-275` — `base_subsistence: float = Field(default=0.0005, ge=0.0, le=0.5,
  description="Biological floor: fixed cost per tick (LINEAR), scaled by class multiplier")`, with
  the preceding comment `# LINEAR burn: cost = base_subsistence * class_multiplier`. Exact match.
- `engine/systems/vitality.py:118-120` — `multiplier = attrs.get("subsistence_multiplier", 1.0)` /
  `# Phase 3 change: Scale by population` / `cost = (base_subsistence * population) * multiplier`.
  Exact match, no drift (Python-side line numbers stay stable across this train; only the rst/Rust
  citations drifted).
- `vitality.bsl:49-50` — the `s-bio`/`s-class` field bindings; `:74` —
  `(binding consumption-needs :expr (+ s-bio s-class))`. Exact match.
- `formulas/vitality.py:29` — docstring: *"subsistence_needs: Per-capita subsistence requirement
  (s_bio + s_class)."* Exact match.

### 1.1 The `population == 1` guard accident

The units table above is not merely a theoretical mismatch — the estate's own tests mask it with a
specific, named accident: `population` **defaults to `1`**, not to an error, absence sentinel, or
`0`, at every read site that touches the survival calculus:

- `src/babylon/engine/systems/survival.py:128` — `population = attrs.get("population", 1)  # Mass
  Line Phase 4`.
- `src/babylon/engine/systems/vitality.py:109` — `population = attrs.get("population", 1)`.
- `tests/unit/engine/systems/test_survival.py:45` — the shared fixture helper
  `_create_entity_node(..., population: int = 1, ...)` — the test suite's own **default population
  is 1**, not a randomly-chosen edge case.

When `population == 1`, `wealth_per_capita = wealth / population` collapses to `wealth` itself — the
`÷ population` operation becomes a numerical no-op, and the STOCK/FLOW/dimensionless-fraction
distinction the units table exists to name disappears from view. The test suite does not merely
tolerate this collapse; it **names and asserts on it as a feature**:
`tests/unit/engine/systems/test_survival.py:134-138`, `test_backward_compat_pop_1`:

> "Population=1 with wealth=X has same P(S|A) as before (no regression). With population=1,
> wealth_per_capita == wealth (aggregate). This ensures backward compatibility with existing
> single-entity scenarios."

With `population` pinned at 1 and hand-picked wealth values in the tens-to-thousands range
(`wealth=100.0`, `wealth=0.1`, `wealth=1000.0` at lines 105, 146, 154 of the same file),
`wealth_per_capita` lands in a range where the sigmoid `1/(1+e^(-k(wealth_per_capita -
default_subsistence)))` (`src/babylon/formulas/survival_calculus.py:41-43`, `steepness_k` default
10.0) produces plausible-looking transitions against `default_subsistence = 0.3`. This is what "made
the tree look commensurable" — not a real unit correspondence, but the fixture default silently
turning a per-capita computation into an identity, at exactly the population value (1) where
`wealth` (a Currency stock, dimension `$`) and `wealth_per_capita` (nominally `$/member`) are
numerically indistinguishable. At any realistic population (thousands to millions of members, as
`test_higher_pop_lower_p_acquiescence`, lines 89–132 of the same file, itself demonstrates —
`population=50000` collapses `wealth_per_capita` to `0.02`), the mismatch between a `[0,1]`
dimensionless design constant and an unbounded Currency-per-member reading is immediate and severe.
The estate's own test docstrings (lines 5–7 and 82–86 of the same file) already state this in plain
language for the *aggregate-vs-per-capita* distinction; what they do not name is that the comparison
itself pairs the per-capita reading against a quantity (`default_subsistence`) nobody ever gave a
Currency dimension — a second, un-named category error riding underneath the one Phase 4 fixed.

---

## 2. The dimensional error nobody had named

The ladder's comparison is `cut_{k-1} · w̄ ≥ S`.

- `w̄ = wealth ÷ population` is a **STOCK** per member (Currency).
- `S = s_bio (+ s_class)` is a **FLOW** per member per tick (Currency · tick⁻¹).

**A stock does not compare to a flow.** A prior treatment of this problem declared the unit
"per-member money" and stopped — true of the left side, false of the right. The frozen engine
already contains the missing term, unnamed. Re-verified verbatim, `src/babylon/formulas/vitality.py`:

```python
# lines 38-43
coverage_ratio = wealth_per_capita / subsistence_needs      # STOCK ÷ FLOW → TICKS
threshold = 1.0 + inequality

# If coverage exceeds threshold, even the poorest survive
if coverage_ratio >= threshold:                             # survive
    return 0.0
```

`coverage_ratio` is a **duration in ticks** — how long a member can reproduce out of held wealth.
`threshold` is that duration's floor.

**Read this as a diagnosis, not as a derivation.** It shows the frozen engine already carries the
missing dimensional term; it does **not** license transcribing that term's *value* into the ladder —
see §4, where an earlier reading of this same diagnostic did exactly that, and the move is withdrawn
there.

---

## 3. The unit system, and the two constructs that bridge it

Declared unit system — the mass rows and the household→person crossing are the half of this system a
prior pass left unnamed:

```
wealth            : Currency                        (stock,  extensive)
population        : count of MEMBERS                (        extensive)
w̄  = wealth ÷ population : Currency / member        (stock,  intensive)   ← licensed, ADR202 R1(c) / D181
s_bio, s_class    : Currency / member / tick         (flow,   intensive)
S   = s_bio  |  s_bio + s_class                      (flow,   intensive)   ← level set, ADR210 R13
τ   : ticks                                          (Ratio defconst)      ← DP-5 = A now, C revisit (§4)
S_stock = S · τ   : Currency / member                (stock,  intensive)
mass_k            : share of MEMBERS in rung k       (dimensionless, intensive, Σ = 1)
  derived from
mass^hh_k         : share of HOUSEHOLDS in rung k    (dimensionless — B19001 household_count)
η_k               : members per household in rung k, RELATIVE to the county mean
                    ← THE CROSSING. §3.1 below. UNMEASURED in-repo. DP-7 = A (§3.2).
```

The comparison becomes `cut_{k-1} · w̄ ≥ S · τ` — Currency-per-member on both sides, dimensionally
exact. D181 (register row below, `docs/reference/bsl-language.rst`) licenses `w̄`'s own kind
(`extensive ÷ extensive → intensive`), the #491 T1 repair of the E-TYPE-040 `*`/`/` bullet; the mass
rows are the other half of the dimensional story a prior pass got wrong.

### 3.1 The household→person crossing, declared

A2's masses come from `fact_census_income.household_count` — **households** (re-verified this pass:
`sqlite3 data/sqlite/marxist-data-3NF.sqlite ".schema fact_census_income"` — `household_count
INTEGER NOT NULL`, keyed `(county_id, source_id, bracket_id, time_id, race_id)`). The measure
consumes them as **member** shares: `clearing = Σ_k mass_k · c_k` over a class whose `population`
counts members, and `deaths = floor(population × failing × κ)` kills people, not households. A
scrupulous per-member-per-tick unit system that then imports a per-household shape into per-member
space without naming the crossing repeats exactly the accident §1.1 diagnosed, one level up the
derivation. Named here:

```
mass_k  =  (mass^hh_k · η_k) / Σ_j (mass^hh_j · η_j)
```

- **A county-constant household size cancels exactly.** Masses arrive as normalised shares, so if
  `η` is the same in every rung the conversion is a numerical no-op. The whole content of the
  assumption is **rung-independence of household size within a county at the declared vintage** —
  `η_k ≡ 1` for all *k*. That, and only that, is the assumption.
- **This does not hold in the world**, and it fails in the direction that matters — household size
  covaries with income across exactly the tails the ladder exists to resolve. The size of that gap
  matters, and:
- **No in-repo instrument measures it.** Re-verified this pass against
  `data/sqlite/marxist-data-3NF.sqlite`:
  - `fact_census_housing` carries *tenure* counts, not size: `.schema fact_census_housing` shows
    `tenure_id`/`household_count` keyed by `(county_id, source_id, tenure_id, time_id, race_id)` —
    owner/renter occupied-unit counts, no size dimension anywhere in the table.
  - `dim_county` carries no population column: `.schema dim_county` — `county_id, fips, state_id,
    county_fips, county_name, h3_res4` only.
  - No B25010 (`household size`), B11016 (`household type by household size`), or B19019
    (`median household income by household size`) table exists anywhere in the schema — `.tables`
    against every plausible name substring returns nothing.

  The reference DB cannot bound this assumption; a record that claimed a bound it cannot compute
  would be exactly the fabrication this train exists to stop producing.

**Declaration, in three places, so it cannot be re-hidden** (sites named here; the first two are
content and land downstream at T4 and T7/T8, the third lands right here, in §9's Aleksandrov table):

1. **Content:** `(defconst wealth-sketch/household-person-equivalence 1.0r)`, which T4's conformance
   scenario authors, with a header comment naming the declared `η_k ≡ 1` bridge, stating that it
   cancels by construction at the identity value, and naming the object that would *change* numbers:
   a per-rung vector the reference DB does not carry. (Confirmed absent today: `grep -rn
   "household-person-equivalence" rust/` returns nothing in this worktree.)
2. **Artifact:** A2 (T7/T8) publishes **both** `mass_household` (measured) and `mass_member`
   (consumed), plus an **empty** `person_equivalence` column (L-ABS: unmeasured stays empty, never
   `1.0` written as if measured). Today `mass_member == mass_household` **as a declared consequence,
   published as such** — never as a coincidence a later reader has to reverse-engineer.
3. **Aleksandrov + PROVISIONAL:** its own row in §9 below, and the `#510` PROVISIONAL string
   extended from "income ≠ wealth" to "**income ≠ wealth AND households ≠ members**" at every entry
   point (the T10 `#510` grep checklist item, plan §8).

### 3.2 DP-7's disposition

**The plan posed DP-7 with three options** (§15): (A) the declared identity crossing above; (B)
household units end-to-end (which relocates rather than resolves the conversion — the engine carries
no per-class household count, so the same crossing re-appears at the consumer with less visibility);
(C) buy a size-by-income instrument (ACS B19019/B11016/B25010 family) under #546's wire-before-buy
procedure.

**Ruled — posted on #491, 2026-08-18T02:42:38Z:** *"DP-7 = A: the η ≡ 1 crossing DECLARED (named
defconst, published dual mass columns, Aleksandrov row, extended #510 string); the measured-η
acquisition folded into #510's revisit."* Provenance: Director-delegated to the standing
gameplay-and-pedagogy compass ("engaging AND instills correct theory" as one criterion),
controller-adjudicated under that delegation, recorded on the issue with the other nine DP rulings
from the same sitting.

This record does not decide DP-7 — it transcribes a decision already made through the plan's own §15
process and cites it. Nothing in T2's scope depends on the crossing's size; §3.1's three
declaration sites remain the correct places to carry the `η ≡ 1` assumption forward, now with a
ruling attached rather than an open option space.

---

## 4. τ, the subsistence horizon — the corrected analysis, and its disposition

A prior pass wrote "τ = 1 tick, derived — not picked." **That derivation does not hold.** Three
independent problems, each re-verified this pass:

1. **The source disqualifies itself.** An earlier pass read τ off the frozen `coverage_ratio ≥ 1 +
   inequality` test. ADR210 R13 (`ai/decisions/ADR210_checkpoint_a_campaign_rulings.yaml:153-159`)
   states this outright: *"the divergence D-row from the frozen s_bio + s_class death threshold is
   owed at the landing (ADR183: the frozen engine is a structure contract, not a threshold
   oracle)."* κ carries an explicit licence for frozen-magnitude calibration (R14: *"its default
   value is DERIVED with the derivation recorded, not picked"*); **τ carries none**. Reading τ off
   the frozen threshold borrowed κ's licence without R14's authorisation.
2. **The level set is wrong.** Re-verified verbatim, `src/babylon/formulas/vitality.py:29`: the
   docstring gives `subsistence_needs` as *"Per-capita subsistence requirement (**s_bio +
   s_class**)"* — the level set ADR210 R13 assigns to **acquiescence**. R13 assigns **mortality
   `s_bio` alone** (line 154: *"mortality reads s_bio (the biological floor kills); acquiescence
   reads s_bio + s_class"*). The frozen `1` is a horizon *for the frozen S*, and the
   mortality reading changes S underneath it: pushed through honestly, this does **not** give 1.
   Preserving the frozen *death level* at R13's mortality level set requires

   ```
   τ_bio  =  (s_bio + s_class) / s_bio        — class-varying, data-dependent, ≠ 1
   ```

   which also re-imports `s_class` into the level set R13 just separated. Transcribing the
   frozen value and honouring R13 are **incompatible**; the earlier pass did neither and named the
   residual `1` a derivation regardless.
3. **It contradicted the plan's own best idea.** §9 traces τ as a hold-out horizon; H3 derives
   ReserveArmy's `L = reserve_army_stock ÷ absorption_flow` **from material flows** — and its
   producer is currently hardcoded to zero, re-verified this pass:
   `src/babylon/domain/economics/reserve_army/accumulation.py:133` —
   `expansion_absorption=0,` (a literal zero, not yet a flow computation). Two horizons of the same
   declared kind, one stipulated and one flow-derived-but-not-yet-wired. One horizon story must
   stand — the substance of DP-5, and H3's elegance is the argument for eventually deriving both.

### 4.1 Two consequences hold under every option, regardless of DP-5's disposition

1. **This record explains `social-class/inequality`.** `vitality-conformance.bscn:30-34`
   (re-verified, corrected from the plan's cited `:31-34` by one line): *"Declared and seeded, read
   by NO rule in this pack. It is here because the frozen engine's Phase 2 requires it and this
   world mirrors that fixture node for node; the rule that would read it does not exist, for the two
   reasons the .bsl header records."* The field is the frozen **dispersion surrogate** — its own
   docstring says so (`formulas/vitality.py:24-25`: *"The formula ensures that with high inequality
   (e.g., 0.8), you need almost 2x subsistence (1.8 coverage) to prevent deaths"*) — and the ladder
   measures what it faked. `inequality` enters the frozen form **twice** — the threshold
   (`vitality.py:39`) *and* the slope (`vitality.py:47`) — so retiring the field is also a slope
   change, which is κ's problem (§7, and register row D188's level-set split, which bounds κ's own
   fixture choice).
2. **`SurvivalDefines.default_subsistence` retires** alongside `steepness_k` under ADR173 (1) — a
   consequence for the Survival port to execute, **not a change this train makes to frozen Python.**

### 4.2 DP-5's ruling does not touch τ's home

A `.bscn` defconst — `(defconst vitality/subsistence-horizon …)` — for the identical reason ADR210
R14 gives for κ: a `GameDefines` field moves `defines_hash`, which `tools/regression_test.py`
compares as a hard gate across all 11 baselines, so it would fail `qa:regression` immediately and owe
a §6.5 ceremony for a constant the frozen engine has no consumer for. As content rather than a
`GameDefines` field, the conformance world may declare a fixture value and T4–T6 can execute before
this record's disposition
existed (the plan's own T4 scenario header text, unread by construction at the time someone authored
it, still says *"τ — FIXTURE-DECLARED, DP-5 pending"* — that comment is now stale relative to §4.3
below; this record flags it here so T4's own implementer updates it rather than rediscovering the
staleness independently).

### 4.3 DP-5's disposition

**The plan posed DP-5 with three options** (§15): (A) τ ≡ 1 tick, definitional — the tick **is** the
reproduction accounting period, invoking no frozen authority, letting the level set do the
theoretical work; (B) frozen-magnitude preservation → the class-varying `τ_bio` derived in item 2
above (**not recommended** — this is the threshold-oracle move R13/ADR183 forbid); (C) τ derived as a
material hold-out horizon, the same construction as ReserveArmy's `L` (theoretically strongest,
**blocked** on the absorption-flow producer, `accumulation.py:133`, which the ReserveArmy port owes
regardless of this train).

**Ruled — posted on #491, 2026-08-18T02:42:38Z:** *"DP-5 = A now, C as a named revisit at the
ReserveArmy port: τ ≡ 1 definitional ships; the shared material hold-out horizon (the emergent form)
lands when the absorption-flow producer exists."* Same provenance as DP-7: Director-delegated to the
gameplay-and-pedagogy compass, controller-adjudicated, recorded on the issue with the other nine DP
rulings from the same sitting (2026-08-18).

**What this ruling is, and is not.** It ships the *same number* (`τ = 1`) an earlier, withdrawn
reading also produced — but for a different, legitimate reason: option (A) invokes no frozen
authority and states its own theoretical warrant (the tick as the reproduction accounting period),
where the withdrawn reading invoked a threshold-oracle transcription ADR183 forbids. The numeric
coincidence is exactly that — a coincidence, not a vindication of the withdrawn derivation. Option
(C) — the H3-symmetric, flow-derived horizon — remains a **named, not-yet-executed revisit**, gated
on `accumulation.py:133`'s absorption-flow producer, which this train does not build. This record
does not decide DP-5; it transcribes and cites an already-made decision, exactly as for DP-7.

---

## 5. The prohibition this record carries as law, not as a note

**Never compute `S / w̄`.** `Currency ÷ Currency → Coefficient` must land in `[0,1]`, or the
expression raises `E-EVAL-013`. Re-verified this pass (drifted from the plan's cited `:2496-2497`):
`docs/reference/bsl-language.rst:2533-2534` — *"``Currency ÷ Currency`` with a zero divisor is
``E-EVAL-012``; the ``Coefficient`` result must land in ``[0,1]`` or it is ``E-EVAL-013``."*
Runtime enforcement, re-verified (drifted from the plan's cited `evaluator.rs:1650-1660`):
`rust/crates/babylon-bsl/src/evaluator.rs:1795-1817` — the `"/"` arm checks the zero divisor first
(`EvalCode::DivisionByZero`, `E-EVAL-012`), then checks `(0..=b.micro_units()).contains(&a.micro_units())`
(or the mirrored negative-divisor range) and raises `EvalCode::CoefficientOutOfRange` (`E-EVAL-013`,
declared at `evaluator.rs:127,214`) when the ratio falls outside `[0,1]`.

A class below subsistence has `S · τ > w̄` by construction — that is what "below subsistence" means,
which means the dimensionless spelling `S / w̄` **fails at runtime for exactly the class the measure
exists to describe**. This is worse than an ordinary bug: an ordinary type or arithmetic defect surfaces on
some representative input during development. This one is invisible on every input where the class
is doing fine — the happy path a conformance suite naturally exercises first and most — and only
fires once a class crosses into the exact condition (below-subsistence) the ladder exists to
measure. A conformance suite built the ordinary way, testing the common case before the edge case,
would ship this defect and only discover it against real distributional data, at the tail the whole
project cares about. The comparison is money-vs-money, always: `cut_{k-1} · w̄ ≥ S · τ`, never
`w̄ / (S·τ)` against a normalised threshold.

---

## 6. The `Currency × Int` obstacle, named so T4 does not discover it

`Currency × Int` is `E-TYPE-030`. Re-verified this pass (drifted from the plan's cited
`bsl-language.rst:2478-2482`): `docs/reference/bsl-language.rst:2519-2525` — *"``E-TYPE-030``. In
particular ``Currency + Real``, ``Currency × Currency``, and ``Currency × Int`` are type errors;
multiply by a ``Coefficient`` or a declared-domain ``Ratio``, or divide by an ``Int``, instead."*
Runtime enforcement, re-verified (drifted from the plan's cited `evaluator.rs:1553-1557`):
`rust/crates/babylon-bsl/src/evaluator.rs:1694-1710` — the `"*"` arm on a `(Currency, other)` pair
requires `other` to destructure as `Value::Real` (the runtime coefficient carrier); an `Int` operand
returns `E-TYPE-030` explicitly, with the comment noting `Int` is a type error "at ANY value"
(`bsl-language.rst:849`) precisely so an integer whose float image would be a legal coefficient (0 or
1) cannot slip through.

The frozen drain's association order, `cost = (base_subsistence × population) × multiplier`
(`engine/systems/vitality.py:120`), thus has **no expression** in the Currency lane as written once
`population` is an `Int` field and `base_subsistence`/`cost` become `Currency`. Two named routes
name the obstacle here, so T4 (T4.3, "run the Currency-drain spike... never weaken an assertion to
make a spike pass") does not have to rediscover it from a failing test:

1. **The spike itself (T4.3).** Attempt the direct transcription; if `population` can carry, or a
   read at this call site can promote it to, the runtime `Real` coefficient lane rather than a
   declared `Int` field, the multiplication stands legal as written. This is the route to try first
   and the one the spike exists to test.
2. **The named fallback: a hydrated `currency extensive` cost field.** If the spike fails, compute
   `cost` in advance as its own `currency extensive` field (load- or seed-time computation from
   `base_subsistence × population × multiplier` outside the BSL arithmetic lane, the same way other
   pre-computed content fields arrive) rather than expressing the multiplication in-language. This
   moves the *association order* the frozen drain performs at tick-evaluation time to a load-time
   computation — a divergence from the frozen structure that owes its own D-row, not a silent
   substitution.

**Citation note.** The plan's own §8 (Global Constraints) attributes the authorisation for this
fallback to "§3.9 clause 1." Re-verified this pass: **no §3.9 section exists anywhere in the current
plan document** — `grep -n "^### 3\.\|^## 3\." docs/superpowers/plans/2026-08-17-491-rung-ladder.md`
shows §3 running §3.1 through §3.6 only. This is a dangling internal cross-reference, most likely a
renumbering artifact from an earlier revision of the plan (the document repeatedly distinguishes
"revision 1" from "revision 2"). It does not block anything this record or T4 needs — §8's own text
states the fallback's substance plainly (a hydrated `currency extensive` field, D-rowed for the
association-order divergence), reproduced above — but a reader who follows "§3.9 clause 1" to the
letter finds nothing there, which this note records rather than silently passes on.

---

## 7. The second constant — κ — a different kind of thing

`deaths = floor(population × failing × κ)` (ADR210 R14: `.bscn` defconst, default **derived** with
the derivation recorded).

- **τ** converts a *flow threshold* into a *stock threshold* — a duration. Unit: ticks.
- **κ** converts a *failing mass* (a share of members) into a *death flow* (members per tick) — a
  hazard rate. Unit: tick⁻¹.

Neither bends a curve: τ slides the level set, κ scales the flow uniformly. Shape lives entirely in
the measured distribution.

**κ's derivation (T6), the frozen form it must reproduce at one point, re-verified verbatim**
(`src/babylon/formulas/vitality.py:38-47`):

```python
coverage_ratio = wealth_per_capita / subsistence_needs
threshold = 1.0 + inequality
...
deficit = threshold - coverage_ratio
attrition_rate = deficit * (attrition_base_factor + inequality)
```

`inequality` enters the threshold **and** the slope, so a one-point fit reproduces the frozen flow
**at that fixture only** and diverges everywhere else. R14's "DERIVED with the derivation recorded"
demands three things of T6.3:

1. **Name and justify the fixture** — the calibration point declared by name, its
   `(coverage_ratio, inequality, population)` written out, with a one-sentence justification.
2. **Kill the level-set contamination by construction** — choose the fixture with `s_class = 0`, so
   R13's two level sets coincide at the reference point and κ absorbs only the shape substitution its
   licence covers. A fixture with `s_class > 0` records the contamination as a named quantity in the
   D-row instead of leaving it implicit.
3. **Publish the divergence surface** — a small table of frozen vs. ported deaths-per-tick over a
   declared `(coverage_ratio, inequality)` sweep, the evidence that κ is a scale and the rest is
   shape substitution.

κ multiplies whichever mass DP-6 rules the mortality driver (`failing` vs. `failing_certain`); that
choice leaves the derivation recipe unchanged but not the fitted value, so T6.3 records which driver
it fitted against. DP-6 sits outside this record's scope (Deliverable 2 does not touch the mortality
driver; the record notes it here only because κ's derivation depends on it).

---

## 8. Two standing items

### 8.1 County-varying subsistence stays open, and is not foreclosed

Issue #546 item 6 (re-verified verbatim, `gh issue view 546`): *"County-varying subsistence
threshold (ERS food atlases): does a spatially-varying subsistence cost enter P(S|A)? If chartered,
the entry point is `base_subsistence` inside the Survival Calculus — where it changes the rupture
condition — never a bolted-on food subsystem. Under ADR173 this is a genuine input to the EMERGENT
S-curve."* This unit system does not foreclose it: `s_bio`/`s_class` are already declared per-class
intensive fields (§3 above), so a per-`(class, county)` `S` needs **zero redesign** — a future ruling
on #546 item 6 is a seeding change, not an architecture change. Recorded here so that ruling, when it
comes, arrives knowing the unit system already carries the degree of freedom it would use.

### 8.2 R13 already rules the level sets, and their divergence D-row comes due here

ADR210 R13 (`ai/decisions/ADR210_checkpoint_a_campaign_rulings.yaml:153-159`) assigns the level sets:

> "LEVEL SETS ASSIGNED — mortality reads s_bio (the biological floor kills); acquiescence reads
> s_bio + s_class (compliance buys reproduction at the class's own standard). 'Two readings' means
> two genuine level sets. The divergence D-row from the frozen s_bio + s_class death threshold is
> owed at the landing (ADR183: the frozen engine is a structure contract, not a threshold oracle)."

The frozen engine's actual mortality computation, re-verified this pass
(`src/babylon/engine/systems/vitality.py:230-232`): `_calculate_deaths` reads `s_bio`, reads
`s_class`, and computes `subsistence_needs = s_bio + s_class` (line 232) as the value it hands to
`calculate_mortality_rate` — the frozen engine's death threshold **is** `s_bio + s_class`, the
combined level set R13 assigns to acquiescence, not the `s_bio`-alone level set R13 assigns to
mortality. R13's divergence D-row names no hypothetical: it names a real, already-observed gap
between what the frozen reference computes (mortality gated on `s_bio + s_class`) and what the
ruling directs the port to compute going forward (mortality gated on `s_bio` alone). This record
discharges R13's "owed at the landing" obligation with register row D188 below; T5/T6, which build
the Grinding-Attrition mortality rule proper under the split level sets, form the landing that
executes the departure R13 already ruled.

(The currently-landed `vitality.bsl` — from an earlier, separate port train, not this one — carries
its own `consumption-needs = s-bio + s-class` term at line 74, but that term guards the block-of-one
Reaper's *starvation* branch, not a Grinding-Attrition mortality rule; the file's own header, lines
12–40, states explicitly that Grinding Attrition is deliberately not yet transcribed. R13 and this
record leave it untouched — cited here only to show the frozen `s_bio + s_class` value is not an
abstraction; a currently-landed guard uses that same literal value, for a different mechanism.)

---

## 9. Aleksandrov traces (the material relation behind each construct)

| Construct | Trace |
|---|---|
| `wealth-mass-k` | The share of a class's members whose held wealth falls in rung *k* — a distribution of a real stock over real people. |
| `cut-k` | The *k*-th ACS bracket edge expressed as a ratio to the mean: a real dollar boundary in a measured schedule, made scale-free so it travels with the class's own mean. |
| `s_bio` | The per-member per-tick cost of biological reproduction — the value of labour power's physical component (*Capital* Vol. I, ch. 6). |
| `s_class` | Its historically-determined social component: what reproduction costs *at this class's own standard*. |
| `τ` | The number of ticks a member can reproduce out of held wealth before the wage relation compels submission. A stock buys time; τ is how much. **DP-5 = A now** (the tick's own accounting period), **C as a named revisit** (the same flow-derived object as ReserveArmy's `L`) at the ReserveArmy port, pending `accumulation.py:133`'s absorption-flow producer. The trace holds under either construction; only the derivation differs. |
| `κ` | The rate at which failure-to-reproduce becomes death — the observed mortality flow of a population that cannot reproduce itself. |
| `clearing` | The mass of class members the ladder can **establish** reproduce themselves for τ ticks at S — a lower bound on the true share, because a rung counts only when its whole span clears. A headcount, not a curve. |
| `failing_certain` | Its dual: the mass the ladder can establish **cannot** reproduce (the whole rung sits below the threshold). Also a lower bound. Which mass mortality reads is DP-6, outside this record's scope. |
| `straddle_band` | `1 − clearing − failing_certain` — exactly the mass of the one rung the threshold cuts through. The ladder's declared resolution: the members whose fate the K=16 grid cannot resolve. Published, never silently assigned to either side. |
| `η` (`household-person-equivalence`) | How many members a household in rung *k* contains, relative to the county mean. The bridge between an instrument that counts households and a class that contains people. **Declared at `η_k ≡ 1` — DP-7 = A, ruled 2026-08-18**; unmeasured in-repo (§3 above); the measured-η acquisition folds into #510's revisit. |
| `mean_county` | The county's mean household income at the declared vintage — the scale that makes a dollar edge comparable to a class's engine-computed mean wealth. Not in the reference DB; DP-4 (outside this record's scope) rules its reconstruction route. |
| `relation_class` | The relation each person stands in to the means of production, as the Census measures it — outside this record's scope (DP-1). |
| `unemployed`, `not_in_labor_force` | Measured ACS labour-force categories, outside this record's scope (DP-1). |
| `pareto_alpha`, top `midratio` | The measured thinning rate of the top of a county's income schedule, outside this record's scope (DP-4). |

---

## 10. Register rows

Four new rows, `docs/reference/bsl-language.rst` Draft-Ruling Register, D185–D188 (tail re-verified
this pass at **D184**, `bsl-language.rst:8337-8365`; `rg -ow 'D1[0-9]{2}|D2[0-9]{2}'` across `docs/
ai/ reports/ rust/`, target/lockfiles excluded, confirms D184 as the ceiling — no drift since T1
landed it). Full text applied to that file in this commit, keeping the register table's own
three-column convention (`#` / `Section` / `Ruling`).

- **D185** — the declared unit system (§3 above): the dimensional table (§1), the STOCK/FLOW
  distinction, `coverage_ratio` as the frozen engine's unnamed missing term, and the household→person
  crossing (§3.1–3.2, DP-7 = A ruled).
- **D186** — τ, the subsistence horizon (§4 above): the corrected analysis (disqualified oracle,
  wrong level set, H3 contradiction), τ's `.bscn` home, and DP-5's disposition (A now, C a named
  revisit at the ReserveArmy port).
- **D187** — the money-vs-money law (§5 above): `E-EVAL-013` as law, never `S / w̄`, and why the
  failure mode is worse than an ordinary bug.
- **D188** — the level-set assignment (ADR210 R13) and its owed divergence D-row (§8.2 above): the
  frozen engine's `s_bio + s_class` mortality computation, R13's split, and where the departure lands
  (T5/T6).

---

## 11. Gate

- `vale reports/subsistence-unit-reconciliation-2026-08-17.md` — clean except a run of
  **precedented jargon/citation residuals** (an exact count is unstable to quote here, since
  describing the residuals adds fresh instances of the flagged words themselves), falling into
  two kinds `vale` cannot tell apart from a genuine misspelling or wordiness flag: (a) technical
  vocabulary already established throughout this repo's own docs —
  `docstring`/`docstrings`, `wealth_per_capita`, `covaries`, `Aleksandrov`, `defconst` (×4),
  `hardcoded`, `fallback's`, `s_bio`/`s_class` (in this record's own prose, e.g. §7's Python
  snippet) — the same words `vale` already tolerates nowhere near a `Vocab` entry in
  `kind-straddle-repair-options-2026-08-18.md` and the rest of `reports/`; and (b) text inside
  verbatim quotations of source code, test docstrings, and ADR prose (`test_backward_compat_pop_1`'s
  docstring, `vitality-conformance.bscn`'s comment, ADR210 R13's own sentence) — rewording those to
  clear a style rule would misquote the cited source, so they stay as written. Two instances of the
  compound "frozen-magnitude" (κ's and DP-5 option B's own name from the plan/ADR210 R14) likewise
  stay, naming the cited construct rather than describing it loosely. No other finding remains: this
  pass rewrote every weasel word, passive-voice flag, cliché, and sentence-initial
  "So" outside a quotation.
- `vale docs/reference/bsl-language.rst` — clean on the touched region (register rows D185–D188)
  except the same three residual kinds: `Aleksandrov` (×1), `frozen-magnitude` (×1), `hardcoded`
  (×1) — pre-existing rows D181–D184 (T1's work, untouched by this commit) carry their own
  unrelated, pre-existing `vale` findings, left alone per the surgical-changes rule.
- rst structure re-verified: `rst2html` (this repo's pinned docutils, via the main checkout's venv)
  parses the touched list-table rows with zero structural errors (the only findings anywhere in the
  file are the pre-existing, whole-document `:doc:`/`literalinclude` Sphinx-role warnings vanilla
  docutils always raises); the rendered HTML contains all four new row anchors (`D185`–`D188`). One
  real defect caught and fixed in this pass: two of the new rows' first drafts broke an inline
  ``code`` span mid-path across a source line-wrap (`` ``src/babylon/engine/systems/\n
  survival.py:128`` ``), which RST's paragraph reflow would render with a spurious inserted space,
  corrupting the citation — reworded to keep each path whole on one source line.
- This record re-opens every `file:line` citation this pass via `Read`/`sed -n`/`cat -n`/`grep -n`/
  `sqlite3 .schema`, rather than carrying it over from the plan's own citations. All
  `bsl-language.rst` and `evaluator.rs` citations drifted (see §5, §6); Python-side citations
  (`survival.py`, `vitality.py`, `economy_basic.py`, `accumulation.py`) did not drift.
- **No register row asserts a τ value as this record's own decision.** D186 records DP-5's
  disposition as an already-ruled fact, cited to its source (#491, 2026-08-18T02:42:38Z) — the
  distinction the plan's original T2 task text (written before the DP sitting ran) could not have
  anticipated, and the one this record's provenance section (top of file) states explicitly.
