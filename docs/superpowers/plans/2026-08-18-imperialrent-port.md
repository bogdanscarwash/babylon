# ImperialRent @9.0 — the Imperial Circuit Port Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** Port `ImperialRentSystem` (`src/babylon/engine/systems/economic.py`, 836 lines, Material
Base **position 9.0** — see §0 on the "@10" correction) into ONE BSL rule pack, `imperial-rent.bsl`,
covering **four of its five phases**: Extraction, Tribute, Wages, Decision (+ the pool save/decay).
**Phase 4 (CLIENT_STATE subsidy) does NOT land** — it is a STOP-escalation, §6.

**Architecture:** New BSL content plus one Rust registration edit. One rule pack (**10 rules**), one
**field-guarded `NodeType/INSTITUTION` carrier node** (`imperial-rent-register`) carrying the frozen
`GlobalEconomy` graph-attr and the `tick_context` accumulator as declared graph fields, **twelve
conformance worlds** with four Python frozen mirrors, additive golden pins, D-rows **D181–D201**,
ADR214. **No language slice is needed** and **no intrinsic is declared** — every construct is landed
and cited in §3/§4.

**Revision note (critique round 1, 2026-08-18).** This plan was returned NEEDS-REVISION on
2 Critical / 9 Important / 12 Minor findings. The two Criticals were **boundary** misses — the plan
audited its boundary against `fundamental-theorem.bsl` and stopped there, missing two live
cross-pack producer/consumer seams: `decomposition.bsl`'s SUPERWAGE_CRISIS latch (§2.1, D194) and
`r06`'s first-writer status over `social-class/value-produced` / `wages-paid` / `wages/value-flow`
(§2.2, D195). The revision adds those two sections, resolves the `r01` clamp AST to one concrete
shape (§1.6-a, D196), fixes every mirror recipe's graph-attr seeds (§9), carves the byte-identical
gate down to the 16 pre-existing pins with a declared re-measure step at each PR boundary, and
splits the train into **three** PRs. Everything the critique verified sound — the transcendentals
verdict (§5), the frozen-source citations (§1), the grammar/capability claims (§3/§4), and the
Phase-4 STOP (§6) — is carried forward unchanged.

**Tech Stack:** Rust workspace (`rust/crates/{babylon-bsl,babylon-graph,babylon-tick}`), BSL content,
cargo via `mise run rust:check`, Python 3.12 host venv for the frozen mirrors.

