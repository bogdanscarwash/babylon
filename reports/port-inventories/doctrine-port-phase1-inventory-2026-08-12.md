# DoctrineSystem Port — Phase-1 Inventory (2026-08-12)

**Executive summary.** DoctrineSystem (700 lines, `src/babylon/engine/systems/doctrine.py`, tick
position 14.7) plus its ~860 lines of `domain.doctrine.*` pure mechanics is the largest and most
structurally hazardous system inventoried in this wave: it is the **only** system among the ones
sampled so far that consumes the tick RNG for a real gameplay branch (the Party Congress purge),
and it is the first to depend on **graph-level metadata registers** (`policy_delivery`,
`electoral_governments`, `political_form_org_positions`) and **untyped, both-direction, all-edge-type
incidence scans** — neither has any BSL substrate representation today. Roughly half of the per-tick
computation (decay, TL accrual, root bootstrap, greedy acquisition, the two pure-tag/simple-practice
traps) is a clean structural match for the landed Slice-1 query lane; the other half (SOLIDARITY-edge
sums, CO_OPTIVE_SHARE incidence, the three graph-registers, the RNG-gated congress) is hard-blocked on
work not yet built, some of it not even fully speced. A live, verified, and previously-unrecorded
finding: the frozen system's own module docstring and its property-law test both claim byte-safety
because "all SIX qa:regression scenarios carry no organization nodes" — **this is now stale**: the
`org_probe` scenario (2 ORGANIZATION nodes) has been live in the canonical `SCENARIOS` registry with a
committed baseline since the Organization-foundation train, so DoctrineSystem is no longer a no-op on
the byte-identical gate.

**Verdict: BLOCKED (edge-attribute-read/write lane / Slice 2) — with a real, non-trivial PORTABLE core.**
The per-org decay/accrual/acquisition/trap loop over pure tags is portable now with D-records; the
practice-variable computation, the mass-work SOLIDARITY decay, the three graph-scope registers, and the
Party Congress RNG draw are each blocked on a different named, currently-unbuilt lane.

---

## 1. FILE MAP

