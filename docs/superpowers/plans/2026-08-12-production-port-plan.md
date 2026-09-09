# Production Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port ProductionSystem (@3.0) to a BSL rule pack — the second complete system port through the query-evaluation lane, and the first with a genuinely new system registration.

**Architecture:** One pack (`production.bsl`, four rules, byte-ordered), one conformance scenario + frozen Python mirror, one new tick golden, D-rows + ADR. No language slice is needed: the scout dossier (`reports/production-bsl-surface-facts-2026-08-12.md` — read it in full before Task 1; its §10 CORRECTIONS section governs where it contradicts the Phase-1 inventory) verified every construct is landed. The only Rust-source change is the `"production"` registration string.

**Tech Stack:** babylon-bsl / babylon-tick (Rust), the frozen Python engine as structure oracle (ADR183).

## Global Constraints

- **Port-as-is** (Director law): transcribe `src/babylon/engine/systems/production.py` exactly — no mid-port refactors, no formula changes. Defects transcribe verbatim with D-records.
- **RESERVED LINE** (Constitution IX.5): the role partition (`_DIRECT_PRODUCER_ROLES = {PERIPHERY_PROLETARIAT}`, `_EMPLOYED_PRODUCER_ROLES = {LABOR_ARISTOCRACY}`, production.py:46-52) and the WAGES-edge employer lookup are Amin/Wallerstein routing — the Director's ideological line. Transcribe the routing structure EXACTLY; any change to which role routes which way escalates to the Director.
- **Six-leg cargo gate per commit** (from `rust/`): `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo clippy -p babylon-kernel --all-targets -- -D warnings -D clippy::pedantic` and same for `-p babylon-bsl`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`; `cargo test -p babylon-tick --test tick_goldens --locked`. The five pre-existing golden pins (two-classes, vitality, us-counties, organization, territory) must stay byte-identical in every commit.
- **After any `docs/reference/bsl-language.rst` edit:** `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run pytest tests/unit/reference/test_bsl_grammar_sync.py -q` (41 tests). If a register probe reds because a new row cross-references an earlier D-code, repair the TEST ANCHOR (`section.index("* - DNNN")`, the test_d118 pattern) — never weaken an assertion — with its own mutation check.
- **An `int` deffield row constrains SEEDING only** — `bind_field_value` returns `Value::Real` for every non-enum field (tick.rs:318-334, the Territory MAJOR-1 lesson). Never add promotion idioms; `(/ (* a b) c)` over int-declared fields is legal as-is.
- **Defines consumed** (name the rows verbatim, no new coefficients): `economy.base_labor_power` (`defines.yaml:73`, default 1.0), `timescale.weeks_per_year` (`defines.yaml:374`, default 52).
- **defenum SocialRole order is hash-bearing (ADR195), transcribe verbatim from `src/babylon/models/enums/social.py:34-41`:** `CORE_BOURGEOISIE, PERIPHERY_PROLETARIAT, LABOR_ARISTOCRACY, PETTY_BOURGEOISIE, LUMPENPROLETARIAT, COMPRADOR_BOURGEOISIE, INTERNAL_PROLETARIAT, CARCERAL_ENFORCER`.
- **Mutation evidence** per rule commit: break → named test flips red → restore byte-identical, recorded in the report.

## Sequencing note (D116 / byte order)

The pack RELIES on cross-rule same-tick visibility (D116, `bsl-language.rst:5931-5957`), exactly as Territory's pack D-1 does: `production/p4-extraction-intensity` folds `social-class/production-value`, which p1/p2/p3 write earlier the same tick. Rule ids are chosen so byte order = intended order: `production/p1-direct-production` < `production/p2-employed-routing` < `production/p3-employed-fallback` < `production/p4-extraction-intensity`. Task 5's D-row records the dependency explicitly (Territory `territory.bsl:14-25` is the model).

The four rules split this way because (a) subject type derives from `:field` namespaces (SOCIAL_CLASS for p1-p3, TERRITORY for p4), and (b) p2's effect ref `(select-max (neighbors self EdgeType/WAGES :in …) 1)` ABORTS on an employer-less subject (E-EVAL-021 class), so employer existence must be split at the `when` level: p2 guards `(exists …)`, p3 guards `(not (exists …))` — `not` is served (grammar.rs:651, evaluator.rs:562), and `if` evaluates only the taken branch (§4.1, evaluator.rs:18), which is what makes the exists-guarded `field-of (select-max …)` bindings below legal.

---

### Task 1: Registration + the scenario ceremony

**Files:**
- Modify: `rust/crates/babylon-tick/src/lib.rs:174-203` (the registered-system `HashSet`)
- Create: `rust/crates/babylon-tick/content/scenarios/production-conformance.bscn`
- Create: `rust/crates/babylon-tick/content/scenarios/production_conformance.py`
- Test: `rust/crates/babylon-tick/tests/production_conformance.rs`

**Interfaces:**
- Produces: the fixture node ids and seeds every later task's assertions use; the mirror script Task 5's golden discipline cites.

- [ ] **Step 1: Write the failing load-smoke test** — `production_conformance.rs` with a `SCENARIO` const holding the full `.bscn` below and a test `scenario_and_empty_pack_load` that calls the real loader with an empty rule source. Expected: FAIL (unregistered system / missing scenario).
- [ ] **Step 2: Add `"production".to_owned()` to the lib.rs HashSet** (genuinely new — Territory had a placeholder, Production does not; scout §3).
- [ ] **Step 3: Write `production-conformance.bscn`.** Vocabulary/enum/field/const declarations:

```scheme
(defvocabulary NodeType (SOCIAL_CLASS TERRITORY))
(defvocabulary EdgeType (TENANCY WAGES))
(defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))
(deffield social-class/role enum SocialRole)
(deffield social-class/active int extensive)
(deffield social-class/population int extensive)
(deffield social-class/wealth int extensive)
(deffield social-class/production-value int extensive)
(deffield territory/biocapacity int extensive)
(deffield territory/max-biocapacity int extensive)
(deffield territory/extraction-intensity coefficient intensive)
(defconst economy/base-labor-power-annual 1.0c)
(defconst timescale/weeks-per-year 52)
```

Match the landed positional `.bscn` deffield dialect exactly (scenario.rs — copy the shape from `territory-conformance.bscn` and the defenum shape from the organization scenario). `1.0c` sits exactly at the coefficient boundary — legal today, fragile under modding; Task 5's D-row records it. `52` is a bare unsuffixed Int const (the Metabolism escape-hatch class; E-LEX-024 bounds only suffixed literals).

Nodes (declaration order IS NodeId order — declare in this order):

| node | type | role | active | population | wealth | prod-value | notes |
|---|---|---|---|---|---|---|---|
| worker-pp | SOCIAL_CLASS | PERIPHERY_PROLETARIAT | 1 | 100 | 10 | 0 | direct producer |
| worker-pp-two-lands | SOCIAL_CLASS | PERIPHERY_PROLETARIAT | 1 | 50 | 10 | 0 | TWO tenancy edges — tiebreak vector |
| worker-la-one | SOCIAL_CLASS | LABOR_ARISTOCRACY | 1 | 40 | 10 | 0 | employed |
| worker-la-two | SOCIAL_CLASS | LABOR_ARISTOCRACY | 1 | 60 | 10 | 0 | employed, same employer — accumulation vector |
| worker-la-orphan | SOCIAL_CLASS | LABOR_ARISTOCRACY | 1 | 30 | 10 | 0 | NO wages edge — fallback vector |
| worker-la-idle | SOCIAL_CLASS | LABOR_ARISTOCRACY | 0 | 80 | 10 | 0 | inactive — hash-neutral vector |
| comprador | SOCIAL_CLASS | COMPRADOR_BOURGEOISIE | 1 | 500 | 10 | 0 | tenant but non-producer — the p4 filter vector |
| employer | SOCIAL_CLASS | CORE_BOURGEOISIE | 1 | 10 | 10 | 0 | receives both LA products |
| t-alpha | TERRITORY | — | — | — | — | — | biocapacity 80, max 100, extraction 0.0c |
| t-beta | TERRITORY | — | — | — | — | — | biocapacity 50, max 100, extraction 0.0c |
| t-dead | TERRITORY | — | — | — | — | — | biocapacity 0, max 0 — zero-guard vector |
| t-empty | TERRITORY | — | — | — | — | — | biocapacity 100, max 100, no tenants — no-production vector |

Edges (TENANCY = worker→territory, so `:out` from the worker; WAGES = employer→worker, so `:in` from the worker — direction verified against production.py:241 `edge.target_id == worker_id` and Territory's D123 `:out`/`:any` register contrast):

```scheme
(edge EdgeType/TENANCY worker-pp t-alpha 1)
(edge EdgeType/TENANCY worker-pp-two-lands t-alpha 1)
(edge EdgeType/TENANCY worker-pp-two-lands t-beta 1)   ; second land — tiebreak
(edge EdgeType/TENANCY worker-la-one t-beta 1)
(edge EdgeType/TENANCY worker-la-two t-beta 1)
(edge EdgeType/TENANCY worker-la-orphan t-alpha 1)
(edge EdgeType/TENANCY worker-la-idle t-beta 1)
(edge EdgeType/TENANCY comprador t-alpha 1)            ; the non-producer tenant
(edge EdgeType/WAGES employer worker-la-one 1)
(edge EdgeType/WAGES employer worker-la-two 1)
(edge EdgeType/WAGES employer worker-la-idle 1)
```

Every declared field seeded on every node of its namespace (the no-defaults law, scenario.rs:56-58). No TENANCY for `employer` (the frozen employer produces nothing); none for `t-empty`.

- [ ] **Step 4: Write the frozen mirror** `production_conformance.py` — same genre as `territory_conformance.py` (read it first): build the identical graph node-for-node (same order), run the frozen `ProductionSystem` one step, print post-tick wealth/production ledger (`la_production` graph attr) per node and extraction_intensity per territory. Structure oracle, not byte oracle (ADR183) — state that disclaimer in its header verbatim from the Territory mirror's.
- [ ] **Step 5: Run the load-smoke test.** Expected: PASS. Run the mirror; record its printed numbers in the test file header comment as the structure oracle.
- [ ] **Step 6: Commit** `test(tick): production conformance scenario + frozen mirror + system registration (port train, fixture ceremony)`.

### Task 2: Rule p1 — direct production

**Files:**
- Create: `rust/crates/babylon-tick/content/rules/production.bsl` (pack header + p1)
- Test: extend `rust/crates/babylon-tick/tests/production_conformance.rs`

**Interfaces:**
- Produces: the shared produced-value binding chain (p2/p3 copy it verbatim); `social-class/production-value` writes p4 consumes.

- [ ] **Step 1: Failing test** — `p1_direct_producer_accumulates_own_wealth`: run the real driver (`run_once_into`) with scenario+pack, assert worker-pp's wealth moved by exactly the mirror-derived amount (pin the BSL-measured bits after first green, per Territory practice) and `production-value` equals the same amount; assert comprador/employer wealth **unmoved** by p1 (10.0).
- [ ] **Step 2: Write the pack header + p1.** Pack header carries the file-local D-record block (D-1..D-n, Dispossession/Lifecycle convention) — reserve D-1 for the byte-order reliance (filled in Task 5). The rule:

```scheme
(rule production/p1-direct-production
  :material-basis "Fundamental Theorem plumbing: the periphery proletariat produces value with its own labor-power on land it occupies (produced = weekly labor-power x population x biocapacity ratio, production.py:151-175) and, as the direct producer with no wage relation, keeps its own product (production.py:179-181)."
  :fuel 160
  (bindings
    (binding role :field social-class/role)
    (binding active :field social-class/active)
    (binding population :field social-class/population)
    (binding annual :const economy/base-labor-power-annual)
    (binding weeks :const timescale/weeks-per-year)
    (binding bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                           (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                     territory/biocapacity)
                           (- 0 0c)))
    (binding max-bio :expr (if (exists (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY))
                               (field-of (select-max (neighbors self EdgeType/TENANCY :out NodeType/TERRITORY) 1)
                                         territory/max-biocapacity)
                               (- 0 0c)))
    (binding bio-ratio :expr (if (> max-bio 0) (/ bio max-bio) (- 0 0c)))
    (binding produced :expr (* (* (/ annual weeks) population) bio-ratio))
    (binding output :expr (if (= active 1) produced (- 0 0c))))
  (when (= role SocialRole/PERIPHERY_PROLETARIAT))
  (effects
    (update-node self social-class/wealth (add output))
    (update-node self social-class/production-value (set output))))
