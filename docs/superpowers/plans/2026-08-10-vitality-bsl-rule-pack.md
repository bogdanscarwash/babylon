# Vitality as the first Material Base rule pack (Program 28 §7 criterion 6)

**Status:** implementation plan. Written against `origin/dev` at `2c28c446`
(PR #480 merged: R9 chapters C1–C13 plus `rust/crates/babylon-bsl`).

**Scope:** port `VitalitySystem` (Material Base position 1.0) from the frozen
Python engine to a BSL rule pack the Rust engine runs, conformance-vectored
from the live frozen surface.

**Verdict up front:** the system ports **in part**. Phases 1 and 3 (The Drain,
The Reaper) land as one BSL rule. Phase 2 (Grinding Attrition) **does not
land**, for two independent reasons — one mechanical (a missing language
construct), one doctrinal (a stipulated functional form that ADR175 puts
behind a Director derivation review). This port improvises around neither.
§6 records both precisely.

---

## 1. What the Python system does (the structure/ordering contract)

`src/babylon/engine/systems/vitality.py`, 256 lines, `partition = MATERIAL_BASE`,
`position = 1.0`, `creates_value = False`. One pass over
`graph.query_nodes(node_type=NodeType.SOCIAL_CLASS)`, in graph order. Per node:

| Step | Reads | Writes / emits |
|---|---|---|
| skip | `active` (default `True`), `population` (default `1`) | `continue` when inactive or `population <= 0` |
| **Phase 1 — The Drain** | `wealth` (default `0.0`), `subsistence_multiplier` (default `1.0`), define `economy.base_subsistence` | `wealth = max(0.0, wealth − (base_subsistence × population) × multiplier)` |
| **Phase 2 — Grinding Attrition** | re-reads the node; `wealth`, `population`, `inequality`, `s_bio`, `s_class` (all `required=True`), define `vitality.attrition_base_factor` | `population −= deaths`; emits `POPULATION_ATTRITION` when `deaths > 0` |
| **Phase 3 — The Reaper** | re-reads the node; `population`, `wealth` (post-drain), `s_bio`, `s_class` (off the **pre-phase** dict, `.get(…, 0.0)`), define `economy.death_threshold` | `active = False`, `population = 0`; emits `ENTITY_DEATH` |

Phase 2's rate comes from `formulas/vitality.py::calculate_mortality_rate`:

```
coverage_ratio = wealth_per_capita / subsistence_needs      # needs = s_bio + s_class
threshold      = 1.0 + inequality
if coverage_ratio >= threshold: return 0.0
deficit        = threshold - coverage_ratio
rate           = deficit * (attrition_base_factor + inequality)
return max(0.0, min(1.0, rate))                             # the clamp
deaths         = int(population * rate)                     # truncation toward zero
```

Phase 3's predicates:

```
consumption_needs  = s_bio + s_class
is_extinct         = current_population <= 0
is_starving        = current_population == 1 and wealth < consumption_needs
is_zombie_trapped  = wealth < death_threshold and current_population == 1
cause              = "extinction" | "wealth_threshold" | "starvation"   (that precedence)
```

**Ordering facts that belong to the contract:** the three phases run
sequentially *within one subject* (Phase 2 re-reads the node, so it sees Phase
1's wealth; Phase 3 re-reads, so it sees Phase 2's population); subjects stay
independent — nothing in the system reads another node, an edge, or a
graph-level measure. Vitality is **entirely self-scoped**, which is what makes
a one-rule port possible at all.

Defines the system reads: `economy.base_subsistence = 0.0005`,
`economy.death_threshold = 0.001`, `vitality.attrition_base_factor = 0.5`
(`src/babylon/data/defines.yaml:88,89,177`).

## 2. Rule decomposition

**One rule, not three.** §4.2 of `bsl-language.rst` states that rules within one
system position **observe the same pre-state**; a rule can never observe another
rule's effects at the same position. A three-rule decomposition would then have
to re-derive the post-drain wealth in each of the two downstream rules — the
"restate the same algebra in four places and eventually restate it differently
in one" hazard that R9 chapter C7 (`:expr` bindings) exists to remove. The R9
gap analysis reaches the same conclusion independently: *"Vitality (three
phases collapse into one rule and must re-derive the post-drain value
algebraically)"* (`reports/bsl-gap-analysis-2026-08-10.md:419`; survey row 1.0
says the same).

