# Decomposition @11.0 + ControlRatio @12.0 — Joint Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port the last two ungated Material Base systems — `DecompositionSystem` (@11.0) and `ControlRatioSystem` (@12.0) — into BSL rule packs, as ONE joint train. The two are coupled by exactly one key (`decomposition.py:223` writes `persistent_data["_class_decomposition_tick"]`; `control_ratio.py:128` reads it), so they must co-design one carrier; the survey rules the train MANDATORY JOINT (`reports/port-estate-survey-2026-08-12.md:175-182`).

**Architecture:** New BSL content plus one Rust registration edit. Two rule packs (`decomposition.bsl`, 6 rules; `control-ratio.bsl`, 4 rules), one **singleton carrier node** carrying the frozen `TickContext.persistent_data` state machine as declared graph fields, seven conformance scenarios with Python mirrors, additive golden pins, D-rows, ADR. **No language slice is needed** — every construct is landed and cited in §3 below. **One frozen behavior CANNOT port** (`add-node`, §4 BLOCKER-1) and is omitted with a D-record plus a named follow-on.

**Tech Stack:** Rust workspace (`rust/crates/{babylon-bsl,babylon-graph,babylon-tick}`), BSL content, cargo via `mise run rust:check`, Python 3.12 host venv for the frozen mirrors.

