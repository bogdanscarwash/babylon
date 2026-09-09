# Production BSL Surface Facts — Verified 2026-08-12

Scout pass for the Production (@3.0) port train (issue #565, Program 29 Wave A). Read-only
(`rg`/`Read` only, no cargo) against `dev` in the main checkout
(`/home/user/projects/game/babylon`) plus the worktree
(`/home/user/projects/game/wt-query-eval`, branch `feat/territory-port-bsl-rules`, PR #569).

**Provenance note, since it changed mid-session and the task's own briefing predates it:** at
scout-start `dev` was at `5b324bb4` (PR #568 merged, PR #569 "in CI") and the worktree carried
Territory's landed pack (`territory.bsl`, `territory-conformance.bscn`,
`territory_conformance.rs`, `territory_conformance.py`) that `dev` did not yet have. **Mid-session,
PR #569 merged** — `dev` is now at `b6ff7c09` ("Merge pull request #569 from
percy-raskova/feat/territory-port-bsl-rules") and `diff`-confirmed byte-identical to the worktree
for `territory.bsl`; the D-record register rows D123/D129/D130/D131 are now in the main checkout's
`docs/reference/bsl-language.rst` too. Every citation below is to the **main checkout**
(`/home/user/projects/game/babylon` @ `b6ff7c09`) unless marked `[worktree-only, pre-merge]`, which
did not happen for anything cited here — the merge closed the gap the task briefing anticipated.

Every fact below carries a `file:line` anchor as of this commit. Where a fact is a claim I
independently re-derived (not just re-cited from the phase-1 inventory), it says so — this dossier
finds three corrections to the phase-1 inventory's own adjudicated verdict, on top of the two
corrections the adjudication already made to itself. Read **§10 CORRECTIONS/FLAGS** first if you
have read the phase-1 inventory already; it is the highest-value section for planning purposes,
exactly as the Territory dossier's own equivalent section was.

**Executive summary (10 lines).** `ProductionSystem` (268 lines, tick @3.0) is two loops: one over
`SOCIAL_CLASS` producer workers computing `produced_value = labor_power × population × bio_ratio`
and routing it to the worker's own wealth or an employer's via a `WAGES` edge (Amin/Wallerstein
routing — RESERVED, described not touched), one over `TERRITORY` broadcasting
`extraction_intensity`. Post-PR-A, `field-of` over an enum-declared field is fully discharged
(D102) — but that fact turns out **not** to be what the port actually needs: the recommended
reformulation below keeps `role` reads on the rule's own subject side (always legal, D101) and
never inside a fold body at all. The phase-1 inventory's "UNVERIFIED-IN-PRACTICE" verdict on the
employer-wealth accumulation is **too pessimistic** — `dev` already proves the exact shape
(D103/D104's collect-then-apply, a `tick.rs` unit test literally named for it). The `la_production`
"BLOCKED — no graph-scope construct" verdict is **the wrong diagnosis** — the channel is keyed by
worker node id, so it is per-node data smuggled through graph scope, not a true aggregate; an
ordinary node field dissolves the blocker without touching the still-unserved carrier/`the`
machinery at all. In exchange, this dossier finds a **new** blocker the inventory's own "PORTABLE
NOW, reformulated as a fold" verdict for the extraction-intensity broadcast missed: the fold-body
compound-expression restriction (§3.4, uncoded, `rule_pipeline.rs`) refuses the role/active filter
a correct fold would need, and the flagship scenario's own `TENANCY` topology is not role-restricted
— a candidate fix is given, reusing the same per-node field the `la_production` fix already adds.

---

## 1. The frozen computation, phase by phase

Source: `src/babylon/engine/systems/production.py`, 268 lines, tick position 3.0
(`production.py:68`), read line by line. All file:line citations in this section are from my own
read of the file, cross-checked against (not merely copied from) the Phase-1 inventory's
`reports/port-inventories/production-port-phase1-inventory-2026-08-12.md` §2.

### Computation 1 — labor-power annualization (`production.py:111-129`)

```python
annual_labor_power = services.defines.economy.base_labor_power   # :112
weeks_per_year = services.defines.timescale.weeks_per_year       # :113
base_labor_power = annual_labor_power / weeks_per_year           # :114
tick: int = context.tick                                          # :123
```

One division, both operands runtime-resolved defines, no bare literal. `economy.base_labor_power`
— default `1.0`, `[0.0, ∞)`, `src/babylon/data/defines.yaml:73` (verified: grepped directly,
`base_labor_power: 1.0  # Base value produced per tick by worker with full biocapacity (>= 0.0)`).
`timescale.weeks_per_year` — default `52`, `[1, ∞)` int, `defines.yaml:374` (verified directly:
`weeks_per_year: 52  # Engineering: physical constant...`).

### Computation 2 — per-worker production (`production.py:131-204`, the core loop)

Gates, in order (`:135-146`): `active` defaults `True` (`attrs.get("active", True)`); `role` must be
in `_PRODUCER_ROLES = {PERIPHERY_PROLETARIAT, LABOR_ARISTOCRACY}` (`:46-52`, a 2-of-8 subset of
`SocialRole`); a `TENANCY` edge to a territory must exist (`_find_tenancy_target`, `:212-225`, a
linear scan of `graph.query_edges(edge_type=EdgeType.TENANCY)` returning the **first** match by
`edge.source_id == worker_id`).

Formula (`:151-175`):
```python
bio_ratio = 0.0 if max_biocapacity <= 0 else biocapacity / max_biocapacity   # :155
population = attrs.get("population", 1)                                      # :158
# effective_labor_power: base_labor_power, UNLESS overridden by the dead tensor branch (below)
produced_value = (effective_labor_power * population) * bio_ratio           # :175
```

**Verified dead branch** (`:160-172`): a `TensorRegistry` lookup keyed by `territory_attrs.get(
"fips_code")` — `Territory`'s real field is `county_fips`
(`src/babylon/models/entities/territory.py:81-91`, independently re-read: `county_fips: str | None
= Field(default=None, min_length=5, max_length=5, ...)`, no `fips_code` field exists anywhere on
the model), so `fips_code` is `None` on every live territory node and the branch never fires. See
§6 for the full defect writeup.

Routing (`:179-198`):
```python
if role in _DIRECT_PRODUCER_ROLES:            # PERIPHERY_PROLETARIAT
    graph.update_node(node.id, wealth=current_wealth + produced_value)      # :181, self-write
elif role in _EMPLOYED_PRODUCER_ROLES:         # LABOR_ARISTOCRACY
    employer_id = self._find_employer(graph, node.id)   # :227-244, first WAGES edge, target=worker
    if employer_id is not None:
        graph.update_node(employer_id, wealth=employer_wealth + produced_value)  # :192, cross-node
        la_production[node.id] = produced_value                                  # :194
    else:
        graph.update_node(node.id, wealth=current_wealth + produced_value)       # :198, fallback
territory_production[territory_id] += produced_value  # :200-204, whenever produced_value > 0
```

Independently re-verified: `SocialClass.wealth` defaults `10.0` (`social_class.py:308-311`,
directly re-read: `wealth: Currency = Field(default=10.0, ...)`), `active` defaults `True`
(`:380-383`), `population` defaults `1`, `ge=0` (`:406-410`). `Territory.biocapacity`/
`.max_biocapacity` both default `100.0` (`territory.py:155-164`, directly re-read) — note this
means production.py's own two `.get(..., default)` fallbacks (`1.0` at `:152`, `100.0` at `:265`,
below) are **both** wrong relative to the Pydantic model's real default, and disagree with each
other — independently re-confirmed, not just copied from the inventory. Under BSL's "no defaults"
law (§1.5, `scenario.rs:56-58`) neither fallback is ever reachable in a correctly-seeded fixture,
so this is inert for the port, exactly as the inventory found — flagged here only because I
re-derived it myself rather than trusting the citation.

### Computation 3 — LA-production ledger publish (`production.py:207`)

```python
graph.set_graph_attr("la_production", la_production)   # dict[worker_id, float]
```

Read back **only** by `ImperialRentSystem` (`economic.py:438,453`, per the inventory, not
independently re-read this session — narrow enough surface I trust the citation): `la_production =
graph.get_graph_attr("la_production", {}); productivity_value = la_production.get(edge.target_id,
0.0)`. See §5 — this is the channel this dossier's biggest correction concerns.

### Computation 4 — extraction-intensity broadcast (`_update_extraction_intensities`,
`production.py:246-268`)

```python
total_production = territory_production.get(node.id, 0.0)
max_biocapacity = attrs.get("max_biocapacity", 100.0)
intensity = min(1.0, total_production / max_biocapacity) if max_biocapacity > 0 else 0.0  # :267
graph.update_node(node.id, extraction_intensity=intensity)
```

Zero-guarded division, upper-only clamp (no lower clamp needed — `total_production` can never be
negative, only ever accumulated from `produced_value > 0` values, `:201`). `Territory.
extraction_intensity` is `float`, `ge=0.0, le=1.0` (`territory.py:171-176`, directly re-read).

**Events emitted: zero** (grep-confirmed, no `EventType`/`event_bus`/`.publish(` reference
anywhere in the file). **No `int(...)` casts anywhere** — `floor` is not needed for this system's
own arithmetic, a favorable contrast with Territory. **No libm calls** (`exp`/`log`/`pow` all
absent).

---

## 2. The role/defenum question post-PR-A — VERIFIED DISCHARGED (but not the fact this port needs)

The phase-1 inventory's own Adjudication (§ "Adjudication (2026-08-12)", correction 2) found that
`field-of` over an `:enum-type`-declared field was refused at load (D102, then-unconditional), and
concluded both of the inventory's own reformulations (the extraction fold, the employer fold) were
therefore blocked unless `role` used an int-ordinal encoding — contradicting the same report's own
recommendation to use the ADR195/196 `defenum` route.

**That deferral is discharged on current `dev`.** Independently verified at the code, not just
cited:

- `typecheck.rs:840-846`: *"D102 discharge (Task 1, P27 territory-port train): field-of over an
  enum-declared field now TYPECHECKS AS THE ENUM, not Real, and not refused.
  `check_no_field_of_on_enum_field` (the unconditional D102 deferral gate) is deleted rather than
  narrowed."*
- `typecheck.rs:854-868` (test `field_of_over_an_enum_declared_field_typechecks_as_enum`): proves
  `(field-of self organization/kind)` classifies as `ScoreClass::Enum`, not `Unknown`/`Real`.
- `evaluator.rs:2486-2522` (test `field_of_over_an_enum_field_compares_correctly_per_node`): proves
  `(= (field-of self organization/kind) OrgKind/STATE_APPARATUS)` evaluates `Bool` correctly
  per-node against a real `MemoryGraph`.
- What still stands, unaffected: score-position refusal (D46, `E-TYPE-016` — an enum-classed
  `select-max`/`select-min` score still refuses, `typecheck.rs:887-899`) and arithmetic refusal
  (D101/D118, `E-EVAL-042` — `apply_arith`'s unconditional fallthrough still refuses `Value::Enum`,
  `evaluator.rs:2558-2580`).
- `docs/reference/bsl-language.rst:5694-5735` (D102's own register row) states the discharge and
  both surviving refusals in the normative text.

**But this port does not actually need the discharge.** Re-reading the adjudication's own concern:
both of its flagged reformulations read `role` **off a non-subject node inside a fold/query body**
(a neighbor's role, via `field-of it social-class/role`). My recommended reformulation (§4, §5)
never does this — `role` gating for Production's own rule(s) stays on the **subject side**, via an
ordinary `:field` binding in the rule's own `when` guard (`(binding role :field social-class/role)`,
`(when (or (= role SocialRole/PERIPHERY_PROLETARIAT) (= role SocialRole/LABOR_ARISTOCRACY)))`) —
which was **already legal before D102 landed**, since a subject's own enum field has always rendered
`Value::Enum` correctly (D101; the landed `organization.bsl:29` precedent,
`(when (and (= active 1) (= kind OrgKind/STATE_APPARATUS)))`, predates D102's discharge entirely).
The `_find_employer`/`_find_tenancy_target` topology walks are resolved by **edge type**, not by
filtering neighbors on their `role` — so no neighbor-side enum read is ever needed for Production's
port at all, under the reformulation this dossier recommends.

**Report this fact to the plan author precisely**: D102's discharge is real and general (a plan
author who prefers a neighbor-role-filtering design has it available), but Production's own
cleanest transcription sidesteps needing it. Do not let a plan draft cite "D102 unblocks role" as
the reason the port works — the reason is the topology-based (edge-type) routing design in §4/§5,
D102 is orthogonal. A **different** law blocks role-filtering **inside a fold body** specifically —
see §5, which is NOT D102 and is not discharged.

`SocialRole` itself, for the `defenum` declaration (order is hash-bearing, ADR195 — a "transcribe
verbatim, do not reorder" fact, independently re-read at `src/babylon/models/enums/social.py:34-41`):
```
CORE_BOURGEOISIE, PERIPHERY_PROLETARIAT, LABOR_ARISTOCRACY, PETTY_BOURGEOISIE,
LUMPENPROLETARIAT, COMPRADOR_BOURGEOISIE, INTERNAL_PROLETARIAT, CARCERAL_ENFORCER
```
8 members, matching the inventory's own count. Only 2 of the 8 (`PERIPHERY_PROLETARIAT`,
`LABOR_ARISTOCRACY`) are ever tested against in this system.

---

## 3. Anti-defect: `NodeType`/`EdgeType` and system registration

`NodeType.SOCIAL_CLASS`/`TERRITORY`, `EdgeType.TENANCY`/`WAGES` — confirmed by the inventory's own
file map (`models/enums/topology.py:61-62,104-106`), not independently re-read this session (narrow
enum-member-name lookup, low re-verification value). `defvocabulary` is an unordered set (unlike
`defenum`), so these don't carry an ADR195 ordering obligation the way `SocialRole` does.

**`"production"` is NOT a registered system on either `dev` or the worktree — a genuinely new
registration, unlike Territory's pre-existing placeholder.** Grepped directly, both locations:
```
rg -n '"production"' rust/crates/babylon-tick/src/lib.rs   # zero hits, both trees
```
The full registered set (`lib.rs:174-203`, identical on both trees post-merge): `economics`,
`vitality`, `consciousness`, `lifecycle`, `dispossession`, `metabolism`, `territory`,
`organization`. Territory's own port train benefited from a pre-existing placeholder entry the
query-evaluation train added defensively; Production has no such gift. The port plan needs one new
string literal, `"production".to_owned()`, added to the `HashSet::from([...])` at `lib.rs:174-203`
— mechanically trivial (the same one-line change Territory's own dossier described as the cost of
registering a genuinely new system), but a real task-list line item this port cannot skip, where
Territory's could cite "zero-cost, already present."

---

## 4. The employer/wealth accumulation — CORRECTED verdict (not "unverified-in-practice")

The phase-1 inventory rated this **"PORTABLE WITH D-RECORD (nontrivial reformulation) —
UNVERIFIED IN PRACTICE"**, reasoning that the frozen system's push-style sequential
read-modify-write (two-or-more LA workers sharing one employer) does not match `for-each`'s
shared-pre-state semantics, and that the natural BSL fix — a pull-side grouped `fold sum` per
employer — is "not exercised by any landed pack for a class→tenancy-territory→employer three-hop
shape."

**This undersells what `dev` already proves.** D103/D104 (resolved rows,
`bsl-language.rst:5736-5775`) establish that `update-node`'s `add`/`sub`/`scale` participate in a
two-pass collect-then-apply split: Pass 1 collects every subject's effects as `PendingWrite`s
against one shared, unmutated pre-tick borrow (subject order); Pass 2 applies them **in the order
collected**, with `add`/`sub`/`scale` reading the **target's current value at apply time, not
collect time** — "which is what lets several subjects each contribute to one shared carrier
without losing any contribution" (`tick.rs:508-510`).

This is not a paper guarantee — it is a pinned, passing unit test that is **the exact shape
Production needs**: `accumulation_into_a_shared_target_reduces_in_subject_order_and_keeps_every_
contribution` (`tick.rs:994-1076`). Three `TERRITORY` subjects each hold an `ADJACENCY` edge to one
shared `ORGANIZATION` carrier node; the rule is:
```scheme
(rule geography/pool-contribution
  :material-basis "each territory contributes its share to a shared regional pool"
  :fuel 256
  (bindings (binding contribution :field territory/contribution))
  (effects
    (update-node
      (select-max (neighbors self EdgeType/ADJACENCY :out NodeType/ORGANIZATION) 1)
      organization/pool
      (add contribution))))
```
(`tick.rs:1031-1040`). All three fire; the pool accumulates all three contributions in subject
order (proven with `1.0 + 1.0e16 + -1.0e16 == 0.0`, an order-dependent-cancellation vector that
would fail silently if any contribution were lost, `tick.rs:1058-1075`). `select-max(…, 1)` — a
**bare literal constant** as the score — is legal (D46: "the score expression must be a comparable
scalar; kind unconstrained") and picks the single candidate deterministically via D45's ascending
element-id tiebreak when (as here) there is exactly one.

**This is Production's employer-lookup shape verbatim, one topology hop shallower than the
inventory assumed a fold was needed for** (WAGES is a direct edge from employer to worker; no
territory hop is involved in the employer lookup at all — the inventory's own "three-hop" framing
conflated the *tenancy* lookup, needed to compute `produced_value`, with the *employer* lookup,
needed to route it; they are two independent single-hop queries against two different edge types,
not one chained three-hop walk). Direction: `EdgeType.WAGES` runs employer(source)→worker(target)
(`production.py:241`, `edge.target_id == worker_id`); from the worker's own `self` scope this is
`:in` (self is the target) — confirmed against Territory's own D123 register row, which pins `:out`
= self is the edge's source (`bsl-language.rst:6175-6189`, the sink-walk-vs-spillover-walk
contrast) — so `:in` is the complementary reading, self is the target, by construction of the two
being exhaustive alternatives over one edge.

Candidate transcription (illustrative — the plan author's exact field/const names may differ):
```scheme
(rule production/p1-routing
  :material-basis "..."
  :fuel <N>
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding wealth :field social-class/wealth)
    (binding weekly-labor-power :expr (/ annual-labor-power weeks-per-year)))
  (when (or (= role SocialRole/PERIPHERY_PROLETARIAT) (= role SocialRole/LABOR_ARISTOCRACY)))
  (effects
    ; direct producers: self-write, hash-neutral (add 0) when inactive — D127-class idiom, below
    (update-node self social-class/wealth
      (add (if (= active 1) <produced-value-expr> (- 0 0c))))
    ; employed producers: route to the computed employer via :in on WAGES, same accumulate-into-
    ; shared-target idiom as pool-contribution above — needs a rule split or a role-gated
    ; second update-node against a DIFFERENT ref per role; see plan-author's own resolution of the
    ; "one ref or the other, never both" shape, which this dossier does not adjudicate.
    ))
```
**Hash-neutral inactive-worker write.** Using `(add (if (= active 1) produced-value (- 0 0c)))`
rather than gating the whole effect on `active` reuses the SAME idiom Territory's own D127 register
row names — "hash-neutral no-op writes where the frozen engine skips the write entirely"
(`bsl-language.rst:6235-6256`, worktree/main both, post-merge) — an `(add 0)` that changes nothing
observable but structurally fires every tick, matching the frozen engine's *intent* (dead workers
produce nothing) without needing a construct finer than the rule's own `when` guard to skip a
single effect conditionally.

**Honest caveat, not resolved here (D-record candidate, same class as Territory's own D124):** the
frozen `_find_employer`/`_find_tenancy_target` return the **first match in `query_edges` iteration
order** (`query_mixin.py:88`, `self._graph.edges(data=True)` — insertion/storage order, S-19's own
"insertion-ordered adjacency"), not sorted by node id. On the flagship `imperial_circuit` scenario
(`_legacy.py:255-500`, independently re-read) each producer class has **exactly one** `TENANCY`
edge and the `WAGES` edge is likewise singular (`LABOR_ARISTOCRACY_ID` has exactly one incoming
`WAGES` edge, `CORE_BOURGEOISIE_ID` → `LABOR_ARISTOCRACY_ID`, `:430`), so the tiebreak never
actually decides between candidates there — moot for the primary conformance vector, matching
Territory's own D124 finding for its own tiebreak divergence. **But `create_us_scenario`'s
`_assign_tenancy_edges` helper** (`_legacy.py:1057-1108`, independently re-read) assigns **multiple**
`TENANCY` edges per class node (percentile zones of many territories per class), where the
insertion-order-first-match-vs-D45-ascending-id-tiebreak divergence would be live — `create_us_
scenario` is **not** in `tools/regression_scenarios.py`'s `SCENARIOS` dict (grepped, zero hits), so
this is out of scope for the qa:regression-covered conformance vector, but worth a D-record note
(same "resolved by not resolving it, both stand independently correct" shape D124 uses) if the plan
author's fixture ever grows a multi-tenancy-edge case.

---

## 5. The extraction-intensity broadcast — a NEW blocker the inventory's own "PORTABLE NOW" verdict missed

The phase-1 inventory's blocker table (§6, last row) rated this **"PORTABLE NOW, reformulated as a
fold"** — a per-territory `fold sum` over `TENANCY`-incident producer nodes' `produced_value`. The
adjudication's correction 2 narrowed this to "PORTABLE ONLY UNDER AN INT-ORDINAL `role` ENCODING"
because the fold body would need to read a neighbor's `role` to filter it, and D102 (then
unconditional) refused that. **Even under D102's now-discharged state (§2), this row is still not
portable as a plain fold** — for a different, independent reason neither the inventory nor its
adjudication caught.

**The blocker: a fold body may not be a compound (conditional/arithmetic) expression at all —
D102's discharge does not touch this.** `rule_pipeline.rs::field_ref_for` (`:639-692`, read in
full) reduces a fold body to a declared field's kind through exactly **three** shapes — a bare
`<qname>`, a `(field-of <expr> <qname>)` accessor, or a nested `fold` — and returns `None` for
anything else, including an `if`. The caller turns `None` into a loud, **uncoded** rejection:
```rust
fn compound_fold_error() -> TypeError {
    TypeError {
        code: None,
        message: "fold body/weight kind-propagation over compound \
                  expressions is not implemented in Phase 1 — rejected \
                  loudly rather than passed unchecked (III.11); use a \
                  field reference, or wait for the Phase-2 checker"
            .to_owned(),
    }
}
```
(`rule_pipeline.rs:770-778`). This is the SAME restriction Territory's own p3-spillover header
independently documents and works around (`territory.bsl:141-156`: *"a compound body is rejected
LOUDLY at load as unverifiable... the fold body here is therefore the bare accessor... the `* rate`
scaling moves OUTSIDE the fold"*) — but Territory's workaround (move arithmetic outside the fold)
does not help here, because what needs to move outside is a **filter** (which elements to include
at all), not a **scaling factor** (applied uniformly to every included element). There is no
`(fold sum <query> <body> :when <predicate>)` form, and `:weight` goes through the identical
`field_ref_for` restriction (confirmed at the same call site, `:730-737`) so it cannot smuggle a
computed 0/1 eligibility flag in either.

**And the filter is not optional — the flagship scenario's own `TENANCY` topology proves it.**
`_assign_tenancy_edges` (`_legacy.py:1057-1108`, independently re-read, cited already in §4) assigns
`TENANCY` edges to **all four** class roles by rent-percentile zone — `CORE_BOURGEOISIE_ID`
(bourgeois zone), `LABOR_ARISTOCRACY_ID`, `COMPRADOR_ID`, `PERIPHERY_WORKER_ID` — not only the two
producer roles. Even on the simpler flagship `imperial_circuit` scenario, `TENANCY` is a general
"worker occupies this land" edge, not a producer-only edge, by construction
(`production.py:143-146`'s own gate exists precisely because `TENANCY` alone is not a sufficient
filter in the frozen model either). A naive unfiltered `(fold sum (neighbors self EdgeType/TENANCY
:in NodeType/SOCIAL_CLASS) (field-of it social-class/population))` would silently include
bourgeoisie/comprador population in the extraction-intensity denominator wherever their own
`TENANCY` edges exist — a real correctness defect relative to the frozen system, not a cosmetic
gap.

**Candidate resolution (same field this dossier's §6 already needs for `la_production` — one field
serves both).** Instead of folding an unfiltered neighbor population and trying to filter inside
the fold, push the **already-filtered, already-computed** `produced_value` onto the **producing**
worker's own node, using a new field — call it `social-class/production-value` (`int extensive`,
same money-workaround class as `wealth`, no Currency storage needed, §7 below covers the type
choice) — written via `(set ...)`, not `(add ...)`, inside Rule A (§4), which **already** only
fires for `role ∈ {PERIPHERY_PROLETARIAT, LABOR_ARISTOCRACY}` (the `when` guard is the filter,
computed once per subject, no fold involved):
```scheme
(update-node self social-class/production-value
  (set (if (= active 1) <produced-value-expr> (- 0 0c))))
```
Because this is `set`, not `add`, it self-resets every tick the rule fires — no cross-tick
accumulator, no reset-rule needed. For non-producer-role classes (bourgeoisie, comprador), Rule A's
`when` guard never fires at all, so their seeded `production-value` (0.0, per the "no defaults" law
— every `SOCIAL_CLASS` node needs this field seeded regardless of role) is **never touched**,
correctly staying at 0 forever. A second rule, subject `TERRITORY`, then folds a **bare accessor**
— fully compliant with `field_ref_for`:
```scheme
(rule production/p2-extraction-intensity
  :material-basis "..."
  :fuel <N>
  (bindings
    (binding max-biocapacity :field territory/max-biocapacity)
    (binding total :expr (fold sum (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)
                                (field-of it social-class/production-value)))
    (binding ratio :expr (if (> max-biocapacity 0) (/ total max-biocapacity) (- 0 0c)))
    (binding clamped :expr (if (< ratio 1) ratio (- 1 0c))))
  (when #t)
  (effects
    (update-node self territory/extraction-intensity (set clamped))))
```
`territory/heat` needed the same "declare EXTENSIVE, not the naive-reading kind" treatment for its
own `fold sum` (D131); `production-value` is money-like and already needs the extensive-workaround
class for the same reason `wealth` does, so this is not a new kind of forcing, just the same one
recurring.

**Report this to the plan author as this dossier's single most important correction.** It is the
opposite direction of the adjudication's own corrections (which narrowed a "portable" verdict) —
this narrows it further, past where the adjudication stopped, for a reason the adjudication's own
close reading of D102 did not surface because the blocker is not D102 at all.

---

## 6. The `la_production` graph-scope channel — CORRECTED verdict (not actually BLOCKED)

The phase-1 inventory rated this **"BLOCKED — named lane: graph-level scratch-state storage (no
`GraphSubstrate` construct)"**, and its adjudication (correction 5) reinforced it further: the R9
chapter-C3 carrier-`NodeType` pattern the inventory named as the on-paper fix is **itself** unserved
— `the` is in `UNSERVED_EXPRESSION_HEADS` under Slice 2 (`evaluator.rs:504-506`), so "BLOCKED
stands, reinforced."