The pack is one rule file,
`rust/crates/babylon-tick/content/rules/vitality.bsl`:

```
(rule vitality/subsistence-and-death …)
```

| | |
|---|---|
| subject | `social-class`, which `tick.rs` derives from the `:field` `namespace` |
| guard | `(and (= active 1) (> population 0))` — the Python `continue`s |
| reads | `active`, `population`, `wealth`, `subsistence-multiplier`, `s-bio`, `s-class` as `:field`; `economy/base-subsistence`, `economy/death-threshold` as `:const` |
| intermediates | `cost`, `drained`, `consumption-needs` as `:expr` (C7) |
| writes | `wealth` always; `active` plus `population` under a `(guard …)` |
| emits | `ENTITY_DEATH` under the same guard |
| **not** written | `attrition-rate`, the `population` decrement, `POPULATION_ATTRITION` — see §6 |

The rule spells `max(0, wealth − cost)` out rather than reaching for a scalar
`min`/`max`: §3.10's rider slate row 5 declines that operator precisely so a
saturation stays legible in the source, and §3.3 frames a silent clamp as
forbidden quiet degradation.

**The landed spelling is `paid = (if (> wealth cost) cost wealth)` followed by
`drained = (- wealth paid)`** — what the class actually hands over, subsistence
or everything it has, whichever is less. The obvious alternative,
`(if (> (- wealth cost) 0) (- wealth cost) 0)`, is **not legal BSL**: §1.5
admits no bare non-integer literal, so its zero branch is an `Int` sitting
under a `Real` branch — two static types under one `if`, in a language that
declares no coercions (§3.1). Subtracting the payment keeps one type
throughout and lands on exactly `0.0` when a class loses everything. A reader
following an earlier draft of this section would have written the illegal
form; this paragraph is the correction.

Because Phase 2 does not land, `is_extinct` becomes unreachable in the ported
subset (the guard already excludes `population <= 0`, and only an attrition
decrement could drive a live population to zero mid-tick). The Reaper's two
remaining causes both require `population == 1`, and both set the same two
fields, so they collapse into **one** guarded block.

### Content that becomes scenario/manifest data

- The six node fields become `deffield` declarations in the scenario. All six
  take integer literals, which is the only lane slice 1's scenario loader
  stores (`scenario.rs::attribute_value`); nothing in the ported subset needs a
  fractional seed, so this port leaves the loader alone.
- `active` travels as an `int` 0/1 field. BSL has `Bool` (§3.1) but `deffield`
  has no `bool` type and `GraphSubstrate` attributes are `f64`; 0/1 is the
  honest rendering available today, and this plan records it rather than
  leaving a later reader to discover it.
- The two defines become `(defconst …)` rows in the scenario — see §3.

## 3. Engine machinery the port needs

Two changes, both inside the slice-1 driver scaffolding, neither touching BSL
grammar, the `<bind-src>` set, or the error codes.

**(a) A defines environment, so a rule can read `:const`.** `tick.rs`'s
`check_sources_servable` refuses `:const` by name today: *"slice 1 has no
defines environment; the coefficient registry is Phase-2 content"*. Vitality
reads two coefficients, and writing `0.0005` into a rule would break the
project's single-source-of-truth rule for coefficients (a define supplies the
scale; a rule supplies the shape). The minimal honest fix:

- `scenario.rs` gains `(defconst <qname> <literal>)`, exactly parallel to the
  `deffield` registry-in-miniature it already carries, producing a
  `HashMap<String, Value>`;