| File | Lines | Role |
|---|---|---|
| `src/babylon/engine/systems/doctrine.py` | 700 | **The target.** `DoctrineSystem` (kernel `step()`, lines 615–700) + the free functions the class calls: `compute_doctrine` (498–612, the per-tick orchestrator), `step_organization` (379–462, the pure per-org tick), `_practice_env` (163–216), `_political_form_position` (219–253), `_officeholder_capture` (270–310), `_apply_practice_drift` (313–344), `_decay_mass_work_solidarity_edges` (116–139), `_delivery_gap` (142–160), `_resolve_line_struggle` (476–495), plus small pure helpers (`_apply_deltas` 94–101, `_cheapest_acquirable` 104–113, `_decouples_cadre_valve` 256–267, `_reachable_traps` 347–358, `_read_tags` 361–369). |
| `src/babylon/domain/doctrine/mechanics.py` | 320 | Pure Unit-3 mechanics: `evaluate_trap_condition` (the trap-condition DSL evaluator, a hand-rolled recursive-descent parser, 34–234), `can_acquire` (237–267), `acquire` (270–278), `accrue_theoretical_labor` (281–294), `decay_tags` (297–310). |
| `src/babylon/domain/doctrine/congress.py` | 142 | Unit-5 Party Congress: `held_sprung_traps` (58–63), `tag_delta_score` (66–71), `purge_probability` (74–82), `run_congress` (85–142). Pure — the RNG roll is injected by the caller. |
| `src/babylon/domain/doctrine/loader.py` | 55 | `load_doctrine_tree` — reads `data/game/doctrine_tree_mvp.json`, builds `DoctrineTree`/`DoctrineNode` Pydantic models, runs `validate_doctrine_tree`. Called once, lazily, by `DoctrineSystem.__init__`/`step` (`self._tree` cache, doctrine.py:631–633, 652–653). |
| `src/babylon/domain/doctrine/validation.py` | 238 | Structural DAG/root/tier/trap/goal validity checks. Load-time only — never touches per-tick state. |
| `src/babylon/domain/doctrine/tags.py` | 74 | `compute_tags`/`starting_tags` — **NOT called anywhere in the live tick path.** Grep-confirmed (`rg -n 'compute_tags\|starting_tags' src/ tests/`): only `doctrine/__init__.py`'s re-export, its own tests, and one comment reference in `tests/unit/engine/laws/test_law_doctrine_system.py:68` (documenting exactly this fact: the `[0,10]` clamp `compute_tags` implies is a *different function the tick loop never calls* — see §4 CAVEAT). Out of scope for the port's tick-path transcription, but its `[0,10]` docstring is the source of the misleading clamp claim on `DoctrineTag` (models/enums/doctrine.py:16–21). |
| `src/babylon/models/entities/doctrine.py` | 235 | `DoctrineNode`/`DoctrineTree`/`DoctrineCapability` Pydantic models — the tree's static content shape. |
| `src/babylon/models/enums/doctrine.py` | 95 | `DoctrineTag` (3 members), `DoctrineTrunk` (3 members), `PracticeVariable` (5 members). |
| `src/babylon/config/defines/doctrine.py` | 111 | `DoctrineDefines` — 11 coefficients (see §2(e)). |
| `src/babylon/config/defines/politics.py` | (relevant: 327–347, 445–524) | `PoliticsDefines` — the 8 fields DoctrineSystem reads via the `coeffs` dict (office_capture_rate, split_asset_retention, the 3 liquidation-absorbing-state thresholds, class_analysis_veto_decay, reformist_theory_decay, co_optive_dependence_drift). |
| `src/babylon/data/game/doctrine_tree_mvp.json` | 256 | The tree content itself: **14 nodes** (see §2 CONTENT note — several docstrings elsewhere still say "11"), 3 trunks, 2 traps (`liquidationism`, `adventurism`), 1 goal (`united_front`). |
| `src/babylon/models/entities/organization.py` | (relevant: 116–245) | `Organization` base Pydantic model — the 7 doctrine-state fields DoctrineSystem reads/writes (`acquired_doctrine_ids`, `theoretical_labor`, `doctrine_tags`, `congress_tag_snapshot`, `study_target_id`, `office_tenure`, `institutional_pull`) plus `cadre_level`/`cohesion` (read-only here). |
| `src/babylon/models/enums/topology.py` | (relevant: 63, 100, 130–155) | `NodeType.ORGANIZATION`, `EdgeType.SOLIDARITY`, `EdgeMode` (5 members, incl. `CO_OPTIVE`). |
| `src/babylon/kernel/system_base.py` | (relevant: 32, 35–55) | `resolve_rng` — the **only** RNG entry point DoctrineSystem uses (`_SYSTEM_RNG_SEED_SALT = 0xBA1AC1A`, `random.Random(salt + tick)` fallback; prefers `services.rng` when a harness injects one). |
| `src/babylon/kernel/graph_protocol.py` | (relevant: 88–98, 152–180, 258–298, 350–370) | `update_node`/`update_edge`/`query_nodes`/`query_edges`/`get_graph_attr`/`set_graph_attr` signatures. **No source/target filter on `query_edges`** (278–298) — `_practice_env`'s CO_OPTIVE_SHARE scans every edge in the graph and filters in Python. |
| `src/babylon/topology/graph.py` | (relevant: 660–702, 892–898) | Concrete `BabylonGraph`: `update_node`/`update_edge` are plain dict merges (no type coercion, same as every other system inventoried); `get_graph_attr`/`set_graph_attr` are a plain dict keyed by string — this **is** the entire "register" mechanism (no schema, no type). |
| `src/babylon/engine/systems/contradiction.py` | (relevant: 562–593) | Downstream reader of `political_form_org_positions` (§5). |
| `src/babylon/engine/systems/policy.py` | (relevant: 107, 473–493, 535, 691) | Writer of `policy_delivery` (read one-tick-stale by `_delivery_gap`); reader of `institutional_pull` (downstream, same tick) and `acquired_doctrine_ids` (entryism gate). |
| `src/babylon/engine/systems/electoral.py` | (relevant: 103, 356, 590–793, 1236) | Writer of `electoral_governments` (read one-tick-stale by `_officeholder_capture`); reader of `acquired_doctrine_ids` (entryism gates) and `solidarity_strength`. |
| `src/babylon/engine/actions/reproduce.py` | 100 | The **REPRODUCE verb resolver** (OODA-dispatched, same tick, position 14.0, prior to Doctrine's 14.7): writes `cadre_level`/`cohesion` — a genuine same-tick prior-write channel (§5). Not called by DoctrineSystem; paired by data dependency only. |
| `src/babylon/engine/actions/_mass_work.py` | 125 | The **symmetric write side** of `_decay_mass_work_solidarity_edges` (module docstring's own cross-reference, doctrine.py:30–34): creates/strengthens org→class SOLIDARITY edges from mass-work verbs, amplified by the org's *previous-tick* `doctrine_tags[MASS_LINK]`. Not called by DoctrineSystem — paired by shared attribute (`solidarity_strength`) and shared coefficients (`DoctrineDefines.mass_work_solidarity_gain`/`mass_link_weight`, which DoctrineSystem's own `step()` never reads — see §2(e)). |
| `src/babylon/engine/scenarios/org_probe.py` | 164 | **Load-bearing for §5's dormancy correction.** `create_org_probe_scenario()` seeds 2 real `NodeType.ORGANIZATION` nodes (`CivilSocietyOrg` cadre_level=0.1, `StateApparatus` cadre_level=0.6) into the canonical `SCENARIOS` registry (`tools/regression_scenarios.py:37–125`) with a committed baseline (`tests/baselines/org_probe.json`, `tests/baselines/dense/org_probe.csv`, `tests/baselines/vault/org_probe`). |

**Not exercised by DoctrineSystem at all:** no `src/babylon/formulas/*` module (grep-confirmed, `rg -n '^from babylon|^import babylon' src/babylon/engine/systems/doctrine.py` shows only `domain.doctrine.*` + `kernel.*` + `models.enums*` imports, doctrine.py:51–74).

**Reference BSL content read for format** (all fully read):
- `rust/crates/babylon-bsl/tests/conformance/doctrine_adventurism.bsl` (13 lines) — transcribes `MASS_LINK <= 0` (still current: adventurism's condition is unchanged by P25 U11).
- `rust/crates/babylon-bsl/tests/conformance/doctrine_liquidationism.bsl` (10 lines) — transcribes the **pre-U11** liquidationism condition (`CLASS_ANALYSIS <= 0 AND MILITANCY <= 0`) — this no longer matches the shipped tree (see §2 CONTENT note).
- `rust/crates/babylon-bsl/tests/conformance/doctrine_liquidation_absorbing.bsl` (18 lines) — **already transcribes the current U11 3-clause practice-gated condition**, proving the trap-condition DSL maps cleanly onto `:field`/`:const` bindings and `when (and ...)` with zero new grammar. Load-bearing caveat: it treats `solidarity-mass`/`co-optive-share`/`petty-bourgeois-drift` as **already-stored simple fields** — it does not attempt the actual computation (`_practice_env`'s edge sums/incidence ratios), which is the blocked half (§6).
- `rust/crates/babylon-tick/content/rules/organization.bsl` (31 lines) — the only landed ORGANIZATION-node rule; establishes `NodeType/ORGANIZATION`, `organization/kind` (enum), `organization/active` are real vocabulary, but exercises none of DoctrineSystem's actual state.
- `rust/crates/babylon-tick/content/scenarios/organization-foundation.bscn` (67 lines) — confirms SOLIDARITY edges (including org↔org) are seedable with a scalar strength value today, and that ORGANIZATION/SOLIDARITY are minted vocabulary (ADR195/ADR196).
- `docs/reference/bsl-language.rst` §2.4–2.6 (query grammar), §2.9–2.10 (edge/hyperedge deffield + field-of, drafted not landed), §3.6 (the graph-scope-state carrier-node-type ruling), §3.10 (intrinsic cap + the RNG carrier-key convention) — all read in full for this inventory.
- `docs/reference/determinism-contract.rst` §"Fuel Cost Model and RNG Seeding" (lines 1026–1095) — the **landed** kernel RNG (`ChaCha8Rng`, `rust/crates/babylon-kernel/src/rng.rs`, 201 lines, read in full).
- `reports/p27-porting-contract-table.md` row 18 (DoctrineSystem's own official Program-27 classification) and `reports/p27-tolerance-and-envelope-derivations.md` §4.1 (DoctrineSystem's stochastic-family conformance methodology, already ruled).

---

## 2. COMPUTATION CATALOG (execution order, `compute_doctrine`, doctrine.py:498–612)

**CONTENT NOTE (load-bearing for everything below).** The shipped `doctrine_tree_mvp.json` has
**14 nodes**, not 11: `class_consciousness` (root), `trade_unionism`, the 5-stance reformist fork
(`abstention_boycott`, `class_struggle_elections`, `entryism`, `independent_ballot_line`,
`governance_road`), `liquidationism` (trap), `democratic_centralism`, `mass_line`, `united_front`
(goal), `armed_vanguard`, `urban_guerrilla`, `adventurism` (trap). Four separate docstrings still say
"11 nodes"/"11-node MVP" (`domain/doctrine/__init__.py:4`, `models/enums/doctrine.py:4`,
`config/defines/doctrine.py:5`, `models/entities/doctrine.py:5`) — **stale, verbatim-transcribed as
found**, not corrected (port-as-is law). The P25 U11 reformist-fork expansion (5 stances replacing
what the 11-node MVP had) postdates these docstrings and they were never updated.

### Step 0 — Congress gate check (`compute_doctrine`, doctrine.py:522)
- **(a)** `is_congress = tick > 0 and rng is not None and tick % defines.congress_interval_ticks == 0`. Congress runs FIRST, before the ordinary per-org step, on qualifying ticks.
- **(b)** Plain modulo comparison, no continuous arithmetic.
- **(e) Defines:** `doctrine.congress_interval_ticks` (int, default 52, `>= 1`) — defines.yaml:976.

### Step 1 — Party Congress (per org, only when `is_congress`; doctrine.py:527–568, delegating to `congress.py:85–142`)
- **(a)** Once every `congress_interval_ticks`, before the ordinary tick: if the org holds a trap AND can afford `trap_escape_tl`, attempt exactly one purge (the first held trap by id) via a weighted seeded-RNG draw biased by tag-vector movement since the last congress; theoretical labor is spent on the attempt whether it succeeds or not. Then resolve any reformist line struggle (>1 stance held → consolidate to the newest, shedding the rest at a retention rate).
- **(b) Exact formulas:**
  - `needs_roll = bool(held_sprung_traps(tree, acquired0)) and tl0 >= float(defines.trap_escape_tl)` (doctrine.py:534–536) — the roll is drawn **only** when an attempt will actually consume it (org-less/trap-less graphs never touch the RNG stream).
  - `roll = rng.random() if needs_roll else 0.0` (doctrine.py:537) — **the single stochastic float in the whole system.**
  - `purge_probability`: `raw = 0.5 + defines.congress_delta_weight * delta_score; return min(max(raw, floor), 1.0 - floor)` (congress.py:81–82) — two-sided clamp idiom #1.
  - `tag_delta_score`: `sum(float(current.get(k,0.0)) - float(snapshot.get(k,0.0)) for k in (set(current)|set(snapshot)))` (congress.py:71).
  - `run_congress`: `tl_after = theoretical_labor - float(defines.trap_escape_tl)` (congress.py:118, spent unconditionally on attempt); on success (`roll < probability`), the trap is dropped from `acquired` and its `tag_deltas` are **subtracted back out** (congress.py:122–125): `new_tags[tag] = new_tags.get(tag,0.0) - float(delta)`.
  - Line-struggle resolution (doctrine.py:476–495): `keep = held[-1]` (id-insertion-order "newest" — **not** re-sorted, so acquisition order IS the tiebreak); `consolidated = tuple(a for a in acquired if a not in shed)`; `tl * retention` (a bare multiply, `retention` defaulting to `1.0` when `coeffs` lacks `split_asset_retention` — doctrine.py:557–561 — a genuine silent-degrade-to-identity fallback, not a loud failure, worth flagging: every real call site (`DoctrineSystem.step`) DOES pass `split_asset_retention`, so this only fires for a direct unit-test call, but it is a **soft default** inside a system whose module docstring elsewhere insists on loud failure for unknown `@coeff`s).
  - Congress ALWAYS re-baselines the tag snapshot even on a no-attempt tick (congress.py:104–114: `snapshot=dict(current_tags)`).
- **(c) Reads:** `acquired_doctrine_ids`, `theoretical_labor`, `doctrine_tags`, `congress_tag_snapshot` (all pre-congress state, doctrine.py:528–531).
- **(d) Writes:** the same four attrs, overwritten in the `attrs` dict (doctrine.py:547–550), later flushed to the graph by the shared `update_node` call at doctrine.py:595. Line split additionally overwrites `acquired_doctrine_ids`/`theoretical_labor` again (doctrine.py:566–567).
- **(e) Defines:** `doctrine.trap_escape_tl` (int, default 300, `>= 0`), `doctrine.congress_delta_weight` (float, default 0.15, `>= 0.0`), `doctrine.congress_contingency_floor` (float, default 0.10, `> 0.0, < 0.5`) — defines.yaml:977,979,980. Plus `politics.split_asset_retention` (float, default 0.4, `[0,1]`) — defines.yaml:1112 — read from the `coeffs` dict built in `DoctrineSystem.step` (doctrine.py:666).
- **(f) Events:** `"escaped"` → `EventType.DOCTRINE_TRAP_ESCAPED`; `"purge_failed"` → `EventType.DOCTRINE_PURGE_FAILED`; `"line_split"` → `EventType.LINE_STRUGGLE_SPLIT` (handled specially in `step()`, doctrine.py:677–693, with a richer payload reconstructed from `node_id.partition("|")`).

### Step 2 — Practice-variable measurement (`_practice_env`, doctrine.py:163–216; per org, every tick)
- **(a)** Five I-FRESH quantities read live from the org's current graph position — never accumulated state — feeding both the reformist fork's absorbing-state trap and the tag-drift step below.
- **(b) Exact formulas:**
  - `SOLIDARITY_MASS = Σ solidarity_strength over org's outgoing SOLIDARITY edges` (doctrine.py:187–189).
  - `CO_OPTIVE_SHARE = co_optive_count / incident_count if incident_count else 0.0` (doctrine.py:191–199) — a plain ratio over **every** edge touching the org, any type, any direction (`graph.query_edges()` with no filter, then Python-side `source_id/target_id` check — doctrine.py:193–196), counting `edge_mode == EdgeMode.CO_OPTIVE.value`.
  - `PETTY_BOURGEOIS_DRIFT = min(1.0, max(0.0, 1.0 - cadre_level))` (doctrine.py:202) — two-sided clamp idiom #2, bare `1.0` literal subtract.
  - `OFFICE_TENURE = tenure / (tenure + 1.0)` (doctrine.py:205) — saturating-division idiom #1.
  - `DELIVERY_DEPENDENCE = gap / (gap + 1.0)` (doctrine.py:208, `gap` from `_delivery_gap`) — saturating-division idiom #1, repeated.
- **(c) Reads:** `SOLIDARITY` out-edges' `solidarity_strength` (edge attr); ALL incident edges' `edge_mode` (edge attr); `attrs["cadre_level"]` (node attr, read-only here — written by `reproduce.py`, §5); the `policy_delivery` graph register (via `_delivery_gap`, §2 Step 6 below); `attrs["office_tenure"]` (node attr, DoctrineSystem's own prior-tick write).
- **(d) Writes:** none (pure measurement into a local dict).
- **(e) Defines:** none directly (all inputs are graph state).
- **(f) Events:** none.

### Step 3 — Officeholder capture (`_officeholder_capture`, doctrine.py:270–310; per org, every tick)
- **(a)** While an org is a seated governing party, its `office_tenure` accrues by 1 and its `institutional_pull` drifts toward 1 (Michels' iron law as a rate), resisted by `cadre_level × cohesion`. A stance with `cadre_valve_decouple=True` (Principled Abstention) still accrues tenure but takes zero pull.
- **(b) Exact formulas:**
  - `governs = any(gov.get("party_id") == org_id for gov in electoral_governments.values())` (doctrine.py:300).
  - Not governing → return `(tenure, pull)` unchanged (early return, doctrine.py:301–302).
  - `tenure += 1.0` (doctrine.py:303) — bare-literal increment.
  - Decoupled stance held → return `(tenure, pull)` (pull frozen, doctrine.py:304–305).
  - `resistance = 1.0 - min(1.0, cadre_level * cohesion)` (doctrine.py:306–308) — single-sided upper clamp idiom #1 (bare `1.0` literal, twice).
  - `pull = min(1.0, pull + capture_rate * resistance * (1.0 - pull))` (doctrine.py:309) — single-sided upper clamp idiom #1 again, a 3-factor multiply-then-add, bare `1.0` literal twice more.
- **(c) Reads:** `attrs["office_tenure"]`, `attrs["institutional_pull"]` (own prior-tick writes); the `electoral_governments` graph register (one-tick-stale, ElectoralSystem @17.45); `_decouples_cadre_valve` (doctrine.py:256–267, reads `attrs["acquired_doctrine_ids"]` against `tree.nodes[id].capabilities.cadre_valve_decouple`, a static content field); `attrs["cadre_level"]`, `attrs["cohesion"]` (read-only).
- **(d) Writes:** none directly — returns `(office_tenure, institutional_pull)`, flushed via the shared `update_node` call.
- **(e) Defines:** `politics.office_capture_rate` (float, default 0.02, `[0,1]`) — defines.yaml:1111, passed as `capture_rate` from the `coeffs` dict built in `DoctrineSystem.step` (doctrine.py:662, 571–575 — degrades to `0.0` if `coeffs` lacks the key, same soft-default pattern as Step 1).
- **(f) Events:** none.

### Step 4 — The ordinary per-org tick (`step_organization`, doctrine.py:379–462; pure, every tick, every org)
- **(a)** Decay all accumulated tag strength; accrue theoretical labour from cadre quality; bootstrap the free root once affordable; either save toward a player-directed study target or greedily auto-acquire the cheapest affordable node; then fire any reachable trap whose condition holds.
- **(b) Exact formulas, in order:**
  1. `tags = decay_tags(tags, defines.tag_decay_rate)` → `factor = 1.0 - decay_rate; {tag: value*factor for ...}` (mechanics.py:309–310) — bare `1.0` literal, per-tag multiply.
  2. `study_allocation = (defines.study_allocation_min + defines.study_allocation_max) / 2.0` (doctrine.py:414) — a **fixed midpoint**, bare `2.0` divisor. **Docstring discrepancy, verbatim-recorded:** `config/defines/doctrine.py:18–19`'s Ruling-4 comment says "the org's OODA picks within the band" — the code does no such thing; it is a hardcoded constant every org shares every tick. The doctrine.py module's OWN top docstring (line 12) correctly says "the midpoint of the ratified band," so only the `config/defines` comment is stale.
  3. `tl += accrue_theoretical_labor(cadre, study_allocation)` → `if surplus <= 0.0: return 0.0; else: surplus * min(1.0, max(0.0, study_allocation))` (mechanics.py:291–294) — two-sided clamp idiom #3 (same shape as Step 1's `purge_probability`), then a multiply. **Naming note, verbatim-recorded:** the parameter is named `surplus` (mechanics.py:281) but the call site passes `cadre_level` (doctrine.py:409, 415) — deliberate per the module docstring ("`cadre_level` is the MVP surplus proxy," doctrine.py:10–12), not a bug, but the function signature's own name is misleading read in isolation.
  4. Root bootstrap: `if root not in acquired and can_acquire(...): acquired = acquire(...); tags = _apply_deltas(tags, root.tag_deltas)` (doctrine.py:417–419) — `can_acquire` is a pure boolean gate (mechanics.py:237–267: not-held AND not-trap AND all-parents-held AND `cost_tl <= theoretical_labor`); `_apply_deltas` is `result[key] = result.get(key,0.0) + float(delta)` per tag (doctrine.py:94–101).
  5. Directed-study branch (target unlocked but possibly unaffordable): `if tl >= target.cost_tl: tl -= target.cost_tl; acquire; apply_deltas; clear target` — **else save, no greedy** (doctrine.py:428–434). Directed-study target still LOCKED: falls through to greedy (doctrine.py:435–443): `candidate = min(acquirable, key=lambda nid: (cost_tl, nid))` (mechanics.py:110–113, deterministic tie-break by node id) — `tl -= candidate.cost_tl`.
  6. Trap loop (doctrine.py:444–460): for every `_reachable_traps(tree, acquired)` (all parents held, id-sorted — mechanics-adjacent doctrine.py:347–358), evaluate `evaluate_trap_condition(trap.trap_condition, env, coeffs)` against the MERGED tag+practice environment (doctrine.py:445–455); on `True`, acquire the trap and apply its (possibly negative) `tag_deltas`.
- **(c) Reads:** `attrs["acquired_doctrine_ids"]`, `attrs["theoretical_labor"]`, `attrs["doctrine_tags"]`, `attrs["cadre_level"]`, `attrs["study_target_id"]`; the static `DoctrineTree` content (cost_tl, tag_deltas, parents, is_trap, trap_condition); the merged tag+practice env from Step 2; the `coeffs` dict (for `@coeff`-referencing trap conditions — currently only `liquidationism`'s).
- **(d) Writes:** none directly — returns `(acquired, tl, tags, sprung_ids, study_target)`, flushed via the shared `update_node`.
- **(e) Defines:** `doctrine.tag_decay_rate` (float, default 0.0055, `[0,1]`) — defines.yaml:973; `doctrine.study_allocation_min`/`study_allocation_max` (floats, default 0.15/0.25, `[0,1]` each) — defines.yaml:974–975. Plus (via `coeffs`, only when a trap's `@name` references them) `politics.solidarity_liquidation_floor` (0.05, `[0,1]`), `politics.co_optive_liquidation_threshold` (0.6, `[0,1]`), `politics.petty_bourgeois_liquidation_threshold` (0.6, `[0,1]`) — defines.yaml:1122–1124.
- **(f) Events:** trap acquisitions here are only detected/reported one level up in `compute_doctrine` (the `sprung` list, doctrine.py:604) → `EventType.DOCTRINE_TRAP_SPRUNG`.

### Step 5 — Practice-driven tag drift (`_apply_practice_drift`, doctrine.py:313–344; per org, every tick, AFTER Step 4)
- **(a)** The re-founded reformist fork's tag movement comes from measured practice, not acquisition deltas: CLASS_ANALYSIS erodes under delivery-gap veto pressure and officeholder capture; MASS_LINK erodes under CO_OPTIVE dependence. Each erosion only applies to a positive tag, floored at 0.
- **(b) Exact formulas:**
  - `ca_decay = coeffs.get("class_analysis_veto_decay",0.0)*delivery_gap + coeffs.get("reformist_theory_decay",0.0)*institutional_pull` (doctrine.py:333–336) — 2 multiplies + 1 add, both coefficients silently degrading to `0.0` if absent from `coeffs` (same soft-default pattern noted above; the real call site always supplies both).
  - `if ca_decay > 0.0 and tags[CLASS_ANALYSIS] > 0.0: tags[CLASS_ANALYSIS] = max(0.0, tags[CLASS_ANALYSIS] - ca_decay)` (doctrine.py:337–338) — single-sided floor clamp idiom #4.
  - `ml_decay = coeffs.get("co_optive_dependence_drift",0.0) * practice_env[CO_OPTIVE_SHARE]` (doctrine.py:339–341) — same floor-clamp pattern for MASS_LINK (doctrine.py:342–343).
- **(c) Reads:** `tags` (post-Step-4), `practice_env[CO_OPTIVE_SHARE]` (Step 2), `institutional_pull` (Step 3), `_delivery_gap(graph, org_id)` — **recomputed a second time here** (doctrine.py:583), not reusing the value already read inside `_practice_env`'s `DELIVERY_DEPENDENCE` computation — a genuine, harmless (both reads see the same pre-state) but real redundant-computation note.
- **(d) Writes:** none directly — returns the updated `tags` dict, folded into the `updates["doctrine_tags"]` written at doctrine.py:588.
- **(e) Defines:** `politics.class_analysis_veto_decay` (0.03, `[0,1]`), `politics.reformist_theory_decay` (0.02, `[0,1]`), `politics.co_optive_dependence_drift` (0.02, `[0,1]`) — defines.yaml:1126–1128.
- **(f) Events:** none.

### Step 6 — Delivery-gap measurement (`_delivery_gap`, doctrine.py:142–160; called from Step 2 and Step 5)
- **(a)** Sum of "promised minus delivered" over every row of the `policy_delivery` register whose `incumbent_id` matches this org, positive parts only.
- **(b)** `total += max(0, promised - delivered)` per matching row, iterating `delivery.values()` (dict insertion order — deterministic in CPython 3.7+, but depends on `policy_delivery`'s own writer, PolicySystem, preserving deterministic insertion — not independently verified here).
- **(c) Reads:** the `policy_delivery` graph register (`POLICY_DELIVERY_ATTR`, `policy.py:107`), one-tick-stale (PolicySystem @17.47 runs after Doctrine @14.7).
- **(d) Writes:** none.
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 7 — Mass-work SOLIDARITY decay (`_decay_mass_work_solidarity_edges`, doctrine.py:116–139; per org, every tick, AFTER `update_node`)
- **(a)** Every SOLIDARITY edge this org is the SOURCE of decays multiplicatively, floored at 0 — "a mass link not renewed by work withers." The write side (`_mass_work.py`, §5) is symmetric but lives in a different call path entirely.
- **(b)** `graph.update_edge(org_id, target, SOLIDARITY, solidarity_strength=max(0.0, strength*(1.0-decay_rate)))` (doctrine.py:134–139) — single-sided floor clamp idiom #4 again, bare `1.0` literal, multiply.
- **(c) Reads:** every `SOLIDARITY` edge where `source_id == org_id` (`graph.query_edges(edge_type=SOLIDARITY.value)`, then Python filter — doctrine.py:128–133), `edge.attributes["solidarity_strength"]`.
- **(d) Writes:** `solidarity_strength` on the same edges.
- **(e) Defines:** `doctrine.mass_work_solidarity_decay_rate` (float, default 0.02, `[0,1]`) — defines.yaml:984.
- **(f) Events:** none.

### Step 8 — Political-form position (`_political_form_position`, doctrine.py:219–253; per org, every tick, after the `update_node` flush)
- **(a)** The org's `(self_organization, representation)` position — self-organization (mass-link + solidarity, saturated) versus representation (mean of institutional pull, CO_OPTIVE share, saturated office tenure).
- **(b) Exact formulas:**
  - `autonomous = max(0.0, mass_link + solidarity)` (doctrine.py:246) — single-sided floor clamp idiom #4.
  - `self_organization = autonomous / (autonomous + 1.0)` (doctrine.py:247) — saturating-division idiom #1, third occurrence.
  - `tenure_saturated = tenure / (tenure + 1.0)` (doctrine.py:250) — saturating-division idiom #1, fourth occurrence, and a **duplicate computation** of the same formula Step 3's `office_tenure` already produced from the same input (harmless, redundant).
  - `representation = (institutional_pull + co_optive + tenure_saturated) / 3.0` (doctrine.py:252) — mean of three roughly-[0,1] quantities, bare `3.0` divisor.
- **(c) Reads:** `tags[MASS_LINK]`, `practice_env[SOLIDARITY_MASS]`, `office_tenure`, `institutional_pull`, `practice_env[CO_OPTIVE_SHARE]` (all already computed this tick).
- **(d) Writes:** none directly — folded into the `positions` dict, written once per tick as the whole-graph register (Step 9).
- **(e) Defines:** none.
- **(f) Events:** none.

### Step 9 — The political-form register write (`compute_doctrine`, doctrine.py:606–611)
- **(a)** After every org is processed, if ANY organizations exist, publish the whole `{org_id: {self_organization, representation}}` map as one graph-level attribute. An org-less world writes nothing (honest-absence, III.11).
- **(b)** `graph.set_graph_attr(POLITICAL_FORM_POSITIONS_ATTR, positions)` (doctrine.py:611) — a dict-of-dicts write, no arithmetic.
- **(c) Reads:** the `positions` dict accumulated across the per-org loop.
- **(d) Writes:** the `political_form_org_positions` graph register.
- **(e) Defines:** none.
- **(f) Events:** none.

**Events emitted by the whole system: 4 distinct `EventType` values** — `DOCTRINE_TRAP_SPRUNG`
(events.py:168), `DOCTRINE_TRAP_ESCAPED` (:169), `DOCTRINE_PURGE_FAILED` (:170),
`LINE_STRUGGLE_SPLIT` (:184) — all published from `DoctrineSystem.step` (doctrine.py:676–700) via
`services.event_bus.publish(Event(...))`. The `_KIND_TO_EVENT_TYPE` dict (doctrine.py:87–91) is
indexed directly (no `.get()` fallback) so an unrecognized `kind` string raises loudly rather than
silently dropping an event — a genuinely good honesty pattern, recorded as such.

**Defines declared in `DoctrineDefines` but NOT read by `DoctrineSystem.step`/`compute_doctrine` at
all** (read elsewhere, or unwired):
- `doctrine.faction_flip_enabled` (bool, default `False`) — zero reads anywhere in the codebase
  (grep-confirmed). Matches its own docstring ("OFF until Phase 2") — a deliberate, self-documented
  placeholder, not a defect.
- `doctrine.theory_bonus_per_class_analysis` — consumed by `ooda/action_effects.py:109`
  (`scaled_delta *= 1.0 + theory_bonus_per_class_analysis * min(...)`), a **different** system's
  consciousness-delta computation, reading the `CLASS_ANALYSIS` tag DoctrineSystem wrote the
  *previous* tick (OODA @14.0 runs before Doctrine @14.7). Same `DoctrineDefines` category, different
  consumer — a cross-system channel, not a DoctrineSystem computation.
- `doctrine.mass_work_solidarity_gain`, `doctrine.mass_link_weight` — consumed only by
  `engine/actions/_mass_work.py:101–105` (a verb resolver dispatched from OODA), never by
  `DoctrineSystem.step` itself.

---

## 3. TYPE INVENTORY

Runtime storage note (same as every other system inventoried): `BabylonGraph.update_node`/
`update_edge` (`topology/graph.py:660–670`, `690–702`) are plain dict merges — no type coercion, no
`SnapToGrid` quantization mid-tick. Pydantic constraints (`Organization`'s field bounds) apply only at
model construction/round-trip, never during a tick's `graph.update_node(...)` call.

| Attribute | Node/edge/graph scope | Python model type | Domain | Category |
|---|---|---|---|---|
| `acquired_doctrine_ids` | ORGANIZATION | `tuple[str, ...]` | subset of 14 closed tree-node ids, in acquisition order | **composite — sequence of content-id references; no BSL scalar shape** |
| `theoretical_labor` | ORGANIZATION | `float`, `ge=0.0` | `[0, ∞)` | **unbounded real accumulator** |
| `doctrine_tags` | ORGANIZATION | `dict[DoctrineTag, float]` | keys closed (3), values **NOT clamped by the tick loop** (see CAVEAT below) — docstring claims `[0,10]` but that is a different, uncalled function | **map-of-enum-to-unbounded-real — 3 independent accumulator scalars, not one field** |
| `congress_tag_snapshot` | ORGANIZATION | `dict[DoctrineTag, float]` | same shape/domain as `doctrine_tags` | same as above |
| `study_target_id` | ORGANIZATION | `str \| None` | one of 14 closed tree-node ids, or absent | **optional string content-id reference — no BSL type at all (no string type exists)** |
| `office_tenure` | ORGANIZATION | `float`, `ge=0.0` | `[0, ∞)` | unbounded real accumulator |
| `institutional_pull` | ORGANIZATION | `float`, `ge=0.0, le=1.0` | `[0, 1]` | bounded real — **declared with inline Field constraints, not the `Probability` Annotated type** other org fields (`cohesion`, `cadre_level`) use; a minor type-consistency inconsistency in the source, worth noting not fixing |
| `cadre_level` | ORGANIZATION | `Probability` (`Annotated[float, ge=0,le=1]`, `SnapToGrid`) | `[0,1]` | unit-interval — **read-only by DoctrineSystem** (written by `reproduce.py`, §5) |
| `cohesion` | ORGANIZATION | `Probability` | `[0,1]` | unit-interval — read-only by DoctrineSystem |
| `solidarity_strength` (edge) | SOLIDARITY edge | raw `float` in a plain dict (no Pydantic model — `GraphEdge.attributes` is untyped) | roughly `[0,1]` by construction (`_mass_work.py`'s `min(1.0, ...)` cap) but **unenforced on the edge itself** | bounded real, enforced only by convention |
| `edge_mode` (edge) | any incident edge | `str` value of `EdgeMode` (5-member StrEnum) | closed set | **enum discriminant, stored as a bare string on an untyped edge dict** |
| `office_tenure`/`institutional_pull` (same fields, cited again for the domain contrast) | — | — | — | see above |
| `policy_delivery` | graph-level register | `dict[str, dict[str, Any]]` (per-row `incumbent_id`/`promised`/`delivered`) | unconstrained | **nested register — no BSL substrate analog at all** |
| `electoral_governments` | graph-level register | `dict[str, dict[str, Any]]` (per-row `party_id`, etc.) | unconstrained | same as above |
| `political_form_org_positions` | graph-level register | `dict[str, dict[str, float]]` (`{org_id: {self_organization, representation}}`) | both poles `[0,1]` by construction | same as above — **but see §6: this specific register's per-org content is a much better fit for two ordinary `deffield`s ON the ORGANIZATION node than for a graph-scope register at all** |
| `tag_decay_rate`, `study_allocation_min/max`, `mass_work_solidarity_decay_rate` (defines) | — | `float` | `[0,1]` | unit-interval coefficients |
| `congress_interval_ticks` (define) | — | `int` | `>= 1` | positive-int coefficient |
| `trap_escape_tl` (define) | — | `int` | `>= 0` | non-negative-int coefficient |
| `congress_delta_weight` (define) | — | `float` | `>= 0.0`, **unbounded above** | unbounded real coefficient |
| `congress_contingency_floor` (define) | — | `float` | `(0, 0.5)` open interval | bounded real coefficient |
| the 8 `politics.*` coefficients (§2(e)) | — | `float` | `[0,1]` each | unit-interval coefficients |

**Enum discriminant flag — the same genuine gap Territory's inventory found, here with THREE distinct
enums.** `DoctrineTag` (3 members), `DoctrineTrunk` (3 members, static content only — never read at
tick time), `PracticeVariable` (5 members) all lack any BSL `deffield` storage representation
(`bsl-language.rst` §3.1's closed vocabulary is `{int, bool, currency, probability, intensity,
coefficient, enum}` — an `enum`-typed field IS now landed per ADR195/ADR196, so `DoctrineTag`/
`PracticeVariable` as VALUE DOMAINS could each become a `defenum`, but the actual per-tick DATA these
enums key — `doctrine_tags[TAG] = float`, a map — is what has no home: a `deffield ... enum
DoctrineTag` would store WHICH tag, not a float PER tag. The workable pattern (confirmed structurally
sound by the `doctrine_liquidation_absorbing.bsl` conformance vector, §1) is one scalar `deffield`
per enum member: 3 fields for `doctrine_tags` (`organization/class-analysis`,
`organization/mass-link`, `organization/militancy`), 3 more for `congress_tag_snapshot`, and — since
`PracticeVariable`'s 5 members are I-FRESH computed quantities, not stored state — no storage needed
for those at all (they'd be recomputed each tick from other stored fields, exactly as `_practice_env`
does today). This is tractable (6 extra scalar fields) — much smaller than the `acquired_doctrine_ids`
gap below.

**String/no-string-type flag — the single largest TYPE INVENTORY finding of this inventory, with no
precedent in Territory's report.** `acquired_doctrine_ids` (a growing subset of 14 closed content ids)
and `study_target_id` (an optional single id from the same 14) have **no BSL field type whatsoever** —
not merely an unlanded slice, but a category the type vocabulary does not contain at all (no string,
no list-of-anything, no optional-reference type exists in `deffield`'s six-plus-enum vocabulary).
Since the doctrine tree is a small, closed, static set of 14 node ids, the same "one bool/int-ordinal
per closed value" workaround Territory used for its 2/5-valued enums generalizes here, just at larger
scale: `acquired_doctrine_ids` → 14 separate `bool` deffields (`doctrine/acquired-<node-id>`, one per
tree node); `study_target_id` → 14 more `bool` deffields (or one `int`-ordinal `0..14` where `0`
means "none"). This is expressible with zero new grammar but is a genuinely larger content-modeling
and D-record burden than any single-enum gap this port-estate has recorded so far (up to 28 extra
boolean fields for one system, versus Territory's 1–2).

---

## 4. FLOAT-OP INVENTORY

**No libm transcendentals** — grep-confirmed zero `exp`/`log`/`sigmoid`/`pow` calls anywhere in
`doctrine.py`, `mechanics.py`, or `congress.py`. Unlike Metabolism, this system carries **no libm
nondeterminism hazard from its arithmetic** — but it is the first system inventoried in this wave to
carry a **different** determinism hazard: RNG (below).

**Four distinct "keep it bounded" idioms coexist in this one system** — more clamp-shape diversity than
any other system inventoried so far:

1. **Two-sided `min(max(x, lo), hi))`** — `purge_probability` (congress.py:82), `accrue_theoretical_labor`'s allocation clamp (mechanics.py:293), `PETTY_BOURGEOIS_DRIFT` (doctrine.py:202).
2. **Single-sided `max(0.0, x)`** (floor only) — `_decay_mass_work_solidarity_edges` (doctrine.py:138), `_apply_practice_drift`'s two tag floors (doctrine.py:338, 343), `_political_form_position`'s `autonomous` (doctrine.py:246).
3. **Single-sided `min(1.0, x)`** (ceiling only) — `_officeholder_capture`'s `resistance` and `pull` (doctrine.py:306, 309, twice).
4. **Saturating division `x / (x + 1.0)`** (asymptotic, never a hard clamp) — `OFFICE_TENURE`/`DELIVERY_DEPENDENCE` (doctrine.py:205, 208), `self_organization`/`tenure_saturated` (doctrine.py:247, 250) — four occurrences of the identical formula, two of them (`office_tenure`, `tenure_saturated`) genuinely redundant recomputations of the same value from the same input in the same tick.

This is a materially richer clamp-inconsistency finding than Territory's single "two clamp shapes for
the same field" note — here it is four idioms across nine call sites, none of them the SAME
conceptual quantity (unlike Territory's `heat`), so it does not read as an inconsistency-BUG so much
as an inconsistency-of-STYLE across the module — still worth a single port-time convention decision
(pick one nested-`if` shape, per landed-pack precedent, and use it uniformly) rather than transcribing
four different idioms verbatim into four different BSL shapes.

**Shapes, in execution order (mirroring §2):**

1. Congress: `raw = 0.5 + weight*delta` (congress.py:81) — bare `0.5` and `1.0` literals (idiom 1's clamp bounds), 1 multiply, 1 add.
2. Congress: `tl_after = tl - trap_escape_tl` (congress.py:118) — subtract, `int`→`float` cast.
3. Congress: per-tag `new_tags[k] = new_tags.get(k,0.0) - float(delta)` (congress.py:124–125) — subtract, `int`→`float` cast (trap `tag_deltas` are `Mapping[DoctrineTag,int]` on the content model).
4. Line struggle: `tl * retention` (doctrine.py:562) — plain multiply.
5. `PETTY_BOURGEOIS_DRIFT`: `min(1.0, max(0.0, 1.0 - cadre))` (doctrine.py:202) — subtract then idiom-1 clamp; bare `1.0` used three times in one expression.
6. `OFFICE_TENURE`/`DELIVERY_DEPENDENCE`: `x/(x+1.0)` (doctrine.py:205,208) — division, denominator never zero since `x >= 0` by construction (no divide-by-zero risk, unlike `CO_OPTIVE_SHARE`'s `co_optive/incident` below).
7. `CO_OPTIVE_SHARE`: `co_optive/incident if incident else 0.0` (doctrine.py:199) — a genuinely guarded division (Python-level `if`, not a clamp) — this IS a real divide-by-zero guard, distinct from the saturating-division idiom above.
8. `_officeholder_capture`: `tenure += 1.0` (doctrine.py:303) — bare-literal increment; `1.0 - min(1.0, cadre*cohesion)` (doctrine.py:306–308) — multiply then idiom-1-flavored subtract-from-one; `pull + capture_rate*resistance*(1.0-pull)` then idiom-3 clamp (doctrine.py:309) — a 3-factor multiply chain.
9. `_apply_practice_drift`: two independent `coeff*x + coeff*y` sums then idiom-2 floor-subtracts (doctrine.py:333–343).
10. `step_organization`: `decay_tags`'s `value * (1.0 - decay_rate)` (mechanics.py:309–310) — bare `1.0`, the SAME shape as `_decay_mass_work_solidarity_edges`'s edge decay (doctrine.py:138) and congress's implicit `(1-floor)` clamp bound — a recurring `x*(1-r)` decay idiom used identically in 2 places (tags, edges) but with the THIRD "keep bounded" idiom wrapped around the edge case and NONE wrapped around the tag case (tags can go negative — see CAVEAT).
11. `accrue_theoretical_labor`: `surplus * min(1.0, max(0.0, study_allocation))` (mechanics.py:293–294) — idiom-1 clamp then multiply; guarded by an early `if surplus <= 0.0: return 0.0` (mechanics.py:291–292), so the function itself never returns a negative value even though its formal domain is unrestricted.
12. `study_allocation` midpoint: `(min+max)/2.0` (doctrine.py:414) — bare `2.0` divisor (§2 Step 4 docstring-discrepancy note).
13. Root/candidate acquisition: plain subtracts (`tl -= cost_tl`, doctrine.py:431, 438) — `cost_tl` is `int` (Pydantic model field), `tl` is `float` — an implicit `int`→`float` promotion on every acquisition, never the reverse (no Real→Int demotion anywhere in this system — a genuine, clean contrast with Territory's two `int(...)` truncations).
14. `_political_form_position`: `max(0.0, mass_link+solidarity)` then `/(+1.0)` (doctrine.py:246–247, idiom 2 + idiom 4 chained); `(a+b+c)/3.0` mean (doctrine.py:252) — bare `3.0` divisor.

**RNG — the single most important float-adjacent finding of this inventory, with no analogue in
Territory's report.** `rng.random()` (doctrine.py:537, via `resolve_rng`, `system_base.py:35–55`) is
a **CPython `random.Random` (MT19937) draw**, the ONLY stochastic value anywhere in this system. This
is a real, distinct determinism hazard from the libm-transcendental class the user's own methodology
already names — worth stating precisely, since the *resolution is already ruled and this inventory
should not re-litigate it*:

- **The kernel-side replacement is landed, not merely designed.** `rust/crates/babylon-kernel/src/rng.rs`
  (201 lines, read in full) implements a **pinned `ChaCha8Rng`** with a per-`(session_id, tick,
  domain, stable_key)` counter-mode stream, seeded `SHA256(session‖tick_le8‖salt_le8‖len_le8(domain)‖
  domain‖len_le8(stable_key)‖stable_key)` — reusing the SAME salt constant (`0xBA1AC1A`,
  `_SYSTEM_RNG_SEED_SALT`) as the Python side, but explicitly, by its own module docstring: **"Streams
  differ from Python by design (R8): this is the pinned Rust-side replacement, not a port. Python's
  MT19937 streams are a closed epoch; stochastic baselines re-bless at cutover under
  ensemble-envelope comparison, not byte replay."**
- **DoctrineSystem's own stochastic family is already named and classified** in
  `reports/p27-tolerance-and-envelope-derivations.md` §4.1: `doctrine_tags`/`acquired_doctrine_ids`/
  `congress_tag_snapshot`/`institutional_pull` (via `faction_balance`/`legitimation_index`/doctrine
  tags, the 5 electoral goldens) are classified STOCHASTIC and get an **ensemble (mean, stdev)
  envelope over 32 seeds**, explicitly NOT a per-value byte comparison — "R8: the RNG algorithm itself
  changes at cutover, so per-value comparison is meaningless for these families; that weakening is
  accepted and stated." `reports/p27-porting-contract-table.md` row 18 independently confirms
  DoctrineSystem is one of the six RNG-touching systems (`resolve_rng` call site named exactly:
  `doctrine.py:673`).
- **What is genuinely still missing (the actual blocker, distinct from the algorithm question above):**
  there is **no RNG intrinsic callable from BSL content at all today.** `DECLARABLE_INTRINSICS` =
  `["exp","log","floor"]` (declarations.rs:110) — no `rng`/`random` name. `bsl-language.rst` §3.10's
  own rider table (row 11, "RNG draw") says "Not a rider. §2.8 already sanctions it as a kernel
  intrinsic; the key convention is below" — and the very next paragraph is explicitly headed
  "**[draft ruling — Phase 1 review]**," and states "**The signature stays Phase-2 work** (§2.7)."
  §2.8 itself (line 1621) states plainly: "There is no ... randomness primitive" in the rule language
  as it stands. The carrier-key SHAPE is fixed (`(session, tick, domain, stable_key)`, all four
  components language-visible per the draft ruling) but the actual calling convention a `.bscl` rule
  would use to invoke a draw does not exist yet, is not declarable, and is not dispatchable.

---

## 5. CROSS-SYSTEM CHANNELS

- **Tick position: 14.7** (doctrine.py:626), confirmed against `_SYSTEM_CLASSES`
  (`simulation_engine.py:328–363`): `... TickDynamics(1)... OODASystem(14.0) → FactionInfluenceSystem
  (14.5) → DoctrineSystem(14.7) → SurvivalSystem(15.0) → StruggleSystem(16.0) → ...→ AllegianceSystem
  (17.42) → ElectoralSystem(17.45) → PolicySystem(17.47) → SovereigntySystem(17.5) →
  MarketScissorsSystem(17.8) → ContradictionSystem(18.0) → ...`.

- **Reads from a same-tick prior system (real, verified):** `attrs["cadre_level"]`/`attrs["cohesion"]`
  are written by the **REPRODUCE verb resolver** (`engine/actions/reproduce.py:37–100`), dispatched
  by OODASystem @14.0 during action resolution — a genuine same-tick prior-write channel: an org that
  trains cadre this tick sees the higher `cadre_level` feed straight into `_officeholder_capture`'s
  `resistance` term and Step 4's `accrue_theoretical_labor` in the SAME tick.

- **Reads that are one-tick-stale by pipeline position (verified, both self-documented in the source
  AND confirmed against `_SYSTEM_CLASSES` ordering):**
  - `policy_delivery` (`_delivery_gap`) — written by PolicySystem @17.47, which runs AFTER Doctrine
    @14.7. Doctrine therefore always reads LAST tick's delivery-gap register.
  - `electoral_governments` (`_officeholder_capture`) — written by ElectoralSystem @17.45, same
    situation (one-tick-stale).

- **Writes consumed later this tick / downstream ticks (grep-confirmed across
  `src/babylon/engine/systems/*.py`):**
  - `acquired_doctrine_ids` — read by **PolicySystem** (`policy.py:691`, the entryism gate) and
    **ElectoralSystem** (`electoral.py:356,590,598,793`, several entryism-stance gates) — both
    downstream same-tick (17.45/17.47 > 14.7), so these see THIS tick's acquisitions, not last tick's.
  - `institutional_pull` — read by **PolicySystem** (`policy.py:493`) — downstream same-tick.
  - `solidarity_strength` (decayed by Step 7) — read broadly across the estate: `policy.py:535`,
    `ideology.py:320–342`, `reactionary.py:200`, `electoral.py:1236`, `survival.py:49–58`,
    `community.py:534–574`, `struggle.py:384–398`, `solidarity.py:83–181` — this is a widely-shared
    attribute, not a Doctrine-specific channel, but Doctrine's decay write on org-sourced edges is one
    of several writers into the same shared pool (the symmetric write side is `_mass_work.py`, not
    part of this system).
  - `political_form_org_positions` — read by **ContradictionSystem** @18.0 (`contradiction.py:562–593`,
    `_political_form_positions`), downstream same-tick, one tick stale in the sense the docstring
    itself claims ("read by ContradictionSystem @18.0 one tick stale by pipeline position: I-ORD
    compliant," doctrine.py:78–79) — **this claim is actually WRONG as I-ORD framing goes**: 18.0 runs
    AFTER 14.7 in the SAME tick, so ContradictionSystem reads THIS tick's freshly-written positions,
    not a stale one. Verbatim-recorded as found (the source docstring's own "one tick stale" language
    appears to be describing the *practice-variable inputs* that feed the position, not the register
    read itself — but the sentence as written is imprecise).
  - `theoretical_labor`, `doctrine_tags`, `congress_tag_snapshot`, `study_target_id`, `office_tenure` —
    grep-confirmed **read by no other engine System**. Terminal/observational outputs from the engine's
    point of view (consumed only by AI/state-observer/web layers, out of scope for this port).

- **Context/service usage with no BSL equivalent:** `resolve_rng(services, tick)` (doctrine.py:673) —
  see §4's RNG finding. No `TickContext` field is read directly by DoctrineSystem (unlike Territory's
  `displacement_mode`) — `context.tick` (doctrine.py:651) is the only context read, and `:tick` IS a
  landed BSL binding source (bsl-language.rst §2.5) — this specific read is portable now.

- **The graph-register triple (`policy_delivery`, `electoral_governments`, `political_form_org_positions`)
  is a genuinely distinct cross-system-channel finding, novel to this inventory.** All three are
  `graph.get_graph_attr`/`set_graph_attr` reads/writes — a Python-engine-only concept
  (`BabylonGraph._graph_attrs`, a plain untyped dict, `topology/graph.py:892–898`) with **zero BSL
  substrate representation** as such. The `[draft ruling — Phase 1 review, R9 chapter C3]`
  (bsl-language.rst §3.6) explicitly rules that "graph-scope state is ordinary node state on a
  declared carrier node type" (a singleton `NodeType` with manifest `:ceiling` 1) for values that are
  "genuinely one-per-graph" — but explicitly does NOT extend that to per-entity registers ("it does
  not make every register a singleton: per-sovereign and per-county registers are ordinary nodes of
  ordinary types"). Applying that distinction here: `policy_delivery` and `electoral_governments` are
  genuinely PER-ROW registers (keyed by policy-item/government id) — out of DoctrineSystem's own port
  scope; their eventual BSL shape is PolicySystem's/ElectoralSystem's own design decision, and
  DoctrineSystem's port is **sequenced behind both of theirs**. `political_form_org_positions`, by
  contrast, is a PER-ORGANIZATION pair of floats that DoctrineSystem itself both computes and writes —
  its natural BSL home is NOT a register at all but two ordinary `deffield`s directly on
  `NodeType/ORGANIZATION` (`organization/self-organization`, `organization/representation`), written
  by the same `update-node` call the rest of the per-org state already needs. This reclassifies what
  looked like a third register-shaped blocker into a much smaller, already-solved-shape problem —
  contingent only on the underlying practice-variable computation (Slice 2) being available.

- **DORMANCY on canonical scenarios — the headline correction of this inventory, verified three
  independent ways.** DoctrineSystem's own module docstring (doctrine.py:36–43) and the property-law
  test grounding it (`test_law_doctrine_system.py:11–22`, L1) both assert: "Byte-safe on the
  qa:regression goldens by construction: all SIX scenarios carry no organization nodes (org_count=0)
  ... this system is a no-op." The OFFICIAL Program-27 porting-contract table repeats the same claim
  verbatim (`reports/p27-porting-contract-table.md` row 18: "no ORGANIZATION nodes are seeded in any
  canonical scenario"). **All three are stale on current dev, verified as follows:**
  1. `tools/regression_scenarios.py:37–125`'s `SCENARIOS` dict (the actual registry `qa:regression`
     iterates, `tools/regression_test.py:1424`, `for name in SCENARIOS`) includes `"org_probe"`
     (line 121–125), dispatching to `create_org_probe_scenario` (`engine/scenarios/org_probe.py:60–163`),
     which seeds **two real `NodeType.ORGANIZATION` nodes** (`CivilSocietyOrg` cadre_level=0.1,
     `StateApparatus` cadre_level=0.6).
  2. `tests/baselines/org_probe.json` and `tests/baselines/dense/org_probe.csv` both exist as
     COMMITTED baselines (`PENDING_CEREMONY` is an empty frozenset, `regression_scenarios.py:139` —
     nothing, including `org_probe`, is pending), so `qa:regression compare` actively byte-checks this
     scenario every run.
  3. `.mise.toml:974` — `qa:vault-regression-ci` explicitly names `--scenario single_county --scenario
     org_probe` as its two CI-lane scenarios, confirming `org_probe` is a load-bearing part of the
     live CI gate, not a stray leftover fixture.
  Both org_probe orgs have `cadre_level > 0`, so `accrue_theoretical_labor` returns a positive value
  every tick and the per-org tick loop (decay/accrue/root-bootstrap/greedy-acquire/trap-check) runs
  for real, every tick, on every `qa:regression` run — this is NOT a no-op today. The default
  `DEFAULT_MAX_TICKS = 52` (`tools/regression_test.py:81`) exactly equals the default
  `congress_interval_ticks` (52), so `is_congress` is TRUE at the final tick of a standard regression
  run (`52 % 52 == 0`) — **structurally reachable**, verified from source; whether a trap is actually
  HELD and AFFORDABLE by tick 52 for org_probe's specific orgs (which would make `needs_roll` true and
  actually draw from the RNG stream) depends on the OODA-driven acquisition path across 52 ticks and
  was NOT independently re-traced here (would require running the simulation, out of scope for a
  read-only inventory) — flagged as UNVERIFIED, not asserted either way.
  **What IS still uncovered:** `graph_content_hash` (`tools/regression_test.py:924–964`) hashes every
  node/edge attribute (so `acquired_doctrine_ids`/`theoretical_labor`/`doctrine_tags`/etc. on
  org_probe's two orgs ARE part of the byte-identical gate) but explicitly EXCLUDES graph metadata
  (`g.graph`) — so the `political_form_org_positions`/`policy_delivery`/`electoral_governments`
  register traffic remains completely outside any byte-identical check, even now. The `dense` per-tick
  CSV trace also carries no Organization-node columns at all (org_probe's dense CSV header has only
  `C900_*`/`edge_C900_T900_*`/economy columns — no `org/probe-*` columns), so per-tick doctrine drift
  would only surface via the coarser checkpoint `content_hash`, not the fine-grained dense diff other
  systems get.

---

## 6. BLOCKER ASSESSMENT

| Computation | Verdict | Detail |
|---|---|---|
| Step 0 congress-gate check (`tick % interval == 0`) | **PORTABLE NOW** | `:tick` is a landed bind-src (§2.5); modulo against a `:const` — same "calendar reads are bindings" pattern (`:tick-in-cycle`) already ruled for other systems. |
| Step 1 Party Congress — the RNG purge roll (doctrine.py:537, congress.py:85–142) | **BLOCKED — no RNG intrinsic in BSL content.** | `DECLARABLE_INTRINSICS = {exp, log, floor}` (declarations.rs:110) has no RNG name; bsl-language.rst §3.10 states "the signature stays Phase-2 work" for the RNG draw — the carrier-key SHAPE is drafted, the calling convention is not. The underlying kernel service (`ChaCha8Rng`, `rng.rs`) IS landed but is not wired to any content-callable intrinsic. Even once it lands, this is **not a byte-identical port target**: R8 already rules Python's MT19937 stream and the Rust ChaCha8 stream diverge by design, and `reports/p27-tolerance-and-envelope-derivations.md` §4.1 already assigns this exact family an ensemble-envelope (32-seed mean±2σ) conformance methodology instead of byte replay — the eventual port needs a NEW conformance vector shape, not a transcription of the frozen roll sequence. |
| Step 1 line-struggle resolution (doctrine.py:476–495) | **PORTABLE WITH D-RECORD** (contingent on the acquired-set representation) | Pure comparison/subtraction/tuple-filter logic, no RNG of its own — but operates on `acquired_doctrine_ids`, which itself needs the 14-bool-field D-record (§3) before this can be expressed at all. |
| Step 2 practice-variable measurement — `SOLIDARITY_MASS` (doctrine.py:187–189) | **BLOCKED — edge-attribute reads (Slice 2).** | Needs `field-of` over an `EdgeRef` to read `solidarity_strength` — `field-of` serves `NodeRef` referents only today (`evaluator.rs:1185–1191`: "an `EdgeRef` referent is unreachable today ... slice 2 mints `EdgeKey`"). Reformulable favorably as `(fold sum (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) (field-of it solidarity/strength))` — structurally clean, same shape as bsl-language.rst's own worked `sum_strength` example (table row, §2.4) — once Slice 2 lands, this specific sub-computation is a strong candidate for early portability given it already has a normative worked example. |
| Step 2 practice-variable measurement — `CO_OPTIVE_SHARE` (doctrine.py:191–199) | **BLOCKED — edge-attribute reads (Slice 2) AND untyped/all-edge-type incidence.** | Needs `edge_mode` read off an `EdgeRef` (same Slice 2 gap) PLUS an "every incident edge regardless of type, regardless of direction" traversal — `neighbors`'s grammar (`(neighbors <expr> <EdgeType> <direction> <NodeType>)`, bsl-language.rst line 946) requires naming ONE `EdgeType` and ONE target `NodeType` per call; reconstructing "all 6 declared edge types, both directions, any target type" needs up to 12 separate `neighbors`/`fold count` calls unioned in content — expressible in principle (closed, finite vocabulary) but a real authoring-scale burden even after Slice 2 lands, distinct from the pure-storage blocker. |
| Step 2 practice-variable measurement — `PETTY_BOURGEOIS_DRIFT`/`OFFICE_TENURE`/`DELIVERY_DEPENDENCE` (doctrine.py:202,205,208) | **PORTABLE WITH D-RECORD, contingent on inputs.** | The arithmetic itself (clamp, saturating division) is trivial and matches landed-pack idioms (nested `if` for the clamp; division is a basic op). `PETTY_BOURGEOIS_DRIFT` needs only `cadre_level` (a stored node field — portable now). `OFFICE_TENURE` needs `office_tenure` (a stored field, portable now — see next row). `DELIVERY_DEPENDENCE` needs `_delivery_gap`, which is BLOCKED on the `policy_delivery` register (below) — not independently portable. |
| `_delivery_gap` (doctrine.py:142–160) | **BLOCKED — no BSL representation for `policy_delivery`, AND sequenced behind PolicySystem's own port.** | `policy_delivery` is a Python-engine-only per-row graph register with no substrate analog; its eventual BSL shape is PolicySystem's design decision, not yet made (PolicySystem is unported). Not a "missing slice," a genuine cross-system sequencing dependency. |
| `_officeholder_capture` (doctrine.py:270–310) | **BLOCKED — no BSL representation for `electoral_governments`, sequenced behind ElectoralSystem's own port.** | Same class of blocker as `_delivery_gap`, one register over. The arithmetic body itself (idioms 1/3, §4) is trivial once the `governs` boolean is available from SOME source. |
| Step 4 tag decay + TL accrual + root bootstrap + directed-study/greedy acquisition (step_organization, doctrine.py:379–443) | **PORTABLE NOW, structurally** — **BLOCKED on the `acquired_doctrine_ids`/`study_target_id` storage gap.** | The arithmetic (decay, clamp, multiply, `min`-by-key tie-break selection — a `select-min`-shaped operation, landed per Slice 1) and the tree-content lookups (parents/cost_tl/tag_deltas as static `defconst`-style content) are all expressible with landed grammar. The blocker is purely the storage gap (§3): there is nowhere to write "which of the 14 nodes are acquired" or "which node is the study target" without the 14(+14)-bool-field D-record. |
| Step 4 trap-firing (`evaluate_trap_condition`, doctrine.py:444–460, mechanics.py:214–234) | **PORTABLE NOW for the DSL/boolean logic — PROVEN, not just argued** (`doctrine_liquidation_absorbing.bsl` already transcribes the current 3-clause practice-gated condition as a working `when (and ...)` guard with `:field`/`:const` bindings, zero new grammar). **BLOCKED end-to-end** because the `:field`-bound quantities it consumes (`solidarity-mass`, `co-optive-share`, `petty-bourgeois-drift`) are themselves not computable yet (Step 2's blockers above) — the existing conformance vector sidesteps this by treating them as pre-stored simple fields, which they are not in the frozen system. |
| Step 5 practice-driven tag drift (doctrine.py:313–344) | **PORTABLE WITH D-RECORD, contingent on inputs.** | The arithmetic (two coefficient-weighted sums, two floor-clamped subtracts) is trivial and matches landed idioms. Blocked transitively on `delivery_gap` (register) and `CO_OPTIVE_SHARE` (edge reads) being available — not independently portable, but nothing about the drift computation ITSELF needs new BSL capability. |
| Step 6 mass-work SOLIDARITY decay (doctrine.py:116–139) | **BLOCKED — edge-attribute READ and WRITE (Slice 2), confirmed from BOTH directions in the evaluator's own source.** | Read: same `field-of`-over-`EdgeRef` gap as `SOLIDARITY_MASS`. Write: `structural_verbs.rs:693–699` REFUSES `update-edge` outright at dispatch, with an explicit reason: "GraphSubstrate keys an edge to one f64 strength ... widening that state widens the canonical state_hash field set, which is a declared Phase-2/substrate decision." Nuance: the storage slot for a scalar edge "strength" DOES already exist and IS seedable (`organization-foundation.bscn`'s `(edge EdgeType/SOLIDARITY reading-group precinct 1)`) — the value can be born into the substrate at scenario-load time, but no rule can read or rewrite it at tick time today. |
| Step 8 political-form position (doctrine.py:219–253) | **PORTABLE WITH D-RECORD, contingent on inputs.** | Pure arithmetic (idioms 2/4, a 3-way mean) over already-computed quantities — no new capability needed for the FORMULA itself once `SOLIDARITY_MASS`/`CO_OPTIVE_SHARE`/`office_tenure`/`institutional_pull` are all available (all four independently blocked above). |
| Step 9 the `political_form_org_positions` register write (doctrine.py:606–611) | **NOT-A-PACK AS WRITTEN — RECLASSIFIES CLEANLY.** | A Python-engine-only "graph register" write has no BSL substrate equivalent and never will under the current architecture (§3.6's carrier-node-type ruling is explicitly for one-per-graph values, and this is per-organization). The content itself, however, is exactly two ordinary `deffield`s per ORGANIZATION node (`organization/self-organization`, `organization/representation`) written by the SAME `update-node` call the rest of DoctrineSystem's per-org state needs — reclassify as **PORTABLE WITH D-RECORD** (the D-record documents "no register; two node fields instead," a genuine but small content-modeling deviation, not a missing-capability blocker) once Step 2/3's inputs are unblocked. |
| Root iteration over ORGANIZATION nodes (`graph.query_nodes(node_type=NodeType.ORGANIZATION)`, doctrine.py:523) | **PORTABLE NOW.** | Typed node iteration is exactly Slice-1's landed `nodes`/`for-each`/rule-anchoring surface; `organization.bsl`'s existing pack already demonstrates a rule anchored at `NodeType/ORGANIZATION`. |

**Summary read:** of the 9 computation steps + the type-storage substrate they all sit on, **3 are
portable now** (congress-gate check, node iteration, the trap-condition DSL's boolean grammar in
isolation), **6 are "portable with D-record" contingent on inputs that are themselves blocked**
(meaning the ARITHMETIC is never the obstacle anywhere in this system — every single blocked
computation is blocked on STORAGE or DATA ACCESS, never on missing math), and **the hard blockers are
exactly three, independently named and independently tracked elsewhere in the estate**: (1) edge-
attribute read/write, Slice 2 — blocks `SOLIDARITY_MASS`, `CO_OPTIVE_SHARE`, and the mass-work
SOLIDARITY decay; (2) the acquired-set/study-target storage gap — no string/list-of-id type exists,
needs a 14(+14)-bool-field D-record; (3) the RNG-as-BSL-intrinsic gap — not even fully speced yet, and
even once landed is explicitly a non-byte-identical, ensemble-envelope conformance target by Director
ruling (R8), not a "wait and transcribe" item.

---

## 7. TEST/BASELINE SURFACE

| Test file | Lines | Relevance |
|---|---|---|
| `tests/unit/engine/systems/test_doctrine_system.py` | 1011 | **Primary conformance-oracle candidate.** Exhaustive coverage of `compute_doctrine`/`step_organization` via `SimulationEngine`-adjacent fixtures: decay, TL accrual, root bootstrap, directed-study vs. greedy acquisition, trap firing, Party Congress (weighted roll, escape/fail, snapshot rebaselining), line-struggle resolution, `_practice_env`/`_officeholder_capture`/`_political_form_position` individually, event publication for all 4 EventTypes. |
| `tests/unit/engine/laws/test_law_doctrine_system.py` | 289 | **Property-based invariant contracts** (hypothesis-based): L1 org-less inactivity (**now stale per §5's DORMANCY finding** — grounded explicitly on the same "all SIX scenarios" claim this inventory corrects), L2 `theoretical_labor >= 0` clamp, L3 acquired-set monotonicity on an ordinary (non-congress) tick, L4 graph-shape invariance (no add/remove node/edge). Also documents, as an explicit CAVEAT rather than a law, that `doctrine_tags` are NOT floored at 0 by the ordinary tick loop despite `DoctrineTag`'s docstring implying `[0,10]` — a genuinely useful, source-grounded behavioral note for the port's own conformance-scenario design (a tag CAN go negative in real play; the port must transcribe that, not "fix" it). |
| `tests/unit/domain/doctrine/test_mechanics.py` | 271 | Unit-3 pure mechanics: `evaluate_trap_condition`'s DSL grammar (all 6 comparisons, AND/OR/NOT, parens), `can_acquire`/`acquire`/`accrue_theoretical_labor`/`decay_tags`. Its own module docstring cites the **pre-U11** liquidationism condition as "the real MVP" condition — stale relative to the shipped tree (§2 CONTENT note), though the code under test (the general DSL evaluator) is unaffected by which specific condition string is used as an example. |
| `tests/unit/domain/doctrine/test_congress.py` | 191 | Unit-5 Party Congress: `held_sprung_traps`, `tag_delta_score`, `purge_probability`, `run_congress` — pure-function unit coverage, roll injected by the caller (no RNG dependency in the test itself). |
| `tests/unit/domain/doctrine/test_doctrine_tags.py` | 197 | Covers `compute_tags`/`starting_tags` — confirmed **not part of the live tick path** (§1/§2). Schema/reference-data-level testing, not a tick-behavior conformance oracle for the port. |
| `tests/unit/domain/doctrine/test_doctrine_validation.py` | 150 | Structural DAG/root/tier/trap/goal validity — load-time only, not tick behavior. Useful for validating a ported tree's CONTENT authoring (the 14-node transcription into BSL `defconst`s), not the system's per-tick math. |
| `tests/unit/models/test_doctrine.py` | 237 | `DoctrineNode`/`DoctrineTree`/`DoctrineCapability` Pydantic model validation — schema-level. |
| `tests/unit/engine/systems/test_doctrine_consciousness_signal.py` | 443 | **A genuine end-to-end production-path conformance oracle**, not fixture-stamped: drives the REAL `SimulationEngine.run_tick` in engine order (not hand-injected `doctrine_tags`) through a real `resolve_campaign` verb dispatch, proving DoctrineSystem's `trade_unionism` acquisition feeds `_mass_work.py`'s SOLIDARITY-edge creation which ConsciousnessSystem then reacts to — the Unit-6b sign-predictability contract (ADR073/ADR087). Explicitly contrasts itself against every earlier doctrine test's fixture-stamped `doctrine_tags` injection (module docstring, lines 1–24) — the single best evidence in the test estate that DoctrineSystem's real production behavior (not a mocked shortcut) has been exercised end-to-end at least once. |

**qa:regression byte-gate coverage — corrected per §5's DORMANCY finding.** `graph_content_hash`
(`tools/regression_test.py:924–964`) hashes every node/edge attribute of every scenario's
`WorldState→graph` projection, INCLUDING `org_probe`'s two ORGANIZATION nodes — so
`acquired_doctrine_ids`/`theoretical_labor`/`doctrine_tags`/`office_tenure`/`institutional_pull`/
`study_target_id`/`congress_tag_snapshot` are all real, live, byte-checked coverage today, contrary to
the system's own module docstring and its property-law test's grounding claim. What remains genuinely
UNCOVERED by any byte-identical mechanism: the three graph-level registers (`graph_content_hash`
explicitly excludes graph metadata, `g.graph`) and the dense per-tick CSV trace (org_probe's dense
CSV carries zero Organization-node columns — only `C900_*`/`edge_C900_T900_*`/economy columns). A
port's own conformance fixtures will still need to be hand-built for anything beyond what org_probe's
checkpoint-level hash happens to exercise, particularly the Party Congress branch (structurally
reachable at tick 52 but whether it actually draws from the RNG stream on org_probe's specific orgs
was not independently re-traced here — UNVERIFIED, see §5).

---

## Adjudication (2026-08-12)

Adjudicated against the **current dev tree** (`9324482f`). This is the strongest inventory in its
batch on the two facts the sibling reports got wrong — it caught the `org_probe` staleness
independently and verified it three ways, and it read `babylon-kernel/src/rng.rs` instead of
grepping only `babylon-bsl`/`babylon-tick`. Both findings re-verify exactly. Its BLOCKED verdict
stands. Two corrections to the blocker taxonomy, and one **material coverage omission**: the
system whose entire subject matter is the Director-reserved doctrine line carries no
RESERVED-LINE section at all.

1. **CORRECTION — no RESERVED-LINE flag anywhere in this inventory.** `rg -ni
   'reserved.line|director'` over the report returns exactly one hit (line 489), and it is about
   RNG conformance methodology, not ideology. Both sibling inventories in this batch flag theirs
   (OODA: the `CLASS_ANALYSIS` theory bonus and the `StateFaction`→`PolicyAxis` proxy;
   FactionInfluence: `colonial_stance` and the four seed factions). This system **is** the
   reserved surface: the 14-node tree, its three trunks, the two traps, the five-stance reformist
   fork, the liquidationism absorbing state and every trap-condition threshold are Director-owned
   MLM-TW content under Constitution §IX.5 / Amendment AD and ADR073/ADR137. Verified content:
   `src/babylon/data/game/doctrine_tree_mvp.json` carries exactly the 14 ids §2's CONTENT NOTE
   lists (`class_consciousness`, `trade_unionism`, `abstention_boycott`,
   `class_struggle_elections`, `entryism`, `independent_ballot_line`, `governance_road`,
   `liquidationism`, `democratic_centralism`, `mass_line`, `united_front`, `armed_vanguard`,
   `urban_guerrilla`, `adventurism`). The report describes this content correctly throughout and
   proposes nothing on it — so the omission is a **labelling** failure, not a line violation, but
   a port train reading this inventory would not learn that transcribing the tree into `.bscn`
   `defconst`s is a Director-gated act rather than a mechanical one.

2. **CORRECTION — the greedy-acquisition row over-reads Slice 1: `select-min` cannot range over a
   content-derived set.** §6's Step-4 row grades `min(acquirable, key=lambda nid: (cost_tl, nid))`
   (`mechanics.py:110-113`) as "a `select-min`-shaped operation, landed per Slice 1," and the
   summary concludes "the ARITHMETIC is never the obstacle anywhere in this system — every single
   blocked computation is blocked on STORAGE or DATA ACCESS." Verified against the evaluator:
   `select-max`/`select-min` — like `fold`, `exists`/`forall` and `for-each` — take a **§2.6
   query form** as their operand and hand it straight to `query::materialize`
   (`evaluator.rs:968`), and `materialize` serves exactly two heads, `nodes` and `neighbors`
   (`query.rs:110-112`); everything else is refused by name (`:117-125`). The acquirable set here
   is a subset of the doctrine **tree** — static content, not graph nodes — so there is no query
   to materialize and no landed form that iterates it. The same applies to the trap loop over
   `_reachable_traps` (`doctrine.py:347-358`). This does not change the row's BLOCKED verdict, but
   it widens the blocker: even after the 14-bool-field D-record lands, "the cheapest acquirable of
   14 content nodes" must be hand-unrolled in content, and the claim that arithmetic/iteration is
   never the obstacle in this system is not supportable as written.

3. **CORRECTION — the verdict line's "RNG-as-BSL-intrinsic not yet even fully speced" understates
   the report's own §4/§6 findings.** §4's RNG subsection and §6's Step-1 row are precise and
   correct: the kernel algorithm is landed (`rust/crates/babylon-kernel/src/rng.rs:69-95`,
   `ChaCha8Rng`, `for_carrier(session_id, tick, domain, stable_key)`, `seed_for`'s length-framed
   SHA-256 at `:53-63`), the carrier key IS fixed by the §3.10 draft ruling
   (`docs/reference/bsl-language.rst:3393-3411`), and what remains is the signature plus a
   dispatch arm. The headline verdict then compresses that to "not yet even fully speced," which
   reads as a *design* gap. It is an *implementation-binding* gap: a name in
   `DECLARABLE_INTRINSICS` (`declarations.rs:110`, today `["exp","log","floor"]`) and an arm in
   `KernelIntrinsicHost::call` (`intrinsic_host.rs:59-70`, today `floor` alone) — the identical
   category as the `exp` gap the sibling Survival inventory names. Amend the verdict line; the
   body needs no change.

4. **CONFIRMATION — the `org_probe` dormancy correction, verified independently and three ways.**
   `"org_probe"` sits in the `SCENARIOS` registry that `qa:regression compare` iterates
   (`tools/regression_scenarios.py:128`; `tools/regression_test.py:1424`, `for name in
   SCENARIOS`); `PENDING_CEREMONY` is an empty frozenset (`:143`); `tests/baselines/org_probe.json`
   and `tests/baselines/dense/org_probe.csv` are both committed; `.mise.toml:974` names
   `org_probe` as one of the two CI-lane vault scenarios. `create_org_probe_scenario` seeds a
   `CivilSocietyOrg` (`cadre_level=0.1`) and a `StateApparatus` (`cadre_level=0.6`) at
   `src/babylon/engine/scenarios/org_probe.py:112-146`, both with `cadre_level > 0`, so
   `accrue_theoretical_labor` returns a positive value every tick. DoctrineSystem's own module
   docstring, its L1 property law, and `reports/p27-porting-contract-table.md` row 18 are all
   stale, exactly as reported. This is the single highest-value finding across the five
   inventories adjudicated in this batch, and the sibling OODA inventory reported both halves of
   the same contradiction and resolved it the wrong way.

5. **CONFIRMATION — the dense-golden blind spot.** `head -1 tests/baselines/dense/org_probe.csv`
   yields exactly 20 columns: `tick`, seven `economy_*`/`financial_*`, ten `C900_*`, two
   `edge_C900_T900_*`. Zero Organization columns. So per-tick doctrine drift surfaces only through
   the coarser checkpoint `content_hash`, precisely as §5 states, and the three graph registers
   sit outside every byte mechanism (`graph_content_hash`'s own docstring,
   `tools/regression_test.py:936-943`, excludes `g.graph` metadata).

6. **CONFIRMATION — the tree really does carry 14 nodes, and the "11-node MVP" docstrings really
   are stale.** Counted from the JSON directly; the four cited docstrings still say 11. Recording
   it verbatim rather than correcting it is the right port-as-is call.

7. **CONFIRMATION — the edge lane, from both directions.** `update-edge`/`update-hyperedge` are
   grammar-recognised and hard-refused with the reason named (`structural_verbs.rs:709-710`,
   module doc `:16-26`: edge state is one `f64` strength keyed by `(type, from, to)`; widening it
   widens the canonical `state_hash` field set). `field-of` over an `EdgeRef` is unreachable
   (`evaluator.rs:1190-1192`). The nuance §6's Step-6 row draws — the scalar strength slot exists
   and is seedable at scenario load, but no rule can read or rewrite it — is exactly right.

8. **CONFIRMATION (with a refinement) — Step 7's mass-work SOLIDARITY decay is inert on the
   canonical estate, but not for the reason the wider estate assumes.** Two canonical scenarios
   now seed a **non-zero** `solidarity_strength`: `debs` (`electoral_goldens.py:474`,
   `_solidarity(_WORKER, "C005", 0.4)`) and `bernie_valve` (`:534`,
   `_solidarity(_WAYNE_WORKER, "C006", 0.4)`), which makes the estate-wide
   "`solidarity_strength=0.0` in every canonical scenario" gap row
   (`tools/regression_scenarios.py:2833-2836`) stale. Those two edges are class→class, and
   `_decay_mass_work_solidarity_edges` filters `source_id == org_id` (`doctrine.py:128-133`), so
   Doctrine's own decay still never fires on them — the conclusion survives, the premise the rest
   of the estate cites does not. Worth recording because the sibling Survival inventory built a
   `:const 1.0` D-record on that same stale premise and is wrong as a result.

9. **CONFIRMATION — tick position and the register readerships.** `position: ClassVar[float] =
   14.7` (`doctrine.py:626`), between `FactionInfluenceSystem` (14.5, `faction_influence.py:53`)
   and `SurvivalSystem` (15.0, `survival.py:78`) in `_SYSTEM_CLASSES`
   (`simulation_engine.py:328-364`). The one-tick-stale reads of `policy_delivery` (PolicySystem
   @17.47) and `electoral_governments` (ElectoralSystem @17.45,
   `ELECTORAL_GOVERNMENTS_ATTR` written at `electoral.py:885,1005`) re-derive correctly from that
   ordering, and §5's correction of the source docstring's own "one tick stale" language for
   `political_form_org_positions` (ContradictionSystem @18.0 reads it **this** tick) is right.

**FINAL VERDICT: BLOCKED — sustained, with the blocker taxonomy widened and the verdict line
amended.** The three hard blockers are (1) edge-attribute read/write, Slice 2; (2) the
acquired-set/study-target storage gap (no string, list, or optional-reference type in
`deffield`'s closed seven-name vocabulary, `declarations.rs:646-675`) — **plus**, newly, the
absence of any construct that iterates a content-derived set, so the greedy/trap selection is not
recovered by the bool-field workaround alone; (3) the RNG binding, which is an
implementation-binding gap (a `DECLARABLE_INTRINSICS` name + a `KernelIntrinsicHost` arm), not an
unspecified one, and whose conformance target is already ruled to be a 32-seed ensemble envelope
rather than byte replay. The report's own headline claim that "arithmetic is never the obstacle
anywhere in this system, only storage/data-access" is **withdrawn** in favour of "arithmetic is
never the obstacle; storage, data access, and content-set iteration are." The per-org
decay/TL-accrual/root-bootstrap loop and the trap-condition DSL's boolean grammar remain portable
now or with a D-record once inputs unblock, as claimed and as
`doctrine_liquidation_absorbing.bsl` already proves.

**INADEQUATE-COVERAGE NOTE.** A re-read must add a **RESERVED-LINE section** naming, at minimum:
the 14-node tree and its trunk taxonomy; the two traps and their conditions; the five-stance
reformist fork and its zero-`tag_delta` acquisition design (ADR137/U11); the liquidationism
absorbing state and its three `politics.*_liquidation_*` thresholds; and the standing rule that
transcribing any of it into BSL content is a Director-gated act, not a mechanical transcription.
It should also state explicitly which of the tree's content the eventual pack may author versus
which requires escalation.