**Independently re-derived this session, reading `ai/bsl-architecture-standard.md` §6.2 (the
carrier-node idiom, ADR198 R6, blessed 2026-08-12 — read in full, `:724-1050`) against
`production.py:207`'s actual read/write shape:** the carrier pattern exists to give **one number
the whole graph agrees on** a home — `NodeType/POLITY` carrying `polity/imperial-rent-pool`
(§6.2's own worked example) is a single scalar, one node, `:ceiling 1`. `la_production` is not
that shape. It is a `dict[str, float]` **keyed by worker node id**
(`production.py:129,194`: `la_production[node.id] = produced_value`), read back **only** by
`ImperialRentSystem` via `la_production.get(edge.target_id, 0.0)` — i.e. every consumer read is
already keyed by a specific node. This is per-node data being routed through graph scope as a
matter of Python convenience (a `dict` is the easiest way to pass N values from one system to
another in one Python call), not a genuine graph-level aggregate — the exact "negative — the easy
case" pattern §6.2 itself warns against minting a carrier for: *"three unrelated systems each
needed 'somewhere graph-scope to put a number'"* (`:787-797`), except here it is not even really
graph-scope, it is node-scope wearing a graph-scope costume.

**The fix: an ordinary `deffield` on the producing node, no carrier, no `the`, fully Slice-1
today.** Declare `social-class/production-value` (§5 above already needs this field for the
extraction-intensity fix — **one field serves both consumers**, since `ImperialRentSystem`'s own
WAGES-edge-keyed lookup would naturally never see a direct producer's contribution anyway — no
`WAGES` edge exists for `PERIPHERY_PROLETARIAT` workers, so writing the field for **all** producer
roles, not just the employed ones, is safe: it widens the write, not the read, and the read stays
exactly as narrow as the frozen `la_production.get(edge.target_id, ...)` call already is, since
only LA workers have an incoming `WAGES` edge for that call to find). Production's own port never
needs to *read* this field back (that is `ImperialRentSystem`'s future port's job, out of this
dossier's scope) — Production's obligation is only to *write* it correctly, which is a plain
`update-node self <field> (set ...)`, a construct every landed pack already uses.

**This dissolves the BLOCKED verdict entirely, without needing the carrier pattern, `the`, or
Slice 2 at all.** Worth stating plainly since it reverses the direction the adjudication moved in
(which reinforced BLOCKED): the inventory and its adjudication were not wrong about the carrier
pattern being unserved — they were answering the wrong question. `la_production` was never a
graph-scope-aggregate problem to begin with.

---

## 7. The `fips_code`/`county_fips` defect — reverified, transcribe-verbatim shape

Independently re-verified, not just cited:
- `production.py:164`: `fips_code = territory_attrs.get("fips_code")` — always `None`.
- `Territory` declares `county_fips`, never `fips_code` (`territory.py:81-91`, directly re-read;
  grepped `fips` across the model file, only hits at the `county_fips` declaration and its own
  docstring).
- `WorldState.to_graph()` stamps territory nodes via `territory.model_dump()` (per the inventory,
  `models/world_state.py:746` — not independently re-read, low-risk citation, a single indirection
  the inventory's own §2 already pinned precisely).
- `resolve_county_identity`'s docstring, directly re-read (`graph_bridge.py:44-48`): *"The county
  identity of a territory lives in its `county_fips` attribute and nowhere else."*

**D-record draft, transcribe-verbatim shape (matching Dispossession's D-1/D-2 "provably uniform"
class and Territory's own D129 "provably dead" class):**

| # | Section | Ruling |
|---|---|---|
| `DNNN` | §2.9/§3.5 | **Production port train.** The frozen `effective_labor_power` tensor-registry branch (`production.py:160-172`) is provably unreachable on every scenario in the estate — `fips_code` is read but `Territory`'s real field is `county_fips` (`territory.py:81-91`); confirmed dead even on `single_county`, the one scenario purpose-built to hydrate a real `TensorRegistry` for it (`engine/scenarios/single_county.py:116`, per the phase-1 inventory §2/§5, not independently re-read this session — a fixture-only claim, low re-verification value). **Omitted from the pack entirely** — no BSL construct exists for an external keyed-cache lookup (`TensorRegistry.get(fips, year)`) in any case, so there is nothing to transcribe even as inert content. `effective_labor_power` is always the Computation-1 fallback value. |

This matches the inventory's own "NOT-A-PACK — verified dead, D-record the omission" verdict
(§6 of the phase-1 inventory) — no correction owed here, only independent re-verification.

---

## 8. RESERVED-LINE inventory — Amin/Wallerstein producer-role routing

Every touchpoint, described precisely, DO-NOT-TOUCH per Director ruling (Constitution §IX.5;
`ai/national-question-ruled-2026-07-28.md` per project memory) — this is the model the Territory
dossier's own genre asks other port dossiers to follow, and the phase-1 inventory's §6 note already
does this correctly; restated here in BSL-surface terms since that is this dossier's job.

- **The role partition itself**: `_DIRECT_PRODUCER_ROLES = {PERIPHERY_PROLETARIAT}`,
  `_EMPLOYED_PRODUCER_ROLES = {LABOR_ARISTOCRACY}` (`production.py:46-52`). In BSL surface terms
  this is **the exact shape of the rule's `when` guard and its effects' target selection** — which
  role gets a self-write, which gets a computed-employer-ref write. A plan that changes which roles
  route which way, or adds a third routing class, is touching the reserved line and escalates to
  the Director (matching the inventory's own §6 RESERVED-LINE note) — not an ordinary port-as-is
  transcription call.
- **The `WAGES`-edge employer lookup** (`_find_employer`, `production.py:227-244`) is the concrete
  mechanism of "the LA works for the Core Bourgeoisie" — the BSL surface is the `(neighbors self
  EdgeType/WAGES :in NodeType/SOCIAL_CLASS)` query in §4's candidate rule. Transcribe the query
  faithfully; do not add, remove, or reinterpret what counts as "employer."
- **Non-doctrine-tree, non-outcome-definition**: confirmed independently — grepped
  `production.py` for any `DoctrineTag`/`PracticeVariable`/outcome-enum reference, zero hits. The
  coefficients (`base_labor_power`, `weeks_per_year`) are engineering constants, not ideologically
  authored values, matching the inventory's own finding.

Nothing in this dossier proposes changing any of the above; every candidate BSL snippet in §4/§5
preserves the routing structure exactly as the frozen system encodes it.

---

## 9. The D116 multi-rule-pack analysis — concrete answer, corrected direction from the adjudication

The task brief and the adjudication's correction 3 both flag this as an owed row. Concrete answer,
given the reformulation this dossier recommends (§4, §5) — **note this reverses the adjudication's
own conclusion**, because the adjudication was analyzing the *original* inventory's fold-based
design, which (before this dossier's corrections) happened not to depend on cross-rule writes; the
corrected design in §5 explicitly does.

**Production cannot be one rule, structurally, independent of any D116 question.** BSL derives a
rule's subject type from its own `:field` bindings' shared namespace (`tick.rs:159-182`, cited by
the carrier-pattern doc, independently cross-checked against Territory's own multi-rule pack, which
never mixes subject types either). Production genuinely needs two subject types — `SOCIAL_CLASS`
(producer routing) and `TERRITORY` (extraction-intensity) — so a minimum of **two rules** is
required regardless of any pre-state question, the same reason Territory's own p1-p3 (subject
`TERRITORY`) and p4-penal-suppression (subject `TERRITORY`, `for-each`-reaching into
`SOCIAL_CLASS`) split the way they do.

**Given two rules at the same `production` anchor position, D103 (within-rule) does not apply
across them — D116 (cross-rule) governs, and this design explicitly relies on it, the same way
Territory's own p1→p2→p3→p4 chain does.** `production/p1-routing` writes `social-class/wealth` and
`social-class/production-value`; `production/p2-extraction-intensity` (byte-ordered after p1) reads
`social-class/production-value` via a fold over `p1`'s producer subjects. Under D116
(`bsl-language.rst:5931-5957`, `babylon-tick`'s `run_once_into` runs each rule in a content set to
completion before the next starts, against the same mutable graph — *"a second rule at the same
anchor position observes the FIRST rule's already-applied writes from this tick"*), `p2` correctly
sees `p1`'s writes **because it runs after `p1` in byte order and the pack relies on that**, not
despite it. This is precisely Territory's own D-record #1 pattern (`territory.bsl:14-25`: *"this
pack RELIES on that divergence rather than fighting it"*) — Production's port should carry an
identical D-record naming the same dependency, citing D116 by row number, rather than treating the
cross-rule visibility as an accident.

**This is a genuine, load-bearing correction to the adjudication's own point 3**, which argued
Production's (then fold-based, unfiltered) design was "safe on the substance" because it "recomputes
`produced_value` from pre-tick `biocapacity`/`population` rather than reading Computation 2's
`wealth` writes." That was true of the design the adjudication was reading. It is **not** true of
the corrected design in §5, which deliberately reads `p1`'s own writes — the plan author needs to
choose byte-ordered rule names (`production/p1-...` < `production/p2-...`) and record the
dependency explicitly, exactly as Territory did, rather than inherit the adjudication's now-stale
"safe because independent" framing.

---

## 10. Scenario/conformance surface

**No canonical `.bscn` scenario exists for Production today** — `rg -l "production" rust/crates/
babylon-tick/content/scenarios/*.bscn` (run this session) matches four files, all incidental
prose ("production-default per-node regeneration_rate", "production defaults",
"always resolves to \[...\] in production" — `metabolism-conformance.bscn:5,30`,
`lifecycle-conformance.bscn:25`), never a `production/*` rule-pack reference. Territory faced the identical
gap and built its own hand-fixture (`territory-conformance.bscn`) plus a frozen-mirror Python
companion script (`territory_conformance.py`) rather than reusing any canonical scenario — the
imperial_circuit/single_county/two_node scenarios are all Python-only, never exported to `.bscn`.
Production's port needs the same two-artifact ceremony:

**A hand-built `.bscn` fixture needs, at minimum** (node types, edges, fields — derived from §1's
computation catalog and §4/§5's reformulation):
- `(defvocabulary NodeType (SOCIAL_CLASS TERRITORY))`
- `(defvocabulary EdgeType (TENANCY WAGES))`
- `(defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))` — full 8-member order, per §2.
- `(deffield social-class/role enum SocialRole)`
- `(deffield social-class/active int extensive)` — bool→int 0/1 workaround, same as Territory's
  `under-eviction`/organization's `active` precedent (`vitality-conformance.bscn:19-22`, per the
  phase-1 inventory §3, itself citing a landed precedent I did not re-verify this session — narrow,
  low-risk citation).
- `(deffield social-class/population int extensive)`
- `(deffield social-class/wealth int extensive)` — money workaround, same class as every landed
  pack's `wealth`/`budget` field.
- `(deffield social-class/production-value int extensive)` — the new field §5/§6 both need.
- `(deffield territory/biocapacity int extensive)`, `(deffield territory/max-biocapacity int
  extensive)` — same money-adjacent workaround as `wealth` (both `Currency`-typed in the Python
  model, `territory.py:155-164`).
- `(deffield territory/extraction-intensity coefficient intensive)` — natural `[0,1]` reading, never
  folded elsewhere (unlike `territory/heat`, no D131-class forcing applies).
- `(defconst economy/base-labor-power-annual 1.0c)` — wait, `1.0` is **not** in `[0,1]` by
  construction (it is the coefficient's *default*, but the type's domain is `[0.0, ∞)` per
  `economy_basic.py:168-173`, unbounded) — this needs the SAME scaled-bare-Int or plain-Int
  `:const` treatment Territory's `rent-level-x1e6`/Metabolism's `entropy-factor-x1e6` use if the
  seeded value can exceed 1.0 in a modded `defines.yaml`; at the *default* `1.0` a `coefficient`
  literal (`1.0c`) is exactly at the boundary and legal, but this is fragile — flag for the plan
  author's own D-record, do not silently assume `coefficient` is safe for an unbounded-domain
  define just because today's default happens to fit.
- `(defconst timescale/weeks-per-year 52)` — plain `int`, no domain issue (`ge=1`, unbounded above,
  and 52 has no `[0,1]` domain to violate — a bare unsuffixed `Int` const, same class as
  Metabolism's `entropy-factor-x1e6` escape hatch, `E-LEX-024` bounds only scaled/suffixed
  literals).
- Nodes: at least one `PERIPHERY_PROLETARIAT` (direct-producer path), at least two
  `LABOR_ARISTOCRACY` sharing one employer (proves the accumulate-into-shared-target shape, §4 —
  this is the vector `pool-contribution`'s own test proves the *mechanism* for, but no landed
  content pack yet proves it for THIS system's own shape, so the conformance fixture is where that
  first happens), one `CORE_BOURGEOISIE`/employer, at least one non-producer class holding a
  `TENANCY` edge to the same territory as a producer (proves §5's filter is load-bearing, not
  vacuously true), a territory with `max_biocapacity <= 0` (proves both zero-guards), a territory
  with zero tenants (proves the `intensity = 0.0` no-production case).
- Edges: `TENANCY` (worker→territory, multiple per class for at least one case, to exercise §4's
  tiebreak honesty note), `WAGES` (employer→worker, two LA workers to one employer for the
  accumulation vector).

**Frozen-mirror approach**: a `production_conformance.py` companion script, same shape as
`territory_conformance.py` (`content/scenarios/territory_conformance.py`, read in full this
session) — builds the identical graph node-for-node, runs `ProductionSystem().step(...)` once,
prints post-tick state. **Structure oracle, not byte oracle (ADR183)** — exactly Territory's own
disclaimer, restated for Production: *"What this script proves is that the BSL pack moves the SAME
fields in the SAME direction for the SAME reasons the frozen engine does — the conformance vectors
pinned in the Rust test file are measured from the BSL engine itself, not copied from this script's
printed floats."* Given §4's D-9-class divergence risk is much lower here than Territory's own
(Production's arithmetic is two plain multiplies and two plain adds, no scaled-multiply-then-divide
reordering anywhere unless the `defconst` domain issue above forces one), bit-identical agreement
between the frozen mirror and the BSL engine is plausible for most vectors, but should still be
**measured, not assumed** — the same discipline `territory_conformance.rs`'s own header states
(`:19-26`).

**qa:regression byte-gate coverage**: `SOCIAL_CLASS.wealth` and `TERRITORY.extraction_intensity`
are both node attributes, covered by `graph_content_hash` — confirmed by the inventory, not
independently re-read (a single, low-risk citation to `tools/regression_test.py:924-964`).
`social-class/production-value` (the new field this dossier's §5/§6 both introduce) would also be
node-attribute-covered automatically, in contrast to `la_production`'s own graph-metadata blind spot
the inventory flagged (§7 of the phase-1 inventory) — **a side benefit of the per-node reformulation
worth naming explicitly**: moving the channel off graph-scope and onto a node field also moves it
inside the byte-gate's coverage, for free.

---

## 11. Surprises, with evidence (recap)

1. **The `la_production` "BLOCKED" verdict was answering the wrong question** — it is per-node data
   wearing a graph-scope costume, not a real aggregate; §6.
2. **The employer-accumulation "unverified-in-practice" verdict undersold `dev`'s own proof** —
   `accumulation_into_a_shared_target_reduces_in_subject_order_and_keeps_every_contribution`
   (`tick.rs:994-1076`) is exactly this shape, already landed and tested, one topology hop shallower
   than the inventory's own "three-hop" framing suggested; §4.
3. **A blocker the inventory's "PORTABLE NOW" verdict missed entirely, independent of D102**: fold
   bodies cannot be conditional, and the flagship scenario's `TENANCY` topology is not
   role-restricted, so a naive fold-based extraction-intensity broadcast is a real correctness bug,
   not a style gap; §5. This is the one correction that goes in the *opposite* direction from this
   dossier's other two — tightening, not loosening, the inventory's verdict.
4. **`"production"` needs a brand-new system registration** — Territory's own dossier could report
   this as zero-cost because of a pre-existing placeholder; Production has no such placeholder; §3.
5. **D102's discharge, while real and independently verified, is not actually load-bearing for this
   port** under the recommended reformulation — worth flagging so a plan draft doesn't cite it as
   the reason the port works when the real reason is topology-based routing; §2.
6. **PR #569 merged mid-session** — the worktree/main-checkout split the task briefing described
   closed itself while this dossier was being written; §0 (provenance note).

---

## CORRECTIONS/FLAGS against `reports/port-inventories/production-port-phase1-inventory-2026-08-12.md`

Mirrors the Territory dossier's own highest-value section. Read this first if you have already read
the phase-1 inventory (including its own Adjudication section).

1. **§6's employer-routing row ("PORTABLE WITH D-RECORD (nontrivial reformulation) —
   UNVERIFIED IN PRACTICE") should read PORTABLE NOW, with a D-record for the tiebreak-order
   honesty caveat only** — `dev` already proves the exact accumulate-into-a-computed-shared-target
   shape (`tick.rs:994-1076`), not merely "grammatically provided for." §4.
2. **§6's extraction-intensity row ("PORTABLE NOW, reformulated as a fold") is WRONG as stated, for
   a reason neither the inventory nor its own Adjudication caught** — the fold-body
   compound-expression restriction blocks the needed role/active filter, and the flagship scenario's
   `TENANCY` topology proves the filter is not optional. A candidate fix is given (§5), but it is
   not a bare fold over `field-of it social-class/population` the way the inventory's text implies.
3. **§6/Adjudication's correction 2 (the "role must be int-ordinal, contradicting the report's own
   D-record" tension) is dissolved, but not for the reason a plan reader might assume** — D102 is
   discharged (verified, §2), but the recommended reformulation never needed a neighbor-side role
   read in the first place. Cite the topology-based design, not the D102 discharge, as the reason.
4. **Adjudication's correction 3 (D116 "safe on the substance" because the extraction fold reads
   pre-tick state) is now stale** under the corrected design, which deliberately depends on
   cross-rule visibility the same way Territory's own pack does — §9 restates the analysis the
   corrected design actually needs.
5. **The `la_production` BLOCKED verdict (§6's blocker table, reinforced by Adjudication correction
   5) is the wrong diagnosis, not merely an outdated one** — §6.
6. **§1's file map and §2-§5's computation catalog, defect writeup (fips_code/county_fips), type
   inventory, float-op inventory, and RESERVED-LINE note are all independently re-verified correct
   in this pass** — no corrections owed there, only confirmation (§1, §7, §8 above).
7. **New, not present in the inventory at all**: the `"production"` system-registration gap (§3) and
   the byte-gate-coverage side benefit of the per-node reformulation (§10, last paragraph).