- `run_tick` takes that map and serves `BindSource::Const`, refusing loudly
  (and naming the qualified name) when a rule reads a coefficient the
  environment does not hold;
- `run_once` feeds the scenario's declared coefficients into both
  `BindingVocabulary::consts` (so `E-LOAD-010` still gates an unknown name at
  load) and the tick.

This is the same declared temporary as `deffield`: the Phase-2 successor is the
real `GameDefines`/`defines.yaml`-backed registry, and the scenario file cites
the `defines.yaml` line each value came from.

**(b) Nothing else.** This port builds no rule-pack sequencer. A pack is a set
of rules at one position observing one pre-state; with a single rule there is no
order to declare, so an ordering mechanism now would be machinery with no
caller — and the anchor-to-total-order resolver belongs explicitly to
`babylon-engine` Phase-3 work (`mod_anchors.rs` module doc). This port leaves
`run_once`'s signature and `TickReport` alone.

## 4. Conformance-vector strategy

Vectors come from the frozen Python `VitalitySystem` run in isolation against a
fixture that mirrors the `.bscn` scenario node for node, single process, one
`step()` call, with the real `ServiceContainer` defines. The repository carries
the script at
`rust/crates/babylon-tick/content/scenarios/vitality_conformance.py`, so anyone
can re-run the provenance, and the Rust test pins the values it prints.

**The fixture makes the un-ported phase contribute nothing.** Every subject
satisfies `int(population × attrition_rate) == 0` — either because coverage
clears the threshold (the rate is exactly `0.0`) or because
`population × rate < 1`. Python performs no population decrement and emits no
`POPULATION_ATTRITION`, so the ported subset's post-tick state should match the
*full* Python system **exactly**, not approximately. Any divergence is a real
defect rather than an artefact of the missing phase.

**That precondition is gate-enforced, not merely documented.** Asserting it
only inside the Python script would leave it unchecked, because nothing runs
that script — no mise task, no CI leg, no `pytest` collection — so a `.bscn` edit
nudging one subject into killing range would void every vector while the gates
stayed green, which is absence reading as success (III.11). The Rust test suite
recomputes the envelope from the committed scenario seeds, in the gate that
already runs, and a mutation check confirms the guard reds on a drifted seed. The rate formula it
uses lives **inside a test, marked a fixture guard rather than content**: no
rule reads it, nothing declares it, it writes nothing. A guard bounding the
fixture takes no position on what the blocked phase should eventually compute.
A guard inside the *rule* would, which is why §6.2 leaves the mechanic empty
and nothing in the shipped content mentions a threshold.

**Tolerance policy: none.** Every operation on both sides is an IEEE-754 basic
operation (`+ − × ÷` and comparison) on binary64, correctly rounded and
reproducible across implementations (§4.3; `bsl-language.rst:1842`). No
transcendental, no libm, no ambiguity about accumulation order — BSL's
`<arith>` is strictly binary (`E-PARSE-040`), so the source states the
association explicitly, and the transcription matches Python's
`(base × population) × multiplier` exactly. Scaled literals reach `Value::Real`
as `unscaled / 10^scale`, a correctly-rounded division that lands on the same
double as the matching Python decimal literal. The test asserts
exact `f64` equality. A tolerance here would hide precisely the transcription
error it would appear to absorb.

A second Python run, on a fixture where `deaths > 0`, goes into the PR body as
the **blocked vector** — the numbers the §6.1 rider would have to reproduce.
Nothing asserts it, because no code claims to produce it.

**Determinism:** the Vitality pair repeats the existing
`run_once_is_deterministic` shape — two loads, two ticks, one identical
post-state hash.

## 5. §5.4 defects this port repairs

Per ADR183 and `ai/bsl-architecture-standard.md` §5.4: the frozen engine is the
contract source for **structure and ordering**, not a correctness oracle.

