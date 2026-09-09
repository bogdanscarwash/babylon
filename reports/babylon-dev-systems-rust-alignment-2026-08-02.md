# Babylon (dev branch) — Every System Walked Through, and How Rust Incorporates Them Under the Postgres-as-Server / TUI-as-Client Model

**Repo:** `percy-raskova/babylon` · **Branch:** `dev` @ `9020cc3` (protected) · **Surveyed:** 2026-08-02
**Method:** five parallel read-only sweeps over the dev branch (engine systems, engine infrastructure, persistence/Postgres, game-runtime/projection/TUI/web, Rust estate + rulings), cross-checked against `ai/state.yaml`, `NORTH_STAR.md`, `CLAUDE.md`, `README.md`, and the ADR corpus.

---

## 0. TL;DR

- The dev branch runs **34 engine Systems** (README's "26" and NORTH_STAR's "30" are stale), in three partitions: 15 Material Base → 1 Action (OODA) → 18 Consequences, sorted at import by each System's `position` ClassVar.
- **Amendment AE (v3.0.0, ADR172) already rules your question:** Rust **is** the engine language; v1.0 *is* the Rust engine's release; Python survives as data-build pipeline, out-of-process AI observer, and CLI/persistence glue (ADR174). Program 27 is the execution vehicle: Phase 0 (freeze prep) and Phase 1 (Rust kernel + BSL language) are **complete**; Phase 2 (Content & Intrinsics — first Rust/BSL-native mechanic, Lane A heat) is underway; Phase 3 is `babylon-engine` (the tick loop itself).
- Your "Postgres as server, TUI as client" model is **already the ratified shape** — with three refinements you need to know:
  1. **The TUI never reads Postgres directly.** It reads (a) the baked markdown vault (the Archive) and (b) frozen JSON view-models served host-side over a 25-method FFI seam. Postgres owns the *record*; the **projection lane** owns the *feed*. ADR183 R3 makes this structural: in Rust, an *ungated* read must be **unexpressible** (the projector takes the reach/vision ledger as a required argument).
  2. **The tick hash chain makes Postgres the verification authority**, not the compute medium. Every tick is ONE envelope transaction (`persist_tick_atomic`) carrying topology snapshots + `tick_commit` marker (ADR176 r28 torn-tick fix). Resume = tick_commit + nearest checkpoint + deterministic replay (r31: SQL reconstruction is never load-bearing).
  3. **Topology persistence is chartered but unbuilt:** ADR179 T4 (binding) — topology persists as a *dedicated Postgres object*, ideally Apache-AGE-queryable (AGE 1.6.0/PG17 verified compatible; AGE+PostGIS coexistence unverified). Today topology rides per-tick relational snapshots.
- The Rust estate is real and ahead of the docs: 7 crates (`babylon-kernel`, `babylon-graph`, `babylon-bsl`, `babylon-tick`, `babylon-md`, `babylon-tui`, `babylon-tui-python`), with the Lane-A heat modules (`induced`, `exposure`, `dossier`, `capacity`, `backfire`) already landed as the template for "Rust/BSL-native mechanic."
- **Do not port naively.** ADR183 ("get it right in Rust") rules the frozen Python is the oracle for **structure and ordering only** — never for values from never-fed adapters (the 100,000 employment literal) or never-called gates (the fog/veil layer that never runs on the shipping path). Defects are repaired **at the port**, never in the frozen lane.

---

## 1. The architecture as it actually is on dev

```
                        ┌─────────────────────────── SERVER SIDE ───────────────────────────┐
                        │                                                                   │
 VERBS (9, ratified)    │   game_turn queue ──► OODASystem (@14) folds next tick            │
 ────────────────►      │                                                                   │
                        │   SimulationEngine.run_tick ── 34 Systems in position order       │
                        │   (in-memory BabylonGraph; Rust future: GraphSubstrate trait)     │
                        │         │                                                         │
                        │         ▼ ONE envelope transaction per tick                       │
                        │   PostgreSQL 16/17 (tuned, docker :5433)                          │
                        │   ├─ dynamic_* tier (hex/consciousness/demographics/employment/   │
                        │   │   relationships/external nodes; delta + 52-tick checkpoints)  │
                        │   ├─ topology snapshots (node_state/edge_state/graph_metadata/    │
                        │   │   tick_log/simulation_event)   [T4: dedicated object, AGE?]   │
                        │   ├─ journal tier (territory/org/class/edge/community snapshots,  │
                        │   │   hex_activity, tick_summary, tick_event, economic_summary)   │
                        │   ├─ boundary_flow_register + conservation_audit_log (22 invars)  │
                        │   ├─ tick_commit (append-only, replay_identity_hash, checkpoints) │
                        │   ├─ immutable_reference_* (10 session copies of reference series)│
                        │   ├─ declared views (v_hex_state_asof, v_*_value_aggregate,       │
                        │   │   v_national_trend, view_runtime_trace_emission, …)           │
                        │   ├─ document_chunk (pgvector, AI reads; babylon_intel role)      │
                        │   └─ babylon_meta.* (campaign/watchlist/nav — client-owned tier)  │
                        │         │                                                         │
                        │   SQLite reference DB (read-only build product, 75 tables, 60M    │
                        │   rows, sha-pinned; opened mode=ro at session init only)          │
                        │         │                                                         │
                        │   PROJECTION LANE (read-only over committed state; Amendment S)   │
                        │   dirty-entity baker ──► markdown vault (dulwich, sim-time        │
                        │   commits) + live view projectors (economy/field/map/trend/…)     │
                        │   fog + veil gating applied AT the projector                      │
                        └───────────────┬───────────────────────────────────────────────────┘
                                        │ vault pages + frozen JSON view-models (Host seam)
                                        │ verb submissions back through game_turn
                        ┌───────────────▼───────────────────────────────────────────────────┐
                        │ CLIENT SIDE (disposable)                                          │
                        │  babylon-tui (Rust/Ratatui) — THE client since M7 (2026-07-28)    │
                        │  reads: wiki/map/dashboard/topology/chronicle/hud/verb-plate      │
                        │  future clients behind same contract: HTML reskin, neovim         │
                        └───────────────────────────────────────────────────────────────────┘

   SIDE PROCESS (never on tick path): NarratorSideProcess → providers (llama.cpp/Ollama/CF/mute)
   → grounding filter (invented noun/number = REJECTED) → {narrative}/{absence} vault fences.
   AI observes and narrates; it never adjudicates (Amendment V/Y).
```