```

Association order `(* (* weekly population) bio-ratio)` matches production.py:175 exactly. The `active` gate lives in the OPERAND, not the `when` (Territory's D127 hash-neutral idiom): an inactive producer still fires, adds `0.0`, and `set`s production-value to 0 — matching the frozen absent-ledger-entry semantics without cross-tick staleness. A tenancy-less producer likewise computes 0 through the exists-guarded bindings rather than a `when` skip. `(select-max … 1)` with a constant score is D46-legal; when a producer holds several TENANCY edges it picks by D45 ascending-id, where the frozen `_find_tenancy_target` takes first-insertion-order — the D124-class divergence Task 5's D-row records; worker-pp-two-lands pins the D45 winner (t-alpha, the lower NodeId).

- [ ] **Step 3: Run the test.** Expected: PASS with mirror-agreeing structure. Pin exact bits.
- [ ] **Step 4: Mutation** — change `(add output)` to `(add produced)` (drops the active gate): the p1 test must stay green (worker-pp is active) but the Task-5 idle-worker assertion would flip; defer that run to Task 5 and instead mutate `(when (= role SocialRole/PERIPHERY_PROLETARIAT))` to `LABOR_ARISTOCRACY`: `p1_direct_producer...` flips red. Restore byte-identical.
- [ ] **Step 5: Six legs + commit** `feat(tick): production p1 direct-producer rule`.

### Task 3: Rules p2/p3 — employed routing + fallback (RESERVED LINE)

**Files:**
- Modify: `rust/crates/babylon-tick/content/rules/production.bsl`
- Test: extend `production_conformance.rs`

- [ ] **Step 1: Failing tests** — `p2_two_la_products_accumulate_into_one_employer` (employer wealth = 10 + product(la-one) + product(la-two), both contributions kept — the D103/D104 shape `tick.rs:994-1076` proves for the mechanism; this is its first content-pack instance); `p2_idle_la_adds_nothing` (employer unmoved by worker-la-idle; its production-value = 0); `p3_orphan_la_keeps_own_product` (worker-la-orphan self-accumulates); `la_wealth_unmoved_by_p2` (employed LA wealth stays 10 — the product routes AWAY).
- [ ] **Step 2: Write p2 and p3** — same binding chain as p1 verbatim, different `when`/target:

```scheme
(rule production/p2-employed-routing
  :material-basis "Amin/Wallerstein: the labor aristocracy's product is appropriated by the employing bourgeoisie through the WAGES relation (production.py:184-194). RESERVED LINE - the routing structure is the Director's ideological line, transcribed exactly."
  :fuel 192
  (bindings <p1's chain verbatim>)
  (when (and (= role SocialRole/LABOR_ARISTOCRACY)
             (exists (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS))))
  (effects
    (update-node (select-max (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS) 1)
                 social-class/wealth
                 (add output))
    (update-node self social-class/production-value (set output))))

(rule production/p3-employed-fallback
  :material-basis "The frozen fallback: an employed-role producer with no employer keeps its own product (production.py:196-198)."
  :fuel 160
  (bindings <p1's chain verbatim>)
  (when (and (= role SocialRole/LABOR_ARISTOCRACY)
             (not (exists (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS)))))
  (effects
    (update-node self social-class/wealth (add output))
    (update-node self social-class/production-value (set output))))
```

`production-value` is written by ALL THREE producer rules where the frozen `la_production` ledger covers only the employed branch (production.py:194) — the scout §6 widening: it widens the write, never the read (only LA workers have an incoming WAGES edge for ImperialRent's future lookup to find), and p4 REQUIRES the widened field (the frozen `territory_production` accumulates from every producer, production.py:200-204). Task 5's D-row records this as the la_production per-node reformulation.

- [ ] **Step 3: Tests green; pin bits.**
- [ ] **Step 4: Mutation** — swap p2's `:in` to `:out` on the WAGES query: the accumulation test flips (employer never found). Restore byte-identical.
- [ ] **Step 5: Six legs + commit** `feat(tick): production p2+p3 wage routing -- Amin/Wallerstein line transcribed exactly`.

### Task 4: Rule p4 — extraction-intensity broadcast

**Files:**
- Modify: `rust/crates/babylon-tick/content/rules/production.bsl`
- Test: extend `production_conformance.rs`

- [ ] **Step 1: Failing tests** — `p4_extraction_reflects_producer_value_only` (t-alpha's intensity counts worker-pp + worker-pp-two-lands' t-alpha share + worker-la-orphan but NOT comprador — the filter vector: comprador's production-value stays 0 because no producer rule fires for it); `p4_zero_max_biocapacity_yields_zero` (t-dead = 0.0); `p4_no_tenants_yields_zero` (t-empty = 0.0); `p4_upper_clamp` — only if the fixture arithmetic can exceed 1.0 (check the mirror; if not, assert the sub-1.0 value and note the clamp is exercised structurally by the `(if (< ratio 1) …)` shape).
- [ ] **Step 2: Write p4:**

```scheme
(rule production/p4-extraction-intensity
  :material-basis "Metabolic coupling: extraction intensity = produced value against the territory's max biocapacity, clamped to [0,1] (production.py:246-268). Reads production-value written by p1-p3 THIS TICK - the pack relies on D116 byte-order cross-rule visibility (see pack D-1)."
  :fuel 128
  (bindings
    (binding max-bio :field territory/max-biocapacity)
    (binding total :expr (if (exists (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS))
                             (fold sum (neighbors self EdgeType/TENANCY :in NodeType/SOCIAL_CLASS)
                                   (field-of it social-class/production-value))
                             (- 0 0c)))
    (binding ratio :expr (if (> max-bio 0) (/ total max-bio) (- 0 0c)))
    (binding clamped :expr (if (< ratio 1) ratio (- 1 0c))))
  (effects
    (update-node self territory/extraction-intensity (set clamped))))
```

The fold body is the BARE ACCESSOR (`field_ref_for` refuses compounds — rule_pipeline.rs:770-778); the role/active filter lives in p1-p3's `when` guards via the per-node field, NOT in the fold (the scout §5 correction — a naive population fold would silently count the comprador). No `when` clause if a landed pack omits it for always-fire rules; otherwise use the landed always-true idiom — copy whichever shape `territory.bsl` p1 uses, do not invent one. `production-value` folds as `int extensive` (sum over extensive is legal; the D131 forcing does not recur — the field is money-like, extensive is its honest kind).

- [ ] **Step 3: Tests green; pin bits.** Cross-check every pinned territory intensity against the mirror's printout.
- [ ] **Step 4: Mutation** — remove `comprador`'s TENANCY edge from a scratch copy? No: mutate the ROLE gate instead — change p1's `when` to also admit `COMPRADOR_BOURGEOISIE`: `p4_extraction_reflects_producer_value_only` flips (t-alpha inflates). Restore byte-identical.
- [ ] **Step 5: Six legs + commit** `feat(tick): production p4 extraction-intensity -- pull-side fold over the per-node production field`.

### Task 5: Composition golden + D-rows + ADR + handoff

**Files:**
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs` (ADD `production_conformance` pin — a pure addition; the five existing pins untouched)
- Modify: `rust/crates/babylon-tick/tests/production_conformance.rs` (full-pack census test; idle-worker hash-neutral assertion)
- Modify: `docs/reference/bsl-language.rst` (register rows; next free numbers — RE-CHECK the register first, D132+ expected)
- Modify: `rust/crates/babylon-tick/content/rules/production.bsl` (pack-header D-1..D-n block)
- Create: `ai/decisions/ADR200_production_port_handoff.yaml` (next free ADR number — verify against `ai/decisions/index.yaml`, add index row)
- Modify: `reports/port-inventories/production-port-phase1-inventory-2026-08-12.md` (append the post-train UPDATE block, Territory-inventory pattern)

- [ ] **Step 1: The sixth golden** — full scenario + full pack through the real driver, pin `fired` (expected 10: p1×2 + p2×3 + p3×1 + p4×4 — verify, don't trust this arithmetic) and the state hash. Also the idle-worker vector: assert worker-la-idle's firing moved nothing observable (wealth 10, employer delta excludes it, production-value 0).
- [ ] **Step 2: Register rows** (global numbers, pack-local D-N mirrors citing them — the two-homes/two-sequences convention, `ai/bsl-architecture-standard.md` §6.2's template):
  - the D116 byte-order reliance (p4 reads p1-p3's same-tick writes; Territory pack D-1 is the model);
  - the fips_code/county_fips dead tensor branch OMITTED (scout §7's drafted row — provably unreachable, nothing to transcribe);
  - the la_production per-node reformulation (dict-keyed-by-node-id → `social-class/production-value`; write widened to all producer roles, read stays WAGES-narrow; byte-gate coverage gained — scout §6/§10);
  - the tiebreak divergence (frozen first-insertion-order vs D45 ascending-id for multi-TENANCY/multi-WAGES; D124 class, live only off the qa estate);
  - the `1.0c` coefficient-boundary fragility for `economy/base-labor-power-annual` (domain [0,∞) vs coefficient [0,1]; a modded value >1.0 refuses at load — loud, acceptable, recorded);
  - the extraction-filter reformulation (role filter moved from fold-body — where `field_ref_for` refuses compounds — to the producer rules' `when` guards via the per-node field; scout §5, the dossier's headline correction).
- [ ] **Step 3: ADR + inventory UPDATE block + the pack-header D-block.** ADR records: four rules, the reformulations, the RESERVED-LINE transcription statement, gate evidence. Inventory UPDATE: verdict PORTED, corrections cross-referenced to the scout dossier.
- [ ] **Step 4: Run the RST sync suite** (see Global Constraints) + all six cargo legs.
- [ ] **Step 5: Commit** `docs(p27): the Production port handoff -- D-records, register rows, ADR200, inventory verdict`.

---

## PR grouping

ONE PR: branch `feat/production-port-bsl`, Tasks 1-5, five commits. Dual review lens for the verification arc: (a) **RESERVED-LINE fidelity** — the routing structure byte-compared against production.py's branch logic, plus the standing composed-path lens; (b) **cross-rule/D116 integrity** — the p1-p3→p4 dataflow, fold-filter correctness (the comprador vector), and mirror parity. Client-lens note: extraction-intensity has a natural county projection; per the Director's lens-rides-the-port policy this train should either land the Bevy lens or carry an explicit deferral — Territory deferred silently, so this plan flags the question to the Director rather than repeating the silence: **deferral recommended** (the 12-county demo world seeds no production fields yet; the lens becomes honest when the demo world grows economics), recorded in the ADR.

## Self-review notes

- Every construct in the four rules is landed and cited: `not` (grammar.rs:651), lazy `if` (evaluator.rs:18), exists-guarded computed-ref bindings (Territory p3 precedent), `select-max` constant score (D46; tick.rs:1031-1040), accumulate-into-shared-target (D103/D104; tick.rs:994-1076), bare-accessor fold (rule_pipeline.rs restriction), `:in`/`:out` direction semantics (D123 contrast).
- The fixture exercises: direct routing, dual-LA accumulation, orphan fallback, inactive hash-neutrality, the non-producer filter, both zero-guards, the tiebreak, and the no-tenant territory. Every SOCIAL_CLASS node carries every social-class/* field; every TERRITORY node carries every territory/* field.
- Known judgment left to the implementer, flagged for the verifier: exact `:fuel` values (declared generously here; tighten to the bound-checker's computed bound + margin if load refuses), the always-fire `when` idiom for p4 (copy landed practice), and whether the fixture arithmetic can reach the upper clamp (mirror decides).