**D-1 — Two absence policies for the same fields in the same tick.**
`_calculate_deaths` reads `wealth`/`population`/`inequality`/`s_bio`/`s_class`
with `required=True` (`vitality.py:227-231`, whose comment names the
silently-masked-missing-field gotcha it repairs). Phase 3, twelve lines later,
reads `s_bio` and `s_class` off the **pre-phase** dict with
`attrs.get("s_bio", 0.0)` (`:154-155`), and the skip block reads `active`,
`population`, `wealth` and `subsistence_multiplier` with `.get` defaults
(`:106,109,116,118`). A `social_class` missing `s_bio` raises in Phase 2 and
quietly reads `0.0` in Phase 3 — where `consumption_needs = 0` makes a starving
class un-starvable. **Repair:** every field a BSL rule reads is a required
`:field` binding, and a never-written field raises the substrate's loud error
(III.11 — absence is not zero). *Behavioural difference from the frozen
engine:* a node missing any of the six fields now aborts the tick instead of
proceeding on a default. That is the intended direction.

**D-2 — `if base_subsistence > 0` is a dead branch.** `vitality.py:117` guards
the whole drain on a define whose value is `0.0005`. Nothing a tick can observe
sets it to zero. **Repair:** the transcription drops the branch; the drain runs
unconditionally and the define supplies its scale. Recorded rather than
quietly dropped: a mod setting `base_subsistence: 0.0` gets a zero-scale drain,
whose observable outcome equals the skipped write, because the effect is
`set drained` and `drained == wealth` at that value.

**D-3 — `calculate_mortality_rate` cannot see `defines.yaml`.**
`formulas/vitality.py:13` binds `_DEFINES = GameDefines()` at import — the
**schema defaults**, not `GameDefines.load_default()`, which is the loader that
reads `src/babylon/data/defines.yaml`. `attrition_base_factor` then bakes into
a default argument that Python evaluates once at import, and `VitalitySystem`
never passes the parameter explicitly. Anyone editing
`defines.yaml: vitality.attrition_base_factor` changes nothing. This is a live
modding-contract defect in the frozen engine, found by reading it for this
port. **Repair:** not applicable to the ported subset, since §6.2 declines the
formula — but this plan records the defect, and the port's `:const` binding is
the structural answer: a coefficient reaches a rule only through the defines
environment, so no import-time capture can go stale.

**D-4 — `cause` is a string in an event payload.** §2.8 admits no string in a
payload, and §1.5 admits string literals only at `:material-basis` and vector
ids (`E-PARSE-010`). The discriminant would need a registered closed enum,
which is a vocabulary addition, and so spec-first work. **Repair:** the
`ENTITY_DEATH` payload carries `entity-id`, `wealth`, `consumption-needs`,
`s-bio`, `s-class` and drops `cause`. This plan names the gap instead of
omitting it silently; with `is_extinct` unreachable in the ported subset a
reader can recover the discriminant from the payload anyway
(`wealth < death_threshold` means `wealth_threshold`, else `starvation`).

**Not a defect, recorded so a later reader does not "fix" it:**
`max(0.0, wealth − cost)` destroys the part a class cannot pay rather than
carrying a debt. That is a modelling choice of the frozen engine and part of
the structure contract, so the rule transcribes it as written, spelled as an
explicit `if`.

## 6. What does NOT land, and exactly why

### 6.1 Mechanical: BSL has no `Real → Int` demotion

`deaths = int(population * attrition_rate)` (`vitality.py:253`) needs a floor.
`bsl-language.rst` §3.1 declares **no coercions**, and §3.3 promotes `Int` to
`Real` **one way only**, so the language holds no demotion path at all.
§3.10's rider slate row 2 (`floor` / `trunc`, "In cap? **No**") records this as
a *proposed* rider — a question for the Director, explicitly non-normative and
declaring nothing. The intrinsic cap holds at `{exp, log}` at most (R10 citing
ADR176 r21), so `floor` sits outside it and no one may declare it.