**Rulings that govern:** ADR183 port-as-is; ADR202 R3 (the Phase-4 `P(S|A)` INHERITED verdict + the
client-state seeding gate + the `steepness_k` removal); ADR173 (no imposed functional forms);
ADR198 R1/R6 + ADR203 + ADR205 (edge-attribute storage and `update-edge` parity — LANDED, this pack
is their first content consumer); ADR210 D6-A/R9 (`phi_cap` stays a BSL DEFCONST — **not this
train's content**, §2); ADR213 / ADR208 C-04 (the `#576` intrinsic host — **not a dependency**, §5);
ADR212 (the immediately-preceding Decomposition+ControlRatio train, whose idioms this plan reuses
wholesale).

**Prior art to read before Task 1 (in this order):**
1. The four charter reports for this train (frozen surface + boundary; the gamma/transcendentals
   sweep; the ruling register; the conformance/data surface). Their **boundary analysis governs
   scope** and their **transcendentals verdict governs §5**.
2. `docs/superpowers/plans/2026-08-15-class-surface-ternary-port.md` (its **Controller amendments**
   section: `int` deffields store verbatim f64; fractional seeds refuse; `defenum` is not shared
   across scenarios) and the Decomposition+ControlRatio plan (the carrier idiom, the mutation-
   evidence idiom, the fuel-measurement idiom).
3. **`rust/crates/babylon-tick/content/rules/decomposition.bsl` IN FULL — not skimmed for the carrier
   idiom.** It is both the freshest landed rule/oracle/test triad (with `control-ratio.bsl`, their
   `tests/*_conformance.rs` and `content/scenarios/*_conformance.py`) **and the pack this port has a
   live producer seam with**. Read `:104-113` (D168's prescription, which names this port's missing
   producer role), `:114-123` (D171 item 1's `payer_id` reasoning), `:247-271`
   (`p02-superwage-warning`'s emit + latch), `:274-294` (`p03-trigger`'s consumer binding), and the
   carrier `(select-max (nodes NodeType/INSTITUTION) 1)` idiom in production (`:254, 266, 269,
   284-290, 323`). **§2.1 is the reason this entry is bolded.**
4. `rust/crates/babylon-tick/tests/edge_write_lane_e2e.rs` and `tests/edge_lane_e2e.rs` — **the two
   files that decide this pack's whole shape** (§3): the `for-each` + `update-edge` write lane, the
   pre-state law on the edge lane, and the recorded absence of `source-of`/`target-of`.
5. `rust/crates/babylon-tick/content/rules/production.bsl` (its header names this port by name and
   fixes the `la_production` seam) and **`consciousness.bsl` IN FULL** — `p0-position`,
   `p4-wage-balance`, `p5-agitation`, `p7-persist-baselines` (the four rules that bind
   `wages-paid`/`value-produced`), `p2-wages-push` (`:224-234`, the existing READER of
   `wages/value-flow`), and `p6-route` (`:294-338`, where `wage-balance`'s sign reaches the
   bifurcation). **§2.2 is the reason.** Plus `fundamental-theorem.bsl` — all 12 lines.
6. `ai/bsl-architecture-standard.md` §3.2 / §4.5 / §6.2 — no imposed functional forms, the fuel
   declare-bound+1 readback discipline, III.11 loud absence, the two-homes D-record convention.

---

## Global Constraints

Every task's requirements implicitly include this section.

- **Port-as-is (Director law, ADR183).** The frozen Python is the **structure and ordering contract,
  not a correctness oracle**. Transcribe exactly; every divergence earns a D-row. Defects transcribe
  verbatim (§1.5 lists four). **Never silently repair.**
- **RESERVED LINES (Constitution IX.5).** Phase 4 (`_process_subsidy_phase`, `economic.py:546-666`)
  is **not implemented by this train**. §6 is its STOP-escalation row. No task may improvise a
  subsidy gate, a wealth-dispersion family, or a `P(S|A)` measure. If a step appears to require one:
  **STOP and escalate to the Director.**
- **The boundary is absolute (§2).** This pack **reads** `social-class/production-value` and
  **never recomputes it**; it **never writes** `social-class/imperial-rent` or `social-class/wages`
  (both belong to `economics/fundamental-theorem` / its fixture); it **never re-homes** anything into
  `fundamental-theorem.bsl`. A shared quantity is READ, not recomputed. §2 has **three** parts and
  all three bind: §2.1 the `decomposition.bsl` SUPERWAGE_CRISIS seam, §2.2 the
  `value-produced`/`wages-paid`/`wages/value-flow` first-writer seam, §2.3 the boundary-rule table.
- **The 16 pre-existing golden pins are byte-identical at landing. This train's OWN pins are
  expected to move, and moving them is a declared step, not a STOP.** This train adds new files; the
  only modified files are `rust/crates/babylon-tick/src/lib.rs` (one registration string),
  `tests/tick_goldens.rs` (additive pins), and docs/records. `tick_goldens.rs` currently holds
  **18 `#[test]` functions, 16 of them `*_hashes_are_pinned`** (verified 2026-08-18). Two separate
  obligations, never conflated:
  1. **The 16 pre-existing pins must stay byte-identical in every commit of every PR. If one moves:
     STOP.** They are the invariant.
  2. **The pins this train adds are re-measured whenever a later rule changes their world.** PR A
     pins `imperial_rent_conformance` and `imperial_rent_trpf_conformance` after `r00`–`r04`; world 1
     carries a WAGES edge and the carrier, so landing `r05`–`r07` in PR B and `r08`/`r09` in PR C
     **will** move both worlds' post-tick hash and `report.fired`. That motion is **expected,
     measured and mutation-disciplined**: each PR's first task re-runs `run_once` for every pin it
     inherits, pastes the new measured value back, and records the per-rule-id `fired` arithmetic
     that explains the delta in the commit body. A pin that moves **without** a matching new rule id
     in the `fired` breakdown is the STOP condition — not motion itself.
- **Copies-agree, or single-source.** Where the split of §8 makes two rules transcribe the same
  frozen expression (`rent` in `r01`/`r02`; `cut`/`tribute` in `r03`/`r04`; `super-wage-bonus` in
  `r05`/`r06`/`r07`; `total-wages` in `r06`/`r07`), **every such pair owes a named conformance row
  asserting the two transcriptions agree** through their observable graph writes. See §8's
  "duplication ledger" — the rows are enumerated there, and D201 records why single-sourcing is not
  available in the language.
- **Mutation coverage floor.** Every rule in §8 owes at least one mutation vector, `r00` included.
  Every clamp owes one, including `r08`'s two double-clamps and `r09`'s `max(0, ·)`. A clamp whose
  fixture cannot make it bind is not exempt — it owes a **converse** vector plus a recorded
  reachability proof (the §1.6-a form).
- **No Python source changes, none.** The frozen engine is read-only reference. `mise run
  qa:regression` and `mise run qa:vault-regression-ci` are therefore byte-identical trivially — run
  them once anyway as proof (Task 9). No file under `tests/baselines/**` may move; if one does,
  **STOP** — that is a §6.5 ceremony, not a side effect.
- **No new formalism.** No new verbs, no new intrinsics, no new mathematics. The carrier node and
  the per-node published fields are **content reformulations** of Python locals/dicts, in the same
  class as Production's `social-class/production-value` and Decomposition's `carceral-register`.
- **No imposed functional forms (ADR173 / NORTH_STAR.md:26-28).** Nothing in this pack may stipulate
  a sigmoid, logistic, tanh or Gaussian shape. The only such shape on the frozen surface is Phase 4's
  acquiescence sigmoid, and Phase 4 does not land. `sigmoid` is additionally a **prohibited BSL
  intrinsic name**, refused at load (`E-LOAD-024`); spelling the logistic out of `exp` + arithmetic
  is the same prohibited motion and is explicitly named as "routing around a gate that is
  deliberately mechanical."
- **Vocabulary discipline.** `NodeType`/`EdgeType`/`SocialRole` members come from the canonical
  Python enums verbatim; **enum member order is hash-bearing (ADR195)**. Transcribe `SocialRole`
  from `src/babylon/models/enums/social.py` in the landed order: `CORE_BOURGEOISIE,
  PERIPHERY_PROLETARIAT, LABOR_ARISTOCRACY, PETTY_BOURGEOISIE, LUMPENPROLETARIAT,
  COMPRADOR_BOURGEOISIE, INTERNAL_PROLETARIAT, CARCERAL_ENFORCER`. Edge types from
  `src/babylon/models/enums/topology.py:99,103-105`: `EXPLOITATION`, `TRIBUTE`, `WAGES` (
  `CLIENT_STATE` is **not declared** — Phase 4 does not land, and no speculative declarations).
  The carrier uses the **existing** `NodeType/INSTITUTION` member; this train mints **no** new node
  or edge type.
- **`defenum` is not shared across scenarios** (one `(scenario …)` form per source): every scenario
  re-declares `(defenum SocialRole …)`, and the suite carries one ordinal-parity test mirroring the
  mint's.
- **Declare only what the pack's own rules read** (the control-ratio header's own discipline). No
  speculative `deffield`, no speculative `defconst`. In particular do **not** declare
  `economy/subsidy-*`, `economy/negligible-subsidy`, `survival/steepness-k` or
  `social-class/repression-faced` — all four are Phase-4-only.
- **Six-leg cargo gate per commit** (from `rust/`): `cargo fmt --check`; `cargo clippy --workspace
  --all-targets -- -D warnings`; `cargo test --workspace`; `cargo clippy -p babylon-kernel
  --all-targets -- -D warnings -D clippy::pedantic` and same for `-p babylon-bsl`;
  `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --no-deps`; `cargo test -p babylon-tick --test
  tick_goldens --locked`. `mise run rust:check` green after every task.
- **After any `docs/reference/bsl-language.rst` edit:** `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run
  pytest tests/unit/reference/test_bsl_grammar_sync.py -q`. If a register probe reds because a new
  row cross-references an earlier D-code, repair the **test anchor** — never weaken an assertion.
- **Fuel is MEASURED, never guessed (§4.5, declare-bound+1 readback).** For every rule: declare a
  deliberately low `:fuel N`, load, read the `E-LOAD-040: … static bound B exceeds its declared
  :fuel N` refusal, then set `:fuel B+1` (bound+1 per §4.5's off-by-one) and confirm it clears load
  **and** runtime against **every** scenario that loads the rule. Query-bearing rules' bounds scale
  with the **worst-case** `CardinalityCeiling` across every scenario in the suite — so re-measure
  after each new scenario lands, and never leave a fuel figure this plan invented. **Landed `:fuel`
  values span 1 → 4096; the query-bearing rules over `(nodes …)`/`(neighbors …)` sit in the
  hundreds-to-thousands band** (`consciousness/p5-agitation` is 224, `p2-wages-push` 128;
  `decomposition/p02`'s 33 is a *non*-query-bearing outlier). **Every rule in this pack is
  query-bearing**, and worlds 8-12 are the largest, so expect the worst-case ceilings there. Size the
  effort accordingly — the earlier draft's "33–177 reference range" was unrepresentative and would
  have made the first `E-LOAD-040` readback look like a defect.
- **Mutation evidence per rule commit:** break → a **named** test flips red → restore
  byte-identical (`git diff` clean), recorded in the commit body with the exact AST mutation. Every
  clamp, guard and constant must be mutation-provable by a fixture that exercises it, plus its
  converse test proving the other fixtures do not.
- **Golden pins measured, never derived.** Run `run_once` once, read the printed hash back, paste it.
  Every `report.fired` count gets an inline per-rule-id arithmetic breakdown in the assertion message.
- **Frozen mirrors pasted verbatim + dated.** Each Rust conformance file's doc-comment header carries
  the plan path, the frozen source file + line count, the exact `PYTHONPATH=… uv run python
  <mirror>.py` command, its **full verbatim stdout**, the date it was captured, and the
  "why exact equality, no tolerance" paragraph citing `bsl-language.rst` §4.3 + ADR183.
- **Branch from `dev` in an isolated worktree.** The worktree already exists at
  `/media/user/data/worktrees/wt-imperialrent` on `feature/imperialrent-port-bsl` (PR A). PR B
  branches off **merged dev** as `feature/imperial-circuit-port-bsl` — never stacked (#193).
  Conventional commits via `mise run commit`; merges only via `mise run pr:merge -- N`, after
  harvesting the Copilot review (ADR181).
- **Token economy:** subagents write artifacts to files and return ≤15-line summaries.

---

## 0. Position correction, and the two "Imperial Rent"s

**Position is 9.0, not 10.** `economic.py:37` declares `position: ClassVar[float] = 9.0`; the
registry comment (`simulation_engine.py:299`) reads `# 9. ImperialRentSystem`; `_DEFAULT_SYSTEMS`
sorts on that `position` field. It is the **10th entry** of the Material Base tuple (0-indexed 9),
which is where "@10" came from. Every artifact this train writes says **@9.0**.

**Two unrelated things are both called "Imperial Rent" / "Φ".** This train ports **only** the first:

| | module | what it is | ported here? |
|---|---|---|---|
| 1 | `engine/systems/economic.py::ImperialRentSystem` (@9.0) | the 5-phase pool-based **Imperial Circuit** (Extraction→Tribute→Wages→Subsidy→Decision) | **YES** (phases 1,2,3,5) |
| 2 | `domain/economics/tick/system/imperial_rent.py` | the **Leontief BEA I-O pipeline** invoked as Step 4 of `TickDynamicsSystem` (@4.0), writing `CountyEconomicState.phi_hour` | **NO** — belongs to `#563` |
| 3 | `formulas/fundamental_theorem.py` | the `W_c > V_c` gap, consumed by `ContradictionSystem` (@18) and the `observe()` projection | **NO** — already landed as `fundamental-theorem.bsl` |

System 1 **never reads** `phi_hour`/`tick_phi_hour` (zero grep hits in `economic.py`) and has **no
gamma/Leontief dependency at all**. It also never imports or calls `formulas/fundamental_theorem.py`.
The name collision is a genuine false-friend trap; the pack header must say so in its first
paragraph. A repo-wide rename of one of the three is **out of scope** and recorded as a follow-on
recommendation only.

---

## 1. Frozen-source archaeology

Transcribe from *here*; verify against the cited lines before writing each rule.

### 1.1 Class surface and `step()` (`economic.py:26-86`)

`partition = MATERIAL_BASE`, `position = 9.0`, `name = "Imperial Rent"`, `creates_value = True`
(`:44`, with a spec-053 INV-001 comment opting out of the per-system c+v+s conservation check —
port-relevant only as a recorded note).

`step()` does six things, in order:
1. `economy = self._load_economy(graph, services)` (`:55`) — `graph.get_graph_attr("economy")` →
   `GlobalEconomy`, else a fresh one seeded from
   `defines.economy.initial_rent_pool` / `defines.economy.super_wage_rate` /
   `defines.survival.default_repression` (`:782-805`).
2. `initial_pool = defines.economy.initial_rent_pool` (`:56`).
3. Seed `tick_context` (`:59-66`): `tribute_inflow=0.0`, `wages_outflow=0.0`, `subsidy_outflow=0.0`,
   `current_pool=economy.imperial_rent_pool`, `wage_rate=economy.current_super_wage_rate`,
   `repression_level=economy.current_repression_level`.
4. Five ordered phase calls (`:70-74`). `_process_subsistence_phase` (`:201-237`) is **dead code**
   — `.. deprecated:: ADR032`, never called by `step()`. **Not ported** (D193).
5. `_save_economy` (`:77`, body `:807-836`) — TRPF pool decay then write-back.
6. Two conditionally-wired sub-stages (`:81`, `:86`), both **silent no-ops** whenever their
   `context.persistent_data` keys are absent — which is every unit test and every qa-six scenario.
   **Not ported** (D192).

### 1.2 Phase 1 — Extraction (`:239-345`)

```
base_eff  = defines.economy.extraction_efficiency / defines.timescale.weeks_per_year   # :253-255
trpf_mult = max(defines.economy.trpf_efficiency_floor,
                1.0 - defines.economy.trpf_coefficient * tick)                          # :259-261
eff       = base_eff * trpf_mult                                                        # :262
```
Then, per `EdgeType.EXPLOITATION` edge (source = **worker**, target = **exploiter**):
`continue` if either end's `active` is falsy (`:276`, `:280`); `worker_wealth =
worker_attrs.get("wealth", 0.0)`; `consciousness = class_consciousness_from_node(worker_attrs)`
(`kernel/node_access.py:15-35`, reads `node_data["ideology"]["class_consciousness"]`, `0.0` when
absent);

```
rent = eff * worker_wealth * (1.0 - consciousness)      # :289
rent = min(rent, worker_wealth)                          # :292
```
Writes: `source.wealth = max(0.0, worker_wealth - rent)` (`:295` — the `max` is **dead**, see
§1.5-a); `target.wealth = target_wealth + rent` (`:297`); `update_edge(…, EXPLOITATION,
value_flow=rent)` (`:300-302`). Optional L-RECEIPTS `register.record(...)` (`:310-321`), a guarded
no-op with no register bound. If the target's `role` is `CORE_BOURGEOISIE` (with a str→enum
coercion, `:324-327`): `tribute_inflow += rent` **and** `current_pool += rent` (`:328-329`). Emits
`SURPLUS_EXTRACTION` when `rent > defines.economy.negligible_rent` (`:332-345`; payload
`source_id`, `target_id`, `amount`, `mechanism="imperial_rent"`).

Note the asymmetry: unlike Phases 2/3/4, Phase 1 has **no** `worker_wealth <= 0` guard.

### 1.3 Phase 2 — Tribute (`:347-400`)

Per `EdgeType.TRIBUTE` edge (source = **comprador**, target = **recipient**): both-ends `active`
guards (`:367`, `:371`); `comprador_wealth = source.wealth`; `continue` if `<= 0` (`:377-378`).

```
cut_amount     = comprador_wealth * defines.economy.comprador_cut   # :381
tribute_amount = comprador_wealth - cut_amount                       # :382
```
Writes: `source.wealth = cut_amount` — an **OVERWRITE, not a decrement** (`:385`, §1.5-c);
`target.wealth += tribute_amount` (`:386-387`); edge `value_flow = tribute_amount` (`:390-392`).
Same `CORE_BOURGEOISIE` gate feeding `tribute_inflow` and `current_pool` (`:395-400`).
**No event.**

### 1.4 Phase 3 — Wages (`:402-544`)

Rule-scoped quantities (computed once, before the loop):
```
super_wage_rate = tick_context["wage_rate"] / defines.timescale.weeks_per_year          # :421-423
ppp_multiplier  = 1.0 + (defines.economy.extraction_efficiency
                         * defines.economy.superwage_multiplier
                         * defines.economy.superwage_ppp_impact)                        # :426-432
negligible      = defines.economy.negligible_rent                                       # :434
available_pool  = tick_context["current_pool"]                                          # :435
la_production   = graph.get_graph_attr("la_production", {})                             # :438
```
Per `EdgeType.WAGES` edge (source = **employer**, target = **worker**):
```
productivity_value = la_production.get(edge.target_id, 0.0)                             # :453
max_bonus          = tick_context["tribute_inflow"] * super_wage_rate                   # :456-457
super_wage_bonus   = min(max_bonus, available_pool)                                     # :458
```
**The SUPERWAGE_CRISIS check runs BEFORE the active-skip** — deliberately (`:447-448`: "the crisis
is about the SYSTEM'S inability to pay wages, not individual status"). Condition
`available_pool <= negligible and super_wage_bonus <= negligible` (`:462`; the second conjunct is
**dead**, §1.5-b). Payload (`:472-485`): `payer_id`, `receiver_id`, `productivity_value`,
`super_wage_bonus`, `available_pool`, `bourgeoisie_wealth`, `bourgeoisie_active`, `narrative_hint`.
Wages proceed afterwards regardless (`:488-489`).

Then the guards fire: employer inactive → `continue` (`:492-493`); worker inactive → `continue`
(`:495-496`); `bourgeoisie_wealth <= 0` → `continue` (`:501-502`).

```
total_wages = productivity_value + super_wage_bonus                                     # :507
total_wages = min(total_wages, bourgeoisie_wealth)                                      # :510
```
Writes — employer `wealth = bourgeoisie_wealth - total_wages` (`:513`); worker gets **six** attrs
at once (`:514-531`):

| attr | value | site |
|---|---|---|
| `wealth` | `current_wealth + total_wages` | `:515,518` |
| `effective_wealth` | `new_nominal_wealth + total_wages*(ppp_multiplier-1.0)` | `:519` |
| `unearned_increment` | `total_wages*(ppp_multiplier-1.0)` | `:520` |
| `ppp_multiplier` | `ppp_multiplier` | `:521` |
| `w_paid` | `total_wages` | `:529` |
| `v_produced` | `productivity_value` | `:530` |

Edge `value_flow = total_wages` (`:534-536`). Then the pool bookkeeping:
```
actual_bonus_paid = min(super_wage_bonus, total_wages - productivity_value)             # :540
actual_bonus_paid = max(0.0, actual_bonus_paid)                                          # :541
tick_context["wages_outflow"] += actual_bonus_paid                                       # :542
tick_context["current_pool"]  -= actual_bonus_paid                                       # :543
available_pool = tick_context["current_pool"]   # for the NEXT iteration                 # :544
```
**Line :544 is the plan's hardest constraint** — pool depletion is applied **per-edge, in query
order**, so later WAGES edges see an already-reduced pool. §4 BLOCKER-2.

### 1.5 Phase 5 — Decision (`:668-750`) and `_save_economy` (`:807-836`)

```
pool_ratio        = current_pool / initial_pool if initial_pool > 0 else 0.0            # :682-683
aggregate_tension = graph.get_graph_attr("opposition_states", {})
                        .get("capital_labor", {}).get("gap", 0.0)                       # :752-780
```
`_calculate_aggregate_tension` is **deliberately one-tick-stale** — `opposition_states` is stamped
by `ContradictionSystem` at position 18, later in the SAME tick, so @9.0 reads last tick's snapshot;
absent on tick 0 → `0.0` (docstring `:755-773`).

`calculate_bourgeoisie_decision` (`formulas/dynamic_balance.py:25-118`) is a **priority-ordered,
pure-arithmetic** if/elif matrix — it does **not** touch survival calculus, does **not** call
`exp`/`log`, and returns `(decision: str, wage_delta, repression_delta)`:

| # | condition (evaluated in this order) | decision | wage_delta | repression_delta |
|---|---|---|---|---|
| 1 | `pool_ratio < critical_threshold` | CRISIS | `crisis_wage_delta` | `crisis_repression_delta` |
| 2 | `pool_ratio >= high_threshold and tension < bribery_tension_threshold` | BRIBERY | `bribery_wage_delta` | `0.0` |
| 3 | `pool_ratio < low_threshold and tension > iron_fist_tension_threshold` | IRON_FIST | `0.0` | `iron_fist_repression_delta` |
| 4 | `pool_ratio < low_threshold` (else-arm of 3) | AUSTERITY | `austerity_wage_delta` | `0.0` |
| 5 | otherwise | NO_CHANGE | `0.0` | `0.0` |

Then (`:724-730`): `new_wage_rate = clamp(wage_rate + wage_delta, min_wage_rate, max_wage_rate)`;
`new_repression = clamp(repression_level + repression_delta, 0.0, 1.0)`; both written back into
`tick_context`. Emits `ECONOMIC_CRISIS` **only** on `decision == CRISIS` (`:733-750`; payload
`pool_ratio`, `aggregate_tension`, `decision`, `wage_delta`, `repression_delta`, `new_wage_rate`,
`new_repression_level`, `current_pool`).

`_save_economy` (`:827-836`): `current_pool *= (1.0 - defines.economy.rent_pool_decay)`, then
`imperial_rent_pool = max(0.0, current_pool)`, `current_super_wage_rate = tick_context["wage_rate"]`,
`current_repression_level = tick_context["repression_level"]`.

### 1.6 The four frozen defects: three transcribed verbatim, one resolved into a decided AST

- **(a) Dead clamp — RESOLVED TO ONE AST, and it is NOT a clamp (D196).** `economic.py:295` is
  `source.wealth = max(0.0, worker_wealth - rent)`: a **SET with a clamp**, against the wealth read
  at the top of *this* iteration. The earlier draft said "`(sub rent)` with the dead clamp
  transcribed", which is self-contradictory — `sub`'s operand slot is a single `<expr>`
  (`structural_verbs.rs:902-920`), so there is nowhere to hang a clamp on a `sub`, and BSL has **no
  binary `max`** (`FoldOp` is the closed five-member `{sum, mean, min, max, count}` aggregation set,
  `grammar.rs:661-716`; `<arith>` is the closed four-member `+ - * /`, `grammar.rs:723`). The two
  candidate shapes and the decision:

  | candidate | AST | verdict |
  |---|---|---|
  | clamp-preserving | `(update-node self social-class/wealth (set (if (> diff 0c) diff (- 0 0c))))` with `diff = (- (field-of self social-class/wealth) rent)` as a rule-scoped binding | **REJECTED** |
  | live arithmetic | `(update-node self social-class/wealth (sub rent))` | **ADOPTED** |

  **Why the clamp-preserving shape is rejected.** Under the pre-state law (§3.3) a `set` inside the
  `for-each` writes the same pre-state-derived value on every iteration, so a worker with **two**
  EXPLOITATION edges ends at `wealth − rent`, where the frozen loop decrements twice. `set` therefore
  silently *repairs* the frozen per-edge repetition — a behavior change in the **opposite** direction
  from D184's declared divergence, and a port-as-is violation. `sub` accumulates, which is the frozen
  shape's direction.

  **Reachability proof that the frozen clamp is dead in every frozen-reachable world.**
  `rent = min(eff · worker_wealth · (1 − consciousness), worker_wealth) ≤ worker_wealth` (`:292`),
  and the frozen loop re-reads `worker_attrs["wealth"]` at the top of every iteration (`:283`), so
  the invariant `worker_wealth − rent ≥ 0` holds at **every** iteration. The `max(0.0, ·)` can
  therefore never bind in the frozen engine. Nothing in the frozen surface can break it: `eff` and
  `(1 − consciousness)` only shrink the product, and `min` caps it at the balance.

  **Where the invariant does NOT hold, and what this train owes because of it.** The ported `rent` is
  a **rule-scoped binding computed once** from the pre-state wealth and applied per edge, so a worker
  with N EXPLOITATION edges ends at `wealth − N · rent`, which **can go negative** (take
  `eff · (1 − c) ≥ 0.5` and N = 2). That is not a new divergence class — it is D184's declared
  divergence seen from its second face — but it means the clamp's absence is **observable** in the
  ported estate where it was unobservable in the frozen one. World 8 seeds exactly that worker (two
  EXPLOITATION edges), so the case is **measured, not assumed**: Task 6 asserts the ported number,
  prints the frozen number beside it, and D196 records both with this proof. `r09`'s own
  `max(0, pool)` and `r07`'s `max(0, actual_bonus_paid)` are **live** clamps and are transcribed as
  `if` chains — this row is about `:295` alone.
- **(b) Dead conjunct.** `available_pool <= negligible and super_wage_bonus <= negligible` (`:462`):
  `super_wage_bonus = min(max_bonus, available_pool) <= available_pool`, so the first conjunct
  implies the second. Transcribe both; the mutation vector must show that dropping the second
  conjunct changes nothing (and record that as the evidence, not as a failure to find a killer).
- **(c) Overwrite/decrement asymmetry.** Phase 2 writes `wealth = cut_amount` (`:385`) where every
  other phase writes `current − delta`. This is a **structurally different update op** (`set` vs
  `sub`) and must be mutation-provable: the comprador is seeded with non-zero wealth so `set` and
  `sub` are distinguishable.
- **(d) Coefficient/docstring drift.** `dynamic_balance.py:38` defaults
  `bribery_tension_threshold = 0.3` and the module docstring's own matrix (`:47`) is written against
  that default — but `defines.yaml:97` ships **0.7**, and `ImperialRentSystem` passes the define
  (`economic.py:689-701`). **The shipped 0.7 governs.** The code governs; the comments are wrong.

### 1.7 Test estate (read, do not modify)

Thirteen unit files under `tests/unit/engine/systems/` exercise this system phase-by-phase:
`test_economic_decision.py` (20), `test_economic_subsidy.py` (15), `test_economic_wages.py` (12),
`test_economic_tribute.py` (10), `test_economic_events.py` (10), `test_default_values.py` (11),
`test_subsistence.py` (11, the DEAD phase), `test_economic_accounting.py` (6),
`test_phi_wiring.py` (6), `test_economic_weekly.py` (5), `test_receipts.py` (4),
`test_superwage_crisis.py` (3), `test_vol2_wiring.py` (2). Integration: `test_modular_engine.py`,
`test_dynamic_balance.py:387` (the `opposition_states` staleness contract),
`test_proletarian_internationalism.py:149`, `test_class_decomposition.py:4`.
**Not a conformance candidate despite the name:** `tests/unit/formulas/test_fundamental_theorem.py`
tests a different module entirely (§0 row 3).

---

## 2. THE BOUNDARY — what this pack owns, what it may only read, and what it becomes the producer of

The first draft of this section audited the boundary against `fundamental-theorem.bsl`, found it
empty, and stopped. That was the plan's central failure: **the boundary that matters is not
"does another pack already compute this?" but "does another pack already WAIT on this?"** Two landed
packs wait on quantities this pack produces, and both seams are live, theory-bearing and inverted by
byte order. They are §2.1 and §2.2. §2.3 carries the duplication verdict and the boundary-rule table.

This is the Solidarity lesson in its second direction — not "silently duplicates a producer" but
**"silently becomes, or fails to become, the producer another pack is waiting on."**

### 2.1 CRITICAL SEAM — `decomposition.bsl`'s SUPERWAGE_CRISIS latch (D194)

**The frozen coupling.** `decomposition.py:161-175` scans `services.event_bus.get_history()` for
**every** `SUPERWAGE_CRISIS` event and takes `min(e.tick for e in crisis_events)` as
`persistent["_superwage_crisis_tick"]`, which drives `DecompositionSystem`'s CLASS_DECOMPOSITION
delay trigger. In the frozen engine that history **includes ImperialRentSystem @9.0's Phase-3
emits** — @9.0 runs before Decomposition @11 in the same tick. Two consequences the earlier draft
missed entirely:

1. **ImperialRent's Phase-3 crisis emit is a live INPUT to Decomposition.** The pool-exhaustion →
   LA-decomposition causal path runs through it.
2. **It also SUPPRESSES Decomposition's own emit.** `decomposition.py:179` gates its early-warning
   publish on `la_approaching_death and superwage_tick is None and la_id is not None`. Once
   ImperialRent has emitted this tick, `superwage_tick` is not None, so Decomposition emits nothing.

**What the landed pack already prescribes.** `decomposition.bsl:104-113` (register row D168) records
the omitted history read and names the re-modelling **verbatim**:

> "BSL has no same-tick or cross-tick event-history query (`bsl-language.rst`'s own gap item 3 —
> *'the emitting rule also stamps a field'* is the prescribed re-modelling). The carrier's
> `superwage-crisis-known`/`-tick` latch, written by p02 the same tick it emits, is the sole source
> of truth — exactly the re-modelling the language document itself names, not an invented shortcut."

`r05-wages-crisis` **is** such an emitting rule. **Disposition: `r05` stamps the latch.** The exact
landed shapes it must match (verified 2026-08-18):

- The fields: `institution/superwage-crisis-known` and `institution/superwage-crisis-tick`, both
  **`int extensive`** (`decomposition-conformance.bscn:152-153`, `carceral-arc-conformance.bscn:111-112`,
  `decomposition-delay-conformance.bscn:91-92`). This pack **declares them exactly as declared there**
  and **does not** give them a `rent-` prefix — they are `decomposition.bsl`'s vocabulary, and a
  parallel qname would be the very failure this section exists to prevent.
- The consumer: `decomposition/p03-trigger`'s
  `(binding delay-elapsed-fire :expr (and (= superwage-crisis-known 1) (>= tick (+ superwage-crisis-tick decomposition-delay))))`
  (`decomposition.bsl:293-294`).
- The write-once discipline: `decomposition/p02-superwage-warning` is gated `(= crisis-known 0)`
  (`decomposition.bsl:260`) and writes `(set 1)` / `(set tick)` (`:266-271`). **`r05`'s latch write
  carries the same `(= crisis-known 0)` guard** — that guard is what reproduces the frozen
  `min(e.tick)` semantics (first-writer-wins). An unguarded `(set tick)` would overwrite a later
  tick over an earlier one and invert `min` into `last`. This is the single most important detail in
  §2.1 and it owes its own named test.

**Payload reconciliation — two emitters, disjoint key sets, and the `payer` disagreement.**
`decomposition.bsl:262-265` emits `EventType/SUPERWAGE_CRISIS` with `(receiver self)`,
`(desired-wages 0.0c)`, `(available-pool 0.0c)`. Planned `r05` emits `payer`, `receiver`,
`productivity-value`, `super-wage-bonus`, `available-pool`, `bourgeoisie-wealth`,
`bourgeoisie-active`. The key sets are disjoint except `receiver` and `available-pool`.

- **We do NOT align the two.** Aligning down (dropping `r05`'s keys) discards frozen payload the
  ADR183 contract requires; aligning up (editing `decomposition.bsl`) moves landed goldens and is a
  separate ceremony. **Both shapes stand, and the divergence is recorded (D194) and pinned by test.**
- **The `payer` disagreement with D171 item 1 is only apparent, and this is the reconciliation.**
  D171 item 1 (`decomposition.bsl:114-116`) DROPS `payer_id` on the stated ground that it is *"a
  second NodeRef, **always** `CORE_BOURGEOISIE_ID`"* — i.e. a constant, carrying no information. In
  **this** pack the payer is **not** a constant: it is the WAGES edge's source, resolved per worker
  as `(select-max (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS) 1)`, and a world with two
  employers gives two different payers. D171's rationale does not transfer, so `r05` **KEEPS**
  `payer`. Record this explicitly — a reviewer comparing the two rows must be able to see that the
  two trains applied *the same rule* to *different facts*, not opposite rules to the same fact.
- The common `available-pool` key carries a literal `0.0c` in `p02` and the live tick-start pool in
  `r05`. A consumer keying on `available-pool` alone cannot tell the emitters apart; the
  discriminator is `payer`'s presence. Stated in D194 so no future consumer assumes one shape.

**The byte-order inversion and its executable constraint** — see §7, which now lists `decomposition/`
and carries the analysis in the Dec+CR train's §5 shape.

**Scope decision:** stamping the latch is IN scope. It is not new mathematics, not a new primitive
and not a reserved line — it is the transcription of a frozen coupling into the re-modelling the
landed pack's own register row prescribes for it. Declining it would land a pack that silently kills
a causal path in every combined world, which is exactly the failure ADR183's port-as-is discipline
exists to prevent.

### 2.2 CRITICAL SEAM — `r06` becomes the first writer of three quantities four landed rules read (D195)

`r06` writes `social-class/wages-paid (set total)` and `social-class/value-produced (set production-value)`;
`r06` also writes `wages/value-flow (set total)`. The earlier draft treated all three as naming
notes (B4/B5 → a deffield-type divergence). They are live producer/consumer seams. The census
(verified 2026-08-18, every read-site re-read in full):

| qname | today's only source | landed READERS | what this pack's write does to them |
|---|---|---|---|
| `social-class/value-produced` | scenario seeds (`two-classes.bscn:9,14,18`; `consciousness-ternary-conformance.bscn:230,268,280,295`) | `economics/fundamental-theorem` (`fundamental-theorem.bsl:9`, **required** binding, guard `(> wages value-produced)`, writes `social-class/imperial-rent`); `consciousness/p0-position` (`:185`), `p4-wage-balance` (`:252`), `p5-agitation` (`:265`), `p7-persist-baselines` (`:345`) — all four `:optional :default -1` behind `(>= value 0)` | becomes live per tick |
| `social-class/wages-paid` | scenario seeds (`consciousness-ternary-conformance.bscn:229,268,280,295`) | the same four `consciousness` rules (`:184`, `:251`, `:264`, `:344`) | becomes live per tick |
| `wages/value-flow` | scenario seeds ONLY — `(edge-attr …)` load-time literals (`consciousness-ternary-conformance.bscn:243,309,311,313`). **No landed rule writes it** | `consciousness/p2-wages-push` (`:232-234`), which accumulates it into `social-class/wages-inbox` | becomes a per-tick FLOW where it is currently a constant |

**Correction to the critique's own attribution, verified line by line.** The four consciousness
line-pairs belong to `p0-position`, `p4-wage-balance`, `p5-agitation` and `p7-persist-baselines` —
**not** `p6`/`p8`. `p6-route` binds neither field (it consumes the derived `social-class/wage-balance`);
`p8-dominant-worldview` has no path to either. The conclusion is unchanged and in fact sharper,
because the four rules use the pair in **two different ways**:

- **`p4-wage-balance` alone does arithmetic on it** (`consciousness.bsl:253-255`):
  `balance = (if (> (+ wages value) 0) (/ (- wages value) (+ value wages)) (- 0 0c))`, written to
  `social-class/wage-balance`. Its own `:material-basis` names the meaning: *"positive = wages
  dominate = **the imperial bribe**."*
- **`p0`, `p5` and `p7` use the pair purely as an ANCHOR-PRESENCE GATE** (`(>= wages 0)`,
  `(>= value 0)` against the `-1` sentinel). `p0-position` is the one that matters: a class with the
  anchors present and no ternary record is positioned at the ruled `(0, 1, 0)` rest state; a class
  without them is **UNPOSITIONED** and never accumulates agitation (D153). So this pack's write is
  what flips a class from UNPOSITIONED to positioned.

**Consumer impact statement — what actually changes downstream, in the model's own terms.**

1. **`wage-balance`'s sign routes the bifurcation, and this pack is what makes it non-zero.**
   `consciousness/p6-route` reads only the **positive** half:
   `(binding chauvinist :expr (* (if (> balance 0) balance 0) chauv-scale))` (`:312`), subtracted
   from the effective-solidarity factor `eff-sol` (`:315-316`), which splits consumed agitation
   between `delta-r` (revolutionary, `:317`) and `delta-f` (fascist, `:318`). A **positive**
   wage-balance therefore routes agitation **fascist-ward**.
2. **This pack's write makes the balance positive exactly when the imperial rent pool pays a
   super-wage.** `r06` writes `wages-paid = total = min(production-value + bonus, employer-wealth)`
   and `value-produced = production-value`, so
   `wages − value = min(bonus, employer-wealth − production-value)`. With `bonus = 0` (no super-wage)
   the balance is exactly **zero** and `p6-route` is unaffected. With `bonus > 0` and an unconstrained
   employer the balance is **positive** — the labor aristocracy's super-wage becomes the chauvinism
   signal. That is MLM-TW theory arriving as a *derived* consequence of two independently transcribed
   packs, not as a stipulated mechanic, and it is exactly the coupling this train must NAME rather
   than let a later reader discover. **It is not a new mechanic and needs no Director gate** — no
   coefficient, no functional form, no new axis; it is the composition of two port-as-is
   transcriptions. It IS a theory-bearing consequence, so it is stated in ADR214 for the Director's
   information.
3. **The balance can go NEGATIVE, and only one thing makes it so:** the `min(·, bourgeoisie_wealth)`
   cap at `economic.py:510`. When the employer's wealth binds below `production-value`, `total <
   production-value` and the balance flips sign — the capital-constrained employer stops bribing.
   Task 5 seeds exactly that world; it is the sign-flip fixture.
4. **`wages/value-flow` turns a constant into a flow, which turns on `p5-agitation`'s exploitation
   term.** `p2-wages-push` accumulates `wages/value-flow` into `social-class/wages-inbox`;
   `p5-agitation` computes `wage-change = (- wages-in prev-wages)` and
   `exploit-delta = (if (< wage-change 0) (- 0 wage-change) 0)` (`:281-282`), with `p7` persisting
   `previous-wages = wages-in` each tick (`:350`). Today `wages/value-flow` is a load-time literal,
   so from tick 2 onward `wage-change` is identically 0 and the exploitation term is **dead**. Once
   this pack writes the flow per tick, a **falling** wage produces a positive `exploit-delta` and
   agitation rises. That is the Φ-disruption loop closing. Same disposition as (2): derived, not
   stipulated; named here and in ADR214.

**The half-anchored `economics/fundamental-theorem` guard — a decided NON-scope.**
`fundamental-theorem.bsl:8-9` binds `social-class/wages` (bare) **and** `social-class/value-produced`,
both **required** (no `:optional`, no `:default`). This pack becomes the producer of the **second**
input and refuses to become the producer of the first (B2 stands: `social-class/wages` is the
`two-classes.bscn` fixture's own name and re-homing it is not this train's scope). So in any combined
world the guard `(> wages value-produced)` compares a **fixture-seeded constant** against a **live**
per-tick value. That is a real, named asymmetry:

- It is **recorded, not repaired** (D195), with the re-open trigger named: whoever ports
  `ContradictionSystem` @18 — the consumer that actually stashes `fundamental_theorem` — or re-homes
  the theorem's inputs, owes the other half.
- **Combined-world load hazard:** a required binding is TICK-FATAL on an absent field for a
  same-subject-type node (`bindings.rs`'s resolve-or-error law, quoted at `consciousness.bsl:301`).
  So a combined `economics/fundamental-theorem` + `imperial-rent` world must seed **both**
  `social-class/wages` and `social-class/value-produced` on **every** SOCIAL_CLASS node. Task 8's
  combined world does; the constraint is stated in its header.
- **Scope decision:** becoming `value-produced`'s producer is IN scope — it is the verbatim
  transcription of `economic.py:530`'s `v_produced=productivity_value`, and declining it would mean
  declining to port Phase 3's own writes. Becoming `social-class/wages`'s producer is OUT of scope.

**Conformance rows owed (Task 5 Step 6 and Task 8 Step 4), proving the write lands what the consumers
expect** — not that it merely lands:

| row | asserts |
|---|---|
| `r06_writes_the_exact_bytes_fundamental_theorem_binds` | post-tick `social-class/value-produced` on the wage target equals `production-value` bit-exact, read back through the same qname `fundamental-theorem.bsl:9` binds |
| `the_combined_world_positions_a_previously_unpositioned_class` | a SOCIAL_CLASS node seeded WITHOUT the ternary, in a combined consciousness + imperial-rent world, is `(0, 1, 0)` after two ticks — `p0-position`'s gate opened by this pack's anchors |
| `the_wage_balance_is_zero_without_a_super_wage_bonus` | `bonus = 0` world ⟹ `social-class/wage-balance == 0.0` exactly ⟹ `p6-route`'s `chauvinist` term contributes nothing |
| `a_super_wage_bonus_drives_the_wage_balance_positive` | `bonus > 0`, unconstrained employer ⟹ balance `> 0` bit-exact against the mirror, and `delta-f` strictly greater than the same world with `bonus = 0` |
| `a_capital_constrained_employer_drives_the_wage_balance_negative` | the `min(·, employer-wealth)` cap binds ⟹ balance `< 0` — the sign-flip fixture |
| `the_wage_flow_is_live_across_two_ticks` | `wages/value-flow` differs between tick 1 and tick 2 in a world whose bonus moves, and `social-class/wages-inbox` tracks it — the seam B4 claims |
| `a_falling_wage_raises_agitation_in_the_combined_world` | the `p5-agitation` exploitation term is non-zero on a tick where the wage falls, and zero on the same world before this pack lands |

**§7 records the inversion**: `consciousness/` and `economics/` both sort **before** `imperial-rent/`,
so **both** readers run before the writer, and every one of these seams is one tick late.

### 2.3 Duplication verdict and the boundary-rule table

**Verdict: `economics/fundamental-theorem` carries ZERO of `ImperialRentSystem`'s surface. This pack
ports the Imperial Circuit from nothing, and its zero-duplication obligations are READ-side.**
(That verdict survives the critique intact; what it does **not** cover is §2.1 and §2.2's
produce-side obligations, which are not duplication questions at all.)

Evidence, all re-verifiable:

- `fundamental-theorem.bsl` is **12 lines / one rule, `economics/fundamental-theorem`** (the rule-id
  namespace is `economics`, **not** `fundamental-theorem` — see §7's corrected namespace list). It
  binds `social-class/wages` and `social-class/value-produced` (both required, `:fuel 64`), guards
  `wages > value-produced`, and writes exactly one derived field, `social-class/imperial-rent`. That
  is its entire owned surface.
- Those two inputs are **scenario-seeded fixture values TODAY** (`two-classes.bscn:8-18` — `wages`
  120/20, `value-produced` 80/90, `imperial-rent` declared but unseeded); no landed `.bsl` rule
  writes either. **This pack changes that for one of the two** — see §2.2. The rule mirrors
  `ContradictionSystem`'s (@18) downstream
  `fundamental_theorem` graph stash and the `observe()` projection that renders it — **not**
  `ImperialRentSystem`'s extraction machinery. `ImperialRentSystem` never imports
  `formulas/fundamental_theorem.py`.
- `ImperialRentSystem`'s own vocabulary — `wealth`, the four edge types, the `GlobalEconomy` pool,
  `w_paid`/`v_produced`/`effective_wealth`/`unearned_increment`/`ppp_multiplier` — appears in **no**
  landed `.bsl` file.
- **ADR210 D6-A / R9 is RATIFIED but UNLANDED**, and it targets a *third* module: the Φ→savings→LA
  coupling in `domain/economics/dynamics/accumulation.py:85-90` +
  `savings_schedule.py:90-92`, which is **`TickDynamicsSystem`'s** transition engine (`#563`, the
  reserved trio). When D6-A lands it will extend `fundamental-theorem.bsl` with **TickDynamics**
  content. `phi_cap` stays a BSL **DEFCONST** when that happens — **on that train, not this one**.
  `rg -n 'savings|phi_cap|mobility' rust/crates/babylon-tick/content/` returns zero hits today.

**Therefore, the binding boundary rules for every task:**

| # | rule |
|---|---|
| B1 | **Do not write `social-class/imperial-rent`.** It is `fundamental-theorem.bsl`'s output. If a reviewer asks "shouldn't the imperial-rent pack write the imperial-rent field?" the answer is the name collision of §0, and the D191 row is where it is written down. |
| B2 | **Do not write or rename `social-class/wages`.** That is the `two-classes.bscn` fixture's own name for `fundamental-theorem.bsl`'s input. This pack's wage output is `social-class/wages-paid` — the qname that already matches `economic.py:529`'s `w_paid` byte for byte. |
| B3 | **READ `social-class/production-value`; never recompute it.** `production.bsl`'s own header (`:67-74`) states the contract for this port by name: *"the read a future ImperialRentSystem port would perform stays exactly as narrow as the frozen `la_production.get(edge.target_id, …)` call already is."* The reformulated per-node field is `social-class/production-value` (`real extensive`, `production-conformance.bscn:114`). Reuse it; declare it in every scenario; **seed it** (§7's byte-order constraint). |
| B4 | **Reuse `wages/value-flow`** (`real intensive`, declared `consciousness-ternary-conformance.bscn:243`, seeded `:309,311,313`, READ by `consciousness/p2-wages-push` at `consciousness.bsl:232-234`). This pack becomes its **first writer** — no landed rule writes it today. §2.2 row 4 states what that turns on. Do not mint a parallel wage-flow qname. |
| B5 | **Reuse `social-class/wages-paid` and `social-class/value-produced`** (declared `consciousness-ternary-conformance.bscn:229-230`; `value-produced` also `two-classes.bscn:9`). This pack becomes their **first writer** — the §2.2 seam, D195, **not** a naming note. Type divergence: they are declared `int extensive` there and `real extensive` here — legal per-scenario re-declaration (the landed estate already carries `social-class/wealth` in both types, and an `int` deffield stores verbatim f64, constraining *seeding* only), recorded in D191. **Combined worlds declare `real extensive`** and Task 0 Step 4(f) verifies the consumers' `:default -1` sentinel still typechecks against a `real` declaration. |
| B6 | **Do not declare `EdgeType/CLIENT_STATE`, `social-class/repression-faced`, `social-class/organization`, `social-class/subsistence-threshold`, or any `economy/subsidy-*` / `survival/steepness-k` defconst.** All are Phase-4-only, and Phase 4 does not land (§6). |
| B7 | `social-class/class-consciousness` is **net-new** for this pack (`coefficient intensive`) — Task 0 Step 3 must first verify that no landed pack already publishes a scalar class-consciousness qname (`consciousness.bsl` publishes `agitation` + the `revolutionary`/`liberal`/`fascist` ternary, none of which is the frozen scalar). If one exists, **read it**; if not, declare it as a frozen INPUT field and D-record the relationship to the ternary surface as a follow-on coupling question, not this train's scope. |
| B8 | **`r05` STAMPS `institution/superwage-crisis-known` and `institution/superwage-crisis-tick`** — `decomposition.bsl`'s own vocabulary, declared exactly as `decomposition-conformance.bscn:152-153` declares it (`int extensive`), **guarded `(= crisis-known 0)`** so first-writer-wins reproduces the frozen `min(e.tick)`. Do NOT `rent-`-prefix these two; do NOT mint a parallel latch. §2.1, D194. |
| B9 | **Carrier identity is resolved by a declared discriminator, never by an id tiebreak.** Every carrier read and write scores on `institution/rent-carrier` (§3.1). The `rent-` prefix keeps the two packs' rosters disjoint **on one node**; the discriminator keeps the *resolution* deterministic when a world holds more than one INSTITUTION node. D198. |
| B10 | **Neither `social-class/wage-balance` nor `social-class/wages-inbox` nor `social-class/agitation` is written by this pack.** They are `consciousness.bsl`'s outputs, derived from what this pack produces. This pack writes the INPUTS (`wages-paid`, `value-produced`, `wages/value-flow`) and stops. If a step appears to need one of the three, the boundary has been crossed — STOP. |

---

## 3. The reformulation: four constraints that decide the whole pack's shape

### 3.1 `GlobalEconomy` + `tick_context` → the field-guarded carrier

Follows the landed `carceral-register` idiom (ADR198 R6 blesses the carrier-node lane; do not invent
a new graph-scope mechanism). One `NodeType/INSTITUTION` node, id `imperial-rent-register`. Ceiling 1
is automatic — the driver derives `CardinalityCeilings` from the counts the scenario mints, which
also statically bounds every fold over it (Power-of-10 rule 2).

**Carrier resolution is field-guarded, NOT a constant score (D198).** The landed idiom
(`decomposition.bsl:254, 266, 269, 323, …` — 14 sites) is `(select-max (nodes NodeType/INSTITUTION) 1)`,
a **constant** score. `select-max`'s second operand is a score expression, and D45 breaks ties to the
first element in §2.6 iteration order — ascending node id (`evaluator.rs:990-1052`, register row D45
at `bsl-language.rst:5103-5108`). A constant score means **every** element ties, so a constant-score
`select-max` over `(nodes NodeType/INSTITUTION)` is exactly "the lowest-id INSTITUTION node". In a
world holding both `carceral-register` and `imperial-rent-register`, `c` < `i`, so **both packs
resolve to `carceral-register`** and the `imperial-rent-register` node is inert — including B8's
latch write, which would then land on the right node only by accident. The `rent-` prefix solves the
*field* collision and leaves the *identity* collision untouched.

**Disposition.** A `real`/`int` `(field-of it <qname>)` is a legal `select-max` score — landed and
acceptance-tested (`query_lane_e2e.rs:252-253`, `r9_chapters.rs:1129-1130` and `:1179-1180`; only an
**enum** `field-of` score is refused, `E-TYPE-016`, `typecheck.rs:909-921`). So:

- Declare `institution/rent-carrier` (`int extensive`), seeded **1** on this pack's carrier and **0**
  on every other INSTITUTION node in the world.
- Every carrier read is
  `(field-of (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier)) institution/rent-…)`
  and every carrier write is
  `(update-node (select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier)) institution/rent-… (op …))`.
  The winner is decided by **declared content**, not by ascending id.
- **Task 0 Step 4(g) verifies the bare-field score form at the byte.** If it refuses for any reason,
  fall back to the landed `territory.bsl:133-136` shape — an `if`-chain score yielding an `Int`,
  `(if (= (field-of it institution/rent-carrier) 1) 1 0)` — which is byte-for-byte a landed shape and
  cannot refuse. If **both** refuse: STOP and re-plan §3.1 before Task 2.
- **The exception, and it is deliberate: B8's latch writes (`institution/superwage-crisis-known`/
  `-tick`) use `decomposition.bsl`'s OWN constant-score expression**, verbatim
  (`(select-max (nodes NodeType/INSTITUTION) 1)`). A latch is only useful on the node its consumer
  reads, and its consumer is `decomposition/p03-trigger`'s constant-score binding
  (`decomposition.bsl:323`). Writing the latch anywhere else is writing it nowhere. This is stated in
  the rule's `:material-basis` and in D194 so it reads as a decision, not an inconsistency.

**Both-packs-world behavior, stated.** In a world loading both packs the intended shape is **one**
INSTITUTION node carrying **both** rosters — that is what the `rent-` prefix is for, and the 19
landed `institution/*` fields (`decomposition-conformance.bscn:152-170`) share no name with any
`institution/rent-*` field, so the union is clean. With exactly one INSTITUTION node the constant
score and the discriminator score agree by construction, and B8's latch is written to the node
`decomposition/p03-trigger` reads. **If a combined world ever mints two INSTITUTION nodes,
`decomposition.bsl`'s 14 constant-score reads silently tiebreak to the lowest id** — a latent hazard
in the *landed* pack that this train **records and does not repair** (repairing `decomposition.bsl`
moves its landed golden pins; that is a separate ceremony). D198 names it with its re-open trigger:
the `the` query head (`("the", "slice 2")`, `evaluator.rs:545`) is the language's own singleton-carrier
reader, unserved today; when slice 2 lands, both packs' carrier reads become `(the NodeType/INSTITUTION)`
and the discriminator retires. Task 8's combined worlds each assert `(nodes NodeType/INSTITUTION)`
has cardinality 1, and one named test seeds a **second** INSTITUTION node to prove the discriminator
still resolves this pack's carrier — the executable form of the constraint.

Carrier fields carry a **`rent-` prefix** so they cannot collide with the carceral carrier's roster
when a world loads both packs:

| field | type | frozen origin |
|---|---|---|
| `institution/rent-carrier` | `int extensive` | none — the D198 discriminator (`1` on this carrier, `0` elsewhere); a **content reformulation of node identity**, not of frozen state |
| `institution/rent-pool` | `real extensive` | `GlobalEconomy.imperial_rent_pool` / `tick_context["current_pool"]` |
| `institution/rent-tribute-inflow` | `real extensive` | `tick_context["tribute_inflow"]` |
| `institution/rent-wages-outflow` | `real extensive` | `tick_context["wages_outflow"]` — **DROPPED, see below (D199)** |
| `institution/rent-wage-rate` | `coefficient intensive` | `GlobalEconomy.current_super_wage_rate` (annual) |
| `institution/rent-repression-level` | `intensity intensive` | `GlobalEconomy.current_repression_level` |
| `institution/rent-aggregate-tension` | `coefficient intensive` | the one-tick-stale `opposition_states` read — **seeded, never written** (§4 BLOCKER-4) |

Plus B8's two latch fields, declared under `decomposition.bsl`'s own names and **not** prefixed:
`institution/superwage-crisis-known` and `institution/superwage-crisis-tick`, both `int extensive`.

**`rent-wages-outflow` is DROPPED (D199) — decided, not resolved by drift.** Frozen
`tick_context["wages_outflow"]` is a **per-tick local**: `_save_economy` (`economic.py:827-836`)
persists only `imperial_rent_pool`, `current_super_wage_rate` and `current_repression_level`, so
`wages_outflow` dies with the dict at the end of `step()` and is read by nothing — not by a later
phase, not by an event payload, not by the saved `GlobalEconomy`. Port-as-is and
declare-what-you-read genuinely conflict here and the earlier draft resolved it silently in the
direction that **changes the tick hash**: a carrier field enters graph state, and graph state enters
the hash. Decision and reasoning:

- **Declaring it would fabricate persistence the frozen engine does not have.** The faithful
  transcription of a per-tick local with no reader is *no field*, exactly as `subsidy_outflow`
  (`economic.py:61`) is not declared. A carrier field is the reformulation of *persistent* state; a
  dead local is not persistent state.
- It would additionally be **hash-bearing with no semantic content**, and would need an `r00` reset
  and an `r07` accumulation to stay correct — two effects and a fuel cost bought for nothing.
- **Consequence, stated:** the wage outflow is not observable from graph state. It stays observable
  where the frozen engine puts it — in the arithmetic. The equivalent assertion is
  `Δ rent-pool == − Σ actual_bonus_paid` across the tick, which Task 6's rows already make. Nothing
  is lost except a write-only field.
- **Re-open trigger:** if a later train needs a published per-tick outflow (a Sankey lens, an
  `observe()` page, a Vol-II circulation consumer), it declares the field **and its reader in the
  same landing**. D199 names it.

The frozen per-tick reset (`tribute_inflow=0.0` at `:59-66`) is **not** free here: carrier fields
persist. `r01`…`r07` must therefore be written so the accumulator is **reset at the top of the tick**,
not carried. The cheapest faithful shape is: `r01` `(set 0)`s `rent-tribute-inflow` before any
`(add …)` in the same rule body would fire — which is impossible in one rule under the pre-state law.
**Disposition: the reset is its own rule, `r00-tick-reset`, carrier-anchored, sorting first.** That
makes 10 rules, not 9; the reset is a real frozen behavior (the `tick_context` dict is re-created
every `step()`), so it earns a rule rather than a D-row. With `rent-wages-outflow` dropped (D199)
`r00` writes exactly one field. `rent-pool` is **not** reset — it is the persistent `GlobalEconomy`
field; neither is `rent-carrier`, which is identity, nor the two latch fields, which are latches.

**`r00` is only provable across TWO ticks, and it owes that test.** Every input to a single-tick
fixture is tick-invariant, so one tick cannot distinguish "`r00` resets correctly" from "`r00` does
nothing" — the exact shape `production.bsl`'s own `p0` reset hit, recorded at
`bsl-language.rst:6686-6697`, where breaking the reset makes the SECOND tick's accumulator exactly
**double** the first. Task 8 Step 1's `the_tick_reset_zeroes_the_accumulator_but_never_the_pool` is a
two-tick `TickSession` test, and Task 8 Step 5's mutation vector is "delete `r00`'s effect ⟹ tick 2's
`rent-tribute-inflow` is exactly twice tick 1's". Without the two-tick shape, `r00` has no mutation
killer at all.

### 3.2 The push form is FORCED — there is no endpoint accessor

`(edges EdgeType/X)` **is** a served query head (`evaluator.rs:567`), and `(for-each (edges …)
(update-edge it …) …)` is landed and tested (`edge_write_lane_e2e.rs:68-71`). It matches the frozen
`graph.query_edges(edge_type=…)` pull form exactly — **and it is unusable here**, because
`ImperialRentSystem` writes **both endpoints' `wealth`** on every phase, and:

> **the language has no `source-of`/`target-of` endpoint accessor** — recorded verbatim at
> `edge_lane_e2e.rs:186-189`: *"needing no `(edges …)` iteration and no `source-of`/`target-of`
> endpoint accessor (the language does not have one — §3.8 item 8's own open item, dossier §8)."*

So every edge-driven phase is **self-anchored**: `(for-each (neighbors self EdgeType/X :out
NodeType/SOCIAL_CLASS) …)` with the edge reached as `(edge-between EdgeType/X self it)` — the exact
idiom `edge_lane_e2e.rs:196-206` landed *"to unblock Solidarity's own port train"*, and the one
`consciousness.bsl:219-233` and `production.bsl:172-220` already use in content.

Two consequences, both D-recorded:
- **Iteration order changes.** Frozen order is `query_edges` insertion order; ported order is
  (subject-node order) × (per-node neighbour order) — and the edge lane's own documented order is
  ascending `(source, target)` (`edge_write_lane_e2e.rs:56-57`, §2.6). For per-edge-independent
  effects this is order-equivalent; for float accumulation into the carrier it is an
  associativity-order divergence, and for §3.3 it is the whole problem.
- **Anchoring choice per phase.** Phase 1 and Phase 2 anchor on the **source** (the worker / the
  comprador), because `rent` and `cut_amount` depend only on the source's own fields — so they are
  rule-scoped bindings. Phase 3 anchors on the **worker (target)** instead, because
  `productivity_value` is the worker's own `production-value`, and the single employer is reached as
  a rule-scoped expression `(select-max (neighbors self EdgeType/WAGES :in NodeType/SOCIAL_CLASS) 1)`
  — the landed `production.bsl:216` idiom. That makes `total_wages` a rule-scoped binding instead of
  an expression re-inlined into eight effect operands. Its cost is the **single-employer assumption**
  (D45/D145 class), which `production.bsl` already made and D-recorded.

### 3.3 `for-each` reads PRE-state — so sequential depletion is unrepresentable

`structural_verbs.rs:130` and `:750` state it as law, and `edge_write_lane_e2e.rs:48-52` tests it:
a `for-each` body's `emit` reads the **PRE-tick** value even though the body's own writes are
collected around it. Effects are COLLECTED, then applied. There is therefore **no running total
readable mid-loop**. Accumulation *into* a carrier works (`(update-node carrier … (add x))` inside a
`for-each` accumulates across elements — `consciousness.bsl:219-222` does exactly this); reading the
partial sum does not.

Frozen `economic.py:544` (`available_pool = tick_context["current_pool"]  # Update for next
iteration`) is precisely a mid-loop read of a running total. **It cannot port.** §4 BLOCKER-2 gives
the disposition and the measured-divergence fixture.

### 3.4 There is no binary `min`/`max`

`min`/`max` are **fold ops** (`grammar.rs:661-716`, the closed five-member `FoldOp` set
`{sum, mean, min, max, count}`), not expression heads. The `<arith>` set is exactly `+ - * /`
(`grammar.rs:723`, `const ARITH: [&str; 4]`). Every frozen `min(a,b)` / `max(a,b)` transcribes as
`(if (< a b) a b)` / `(if (> a b) a b)`, with `if` taking **exactly three operands, the else branch
mandatory** (`grammar.rs:650`, `("if", 3, 3, "exactly 3")`; a wrong count is `E-PARSE-042`), and
both branches sharing one static type (`E-TYPE-020`). Use the landed `(- 0 0c)` / `(- 1 0c)`
promotion idiom (`consciousness.bsl:255, 316`) where a branch needs a typed zero/one; **do not invent
a promotion**.

**There are TWELVE, not nine** — the earlier count was wrong and it feeds the fuel and effort
estimates. The census: TRPF floor (1) + Phase 1 `min(rent, wealth)` and the `:295` clamp (2) +
Phase 3 `min(max_bonus, pool)`, `min(total, employer_wealth)`, `min(bonus, total − productivity)`,
`max(0, actual_bonus)` (4) + Phase 5's two double-clamps (4) + `_save_economy`'s `max(0, pool)` (1)
= **12**. Eleven become 3-operand `if`s; the twelfth (`:295`) is not transcribed at all, per §1.6-a's
resolved AST and D196.

Related: **negative literals DO lex — the earlier claim was false.** `consciousness.bsl` carries
`:optional :default -1` at `:184, 185, 251, 252, 264, 265, 344, 345`. What remains genuinely
unverified is the narrower question of a negative literal in **`defconst` value position** in a
`.bscn`, which is where `austerity_wage_delta = -0.05` and `crisis_wage_delta = -0.15` would sit.
Task 0 Step 4(a) settles that one specifically; the fallback (declare positive magnitudes and apply
via `(- 0 x)` / `sub`) stands, but the risk is much smaller than the earlier draft implied and the
step should not be skipped on that account.

---

## 4. BLOCKERS — flagged, not planned around

**BLOCKER-1 (shape-forcing, resolved in §3.2): no `source-of`/`target-of`.** Disposition: the push
form, per phase-specific anchoring. Follow-on rider: §3.8 item 8's open endpoint-accessor item. Not
blocking; recorded so the pack can be simplified when the accessor lands.

**BLOCKER-2 (HARD, divergence-producing): the per-edge pool depletion (`:543-544`).**
Under the pre-state law (§3.3) every WAGES edge in one tick sees the **tick-start** pool, whereas the
frozen loop shows each edge the pool as depleted by its predecessors.

> **Disposition (no Director gate needed — this is ADR183's own "reformulation with a D-row" class,
> the same class as D165's per-node fold-sum):** transcribe the **per-edge arithmetic verbatim**
> against a **tick-start pool snapshot** (`institution/rent-pool` as written by `r00`…`r04`), and
> apply the depletion as a **batched accumulation** (`r07` subtracts every edge's `actual_bonus_paid`
> from `rent-pool`). The two are **byte-identical whenever
> `Σ max_bonus ≤ available_pool`** — i.e. whenever the pool does not bind — and diverge only when it
> does. The divergence is **measured, not hidden**: `imperial-rent-pool-exhaustion-conformance.bscn`
> (world 8) is built specifically so the pool binds across two WAGES edges, the Python mirror prints
> the frozen sequential answer, the Rust test asserts the **ported** answer, and both numbers sit
> side by side in the D183 row and in the test's own comment. A test asserting the frozen number
> would be a lie; a test with no exhaustion world would hide the defect.
> (The batched half now subtracts from `rent-pool` only — `rent-wages-outflow` is dropped, D199.)

The same class, one order down, applies **twice more**, and both instances now sit in D184:
- **Phase 1's per-worker wealth re-read.** A worker with two EXPLOITATION edges has `rent`
  recomputed from its already-decremented wealth by the frozen loop (`economic.py:283`), but the
  ported rule's `rent` is a single rule-scoped binding applied once per edge. World 8 measures it,
  and it is also the world where §1.6-a's dropped clamp becomes observable (D196).
- **Phase 2's per-edge comprador re-read.** `economic.py:375` re-reads `source_attrs["wealth"]` per
  TRIBUTE edge, so a comprador with two TRIBUTE edges takes a second cut off the **already-overwritten**
  balance (`800 → 720 → 648`); the ported rule-scoped `cut` writes `720` twice (`800 → 720`). Same
  class, previously undeclared. **World 10** (`imperial-rent-multi-tribute-conformance.bscn`, new in
  this revision) measures it, and the repeated-`set` shape it depends on is D200's own question.

And to **Phase 3's multi-employer case**, which the `select-max` single-employer assumption does not
port at all (D185) — a world with two WAGES edges into one worker is **unported behavior**, not
equivalent behavior.

**BLOCKER-3 (RESERVED — STOP): Phase 4.** See §6. Not a technical blocker; a governance one.

**BLOCKER-4 (provable-absence): `opposition_states` has no producer on the ported estate.**
`ContradictionSystem` (@18) is unported; nothing in the BSL content estate writes an
opposition/tension gap. The frozen read is *already* one-tick-stale and *already* defaults to `0.0`
when absent (`:776-780`). **Disposition:** `institution/rent-aggregate-tension` is a **seeded**
carrier field, never written by this pack — the "seed the post-something state directly" idiom
(control-ratio's `_class_decomposition_tick` precedent). D187 records the explicit **re-open
trigger**: when ContradictionSystem ports, this pack owes a producer-written-field handoff.

**BLOCKER-5 (encoding): string payload values.** `mechanism` (`"imperial_rent"`), `decision`
(`"crisis"`/`"bribery"`/…), and every `narrative_hint` are strings. There is **no `Str` variant in
`Value`**, and string literals are barred from expression position at load. **Disposition:** drop
`mechanism` and the narrative hints (AI-narration surface, not engine state); encode `decision`
numerically — `0 = no_change, 1 = bribery, 2 = austerity, 3 = iron_fist, 4 = crisis`, transcribed in
the declaration order of `BourgeoisieDecision` (`dynamic_balance.py:18-22`), mapping recorded in D188
and pinned by a test. Encode `bourgeoisie_active` as `0/1`. Rejected alternative, recorded not
silent: minting `(defenum BourgeoisieDecision …)` — the most faithful shape, but the pack emits the
code in exactly one payload and a new enum is new hash-bearing vocabulary for one payload key; note
it in D188 as the shape to adopt if a reviewer prefers it.

**BLOCKER-5b (encoding, previously undeclared): the id→NodeRef payload key rename.** The frozen
payload keys are `source_id`/`target_id` (Phase 1, `:337-344`) and `payer_id`/`receiver_id`
(Phase 3, `:472-485`), all carrying **strings**; the ported payloads use `source`/`target`/`payer`/
`receiver` carrying **NodeRefs**. The earlier draft's D188 enumerated `mechanism`, `narrative_hint`,
`decision` and `bourgeoisie_active` but **not** this rename, so every conformance test asserting
`source`/`payer` was asserting an undeclared divergence. `decomposition.bsl`'s D171 item 1 is the
precedent that *does* record it (`decomposition.bsl:114-123`: *"`<payload-item>` values are flat
`<expr>` (number/bool/enum-ref/NodeRef) — no dict, no string"*). **Disposition:** the rename is
forced (there is no `Str` payload value), it is declared in D188 alongside the other four, and the
dropped `_id` suffix is stated as deliberate — a NodeRef is not an id string, and keeping the suffix
would misdescribe the value's type. §2.1 covers the separate question of `payer`'s *retention*, which
is a scope decision, not an encoding one.

**BLOCKER-6 (provable-absence): the two spec-063 sub-stages and the L-RECEIPTS register.**
`_invoke_phi_distribution_if_wired` (`:88-156`), `_invoke_vol2_circulation_if_wired` (`:158-199`) and
the Phase-1 `services.boundary_register.record(...)` (`:310-321`) are all **silent no-ops in every
unit test and every qa-six scenario** — none binds their `context.persistent_data` keys or a session
register. Their inputs (`boundary_flow_register`, `session_id`, `external_nodes_phi`,
`county_exposure_by_external`, `vol2_step`, `simulated_year`) have **no BSL lane at all** — the
"no lane" family. **Disposition:** omitted, D192, each with a named re-open trigger (the phi
distribution rides `#563`'s Φ estate; the Vol-II circulation and the boundary register ride the
session-infrastructure lane). The mirrors must run with no register bound so the frozen stdout shows
the no-op path, and the mirror header must say so.

**NOT blockers — facts verified for this train (2026-08-18):**
- The carrier idiom needs **no spike**: `(select-max (nodes NodeType/INSTITUTION) 1)` reads and
  writes, and `(fold sum (nodes NodeType/SOCIAL_CLASS) …)` are all landed in `decomposition.bsl`.
- `update-edge` from a **content pack** is net-new (only `edge_write_lane_e2e.rs`, `r9_chapters.rs`
  and a bsl conformance fixture use it today) — Task 1 Step 5 spikes it against a real `.bscn`-seeded
  edge attribute in a NEW namespace, before four rules depend on it.
- **`EventType` needs no declaration** — the kind stays inert when a scenario declares no `EventType`
  vocabulary, and no landed `.bscn` declares one. `SURPLUS_EXTRACTION`, `SUPERWAGE_CRISIS` and
  `ECONOMIC_CRISIS` emit with **zero** Rust changes beyond the one registration string.
- **Events are observable and pinnable** — `run_once_into(SCENARIO, RULE, &mut graph, &mut sink)`
  exposes a `CollectingSink` whose `events: Vec<(String, Vec<(String, Value)>)>` is asserted
  key-by-key in landed tests.
- **`deffield` surface:** exactly `(deffield <qname> <type> <intensive|extensive>)` or `(deffield
  <qname> enum <EnumTypeName>)` — a positional 4-tuple with no keyword slot (`scenario.rs:1113-1166`);
  accepted type tokens are `int, real, probability, intensity, coefficient, currency, enum`;
  **`bool` is not accepted** by the scenario loader (it *is* accepted by the rule-file
  `deffield`, `declarations.rs:665-696` — a divergence between the two loaders, noted, not used).
  An `int` deffield **stores verbatim f64** and constrains *seeding* only.
- **`:optional` / `:default` ARE available — on RULE BINDINGS, and the earlier draft was wrong to say
  otherwise.** §2.5's `<bind-opt> ::= ":optional" | ":default" <literal>` (`bsl-language.rst:845-846`)
  is landed and heavily used: `consciousness.bsl` carries `:optional :default -1`, `0`, `0.0p` and
  `0.5i` across `p0`, `p4`, `p5`, `p6` and `p7`. The keywords are barred only from the `deffield`
  *declaration*. Two consequences this plan now depends on:
  1. **A sentinel default is a legal absence encoding**, so "every absence encoding is a companion
     known-flag" is false and is struck.
  2. **`r06`'s `production-value` binding is `:optional :default 0`** — that is the *faithful*
     transcription of `la_production.get(edge.target_id, 0.0)` (`economic.py:453`), a `.get` with a
     `0.0` default. It is port-as-is, and it is what makes §7's unseeded-`production-value`
     constraint test executable (I7). Task 0 Step 4(f) confirms the default literal's type against a
     `real extensive` field (`consciousness.bsl:271` uses a bare `0` against `social-class/wealth`).
- **A REQUIRED binding is TICK-FATAL on an absent field** for a same-subject-type node
  (`bindings.rs`'s resolve-or-error law, quoted at `consciousness.bsl:301`) — it is not a skip. Every
  non-optional binding this pack writes therefore imposes a seeding obligation on every scenario that
  loads it, combined worlds included (§2.2's `economics/fundamental-theorem` note).
- **`update-node`/`update-edge` ops** are the closed four-member set `add | sub | set | scale`
  (`grammar.rs:722`); a fifth symbol is `E-PARSE-015`.

**Type traps to verify at Task 1, not at Task 7:** `Int ÷ Int` is a **loud error** (truncation is
never implicit) — `pool_ratio = rent-pool / initial-rent-pool` must have a Real numerator;
`:tick` binds the driver's tick number and every comparison mixing it with a field-sourced value
needs checking once, up front; `E-LEX-023` caps literals at 9 fractional digits and `E-LEX-024`
bounds suffixed literals (`0.0005` for `trpf-coefficient` is inside both); `fold sum` refuses an
intensive field (`E-TYPE-041`) — irrelevant here only if no fold touches an intensive, so check
before adding one; `exists` appears in landed content with both one and two operands — copy the
neighbouring pack, do not mix.

**One more trap, new in this revision and load-bearing for `r03` (D200): what does a REPEATED `set`
of one field within one tick do?** `r03-tribute`'s `(update-node self social-class/wealth (set cut))`
sits inside a `for-each` over TRIBUTE neighbours, so a comprador with **two** TRIBUTE edges collects
**two** `set` effects on the same field in the same tick, both carrying the same pre-state-derived
value. Three questions the effect applier must answer before `r03` is written, and none of them is
answered by any landed pack (no landed rule `set`s one field twice in a tick):
1. Is a duplicate `(set …)` on one (node, field) accepted, or is it a collision refusal?
2. If accepted, is last-write-wins, first-write-wins, or order-dependent-and-therefore-hash-relevant?
3. Does the answer differ for `add`/`sub` (which are commutative and obviously accumulate)?
**Task 0 Step 4(e) settles all three at the byte; Task 1 Step 5's spike proves the accepted answer
against the real driver.** If the answer is "refusal", §8's `r03` shape is re-planned before Task 3 —
not worked around inside it. The frozen behavior it is being compared against is itself divergent:
`economic.py:375` re-reads `source_attrs["wealth"]` per edge, so a two-TRIBUTE comprador takes a
second cut off the **already-overwritten** balance (`800 → 720 → 648`), while the ported rule-scoped
`cut` writes `720` twice (`800 → 720`). That divergence is the same class as D184's and is now
recorded there explicitly.

---

## 5. The transcendentals verdict — PURE ARITHMETIC, the pack declares no intrinsics

**Verdict: PURE-ARITHMETIC. This pack declares NO intrinsics — not `exp`, not `log`, not `floor`.**
`#576`'s `DECLARABLE_INTRINSICS` estate and D175's libm crossing contract are **not needed** and must
not be cited as a dependency.

The evidence, in three independent strands:

1. **The Leontief/gamma estate is not on this system's path at all.** The gamma sweep found the
   whole tensor pipeline (`np.linalg.inv`, `np.sum`, the ERDI/QCEW allocation) belongs to
   `TickDynamicsSystem` @4.0's `imperial_rent.compute()` — a different module (§0 row 2) —
   and that path is itself transcendental-free (`+ − × ÷` and `max(x,0)` clamps only; zero hits for
   `exp|log|sqrt|pow|math\.|np\.exp|np\.log|scipy` across all nine files on it). Its real hazards
   (LAPACK LU pivot order, `np.sum` pairwise summation, an unordered SQL feeding dict-order float
   accumulation, and the post-2017 ERDI coverage cliff that makes roughly the back half of a
   canonical run take the `_stub_zero_pass_through` freeze branch) are **`#563`'s problems, not
   this train's** — recorded here only so a reader does not import them.
2. **ImperialRentSystem @9.0's own four ported phases are pure arithmetic.** Every formula in §1.2–
   §1.5 is `+ − × ÷`, comparisons, and `min`/`max` clamps. The one registry formula they use,
   `calculate_bourgeoisie_decision`, is a priority-ordered if/elif over string constants
   (`dynamic_balance.py:82-118`) — no libm call anywhere in it.
3. **The only `exp` on the frozen surface is Phase 4's, and Phase 4 does not land.** Phase 4 calls
   `calculate_acquiescence_probability` — `1/(1+exp(−k·(wealth−subsistence)))` with the exponent
   clamped to `[−500,500]` (`survival_calculus.py:21-43`). **ADR202 R3 rules that logistic
   INHERITED by ADR173's emergent-measure construct: it must NOT be transcribed verbatim.** So even
   if Phase 4 landed, the `exp` would not cross — and `sigmoid` is a prohibited BSL intrinsic name
   (`E-LOAD-024`), with spelling the logistic out of `exp` + arithmetic named as the same prohibited
   motion. Phase 4's other formula, `calculate_revolution_probability`
   (`min(1.0, cohesion/(repression+EPSILON))`, `survival_calculus.py:46-65`), is pure arithmetic.

This closes the question ADR210 left open (*"ImperialRent on `#576` with its `exp`/`log` dependency
UNVERIFIED — to be settled at its own train's archaeology (it may decouple)"*) and confirms
ADR213's independent errata against the port-estate survey's rows 83/197. **It decouples.**
Task 9's ADR214 records the closure and cross-references the survey errata.

---

## 6. RESERVED LINE — Phase 4 (CLIENT_STATE subsidy) STOPS. It does not land.

**This is not a scoping preference. It is a governance STOP, and no task may route around it.**

| what | status |
|---|---|
| **M1 — the `P(S\|A)` call is INHERITED** (ADR202 R3; ADR173 decision (1)). The frozen logistic MUST NOT be transcribed. `P(S\|A)` re-derives as "the measure of class members whose wealth clears subsistence." | binding, and it means **the frozen Phase-4 gate has no transcription** |
| **M2 — landing is GATED on a D-record**: `population` and `inequality` must be seeded on CLIENT_STATE-edge-target `SocialClass` nodes before the measure is evaluable. Today every canonical scenario builds the comprador with `population=1, inequality=0.0` by omission, degenerating the measure to a step function — *"strictly worse than the curve it replaces."* | **not built** |
| **R1 — OPEN, UN-RULED**: whether the comprador `SocialClass` node is the correct carrier for "the client state" as a whole, and therefore whether `repression_boost`/`repression_faced` (`economic.py:640`) lands on the correct side of the relation (the comprador itself vs. the periphery proletariat). Raised in the T4 curves session; **absent from ADR202 R3's actual ruling text**; no ADR or spec rules on it. | **escalate** |
| **R2 — OPEN**: the within-class wealth-dispersion **family** (uniform vs. a smooth heavy-tail, …) that parameterizes the emergent measure — it decides whether the gate is a ramp, a step, or a smooth S-curve. The dossier presents uniform as the minimum-assumption *instance*, explicitly **not** as a proposal to adopt: *"Choosing it is a claim about how wealth disperses within a class — theory, hence Director-reserved."* | **escalate** |
| **R3 — binds M2**: the `#510` provisional income-shape proxy's Director-mandated expiry now reaches into the population/inequality seeding this gate needs. | **escalate with M2** |
| **M3 — `steepness_k` removal** from `rust/crates/babylon-kernel/tests/fixtures/canonical_defines.json` is owed **at the inherited measure's landing**. | **NOT discharged by this train, and must NOT be done early** — the Phase-4 frozen mirror will still need it |

**What this train does instead, and nothing more:**
1. Ports Phases 1, 2, 3, 5 + the pool save/decay. Phase 4 is absent from the pack, absent from every
   scenario, and absent from every declaration (B6).
2. Lands **D186**, the STOP row: it states Phase 4 is unported; quotes M1/M2/M3 with their ADR202 R3
   and ADR173 citations; states R1 and R2 verbatim as **OPEN and un-ruled**; **RECORDS** the ADR202
   R3 client-state seeding D-record and states in the same sentence that **the gate M2 names stays
   OPEN**; names R3 as binding it; and states that a world with CLIENT_STATE edges is **unported
   behavior, not equivalent behavior**.

   > **Wording, and it is not a nicety.** M2's text is a **landing gate** — `population` and
   > `inequality` must be seeded on CLIENT_STATE-edge-target `SocialClass` nodes *before the measure
   > is evaluable*. Writing "discharged" into the register invites the next train to treat the gate
   > as satisfied by the existence of a paragraph. The correct verbs are **recorded** and
   > **restated**; the gate is **NOT discharged by this train**. Match the phrasing this plan already
   > uses correctly for M3 (`steepness_k`), which it explicitly refuses to discharge early. Any
   > artifact this train produces — D186, ADR214, the follow-on issue — uses "recorded", never
   > "discharged", for M2.

3. Names the follow-on issue explicitly: a Phase-4 train **cannot open** until R1 and R2 are ruled.
   Task 9 Step 4 files it, blocked, with the three questions as its body — it is a Director-facing
   artifact, and it must **describe, never propose**.
4. **Records R4 as CONSIDERED AND DECLINED.** ADR171's national-question axis (the MIM+MLP line,
   bribe:deprivation = 1.55, `colonial_stance` as principal contradiction) is a **separate construct**
   from the Phase-3 super-wage and the Phase-4 CLIENT_STATE subsidy; no ADR and no `#564` row couples
   them, and the adjudicated survey's reserved-line column for row 9.0 names **only** the ADR172/173
   sigmoid question. Coupling the bribe:deprivation ratio into the Phase-3 super-wage would be
   **NEW reserved scope** (Constitution I.1, the principal-contradiction axis). **This train does not
   do it, and D186 says so** — a reviewer must be able to confirm the reserved line was seen and
   declined, not merely unmentioned. This matters more than usual here because §2.2 shows the pack
   *does* feed a chauvinism term (`consciousness/p6-route`'s `chauvinist`) — the point is precisely
   that it feeds it through the **already-landed** `wage-balance` path with no new coupling minted.

**S-guidance recorded but not adopted** (workforce-licensed defaults, consistent with the record, all
Phase-4 scoped and therefore inert for this train): the dossier's illustrative population-weighted
`fold mean` sketch for the measure's within-block shape is *"illustrative, not typechecked"* — do not
silently promote it; conformance vectors for the reformulated `P(S|A)` compare against the emergent
formulation's own vectors, **never Python replay** (the frozen lane diverges BY DESIGN); and the
form-pinning tests that encode the retired logistic (midpoint value, steepness monotonicity) retire
**as a class** at that port rather than transcribing.

---

## 7. Byte-order hazard, stated explicitly

Execution order today is **ascending rule-id byte order across all loaded packs** (register rows
D16/D100); `(anchor :after …)` validates shape only and is inert for ordering until Phase 3.

**The landed rule-id namespaces, corrected and verified (2026-08-18).** The earlier draft named two
namespaces that do not exist. A rule-id's namespace is the segment before the first `/` **in the
`(rule …)` form**, which is independent of the filename. The complete sorted list — every
`(rule …)` header in all 13 `content/rules/*.bsl` files:

`consciousness`, `control-ratio`, `decomposition`, `dispossession`, `economics`, `lifecycle`,
`metabolism`, `organization`, `production`, `solidarity`, `territory`, `vitality`.

- **`fundamental-theorem/` is not a namespace.** `fundamental-theorem.bsl`'s only rule is
  `economics/fundamental-theorem`.
- **`worldview/` is not a namespace.** `worldview.bsl`'s only rule is
  `consciousness/worldview-mint-probe` — two files share the `consciousness` namespace.

So `imperial-rent/…` sorts **after** `consciousness/`, `control-ratio/`, `decomposition/`,
`dispossession/`, `economics/` and **before** `lifecycle/`, `metabolism/`, `organization/`,
`production/`, `solidarity/`, `territory/`, `vitality/`. The earlier draft's *conclusion* survives
(`economics` < `imperial-rent` < `lifecycle`), but it was presented as verified when it was not.

Against the frozen tick order this inverts **four** real dependencies, not two. Each is disclosed,
constrained, and closed by an executable constraint test, in the shape the Decomposition+ControlRatio
train's §5 established: name the datum, prove benign or state the harm, name the exact condition
under which the harm occurs, and land a test that fails if the condition is violated.

#### 7.1 `production/` — ImperialRent reads `social-class/production-value`

- **Frozen:** Production @3.0 → ImperialRent @9.0. **Ported:** `imperial-rent` < `production`, so
  imperial-rent runs first and Phase 3 reads a **stale** `production-value`.
- **Harm condition:** a combined world where `production-value` moves tick-to-tick. On a world where
  it is seeded and never rewritten, the inversion is unobservable.
- **Constraint:** every conformance scenario **seeds `social-class/production-value` directly**
  rather than relying on same-tick production — the "seed the post-something state" idiom
  control-ratio used for `_class_decomposition_tick`, and what makes the single-tick goldens
  meaningful. The pack declares `(anchor :after production)` **documentary** — shape-validated, inert
  today, load-bearing when Phase 3 ordering lands.
- **Executable constraint (Task 8 Step 3, world 11):** a combined `production` + `imperial-rent`
  world run for **two** ticks with `production-value` **unseeded**. Tick 1 must show Phase 3 paying
  only the super-wage bonus (productivity 0, via `r06`'s `:optional :default 0` binding — the
  faithful transcription of `la_production.get(…, 0.0)`); tick 2 must show it paying
  `production.bsl`'s tick-1 output. One tick cannot distinguish the two orders; two can. **This is
  the I7 repair**: the earlier draft's single-tick unseeded fixture was described as unexecutable
  under a "no-defaults law" the plan had itself mis-stated — absence is legal, and the optional
  binding with a `0` default is both the frozen transcription and the lever that makes the fixture
  run.

#### 7.2 `consciousness/` — this pack writes what four consciousness rules read (§2.2)

- **Frozen:** ImperialRent @9.0 → Consciousness @16. **Ported:** `consciousness` < `imperial-rent`,
  so **all four readers run before the writer**: `p0-position` (`:185`), `p4-wage-balance` (`:252`),
  `p5-agitation` (`:265`), `p7-persist-baselines` (`:345`), plus `p2-wages-push` (`:232-234`) on
  `wages/value-flow`.
- **Harm:** every §2.2 seam is **exactly one tick late**. Tick 1 consciousness reads the scenario
  seeds; from tick 2 it reads this pack's writes. Nothing is lost, but nothing is same-tick.
- **Constraint:** a combined world must seed `wages-paid`, `value-produced` and `wages/value-flow`
  to the values that make tick 1 meaningful, and no single-tick golden may be read as proof of the
  seam.
- **Executable constraint (Task 5 Step 6 + Task 8 Step 3, world 12):** the seven §2.2 conformance
  rows, all of them on a **two-tick** `TickSession` over a combined
  `consciousness` + `imperial-rent` world. `the_wage_flow_is_live_across_two_ticks` is the one that
  fails if the inversion is ever mis-stated as same-tick.

#### 7.3 `decomposition/` — the SUPERWAGE_CRISIS latch (§2.1)

- **Frozen:** ImperialRent @9.0 emits, Decomposition @11 reads the same tick's event history and
  takes `min(e.tick)`; ImperialRent's emit therefore both **feeds** the delay clock and
  **suppresses** Decomposition's own early-warning emit (`decomposition.py:179`'s
  `superwage_tick is None` gate).
- **Ported:** `decomposition` < `imperial-rent`, so `decomposition/p02-superwage-warning` runs
  **first**, reads `crisis-known = 0`, emits its own SUPERWAGE_CRISIS and latches
  `(set 1)`/`(set tick)`. `r05` then finds `crisis-known = 1` and its own `(= crisis-known 0)`-guarded
  latch write correctly does nothing. **Net ported behavior in a combined world: the latch is set on
  the right tick with the right value, but by the WRONG rule, and the payload that reaches the bus is
  `p02`'s three-key shape rather than `r05`'s seven-key shape** (both may be emitted; only the latch
  is single-writer).
- **Harm condition, named precisely:** the inversion is **benign for the delay clock** (both rules
  latch the same `tick`, and `p03-trigger` reads only `known` and `tick`) and **harmful for any
  consumer that reads the PAYLOAD**. Today there is no such consumer in the BSL estate, so the harm
  is latent. It becomes real the moment a rule or an `observe()` page keys on `payer` /
  `super-wage-bonus` / `available-pool`.
- **It is also harmful in the other direction if `r05` does NOT latch at all** — which is what the
  earlier draft would have shipped. In a world where the LA is *not* approaching subsistence (so
  `p02`'s guard is closed) but the **pool is exhausted** (so `r05`'s guard is open), the frozen
  engine starts the decomposition delay clock and the un-latched port does not. That path is
  **silently dead**, and it is the pool-exhaustion → LA-decomposition causal path — the one the
  frozen system exists to model.
- **Executable constraint (Task 5 Step 7 + Task 8 Step 3, world 12):** a combined
  `decomposition` + `imperial-rent` world seeded so that **`p02`'s guard is closed and `r05`'s is
  open** (LA wealth comfortably above `subsistence + 2·consumption`; `rent-pool ≤ negligible-rent`).
  Assert: (a) `institution/superwage-crisis-known` becomes `1` and `-tick` becomes the emitting tick —
  **the test that fails if `r05` does not latch**; (b) `decomposition/p03-trigger` fires
  `decomposition-delay` ticks later; (c) the mutation "drop `r05`'s latch effect" flips (a) and (b).
  Plus the converse world where **both** guards are open, asserting `-tick` holds `p02`'s value and
  not a later overwrite — **the test that fails if `r05`'s latch write loses its
  `(= crisis-known 0)` guard**, i.e. the test that pins `min` against `last`.

#### 7.4 `economics/` — the half-anchored fundamental-theorem guard (§2.2)

- **Frozen:** no dependency at all — `ImperialRentSystem` never imports `formulas/fundamental_theorem.py`
  and `economics/fundamental-theorem` mirrors `ContradictionSystem` @18's downstream stash. The
  inversion is an artifact of the port, not of the frozen order.
- **Ported:** `economics` < `imperial-rent`, so the theorem reads the previous tick's
  `value-produced` against a fixture-seeded `social-class/wages`.
- **Harm condition:** a combined world where `value-produced` moves. The guard then compares a live
  quantity against a frozen one and `social-class/imperial-rent` becomes a hybrid.
- **Constraint:** **this train does not build a combined `economics` + `imperial-rent` world.**
  Becoming `social-class/wages`'s producer is out of scope (B2), so a combined world would pin a
  half-anchored result as if it were a contract. D195 records the asymmetry and its re-open trigger
  (the ContradictionSystem @18 port, or a re-homing of the theorem's inputs) instead. Stating a
  non-construction as a deliberate decision is the point; silently not building it is what the
  earlier draft did.

Within the pack, rule ids are chosen so byte order equals frozen phase order (§8), and **every
same-tick dependency is a deliberate D116 reliance** — named in the pack header's ordering map **and
recorded as its own register row, D197** (see §8's D116 ledger).

---

## 8. Rule layout — 10 rules, one pack

One frozen system → **one pack**, `content/rules/imperial-rent.bsl`, namespace `imperial-rent/`.
(Contrast the Decomposition+ControlRatio train, which was two packs *because* it was two systems.
One pack here also removes the cross-pack ordering hazard entirely — the only inversions left are the
inter-pack ones of §7.)

| id | subject | frozen site | does |
|---|---|---|---|
| `r00-tick-reset` | INSTITUTION | `:59-66` | `(set 0)` on `rent-tribute-inflow` — the per-tick re-creation of the `tick_context` dict. **Does not touch `rent-pool`, `rent-carrier`, or the two latch fields.** One effect, not two (`rent-wages-outflow` dropped, D199). Only provable across two ticks — §3.1. |
| `r01-extraction` | SOCIAL_CLASS (worker) | `:239-345` | rule-scoped `eff` (weekly ÷ then TRPF-multiplied, floor-clamped by `if`), `rent = min(eff·wealth·(1−consciousness), wealth)`; `when` self `active = 1`; `for-each` over `(neighbors self EdgeType/EXPLOITATION :out NodeType/SOCIAL_CLASS)` guarded on `it` active: **`(update-node self social-class/wealth (sub rent))` — the §1.6-a resolved AST, the frozen `max(0.0, ·)` NOT transcribed, D196** — `(update-node it social-class/wealth (add rent))`, `(update-edge (edge-between EdgeType/EXPLOITATION self it) exploitation/value-flow (set rent))`, and a nested guard `(> rent negligible-rent)` → `(emit EventType/SURPLUS_EXTRACTION …)`. |
| `r02-extraction-credit` | SOCIAL_CLASS (worker) | `:324-329` | same `for-each`, guarded on `it` active **and** `(= (field-of it social-class/role) SocialRole/CORE_BOURGEOISIE)` → `(add rent)` into both `institution/rent-tribute-inflow` and `institution/rent-pool` on the **discriminator-scored carrier** (§3.1). Split from `r01` so the role gate is independently mutation-killable and so each rule's fuel bound stays tractable — the split's duplication cost is closed by the ledger below. |
| `r03-tribute` | SOCIAL_CLASS (comprador) | `:347-400` | `cut = wealth · comprador-cut`, `tribute = wealth − cut`; `when` self active `= 1` **and** `wealth > 0`; `for-each` `(neighbors self EdgeType/TRIBUTE :out …)` guarded on `it` active: `(update-node self social-class/wealth (set cut))` — **the §1.6-c OVERWRITE, verbatim; the repeated-`set` shape is D200's open question, settled at Task 0 Step 4(e) and spiked at Task 1 Step 5 BEFORE this rule is written** — `(update-node it social-class/wealth (add tribute))`, `(update-edge … tribute/value-flow (set tribute))`. **No emit.** |
| `r04-tribute-credit` | SOCIAL_CLASS (comprador) | `:395-400` | the CORE_BOURGEOISIE-gated carrier credit of `tribute`, mirroring `r02` on the discriminator-scored carrier. |
| `r05-wages-crisis` | SOCIAL_CLASS (worker) | `:462-489` | **no `active` gate at all** — the frozen emit-before-skip, verbatim (§1.4). `when` an `:in` WAGES neighbour exists; `bonus = min(tribute-inflow · wage-rate ÷ weeks, pool)`; guard `(<= pool negligible)` **and** `(<= bonus negligible)` → **two** effects: (1) `(emit EventType/SUPERWAGE_CRISIS …)` with `payer` = the `select-max` employer ref, `receiver` = self, plus `productivity-value`, `super-wage-bonus`, `available-pool`, `bourgeoisie-wealth`, `bourgeoisie-active` (0/1); (2) **the B8 latch stamp** — `(update-node (select-max (nodes NodeType/INSTITUTION) 1) institution/superwage-crisis-known (set 1))` and `… superwage-crisis-tick (set tick)`, **both behind a `(= crisis-known 0)` conjunct in the same guard** so first-writer-wins reproduces the frozen `min(e.tick)` (§2.1, D194). Note the deliberate **constant-score** carrier expression here, matching `decomposition.bsl:266,269` byte for byte — §3.1 explains why this one write does not use the discriminator. |
| `r06-wages-pay` | SOCIAL_CLASS (worker) | `:507-536` | `total = min(production-value + bonus, employer-wealth)` with **`(binding production-value :field social-class/production-value :optional :default 0)`** — the faithful transcription of `la_production.get(edge.target_id, 0.0)` (`:453`); `when` employer exists **and** employer `active = 1` **and** self `active = 1` **and** employer `wealth > 0`; effects: employer `wealth (sub total)`; self `wealth (add total)`, `effective-wealth (set …)`, `unearned-increment (set …)`, `ppp-multiplier (set …)`, `wages-paid (set total)`, `value-produced (set production-value)`; `(update-edge (edge-between EdgeType/WAGES employer self) wages/value-flow (set total))`. **Three of these seven writes are the §2.2 producer seam (D195) — `wages-paid`, `value-produced`, `wages/value-flow`.** |
| `r07-wages-pool` | SOCIAL_CLASS (worker) | `:540-543` | `abp = max(0, min(bonus, total − production-value))`; same gate as `r06`; `(sub abp)` from `rent-pool` on the discriminator-scored carrier. **BLOCKER-2's batched half.** One effect, not two (D199). |
| `r08-decision` | INSTITUTION | `:668-750`, `dynamic_balance.py:82-118` | `pool-ratio = rent-pool ÷ initial-rent-pool` (guarded `> 0`, else 0); the §1.5 five-branch matrix as a nested `if` chain in **exactly that priority order**; `new-wage-rate` and `new-repression` double-clamped by `if`; writes `rent-wage-rate`, `rent-repression-level`; guard `decision = CRISIS` → `(emit EventType/ECONOMIC_CRISIS …)` with the numeric decision code (BLOCKER-5). |
| `r09-pool-decay` | INSTITUTION | `:827-836` | `(set (if (< decayed 0) (- 0 0c) decayed))` where `decayed = rent-pool · (1 − rent-pool-decay)` — the decay and the `max(0, ·)` in one write, **after** `r08` reads the pool. |

### 8a. The duplication ledger — every shared expression and the row that proves the copies agree (D201)

The `r01`/`r02`, `r03`/`r04` and `r05`/`r06`/`r07` splits mean four frozen expressions are
transcribed more than once. That is a real hazard: a mutation to `r01`'s `(1 − consciousness)` factor
is caught by a named test, but **drift between `r01`'s `rent` and `r02`'s `rent` is caught by
nothing**. The earlier draft flagged the split only as a review preference.

**The choice, and why: keep the split, add copies-agree rows. Single-sourcing is not available in the
language.** A `.bsl` file's top-level forms are the closed set
`rule | deffield | intrinsic-decl | manifest | metric-decl` (`bsl-language.rst:650-652`); a `.bscn`'s
are `defenum | defvocabulary | deffield | defconst | node | edge | edge-attr` (`scenario.rs:561-628`).
There is **no `defexpr`, no macro, no cross-rule `let`**, and a rule's `:expr` bindings are private to
that rule. So "single-source" here means exactly one thing — **merge the rules** — and merging costs
what the split bought: `r02`'s role gate and `r03`'s `set`-vs-`sub` asymmetry stop being
independently mutation-killable, and the merged rules' fuel bounds compound. The copies-agree row is
cheaper than either loss, and it is *observable*: each duplicate has a distinct graph write, so
asserting the two writes agree asserts the two transcriptions agree.

| duplicated expression | in | copies-agree row | asserts |
|---|---|---|---|
| `rent` | `r01`, `r02` | `r01_and_r02_agree_on_the_rent` | `Δ(rent-tribute-inflow) == Δ(core-bourgeoisie wealth)` bit-exact on world 1, where the only EXPLOITATION target IS the core bourgeoisie |
| `cut` / `tribute` | `r03`, `r04` | `r03_and_r04_agree_on_the_tribute` | `Δ(rent-tribute-inflow) == Δ(recipient wealth) == (seed − post-tick comprador wealth)` bit-exact |
| `super-wage-bonus` | `r05`, `r06`, `r07` | `r05_r06_and_r07_agree_on_the_bonus` | on a crisis-free world: `wages-paid − value-produced == −Δ(rent-pool before r09)` bit-exact — the bonus computed three times, observed once through each rule's own write |
| `total-wages` | `r06`, `r07` | covered by the row above | `r07`'s `abp` is a function of `r06`'s `total`; agreement is implied by the identity |

Each row is a **first-class conformance test**, not an assertion tucked into another test, so a
reviewer can find them by name. Mutation evidence: perturb the factor in ONE copy only (e.g. change
`r02`'s `rent` to omit `(1 − consciousness)`) and the agreement row must flip while every
single-rule row stays green — that is what proves the row is catching drift rather than restating a
transfer test.

### 8b. The D116 ledger — every same-tick cross-rule read this pack relies on (D197)

`bsl-language.rst:6184-6210` (D116) is a **RECORDED GAP**, not a semantics guarantee:
`run_once_into`/`TickSession::advance` run each rule to **completion — collect and apply —** before
the next starts, against the same mutable graph, so rule N+1 observes rule N's already-applied writes
from this tick. Its own text defers the repair ("collect-across-rules-then-apply … carrying the same
golden-baseline exposure") **to its own train**. The earlier draft named the reliance in the pack
header only; **no row recorded the exposure or its re-open trigger**, and the `r02`/`r04` split
*increases* it. Under ADR183 the D-row is exactly the required artifact.

| reader | reads | written same tick by | breaks how, when D116 is repaired |
|---|---|---|---|
| `r05` | `institution/rent-tribute-inflow` | `r02`, `r04` | reads last tick's inflow ⟹ `bonus` computed from a stale tribute base |
| `r05`, `r06` | `institution/rent-pool` | `r02`, `r04` | reads the pre-credit pool ⟹ the `min(max_bonus, pool)` cap binds a tick early |
| `r02`, `r04` | `institution/rent-tribute-inflow` (as the `add` target) | `r00`'s reset | the reset does not apply ⟹ the accumulator compounds across ticks (the exact failure `production.bsl`'s `p0` row records at `bsl-language.rst:6693-6697`) |
| `r07` | `institution/rent-pool` | `r02`, `r04` | depletion applied to a stale base |
| `r08` | `institution/rent-pool` | `r02`, `r04`, `r07` | `pool-ratio` computed on last tick's pool ⟹ **the decision branch can differ** |
| `r09` | `institution/rent-pool` | `r08` (reads, does not write) + `r07` | decay applied to a stale base |

**Six same-tick cross-rule reads. When the D116 repair lands, every one of them silently reads
pre-tick state and this pack's arithmetic goes wrong without a single transfer test flipping** —
because every transfer test's inputs are tick-invariant. The only guards are the ordering vectors
(`r09_decays_the_pool_after_the_decision_reads_it`, `the_tick_reset_zeroes_the_accumulator_but_never_the_pool`),
which is thin. So:

- **D197 records the whole ledger**, names the Q14 repair train as the re-open trigger, and states
  that **this row is that train's acceptance-criterion input for imperial-rent specifically** —
  the phrasing `production.bsl`'s own row uses (`bsl-language.rst:6698-6702`), and it is not
  optional: a stale row feeds a wrong acceptance criterion into a future train.
- **Every row in the ledger is named individually in the pack header's ordering map**, not summarised.
- **Task 8 adds a two-tick arc assertion per accumulator** (`rent-tribute-inflow`, `rent-pool`) so
  the compounding failure mode has a killer, not just the reset's own vector.

**`defconst` block** — every value from `src/babylon/data/defines.yaml`'s `economy:` section
(`:70-99`), with an inline `file:line` citation per row (the `control-ratio-conformance.bscn:102-107`
discipline). **Mint no new coefficient.**

| qname | value | source |
|---|---|---|
| `economy/extraction-efficiency` | `0.8` | `defines.yaml:71` |
| `economy/comprador-cut` | `0.9` | `:72` |
| `economy/super-wage-rate` | `0.2` | `:74` (the *seed* for `rent-wage-rate`; the live value is the carrier field) |
| `economy/superwage-multiplier` | `1.0` | `:75` |
| `economy/superwage-ppp-impact` | `0.5` | `:76` |
| `economy/initial-rent-pool` | `100.0` | `:77` |
| `economy/pool-high-threshold` | `0.7` | `:78` |
| `economy/pool-low-threshold` | `0.3` | `:79` |
| `economy/pool-critical-threshold` | `0.1` | `:80` |
| `economy/min-wage-rate` | `0.05` | `:81` |
| `economy/max-wage-rate` | `0.35` | `:82` |
| `economy/negligible-rent` | `0.01` | `:86` |
| `economy/trpf-coefficient` | `0.0005` | `:90` |
| `economy/rent-pool-decay` | `0.002` | `:91` |
| `economy/bribery-wage-delta` | `0.05` | `:92` |
| `economy/austerity-wage-delta` | `-0.05` | `:93` — **sign handling per Task 0 Step 4** |
| `economy/iron-fist-repression-delta` | `0.1` | `:94` |
| `economy/crisis-wage-delta` | `-0.15` | `:95` — same |
| `economy/crisis-repression-delta` | `0.2` | `:96` |
| `economy/bribery-tension-threshold` | `0.7` | `:97` — **shipped 0.7 governs, not the signature's 0.3** (§1.6-d) |
| `economy/iron-fist-tension-threshold` | `0.5` | `:98` |
| `economy/trpf-efficiency-floor` | `0.1` | `:99` |
| `timescale/weeks-per-year` | `52` | `:374` |

Companion scenarios MAY vary a threshold or a seed to make a branch reachable at tick 1 — the landed
dispossession practice (7 companion `:const` environments). **Constraint:** no companion may make a
decision branch reachable by changing the **priority order** of the matrix; only the inputs move.

**Class fields.** Reused (do NOT re-mint): `social-class/wealth` (`real extensive`),
`social-class/active` (**`int intensive`**, 0/1 — no `bool` on the seed dialect; corrected against
`consciousness-ternary-conformance.bscn:227`, where the earlier draft said `extensive` while
claiming verbatim reuse), `social-class/role` (`enum SocialRole`), `social-class/production-value`
(`real extensive`), `social-class/wages-paid`, `social-class/value-produced` (both `real extensive`
here — see B5, D191, §2.2).
Net-new: `social-class/class-consciousness` (`coefficient intensive`, pending B7's verification),
`social-class/effective-wealth` (`real extensive`), `social-class/unearned-increment`
(`real extensive`), `social-class/ppp-multiplier` (`coefficient intensive`).
Edge attrs: `wages/value-flow` (`real intensive`, **reused**), `exploitation/value-flow` and
`tribute/value-flow` (both `real intensive`, net-new; seeded via the D156 `(edge-attr …)` form).
Carrier: the seven `institution/rent-*` fields of §3.1 plus B8's two unprefixed latch fields.

---

## 9. The frozen-mirror recipe — the exact graph-attr seeds, verified against the frozen entry points

**This section exists because the earlier draft's mirror recipe seeded only `la_production`, and
would therefore have printed a DIFFERENT WORLD's oracle for six of the nine worlds it planned —
which the Rust tests would then have been pinned to.** "Paste the mirror stdout verbatim" is a discipline for
transcription fidelity; it cannot catch a mirror that faithfully reports the wrong world. Every
mirror in this train follows the recipe below, and each mirror's header states which of the four
seeds it sets and to what.

**Three graph attributes and one context field decide the world.** Verified against the frozen
reads, 2026-08-18:

| seed | frozen reader | shape | what happens if a mirror omits it |
|---|---|---|---|
| `"economy"` | `_load_economy` (`economic.py:794-805`) — `graph.get_graph_attr("economy")`, then `GlobalEconomy.model_validate(…)` | a `GlobalEconomy(…).model_dump()` dict | falls back to `imperial_rent_pool = defines.economy.initial_rent_pool` (**100.0**), `current_super_wage_rate = defines.economy.super_wage_rate` (**0.2**), `current_repression_level = defines.survival.default_repression` (**0.5**, `defines.yaml:167`). **Worlds 3-8 are DEFINED by a non-default pool and are unreachable without this seed.** |
| `"la_production"` | `:438`, `:453` — `la_production.get(edge.target_id, 0.0)` | `dict[str, float]`, keys are **worker node ids** (`production.py:194`) | every `productivity_value` is `0.0` |
| `"opposition_states"` | `_calculate_aggregate_tension` (`:776-780`) — `states.get("capital_labor")` then `.get("gap", 0.0)` | `dict[str, dict]`; the reader does raw `.get()`s with **no `model_validate`**, so `{"capital_labor": {"gap": G}}` is behaviorally exact | tension is `0.0`. **IRON_FIST requires `tension > 0.5` and is UNREACHABLE at 0** — world 6 cannot exist without this seed |
| `TickContext.persistent_data` | the two spec-063 sub-stages (`:88-156`, `:158-199`) | defaults to `{}` (`context.py:51`) | correct: `{}` is the no-op path BLOCKER-6 relies on. **Leave it empty and say so in the header.** |

**The canonical mirror preamble** — every mirror in this train opens with this block, adapted only in
its values:

```python
from babylon.models.entities.economy import GlobalEconomy   # economic.py:15's own import

graph.set_graph_attr("economy", GlobalEconomy(
    imperial_rent_pool=<POOL>,              # == the .bscn's institution/rent-pool seed
    current_super_wage_rate=<WAGE_RATE>,    # == institution/rent-wage-rate
    current_repression_level=<REPRESSION>,  # == institution/rent-repression-level
).model_dump())
graph.set_graph_attr("la_production", {<worker-id>: <PRODUCTION>})  # == social-class/production-value
graph.set_graph_attr("opposition_states", {"capital_labor": {"gap": <TENSION>}})  # == institution/rent-aggregate-tension
```

Four rules that make this exact rather than approximate:

1. **Always seed `"economy"` explicitly; never let the fallback run.** The fallback reads three
   defines, so a future defines edit would silently move the oracle. `GlobalEconomy`'s
   `model_config` sets `populate_by_name=True` (`models/entities/economy.py`), so the
   `model_dump()` → `model_validate()` round-trip is exact despite the
   `super_wage_rate`/`repression_level` validation aliases — construct by **field name**, dump, seed.
2. **World 1 seeds `rent-repression-level` = `0.5`, not `0.3`.** The earlier draft's Task-1 carrier
   table said `0.3` while the frozen fallback default is `0.5` (`defines.yaml:167`). Aligning world 1
   on `0.5` makes the explicit seed and the fallback agree, so a reviewer comparing the two can never
   be reading two different worlds. Worlds that need a different repression level (world 6, the
   repression-clamp fixtures) seed **both sides** to the same varied value and say so in their header.
3. **Every seed above has a `.bscn` counterpart, and the pairing is checked in the plan text, not
   assumed.** Task 1 Step 4 and Task 7 Step 3 each carry an explicit four-row table —
   `.bscn` field ⟷ mirror seed ⟷ value — and the reviewer's lens (Task 4 Step 7) names checking it.
4. **`_save_economy` applies `rent_pool_decay` whenever `services is not None`** (`:827-836`), so the
   mirror's post-tick `economy` print is **already decayed**. Compare it against `r09`'s output, not
   `r07`'s. State this in each mirror header; it is the single easiest place to mis-read the oracle.

**Per-world seed matrix** (the `.bscn` values are the same numbers; derive nothing, verify against
the mirror):

| world | `economy` pool | wage rate | repression | `opposition_states` gap | why |
|---|---|---|---|---|---|
| 1 primary | 100.0 | 0.2 | 0.5 | 0.0 | all four phases, NO_CHANGE, the non-binding control |
| 2 TRPF | 100.0 | 0.2 | 0.5 | 0.0 | multi-tick decay + floor clamp |
| 3 superwage crisis | ≤ `negligible-rent` | 0.2 | 0.5 | 0.0 | exhausted pool + inactive employer |
| 4 CRISIS | < 10.0 (ratio < 0.1) | 0.2 | 0.5 | 0.0 | branch 1 — and low tension, so BRIBERY's tension clause would pass were priority reversed |
| 5 BRIBERY | ≥ 70.0 (ratio ≥ 0.7) | 0.2 | 0.5 | 0.0 (< 0.7) | branch 2 |
| 6 IRON_FIST | 10.0 ≤ p < 30.0 | 0.2 | 0.5 | **0.6** (> 0.5) | branch 3 — **unreachable at gap 0** |
| 7 AUSTERITY | 10.0 ≤ p < 30.0 | 0.2 | 0.5 | **0.5** (not > 0.5) | branch 4, the exact-threshold witness |
| 8 pool exhaustion | seeded so `Σ max_bonus > pool` | 0.2 | 0.5 | 0.0 | BLOCKER-2's measured divergence + the two-EXPLOITATION-edge worker (D184/D196) |
| 9 arc | 100.0 | 0.2 | 0.5 | 0.0 | the multi-tick circuit |
| 10 multi-tribute | 100.0 | 0.2 | 0.5 | 0.0 | the two-TRIBUTE-edge comprador (D184 Phase 2, D200) |
| 11 production-combined | 100.0 | 0.2 | 0.5 | 0.0 | §7.1's two-tick ordering constraint |
| 12 consciousness+decomposition-combined | per §7.2/§7.3 | 0.2 | 0.5 | 0.0 | §2.1 and §2.2's seams |

**Do not derive the branch boundaries from this table** — `initial_pool` is `defines.economy.initial_rent_pool`
(100.0) and the ratio thresholds are 0.1/0.3/0.7, so the pool figures above follow; verify each
against the mirror's printed `pool_ratio` before pinning anything.

---

## File Structure

| File | Responsibility |
|---|---|
| Create `reports/imperial-rent-bsl-surface-facts-2026-08-18.md` | Task 0's dossier |
| Modify `rust/crates/babylon-tick/src/lib.rs` | one registration string in `prepare_rules`' system `HashSet` |
| Create `rust/crates/babylon-tick/content/rules/imperial-rent.bsl` | the pack: 10 rules + the `defconst` block + the file-local `D-N` header |
| Create `content/scenarios/imperial-rent-conformance.bscn` + `imperial_rent_conformance.py` | world 1 (all four phases, NO_CHANGE) + the primary mirror (drives worlds 1,2,3,8) |
| Create `content/scenarios/imperial-rent-trpf-conformance.bscn` | world 2 — multi-tick TRPF decay + the floor clamp |
| Create `content/scenarios/imperial-rent-superwage-crisis-conformance.bscn` | world 3 — exhausted pool + an INACTIVE employer (the emit-before-skip vector) |
| Create `content/scenarios/imperial-rent-decision-{crisis,bribery,iron-fist,austerity}-conformance.bscn` + `imperial_rent_decision_conformance.py` | worlds 4-7 — one per decision branch + their shared mirror |
| Create `content/scenarios/imperial-rent-pool-exhaustion-conformance.bscn` | world 8 — BLOCKER-2's measured divergence (two WAGES edges, binding pool; two EXPLOITATION edges off one worker, which is also §1.6-a/D196's reachability fixture) |
| Create `content/scenarios/imperial-rent-multi-tribute-conformance.bscn` | **world 10 (new)** — the two-TRIBUTE-edge comprador; D184's Phase-2 half and D200's repeated-`set` shape |
| Create `content/scenarios/imperial-circuit-arc-conformance.bscn` + `imperial_circuit_arc_conformance.py` | world 9 — the multi-tick circuit (inflow → outflow → decay → decision) + its mirror |
| Create `content/scenarios/imperial-rent-production-order-conformance.bscn` | **world 11 (new)** — combined `production` + `imperial-rent`, `production-value` unseeded, §7.1's two-tick ordering constraint |
| Create `content/scenarios/imperial-rent-combined-seams-conformance.bscn` + `imperial_rent_combined_seams_conformance.py` | **world 12 (new)** — combined `consciousness` + `decomposition` + `imperial-rent`; §2.1's latch seam and §2.2's producer seam, plus the second-INSTITUTION-node discriminator vector (D198). Its mirror chains the frozen systems in @9.0 → @11 → @16 order, the `carceral_arc_conformance.py:173-174` precedent |
| Create `rust/crates/babylon-tick/tests/imperial_rent_conformance.rs` | worlds 1,2,3,8,10 + mutation vectors + the 8a copies-agree rows |
| Create `rust/crates/babylon-tick/tests/imperial_rent_decision_conformance.rs` | worlds 4-7 + branch vectors |
| Create `rust/crates/babylon-tick/tests/imperial_circuit_arc_conformance.rs` | world 9 via `TickSession` + the 8b two-tick accumulator assertions |
| Create `rust/crates/babylon-tick/tests/imperial_rent_seams_conformance.rs` | **new** — worlds 11 and 12: §7.1-§7.3's executable constraints, §2.2's seven conformance rows, the D198 discriminator vector |
| Modify `rust/crates/babylon-tick/tests/tick_goldens.rs` | additive pins (**12 new**); the 16 pre-existing pins untouched; this train's own pins re-measured at each PR boundary |
| Modify `docs/reference/bsl-language.rst` | register rows **D181-D201** (tail verified **D180** on 2026-08-18; re-check before allocating) |
| Create `ai/decisions/ADR214_imperial_rent_port_handoff.yaml` + `index.yaml` row | handoff record (index tail was **ADR213**; verify) |
| Modify `ai/state.yaml` | closing entry |

---

### Task 0: Surface-facts dossier — the owed re-reads, the two seams, and the seven sign/shape questions

**Files:** Create `reports/imperial-rent-bsl-surface-facts-2026-08-18.md`.

**Interfaces:** Produces the CORRECTIONS section every later task cites, and settles four questions
this plan deliberately left open rather than guessing.

- [ ] **Step 1: The edge lane's owed read** — `babylon-bsl/src/query.rs` (`materialize_edges`,
      `:135, :282`), `evaluator.rs` (`SERVED_QUERY_HEADS :567`, `UNSERVED_EXPRESSION_HEADS :544`),
      `structural_verbs.rs` (`for_each :473-570`, the pre-state law at `:130` and `:750`, the
      `update-edge` arm). Record verbatim: the served heads, the `for-each` arity form
      (`(for-each <query> <elem-name>? <effect-item>+)`), the pre-state paragraph, and the exact
      absence of an endpoint accessor (`edge_lane_e2e.rs:186-189`).
- [ ] **Step 2: The write lane's owed read** — `edge_write_lane_e2e.rs` in full (the `update-edge it`
      shapes, both write kinds, the emit-reads-pre-state test) and `edge_lane_e2e.rs`'s self-anchored
      `neighbors`+`edge-between` vector. Record the exact `(edge-attr EdgeType/X <from> <to> <qname>
      <value>)` seeding form from `consciousness-ternary-conformance.bscn:309-313`.
- [ ] **Step 3: Discharge B7** — grep `content/rules/consciousness.bsl` and every `.bscn` for a
      landed scalar class-consciousness qname. If one exists, the dossier fixes it as the qname
      `r01` reads and this plan's `social-class/class-consciousness` is **struck**. If none exists,
      record the absence with the grep as evidence.
- [ ] **Step 4: Settle the SEVEN open shape questions at the byte**, each with the source line that
      decides it:
      **(a)** do negative literals lex in **`defconst` value position** in a `.bscn`, or must
      `austerity-wage-delta` / `crisis-wage-delta` be declared positive and applied via `(- 0 x)` /
      `sub`? (Narrowed, not dropped: negative literals demonstrably lex in **binding-default**
      position — `consciousness.bsl:184,185,251,252,264,265,344,345` all carry `:default -1` — so
      the earlier "no landed content uses a negative literal / the `(- 0 x)` idiom is universal"
      claim is FALSE. The `defconst` position specifically is still unverified.)
      **(b)** does a `for-each` body admit a nested `(guard <pred> <effect>+)` with **multiple**
      effects? (`r05` now needs **three** — one `emit` plus two latch writes.)
      **(c)** does `(field-of it social-class/role)` inside a `for-each` guard typecheck against an
      enum-ref equality (D102 is **discharged** — `field-of` over an `:enum-type` field typechecks
      and evaluates; the two surviving refusals are `E-TYPE-016`, an enum `field-of` as a
      `select-max`/`select-min` **score**, and the static half of `E-EVAL-042`, arithmetic on an
      enum field — neither is used here)?
      **(d)** what is the result type of `(field-of (select-max …) …)` when the field is
      `real extensive`, and does `pool-ratio`'s division avoid the `Int ÷ Int` loud error?
      **(e) — NEW, D200, blocking `r03`:** what does a **repeated `set` of one field within one
      tick** do? Read the effect applier's collision handling in `structural_verbs.rs` and answer
      all three of §4's questions (accepted or refused; if accepted, which write wins; whether
      `add`/`sub` differ). No landed pack `set`s one field twice in a tick, so this is genuinely
      unprecedented content.
      **(f) — NEW, blocking `r06` and every combined world:** confirm `:optional :default <literal>`
      on a binding over a **`real extensive`** field, and which literal form the default takes
      (`0` vs `0.0c` vs `0.0`). `consciousness.bsl:271` binds `social-class/wealth` with a bare `0`;
      confirm that shape and confirm the consumers' `:default -1` sentinel still typechecks when the
      field is re-declared `real extensive` rather than `int extensive` (B5).
      **(g) — NEW, D198, blocking the carrier idiom:** confirm a bare
      `(field-of it institution/rent-carrier)` is legal as a `select-max` **score**
      (`query_lane_e2e.rs:252-253` and `r9_chapters.rs:1129-1130` are the landed precedents;
      `E-TYPE-016` refuses only a **Bool/Enum/reference** score, `typecheck.rs:460-499`). If it
      refuses, record the refusal and adopt the `territory.bsl:133-136` `if`-chain score instead. If
      **both** refuse: STOP and re-plan §3.1.
- [ ] **Step 5: Verify the register/ADR tails and the D181-D201 allocation** —
      `docs/reference/bsl-language.rst`'s last row (**D180** on 2026-08-18, contiguous D152→D180,
      verified twice) and `ai/decisions/index.yaml`'s tail (**ADR213**, verified). This train needs
      **21 contiguous rows, D181-D201**. If either tail moved, the dossier fixes the new allocation
      and every later task uses it.
- [ ] **Step 6: Verify the registration symbol spelling** — confirm the rule-id first segment admits
      a hyphen (`imperial-rent`); the landed `control-ratio` registration is the precedent, so this
      should confirm, but confirm it rather than assume.
- [ ] **Step 7 — NEW: Transcribe the two cross-pack seams' landed shapes into the dossier**, so
      Tasks 5 and 8 copy rather than re-derive: (a) `decomposition.bsl:104-113` (D168's prescription),
      `:247-271` (`p02-superwage-warning` in full — its guard, its three-key payload, its two latch
      writes and its constant-score carrier expression), `:274-294` (`p03-trigger`'s
      `delay-elapsed-fire` binding), `:114-123` (D171 item 1's `payer_id` reasoning), and the
      `deffield`/seed lines for both latch fields (`decomposition-conformance.bscn:152-153, 271-272`);
      (b) `fundamental-theorem.bsl` in full (12 lines), and `consciousness.bsl`'s `p0-position`,
      `p4-wage-balance`, `p5-agitation`, `p7-persist-baselines`, `p2-wages-push` and `p6-route`'s
      `chauvinist`/`eff-sol`/`delta-r`/`delta-f` bindings. **Correct in the dossier**, with the
      evidence, that the pair-binding rules are `p0`/`p4`/`p5`/`p7` and **not** `p6`/`p8` — `p6`
      consumes only the derived `social-class/wage-balance` and `p8` binds neither field.
- [ ] **Step 8 — NEW: Record the corrected namespace list** (§7's twelve) with the `(rule …)` header
      of every one of the 13 `content/rules/*.bsl` files as evidence, so the `fundamental-theorem/`
      and `worldview/` errors cannot recur.
- [ ] **Step 9: Commit** `docs(port): imperial-rent BSL surface-facts dossier (the edge-lane, the
      sign/shape re-reads and the two cross-pack seams)`.

**Gate:** none (docs only) — but run `vale` over the dossier.
**Estimate:** ~2h.

### Task 1: Registration + the carrier/scenario ceremony + THE SPIKE (PR A)

**Files:** Modify `rust/crates/babylon-tick/src/lib.rs`; create
`content/scenarios/imperial-rent-conformance.bscn`, `content/scenarios/imperial_rent_conformance.py`,
`rust/crates/babylon-tick/tests/imperial_rent_conformance.rs`.

**Interfaces:** Produces the node ids, the carrier field roster, the edge seeding, and the mirror
numbers every later task asserts against.

- [ ] **Step 1: Failing load-smoke test** — `imperial_rent_conformance.rs` with
      `const SCENARIO: &str = include_str!(…)` and a test `scenario_and_empty_pack_load` calling the
      real loader with an empty rule source. Expected: FAIL (unregistered system / missing scenario).
- [ ] **Step 2: Register the system** — add `"imperial-rent".to_owned()` to the `HashSet` with a
      comment citing **Material Base @9.0**, exactly as the landed `"control-ratio"` / `"production"`
      rows do.
- [ ] **Step 3: Write `imperial-rent-conformance.bscn`** — the primary world. Declarations per §8
      (all class fields, the **seven** `institution/rent-*` carrier fields **plus B8's two unprefixed
      latch fields**, all 23 defconsts, the three edge namespaces).
      Declaration order is NodeId order — declare in this order and never renumber when extending:

  | node | type | role | active | wealth | production-value | class-consciousness | notes |
  |---|---|---|---|---|---|---|---|
  | `core-bourgeoisie` | SOCIAL_CLASS | CORE_BOURGEOISIE | 1 | 10000 | 0 | 0 | extraction target, tribute recipient, wage payer |
  | `periphery-worker` | SOCIAL_CLASS | PERIPHERY_PROLETARIAT | 1 | 500 | 0 | 0.2 | EXPLOITATION source; non-zero consciousness makes the `(1 − c)` factor provable |
  | `comprador` | SOCIAL_CLASS | COMPRADOR_BOURGEOISIE | 1 | 800 | 0 | 0 | TRIBUTE source; non-zero wealth makes the §1.6-c `set`-vs-`sub` mutation provable |
  | `labor-aristocracy` | SOCIAL_CLASS | LABOR_ARISTOCRACY | 1 | 300 | 40 | 0 | WAGES target; **`production-value` SEEDED** (§7) |
  | `petty-b` | SOCIAL_CLASS | PETTY_BOURGEOISIE | 1 | 250 | 0 | 0 | the non-participant witness — no edge touches it, and **every** field of it must be unchanged post-tick |
  | `imperial-rent-register` | INSTITUTION | — | — | — | — | — | carrier: `rent-carrier 1`, `rent-pool 100`, `rent-tribute-inflow 0`, `rent-wage-rate 0.2`, **`rent-repression-level 0.5`**, `rent-aggregate-tension 0`, `superwage-crisis-known 0`, `superwage-crisis-tick 0` |

  Edges: `(edge EdgeType/EXPLOITATION periphery-worker core-bourgeoisie)`,
  `(edge EdgeType/TRIBUTE comprador core-bourgeoisie)`,
  `(edge EdgeType/WAGES core-bourgeoisie labor-aristocracy)`, each with its `(edge-attr …
  <ns>/value-flow 0)` seed.

  **Three seeding facts, corrected:**
  - **`rent-repression-level` is `0.5`, not `0.3`** — it matches `defines.survival.default_repression`
    (`defines.yaml:167`), which is what `_load_economy`'s fallback would construct, so the explicit
    seed and the fallback can never describe two different worlds (§9 rule 2).
  - **`rent-wages-outflow` is not declared** (D199).
  - **The seeding law is not "every declared field on every node".** Absence is legal: a field may be
    omitted on a node, and `consciousness-ternary-conformance.bscn:287-288, 300-301` deliberately
    omits `wages-paid`/`value-produced` on two nodes as "the anchorless witness". What is fatal is a
    **required** binding over an absent field (`bindings.rs`'s resolve-or-error law). So the rule is:
    **seed every field this pack binds NON-optionally, on every node of that binding's subject type.**
    `production-value` is bound `:optional :default 0` and is therefore the one field a world may
    legitimately omit — which world 11 does. Fractional seeds on `int` fields still refuse.
- [ ] **Step 4: Write the frozen mirror** `imperial_rent_conformance.py` **per §9's canonical
      preamble** — import `ImperialRentSystem` and `GlobalEconomy`, build the identical graph
      node-for-node **and edge-for-edge in the same order** with `BabylonGraph()`,
      `services = ServiceContainer.create()` (which wires the real `formula_registry`, so
      `bourgeoisie_decision` resolves, and leaves `boundary_register = None`), then seed **all three**
      graph attributes:

  | `.bscn` seed | mirror seed | world-1 value |
  |---|---|---|
  | `institution/rent-pool` / `rent-wage-rate` / `rent-repression-level` | `graph.set_graph_attr("economy", GlobalEconomy(imperial_rent_pool=…, current_super_wage_rate=…, current_repression_level=…).model_dump())` | `100.0` / `0.2` / `0.5` |
  | `social-class/production-value` on `labor-aristocracy` | `graph.set_graph_attr("la_production", {"labor-aristocracy": 40.0})` | `40.0` |
  | `institution/rent-aggregate-tension` | `graph.set_graph_attr("opposition_states", {"capital_labor": {"gap": 0.0}})` | `0.0` |

      Then run one `ImperialRentSystem().step(graph, services, context)` with `TickContext(tick=1)`
      and print: the defines block; per-node post-tick `wealth`/`effective_wealth`/
      `unearned_increment`/`ppp_multiplier`/`w_paid`/`v_produced`; every edge's `value_flow`; the
      post-tick `graph.get_graph_attr("economy")`; and `services.event_bus.get_history()` with full
      payloads. The header must state **three** things: (a) **no boundary register is bound**
      (BLOCKER-6) and `persistent_data` is `{}` (D192), so the stdout shows the no-op path; (b) the
      printed `economy` is **already decayed** by `_save_economy` (`:827-836`), so it corresponds to
      `r09`'s output and not `r07`'s; (c) the full `OppositionState` shape
      (`domain/dialectics/core/opposition.py:275-297`) is deliberately elided to `{"gap": …}` because
      `_calculate_aggregate_tension` does raw `.get()`s with no `model_validate` — behaviorally exact,
      and recorded so a later reader knows what was left out. Run it, paste stdout **verbatim +
      dated** into the Rust test's doc comment.
- [ ] **Step 5: THE SPIKE — prove seven shapes from a content pack, before eight rules depend on
      them.** A throwaway spike rule (deleted at the end of this step, verdict recorded in the
      scenario header) proving, against the real driver and this `.bscn`: (a) a `for-each` over
      `(neighbors self EdgeType/EXPLOITATION :out NodeType/SOCIAL_CLASS)` fires; (b)
      `(update-edge (edge-between EdgeType/EXPLOITATION self it) exploitation/value-flow (set 1))`
      writes a **`.bscn`-seeded edge attribute in a brand-new namespace**; (c) the same body writes
      BOTH `(update-node self …)` and `(update-node it …)`; (d) a nested
      `(guard <pred> <effect> <effect> <effect>)` with **three** effects loads and runs (`r05` needs
      three); (e) a `(field-of it social-class/role)` enum equality inside that guard typechecks;
      **(f) — NEW: `(select-max (nodes NodeType/INSTITUTION) (field-of it institution/rent-carrier))`
      resolves this pack's carrier in a world seeded with a SECOND INSTITUTION node whose
      `rent-carrier` is 0** (D198's whole disposition rests on this, and Task 0 Step 4(g)'s reading
      is not the same as running it); **(g) — NEW: a repeated `(set …)` of one field within one
      `for-each`** behaves as Task 0 Step 4(e) predicted (D200, blocking `r03`). **If any refuses:
      STOP, record the refusal text and its `E-` code, and re-plan §8's rule split before Task 2** —
      do not attempt a workaround inside a later task.
- [ ] **Step 6: Load-smoke green** + a `defenum` ordinal-parity test mirroring the mint's + **the
      carrier singleton/discriminator test** (`(nodes NodeType/INSTITUTION)` cardinality, and the
      two-node discriminator vector from Step 5(f), kept as a permanent test rather than deleted with
      the spike). Pin, by name in the test header, that this is the first **content** pack to use
      `update-edge`, the first to write `wages/value-flow`, and the first to resolve a carrier by a
      declared discriminator rather than a constant score.
- [ ] **Step 7: Commit** `test(tick): imperial-rent conformance scenario + frozen mirror + system
      registration`.

**Gate:** six cargo legs; the 16 existing golden pins byte-identical.
**Estimate:** ~4h (the seven-shape spike dominates).

### Task 2: `r00` + `r01` + `r02` — the tick reset and the extraction phase

**Files:** Create `content/rules/imperial-rent.bsl` (header + `r00` + `r01` + `r02` + the `defconst`
block); extend `tests/imperial_rent_conformance.rs`.

**Interfaces:** Produces `institution/rent-tribute-inflow` and the first `rent-pool` credit — read by
`r05`, `r06`, `r07`, `r08`.

- [ ] **Step 1: Failing tests** — `r01_extracts_the_frozen_rent_from_the_active_worker` (worker
      wealth decremented, bourgeoisie wealth incremented, both bit-exact against the mirror via the
      `.to_bits()` idiom); `r01_applies_the_weekly_conversion_before_the_trpf_multiplier`;
      `r01_writes_the_exploitation_value_flow`;
      `r01_emits_surplus_extraction_above_the_negligible_floor` (payload key-by-key: `source` NodeRef,
      `target` NodeRef, `amount`; **no** `mechanism` key — BLOCKER-5; and the `source_id`→`source`
      rename is BLOCKER-5b, declared, not incidental);
      `r01_skips_an_inactive_counterparty`; `r02_credits_only_a_core_bourgeoisie_target`
      (`rent-tribute-inflow` and `rent-pool` both moved by exactly `rent`);
      `the_petty_bourgeois_witness_is_untouched`; **`r01_and_r02_agree_on_the_rent`** (§8a's
      copies-agree row).
- [ ] **Step 2: Write the pack header** — the §0 name-collision paragraph first; then the file-local
      `D-N` block (**reserve rows for all twenty-one** of Task 9 Step 1); then the byte-order map
      `r00 → r01 → … → r09` with **every one of §8b's six same-tick D116 dependencies named
      individually**, not summarised; then §7's **four** inter-pack inversion disclosures and their
      constraints; then the `defconst` table with its `defines.yaml` line citations.
- [ ] **Step 3: Write `r00-tick-reset`** — carrier-anchored, unconditional, **one** `(set 0)` write
      (`rent-tribute-inflow`; `rent-wages-outflow` is dropped, D199), on the discriminator-scored
      carrier.
- [ ] **Step 4: Write `r01-extraction`** per §8. The TRPF floor is an `if`, not a `max`. **The
      `:295` clamp is NOT transcribed** — `(update-node self social-class/wealth (sub rent))` is the
      resolved AST (§1.6-a, D196). Write the reachability proof into the rule's `:material-basis` and
      add `r01_never_drives_a_single_edge_worker_negative` as the **converse** row (the proof that on
      a one-edge world `wealth − rent ≥ 0` holds without a clamp); the two-edge case where it does
      NOT hold is world 8's, measured in Task 6.
- [ ] **Step 5: Write `r02-extraction-credit`** per §8.
- [ ] **Step 6: Measure fuel** for all three rules per the §4.5 readback discipline; record the
      `E-LOAD-040` bound and the `B+1` figure in the commit body. Expect the hundreds-to-thousands
      band, not the tens.
- [ ] **Step 7: Mutation — five vectors, `r00` included** — drop `r01`'s `(1 − consciousness)`
      factor → `r01_extracts_the_frozen_rent…` flips; swap the TRPF `if` comparison →
      `r01_applies_the_weekly_conversion…` flips (add a tick>1 assertion in Task 4 if nothing flips
      at tick 1, and **record that** rather than moving on); change `r02`'s role gate to
      `PETTY_BOURGEOISIE` → `r02_credits_only_a_core_bourgeoisie_target` flips; change `>` to `>=`
      on the negligible-rent emit gate → a boundary fixture must flip (add it here if absent);
      **perturb `r02`'s `rent` transcription ONLY** (omit its `(1 − consciousness)` factor) →
      `r01_and_r02_agree_on_the_rent` flips while every single-rule row stays green — the vector that
      proves the copies-agree row catches drift rather than restating a transfer test.
      **`r00`'s own vector is deferred to Task 8** and named there, because it is only killable
      across two ticks (§3.1) — record that deferral in this commit body rather than shipping `r00`
      with no vector and no note.
      Restore byte-identical each time.
- [ ] **Step 8: Six legs + commit** `feat(tick): imperial-rent r00-r02 — the tick reset, the
      extraction transfer and the tribute-inflow credit`.

**Estimate:** ~4h.

### Task 3: `r03` + `r04` + world 10 — the tribute phase and its multi-edge divergence

**Files:** Modify `content/rules/imperial-rent.bsl`; create
`content/scenarios/imperial-rent-multi-tribute-conformance.bscn`; extend
`tests/imperial_rent_conformance.rs` and the primary mirror.

**Precondition:** Task 0 Step 4(e) and Task 1 Step 5(g) must both have ANSWERED the repeated-`set`
question (D200). If either is unanswered, **STOP** — `r03`'s shape depends on it.

- [ ] **Step 1: Failing tests** — `r03_overwrites_the_comprador_wealth_with_the_cut`
      (`wealth == 800 · 0.9 == 720` exactly, **not** `800 − 720`; this is the §1.6-c defect vector);
      `r03_transfers_the_remainder_to_the_recipient`; `r03_writes_the_tribute_value_flow`;
      `r03_skips_a_non_positive_comprador` (a second comprador seeded `wealth 0`);
      `r03_emits_nothing` (the phase has no event — assert the event count is unchanged);
      `r04_credits_the_pool_and_the_tribute_inflow`; **`r03_and_r04_agree_on_the_tribute`** (§8a);
      and on world 10: **`the_two_tribute_edges_apply_the_rule_scoped_cut_once`** — asserting the
      **ported** number (`800 → 720`) with the **frozen** sequential number (`800 → 720 → 648`) in
      the assertion message and in a comment, exactly the D183 publication discipline.
- [ ] **Step 2: Write world 10** — `imperial-rent-multi-tribute-conformance.bscn`: one comprador with
      **two** TRIBUTE edges to two distinct recipients. Its header states, in numbers, what the frozen
      loop does (`economic.py:375`'s per-edge `source_attrs["wealth"]` re-read) and what the pack
      does, and names D184's Phase-2 half and D200. **Extend the primary mirror to drive it** and
      re-paste stdout verbatim + dated; the mirror's comprador number will NOT match the Rust
      assertion, and the test file's header must say so in a dedicated paragraph.
- [ ] **Step 3: Write `r03-tribute` and `r04-tribute-credit`** per §8, using the repeated-`set`
      semantics D200 recorded. Measure fuel against worlds 1 **and** 10 (worst-case ceiling).
- [ ] **Step 4: Mutation** — swap `r03`'s `(set cut)` for `(sub cut)`:
      `r03_overwrites_the_comprador_wealth_with_the_cut` flips (this is the mutation the non-zero
      comprador seed exists for). Swap `(add tribute)` for `(set tribute)` on the recipient: the
      transfer test flips. Perturb `r04`'s `tribute` transcription only:
      `r03_and_r04_agree_on_the_tribute` flips while the single-rule rows stay green.
      Restore byte-identical each time.
- [ ] **Step 5: Six legs + commit** `feat(tick): imperial-rent r03/r04 — the comprador cut, the
      tribute transfer, its pool credit and the measured multi-edge divergence`.

**Estimate:** ~3h.

### Task 4: TRPF + the inflow goldens — PR A close

**Files:** Create `content/scenarios/imperial-rent-trpf-conformance.bscn`; modify
`tests/tick_goldens.rs`, `tests/imperial_rent_conformance.rs`,
`content/scenarios/imperial_rent_conformance.py`.

- [ ] **Step 1: Failing tests** — `the_trpf_multiplier_decays_with_the_tick` (a `TickSession` run to
      a tick where `1 − coefficient·tick` is measurably below 1, extraction bit-exact against the
      mirror at that tick); `the_trpf_multiplier_clamps_at_the_efficiency_floor` (a tick past
      `(1 − floor)/coefficient` — **derive the tick from the arithmetic and verify it against the
      mirror; do not trust this plan's numbers**).
- [ ] **Step 2: Write the TRPF world** and extend the mirror to drive it (a second `main()` section
      printing the same census at the two chosen ticks). Re-paste the mirror stdout verbatim + dated.
- [ ] **Step 3: Add the three PR-A golden pins** — `imperial_rent_conformance`,
      `imperial_rent_trpf_conformance` and `imperial_rent_multi_tribute_conformance`, each pinning
      `hex(before)`, `hex(after)` and `report.fired` with the firing arithmetic **verified, not
      trusted**. Measured, never derived. The 16 pre-existing pins stay untouched. **Write into each
      new pin's own comment that it covers `r00`-`r04` only and WILL be re-measured at PR B and again
      at PR C** — so the next agent reads the motion as scheduled, not as a regression (the Global
      Constraints' two-obligation split).
- [ ] **Step 4: Mutation** — raise `trpf-efficiency-floor` above the decayed value:
      `the_trpf_multiplier_clamps_at_the_efficiency_floor` flips. Restore byte-identical.
- [ ] **Step 5: Re-measure fuel** for `r01`-`r04` against the **new** worst-case ceiling across all
      three scenarios (§4.5's worst-case rule) and update the declarations if the bound moved.
- [ ] **Step 6: Six legs + commit** `test(tick): imperial-rent TRPF decay world + the inflow golden
      pins`.
- [ ] **Step 7: Open PR A** `feature/imperialrent-port-bsl`, Tasks 0-4 (five commits). Review lens:
      (a) transcription fidelity against §1.2/§1.3 line by line; (b) the spike's **seven** verdicts
      and the carrier/edge idioms — including the D198 discriminator, which is this PR's one genuinely
      novel shape; (c) the boundary rules B1-B10, and specifically that no `social-class/imperial-rent`
      write and no `production-value` recomputation appears; (d) the §9 seed table — check the
      `.bscn` ⟷ mirror pairing row by row rather than trusting the verbatim paste.
      Harvest the Copilot review (every inline comment gets a fix or a reply); merge via
      `mise run pr:merge -- N`.

**Estimate:** ~2.5h + review.

### Task 5: `r05` + `r06` + the two cross-pack seams — the wages phase (PR B)

Branch off **merged dev** as `feature/imperial-circuit-port-bsl`. Never stack on PR A (#193).
**No Rust source edits** — Task 1 registered the system.

**This task lands BOTH Criticals: `r05`'s latch stamp (§2.1, D194) and `r06`'s producer status
(§2.2, D195).** It is the largest task in the train.

- [ ] **Step 0 (PR-B opening step): re-measure PR A's three pins.** `r05`-`r07` change world 1's and
      world 10's post-tick hash and firing count; world 2's too. Run `run_once` for each, paste the
      new measured values, and record the per-rule-id `fired` delta in the commit body. This is the
      **declared** re-measure of the Global Constraints' obligation 2 — it is expected, and it is not
      a STOP. The 16 pre-existing pins are re-verified byte-identical in the same step; **those**
      moving IS a STOP.

**Files:** Modify `content/rules/imperial-rent.bsl`; create
`content/scenarios/imperial-rent-superwage-crisis-conformance.bscn`,
`content/scenarios/imperial-rent-combined-seams-conformance.bscn` (world 12),
`content/scenarios/imperial_rent_combined_seams_conformance.py`,
`rust/crates/babylon-tick/tests/imperial_rent_seams_conformance.rs`; extend
`tests/imperial_rent_conformance.rs`, `tests/tick_goldens.rs` and the primary mirror.

- [ ] **Step 1: Failing tests** — `r06_pays_productivity_plus_the_super_wage_bonus` (employer
      decremented, worker `wealth` incremented, both bit-exact); `r06_caps_the_payment_at_the_employer_wealth`
      (a companion seed where `production-value + bonus > employer wealth`);
      `r06_writes_the_five_ppp_and_accounting_fields` (`effective-wealth`, `unearned-increment`,
      `ppp-multiplier`, `wages-paid`, `value-produced` — each bit-exact, and `ppp-multiplier ==
      1 + 0.8·1.0·0.5` derived from the defconsts, not hard-coded);
      `r06_writes_the_wages_value_flow` (the cross-pack seam of B4);
      `r05_emits_the_crisis_when_the_pool_is_exhausted` (world 3);
      `r05_emits_even_when_the_employer_is_inactive` — **the emit-before-skip defect vector**, the
      single most important test in this task — paired with
      `r06_does_not_pay_when_the_employer_is_inactive` on the same world.
- [ ] **Step 2: Write world 3** — `imperial-rent-superwage-crisis-conformance.bscn`: `rent-pool`
      seeded at or below `negligible-rent`, the employer seeded `active 0`, the worker active. This
      world exists to make **one** frozen ordering decision provable and its header must say so.
- [ ] **Step 3: Write `r05-wages-crisis` and `r06-wages-pay`** per §8. `r05` carries **no** `active`
      gate — transcribe the frozen ordering verbatim, and put the `economic.py:447-448` comment in
      the rule's `:material-basis`. Transcribe the §1.6-b dead conjunct and record in the commit body
      that no mutation can kill it (that IS the evidence). **`r05` also stamps the B8 latch** — two
      `(update-node …)` effects on `institution/superwage-crisis-known`/`-tick`, using
      `decomposition.bsl`'s **constant-score** carrier expression verbatim, behind a
      `(= crisis-known 0)` conjunct in the same guard. The `:material-basis` must cite D168's
      prescription and explain the deliberate constant-score exception (§3.1). **`r06` binds
      `production-value` `:optional :default 0`**, the faithful transcription of `:453`'s `.get`.
- [ ] **Step 4: Extend the primary mirror** to world 3 per §9's preamble (world 3's `economy` seed is
      the exhausted pool); re-paste stdout verbatim + dated. Measure fuel.
- [ ] **Step 5 — NEW: land §2.1's latch seam (D194).** Write world 12
      (`imperial-rent-combined-seams-conformance.bscn`) and its mirror
      (`imperial_rent_combined_seams_conformance.py`, chaining the frozen `ImperialRentSystem` →
      `DecompositionSystem` in @9.0 → @11 order, the `carceral_arc_conformance.py:173-174`
      precedent), plus `tests/imperial_rent_seams_conformance.rs`. The world is seeded so
      **`decomposition/p02`'s guard is CLOSED and `r05`'s is OPEN** (LA wealth comfortably above
      `subsistence + 2·consumption`; `rent-pool ≤ negligible-rent`). Rows:
      `r05_latches_the_superwage_crisis_when_p02_cannot`;
      `p03_trigger_fires_the_decomposition_delay_from_r05s_latch`;
      `the_latch_records_the_first_crisis_tick_not_the_last` (the converse world where **both** guards
      open — asserts `-tick` holds `p02`'s value, pinning `min` against `last`);
      `both_emitters_produce_distinguishable_payloads` (key-set assertion on the two
      `EventType/SUPERWAGE_CRISIS` payloads; `payer`'s presence is the discriminator).
- [ ] **Step 6 — NEW: land §2.2's producer seam (D195)** on the same world 12, as a **two-tick**
      `TickSession` (the §7.2 inversion makes every one of these one tick late): all seven
      conformance rows from §2.2's table. `r06_writes_the_exact_bytes_fundamental_theorem_binds` runs
      on world 1 (no combined pack needed — it asserts the bytes, not a consumer).
- [ ] **Step 7: Mutation — six vectors** — move `r05`'s guard behind an `active = 1` check:
      `r05_emits_even_when_the_employer_is_inactive` flips (the defect's killer). Swap `r06`'s cap
      `if` comparison: `r06_caps_the_payment…` flips. Drop the `(ppp-multiplier − 1)` factor:
      `r06_writes_the_five_ppp…` flips. **Drop `r05`'s latch effect entirely:**
      `r05_latches_the_superwage_crisis_when_p02_cannot` and
      `p03_trigger_fires_the_decomposition_delay_from_r05s_latch` both flip — **the vector that
      proves the C1 seam is live and not a comment.** **Drop the `(= crisis-known 0)` conjunct from
      `r05`'s latch guard:** `the_latch_records_the_first_crisis_tick_not_the_last` flips — the vector
      that pins `min` against `last`. **Drop `r06`'s `value-produced` write:**
      `r06_writes_the_exact_bytes_fundamental_theorem_binds` and
      `the_combined_world_positions_a_previously_unpositioned_class` both flip. Restore
      byte-identical each time.
- [ ] **Step 8: Add world 12's golden pin** (measured) and re-verify the 16 pre-existing pins.
- [ ] **Step 9: Six legs + commit** — two commits, not one, because the seams are a separable unit:
      `feat(tick): imperial-rent r05/r06 — the superwage crisis emit and the Amin/Wallerstein wage
      transfer`, then `feat(tick): imperial-rent — the decomposition latch seam and the
      value-produced/wage-flow producer seam`.

**Estimate:** ~5h.

### Task 6: `r07` + the measured pool-exhaustion divergence

**Files:** Modify `content/rules/imperial-rent.bsl`; create
`content/scenarios/imperial-rent-pool-exhaustion-conformance.bscn`; extend
`tests/imperial_rent_conformance.rs` and the primary mirror.

**This task lands BLOCKER-2. Its output is a measured, published divergence — not a hidden one.**

- [ ] **Step 1: Failing tests** — `r07_depletes_the_pool_by_the_actual_bonus_only`
      (`rent-pool` moves by `actual_bonus_paid`, **not** by `total_wages` — the productivity portion
      does not come from the pool); `r07_clamps_a_negative_bonus_to_zero`;
      `r07_does_not_deplete_when_the_pay_gate_fails`; and on world 8:
      `the_batched_pool_depletion_diverges_from_the_frozen_sequential_answer` — asserting the
      **ported** numbers, with the frozen sequential numbers in the assertion message and in a
      comment, plus `the_two_exploitation_edges_apply_the_rule_scoped_rent_twice` (D184's vector) and
      **`r05_r06_and_r07_agree_on_the_bonus`** (§8a's third copies-agree row) and
      **`the_two_exploitation_edges_can_drive_the_worker_wealth_negative`** — D196's reachability
      measurement, asserting the ported (possibly negative) number with the frozen non-negative
      number beside it, so the dropped `:295` clamp is **measured, not assumed dead**.
- [ ] **Step 2: Write world 8** — two WAGES edges into two workers off one employer, with `rent-pool`
      seeded so `Σ max_bonus > pool` (the pool binds); plus one worker with two EXPLOITATION edges
      **whose `eff · (1 − consciousness)` is seeded high enough that `wealth − 2·rent < 0`** — that is
      what makes D196's row a measurement rather than a tautology. The header states, in numbers,
      exactly what the frozen loop does and what the pack does, and why (§3.3's pre-state law), and
      names D183, D184 and D196.
- [ ] **Step 3: Write `r07-wages-pool`** per §8, gated identically to `r06`. **One effect** —
      `(sub abp)` from `rent-pool`; no `rent-wages-outflow` write (D199).
- [ ] **Step 4: Extend the mirror** to world 8 per §9's preamble and paste its stdout verbatim +
      dated. **The mirror's numbers will not match the Rust assertions on this world.** That is the
      point; the test file's header must say so in a dedicated paragraph naming D183, D184 and D196.
- [ ] **Step 5: A non-binding control** — assert on world 1 (where the pool does not bind and every
      worker has one EXPLOITATION edge) that the ported and frozen numbers ARE byte-identical.
      Without this control, the divergence looks like a general defect rather than a bounded one.
- [ ] **Step 6: Mutation — four vectors** — deplete by `total-wages` instead of `actual-bonus-paid`:
      `r07_depletes_the_pool_by_the_actual_bonus_only` flips. Drop `r07`'s zero clamp: a
      negative-bonus fixture flips. Perturb `r07`'s `bonus` transcription only:
      `r05_r06_and_r07_agree_on_the_bonus` flips while the single-rule rows stay green. Swap `r01`'s
      `(sub rent)` for `(set …)`: `the_two_exploitation_edges_apply_the_rule_scoped_rent_twice` flips —
      **the vector that pins §1.6-a's rejected candidate as rejected.** Restore byte-identical each
      time.
- [ ] **Step 7: Add world 8's golden pin** (measured) and re-verify the 16 pre-existing pins.
- [ ] **Step 8: Six legs + commit** `feat(tick): imperial-rent r07 — the batched pool depletion and
      its measured divergence from the frozen sequential loop`.
- [ ] **Step 9: Open PR B** `feature/imperial-circuit-port-bsl`, Tasks 5-6 (three commits). Review
      lens: (a) **the two cross-pack seams** — is the latch actually stamped, is the
      `(= crisis-known 0)` guard present, do the seven §2.2 rows assert consumer-visible bytes rather
      than "the write happened"; (b) BLOCKER-2's divergence — measured, published, bounded by the
      non-binding control; (c) §1.6-a's resolved AST and its two vectors. Harvest Copilot; merge via
      `mise run pr:merge -- N`.

**Estimate:** ~3.5h + review.

### Task 7: `r08` + `r09` + the four decision worlds (PR C)

Branch off **merged dev** as `feature/imperial-decision-port-bsl`. Never stack on PR B (#193).

**Files:** Modify `content/rules/imperial-rent.bsl`; create
`content/scenarios/imperial-rent-decision-{crisis,bribery,iron-fist,austerity}-conformance.bscn` and
`content/scenarios/imperial_rent_decision_conformance.py`; create
`rust/crates/babylon-tick/tests/imperial_rent_decision_conformance.rs`.

- [ ] **Step 0 (PR-C opening step): re-measure every pin PR A and PR B landed.** `r08`/`r09` change
      the post-tick pool on every world that carries the carrier — which is all of them. Run
      `run_once` for each inherited pin, paste the new measured values, record the per-rule-id
      `fired` delta in the commit body. Declared motion, not a STOP. The 16 pre-existing pins are
      re-verified byte-identical in the same step.

- [ ] **Step 1: Failing tests**, one per matrix row of §1.5 plus the boundaries:
      `crisis_wins_over_every_other_branch` (a world with `pool_ratio < critical` **and**
      `pool_ratio >= high` impossible — instead: `pool_ratio < critical` **and** low tension, which
      would satisfy BRIBERY's tension clause were the priority reversed);
      `bribery_requires_both_a_high_pool_and_a_low_tension` (plus its converse: high pool + tension
      **at** `bribery-tension-threshold` → NO_CHANGE, the `<` boundary);
      `iron_fist_requires_a_low_pool_and_a_tension_strictly_above_the_threshold` (plus the exact-
      threshold witness → AUSTERITY, the `>` boundary); `austerity_is_the_low_pool_else_arm`;
      `no_change_is_the_mid_range` (world 1); `the_wage_rate_clamps_at_both_ends` (two fixtures);
      `the_repression_level_clamps_at_zero_and_one`;
      `economic_crisis_emits_only_on_the_crisis_branch` (payload key-by-key with the **numeric**
      decision code `4`, and no `narrative_hint` key); `r09_decays_the_pool_after_the_decision_reads_it`
      (the ordering vector: `r08`'s `pool-ratio` uses the pre-decay pool);
      `r09_clamps_the_pool_at_zero`.
- [ ] **Step 2: Write the four decision worlds** — each is world 1 with a different seeded
      `rent-pool` / `rent-aggregate-tension`, per §9's per-world seed matrix, and each header states
      **which single constant or comparison it makes mutation-provable**. Do not vary the defconsts
      to reach a branch; vary the seeds (§8's constraint). **World 6 (IRON_FIST) requires
      `rent-aggregate-tension = 0.6` and is UNREACHABLE at 0** — the branch needs `tension > 0.5`;
      world 7 (AUSTERITY) is the exact-threshold witness at `0.5`.
- [ ] **Step 3: Write `imperial_rent_decision_conformance.py`** — one mirror driving all four worlds
      (the `control_ratio_conformance.py` four-world precedent), printing the post-tick `economy`
      graph attr and the event history per world. **Per §9, each world seeds all three graph
      attributes**, and this mirror is the one that would silently print the wrong world without the
      `opposition_states` seed:

  | world | `.bscn` `institution/rent-pool` | mirror `economy` pool | `.bscn` `rent-aggregate-tension` | mirror `opposition_states` gap |
  |---|---|---|---|---|
  | 4 CRISIS | < 10.0 | same | 0.0 | `{"capital_labor": {"gap": 0.0}}` |
  | 5 BRIBERY | ≥ 70.0 | same | 0.0 | `{"capital_labor": {"gap": 0.0}}` |
  | 6 IRON_FIST | 10.0 ≤ p < 30.0 | same | **0.6** | `{"capital_labor": {"gap": 0.6}}` |
  | 7 AUSTERITY | 10.0 ≤ p < 30.0 | same | **0.5** | `{"capital_labor": {"gap": 0.5}}` |

      Verify each row against the mirror's printed `pool_ratio` and `aggregate_tension` before
      pinning; do not trust the pool figures above. Paste stdout verbatim + dated.
- [ ] **Step 4: Write `r08-decision` and `r09-pool-decay`** per §8. The matrix is a nested `if` chain
      in **exactly** the frozen priority order (`dynamic_balance.py:85, 93, 101-115, 118`) — a
      reordering that happens to agree on the fixtures is still wrong and the crisis-priority test
      exists to catch it. Use `economy/bribery-tension-threshold = 0.7` (§1.6-d) and say so in the
      `:material-basis`.
- [ ] **Step 5: Mutation — nine vectors; every clamp gets one** — reorder CRISIS below BRIBERY:
      `crisis_wins_over_every_other_branch` flips. Flip `>=` to `>` on the high-pool clause: a
      boundary fixture flips. Flip `>` to `>=` on the iron-fist tension clause: the exact-threshold
      witness flips. Swap two decision codes: the emit test flips. Move `r09` above `r08` in byte
      order: `r09_decays_the_pool_after…` flips. **Drop `r08`'s `min-wage-rate` clamp arm**, then
      **its `max-wage-rate` arm**: `the_wage_rate_clamps_at_both_ends` flips on each (two separate
      vectors — the earlier draft had the test and no mutation). **Drop `r08`'s repression lower
      clamp**, then **its upper clamp**: `the_repression_level_clamps_at_zero_and_one` flips on each.
      **Drop `r09`'s `max(0, ·)` arm**: `r09_clamps_the_pool_at_zero` flips (a fixture whose decayed
      pool is negative is required — add it here if `r07` has not already produced one).
      Restore byte-identical each time.
- [ ] **Step 6: Add the four decision worlds' golden pins** (measured) and re-verify the 16
      pre-existing pins.
- [ ] **Step 7: Six legs + commit** `feat(tick): imperial-rent r08/r09 — the bourgeoisie decision
      matrix, the rate clamps and the TRPF pool decay`.

**Estimate:** ~4.5h.

### Task 8: The arc, the byte-order constraint test, and all remaining goldens

**Files:** Create `content/scenarios/imperial-circuit-arc-conformance.bscn`,
`content/scenarios/imperial_circuit_arc_conformance.py`,
`content/scenarios/imperial-rent-production-order-conformance.bscn` (world 11),
`rust/crates/babylon-tick/tests/imperial_circuit_arc_conformance.rs`; extend
`tests/imperial_rent_seams_conformance.rs`; modify `tests/tick_goldens.rs`.

**Interfaces:** The train's acceptance test — the proof the four phases compose across ticks in
frozen order.

- [ ] **Step 1: Failing test** — `the_imperial_circuit_runs_in_frozen_phase_order`: a `TickSession`
      advanced far enough that the pool visibly rises (inflow), falls (wage bonus + decay), and
      crosses a decision threshold. Assert the per-tick carrier trajectory bit-exact against the
      mirror, and the event sequence with its ticks. **Derive every tick from the arithmetic and
      verify it against the mirror; do not trust this plan's numbers.** Plus
      `each_phase_fires_exactly_once_per_tick_per_edge`;
      **`the_tick_reset_zeroes_the_accumulator_but_never_the_pool`** — a **TWO-TICK** test (`r00`'s
      only possible killer, §3.1); and **§8b's two-tick accumulator assertions**:
      `the_tribute_inflow_does_not_compound_across_ticks` and
      `the_pool_trajectory_is_bit_exact_across_the_arc`.
- [ ] **Step 2: Write the arc scenario and its mirror** — the mirror runs the frozen
      `ImperialRentSystem().step(...)` over the same tick range with one persistent graph, per §9's
      preamble, printing the `economy` graph attr and every event per tick. This is what proves the
      ported multi-tick composition matches the frozen one despite §3.3's batching.
- [ ] **Step 3: §7's executable constraints** — world 11
      (`imperial-rent-production-order-conformance.bscn`): a combined `production.bsl` +
      `imperial-rent.bsl` world with `production-value` **unseeded**, run for **TWO** ticks. Tick 1
      asserts Phase 3 pays only the super-wage bonus (productivity 0 via `r06`'s
      `:optional :default 0`); tick 2 asserts it pays `production.bsl`'s tick-1 output. **One tick
      cannot distinguish the two orders.** Plus, on world 12, the §7.2 and §7.3 seam assertions Task 5
      landed re-run as a suite so all four inversions have executable form in one file.
- [ ] **Step 4: Add the remaining golden pins** — worlds 9 and 11 (2 new; worlds 1, 2, 10 landed at
      Task 4, world 12 at Task 5, world 8 at Task 6, worlds 3-7 at Tasks 5/7 — **12 total**). Every
      pin **measured**. The 16 pre-existing pins byte-identical; this train's own pins re-measured
      wherever a rule landed after they were set.
- [ ] **Step 5: Final fuel re-measurement** across the **whole twelve-world suite** (worst-case
      ceiling), for every rule. Record each `E-LOAD-040` bound and its `B+1` in the commit body.
      Worlds 8, 11 and 12 are the largest and will dominate the ceilings.
- [ ] **Step 6: Mutation — `r00`'s vector, finally killable** — delete `r00`'s effect: tick 2's
      `institution/rent-tribute-inflow` becomes exactly **twice** tick 1's, and
      `the_tick_reset_zeroes_the_accumulator_but_never_the_pool` plus
      `the_tribute_inflow_does_not_compound_across_ticks` both flip. This is the vector Task 2
      deferred; it is the `production.bsl` `p0` precedent (`bsl-language.rst:6693-6697`) applied
      verbatim. Restore byte-identical.
- [ ] **Step 7: Six legs + commit** `test(tick): the imperial circuit arc, the four byte-order
      constraint tests and the remaining golden pins`.

**Estimate:** ~5h.

### Task 9: Records, docs, gates, handoff

**Files:** Modify `docs/reference/bsl-language.rst`, `content/rules/imperial-rent.bsl` (finalize the
file-local `D-N` block); create `ai/decisions/ADR214_imperial_rent_port_handoff.yaml` +
`ai/decisions/index.yaml` row; modify `ai/state.yaml`.

- [ ] **Step 1: Register rows — twenty-one, D181-D201** (global numbers: **re-check the tail first**;
      it was D180 on 2026-08-18):
  1. **D181 — `GlobalEconomy` + `tick_context` → the INSTITUTION carrier.** The
     `imperial-rent-register` node; the `institution/rent-*` prefix convention and why (field-roster
     disjointness with the carceral carrier's 18 fields); `r00` as the ported form of the frozen
     per-tick `tick_context` re-creation, `rent-pool`'s exemption from it, and the fact that `r00` is
     provable only across two ticks. **Carrier *identity* is D198's row, not this one.**
  2. **D182 — the push form is forced.** No `source-of`/`target-of` (§3.8 item 8's open item,
     `edge_lane_e2e.rs:186-189`), so `(edges …)` pull-iteration cannot write endpoints; every edge
     phase is self-anchored `neighbors` + `edge-between`; the frozen global `query_edges` insertion
     order is replaced by subject-node × neighbour order; the per-phase anchoring choice
     (source-anchored for Phases 1-2, target-anchored for Phase 3) and why.
  3. **D183 — the sequential pool depletion does not port.** `for-each` reads PRE-state
     (`structural_verbs.rs:130,750`); tick-start snapshot + batched decrement; byte-identical
     whenever the pool does not bind; **the measured divergence numbers from world 8**, both sides
     printed; the non-binding control on world 1.
  4. **D184 — the per-edge sequential source re-read does not port, in BOTH phases.**
     (a) Phase 1: multi-EXPLOITATION-edge workers; the frozen loop re-reads
     `worker_attrs["wealth"]` per edge (`:283`), the ported `rent` is one rule-scoped binding applied
     once per edge; measured on world 8. (b) **Phase 2: multi-TRIBUTE-edge compradors** —
     `economic.py:375` re-reads `source_attrs["wealth"]` per edge, so a two-edge comprador takes a
     second cut off the already-overwritten balance (`800 → 720 → 648`) where the port writes `720`
     twice (`800 → 720`); measured on **world 10**. The earlier draft covered only (a).
  5. **D185 — the single-employer `select-max` assumption** (D45/D145 class, `production.bsl`
     precedent). A world with two WAGES edges into one worker is **unported behavior, not equivalent
     behavior**.
  6. **D186 — Phase 4 STOP.** The full §6 packet: unported; M1/M2/M3 quoted with ADR202 R3 + ADR173
     citations; **R1 and R2 stated verbatim as OPEN and un-ruled**; the **ADR202 R3 client-state
     seeding D-record RECORDED — and the landing gate M2 names explicitly NOT discharged** (the verb
     matters; §6 item 2's blockquote is the wording); R3 (`#510` expiry) named as binding it; M3
     (`steepness_k` removal) explicitly **not** discharged and explicitly **not to be done early**;
     **R4 (ADR171's national-question axis) recorded as CONSIDERED AND DECLINED**, with the reason
     (coupling the bribe:deprivation ratio into the Phase-3 super-wage is NEW reserved scope,
     Constitution I.1) and the note that §2.2's chauvinism consequence rides the already-landed
     `wage-balance` path and mints no coupling; a world with CLIENT_STATE edges is unported behavior.
  7. **D187 — `opposition_states` has no producer.** The one-tick-stale frozen read; the seeded
     `institution/rent-aggregate-tension`; the explicit re-open trigger when ContradictionSystem
     ports.
  8. **D188 — payload divergences, FIVE not four.** `mechanism` and every `narrative_hint` dropped;
     `decision` encoded `0..4` in `BourgeoisieDecision` declaration order; `bourgeoisie_active` as
     `0/1`; **the `source_id`/`target_id`/`payer_id`/`receiver_id` → `source`/`target`/`payer`/
     `receiver` id-string→NodeRef key rename** (BLOCKER-5b — previously undeclared, and every
     conformance row asserting `source`/`payer` depends on it; `decomposition.bsl`'s D171 item 1 is
     the precedent that records the same class); the rejected `(defenum BourgeoisieDecision …)`
     alternative recorded as the shape to adopt if a reviewer prefers it.
  9. **D189 — the transcribed frozen defects**, each with its line and its (non-)mutation evidence:
     the dead second conjunct in the SUPERWAGE_CRISIS condition (no mutation can kill it — that IS
     the evidence); the Phase-2 wealth OVERWRITE; the `bribery_tension_threshold` 0.3-vs-0.7
     signature/docstring drift with the shipped value governing. **The fourth defect — the `:295`
     dead clamp — moved to its own row, D196**, because it resolved into a *decision* about an AST
     rather than a transcription.
  10. **D190 — the inter-pack byte-order inversions — FOUR, each with its executable constraint.**
      D100's class; the corrected namespace list (`economics`, not `fundamental-theorem/`; no
      `worldview/`); `imperial-rent/*` vs `production/*` (§7.1), vs `consciousness/*` (§7.2), vs
      `decomposition/*` (§7.3) and vs `economics/*` (§7.4); the `production-value` seeding
      constraint; the documentary `(anchor :after production)`; and the **test** that closes each
      one — including the statement that the §7.4 world is deliberately NOT built and why.
  11. **D191 — the duplication boundary with `economics/fundamental-theorem`.** B1-B3 as a record:
      the pack writes neither `social-class/imperial-rent` nor `social-class/wages`;
      `social-class/production-value` is READ from `production.bsl`'s publication, never recomputed;
      the `wages-paid`/`value-produced` cross-scenario `int`-vs-`real` deffield type divergence; and
      the §0 three-way name collision, named so the next reader is not trapped by it. **The
      produce-side seams are D194 and D195, not this row.**
  12. **D192 — the provably-absent wiring.** The two spec-063 sub-stages and the L-RECEIPTS boundary
      register: no BSL lane, silent no-ops in every unit test and the qa-six (`ServiceContainer.create()`
      never binds `boundary_register`; `TickContext.persistent_data` defaults to `{}`), omitted with
      named re-open triggers.
  13. **D193 — the dead subsistence phase.** `_process_subsistence_phase` (`:201-237`) is
      `.. deprecated:: ADR032` and never called by `step()`; **not ported**; its 11-test unit file
      is not a conformance candidate.
  14. **D194 — the SUPERWAGE_CRISIS producer seam and the latch (§2.1, CRITICAL).** The frozen
      `decomposition.py:161-175` `min(e.tick)` history scan and its dual role (feeds the delay clock
      AND suppresses `p02`'s own emit); D168's verbatim prescription; `r05` stamps
      `institution/superwage-crisis-known`/`-tick` under `decomposition.bsl`'s own qnames and
      constant-score carrier expression, behind a `(= crisis-known 0)` guard that reproduces `min`
      rather than `last`; **the two-emitter payload divergence** (`p02`'s three keys vs `r05`'s
      seven, `payer`'s presence as the discriminator); **the reconciliation with D171 item 1** —
      `payer_id` is a constant there and edge-derived here, so the same rule yields opposite
      dispositions on different facts; the §7.3 inversion and the tests that close it.
  15. **D195 — first-writer status over three quantities (§2.2, CRITICAL).** `r06` becomes the first
      writer of `social-class/wages-paid`, `social-class/value-produced` and `wages/value-flow`;
      the full reader census (`economics/fundamental-theorem:9`; `consciousness/p0-position`,
      `p4-wage-balance`, `p5-agitation`, `p7-persist-baselines`; `consciousness/p2-wages-push`);
      what the writes turn on (UNPOSITIONED → positioned; a non-zero `wage-balance`, whose positive
      half routes agitation fascist-ward through `p6-route`'s `chauvinist`; a live wage FLOW, which
      turns on `p5-agitation`'s dead exploitation term); the sign analysis
      (`wages − value = min(bonus, employer-wealth − production-value)`, positive with a super-wage,
      negative only when the employer's wealth binds); the **half-anchored** `fundamental-theorem`
      guard, recorded not repaired, with the ContradictionSystem-@18 re-open trigger; and the
      statement that all of this is DERIVED from two port-as-is transcriptions and mints no coupling.
  16. **D196 — the `economic.py:295` clamp is not transcribed, and why (§1.6-a).** The rejected
      clamp-preserving AST and the reason (a `set` inside a `for-each` silently repairs the frozen
      per-edge repetition); the adopted `(sub rent)`; the **reachability proof** that the clamp is
      dead in every frozen-reachable world (`rent ≤ worker_wealth` by `:292`, re-established each
      iteration by `:283`'s re-read); and the **measured** world-8 case where the ported batched form
      breaks that invariant — D184's second face — with both numbers published.
  17. **D197 — the D116 cross-rule apply-in-place reliance (§8b).** D116 quoted as the RECORDED GAP
      it is; **all six same-tick cross-rule reads enumerated individually** with what each does when
      the repair lands; the observation that no transfer test would flip (every input is
      tick-invariant), so the two-tick accumulator assertions are the only guards; the **re-open
      trigger** — the Q14 collect-across-rules-then-apply train — and the statement that **this row
      is that train's acceptance-criterion input for imperial-rent specifically**, so a stale row
      feeds a wrong criterion into a future train.
  18. **D198 — carrier identity is field-guarded, not id-tiebroken.** Why a constant-score
      `select-max` over `(nodes NodeType/INSTITUTION)` is exactly "the lowest-id INSTITUTION node"
      (D45's tiebreak, `evaluator.rs:990-1052`); the `institution/rent-carrier` discriminator and its
      landed-score precedents; the **deliberate exception** for B8's latch writes and why; the
      both-packs-world shape (one node, two disjoint rosters); the **latent hazard recorded and not
      repaired** in `decomposition.bsl`'s 14 constant-score reads should a world ever mint two
      INSTITUTION nodes; and the re-open trigger — the unserved `the` query head
      (`evaluator.rs:545`), slice 2, which retires the discriminator when it lands.
  19. **D199 — `rent-wages-outflow` is dropped.** `tick_context["wages_outflow"]` is a per-tick local
      that `_save_economy` (`:827-836`) never persists and nothing reads; declaring it would fabricate
      persistence and enter the tick hash write-only, against the pack's own
      declare-what-you-read discipline. The equivalent observable (`Δ rent-pool == −Σ actual_bonus_paid`)
      and the re-open trigger (declare the field **and** its reader in the same landing).
  20. **D200 — the repeated `set` of one field within one tick.** The unprecedented shape `r03`
      needs; the three questions Task 0 Step 4(e) settled and Task 1 Step 5(g) proved; the answer as
      measured; and its interaction with D184(b)'s Phase-2 divergence.
  21. **D201 — the duplication ledger (§8a).** The four expressions transcribed more than once; why
      single-sourcing is **not available** (the closed `.bsl`/`.bscn` top-form sets carry no
      `defexpr`, macro or cross-rule binding, so "single-source" means "merge the rules", which
      costs the independent mutation-killability the split bought); the copies-agree rows; and the
      perturb-one-copy mutation vector that proves each row catches drift.
- [ ] **Step 2: ADR214** (`ADR214_imperial_rent_port_handoff.yaml` + the `index.yaml` row) — records:
      the one pack and ten rules; the position correction (@9.0, not @10); **the boundary verdict of
      §2.3** (`economics/fundamental-theorem` carries zero of this system's surface; ADR210 D6-A is
      unlanded and belongs to `#563`; `phi_cap` stays a DEFCONST **on that train**); **the two
      produce-side seams of §2.1 and §2.2**, and specifically the two **theory-bearing consequences**
      stated for the Director's information — that this pack becomes the material producer of the
      `wage-balance` bribe signal `consciousness/p6-route` routes fascist-ward on, and of the live
      wage flow that turns on `p5-agitation`'s exploitation term — both **derived from two port-as-is
      transcriptions, minting no coefficient, no functional form and no new axis** (with R4 recorded
      as considered and declined); **the transcendentals verdict of §5** (PURE-ARITHMETIC — no
      intrinsics declared; the closure of ADR210's open question and the confirmation of ADR213's
      errata against the port-estate survey rows 83/197; `#576` is not a dependency); the carrier
      reformulation and its identity discriminator; BLOCKER-2's measured divergence; the **Phase-4
      STOP** with R1/R2 escalated and M2 **recorded, not discharged**; the four byte-order
      disclosures; and the gate evidence.
- [ ] **Step 3: Issue hygiene** — file the implementation issue under the Program 29 / Checkpoint A
      umbrella, linking all **three** PRs and ADR214; close it with evidence. File the **blocked**
      Phase-4 follow-on issue per §6 item 3, with R1/R2/R3 as its body and an explicit "cannot open
      until ruled" gate. Update the Checkpoint-A tally: with ImperialRent ported (four of five
      phases), the Material Base remainder is Substrate @2.5 (a Director scope question),
      TickDynamics @4.0 (`#563`), ReserveArmy, Community, **and this system's own Phase 4** —
      Checkpoint A (all 13 ported) is **not** reached by this train, and WS3 stays HELD. Say so
      explicitly rather than implying completion.
- [ ] **Step 4: Full gates, once** — `mise run rust:check`; `mise run check`; `mise run qa:regression`;
      `mise run qa:vault-regression-ci`; `PYTHONPATH="$PWD/src" UV_FROZEN=1 uv run pytest
      tests/unit/reference/test_bsl_grammar_sync.py -q`; `vale` over every touched Markdown/RST.
      **Nothing under `tests/baselines/**` may move.**
- [ ] **Step 5: `ai/state.yaml` closing entry** + commit `docs(p27): imperial-rent port handoff —
      register rows D181-D201, ADR214, the Phase-4 STOP`.
- [ ] **Step 6: Open PR C** `feature/imperial-decision-port-bsl`, Tasks 7-9 (four commits). Review
      lens: (a) **the Phase-4 STOP row read as a Director-facing artifact** — R1 and R2 stated, not
      resolved; M2 recorded, not discharged; R4 declined on the record; (b) the decision matrix's
      priority order and its nine mutation vectors; (c) the **twenty-one** register rows read as a
      set — every divergence in this plan has exactly one row, and every row names its re-open
      trigger. Harvest Copilot; merge via `mise run pr:merge -- N`.

**Estimate:** ~4h.

---

## PR structure — THREE PRs, and why the earlier two-PR split does not hold

| | branch | tasks | commits | contents |
|---|---|---|---|---|
| **PR A** | `feature/imperialrent-port-bsl` (worktree exists) | 0-4 | 5 | dossier; registration; carrier + discriminator + primary/TRPF/multi-tribute scenarios; the seven-shape spike; `r00`-`r04` (the pool **inflow** half); 3 golden pins |
| **PR B** | `feature/imperial-circuit-port-bsl`, off **merged dev** | 5-6 | 3 | `r05`-`r07` (wages, the latch, the pool depletion); worlds 3, 8, 12; **both Criticals** — the D194 latch seam and the D195 producer seam; the BLOCKER-2 divergence; 3 golden pins; PR A's pins re-measured |
| **PR C** | `feature/imperial-decision-port-bsl`, off **merged dev** | 7-9 | 4 | `r08`/`r09`; worlds 4-7, 9, 11; the arc; the four byte-order constraint tests; the remaining pins; **D181-D201**; ADR214; the Phase-4 STOP; PR A+B's pins re-measured |

**Why three, not two.** The earlier draft's own rationale — "ten rules, nine scenarios, three mirrors
and thirteen D-rows is far past a reviewable diff" — argued for a third PR and then rejected it on
*cycle cost* rather than *diff size*. The revision makes the case decisive: PR B under the old split
would have carried five rules, six scenarios, two mirrors, the arc, a combined world, seven pins,
thirteen D-rows, ADR214 and `state.yaml` — roughly **double** PR A — and the revision adds two more
worlds, a fourth mirror, eight more register rows and both Critical seams on top of that. The three
halves also want genuinely different review lenses:

- **PR A — transcription fidelity** plus the never-before-used edge write lane and the D198 carrier
  discriminator (the one novel shape).
- **PR B — cross-pack boundary.** Is the latch stamped, is its guard right, do the producer rows
  assert consumer-visible bytes? This is the lens the first draft did not have at all, and it wants a
  reviewer who is reading `decomposition.bsl` and `consciousness.bsl` side by side.
- **PR C — governance and closure.** The STOP row as a Director-facing artifact, the decision
  matrix's priority order, and the twenty-one register rows read as a set.

**The dependency direction is still clean.** PR A lands `institution/rent-pool` and its only writers;
PR B adds readers of PR A's state plus the two seams; PR C adds readers of both. **No earlier PR's
rule reads anything a later PR writes.** What is NOT true — and the earlier draft implied it — is
that "PR A's evidence survives PR B": PR A's golden pins **will** move at PR B and again at PR C, by
construction, and each later PR's Step 0 re-measures them. That is scheduled work, not a regression;
see the Global Constraints' two-obligation split.

**Never stacked** (#193: `--delete-branch` on a stacked PR closes rather than merges the child). Each
PR branches off **merged dev**.

---

## Estimate

**10 tasks · 12 commits · 3 PRs · ~34-42 agent-hours** (Task 0 ~2h, Task 1 ~4h, Task 2 ~4h,
Task 3 ~3h, Task 4 ~2.5h, Task 5 ~5h, Task 6 ~3.5h, Task 7 ~4.5h, Task 8 ~5h, Task 9 ~4h ⇒ ~37.5h
midpoint), plus **three** Copilot-harvest review cycles.

**Why the earlier 18-22h band was ~2× light, and what the revision added on top.** The critique's
audit of Task 2 alone is the illustration: the full pack header (a name-collision paragraph, the
reserved D-rows, a byte-order map naming each dependency, the §7 disclosures, and a 23-row cited
`defconst` table) plus three rules plus seven named tests plus three `E-LOAD-040` fuel readback
cycles plus four mutation vectors plus six cargo legs — of which `cargo test --workspace` and
`cargo doc --workspace` are minutes each, per task — is not two hours. ADR212's comparable train
produced a 2150-line conformance file for the same rule count. On top of that honest 30-40h baseline
this revision adds: three new worlds (10, 11, 12), a fourth mirror, eight more register rows, the
two Critical seams with eleven new conformance rows and three new mutation vectors, the copies-agree
ledger, the §9 mirror-recipe verification, and a third review cycle.

**Highest-variance step: Task 1 Step 5's spike — now SEVEN shapes**, individually served but jointly
content-unprecedented, and two of them (the discriminator score, the repeated `set`) have no landed
content precedent at all. If any fails, §8's rule split or §3.1's carrier idiom is re-planned before
Task 2, not worked around inside it.

**Second-highest: Task 5.** It lands both Criticals, a new world, a new mirror and a new test file,
and it is the one task whose failure mode is silent — a latch that is written but to the wrong node,
or a producer row that asserts "the write happened" instead of "the consumer sees it", both pass CI.

---

## Self-review notes (plan author)

- **What the first draft got wrong, named plainly, because the failure class matters more than the
  fixes.** It audited its boundary in one direction only — "does another pack already compute this?"
  — and answered correctly. It never asked the other direction: **"does another pack already WAIT on
  this?"** Two did. `decomposition.bsl`'s own register row named this port as its missing producer,
  in prose, in the landed tree, and the draft cited that file five times without reading D168. The
  lesson is procedural, not local: **a port's boundary audit is incomplete until every qname the pack
  WRITES has been grepped across the whole content estate for readers**, and every EventType it emits
  has been grepped for other emitters. §2's structure now enforces that order — produce-side first,
  duplication second. Task 0 Step 7 makes it a mechanical step rather than an act of insight.
- **Every construct is landed and cited:** `(nodes …)` and `(neighbors …)` as served query heads
  (`evaluator.rs:567`, `SERVED_QUERY_HEADS = ["nodes","neighbors","edges"]`); the carrier read/write
  idiom (`decomposition.bsl:254,266,269,323`); a **numeric `(field-of it …)` as a `select-max`
  score** (`query_lane_e2e.rs:252-253`, `r9_chapters.rs:1129-1130,1179-1180`) and the `if`-chain
  score fallback (`territory.bsl:133-136`); `(edge-between EdgeType/X self it)` inside a
  self-anchored traversal (`edge_lane_e2e.rs:196-206`, `consciousness.bsl:232-234`); `for-each` with
  multiple effect items and `update-edge it` (`edge_write_lane_e2e.rs:61-71`); the pre-state law
  (`structural_verbs.rs:130,750`); the closed update-op set `add|sub|set|scale` (`grammar.rs:718`);
  the closed arith set `+ − × ÷` (`grammar.rs:723`) and the five-member `FoldOp` set that makes
  `min`/`max` fold-ops, not binary heads (`grammar.rs:661-716`); `if`'s mandatory three operands
  (`grammar.rs:650`, `E-PARSE-042`) and the `(- 0 0c)` promotion idiom (`consciousness.bsl:255,316`);
  `:optional :default <literal>` on bindings (§2.5, `bsl-language.rst:845-846`, and
  `consciousness.bsl` throughout); `(select-max (neighbors self EdgeType/WAGES :in …) 1)` as an
  effect target (`production.bsl:216`); `(edge-attr …)` seeding (D156,
  `consciousness-ternary-conformance.bscn:309-313`); `emit` with NodeRef payload values; `:tick` as a
  servable bind source; the two latch fields (`decomposition-conformance.bscn:152-153`).
- **Three genuine capability risks, not one.** (a) The edge WRITE lane from a content pack:
  `update-edge` is tested in `edge_write_lane_e2e.rs` and `r9_chapters.rs` but **no landed rule pack
  uses it**, and two of this pack's three edge namespaces are net-new. (b) **A `(field-of it …)`
  carrier discriminator score** — landed in test fixtures, never in a content pack, and D198's whole
  disposition rests on it. (c) **A repeated `(set …)` of one field within one tick** — no landed pack
  does it, and `r03`'s shape depends on the answer. Task 1 Step 5 converts all three from assumptions
  into evidence before eight rules depend on them. A reviewer should check the spike landed as a real
  spike and not as a comment.
- **What is still a Task-0 obligation:** whether negative literals lex in **`defconst` value
  position** specifically (they demonstrably do in binding-default position —
  `consciousness.bsl:184`); whether a `for-each` body admits a nested **three**-effect `guard`
  (`r05` needs three); the `:optional :default` literal form against a `real extensive` field;
  whether a landed scalar class-consciousness qname already exists (B7); the repeated-`set`
  semantics; the discriminator-score form; and every `:fuel` figure, which is measured, never
  guessed. **The earlier draft's "no landed content uses a negative literal / the `(- 0 x)` idiom is
  universal" was simply false** and is struck.
- **Numbers this plan asserts that the implementer must re-derive, not trust:** the TRPF clamp tick;
  the arc's tick schedule; every `report.fired` count; §9's per-world pool figures; and every
  arithmetic result in §1 — all come from the mirrors, and the mirrors are the contract. **And the
  mirrors are only a contract if their seeds are right** — §9 exists because a verbatim paste of the
  wrong world's stdout is indistinguishable from a verbatim paste of the right one.
- **Fixture design intent:** the comprador is seeded with non-zero wealth specifically so the §1.6-c
  `set`-vs-`sub` asymmetry is mutation-provable (with a zero seed the two are indistinguishable and
  the defect would port silently); the worker carries non-zero `class-consciousness` so the
  `(1 − c)` factor is provable; world 3's employer is inactive so the emit-before-skip ordering is
  provable; world 6 carries tension `0.6` because IRON_FIST is **unreachable** below `0.5`; world 8's
  pool is seeded to **bind** so BLOCKER-2's divergence is measured rather than asserted, and its
  two-EXPLOITATION-edge worker is seeded so `wealth − 2·rent < 0` so D196's dropped clamp is measured
  rather than assumed; world 10's comprador has two TRIBUTE edges so D184(b) is measured; world 12 is
  seeded so `decomposition/p02`'s guard is **closed** while `r05`'s is **open**, which is the only
  configuration that proves the latch seam is live; and world 1 doubles as the non-binding control
  that bounds all of it.
- **The structural choice, now decided rather than left open.** `r02`/`r04` split from `r01`/`r03`
  (four rules for two phases) versus folding each credit into its phase's `for-each` as a third
  guard (two rules). **Keep the split**, because merging is the only form "single-sourcing" can take
  in this language (§8a) and it costs the independent mutation-killability of the role gate and the
  overwrite asymmetry. The duplication hazard the split creates is closed by §8a's copies-agree rows
  and their perturb-one-copy vectors — which the earlier draft did not have, and without which the
  split really was the wrong call.
- **What this plan deliberately does NOT do:** rename any of the three "Imperial Rent"s (§0);
  implement, sketch, or parameterize Phase 4 (§6); couple ADR171's national-question axis into the
  super-wage (§6 item 4 — considered and declined); become `social-class/wages`'s producer or build a
  combined `economics` + `imperial-rent` world (§7.4); repair `decomposition.bsl`'s latent
  constant-score carrier hazard (D198 — recorded, because repairing it moves landed goldens); declare
  `institution/rent-wages-outflow` (D199); import any of the Leontief estate's hazards (§5 strand 1);
  or declare a single intrinsic.