**Where your model lands:** Postgres *is* the server — the system of record, the hash-chain authority, the read-model host, the retention/archive owner. The TUI *is* the client — disposable, render-only, verb-issuing. The one correction to the naive picture: between them sits the **projection contract** (Amendment V's `observe()`), which is where epistemic gating (fog/veil) is *structurally* enforced. A client that reads Postgres directly would bypass the gate; the ratified design forbids it by construction.

**Determinism spine (why the server can be the authority):** same seed + same defines + same rules ⇒ same bytes. Three honest hashes (ADR179 T2): `replay_identity_hash` (lineage, sha256(session:tick:seed)), `hex_frame_hash` (content over the hex frame), and the P27 `content_hash` (full tick content, `babylon.kernel.tick_hash` Python-side / `state_hash.rs` Rust-side). Constitution III.7 points at the content hash.

---

## 2. The Postgres server estate

### 2.1 What the server owns (condensed inventory; full 60+ table map in appendix of the persistence dossier)

| Tier | Objects | Cadence |
|---|---|---|
| **Game management** | `game_session`, `game_turn`, `action_result` | init / per tick (interactive) |
| **Topology snapshots** | `node_state`, `edge_state`, `graph_metadata`, `tick_log`, `simulation_event` | per tick, inside the envelope TX |
| **Dynamic tier (spec-062)** | `dynamic_hex_state` (c/v/s/k per res-7 hex; delta rows, full frame every 52 ticks), `dynamic_external_node_state`, `dynamic_consciousness_state`, `dynamic_demographics_state`, `dynamic_employment_state`, `dynamic_relationship_state` | per tick (delta) |
| **Journal tier** | `territory_snapshot` (~3,100 rows/tick), `org_snapshot`, `class_snapshot`, `edge_snapshot`, `community_snapshot`, `hex_activity` (~5K sparse), `tick_summary`, `tick_event`, `economic_summary`, `hex_latest` (R7 UPSERT cache), `hex_substrate` (R8 once) | per tick / once |
| **Boundary + audit** | `boundary_flow_register` (DRAIN/TRADE/COMMUTE/FISCAL flows), `conservation_audit_log` (append-only, 22 invariants, ok/warn/alarm) | per tick |
| **Commit chain** | `tick_commit` (append-only, partitioned, PK session+tick; marker rides the envelope TX) | per tick |
| **Reference copies** | 10× `immutable_reference_*` (BEA I-O, MELT τ, γ, ERDI, Hickel, Ricci, FAF, QCEW, REIS rent, FRED) + `immutable_reference_lodes_od_matrix`, `immutable_reference_tiger_county` | once per session |
| **Read interfaces (II.11)** | `v_hex_state_asof`, `v_county/state/national_value_aggregate`, `v_global_phi_balance`, `view_runtime_trace_emission`, `v_national_trend`, 5 composition views over `hex_latest` | computed on read |
| **Semantic** | `document_chunk` (pgvector 768-dim HNSW; `babylon_intel` least-privilege role: SELECT views + INSERT chunks) | append (AI lane) |
| **Client-owned epistemic tier** | `babylon_meta.campaign` (rng_seed + content_digest = replay identity → "rebuild save"), watchlist/jumplist/breadcrumb | client writes only |

### 2.2 Server disciplines that bind any Rust engine

- **35 migrations (0010–0044)**, digest-stamped, advisory-locked exactly-once apply (`ensure_ddl_applied`, lock `0xBAB10537`). The headless runner is the schema author; web is a second idempotent applier.
- **7 PG DOMAINs** (0039, ADR138): probability/currency/ratio/labor_hours/fips5/fips2/h3index — **ENFORCE, never compute**. Language-agnostic, byte-checkable: this is precisely the contract a Rust writer must satisfy (and why `h3_index` becomes BIGINT in the Rust schema, r30).
- **9 LIST(session_id) partitioned families** + `tick_commit` → O(1) purge; **1-live-session retention enforced in code** (r32) with fail-closed purge gate and disk preflight; archives = local zstd parquet + manifest, DuckDB-readable.
- **`synchronous_commit=off` for the player role only** — safe *because* crash-resume replays deterministically. The server's durability posture is a consequence of the determinism doctrine, not an accident.
- **Delta persistence** (spec-089): changed-rows-only between 52-tick checkpoints (~98% duplicates eliminated; ~582 MB/session vs ~177 GB untiered). `MAX(dynamic_hex_state.tick)` is **not** the commit test — `tick_commit` is.
- **T4 (binding, unbuilt):** dedicated topology object in Postgres, AGE-queryable aspirational. Today: per-tick relational snapshots — the "two runtimes lying past each other" gap T4 exists to close.

### 2.3 What the server is *not*

- **Not the compute medium.** No tick math in SQL; stored procedures would contradict ADR174 (hot-path calc → Rust) and the determinism doctrine (one computer, seedable, hash-reproducible). Declared **views** are the sanctioned SQL surface (read models); **AGE** is the sanctioned *query* expansion (topology), not a compute host.
- **Not the reference-data authority at tick time.** The 75-table SQLite build product is opened `mode=ro` at session init, copied into `immutable_reference_*`, and closed; per-tick reference reads hit Postgres copies (`ImmutableReferenceLookup`, year-scoped SLOWLY_VARYING/EVENT_DISCRETE policies).

---

## 3. The 34 Systems — walkthrough + Rust incorporation

Source of truth: `simulation_engine.py::_SYSTEM_CLASSES`, import-sorted by `position` ClassVar (ADR081 declarative ordering; duplicate positions raise at import). Legend: **Status** = live / gated-dormant (early-return on absent inputs) / shadow (write-only, no consumers) / default-off. **Rust target** = where the mechanic lands under Program 27.

### 3.1 Material Base (positions 1.0–13.0) — the world produces its conditions

**1. VitalitySystem @1.0** (`systems/vitality.py`) — subsistence drain, coverage-ratio mortality ("Grinding Attrition"), extinction ("The Reaper"). Writes `wealth/population/active` on social_class. Events: POPULATION_ATTRITION, ENTITY_DEATH. Status: live.
*Postgres:* class rows ride `class_snapshot`/`node_state`. *Client:* class dossiers, chronicle. *Rust:* port as an early `babylon-engine` system; mortality math is a BSL-rule candidate (measured rates via `formulas.vitality`); state = graph node attrs → envelope. Conformance: structure from frozen lane; coefficients from GameDefines.

**2. TerritorySystem @2.0** (`systems/territory.py`) — heat decay/gain, eviction pipeline (heat→rent spike→displacement), heat spillover over ADJACENCY (now symmetric, #417), necropolitics (camp decay). Writes territory `heat/under_eviction/rent_level/population`. Status: live.
*Postgres:* `territory_snapshot`. *Client:* map heat lens, county pages. *Rust:* engine port; ADJACENCY from the content-hashed TIGER artifact (T1) — in Rust this is a CSR compiled at startup (the spatial LOOKUP-ESTATE directive: invariant substrate → static tables + indices, never per-tick state).

**3. SubstrateSystem @2.5** (`systems/substrate.py`) — county raw-material stock depletion/regeneration (ΔB = R − E·η); first ScaleAdjunction consumer publishing CZ/MSA/state/nation aggregates to `persistent_data`. Status: live but inert on county-free graphs.
*Postgres:* substrate stocks in `dynamic_hex_state`. *Rust:* engine port + the scale lattice (Amendment U) becomes trait-level allocate/aggregate adjunctions; aggregates are derived-never-stored in Rust (publish to read models at envelope time).

**4. ProductionSystem @3.0** (`systems/production.py`) — value creation `labor × biocapacity`; labor-aristocracy routing; sets `extraction_intensity`; stashes `la_production`. Invariant: NonNegativeWealth. Status: live.
*Rust:* engine port; Currency i128 (kernel) for value flows; tensor registry (ValueTensor4x3) needs a Rust home — candidate for a domain crate with conformance to the reference-DB hydration fixtures (not to never-fed adapter outputs).

**5. TickDynamicsSystem @4.0** (`domain/economics/tick/system/`) — the annual county economic pipeline: MELT params, county c/v/s, precarity, Vol I wage pressure, accumulation loop, Leontief imperial rent, Vol II circulation, crisis triggers, Vol III endogenous interest, class transitions; between year boundaries accrues `flow_phi_accrued/flow_wage_accrued`. Events: CRISIS_PHASE_TRANSITION, ECONOMIC_CRISIS, DISPOSSESSION_CASCADE, BIFURCATION_THRESHOLD. Status: live (year-boundary + wiring gated).
*Postgres:* `tick_summary`, `dynamic_*` county rows, `territory_snapshot.tick_*`. *Client:* dashboard economy, trend view (`v_national_trend`). *Rust:* the heaviest port — decompose into per-concern BSL rule packs + kernel intrinsics; **do-not-transcribe**: the 100,000 employment literal (never-fed `employment_source` on the canonical path — §5.4 row 2), housing/dispossession/Z1 never-fed adapters. phi_hour tick-52 regression is a Phase-0 finding — repair at the port (allocator-clamp fix authorized under Director gate).

**6. ReserveArmySystem @5.0** (`systems/reserve_army.py`) — reserve ratio → wage pressure on `median_wage`; border-regime overlay valve. Events: RESERVE_ARMY_PRESSURE. Status: live (input-gated).
*Rust:* BSL-rule candidate (measured pressure from stocks); producer = TickDynamics accumulation loop.

**7. CommunitySystem @6.0** (`systems/community.py`) — community hypergraph layer (XGI): decay, ternary consciousness from member orgs, solidarity amplification via overlap. Status: **gated-dormant** (no wired community hypergraph anywhere).
*Rust:* **waits for the native-hyperedge milestone** (ADR180 R2b verbatim) — this is *the* case for Amendment D hyperedges; do not port the XGI shape; re-express over `GraphSubstrate.add_hyperedge`.

**8. LifecycleSystem @7.0** (`systems/lifecycle.py`) — D–P–D′ demographic cohorts, legitimation index/crisis, inheritance, ideology transmission, class mobility. Events: LIFECYCLE_TRANSITION, LEGITIMATION_CRISIS/RECOVERY, INHERITANCE_TRANSFER. Status: live.
*Rust:* engine port; cohort ODEs as BSL rules with kinded dispersion fields on coarse carriers (r17: moments/quantile sketch, Jensen guard).

**9. SolidaritySystem @8.0** (`systems/solidarity.py`) — consciousness transmission along SOLIDARITY edges (the anti-bribery infrastructure); MASS_AWAKENING threshold. Status: live.
*Rust:* engine port; transmission is a **measure** over edge strength, no imposed curve (ADR172 r5).

**10. ImperialRentSystem @9.0** (`systems/economic.py`) — the five-phase imperial circuit: extraction (TRPF pool decay), tribute, Amin super-wage, CLIENT_STATE "Iron Lung" subsidy, bourgeoisie wage-vs-repression decision; spec-063 seams 5b (Φ→counties) / 5c (Vol II circulation) context-key gated. Writes `wealth`, `w_paid/v_produced`, edge `value_flow`, boundary-register DRAIN rows. Events: SURPLUS_EXTRACTION, SUPERWAGE_CRISIS, IMPERIAL_SUBSIDY, ECONOMIC_CRISIS. Status: live; 5b/5c gated.
*Postgres:* `boundary_flow_register`, `dynamic_external_node_state`, Φ conservation invariants (`imperial_rent_phi_week_distribution`). *Client:* dashboard Φ line, trade dossiers. *Rust:* flagship engine port; Φ accounting in Currency i128; boundary register stays a server-side append-only table; the sigma-composition attribution (P26) ports as pure domain math fed by reference series.

**11. TransportSystem @9.5** (`systems/transport.py`) — corridor-mesh decay, demand signal, connectivity, Vol II overhang damping. Status: **DEFAULT-OFF** (`TransportDefines.enabled`).
*Rust:* do not port until enabled; when it lands, it lands Rust/BSL-native (like Lane A) — corridor mesh as reference-artifact-fed static tables.

**12. DispossessionEventSystem @10.0** (`systems/dispossession_events.py`) — composite dispossession intensity (foreclosure/eviction/displacement weights) + wealth transfer. Events: VALUE_TRANSFER, DISPOSSESSION_EVENT. Status: live (input-gated).
*Rust:* BSL rule over territory attrs; producers (accumulation loop, MarketScissors) port first.

**13. DecompositionSystem @11.0** (`systems/decomposition.py`) — on SUPERWAGE_CRISIS the labor aristocracy splits one-time into CARCERAL_ENFORCER (30%) + INTERNAL_PROLETARIAT (70%). Events: CLASS_DECOMPOSITION. Status: live (event-gated, one-time).
*Rust:* engine port; node creation via structural verbs (`add-node` with range-checked field-inits); the one-time flag becomes rule content.

**14. ControlRatioSystem @12.0** (`systems/control_ratio.py`) — guard:prisoner ratio vs capacity → CONTROL_RATIO_CRISIS → TERMINAL_DECISION (revolution if organization ≥ threshold, else genocide). Status: live (dormant until decomposition).
*Rust:* engine port; terminal outcomes become measure-derived, not threshold-stipulated where the rulings require emergence.

**15. MetabolismSystem @13.0** (`systems/metabolism.py`) — ΔB = R − E·η biocapacity update, hysteresis ceiling ratchet, overshoot ratio; applies sovereign metabolic_impact (prior-tick). Events: ECOLOGICAL_OVERSHOOT. Status: live.
*Postgres:* biocapacity in `dynamic_hex_state`; overshoot on `tick_summary`. *Client:* map habitability (county-inherited), matter-book on economy dashboard. *Rust:* engine port; pure measure math, BSL-friendly.

### 3.2 Action Phase (position 14.0) — organizations observe and act

**16. OODASystem @14.0** (`systems/ooda.py` + `babylon/ooda/*` + `engine/actions/*`) — three-layer turn resolution: Layer 0 business metabolism; initiative-ordered org actions (player verbs via the 13-module resolver estate — 9 canonical verbs registered; NPC via deterministic CPU policy + `RuleBasedStateAI` with 6 verbs / ~24 sub-verbs incl. Sparrow topological targeting); Layer 3 consequence propagation. Events: ORGANIZATIONAL_ACTION, POGROM, LOCKOUT, VIGILANTISM, STATE_REPRESSION, STATE_SURVEILLANCE. Status: live.
*Postgres:* reads `game_turn` queue (via `build_player_actions`); results persisted server-side. *Client:* verb plate (F1–F9), action previews, chronicle. *Rust:* the verb algebra **already exists in Rust** — BSL §2.8 typed structural verbs + the ratified 3×3 matrix is frozen Python data (`game/actions/matrix.py`, 9-pin sentinel). Port path: verb resolvers → BSL rule packs + `EffectExecutor` over `GraphSubstrate`; the write log (ADR182 R1) attributes every mutation to its rule, replay-complete; NPC policy = deterministic content, in-hash. Repression estate: Lane A modules (dossier/capacity/exposure/backfire) are the Rust-native shape — repression priced by `Capacity` (Currency budgets, ADR184), targeted by exposure (Sparrow Δφ/|class|), never a stored heat score.

### 3.3 Consequences (positions 14.5–22.0) — the world answers

**17. FactionInfluenceSystem @14.5** (`systems/faction_influence.py`) — per-territory winning faction from INFLUENCES+ADJACENCY, transitions, red-settler trap, secession hysteresis. Events: TERRITORY_TRANSITION, FACTION_VICTORY, RED_SETTLER_TRAP_DETECTED, SECESSION_DECLARED. Status: live (no-op without faction topology).
*Rust:* engine port over hypergraph adjacency; RNG via `KernelRng::for_carrier` (r20 — one stream per (session, tick, domain, stable_key)).

**18. DoctrineSystem @14.7** (`systems/doctrine.py` + `domain/doctrine/`) — the org doctrine tree: tag decay, theoretical-labor accrual, traps (`@coeff` DSL), Party Congress (purge, line-split), officeholder capture, practice→tag drift (PracticeVariable disjoint from DoctrineTag), mass-work SOLIDARITY decay. Events: DOCTRINE_TRAP_SPRUNG/ESCAPED, DOCTRINE_PURGE_FAILED, LINE_STRUGGLE_SPLIT. Status: live (no-op on org-less graphs).
*Rust:* **the doctrine corpus is the proven BSL content shape** — the conformance corpus already transcribes the doctrine trap rules (12 `.bsl` files incl. `doctrine_adventurism`, `doctrine_liquidationism`, `doctrine_liquidation_absorbing`; the `:default` allowlist's 6 governed rows are doctrine trap-DSL sites). Port as BSL content + host intrinsics; congress cadence via sim clock.

**19. SurvivalSystem @15.0** (`systems/survival.py`) — P(S|A) sigmoid on per-capita wealth vs subsistence (net of policy_delivery); P(S|R) = organization/repression × solidarity. Writes `p_acquiescence/p_revolution`. Status: live.
*Postgres:* `dynamic_consciousness_state` (county p/p + r/l/f). *Rust:* **the doctrine changes here** — ADR173: the logistic form is the *frozen reference's*, not the going-forward law. In Rust, P(S|A) = the **measure** of class members whose wealth clears subsistence (kinded dispersion fields, r17); the S-curve *emerges*. No sigmoid intrinsic exists to misuse (r21: {exp, log} at most; sigmoid/tanh/entropy never registered).

**20. StruggleSystem @16.0** (`systems/struggle.py`) — George Floyd Dynamic: stochastic sparks, uprisings, power-vacuum bifurcation (revolutionary offensive vs fascist revanchism), peripheral revolt, spontaneous riot. 8 event types. Status: live.
*Rust:* engine port; stochasticity strictly through `KernelRng`; event emission via `emit` → `EventSink` → kernel bus (Phase 3 wiring).

**21. ConsciousnessSystem @17.0** (`systems/ideology.py`) — agitation from wage/wealth deltas, routed revolutionary vs fascist via ternary router; solidarity/chauvinist pressure, repression, popular-front suppression, working-day visibility. Writes `ideology` + `material_conditions`. Status: live.
*Postgres:* `dynamic_consciousness_state` r/l/f. *Client:* class cards, map tension lens inputs. *Rust:* engine port; routing math as BSL-expressible measures over the opposition registry; org-sourced SOLIDARITY read path (ADR087) ports with it.

**22. FascistFactionSystem @17.4** (`systems/reactionary.py`) — fascist pull on entitled strata, recruitment capture, MEMBERSHIP chauvinism, crisis defection, StanceIntervention on capital_labor. Events: FASCIST_DRIFT/RECRUITMENT, ORGANIZATIONAL_FRACTURE, RED_BROWN_COUP. Status: live (dormant absent entitled roles).
*Rust:* engine port; intervention writes consume-once graph attrs (port the consume-once discipline exactly).

**23. AllegianceSystem @17.42** (`systems/allegiance.py`) — the electoral valve: allegiance drift, hope field H(c), agitation→organization valve, `political_labor_share` producer. Events: HOPE_SPIKE. Status: live, fully input-gated (zero writes without PoliticalFaction orgs).
*Rust:* engine port; also the **owner of co-optation composition** (ADR180 R3: co-optation is one implementation owned by the electoral machinery, priced/triggered by the heat system).

**24. ElectoralSystem @17.45** (`systems/electoral.py`) — the clocked ambient machine: per-sovereign cycle clocks, FPTP + spoiler + recount, bonapartist suspension, government formation, legitimation refresh, institution balance, disillusion windows, popular-front conjuncture, derecognition, governance endgames. 10 event types. Status: live, input-gated.
*Postgres:* `graph_metadata` electoral registers. *Rust:* engine port; clocks from `SimClock` (pure (session, tick) — no uuid4); seeded rolls via r20 streams.

**25. PolicySystem @17.47** (`systems/policy.py`) — legislative executor: federal preemption, judicial strike, funding identity (L-CEILING), capital strike, host-discipline clamp, delivery ledger + betrayal integral, SYRIZA fork, L-RECEIPTS boundary rows. 7 event types. Status: **gated-dormant** in canonical scenarios (needs agenda/fiscal registers).
*Postgres:* `boundary_flow_register` FISCAL_FUNDING/SOCIAL_WAGE rows (0040 vocabulary). *Rust:* engine port; pure pipelines (`domain/politics`) port as domain math; receipts vocabulary is already server-blessed.

**26. SovereigntySystem @17.5** (`systems/sovereignty.py`) — effective controller + metabolic_impact per territory from claims & extraction policy. Events: DUAL_POWER_ACTIVE. Status: live (dormant without sovereigns).
*Rust:* engine port; the one-tick-delayed handoff to Metabolism (via `persistent_data`) becomes explicit tick-staged reads in the Rust tick contract.

**27. MarketScissorsSystem @17.8** (`systems/market_scissors.py`) — price⟷value + fictitious oscillators; correction feedback (snap, wealth evaporation brackets 0/1, reserve-ratio swell, shock stamp); per-county divergence observe-only. Events: MARKET_CORRECTION. Status: live (Phase 2 on).
*Postgres:* `tick_summary` price_log/fictitious_log/market_corrections (0033/0034) → `v_national_trend`. *Client:* dashboard scissors charts. *Rust:* engine port; oscillators in binary64 lane with exact-equality rules; monetary anchors (FRED-derived) via reference series; honest absence post-2024.

**28. ContradictionSystem @18.0** (`systems/contradiction.py` + `domain/dialectics/`) — the Lawverian core: opposition registry (gap/rate/principal), frames, rupture, regime classification, LEVEL_TRANSITION, pole-channel shadow, Fundamental Theorem stash. Status: live; pole σ attrs shadow.
*Postgres:* `tick_summary`/contradiction rows via envelope. *Rust:* the ⊗/⊕ combinators (`domain/dialectics/core/composition.py`) carry forward **with priority** (ADR183 R4) — they are the closed algebra BSL expresses. This is the system where "BSL expresses, engine executes" pays off most directly; registry/coupling port as Rust domain code, measures as BSL content.

**29. ContradictionFieldSystem @19.0** (`systems/contradiction_field.py`) — per-class fields: exploitation (mean fresh edge tension), atomization (opposition gap), 3-tick history. Status: live.
*Postgres:* `contradiction_field` table. *Client:* field-state view (the Weather Layer). *Rust:* engine port; O(N+M) single-pass (the 2026-07-06 optimization) preserved.

**30. FieldDerivativeSystem @20.0** (`systems/field_derivative.py`) — spatial gradients on edges, Laplacian + df/dt + d²f/dt², principal-field identification. Events: PRINCIPAL_CONTRADICTION_SHIFT. Status: live.
*Rust:* engine port; pure graph math — early candidate for Rust perf win; floats by `to_bits` in `state_hash.rs` (NaN refused loudly).

**31. CollapseTransitionSystem @20.5** (`systems/collapse_transition.py`) — sovereign collapse → partition claims among winning factions; secession execution; orphan cleanup. Events: SOVEREIGN_COLLAPSE, TERRITORY_TRANSITION, CIVIL_WAR_DECLARED. Status: live (dormant without sovereigns).
*Rust:* engine port; node removal **cascades observably** (ADR185 R2: incident edges removed, memberships dropped, one write-log record per cascaded item).

**32. EdgeTransitionSystem @21.0** (`systems/edge_transition/`) — compound-predicate edge-mode state machine (solidarity↔rivalry flips). Status: **largely inert** (predicates reference field names ContradictionFieldSystem never writes).
*Rust:* **do not port the defect** — this is a §5.4-class find: the port re-charters the predicate vocabulary against the real field producers (I.15 edge-mode machine is *not* enforced by the `GraphSubstrate` trait — it lands as engine-level content with a vocabulary ceremony, cf. r34).

**33. WealthDistributionSystem @21.5** (`systems/wealth_distribution.py`) — national 4-bracket wealth-share ODE axis; consumes the market-correction shock. Status: **shadow** (Phase 1 write-only).
*Postgres:* `graph_metadata` wealth_distribution. *Rust:* port when Phase 2 (owner-gated) activates consumers; ODE in binary64 lane.

**34. EpistemicHorizonSystem @22.0** (`systems/epistemic_horizon.py`) — fog-of-war shadow: mass receptivity M_r, intel confidence, vision_state per territory (desert/mud/water); runs last. Status: **shadow**.
*Client:* map fog lens (county_fog_status reads vision tiers), INVESTIGATE intel. *Rust:* **this is where ADR183 R3 / ADR182 R2 bite hardest** — "structure public, magnitudes earned." In Rust the fog isn't a filter you remember to call; the projector's signature demands the reach/vision ledger. Port the *measures*; retire the never-called gate layer (§5.4 row 1) by making it unexpressible.

### 3.4 Status ledger (what actually fires)

- **Default-off:** TransportSystem @9.5. **Shadow:** WealthDistribution @21.5, EpistemicHorizon @22.0, pole-channel σ attrs. **Gated-dormant:** Community @6.0, Policy @17.47, Allegiance/Electoral/Doctrine (need faction/org nodes), ImperialRent 5b/5c, Sovereignty/Collapse/FactionInfluence (need sovereign topology), Dispossession/ReserveArmy (zero-rate inputs), Decomposition/ControlRatio (crisis-gated). **Partially inert:** EdgeTransition @21.0 (predicate vocabulary), Substrate @2.5 (county-free canonical scenarios). Only **12 of 34** fire on the canonical byte-identical scenarios (ADR090) — every port should name which gate class it sits in, because conformance vectors from a dormant system certify nothing.

---

## 4. Engine infrastructure — walkthrough + Rust incorporation

| Unit | What it is (dev) | Postgres touchpoints | Rust incorporation |
|---|---|---|---|
| `simulation_engine.py` (envelope) | Pure step: to_graph → ServiceContainer → run_tick (34 systems, timed) → save context → bus→pydantic events → from_graph(tick+1); optional ConservationAuditor end-of-tick | none directly (audit rows reach PG via bridge) | Phase 3 `babylon-engine`: system registry + anchor total-order (BSL `mod_anchors` deferred half) + tick sequence. The envelope's save/restore graph-context dance disappears — Rust holds state, server holds record |
| `services.py` (ServiceContainer) | ~45-slot DI: config/database/bus/formulas/defines/metrics + Vol I–III calculators, data sources, boundary register, auditor | database slot (protocol); PG-backed bus/register/auditor in headless | Rust: compile-time DI (traits + generics). Data sources (FRED/QCEW/BEA) stay **Python glue** (ADR174) behind a host-intrinsic seam: Rust asks, Python serves, values enter as declared BSL bindings (`:field`/`:const`/`:metric`/`:tick`) |
| `kernel/event_bus.py` | Frozen Event; interceptor chain (priority desc, stable; ALLOW/BLOCK/MODIFY; blocked→audit channel with ORIGINAL event); append-before-fanout; handler isolation (ExceptionGroup analog) | EventCapture drains to summary.json in headless | **Done** — `babylon-kernel::event_bus` ports all four ordering guarantees as tests. Remaining: 100-value EventType enum + payload/timestamp (Phase 2/3, babylon-domain) |
| `event_builders.py` / `event_evaluator.py` | ~80 of 100 EventTypes have bus→pydantic builders; EventTemplate evaluator (preconditions → resolutions, cooldowns) | `simulation_event` (natural-key dedup, 0043) | Event vocabulary re-mints as **BSL content with a ceremony** (r34 precedent: five dead edge types retire and re-mint in BSL, not as copied Rust enums) |
| `formula_registry.py` | 24 hot-swappable formulas by name | none | Splits three ways: kernel intrinsics (pinned, golden-vectored — r21), BSL content (rules hash covers them), or domain-crate functions. The registry's hot-swappability *is* BSL (rules are content, `rules_hash` mandatory on ContentDigest) |
| `topology_monitor.py` | Percolation condensation tracker (gaseous→solid, cadre density), observer-pending events | none | ADR183: inert construct, not a defect — port as an observer over `GraphSubstrate`; Π₀ components via dialectics `pieces()` |
| `bifurcation_monitor.py` | George Jackson bifurcation tendency (fascist vs revolutionary routing) | none | Same — observer; XGI dependence waits for hyperedge milestone |
| `trap_detection.py` | Liberal/ultra-left/rightist trap scores | legacy web only | **Declared superseded** (P25 U11) — do not port |
| `observers/` (SessionRecorder, EndgameDetector, MetricsCollector, CausalChain, JSONL) | Post-freeze observation; EndgameDetector = 5 terminal outcomes as *recognized patterns* (never terminators since spec-116) | none (facade lane) | EndgameDetector ports (Rust, axis evaluators are pure re-derivations of the committed stream; driver's permanent latch). The recorder family is subsumed by `tick_commit` + write log |
| `headless_runner/` | Canonical Postgres-backed runs: migrations → initialize_session (hydrate scope from SQLite) → economics overrides → 520-tick loop (replay hash, atomic persist, dense trace) → artifacts (trace/summary/manifest) | everything | The *shape* survives cutover: Rust engine behind the same envelope contract; hydration stays Python glue (I/O); artifacts keep the manifest schema. `sim:e2e-bg` becomes the Rust engine's conformance harness against the 11-scenario golden estate |
| `invariants.py` + `conservation_audit.py` | INV-001/002/006/008/009/010 + 22 conservation invariants (ε-graded ok/warn/alarm), hex_frame_hash | `conservation_audit_log` | Conservation audit ports as an engine-side post-tick pass writing the same table; invariants become Rust type-level + runtime checks (overflow-checks=true in release is the III.11 precedent) |

---

## 5. The math estate — formulas + domain

**23 formula modules** (`formulas/`, pure functions, ~24 registered): balkanization, class_dynamics, community, consciousness(+routing), contradiction, curvature, dynamic_balance, fundamental_theorem, lifecycle, market, metabolic_rift, politics, reactionary, solidarity, state_ai, survival_calculus, sustained_exploitation, trpf, unequal_exchange, vitality. **Rust split:** (a) *measures* that BSL rules express directly (most of solidarity/consciousness/politics); (b) *kernel intrinsics* (numeric annex §6 — only what r21 permits, {exp, log} at most, soft-float libm pinned + golden vectors); (c) *domain-crate math* (tensor/Leontief/sigma/circulation — heavy linear algebra, np.linalg.inv's one live site noted in the pre-freeze traces).

**Domain packages** (`domain/`): economics (tensor, sigma/, working_day/, reserve_army/, monetary/, circulation/, distribution/, melt/gamma/throughput, trade), dialectics (core categorical machinery + instances), organizations, institution, bifurcation, geography, politics, doctrine. **Rust split:** the categorical core (oppositions, adjunctions, ⊗/⊕) ports with priority (ADR183 R4); the economics calculators port as domain crates consuming reference series through host bindings; the SQLite/FRED *adapters* stay Python glue.

**Config:** GameDefines (~50 category models across 29 modules; canonical `defines.yaml`; `canonical_defines_hash` = SHA-256 over canonical JSON). **Rust:** defines parse stays Python (YAML glue); the *hash* is already conformance-pinned cross-language (25,184-byte fixture → `4af11780…`). BSL content adds `rules_hash`; `ContentDigest{defines_hash, rules_hash}` is the campaign identity on `babylon_meta.campaign`.

---

## 6. The game runtime around the engine

- **`game/session.py` (the composition root, ~104KB):** the ONE place engine + persistence + projection meet. `create_new_campaign` / `resume_campaign` / `advance_tick` (queue fold → run_tick → chronicle → tick_summary → boundary flush → `persist_tick_atomic` → vault bake → narration envelope → autopause). Live read surfaces computed fresh per call: dashboard/trend/national snapshot/subject views/endgame/verb plate/choropleth/topology/field-state/read_page. **Rust alignment:** the session is exactly the "seam 15" ADR183 assigns to Rust. Long-term, `babylon-engine` + the T4 topology object + checkpoint-only hydration replace the hydrate_graph resume; the read surfaces become Rust projectors over the committed stream.
- **`game/pacing.py`:** pacing POLICY over one TickAdvancer — strict tick monotonicity, permanent endgame latch, autopause-ack, busy lock, `run_until_paused(max=5200)`. **Rust:** ports as driver policy over the engine; wall-clock `tick_delay` stays outside the hash.
- **`game/actions/matrix.py`:** the ratified Article V 3×3 — Build-org×Population holds BOTH educate+campaign (Iskra double cell); Manage-resources×Organization honestly EMPTY until the funding train. Frozen data under a 9-pin sentinel; moving a verb = Director ruling. **Rust:** transcribe as data (it's already frozen); verb resolvers → BSL rule packs.
- **`game/standing_orders.py`:** a verb persists across ticks (suppress-never-cancel; material interrupts; session-lifetime). **Rust:** engine-side standing-order registry, same semantics.
- **`game/tutorial*.py`:** the tutorial IS the BDD acceptance suite — one `TutorialScript` consumed three ways (player overlay, headless Pilot, docs). Any future client is correct iff the tutorial suite passes (the "rewrite test"). **Rust:** already green against the Rust client through the real engine (17 tests, byte-identity); it's the constitutional parity gate for every later port.
- **`game/trade.py` + `game/vol2.py`:** interactive trade wiring (boundary register, external Φ, county exposure, simulated_year) and the Vol II composer. **Rust:** trade math ports; wiring stays composition-root glue.

---

## 7. The projection lane & the Archive (the server→client feed)

- **`projection/vault/`:** deterministic page baking — Jinja2 sandboxed (StrictUndefined), dulwich commits stamped with **sim time** (identical state ⇒ identical shas), content-hash skip + one commit per tick; `IncrementalArchiveTickBaker` (dirty-entity-driven; full bake = CI correctness baseline); `NarratorCache` (`{narrative}`/`{absence}` fences, III.6 keyed by (entity, tick, model_pin)); `NarrationEnvelope` JSONL (the artifact the Rust engine must eventually emit — specified Python-first so goldens exist pre-port, closes BSL OQ-32).
- **`projection/fog/`:** two-layer epistemic masking (material public / political gated outside organizing reach); `IntelLedger` (exact→approximate→unknown aging); reach = PRESENCE ∪ SOLIDARITY-neighborhood. **Fog is epistemic, never material; never in the tick hash.**
- **`projection/veil.py`:** value-axis gating by doctrine acquisition (tier 0/1/2); TIER1 = value-relation fields, TIER2 = scissors fields. Monotonic by construction.
- **`projection/topology/`:** tension lens (ADR170: w = (φ−θ)/(φ+θ), diverging crimson↔gold), county fog status, hex habitability (county-inherited), map bands as DATA.
- **The projectors** (`economy.py`, `field_state.py`, `faction.py`, `territory_anchor.py`, …): read the live post-tick graph directly (never a round trip); absence projects as None, never a fabricated default.
- **`verbs/`:** preview (preview == resolution), plate (eligibility from real target existence + candidate sets), submit (canonical-verb + affordability gate → `game_turn` row; the ONE write the read side owns).

**Rust alignment:** projectors are "seam 11" — port the *contract*, not the call graph (ADR183 R4); gating becomes a required-argument type property. The vault/baker can stay Python (I/O glue) initially; the TUI's markdown rendering is already Rust (`babylon-md` fork + pulldown-cmark wikilinks).

---

## 8. The TUI itself (client side of the seam)

- **`babylon-tui`** (Rust lib): view stack + event loop; 16 view modules (lobby/wiki/map/dashboard/topology/hud/keybar/chronicle/watchlist/verbs/palette/peek/help/tutorial/msg); KSBC theme (crimson/gold on dark); 100×30 declared floor; insta goldens; `raster` feature (hypergraph-rs rev-pinned `0c95db06…`, cells3d + raster-png, kitty-only StatefulProtocol pixel path). Lib doc states the law: **"Rust owns the terminal event loop; Python remains the single writer and serves frozen view-models as JSON across the Host seam."**
- **`babylon-tui-python`** (cdylib): PyO3 FFI — `run(host, config_json)`; `PyHost` implements the 25-method `Host` trait by delegating to Python `RustClientHost` (`tui/host.py`, 77KB); a raising host PANICS across the FFI (III.11); headless TestBackend scripted replay = the BDD foundation.
- **The 25-method Host seam:** lobby (catalog/new/load), read surface (read_page/known_subjects/backlinks/subject_view/watchlist), playable surface (pacing_state/advance_tick/run_until_paused/acknowledge_pause/chronicle_rail/verb_plate/topology/choropleth/trend/dashboard/field_state/render_config/issue_verb/endgame/pin/nav/tutorial_state).
- **What changes when the engine is Rust:** the *contract* doesn't. Two endgames are compatible with the rulings: (a) **FFI inversion** — the Rust engine becomes the writer and Python's persistence/AI glue is called *by* Rust (PyO3 the other way, or IPC); (b) **Rust-native client-server** — engine + Postgres access in one Rust process (sqlx against the same DDL/DOMAINs/views), TUI links the engine crate directly, Python shrinks to data builds + AI sidecar. ADR174 currently assigns persistence to Python glue, so (a) is the ratified near-term shape; T4's dedicated topology object and checkpoint-only hydration are the stepping stones either way. The TUI as a *client* is insulated from the choice — that's the point of Amendment V's `observe()` contract.

---

## 9. The intelligence layer (observes, never controls)

- **`intelligence/providers.py`:** the ONE transport seam — `NarratorProvider` (narrate/embed/health), OpenAI-compatible; resolution order llama.cpp bundled → Ollama → Cloudflare-if-keyed → **mute is always legal** (R4). No LLM in the input path (no parse() lane — verbs enter only through the deterministic registry).
- **`llama_server.py`:** bundled CPU llama.cpp sidecar, loopback only, sha256-pinned weights.
- **`rag/` + `pgvector_store.py`:** corpus RAG over `document_chunk` (768-dim, HNSW); corpus canon/apocrypha manifest (ADR107).
- **Grounding filter (`projection/narration_grounding.py`):** narration may use only proper nouns/numbers present in the grounded inputs; invention ⇒ REJECTED → visible `{absence}` page naming the offender.
- **`ai/director.py` / `ai/judge.py`:** the legacy observer pair (Gramscian Wire dual perspectives; LLM-as-judge) — production narration is the session→envelope→side-process lane.

**Rust alignment:** none directly — the AI lane is *why* Python survives (ADR174). The contract it consumes is the `NarrationEnvelope` (JSONL); when the Rust engine exists, it must emit byte-identical envelopes (that's why the format was specified Python-first). pgvector stays server-side; `babylon_intel` role keeps the DB-enforced "observes-never-adjudicates."

---

## 10. The sentinel estate (why ports can't silently rot)

26 registered sensors run via `tools/sentinel_check.py` (20 static in the fast gate): seam, seam-algebra (∂L reachability), coverage, vocabulary, inert, unconsumed, dangling, liveness, coupling, surface, synthetic, absence, aggregation(+intensive), fog, masked_arithmetic, gate-coverage(+truth), reachability, fallback-coverage, formula_registration, defines_passthrough, domain_sync, superstructure, tutorial_coverage, partition, assumptions, roundtrip, conservation, determinism. Cross-language pattern already proven: `sentinels/_rust.py` text-parses the Rust keybar (62 rows over floor 40) — the template for Python-side sentinels pinning Rust-source truth during the transition. **Every ported system should land with its sentinel row** (ADR109 wiring doctrine: W-C/W-𝔇/W-G/W-P/W-A4 typed motions, never a bare import-and-call).

---

## 11. Master alignment map — every system → server ownership → client surface → Rust target

| # | System (slot) | Server-side state (Postgres) | Client surface (today) | Rust target | Porting doctrine notes |
|---|---|---|---|---|---|
| 1 | Vitality @1.0 | class_snapshot, node_state | class dossier, chronicle | engine system / BSL rule | structure from frozen lane |
| 2 | Territory @2.0 | territory_snapshot | map heat, county pages | engine system | ADJACENCY as startup CSR (T1 artifact) |
| 3 | Substrate @2.5 | dynamic_hex_state | (aggregates) | engine + adjunction trait | derived-never-stored aggregates |
| 4 | Production @3.0 | territory_snapshot, dynamic_hex_state | economy dashboard | engine + tensor domain crate | Currency i128 |
| 5 | TickDynamics @4.0 | tick_summary, dynamic_* county rows, territory_snapshot | dashboard, trend view | engine + BSL packs + intrinsics | §5.4: 100k employment literal, never-fed adapters; phi_hour fix at port |
| 6 | ReserveArmy @5.0 | territory_snapshot | map wage-pressure lens | BSL rule | producer ports first |
| 7 | Community @6.0 | community_state/membership | topology views (honest absence today) | **waits for hyperedge milestone** (R2b) | re-express over native hyperedges, not XGI |
| 8 | Lifecycle @7.0 | territory_snapshot | county pages | engine + dispersion fields (r17) | cohort measures |
| 9 | Solidarity @8.0 | dynamic_relationship_state | org network | engine | measure over edge strength |
| 10 | ImperialRent @9.0 | boundary_flow_register, dynamic_external_node_state, audit invariants | Φ line, trade dossiers | engine flagship | Currency i128; σ-attribution as domain math |
| 11 | Transport @9.5 | persistent_data only | — | **don't port until enabled** | lands Rust-native when chartered |
| 12 | Dispossession @10.0 | territory_snapshot | map lenses | BSL rule | FRED adapters stay glue |
| 13 | Decomposition @11.0 | node_state | chronicle, class pages | engine | structural verbs; one-time flag as content |
| 14 | ControlRatio @12.0 | node_state | chronicle, endgame axes | engine | emergence over thresholds |
| 15 | Metabolism @13.0 | dynamic_hex_state, tick_summary | habitability lens, matter-book | engine / BSL | pure measure |
| 16 | OODA @14.0 | game_turn→action_result | verb plate, chronicle, previews | **BSL structural verbs + EffectExecutor** (exists) | write log (R1); NPC policy in-hash; Lane A = repression estate |
| 17 | FactionInfluence @14.5 | node_state, balkanization_history | map overlays | engine | KernelRng r20 |
| 18 | Doctrine @14.7 | org_snapshot, graph_metadata | org doctrine canvas | **BSL content (corpus exists)** | 6 allowlist rows; congress via SimClock |
| 19 | Survival @15.0 | dynamic_consciousness_state | class cards, HUD | engine | **P(S|A) = measure, not sigmoid** (ADR173) |
| 20 | Struggle @16.0 | node_state, simulation_event | chronicle | engine | r20 streams; emit→EventSink |
| 21 | Consciousness @17.0 | dynamic_consciousness_state | class cards, tension inputs | engine | ternary router as measures |
| 22 | FascistFaction @17.4 | node_state, graph_metadata | faction dossiers | engine | consume-once attrs discipline |
| 23 | Allegiance @17.42 | graph_metadata | electoral surfaces | engine | owns co-optation composition (R3) |
| 24 | Electoral @17.45 | graph_metadata | electoral surfaces | engine | SimClock; r20 |
| 25 | Policy @17.47 | boundary_flow_register (FISCAL/SOCIAL_WAGE) | policy surfaces | engine (when activated) | dormant in canonical — no vectors from it |
| 26 | Sovereignty @17.5 | persistent_data | map sovereignty | engine | explicit tick-staged handoff |
| 27 | MarketScissors @17.8 | tick_summary (0033/34) → v_national_trend | scissors charts | engine | binary64 exact-equality; FRED anchors honest-absent post-2024 |
| 28 | Contradiction @18.0 | tick_summary/contradiction rows | HUD axes, field views | **engine + dialectics crate (priority)** | ⊗/⊕ carried forward (R4); BSL expresses |
| 29 | ContradictionField @19.0 | contradiction_field | field-state view | engine | keep O(N+M) |
| 30 | FieldDerivative @20.0 | edge_curvature | field-state view | engine (early perf win) | floats to_bits; NaN loud |
| 31 | CollapseTransition @20.5 | node_state, balkanization_history | chronicle, map | engine | cascades observable (ADR185 R2) |
| 32 | EdgeTransition @21.0 | edge_state | — | **re-charter, don't port** | predicate vocabulary ceremony (r34-class) |
| 33 | WealthDistribution @21.5 | graph_metadata | (shadow) | defer to Phase 2 activation | owner-gated |
| 34 | EpistemicHorizon @22.0 | territory attrs (dropped on round-trip) | fog lens | engine measures + **structural gating** | ADR183 R3: ungated read unexpressible |

**Cross-cutting infrastructure:** event bus ✅ (kernel, done) · RNG ✅ (r20, done) · scalars/Currency ✅ (done) · sim clock ✅ (done) · ContentDigest ✅ (done) · state hash ✅ (`state_hash.rs`) · event vocabulary (Phase 2/3, BSL ceremony) · conservation audit (engine-side pass, same table) · ServiceContainer (compile-time DI; data sources via host bindings) · headless runner (shape survives; hydration stays glue) · EndgameDetector (ports; driver latch) · observers (subsumed by tick_commit + write log) · trap_detection (**superseded — never port**).

---

## 12. Sequencing and open questions

**Ratified sequence (already in motion):**
1. **P27 Phase 2 (now):** kernel intrinsic table (r21: {exp, log} at most, soft-float libm + golden vectors); Currency-typed attribute storage + hyperedge field-inits; deffield registry; Lane A heat completes as the first full Rust/BSL-native mechanic (dossier L / capacity K / exposure X / backfire / induced — the template every later mechanic copies); retired-edge re-mint ceremonies (r34).
2. **P27 Phase 3 — `babylon-engine`:** anchor total-order + E-LOAD-003 interleave check; system registry; tick sequence; EventSink → kernel bus; checkpoint-only hydration (r31); T4 dedicated Postgres topology object (AGE feasibility: AGE+PostGIS coexistence, digest-pinned CI image, upgrade path — all unverified).
3. **System ports in tick order**, Material Base first (they're the producers everything else reads), each landing with: structure from the frozen lane, conformance vectors **only** from live surfaces, sentinel row, §5.4 defect check, and a §6.5 ceremony if values move.

**Tensions to watch:**
- **T3 reality vs. record:** state.yaml says "babylon-graph consumes hypergraph-rs"; today babylon-graph depends on babylon-kernel alone, `MemoryGraph` is the production store, and the capability delta (5 of 7 gaps are XGI-parity silent permissiveness vs III.11 loudness) defers the swap behind the trait. Plan storage work against `GraphSubstrate` only.
- **ADR174 vs. scale:** "persistence stays Python glue" is comfortable at Wayne scale; at national scale the FFI/IPC chatter between a Rust tick loop and Python persistence may force the issue. T4 + checkpoint-only hydration is the pressure-release valve — decide the boundary before the 520-tick national run is the conformance target.
- **Dormant-system ports:** 12 of 34 systems fire on canonical scenarios. Porting dormant systems (Policy, Community, EdgeTransition, Transport) without live scenarios produces code that can't be conformance-proven — either charter a scenario that fires them, or port them as BSL content where rules are cheap to leave dark honestly.
- **Narration envelope:** the Rust engine must emit byte-identical `NarrationEnvelope` JSONL; goldens exist; treat as a hard contract (OQ-32 closure).

**Open questions for the Director (not rulings):**
1. T4 sub-question (already flagged in ADR179): topology object cadence — every tick vs checkpoint-only?
2. When the engine is Rust, does `babylon-engine` call Python persistence (FFI inversion, (a)) or does the server boundary move to sqlx + the same DDL ((b))? Current rulings imply (a) near-term; worth a ruling before Phase 3 hardens the seam.
3. AGE adoption gate: who verifies AGE+PostGIS coexistence and the CI image story (r33 pin ceremony) before T4 commits to it?

---

## Appendix A — Documents of authority (read these first)

| Doc | Role |
|---|---|
| `NORTH_STAR.md` | The mental model (four strata + projection lane; Rust = engine language since AE) |
| `CONSTITUTION.md` (v3.0.1) | The law; Amendments AC/AD/AE + D/T/V/W/X/Y/AA |
| `ADR172` | Amendment AE ratified — Rust engine language, native hyperedge, no imposed forms |
| `ADR183` | "Get it right in Rust" — the porting doctrine (structure oracle; defects at port; structural gating) |
| `ADR174` | Python glue boundary — hot-path calc → Rust; I/O → Python |
| `ADR179` | Topology spine T1–T4 (adjacency live; two hashes; hypergraph-rs behind trait; T4 Postgres object) |
| `ADR176` | r20 RNG · r21 intrinsics · r30 h3 BIGINT · r31 checkpoint-only hydration · r32 retention · r33 PG17 CI · r34 edge retirement |
| `ADR180` | Lane A heat rulings R3–R7 (L/K/X split; co-optation composition) |
| `ADR182` | Write log (R1); structure public, magnitudes earned (R2) |
| `ADR184` | Capacity belongs to organizations (one Capacity replaces two finance models) |
| `ADR185` | Cascades observable (R2); iteration order (R4) |
| `ai/bsl-architecture-standard.md` | BSL invariants S-1…S-32 + §5.4 "Defects not to transcribe" |
| `docs/reference/phase-1-exit-checklist.md` | P27 Phase 1 exit + Phase 2 reading list |
| `ai/state.yaml` | Live truth-status ledger (read its banner first) |
| `CLAUDE.md` | Operating guidance incl. per-system annotations + gotchas |

## Appendix B — The "do not transcribe" register (§5.4 + adjacent)

1. The never-called gate layer (`apply_fog`/`compute_veil_status`/`gate_value_axis_fields` never run on the shipping path) → replace with structural gating (required-argument projectors).
2. The 100,000 employment literal (`_DEFAULT_EMPLOYMENT` divides every canonical tick; `employment_source` only ever wired by the legacy web bridge).
3. Housing/dispossession never-fed adapters.
4. The fictitious-capital Z1 adapter.
5. Two finance models → **one `Capacity`** (ADR184; budgets in Currency; Φ/tribute replenishment without conversion).
6. `RevolutionaryFinance.heat` → never ports (ADR184 R6; heat becomes derived exposure X).
7. Edge-transition predicates referencing unproduced field names → re-charter vocabulary as BSL content with ceremony.
8. `trap_detection.py` → superseded (P25 U11), legacy-web only.
9. phi_hour tick-52 regression + RNG seed-threading gap (Phase 0 findings) → repair at port; the Python N=32 ensemble is dropped (R8), single Python trajectory is the reference.