Consequences, stated exactly: nothing computes `deaths`; nothing writes the
population decrement; `POPULATION_ATTRITION` has no trigger. **What it would
take:** a Director ruling on rider-slate row 2, then §3.1/§3.2 gaining the
demotion with its rounding mode pinned, an `E-` code for a non-finite or
out-of-`i64` demotion, a §3.7 cost row, and CAS coverage. That is spec work
rather than implementation work, and this port does not improvise it.

### 6.2 Doctrinal: the mortality curve is a stipulated functional form

Even with a floor, nobody should transcribe the rate verbatim.

```
rate = (1.0 + inequality − coverage_ratio) × (attrition_base_factor + inequality)
```

is a piecewise-linear form with a **tuned knob** — `attrition_base_factor = 0.5`,
described in its own schema as *"Base multiplier in grinding attrition"*
(`config/defines/survival.py:88-92`), a feel-tier define with no written
`Aleksandrov` chain. Standard S-7 and the Director ruling of 2026-07-29
(ADR172 ruling 5): *no functional form may be imposed on a mechanic; curve
shapes must emerge from the algebraic operations.*

The construct is not even a new one. Vitality's own module header says what
it approximates: *"One agent = one demographic block. High inequality within a
block means you need MORE coverage to prevent deaths"* — that is **the mass of
the within-class wealth distribution that fails to clear subsistence** — which
is exactly ADR173's ruled formulation for `P(S|A)`
(`ai/bsl-architecture-standard.md` §3.2 fact 2): the measure of class members
whose wealth clears subsistence, with the curve read off the distribution
rather than stipulated. Grinding Attrition repeats that same construct, wearing a linear proxy with a steepness knob where the distribution
integral belongs.

ADR175 governs every non-survival site: the Python reference freezes as-is,
each site receives an **emergent re-derivation from material operations at its
Rust/BSL port**, and the Director reviews each derivation per family before it
lands (§3.2 fact 3; S-7's evidence column: *"each non-survival family
additionally requires its ADR175 per-family derivation review before
landing"*). Mortality has had no such review.

**This port transcribes no formula and invents no replacement.**
Writing the stipulated form into BSL would enshrine exactly what ADR172 ruling
5 retires; writing a *substitute* measure would be an ideological call nobody has reviewed, which Constitution IX.5 reserves to the Director. This is an
escalation, not a workaround.

**What it would take:** an ADR175 derivation review for the vitality/mortality
family, answering the question §3.2 asks of every site — *can this be
re-derived as a measure instead?* — plus (open, and shared with `P(S|A)`) the
canonical within-class wealth distribution, which audit Q3 records as
undecided. §6.1's rider is then still needed to turn a measure into whole
people.

### 6.3 Consequence for the delivered pack

On a world where the frozen engine's attrition would kill, the rule pack
under-kills. That is why §4 picks a fixture where the frozen engine kills
nobody — the delivered rule holds conformance exactly on the ground it claims,
and claims no ground it cannot hold. Nothing wires it into an always-on path:
`run_once` still takes its rule as an argument, and tests exercise the Vitality
pair. Nothing in the tree runs Vitality over an arbitrary world and reports a
population figure.

## 7. Steps and verification

1. Plan committed. → *verify:* file exists, `git log --oneline -1` moved.
2. Defines environment (`defconst` plus `:const` serving) and unit tests. →
   *verify:* `cargo test -p babylon-bsl` green; an unknown coefficient fails
   loudly at load and a missing one fails loudly at bind.
3. `vitality-conformance.bscn` and `vitality.bsl` content. → *verify:* the rule passes
   every load gate (`load_rule`); the tick runs.
4. Conformance script and Python run. → *verify:* single-process
   `uv run python …` prints post-tick state per subject.
5. Rust conformance test pinning those exact values, plus a determinism test. →
   *verify:* `cargo test -p babylon-tick` green, exact `f64` equality.
6. Gates. → *verify:* `mise run rust:check` green; `cargo clippy -p babylon-bsl
   -- -W clippy::pedantic` clean.