**Rulings that govern:** joint train + port-as-is + revolution-vs-genocide-verbatim-under-a-P19-cutover-pending-D-record (ruled 2026-08-12; reaffirmed **ADR208 R29 / C-03**, `ai/decisions/ADR208_docket_sitting_2026_08_17.yaml:246-256`, which also verifies the train **fully independent of #576**, the intrinsic-host train).

**Prior art to read before Task 1 (in this order):**
1. `reports/port-inventories/decomposition-port-phase1-inventory-2026-08-12.md` (561 lines) and `reports/port-inventories/control-ratio-port-phase1-inventory-2026-08-12.md` (485 lines) — including their **Adjudication** sections, which correct their own bodies.
2. `reports/port-estate-survey-2026-08-12.md` §2a rows 11.0/12.0, §3, §4.1 MB-3, §6 (the coverage ledger that names this train's owed re-reads).
3. `docs/superpowers/plans/2026-08-12-production-port-plan.md` — the closest structural precedent (new registration + new scenario + fold reformulation).
4. `docs/superpowers/plans/2026-08-15-class-surface-ternary-port.md` — the freshest plan; its **Controller amendments** section carries live type-lane facts (int fields store verbatim f64; fractional seeds refuse; `defenum` is not shared across scenarios).
5. `rust/crates/babylon-tick/content/rules/dispossession.bsl` + `tests/dispossession_conformance.rs` + `content/scenarios/dispossession_conformance.py` — the rule/oracle/test triad idiom, and the 7-companion-scenario branch-coverage pattern this train reuses.
6. `ai/bsl-architecture-standard.md` §3.2 / §6.2 — no imposed functional forms, declared domains, III.11 loud absence, the two-homes D-record convention.

---

## Global Constraints

Every task's requirements implicitly include this section.

- **Port-as-is (Director law, ADR183).** The frozen Python is the **structure and ordering contract, not a correctness oracle**. Transcribe exactly; every divergence earns a D-row. Defects transcribe verbatim (§2 lists four). Never silently repair.
- **RESERVED LINE (Constitution IX.5 / ADR070 / Program 19).** `ControlRatioSystem`'s revolution-vs-genocide branch (`control_ratio.py:210-247`) is Director-reserved and is ruled **explicitly LAST** in the emergent-class-partition cutover. This train **transcribes the branch verbatim** — same threshold source (`carceral.revolution_threshold`), same `>=` comparison, same two outcomes — under a **P19-cutover-pending D-record**. Any change to *which* organization measure decides, or to the partition the roles come from, escalates to the Director. Describe, never propose (the inventory's own discipline, control-ratio inventory §6 last row).
- **Every pre-existing golden byte-identical at landing.** This train adds new files; the only modified files are `rust/crates/babylon-tick/src/lib.rs` (two registration strings), `tests/tick_goldens.rs` (additive pins), and docs/records. The 8 existing pinned scenario/rule pairs (`two_classes`, `vitality`, `us_counties_lifecycle_demo`, `organization_foundation`, `territory_conformance`, `production_conformance`, `worldview_foundation`, `consciousness_ternary_foundation`) must stay byte-identical in **every** commit. If one moves: **STOP**.
- **No Python source changes, none.** The frozen engine is read-only reference. `mise run qa:regression` and `mise run qa:vault-regression-ci` must therefore be byte-identical trivially — run them once anyway as proof (Task 9). No file under `tests/baselines/**` may move; if one does, **STOP** — that is a §6.5 ceremony, not a side effect.
- **No new formalism.** No new verbs, no new intrinsics, no new mathematics. The carrier node and the per-node census fields are **content reformulations** of Python locals/dicts, in the same class as Production's `social-class/production-value` (D-recorded, not minted math).
- **Vocabulary discipline.** `NodeType`/`EdgeType`/`SocialRole` members come from the canonical Python enums verbatim; **enum member order is hash-bearing (ADR195)** — transcribe `SocialRole` from `src/babylon/models/enums/social.py:34-41` in exactly the landed order: `CORE_BOURGEOISIE, PERIPHERY_PROLETARIAT, LABOR_ARISTOCRACY, PETTY_BOURGEOISIE, LUMPENPROLETARIAT, COMPRADOR_BOURGEOISIE, INTERNAL_PROLETARIAT, CARCERAL_ENFORCER`. The carrier uses the **existing** `NodeType/INSTITUTION` member (`src/babylon/models/enums/topology.py:53`) — this train mints **no** new node or edge type.
- **`defenum` is not shared across scenarios** (one `(scenario …)` form per source): every scenario re-declares `(defenum SocialRole …)` and the suite carries one ordinal-parity test mirroring the mint's (class-surface plan amendment 7).
- **Six-leg cargo gate per commit** (from `rust/`): `cargo fmt --check`; `cargo clippy --workspace --all-targets -- -D warnings`; `cargo test --workspace`; `cargo clippy -p babylon-kernel --all-targets -- -D warnings -D clippy::pedantic` and same for `-p babylon-bsl`; `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`; `cargo test -p babylon-tick --test tick_goldens --locked`. `mise run rust:check` green after every task.
- **After any `docs/reference/bsl-language.rst` edit:** `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run pytest tests/unit/reference/test_bsl_grammar_sync.py -q`. If a register probe reds because a new row cross-references an earlier D-code, repair the **test anchor** (`section.index("* - DNNN")`, the `test_d118` pattern) — never weaken an assertion.
- **Mutation evidence per rule commit:** break → a named test flips red → restore byte-identical, recorded in the commit body. Every clamp/branch constant must be mutation-provable by a fixture that exercises it, plus its converse test proving the other fixtures do not (the `t-tight` / `p4_all_original_territories_stay_sub_one` idiom, `production-conformance.bscn:84-92`).
- **Defines consumed — name them verbatim, mint no new coefficient.** All from `CarceralDefines` (`src/babylon/config/defines/territory.py:248-300`), values from `src/babylon/data/defines.yaml:293-300`:

  | defconst qname | value | frozen source |
  |---|---|---|
  | `carceral/control-capacity` | `4` | `defines.yaml:294` |
  | `carceral/enforcer-fraction` | `0.15c` | `defines.yaml:295` |
  | `carceral/proletariat-fraction` | `0.85c` | `defines.yaml:296` |
  | `carceral/revolution-threshold` | `0.5c` | `defines.yaml:297` |
  | `carceral/decomposition-delay` | `52` | `defines.yaml:298` |
  | `carceral/control-ratio-delay` | `52` | `defines.yaml:299` |
  | `carceral/terminal-decision-delay` | `1` | `defines.yaml:300` |
  | `carceral/approaching-consumption-multiple` | `2` | **NO defines backing** — a bare `2` literal at `decomposition.py:155`. D-row obligation. |

  Companion scenarios MAY vary a delay/fraction to make a branch reachable at tick 1 — the landed dispossession practice (7 companion `:const` environments, `dispossession.bsl:49-72`). **Constraint:** no companion may set `carceral/control-ratio-delay` to `0` unless its `decomposition-fire-tick` is **seeded** rather than written the same tick (§5 byte-order hazard).
- **Branch from `dev` in an isolated worktree** (superpowers:using-git-worktrees). PR A branch `feature/decomposition-port-bsl`; PR B branches off **merged dev**, `feature/control-ratio-port-bsl` — never stacked (#193). Conventional commits via `mise run commit`; merges only via `mise run pr:merge -- N`, after harvesting the Copilot review (ADR181).
- **Token economy:** subagents write artifacts to files and return ≤15-line summaries.

---

## 1. Frozen-source archaeology

Done for this plan by direct line-by-line read of both modules (369 + 247 lines). This section is the plan's foundation; implementers transcribe from *here* and verify against the cited lines.

### 1.1 `DecompositionSystem` — `src/babylon/engine/systems/decomposition.py`

**Class surface** (`:89-108`): `partition = MATERIAL_BASE`, `position = 11.0`, `name = "Decomposition"`, `creates_value = True` (spec-053 INV-001 default-deny, comment at `:105-108` says the flip to `False` awaits a sum-preserving audit — port-relevant only as a recorded note).

**Module constants** (`:32-33`): `_ENFORCER_ID_OFFSET = 700`, `_INTERNAL_PROLETARIAT_ID_OFFSET = 800`.

**Helpers**
- `_derive_entity_id(graph, base_id, offset)` (`:36-50`): digits of `base_id` mod 1000, plus offset, advanced past collisions in a **bounded 1000-iteration loop**; deterministic (no `hash()`, III.7).
- `_find_entity_by_role(graph, role, include_inactive=False)` (`:53-86`): iterates `graph.query_nodes(node_type=NodeType.SOCIAL_CLASS)`, skips `attrs.get("active", True)` falsy unless `include_inactive`, coerces `role` from str, returns the **FIRST** match as `(node_id, attrs)` or `None`.

**`step()` (`:110-223`) — reads**

| datum | site | note |
|---|---|---|
| `context.get("tick", 0)` | `:123` | the delay arithmetic's clock |
| `context.persistent_data` | `:125` | the whole state machine; keys `_decomposition_complete`, `_superwage_crisis_tick`, `_class_decomposition_tick` |
| LA `wealth` | `:149` | via `_find_entity_by_role(LABOR_ARISTOCRACY)`, **active-only** (`:143`) |
| LA `subsistence_threshold` | `:150` | default `0.0` |
| LA `population` | `:151` | default `0` |
| LA `s_bio` + `s_class` | `:153` | summed into `consumption` |
| `services.event_bus.get_history()` | `:167-171` | filtered `e.type == EventType.SUPERWAGE_CRISIS.value`, `min(e.tick)` |
| `services.defines.carceral.decomposition_delay` | `:206` | |

**`step()` — control flow, verbatim**
1. `:128-129` return if `_decomposition_complete`.
2. `:144-159` compute two flags off the single active LA: `la_approaching_death` iff `la_wealth < subsistence + (2 * consumption) and la_pop > 0` (`:155-156`); `la_about_to_die` iff `la_wealth < subsistence and la_pop > 0` (`:158-159`).
3. `:162-175` `superwage_tick = persistent["_superwage_crisis_tick"]`; if unset, scan event history and latch `min(tick)`.
4. `:179-197` **early warning**: if `la_approaching_death and superwage_tick is None and la_id is not None` → emit `SUPERWAGE_CRISIS` (payload `payer_id=CORE_BOURGEOISIE_ID`, `receiver_id=la_id`, `desired_wages=0.0`, `available_pool=0.0`, `narrative_hint`) and set `_superwage_crisis_tick = tick`.
5. `:200-208` decide: `la_about_to_die` → decompose now (fallback, bypasses the delay); else `superwage_tick is not None and tick >= superwage_tick + delay` → decompose.
6. `:210-211` return if not decomposing.
7. `:217` `_execute_decomposition(...)`; `:221-223` on success set `_decomposition_complete = True` and `_class_decomposition_tick = tick`.

**`_create_target_entity` (`:225-261`) — writes (BLOCKED, see §4)**
`graph.add_node(new_id, "social_class", id=new_id, name=f"{role.value} (decomposed from {la_id})", role=role.value, active=False, population=0, wealth=0.0, county_fips=<LA's>, subsistence_threshold=<LA's>, s_bio=<LA's or 0.01>, s_class=<LA's>, inequality=<LA's>)`.

**`_execute_decomposition` (`:263-369`) — writes and emits**

| step | site | exact semantics |
|---|---|---|
| re-find LA (active-only) | `:280-284` | `return False` if absent (`:282`) |
| population guard | `:290-291` | `return False` if `la_population <= 0` |
| fractions | `:294-295` | `enforcer_fraction` 0.15, `proletariat_fraction` 0.85 |
| splits | `:298-301` | `enforcer_pop_gain = int(la_population * enforcer_fraction)`; `proletariat_pop = int(la_population * proletariat_fraction)`; `enforcer_wealth_gain = la_wealth * enforcer_fraction`; `proletariat_wealth = la_wealth * proletariat_fraction` — **`int()` truncation on the two populations, none on the wealth** |
| find targets | `:306, :311-313` | `include_inactive=True` on **both** (contrast with the LA lookup's active-only default) |
| create if absent | `:307-310, :314-321` | the BLOCKED branch |
| **ENFORCER write** | `:327-332` | `population = current_pop + enforcer_pop_gain`, `wealth = current_wealth + enforcer_wealth_gain`, `active=True` — **ADDITIVE** |
| **IP write** | `:336` | `population = proletariat_pop`, `wealth = proletariat_wealth`, `active=True` — **OVERWRITE, not additive** |
| **LA write** | `:339` | `active=False` **only** — `wealth` and `population` are NEVER zeroed |
| emit | `:342-368` | `CLASS_DECOMPOSITION`, payload: `source_class`, `source_population`, `source_wealth`, `enforcer_fraction`, `proletariat_fraction`, `population_transferred={to_enforcer, to_proletariat}`, `wealth_transferred={to_enforcer, to_proletariat}`, `trigger_event="superwage_crisis"`, `narrative_hint` |

**Events emitted:** 2 — `SUPERWAGE_CRISIS` (`:182`), `CLASS_DECOMPOSITION` (`:344`).
**Edges:** none. Zero `query_edges`/`add_edge`/`update_edge` calls (grep-confirmed, decomposition inventory §6 closing note) — so **no D35/D65 edge-attribute exposure**, no Slice 2 dependency.
**libm:** none.

### 1.2 `ControlRatioSystem` — `src/babylon/engine/systems/control_ratio.py`

**Class surface** (`:88-103`): `partition = MATERIAL_BASE`, `position = 12.0`, `name = "ControlRatio"`, `creates_value = False`.

**Module constant** (`:32-37`): `_PRISONER_ROLES = frozenset({INTERNAL_PROLETARIAT, LUMPENPROLETARIAT})` — **two** roles, not one.

**Census helpers**
- `_count_enforcer_population` (`:53-62`): over all `SOCIAL_CLASS` nodes, skip inactive (`:58-59`), role `== CARCERAL_ENFORCER`, `total += attrs.get("population", 0)`.
- `_count_prisoner_population_and_org` (`:65-85`): same iteration, role `in _PRISONER_ROLES`, accumulates `total_pop += pop` and `org_sum += pop * org` (`organization`, default `0.0`) — a **population-weighted** sum, the correct intensive aggregation.

**`step()` (`:105-173`) — control flow, verbatim**
1. `:119` tick; `:121` `persistent`.
2. `:124-125` return if `_terminal_decision_emitted`.
3. `:128-130` return if `_class_decomposition_tick` is `None` — **the joint-train coupling**.
4. `:132-134` return if `tick < decomposition_tick + defines.carceral.control_ratio_delay`.
5. `:137-138` census.
6. `:141-142` return if `prisoner_pop == 0`.
7. `:146-147` `control_capacity`; `max_controllable = enforcer_pop * control_capacity`.
8. `:150-151` return if `prisoner_pop <= max_controllable` — **`<=`, not `<`** (the frozen suite's own MutationKillers class pins this boundary).
9. `:154-159` if not `_control_crisis_emitted` → `_emit_crisis(...)`, set `_control_crisis_emitted = True`, `_control_ratio_crisis_tick = tick`.
10. `:166-168` return if `tick < crisis_tick + defines.carceral.terminal_decision_delay`.
11. `:171` `avg_organization = prisoner_org_sum / prisoner_pop`.
12. `:172-173` `_emit_terminal_decision(...)`, set `_terminal_decision_emitted = True`.

**`_emit_crisis` (`:175-208`)**: `actual_ratio = prisoner_pop / enforcer_pop if enforcer_pop > 0 else float("inf")` (`:185`); `over_capacity_by = prisoner_pop - max_controllable` (`:186`). Payload: `enforcer_population`, `prisoner_population`, `control_capacity`, `max_controllable`, `actual_ratio`, `over_capacity_by`, **`control_ratio` (a duplicate of `actual_ratio`)**, `capacity_threshold = float(control_capacity)`, `narrative_hint`.

**`_emit_terminal_decision` (`:210-247`)** — **the ADR070-reserved branch**: `avg_organization >= revolution_threshold` → `outcome = "revolution"` else `"genocide"`. Payload: `outcome` (string), `avg_organization`, `revolution_threshold`, `prisoner_population`, `enforcer_population`, `narrative_hint`.

**Events emitted:** 2 — `CONTROL_RATIO_CRISIS` (`:190`), `TERMINAL_DECISION` (`:237`).
**Graph writes:** **ZERO** (grep-confirmed: no `update_node|add_node|remove_node|set_graph_attr|update_edge`, control-ratio inventory adjudication confirmation 2). Its entire state is `persistent_data` — state the frozen reference itself loses on save/load.
**Edges:** none. **libm:** none.

### 1.3 The four frozen defects, transcribed verbatim (port-as-is)

1. **Docstring drift.** `decomposition.py:3-5` says "30% … 70%"; `CarceralDefines`' shipped values are `0.15`/`0.85` (`defines.yaml:295-296`), and `territory.py:265-267`'s "With 70/30 decomposition, prisoner/enforcer = 2.33:1 … control_capacity >= 3: No crisis" is arithmetic for the *stale* fractions. At the shipped values the ratio is **5.67:1** and `control_capacity = 4` **does** produce a crisis. The code governs; the comments are wrong. D-row.
2. **Additive/overwrite asymmetry.** Enforcer `+=`, internal proletariat `=` (`:327-332` vs `:336`). Two different update ops, never unified.
3. **Non-conservation.** The LA keeps its `wealth` and `population` while being deactivated (`:339`), so the transfer is a copy, not a move. A ported `fold sum` over wealth must NOT silently "fix" it.
4. **Payload duplication + a non-finite value.** `actual_ratio` and `control_ratio` carry the same number (`:198-199`); the zero-enforcer case is `float("inf")` (`:185`) — unrepresentable in BSL (§4 BLOCKER-4).

### 1.4 Coverage reality (why the fixtures must be hand-built)

Both systems are **totally dormant on all 12 canonical scenarios** — `SUPERWAGE_CRISIS` never fires inside 150 ticks, so `tools/regression_test.py::graph_content_hash` gives **zero** byte-gate coverage today (both inventories §7; coverage-gap rows verbatim at `tools/regression_scenarios.py:2817-2830`). The only live end-to-end exercise in the estate is `tests/scenarios/test_carceral_equilibrium.py` (`MAX_TICKS = 5200`). Consequence: **the conformance fixtures must recalibrate the trigger, not merely run longer** (survey §4.1 MB-3). This plan recalibrates via the **fallback trigger** (`la_about_to_die`, `:201-202`), which fires with no delay at all, plus companion scenarios that vary a delay defconst.

The frozen behavioral estate that the port must not contradict (read, do not modify): `tests/unit/engine/systems/test_la_decomposition.py` (533), `test_control_ratio.py` (756, incl. a `TestControlRatioMutationKillers` class pinning `<=` vs `<`), `tests/unit/engine/laws/test_law_decomposition_system.py` (292), `test_law_control_ratio.py` (270), `tests/integration/mechanics/test_class_decomposition.py` (399), `test_control_ratio_crisis.py` (373), `tests/unit/engine/systems/test_decomposition_enforcer_creation.py` (121).

---

## 2. The reformulation: how `persistent_data` becomes graph state

The frozen state machine lives in a Python dict that no `<bind-src>` can name (`:field`/`:const`/`:tick`/`:tick-in-cycle`/`:expr` are the servable set, `tick.rs:438-481`). The ruled escape is a **`:ceiling 1` carrier reached without `the`** (survey §3 row "§3.6 carrier ruling"; control-ratio inventory adjudication confirmation 1).

**The carrier.** One `NodeType/INSTITUTION` node, id `carceral-register`. Ceiling 1 is automatic: the driver derives `CardinalityCeilings` from the counts the scenario actually mints (`rust/crates/babylon-tick/src/lib.rs`, the `scenario.node_types` → `NodeType/{member}` map), so a scenario minting exactly one INSTITUTION gives that type ceiling 1 — which also statically bounds every fold over it (Power-of-10 rule 2).

**Reaching it in both directions, with landed constructs only:**
- A **carrier-anchored rule** (its `:field` bindings all in the `institution/*` namespace, so `subject_type_of` resolves INSTITUTION and the rule fires exactly once per tick — `tick.rs:159-182`) reads the class population by folding over the served `nodes` query head: `(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/<numeric-field>))`. `nodes` is served (`evaluator.rs:546` `SERVED_QUERY_HEADS = ["nodes","neighbors","edges"]`; materialized at `query.rs:133,177`), legal as the query operand of `fold`/`exists`/`forall`/`select-max`/`select-min`/`for-each`.
- A **class-anchored rule** reads a carrier field with the poor-man's `the`: `(field-of (select-max (nodes NodeType/INSTITUTION) 1) institution/<field>)`. `(select-max (nodes NodeType/X) <score>)` is a tested shape (`babylon-bsl/tests/r9_chapters.rs:1091-1310`); a constant score is D46-legal (production p1's `(select-max … 1)` precedent), and with exactly one candidate there is no tiebreak to reason about.
- A class-anchored rule **writes** the carrier by the same computed ref: `(update-node (select-max (nodes NodeType/INSTITUTION) 1) institution/<field> (set …))` — the accumulate-into-a-non-self-target lane (D103/D104, `tick.rs:994-1076`).

**Why no edges are needed:** the census is type-scoped, exactly like the frozen `query_nodes(SOCIAL_CLASS)` loop. Do **not** introduce a census edge type; `nodes` makes it unnecessary and an edge-scoped census would silently miss any class lacking the edge.

> **RISK — the two load-bearing shapes are SERVED but not yet exercised by any landed pack, so Task 1 must spike them before Task 2 depends on them.** The only `(fold …)` call in the entire landed content estate is over `neighbors` (`territory.bsl:157-160`), and every landed `select-max` is over `neighbors` too (`production.bsl:172-175`). `nodes` is in `SERVED_QUERY_HEADS` (`evaluator.rs:546`) and materialized at `query.rs:133,177`; `(select-max (nodes NodeType/X) …)` is exercised in `babylon-bsl/tests/r9_chapters.rs:1091-1310`; `field-of`'s first operand accepts a `select-max`/`select-min` return (`evaluator.rs:1277-1301`). That is capability, not content precedent. **Task 1 Step 5 lands a throwaway spike rule proving both shapes load and evaluate against the real driver.** If either refuses, the fallback is the incidence-edge design (seed one edge from the carrier to every class and use `neighbors`) — strictly worse (it adds a seeding obligation and an edge-type choice) and it must be recorded as such, not adopted silently.

**The census reformulation — the plan's headline design finding.** Both inventories recommend abandoning `enum SocialRole` for an int-ordinal encoding, because `field-of` on an enum-declared field was refused at LOAD (register row D102). **That recommendation must not be followed, and its premise has moved:** D102 is now **discharged** — `field-of` over an `:enum-type` field typechecks and evaluates (`rule_pipeline.rs:294-308`); only two enum refusals survive (`E-TYPE-016`, an enum `field-of` used as a `select-max`/`select-min` **score**, `typecheck.rs:246-266, 464-495`; and the static half of `E-EVAL-042`, `add`/`sub`/`scale` targeting an enum field, `typecheck.rs:324-357`). Landed content already declares `(deffield social-class/role enum SocialRole)` (`production-conformance.bscn:109-110`) and reads it via a `:field` binding on `self` with an enum-ref equality (`production.bsl:167,183`). Keep the enum.

The constraint that **actually** binds the census is a different one, and it is absolute:

> **`field_ref_for` reduces a fold body (and a fold's `:weight`) to exactly three shapes** — a bare `<qname>`, a `field-of` accessor, or a nested `fold` — and refuses everything else, **including arithmetic and including an `if`-based role filter**, as `compound_fold_error` (`rule_pipeline.rs:624-670, 678-742, 764-773`), unconditionally on every rule at load (`:291`). `E-TYPE-044` separately refuses an enum-typed fold body for every op except `count` (`typecheck.rs:145-163`).

Two frozen computations therefore cannot be folds at all: the role/active **filter** (`if`-gated) and `sum(population × organization)` (a product). Landed content states this in its own words — `production.bsl:129-148` (D138): *"refuses anything else, including an `if`-based role filter — which is WHY a naive territory-side fold … reading a neighbour's `role`/`population` directly could never load"*; `territory.bsl:140-156` records the same rejection for a per-edge product, which had to move outside the fold.

So: every role/`active` filter and the `pop × org` product live in the **class-anchored** rule (reading `role` off `self` — legal), which publishes a **numeric per-node census-contribution field**; the carrier's folds read those fields with bare `field-of` bodies. This is Production's p4 reformulation applied to a census. Two consequences to honor in the declarations: **`fold sum` is extensive-only** (`E-TYPE-041` on an intensive field, `typecheck.rs:165-176`), so every folded census field is declared `extensive`; and `fold mean`'s `:weight` operand is **rejected as the alternative route** to `avg_organization` — it would compute Σ(w·x)/Σw inside one opaque reduction, where the frozen code computes Σ(pop·org) first and divides second (`control_ratio.py:84, 171`). The two-published-sums design transcribes that two-step arithmetic exactly and can be checked against the mirror bit for bit; the weighted-mean fold cannot. Recorded as a rejected alternative, not an oversight.

**Declarations (all scenarios).**

```scheme
(defvocabulary NodeType (SOCIAL_CLASS INSTITUTION))
(defenum SocialRole (CORE_BOURGEOISIE PERIPHERY_PROLETARIAT LABOR_ARISTOCRACY PETTY_BOURGEOISIE LUMPENPROLETARIAT COMPRADOR_BOURGEOISIE INTERNAL_PROLETARIAT CARCERAL_ENFORCER))
```

**Class fields** (`social-class/*`) — frozen inputs plus the seven published census contributions:

| field | type | frozen origin |
|---|---|---|
| `role` | `enum SocialRole` | `role` |
| `active` | `int extensive` (0/1 latch) | `active` — no `bool` on the live seed dialect; the landed `production-conformance.bscn:111` precedent |
| `population` | `int extensive` | `population` |
| `wealth` | `real extensive` | `wealth` |
| `subsistence-threshold` | `real extensive` | `subsistence_threshold` |
| `s-bio` | `real extensive` | `s_bio` |
| `s-class` | `real extensive` | `s_class` |
| `organization` | reuse the landed declaration — check `query-lane-e2e.bscn:57` first | `organization` |
| `la-census-population` | `int extensive` | published by `p01`; LA-and-active gated |
| `la-census-wealth` | `real extensive` | published by `p01` |
| `la-approaching-flag` | `int extensive` (0/1) | `la_approaching_death`, `:155-156` |
| `la-dying-flag` | `int extensive` (0/1) | `la_about_to_die`, `:158-159` |
| `enforcer-census-population` | `int extensive` | published by `c01` |
| `prisoner-census-population` | `int extensive` | published by `c01` |
| `prisoner-census-org-weighted` | `real extensive` | published by `c01` — the `pop * org` pre-multiplication |

**Carrier fields** (`institution/*`) — the `persistent_data` state machine plus the published aggregates:

| field | type | frozen origin |
|---|---|---|
| `superwage-crisis-known` | `int` (0/1) | `"_superwage_crisis_tick" is None` — the **III.11 loud-absence encoding**: a companion known-flag, never a sentinel value |
| `superwage-crisis-tick` | `int` | `_superwage_crisis_tick` |
| `decomposition-complete` | `int` (0/1) | `_decomposition_complete` |
| `decomposition-fired-known` | `int` (0/1) | `"_class_decomposition_tick" is None` |
| `decomposition-fire-tick` | `int` | `_class_decomposition_tick` — **the joint-train key** |
| `control-crisis-emitted` | `int` (0/1) | `_control_crisis_emitted` |
| `control-crisis-tick` | `int` | `_control_ratio_crisis_tick` |
| `terminal-decision-emitted` | `int` (0/1) | `_terminal_decision_emitted` |
| `la-population` / `la-wealth` | `int` / `real` | the folded LA census (`:287-288`) |
| `la-approaching-count` / `la-dying-count` | `int` / `int` | folded 0/1 flags; `> 0` means "at least one" |
| `enforcer-pop-gain` / `enforcer-wealth-gain` | `int` / `real` | `:298, :300` |
| `ip-population` / `ip-wealth` | `int` / `real` | `:299, :301` |
| `enforcer-population` / `prisoner-population` / `prisoner-org-weighted` | `int` / `int` / `real` | `:137-138` |

**Declaration facts that bind these tables** (`scenario.rs::load_deffield`, `:899-972`):
- The form is exactly `(deffield <qname> <type> <intensive|extensive>)`, or `(deffield <qname> enum <EnumTypeName>)` — a fixed four-element match.
- The accepted type tokens are **exactly** `int`, `real`, `probability`, `intensity`, `coefficient`, `currency`, `enum`. **`bool` is not accepted** by the scenario loader (though `declarations.rs:648-665` accepts it for standalone `.bsl` content sets — the two readers genuinely disagree, and the seeding one is the one that matters). Hence every latch is an `int` 0/1, on the landed `production.bsl` D-2 precedent.
- **There is no `:optional` and no `:default`.** The III.11 loud-absence encoding above (a companion `*-known` flag rather than a sentinel value) is therefore **mandatory**, not stylistic — there is no optional-field route to fall back on.
- Every folded field must be `extensive` (`fold sum` refuses intensive, `E-TYPE-041`).
- Class-surface amendment 1: an `int` deffield **stores verbatim f64** — it constrains *seeding* only (fractional seeds refuse; fractional writes store exactly). So `enforcer-pop-gain` being `int`-declared does not truncate; the `(floor …)` call is the **only** truncation, exactly as `int()` is in the frozen code.
- `update-node`'s op is the closed four-member set `add | sub | set | scale` (`grammar.rs:721-723`); a fifth symbol is `E-PARSE-015`.

---

## 3. Rule layout (10 rules, 2 packs)

Execution order today is **ascending rule-id byte order** across all loaded packs (`lib.rs`'s `prepare_rules` sort; register row D16/D100). `(anchor :after …)` validates shape only and is inert for ordering until Phase 3 (`mod_anchors.rs:1-13`). Rule ids are therefore chosen so byte order equals intended order, and every same-tick dependency is a deliberate D116 reliance (Production p1→p4 precedent).

### Pack A — `content/rules/decomposition.bsl`, namespace `decomposition/`

| id | subject | does |
|---|---|---|
| `p01-la-census` | SOCIAL_CLASS | publishes `la-census-population`, `la-census-wealth`, `la-approaching-flag`, `la-dying-flag`. Role/active gate lives in the **operand** (the D127 hash-neutral idiom: an inactive or non-LA class fires and writes 0, never a `when` skip, so nothing goes stale across ticks). |
| `p02-superwage-warning` | SOCIAL_CLASS | LA-anchored early warning (`:179-197`): `when` role==LA, active==1, `la-approaching-flag`==1, and carrier `superwage-crisis-known`==0 → `emit EventType/SUPERWAGE_CRISIS (receiver self) (desired-wages 0) (available-pool 0)` **and** write the carrier's `superwage-crisis-known`/`-tick`. |
| `p03-trigger` | INSTITUTION | folds the four LA census fields; evaluates the decision (`:200-208`): fallback `la-dying-count > 0`, or `superwage-crisis-known == 1 and tick >= superwage-crisis-tick + decomposition-delay`; gated on `decomposition-complete == 0` and `la-population > 0`; writes `decomposition-fire-tick = tick`, `decomposition-fired-known = 1`, `decomposition-complete = 1`, and the four transfer amounts (`(floor (* la-population enforcer-fraction))` etc.). |
| `p04-enforcer-intake` | SOCIAL_CLASS | `when` role==CARCERAL_ENFORCER and carrier `decomposition-fire-tick == tick` → `(add …)` population and wealth, `(set 1)` active. **ADDITIVE** (`:327-332`). |
| `p05-ip-intake` | SOCIAL_CLASS | `when` role==INTERNAL_PROLETARIAT and fire-tick==tick → `(set …)` population and wealth, `(set 1)` active. **OVERWRITE** (`:336`). |
| `p06-la-deactivate` | SOCIAL_CLASS | `when` role==LA, active==1, fire-tick==tick → `(set 0)` active **only** (never zero wealth/population, `:339`) + `emit EventType/CLASS_DECOMPOSITION (source-class self) …` with the flattened payload. |

`(intrinsic floor :params (real) :returns int :cost 5)` is declared **once, in Pack A only** — a duplicate declaration refuses the whole load (`floor_intrinsic_e2e.rs:137-138`), and the joint-arc scenario concatenates both pack sources. Pack B needs no intrinsic.

The `fire-tick == tick` idiom is what makes the transfers exact and idempotent: the carrier's latch is written by `p03` **this tick** and read by `p04`–`p06` **this tick** (D116); on every later tick `fire-tick != tick`, so nothing re-fires, and `decomposition-complete` independently gates `p03`.

### Pack B — `content/rules/control-ratio.bsl`, namespace `control-ratio/`

| id | subject | does |
|---|---|---|
| `c01-prisoner-census` | SOCIAL_CLASS | publishes `enforcer-census-population` (role==CARCERAL_ENFORCER && active==1), `prisoner-census-population` and `prisoner-census-org-weighted` (`(or (= role SocialRole/INTERNAL_PROLETARIAT) (= role SocialRole/LUMPENPROLETARIAT))` && active==1). Operand-gated, hash-neutral zeros otherwise. |
| `c02-publish-census` | INSTITUTION | unconditional folds → `enforcer-population`, `prisoner-population`, `prisoner-org-weighted`. |
| `c03-crisis` | INSTITUTION | the readiness gate (`:128-134`), the `prisoner-population == 0` guard (`:141`), `max-controllable = enforcer-population * control-capacity`, the `prisoner-population <= max-controllable` guard (`:150`, **`<=`**), then a **guard-split** `emit EventType/CONTROL_RATIO_CRISIS` (one form with `actual-ratio`, one without — §4 BLOCKER-4) + the two latch writes. |
| `c04-terminal` | INSTITUTION | **the ADR070-reserved branch, verbatim**: gated on `control-crisis-emitted == 1`, `terminal-decision-emitted == 0`, `tick >= control-crisis-tick + terminal-decision-delay`; `avg-organization = prisoner-org-weighted / prisoner-population`; `(guard (>= avg-organization revolution-threshold) (emit … (outcome 1) …))` and `(guard (< avg-organization revolution-threshold) (emit … (outcome 0) …))`; then `terminal-decision-emitted = 1`. |

**Two packs, not one** — because these are two frozen systems at two positions, and the `(anchor :after decomposition)` declaration on Pack B keeps that documentary. The cost is disclosed in §5.

---

## 4. BLOCKERS — flagged, not planned around

**BLOCKER-1 (HARD, scope-reducing): `add-node` is refused at content LOAD.** `DEFERRED_SHAPE_VERBS` = `["add-node","remove-node","add-edge","remove-edge","add-hyperedge","remove-hyperedge"]` (`structural_verbs.rs:1723-1730`); `check_no_deferred_shape_verbs` (`:1759-1776`) refuses any rule containing one, anywhere in the form, called unconditionally at `rule_pipeline.rs:269`. The refusal text names the missing follow-on: a **placeholder-id scheme** for deferring a minting verb. Therefore `_create_target_entity`/`_derive_entity_id` (`decomposition.py:225-261, 36-50`) **cannot port**.

> **The survey's row 11.0 is wrong on this point.** It grades Decomposition "PORTABLE WITH D-RECORDS — none blocking" with a soft "node-identity D-record (`add-node` names are rule-local)", and its own §6 explains why: the Decomposition inventory never read `structural_verbs.rs` (survey §6: "Decomposition @11.0 (add `babylon-bsl` source: `structural_verbs.rs`, `evaluator.rs`, `substrate.rs`)… must land before MB-3 opens"). Task 0 discharges that owed re-read and this row is its first finding.

**Disposition (recommended, no Director gate needed):** the create-on-demand branch is **OMITTED** with a D-record and discharged by a **seeding obligation** — every conformance scenario seeds an inactive `CARCERAL_ENFORCER` and an inactive `INTERNAL_PROLETARIAT` node. This is faithful to the frozen code's own primary path: `_find_entity_by_role(..., include_inactive=True)` finds a seeded inactive target and no minting occurs (`:306, :311-313`); creation is the *canonical-world fallback* only, because the bridged world seeds neither (`:302-305`). Follow-on: **#562 (P29-T5, structural-verb execution surface — §2.8 add/remove node/edge at tick time)**. The D-row must state that a world lacking both targets is **unported behavior**, not equivalent behavior.

**BLOCKER-2 (design-shaping, resolved above): `the` is unserved.** `UNSERVED_EXPRESSION_HEADS` tags `("the", "slice 2")` (`evaluator.rs:523-530`); its singleton guard `E-LOAD-043` exists only at load (`manifest.rs:51-53, 100-104`) and **no landed content declares a `manifest` at all**. Hence the `(select-max (nodes NodeType/INSTITUTION) 1)` idiom of §2. Rider: **#572** (the `the`-accessor disposition). Not blocking; recorded so the pack can be simplified when `the` lands.

**BLOCKER-3 (provable-absence, resolved): the event-history read has no BSL lane.** `services.event_bus.get_history()` (`:167-171`) — cross-tick event read-back is the "no lane at all" family (survey §3). It is **unreachable on the ported estate**: the only other `SUPERWAGE_CRISIS` producer is `ImperialRentSystem` @9.0, which is unported and blocked (D35/D65 + the acquiescence sigmoid). The live behavior is entirely the self-emitted early-warning latch (`:179-197`), which needs no history read. Port the latch path; D-record the omission in the Metabolism-D-2 "provably absent" class, with the explicit re-open trigger: **when ImperialRent @9.0 ports, this pack owes a producer-written-field handoff** (the Q16 shape the gap analysis names).

**BLOCKER-4 (representation): `float("inf")` and division by zero.** `:185`'s zero-enforcer branch is genuinely reachable — with `enforcer_pop == 0`, `max_controllable == 0`, so any `prisoner_pop > 0` clears the `<=` guard and the crisis fires with `actual_ratio = inf`. BSL cannot represent it: a non-finite result is `E-EVAL-014` and `x/0` is `E-EVAL-012`. **Disposition:** guard-split the emit in `c03` — `(guard (> enforcer-population 0) (emit … (actual-ratio …) (control-ratio …) …))` and `(guard (= enforcer-population 0) (emit … <same payload minus the two ratio keys> …))`. D-record the divergence precisely: the frozen payload carries `inf`; the ported payload **omits the key**, which is loud absence, not a fabricated number. A dedicated `control-ratio-zero-enforcer-conformance.bscn` makes the branch mutation-provable.

**BLOCKER-5 (encoding): string payload values.** `outcome` ("revolution"/"genocide"), `trigger_event`, and every `narrative_hint` are strings. `<payload-item>` is `(<symbol> <expr>)` (`structural_verbs.rs:1392-1410`) and nothing in `emit` restricts the value's type — it may be `Int`, `Real`, `Currency`, `Ratio`, `Bool`, an `Enum` member, or a `NodeRef`/`EdgeRef`/`HyperedgeRef` — **but there is no `Str` variant in `Value` at all**, and string literals are barred from expression position at load (`GrammarError::StringInExpressionPosition`). So a string payload value is not merely awkward; it is unrepresentable. **Disposition:** drop the narrative hints and `trigger_event` (AI-narration surface, not engine state) and encode `outcome` as a numeric key `(outcome 1)` = revolution / `(outcome 0)` = genocide, mapping recorded in the D-row and pinned by a test. Two alternatives exist and are **rejected with reasons**: a `Bool` payload value (available, but it silently discards the fact that the frozen datum is a two-valued *name*, and reads as a predicate rather than an outcome), and a new `(defenum TerminalOutcome (REVOLUTION GENOCIDE))` (the most faithful shape, but minting a new enum on the ADR070-reserved surface is exactly what this train must not do unilaterally — **note it in the D-row as the shape to adopt if the Director rules the naming**). Nested payload dicts flatten (`population-transferred-to-enforcer`, …), the flattening also D-recorded.

**NOT a blocker — corrections to the inventories, to be recorded in Task 0's dossier:**
- **D102 is DISCHARGED** — `field-of` over an enum field now typechecks and evaluates (`rule_pipeline.rs:294-308`); the two surviving enum refusals (`E-TYPE-016` score, `E-EVAL-042` arithmetic) do not touch this train. The int-ordinal `role` workaround both inventories recommend must be **rejected**: it is unnecessary, and it would contradict landed content (`production-conformance.bscn:110`). The constraint that actually shapes the census is `field_ref_for`'s compound-body refusal (D138) plus `E-TYPE-044`, not D102 — §2 states it precisely.
- **Events ARE observable and pinnable today.** Both inventories say `emit` is "unpinnable pending WS1 (#502)". That is stale: `run_once_into(SCENARIO, RULE, &mut graph, &mut sink)` exposes a `CollectingSink` whose `events: Vec<(String, Vec<(String, Value)>)>` is asserted key-by-key in landed tests (`dispossession_conformance.rs:139-200`). Every one of this train's four events is directly testable.
- **`EventType` needs no declaration.** The kind is registered for `emit`'s check but stays **inert when a scenario declares no `EventType` vocabulary** (`lib.rs:563-592`), and no landed `.bscn` declares one. So `SUPERWAGE_CRISIS`, `CLASS_DECOMPOSITION`, `CONTROL_RATIO_CRISIS`, `TERMINAL_DECISION` emit with **zero** Rust changes.
- **No Director-reserved value is missing and no carrier is absent.** Every coefficient this train needs already exists in `CarceralDefines` with a shipped value. The one un-backed literal is the bare `2` at `:155`, which becomes a declared `defconst` with a "no defines backing" D-row — a transcription note, not an escalation.
- **`manifest`, ceilings and `the` are test-corpus-only.** The scenario loader accepts exactly `defenum | defvocabulary | deffield | defconst | node | edge | edge-attr` (`scenario.rs:378-437`); a `manifest` form is refused, `check_rule_against_manifest` is exported but never called from the pipeline, and ceilings come from minted counts (`babylon-tick/src/lib.rs:187-217`). Nothing in this train may declare a manifest.

**Type traps the implementer will hit (verify at Task 1, not at Task 6):**
- `Int ÷ Int` is a **loud error** — "truncation is never implicit (§3.2) and §3.3 promotes `Int` only in a binary64 expression" (`evaluator.rs` module doc, lines 35-37). `avg-organization = prisoner-org-weighted / prisoner-population` is Real ÷ (field-sourced) — fine, but confirm the fold's result type.
- `:tick` binds the driver's tick number; every comparison mixing it with a field-sourced value must be checked once, up front.
- `if`'s two branches must share one static type (`E-TYPE-020`) and `if` takes **exactly three operands — the else branch is mandatory** (`grammar.rs:649`). Hence the landed `(- 0 0c)` / `(- 1 0c)` promotion idiom (`dispossession.bsl`, `lifecycle.bsl:284`). Use it; do not invent a promotion.
- The fold element name is the implicit `it` in landed practice (`territory.bsl:158`); the grammar also admits an explicit element name (`(fold <op> <query> <elem>? <expr> (:weight <expr>)?)`, `grammar.rs:797-816`). Copy landed practice.
- `exists` appears in landed content with both one operand (`production.bsl:172`) and two (`(exists (neighbors …) #t)`, `territory.bsl:157`). Copy whichever the neighbouring pack uses; do not mix.
- `E-LEX-023` caps literals at 9 fractional digits; `E-LEX-024` bounds suffixed literals (`0.15c`, `0.85c`, `0.5c` are all inside the coefficient domain). Bare `52`/`4`/`1`/`2` are unsuffixed Int consts (the Metabolism escape-hatch class).

---

## 5. Byte-order hazard, stated explicitly

`control-ratio/…` sorts **before** `decomposition/…`, so Pack B's rules execute **before** Pack A's within a tick — inverting the frozen @11.0-then-@12.0 order. This is register row **D100**'s already-disclosed cross-pack hazard ("inverts the frozen engine's own tick order on 4 of 6 shipped pack pairs").

**It is benign here, and the plan must prove it rather than assume it:**
- The only cross-pack datum is `institution/decomposition-fire-tick`. Pack B reads it behind the gate `tick >= fire-tick + control-ratio-delay`. With the shipped `control_ratio_delay = 52`, the latch is 52 ticks old when Pack B first acts, so reading it one rule-order earlier changes nothing.
- Pack B's own census chain is self-contained (`c01 < c02 < c03 < c04`), so no Pack-A write feeds it.
- **The hazard is real only if a scenario sets `control-ratio-delay = 0` while `decomposition-fire-tick` is written the same tick.** Hence the Global Constraint: a zero-delay companion must **seed** the fire tick, never rely on Pack A writing it that tick. Task 8's D-row records this as a content-authoring constraint, and Task 8 adds a **test that fails if the constraint is violated** (a zero-delay scenario whose fire-tick is unseeded must produce no crisis on the firing tick).

If a reviewer prefers one total order over the disclosure, the fallback is a single pack under `decomposition/` with zero-padded ids `p01..p10` — mechanically simpler, but it mislabels four ControlRatio rules as belonging to the Decomposition system. **Recommendation: two packs + the D-row + the constraint test.**

---

## File Structure

| File | Responsibility |
|---|---|
| Create `reports/decomposition-controlratio-bsl-surface-facts-2026-08-17.md` | Task 0's dossier — discharges the two owed INADEQUATE-COVERAGE re-reads (survey §6) |
| Modify `rust/crates/babylon-tick/src/lib.rs` | Two registration strings in `prepare_rules`' system `HashSet` |
| Create `rust/crates/babylon-tick/content/rules/decomposition.bsl` | Pack A, 6 rules + `(intrinsic floor …)` + the file-local `D-N` header block |
| Create `rust/crates/babylon-tick/content/rules/control-ratio.bsl` | Pack B, 4 rules + its own `D-N` header block |
| Create `content/scenarios/decomposition-conformance.bscn` + `decomposition_conformance.py` | The fallback-trigger world (fires at tick 1) + frozen mirror |
| Create `content/scenarios/decomposition-delay-conformance.bscn` | The delay path (early warning tick 1, decomposition tick 53) |
| Create `content/scenarios/control-ratio-conformance.bscn` + `control_ratio_conformance.py` | Genocide at tick 1 (zero delays, seeded fire tick) + frozen mirror |
| Create `content/scenarios/control-ratio-revolution-conformance.bscn` | The revolution branch (ADR070-reserved) |
| Create `content/scenarios/control-ratio-within-capacity-conformance.bscn` | The negative vector — `<=` boundary, no crisis |
| Create `content/scenarios/control-ratio-zero-enforcer-conformance.bscn` | BLOCKER-4's guard-split branch |
| Create `content/scenarios/carceral-arc-conformance.bscn` + `carceral_arc_conformance.py` | The full five-phase arc on shipped delays, both packs |
| Create `rust/crates/babylon-tick/tests/decomposition_conformance.rs` | Pack A conformance + mutation vectors |
| Create `rust/crates/babylon-tick/tests/control_ratio_conformance.rs` | Pack B conformance + branch vectors |
| Create `rust/crates/babylon-tick/tests/carceral_arc_conformance.rs` | The multi-tick arc via `TickSession` |
| Modify `rust/crates/babylon-tick/tests/tick_goldens.rs` | Additive pins (7 new scenario/rule pairs); 8 existing pins untouched |
| Modify `docs/reference/bsl-language.rst` | Register rows (next free — the tail was **D155/D156** at plan time; re-check) |
| Create `ai/decisions/ADR209_decomposition_controlratio_port_handoff.yaml` + `index.yaml` row | Handoff record (index ended at **ADR208**; verify) |
| Modify both `reports/port-inventories/*-2026-08-12.md` | Post-train UPDATE blocks (verdict PORTED-with-omission; corrections cross-referenced) |
| Modify `ai/state.yaml` | Closing entry |

---

### Task 0: The two owed re-reads — surface-facts dossier

The survey **forbids opening MB-3 without these** ("Both must land before MB-3 opens", §6). They are cheap and narrow, and this plan already contains most of the answers — Task 0 verifies them at the byte and records the corrections so the implementer tasks argue from a checked dossier.

**Files:** Create `reports/decomposition-controlratio-bsl-surface-facts-2026-08-17.md`.

**Interfaces:** Produces the §10-style CORRECTIONS section every later task cites when it contradicts a 2026-08-12 inventory row.

- [ ] **Step 1: Decomposition's owed sources** — read `babylon-bsl/src/structural_verbs.rs` (especially `DEFERRED_SHAPE_VERBS` at `:1723` and `check_no_deferred_shape_verbs` at `:1759`), `evaluator.rs` (`SERVED_QUERY_HEADS` `:546`, `UNSERVED_EXPRESSION_HEADS` `:523`, the module doc's `Int ÷ Int` note), `babylon-graph/src/substrate.rs`. Record verbatim: the six refused verbs, the refusal text, the served query heads.
- [ ] **Step 2: ControlRatio's owed sources** — `typecheck.rs:246-289` + `rule_pipeline.rs:293-301` (the D102 gate), `evaluator.rs:1274-1292, 1315-1320, 1594-1632` (`field_of_node`, the write catch-all).
- [ ] **Step 3: Write the dossier** with these sections: (1) confirmed inventory findings; (2) **CORRECTIONS** — BLOCKER-1 (`add-node` refused at load; the survey's row 11.0 amended), D102 **discharged** (int-ordinal `role` **rejected**; the binding law is `field_ref_for`'s compound-body refusal + `E-TYPE-044`), events-are-observable (`CollectingSink`), `EventType`-inert-when-undeclared, `the`-unserved-so-`select-max`-over-`nodes`, `manifest`-is-test-corpus-only; (3) the type-trap list from §4; (4) the byte-order analysis of §5; (5) the `deffield` surface as read from `scenario.rs:899-972` — the seven accepted type tokens, no `bool`, no `:optional`/`:default`, and the `declarations.rs:648-665` disagreement recorded as a fact about two readers, not a usable route.
- [ ] **Step 4: Verify the two registration symbol names load.** Confirm the rule-id first segment admits a hyphen (`control-ratio`); if it does not, the dossier fixes the namespace as `controlratio` and every later task uses that spelling.
- [ ] **Step 5: Commit** `docs(port): decomposition + control-ratio BSL surface-facts dossier (the two owed re-reads, survey §6)`.

### Task 1: Registration + the carrier/scenario ceremony (PR A)

**Files:**
- Modify: `rust/crates/babylon-tick/src/lib.rs` (the `prepare_rules` system `HashSet`)
- Create: `rust/crates/babylon-tick/content/scenarios/decomposition-conformance.bscn`
- Create: `rust/crates/babylon-tick/content/scenarios/decomposition_conformance.py`
- Test: `rust/crates/babylon-tick/tests/decomposition_conformance.rs`

**Interfaces:** Produces the node ids, the carrier field roster, and the mirror numbers every later task asserts against.

- [ ] **Step 1: Failing load-smoke test** — `decomposition_conformance.rs` with `const SCENARIO: &str = include_str!(...)` and a test `scenario_and_empty_pack_load` calling the real loader with an empty rule source. Expected: FAIL (unregistered system / missing scenario). Follow `production_conformance.rs:75-94`'s own registration probe.
- [ ] **Step 2: Register both systems** — add `"decomposition".to_owned()` and `"control-ratio".to_owned()` (spelling per Task 0 Step 4) to the `HashSet`, each with a comment citing its Material Base position (`@11.0`, `@12.0`) exactly as the landed `"dispossession"` / `"production"` rows do. Registering both now keeps PR B free of Rust source edits.
- [ ] **Step 3: Write `decomposition-conformance.bscn`** — the **fallback-trigger** world, so decomposition fires at tick 1 and the single-tick golden is meaningful. Declarations per §2 (all class fields, all carrier fields, all eight `carceral/*` defconsts at shipped values). Declaration order is NodeId order — declare in this order, and never renumber when extending (`production-conformance.bscn:207-209`):

  | node | type | role | active | population | wealth | subsistence | s-bio | s-class | organization | notes |
  |---|---|---|---|---|---|---|---|---|---|---|
  | `la-dying` | SOCIAL_CLASS | LABOR_ARISTOCRACY | 1 | 1000 | 400 | 500 | 5 | 5 | 0 | `wealth < subsistence` → `la-dying-flag` 1, the fallback vector |
  | `enforcer-seed` | SOCIAL_CLASS | CARCERAL_ENFORCER | 0 | 20 | 100 | 0 | 0 | 0 | 0 | seeded **inactive** with NON-ZERO population/wealth — makes the ADDITIVE write provable (a `set` mutation must flip a test) |
  | `ip-seed` | SOCIAL_CLASS | INTERNAL_PROLETARIAT | 0 | 77 | 33 | 0 | 0 | 0 | 0 | seeded inactive with non-zero values — makes the OVERWRITE provable (an `add` mutation must flip a test) |
  | `lumpen` | SOCIAL_CLASS | LUMPENPROLETARIAT | 1 | 200 | 10 | 0 | 0 | 0 | 0.2 | prisoner census's second role; untouched by Pack A |
  | `bourgeois` | SOCIAL_CLASS | CORE_BOURGEOISIE | 1 | 10 | 9000 | 0 | 0 | 0 | 0 | the non-participant vector — every published census field must stay 0 |
  | `carceral-register` | INSTITUTION | — | — | — | — | — | — | — | — | the carrier; every latch seeded 0, every aggregate 0 |

  Seed **every** declared field on **every** node of its namespace (the no-defaults law). Fractional seeds on `int` fields refuse — keep `population`/latches integral.
- [ ] **Step 4: Write the frozen mirror** `decomposition_conformance.py` — same genre as `dispossession_conformance.py`: import the frozen `DecompositionSystem`, build the identical graph node-for-node in the same order, run one `step()` with a `TickContext` whose `persistent_data` starts empty, print post-tick `population`/`wealth`/`active` per node plus the event bus history with full payloads. Header carries the ADR183 disclaimer verbatim from the dispossession mirror ("structure and ordering contract, not a correctness oracle"). Run it and paste its stdout **verbatim, dated** into the Rust test's doc comment (`dispossession_conformance.rs:16-39` idiom).
- [ ] **Step 5: THE SPIKE — prove the two unexercised shapes, before any rule depends on them.** Add a throwaway spike rule (deleted at the end of this step, its verdict recorded in the scenario header) that (a) folds over `(nodes NodeType/SOCIAL_CLASS)` with a bare `field-of` body, and (b) reads a carrier field through `(field-of (select-max (nodes NodeType/INSTITUTION) 1) institution/…)`, and run it through the real driver. Both are SERVED but content-unprecedented (§2's RISK box). **If either refuses: STOP, record the refusal text, and switch to the incidence-edge fallback before Task 2** — do not attempt a workaround inside a later task.
- [ ] **Step 6: Load-smoke green** + a `defenum` ordinal-parity test mirroring the mint's (class-surface amendment 7). Also pin, by name in the test header, that this is the first content to use `NodeType/INSTITUTION` as a singleton carrier and the first to fold over `nodes`.
- [ ] **Step 7: Commit** `test(tick): decomposition conformance scenario + frozen mirror + carceral system registration`.

**Gate:** six cargo legs; the 8 existing golden pins byte-identical.

### Task 2: Pack A rules `p01` + `p02` — the LA census and the early warning

**Files:**
- Create: `rust/crates/babylon-tick/content/rules/decomposition.bsl` (header + `p01` + `p02` + the `floor` intrinsic declaration)
- Test: extend `decomposition_conformance.rs`

**Interfaces:** Produces the four published LA census fields `p03` folds, and the `superwage-crisis-*` latch.

- [ ] **Step 1: Failing tests** — `p01_publishes_the_la_census_only_for_the_active_la` (la-dying's four fields carry its own values; enforcer/ip/lumpen/bourgeois all 0 — the hash-neutral vector); `p01_flags_the_dying_la` (`la-dying-flag` 1, `la-approaching-flag` 1 — `wealth < subsistence` implies `wealth < subsistence + 2·consumption`); `p02_emits_superwage_crisis_once_with_the_receiver_ref` (one `SUPERWAGE_CRISIS`, payload `receiver` == `Value::NodeRef(la-dying)`, `desired-wages` 0.0, `available-pool` 0.0); `p02_latches_the_crisis_tick_on_the_carrier`.
- [ ] **Step 2: Write the pack header** — the file-local `D-N` block (reserve rows for: the carrier reformulation, the omitted `add-node` branch, the omitted history read, the payload flattening, the dropped narrative hints, the D116 byte-order map, the bare-`2` literal, the docstring drift, the non-conservation). Header also carries the byte-order map `p01 → p02 → p03 → p04 → p05 → p06` with each same-tick dependency named.
- [ ] **Step 3: Write `p01-la-census`** — bindings: `role`, `active`, `population`, `wealth`, `subsistence-threshold`, `s-bio`, `s-class` (all `:field` on self), `approaching-multiple :const`, then `consumption :expr (+ s-bio s-class)`, `approaching-bound :expr (+ subsistence-threshold (* approaching-multiple consumption))`, and four gated operands using the D127 hash-neutral idiom (`(if (and (= role SocialRole/LABOR_ARISTOCRACY) (= active 1)) <value> (- 0 0c))`). Effects: four `(update-node self … (set …))`. **No `when` clause** — the rule fires for every class and writes 0 for non-LAs, which is what keeps the census fresh instead of stale.
- [ ] **Step 4: Write `p02-superwage-warning`** — carrier read via `(field-of (select-max (nodes NodeType/INSTITUTION) 1) institution/superwage-crisis-known)`; `when` conjunction per §3; effects: `emit` + two `(update-node (select-max (nodes NodeType/INSTITUTION) 1) institution/… (set …))`. Transcribe the frozen order: emit first, then the latch (`:180-197`).
- [ ] **Step 5: Tests green; pin the exact bits** measured from the run and cross-checked against the mirror printout.
- [ ] **Step 6: Mutation** — flip `p01`'s role gate to `PETTY_BOURGEOISIE`: `p01_publishes_the_la_census_only_for_the_active_la` flips red. Change the `2` to `1` in the approaching bound: a dedicated approaching-boundary assertion (added in Task 4's delay scenario) must flip — if no test flips now, record that and make it flip in Task 4. Restore byte-identical.
- [ ] **Step 7: Six legs + commit** `feat(tick): decomposition p01/p02 — LA census publication + the superwage early warning`.

### Task 3: Pack A rule `p03` — the carrier trigger and the transfer amounts

**Files:**
- Modify: `rust/crates/babylon-tick/content/rules/decomposition.bsl`
- Test: extend `decomposition_conformance.rs`

**Interfaces:** Produces `decomposition-fire-tick` / `-fired-known` / `decomposition-complete` and the four transfer amounts — every one of them consumed by `p04`–`p06` and by Pack B.

- [ ] **Step 1: Failing tests** — `p03_folds_the_la_census_into_the_carrier` (`la-population` 1000, `la-wealth` 400.0, `la-dying-count` 1, `la-approaching-count` 1); `p03_fires_on_the_fallback_trigger_without_any_delay` (`decomposition-fire-tick` == 1, `-fired-known` == 1, `decomposition-complete` == 1); `p03_computes_the_frozen_splits` — `enforcer-pop-gain` == `floor(1000*0.15)` == 150, `ip-population` == `floor(1000*0.85)` == 850, `enforcer-wealth-gain` == 400·0.15, `ip-wealth` == 400·0.85, each asserted **bit-exact** against the mirror's `repr` output (`.to_bits()` idiom, `production_conformance.rs:226-229`); `p03_is_idempotent_across_two_ticks` (a two-tick `TickSession` run: the second tick must not move `decomposition-fire-tick`).
- [ ] **Step 2: Write `p03-trigger`** — subject INSTITUTION via `:field` bindings on `institution/decomposition-complete`, `institution/superwage-crisis-known`, `institution/superwage-crisis-tick`; `tick :tick`; the four census folds as `:expr` bindings (`(fold sum (nodes NodeType/SOCIAL_CLASS) (field-of it social-class/la-census-population))` etc. — **bare accessors only**); the two `:const` delays/fractions; `fire? :expr` transcribing `:200-208` (fallback OR delay-elapsed) conjoined with `decomposition-complete == 0` and `la-population > 0` (`:290-291`). Effects: the three latch writes plus the four amount writes, all inside one `(guard fire? …)`.
- [ ] **Step 3: Tests green; pin bits.** Verify the fold result type against the `Int ÷ Int` trap before writing any quotient.
- [ ] **Step 4: Mutation** — change `(> la-population 0)` to `(>= la-population 0)`: add a zero-population LA fixture in Task 4's delay scenario if nothing flips here, and record which test flips. Change the fallback disjunct to a conjunct: `p03_fires_on_the_fallback_trigger_without_any_delay` flips red. Restore byte-identical.
- [ ] **Step 5: Six legs + commit** `feat(tick): decomposition p03 — the carrier trigger, the delay gate and the frozen split arithmetic`.

### Task 4: Pack A rules `p04`–`p06` + the delay scenario + Pack A goldens

**Files:**
- Modify: `rust/crates/babylon-tick/content/rules/decomposition.bsl`
- Create: `content/scenarios/decomposition-delay-conformance.bscn`
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs`, `tests/decomposition_conformance.rs`

**Interfaces:** Closes Pack A. Produces the post-decomposition class state Pack B's fixtures mirror.

- [ ] **Step 1: Failing tests** — `p04_adds_to_the_seeded_enforcer` (population 20+150 == 170, wealth 100+60.0, active 1 — **additive**); `p05_overwrites_the_seeded_internal_proletariat` (population == 850 exactly, **not** 77+850; wealth == 340.0, active 1); `p06_deactivates_the_la_without_zeroing_it` (active 0, population still 1000, wealth still 400.0 — the non-conservation vector); `p06_emits_class_decomposition_with_the_flattened_payload` (payload asserted key-by-key: `source-class` NodeRef, `source-population`, `source-wealth`, `enforcer-fraction`, `proletariat-fraction`, `population-transferred-to-enforcer`, `population-transferred-to-proletariat`, `wealth-transferred-to-enforcer`, `wealth-transferred-to-proletariat`); `the_bourgeois_class_is_untouched_by_the_whole_pack`; and on the delay scenario: `the_delay_path_emits_the_warning_at_tick_1_and_decomposes_at_tick_53` (a 54-tick `TickSession`), `the_delay_path_does_not_decompose_at_tick_52` (the exact boundary, `:207`).
- [ ] **Step 2: Write `p04`/`p05`/`p06`** per §3. Each reads its amount from the carrier via the `select-max`+`field-of` idiom and gates on `fire-tick == tick`. `p06`'s emit reads the amounts from the carrier and `self` for the source ref.
- [ ] **Step 3: Write `decomposition-delay-conformance.bscn`** — identical to the primary world except `la-approaching`'s `wealth` sits **above** `subsistence` but **below** `subsistence + 2·consumption` (so `la-approaching-flag` 1, `la-dying-flag` 0) and the shipped `decomposition-delay 52` governs. This scenario is what makes the bare-`2` literal and the `>=`-delay boundary mutation-provable (Task 2 Step 6, Task 3 Step 4).
- [ ] **Step 4: Add the two Pack A golden pins** to `tick_goldens.rs` — `decomposition_conformance` and `decomposition_delay_conformance`, each pinning `hex(before)`, `hex(after)` and `report.fired` with the firing arithmetic **verified, not trusted**. **Measured, never derived**: run `run_once` and read the printed value back. The 8 pre-existing pins stay untouched.
- [ ] **Step 5: Mutation** — swap `p04`'s `(add …)` for `(set …)`: `p04_adds_to_the_seeded_enforcer` flips. Swap `p05`'s `(set …)` for `(add …)`: `p05_overwrites…` flips. Add a wealth-zeroing effect to `p06`: `p06_deactivates_the_la_without_zeroing_it` flips. Restore byte-identical each time.
- [ ] **Step 6: Six legs + commit** `feat(tick): decomposition p04-p06 — the transfers, the deactivation and the CLASS_DECOMPOSITION emit`.
- [ ] **Step 7: Open PR A** `feat/decomposition-port-bsl`, Tasks 1-4 (four commits). Review lens: (a) transcription fidelity against §1.1's table, line by line; (b) the carrier idiom and the D116 map. Harvest the Copilot review; merge via `mise run pr:merge`.

### Task 5: Pack B registration ceremony — four scenarios, the mirror, `c01`/`c02` (PR B)

**Files:**
- Create: `content/scenarios/control-ratio-conformance.bscn`, `control-ratio-revolution-conformance.bscn`, `control-ratio-within-capacity-conformance.bscn`, `control-ratio-zero-enforcer-conformance.bscn`
- Create: `content/scenarios/control_ratio_conformance.py`
- Create: `rust/crates/babylon-tick/content/rules/control-ratio.bsl` (header + `c01` + `c02`)
- Test: `rust/crates/babylon-tick/tests/control_ratio_conformance.rs`

Branch off **merged dev** (never stack on PR A). No Rust source edits (Task 1 registered both systems).

- [ ] **Step 1: Failing tests** — `c01_publishes_the_two_prisoner_roles_and_the_enforcer_count` (an INTERNAL_PROLETARIAT **and** a LUMPENPROLETARIAT node both contribute; an inactive one contributes 0; a bourgeois contributes 0); `c01_premultiplies_population_by_organization` (bit-exact); `c02_publishes_the_three_aggregates_unconditionally` (they are written even when the readiness gate will fail — the state-surface widening this train records).
- [ ] **Step 2: Write the four scenarios.** All four seed a **post-decomposition** carrier state directly (`decomposition-fire-tick 0`, `decomposition-fired-known 1`, `decomposition-complete 1`) so Pack B is exercised without Pack A — which is also why a zero delay is safe here (§5). Differences: the primary sets `carceral/control-ratio-delay 0` and `carceral/terminal-decision-delay 0` with prisoner `organization 0.2` (**genocide** at tick 1); the revolution scenario is identical but `organization 0.6`; the within-capacity scenario sets prisoner population **at or below** `enforcer-population × 4` (the `<=` boundary — no crisis); the zero-enforcer scenario seeds enforcer population 0 with prisoners > 0 (BLOCKER-4's branch). Each scenario's header states which single constant it makes mutation-provable.
- [ ] **Step 3: Write the frozen mirror** `control_ratio_conformance.py` — build each of the four worlds, pre-seed `persistent_data["_class_decomposition_tick"] = 0` and the matching delays via a `GameDefines` override, run one `step()`, print the event history with full payloads. Paste stdout verbatim + dated into the Rust test's doc comment.
- [ ] **Step 4: Write the pack header + `c01`/`c02`.** Header carries Pack B's file-local `D-N` block (the two-role prisoner set, the unconditional census publication, the `<=` boundary, the guard-split emit, the numeric `outcome` encoding, the cross-pack byte-order disclosure) and the `c01 → c02 → c03 → c04` map. `c01` uses the D127 operand gate with `(or (= role …INTERNAL_PROLETARIAT) (= role …LUMPENPROLETARIAT))`; `c02` folds bare accessors.
- [ ] **Step 5: Tests green; pin bits. Six legs + commit** `test(tick): control-ratio conformance scenarios + frozen mirror` / `feat(tick): control-ratio c01/c02 — the prisoner census and its carrier publication`.

### Task 6: Pack B rule `c03` — the crisis gate and the guard-split emit

**Files:**
- Modify: `content/rules/control-ratio.bsl`
- Test: extend `control_ratio_conformance.rs`

- [ ] **Step 1: Failing tests** — `c03_emits_the_crisis_when_prisoners_exceed_capacity` (payload key-by-key: `enforcer-population`, `prisoner-population`, `control-capacity`, `max-controllable`, `actual-ratio`, `over-capacity-by`, `control-ratio` **duplicating** `actual-ratio` verbatim, `capacity-threshold`); `c03_does_not_emit_at_or_below_capacity` (the within-capacity scenario — the `<=` mutation killer: flipping `<=` to `<` must flip this test); `c03_omits_the_ratio_keys_when_there_are_no_enforcers` (the zero-enforcer scenario — the payload has the other keys and **neither** ratio key, and the tick does not abort); `c03_latches_once` (a two-tick run emits exactly one crisis); `c03_stays_silent_before_the_readiness_gate` (a fifth ad-hoc fixture with `decomposition-fired-known 0`).
- [ ] **Step 2: Write `c03-crisis`** per §3, with the two guard-split emit forms of BLOCKER-4. Transcribe `:150`'s `<=` exactly; transcribe the duplicated `control_ratio` key exactly (a defect, port-as-is).
- [ ] **Step 3: Mutation** — `<=` → `<` flips `c03_does_not_emit_at_or_below_capacity`; dropping the `= enforcer-population 0` guard makes the zero-enforcer scenario abort the tick (`E-EVAL-012`), which the test must show as a named failure rather than a silent pass. Restore byte-identical.
- [ ] **Step 4: Six legs + commit** `feat(tick): control-ratio c03 — the crisis gate, the capacity boundary and the no-enforcer guard split`.

### Task 7: Pack B rule `c04` — the terminal decision (ADR070-RESERVED)

**Files:**
- Modify: `content/rules/control-ratio.bsl`
- Test: extend `control_ratio_conformance.rs`

**This task touches the Director-reserved line.** Transcribe; do not redesign. The rule's `:material-basis` must name the reservation and the P19 cutover explicitly, and the commit body must state that the branch is transcribed verbatim under ADR070.

- [ ] **Step 1: Failing tests** — `c04_routes_to_genocide_below_the_threshold` (primary scenario, `organization 0.2` → `outcome 0`, `avg-organization` bit-exact, `revolution-threshold` 0.5, both populations present); `c04_routes_to_revolution_at_or_above_the_threshold` (revolution scenario, `organization 0.6` → `outcome 1`); `c04_at_exactly_the_threshold_routes_to_revolution` (a sixth fixture with `organization` exactly 0.5 — the `>=` boundary, the frozen MutationKillers' own target); `c04_respects_the_terminal_delay` (on a fixture with `terminal-decision-delay 1`: crisis at tick 1, terminal at tick 2, nothing at tick 1); `c04_emits_once`; `the_avg_organization_is_population_weighted_not_a_bare_mean` (two prisoner classes with different populations and organizations, where the unweighted mean differs — the intensive-aggregation guard).
- [ ] **Step 2: Write `c04-terminal`** per §3: `:field` bindings on the carrier's aggregates and latches, `tick :tick`, `avg-organization :expr (/ prisoner-org-weighted prisoner-population)`, `threshold :const carceral/revolution-threshold`, and two `guard`-split emits differing only in the numeric `outcome`. Then `(update-node self institution/terminal-decision-emitted (set 1))`.
- [ ] **Step 3: Mutation** — `>=` → `>` flips `c04_at_exactly_the_threshold_routes_to_revolution`; swapping the two outcome codes flips both routing tests. Restore byte-identical.
- [ ] **Step 4: Six legs + commit** `feat(tick): control-ratio c04 — the terminal decision, transcribed verbatim under ADR070`.

### Task 8: The joint arc — the full five-phase scenario and all goldens

**Files:**
- Create: `content/scenarios/carceral-arc-conformance.bscn`, `content/scenarios/carceral_arc_conformance.py`
- Create: `rust/crates/babylon-tick/tests/carceral_arc_conformance.rs`
- Modify: `rust/crates/babylon-tick/tests/tick_goldens.rs`

**Interfaces:** The train's acceptance test — the ported analogue of `tests/scenarios/test_carceral_equilibrium.py`'s phase-sequence assertion, and the proof the two packs compose.

- [ ] **Step 1: Failing test** — `the_full_carceral_arc_runs_in_order`: load **both** pack sources concatenated (`floor` declared only in Pack A, §3) against `carceral-arc-conformance.bscn` on the **shipped** delays, advance a `TickSession` past the terminal decision, and assert the event sequence `SUPERWAGE_CRISIS` (tick 1) → `CLASS_DECOMPOSITION` (tick 53) → `CONTROL_RATIO_CRISIS` (tick 105) → `TERMINAL_DECISION` (tick 106) — **derive each tick from the delay arithmetic and then verify it against the mirror; do not trust this plan's numbers.** Plus `the_arc_emits_each_event_exactly_once` and `the_arc_ends_in_genocide_with_no_organization` (the frozen scenario test's own default outcome).
- [ ] **Step 2: Write the arc scenario** — the delay-path LA (approaching, not dying), seeded inactive enforcer and internal proletariat, a lumpen class, a bourgeois class, the carrier with every latch 0. Header records the derived tick schedule and the frozen test it mirrors.
- [ ] **Step 3: Write `carceral_arc_conformance.py`** — the frozen composition oracle: run **both** frozen systems in position order (@11.0 then @12.0) over the same tick range with one shared `persistent_data`, printing every event with its tick. This mirror is what proves the ported cross-pack composition matches the frozen cross-system composition despite §5's rule-order inversion.
- [ ] **Step 4: The byte-order constraint test** — a scenario (or an in-test variant) with `control-ratio-delay 0` and an **unseeded** fire tick, asserting that no `CONTROL_RATIO_CRISIS` fires on the tick decomposition fires. This is the executable form of §5's constraint; without it the constraint is a comment.
- [ ] **Step 5: Add the remaining golden pins** — the four Pack B scenarios plus the arc (5 new pins; 2 landed in Task 4). Every pin measured. The 8 pre-existing pins byte-identical.
- [ ] **Step 6: Six legs + commit** `test(tick): the joint carceral arc — five-phase composition across both packs`.

### Task 9: Records, docs, gates, handoff

**Files:**
- Modify: `docs/reference/bsl-language.rst` (register rows)
- Modify: `content/rules/decomposition.bsl`, `control-ratio.bsl` (finalize the file-local `D-N` blocks)
- Create: `ai/decisions/ADR209_decomposition_controlratio_port_handoff.yaml` + `ai/decisions/index.yaml` row
- Modify: `reports/port-inventories/decomposition-port-phase1-inventory-2026-08-12.md`, `control-ratio-port-phase1-inventory-2026-08-12.md` (UPDATE blocks)
- Modify: `ai/state.yaml`

- [ ] **Step 1: Register rows** (global numbers — re-check the tail first; **D155/D156** at plan time). One row each for:
  1. **`persistent_data` → singleton INSTITUTION carrier** — the reformulation, the `select-max`-over-`nodes` idiom that replaces the unserved `the` (#572), and the loud-absence known-flag encoding instead of sentinel values.
  2. **The `add-node` omission** — `_create_target_entity`/`_derive_entity_id` unported; `DEFERRED_SHAPE_VERBS` refusal cited; the seeding obligation; a world lacking both targets is **unported**, not equivalent; follow-on **#562**.
  3. **The event-history read omitted as provably unreachable** — ImperialRent @9.0 is the only other producer and is unported; the explicit re-open trigger when it ports.
  4. **The census reformulation** — per-node numeric contribution fields + `enum SocialRole` retained; the int-ordinal recommendation in both inventories **rejected**, with D102's discharge (`rule_pipeline.rs:294-308`) as the reason its premise no longer holds; the compound-fold-body law (`field_ref_for`, D138) + `E-TYPE-044` as the constraints that actually force both the filter and the `pop × org` product out of the fold; and `fold mean :weight` recorded as the **rejected alternative** for `avg_organization` (it hides the reduction that the frozen two-step arithmetic must be checked against bit for bit).
  5. **The unconditional census publication** — the frozen system computes the census only past the readiness gate; the port publishes it to carrier fields every tick, widening the observable state surface.
  6. **Payload divergences** — flattened nested dicts, dropped `narrative_hint`s, the numeric `outcome` encoding (1 revolution / 0 genocide), the omitted ratio keys in the zero-enforcer case (`float("inf")` unrepresentable), and the deliberately-preserved duplicate `control_ratio` key.
  7. **The cross-pack byte-order inversion** — D100's class; the delay-protection argument; the authoring constraint and the test that enforces it.
  8. **The four transcribed defects** — docstring drift (30/70 vs 0.15/0.85 and the stale 2.33:1 arithmetic), additive/overwrite asymmetry, LA non-conservation, the bare `2` literal with no defines backing.
  9. **The P19-cutover-pending revolution-vs-genocide row** — *this is the content need the charter names.* It must state: the branch is transcribed verbatim (threshold source, `>=` comparison, both outcomes, the two prisoner roles); **ADR070 / Program 19 rules ControlRatio explicitly LAST in the emergent-class-partition cutover**, so the port deliberately does **not** re-base the branch onto a derived class-cell partition; the transcription is consistent with ADR070's own slots-as-positions ruling; the cutover remains **Director-gated and open**; cite the **2026-08-12 ruling** and its reaffirmation in **ADR208 R29 / C-03**, plus register row 12 of #564.
- [ ] **Step 2: ADR209** — records: the two packs and ten rules; the carrier reformulation; the omitted `add-node` branch and its follow-on; the D102 correction; the RESERVED-LINE transcription statement (verbatim, with the P19 pointer); the byte-order disclosure; gate evidence; and the Task-0 dossier's corrections to the survey's row 11.0. Add the `index.yaml` row.
- [ ] **Step 3: Inventory UPDATE blocks** — verdict **PORTED (with one named omission)** for both; each correction cross-referenced to the Task-0 dossier and to the register rows; the Territory-inventory UPDATE-block pattern.
- [ ] **Step 4: Issue hygiene** — **#566 is CLOSED** (its bookkeeping close, ADR208 R29). File a **fresh implementation issue** under umbrella **#557 (Program 29, Wave A)** titled for this train, linking both PRs, ADR209, and the two follow-ons (#562 for the mint branch, #572 for `the`); close it with evidence. Update **#578**'s Material-Base row and note the Checkpoint-A implication: with these two ported, the Material Base remainder is Substrate @2.5 (a Director scope question), TickDynamics @4.0 (#563), and the seven gate-cleared systems — Checkpoint A (ADR208 R14: **all 13** ported) is closer but not reached, and **WS3 stays HELD**.
- [ ] **Step 5: Full gates, once** — `mise run rust:check`; `mise run check`; `mise run qa:regression`; `mise run qa:vault-regression-ci`; the RST sync suite; `vale` over every touched Markdown/RST. Nothing under `tests/baselines/**` may move.
- [ ] **Step 6: `ai/state.yaml` closing entry** + commit `docs(p27): decomposition + control-ratio port handoff — register rows, ADR209, inventory verdicts`.
- [ ] **Step 7: Open PR B** `feat/control-ratio-port-bsl`, Tasks 5-9. Review lens: (a) **RESERVED-LINE fidelity** — the terminal branch byte-compared against `control_ratio.py:210-247`, and the D-row read as a Director-facing artifact; (b) composition integrity — the arc mirror, the byte-order constraint test, and the census reformulation's equivalence argument. Harvest Copilot; merge via `mise run pr:merge`.

---

## Self-review notes (plan author)

- **Every construct is landed and cited:** `nodes` as a served query head (`evaluator.rs:546`, `query.rs:133,177`), `select-max` over `nodes` (`r9_chapters.rs:1091-1310`), constant-score selection (D46), `field-of` over a `select-max` return (`evaluator.rs:1277-1301`), foreign-namespace `field-of` (`production.bsl` p1), bare-accessor fold bodies (`rule_pipeline.rs:624-670, 764-773`), the closed fold-op set `sum|mean|min|max|count` (`grammar.rs:661-717` — only `sum` is used), non-self `update-node` (D103/D104, `tick.rs:994-1076`), the closed op set `add|sub|set|scale` (`grammar.rs:721-723`), `guard` inside effects (`dispossession.bsl:394+`), `emit` with NodeRef payload values (`dispossession_conformance.rs:150-176`), `:tick` as a servable bind source (`tick.rs:418-419,467-471`), the declared-and-called `floor` intrinsic (`floor_intrinsic_e2e.rs:34,41`), subject derivation from the `:field` namespace (`tick.rs:165-186`), ceilings from minted counts (`lib.rs:187-217`), D116 cross-rule same-tick visibility, the D127 hash-neutral operand gate.
- **The one genuine capability risk** is that `nodes`-scoped `fold`/`select-max` is served-but-content-unprecedented — every landed instance of both is over `neighbors`. Task 1 Step 5's spike exists solely to convert that from an assumption into evidence *before* six rules depend on it, with a named fallback if it fails. This is the plan's highest-variance step; a reviewer should check it landed as a real spike and not as a comment.
- **What I could not verify and left as a Task-0/Task-1 obligation:** whether a rule-id first segment admits a hyphen (`control-ratio`); the result type of a `fold sum` over an `int`-declared field, which decides whether any quotient touches the `Int ÷ Int` loud error; and the exact `:fuel` figures (declare generously, tighten if the bound checker refuses). The `deffield` question is now **answered** (seven type tokens, no `bool`, no `:optional`/`:default`) and folded into §2.
- **Numbers this plan asserts that the implementer must re-derive, not trust:** the arc's tick schedule (1 / 53 / 105 / 106), every `report.fired` count, and the split arithmetic (`floor(1000·0.15)` etc.) — all three come from the mirrors, and the mirrors are the contract.
- **Fixture design intent:** the seeded enforcer and internal proletariat carry **non-zero** starting values specifically so the additive/overwrite asymmetry is mutation-provable — with zero seeds, `add` and `set` are indistinguishable and defect #2 would port silently. Likewise the two-prisoner-class weighted-mean fixture exists to make the population-weighting provable, and the exactly-0.5 organization fixture to make the `>=` boundary provable.
- **The one open recommendation for the reviewer:** two packs (with the disclosed byte-order inversion, the delay-protection argument and its enforcing test) versus one pack with a single total order. I recommend two, because these are two frozen systems and the namespace is the only honest place that fact lives today — but this is the plan's single reversible structural choice.
