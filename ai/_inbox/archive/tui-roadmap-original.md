Babylon — July Design Backlog Roadmap
As of: `dev@744f865` (2026-07-17). Latest ADR: 078. Everything below was designed/ruled in July chats and is not yet in the tree unless marked otherwise. Verified by clone inspection, not memory.
Ruled sequencing: Spine → Track 1 → Track 2 → Track 3, strictly one merge at a time. Command Ledger runs as engine sibling after the Spine (R-7). Systems dedup before Clone Sentinel. Everything in Epic 12 is post-1.0, parked.
Ceremony budget warning (carried from today): 2 declared baseline ceremonies in the Viable Game + 3 in Command Ledger + shop Phase 2 + K-wave Phase 2. Shadow-everything default; single-flight merges.
EPIC 1 — The Viable Game (top priority; design record committed, zero implementation)
Source: `docs/superpowers/specs/2026-07-17-viable-game-design.md`, rulings D1–D7. Needs ADR + spec numbers allocated at plan time.
1.1 Playability Spine

* Campaign pacing instrument + recalibration (ceremony #1): headless nationwide 520-tick null-play run, tick-of-first-crossing per endgame axis, tune at defines level until the arc holds tension. Fix the tick-0 "Sovereign Collapse" threshold bug.
* Event salience: consecutive same-type/subject dedup with count+age; severity tiers (crimson only for genuine rupture); autopause fires once per distinct critical event.
* Lobby + Scenario Briefing: generated codenames, delete/archive, briefing screen (who you are, stakes, five outcomes in plain language via `get_journal_objectives`), curated config via `CreateGameSerializer` overrides.
* Quick-win wiring set (all evidence-backed): CausalChainObserver → bridge → wire/journal; five distinct endgame epilogues; `preview_action` costs/warnings in ActionComposer; per-target expected deltas in TargetPicker; serialize the 26 dead `tick_*` attributes; fix the 8 phantom EconomyDashboard chips; widen the event whitelist past 44/79 and give POGROM/LOCKOUT/VIGILANTISM real EventTypes; verb dead-ends become disabled-with-reason; wire-or-remove the permanent "PROFIT no data" chip.
* Owner box-ticks (recommended YES, decide at spec review): fast-forward-to-epilogue when a basin locks; RED_OGV seed repair (CLAIMS edges never match territory IDs — one ending is structurally dead).
* Acceptance: pacing floor, no duplicate consecutive cards, autopause ≤1/event, distinct epilogues, preview-before-submit, unaided-first-action e2e.

1.2 Track 1 — The Organizer's Map (fog + solidarity front)

* Two-layer read model: material layer always public; political layer visible only within organizing reach; unknowns render as honest unknown (Loud Failure extends to pixels).
* Intel ledger: session-scoped, event-sourced from INVESTIGATE resolutions; visibility = pure function of (graph, ledger); fog lives at the serialization boundary, engine untouched.
* Solidarity edges drawn as literal map lines; edge gain/loss is a visible territorial event.
* No fogged dead ends: unknown-click yields a card naming what you don't know + INVESTIGATE link. Lenses respect fog; material lenses stay full-map.
* Contracts: no political value serializes outside reach (sentinel-style property test); monotonic intel aging; byte-identical view for identical (graph, ledger).

1.3 Track 2 — The Circuit (scoreboard room)

* Φ drawn as a literal flow circuit (periphery → core) from the imperial-rent tensor.
* Fundamental Theorem meter (Wc vs Vc per region, Φ trending) as centerpiece — veil-gated.
* Relocate scissors chart, MELT gauge, correction markers, wealth axes as designed instruments on the spine's repaired payloads.
* Chips↔payload seam contract (extend `SEAM_REGISTRY`) so phantom fields can never return; golden-pinned meter formulas.

1.4 Track 3 — The Line (doctrine screen)

* Doctrine page: tree, acquisition history, theoretical-labor economy, AP/cadre glossary via concept cards.
* Unit 6 feedback wiring (ceremony #2): doctrine feeds bifurcation/consciousness — the only remaining engine work in the program. Sign-predictability contract.
* Light cast-of-characters scaffold (factions, key figures).

1.5 The Veil of Money (D7)

* Conceptual-visibility tiers (0 money-form / 1 exploitation / 2 scissors) as a pure function of the ADR073 doctrine accumulator, enforced at serialization (below-threshold sessions never receive value-axis payloads).
* Thresholds in defines; locked instruments render veiled-with-study-path; monotonic unlocks per session. Doubles as the Circuit's onboarding.

1.6 The Voice

* CausalChainObserver frames = canonical record; narration panel UNFROZEN (persistent 3–5 causal sentences/tick); Wire stays event-anchored; Chronicle holds epilogues.
* LLM garnish contract: may re-prose a frame, never add a claim — `ai`-marked entailment eval; template fallback so nothing is ever empty. Observer-layer only, outside the tick hash.

1.7 Route migration

* Top-bar takeovers → routed, deep-linkable pages on the `(kind, id, tick?)` address scheme; every frozen testid preserved.

EPIC 2 — Command Ledger / Labor-Command Unification (engine sibling; after Spine per R-7)
Source: today's program doc `program-command-ledger-lawverian-unification.md`. Phases separable at every boundary; Phase A alone is independently valuable.

* Phase A: CommandLedger as a recomputed per-tick view (commander, commanded, hours, channel, medium; money→hours at τ_effective) + reconciliation/conservation sentinel against the value tensor.
* Phase B: hegemonic mass M_k + command-field source term (grounds the tidal lock and the W-B gravity idiom). Blocked on R-1.
* Phase C: consent⇄structure adjunction as a registry instance; θ latch makes `Organization.is_institution` computed (replaces the hand-set bool).
* Phase D: command lattice L0–L3, basin detection (dwell hysteresis vs flicker), mimetic surplus as the L1⇄L2 defect, state classifier-vs-truth dual (repression sublinearity; the player exploits the biased map).
* Phase E: faux frais decomposition F; behavioral contract that fascist-convergence firing coincides with dF/dt > 0 under dΦ/dt < 0.
* Phase F: resolve `RevolutionaryFinance` dormancy (activate-or-orphan, ADR036 spirit) and hand the shop its treasury substrate.
* Open rulings gate list: R-1 mass counting (per-relation vs value-added — blocks B1), R-2 defect signs, R-3 performativity coupling, R-4 drift-generalization timing (after Track 3 ceremony), R-5 faux frais taxonomy, R-6 is_institution grandfathering, R-7 sequencing (ruled: engine sibling, one merge at a time).

EPIC 3 — Treasury & the Shop (player economy)
Source: today's budgetary-system thread. Ties to Command Ledger Phase F and Track 2.

* Phase 1 (org-boundary, byte-identical goldens like Doctrine): org treasury, dues revenue, expense orders through the action queue; data-driven YAML catalog (`id, value_hours, category, k_self, discipline_req, mass_labor_ok, legibility, effects`); one denomination, two routes (labor at value / money at scissors price); cadre pool + sympathizer pool with deterministic availability; vendors as edges bound to seller nodes, edge-mode progression EXTRACTIVE→TRANSACTIONAL→SOLIDARISTIC as a gameplay rule; purchases prop vendor P(S|A).
* Phase 2 (promotion ceremony): conservation coupling — dues deduct member wealth, vendor wealth increments, rent flows to landlord; regenerated baselines + authorizing ADR.
* Decisions needed: hiring as a third route (money→cadre labor, i.e. variable capital inside the party); system placement in the 32-system registry (pay-last-tick's-price recommended to dodge MarketScissors ordering).

EPIC 4 — Generalized Cycle Construct + Economic Conservation Sentinel
Source: today's cycle-abstraction thread; implementation prompt written.

* Phase 0 instrumentation audit + owner checkpoint (probe-first, zero engine mutation by default).
* Cycle as closed walk in the value-form category; M/C adjunction with monad/comonad recovering C-M-C and M-C-M′; yield as holonomy; surplus localized at production morphisms.
* Ship the four laws L1–L4 (no creation in circulation; cycle accounting balance; rotation invariance; global transfer antisymmetry) — this is the organizing schema for the queued Economic-Conservation sentinel work.
* D-P-D′ recorded in the ADR future-work table only (no phantom registry entries).

EPIC 5 — Systems Dedup Refactor → Clone Sentinel (ordered pair)
Source: today's kernel-and-sentinels thread; both spec prompts written and pinned to `dev@744f865`.

* 5.1 Systems dedup refactor: execute the census'd duplicate-site consolidation across the 32 systems; ADR commit first; event registry frozen at 47 types (the 45-type widening is the separate ADR068 §d ruling — do not be helpful about it); tombstones for retired patterns.
* 5.2 Clone Sentinel: new sentinel package + CLI verb (fingerprint twins, name-net, complexity ratchet with stale-budget detection, tombstone enforcement incl. `CT-001`); exit-code contract 0/1/2; committed `reports/clone-punchlist.md`; full run < 2 s; zero runtime imports into the engine.
* Then seed baselines/blessings against the post-refactor tree.

EPIC 6 — Observer Husk Refactor (verified unexecuted: EndgameDetector still in `engine/observers/`)
Source: today's topological-dialectic thread; `observer_husk_refactor_prompt.md` written.

* Binding trifurcation: output-to-state → system; output-as-law → sentinel; output-for-eyes → bridge read; the fourth role does not exist.
* Census-with-refutation before any deletion (two known live wires: the `.mise.toml:789` task → `vertical_slice.py`; `persistence/protocols.py` recorder references).
* Phase A harvest: move pure formulas down-stack per Program 14 layering. Phase B: sever EndgameDetector's protocol inheritance, then retire the husk; per-deletion III.12 test-knowledge audit. Phase C: promote only the β₀ cross-check (+ migration-audit finds); defer sparrow-β₁ and κ.
* ADR-079 + fork-reconciliation ledger merge with the branch. Zero legitimate baseline moves.
* Pre-ruling to save an escalation round-trip: who consumes `[CRISIS_DETECTED]` (engine EventType vs bridge read).
* Coordinate with the standing direction change: endgame moving off the 5-outcome adjudicated detector toward emergent/fixed-horizon patterns.

EPIC 7 — Sentinel Expansion (beyond the Clone Sentinel)

* Spectral Sentinel (#7, advisory-only): λ₂ on the normalized Laplacian; conductance of declared cuts; Cheeger sandwich consistency; Fiedler sign structure scored against the Program 19 partition. Determinism spec: stable node order, declared sign/tie rule, Amendment Q rounding, honest null on degenerate λ₂. Next step: draft the registry model + four check definitions (offered, not yet written).
* Cycle-conservation checks land under Epic 4; the command-reconciliation sentinel under Epic 2 Phase A — don't double-build.

EPIC 8 — Consciousness Coupling Law (CCL)
Source: today's Lawvere-category thread; amendment drafted, not in tree.

* Ratify the amendment: exactly two stored variables (P(C) simplex position, Q(e) suppressed-contradiction charge); A/G/T/H/D derived-never-stored; CCL-1 routing discipline (nothing writes P directly); CCL-2 anchor law (rent-held × rate-of-decline); CCL-3 drive decomposition (intra-group solidarity routes to the anchor — fascist for rent-threatened communities; cross-divide routes revolutionary).
* Implement post-ratification: defines registry + Macomb-bellwether calibration targets; six-row falsifiability table wired to named data sources; eight anti-patterns into the constitution.
* Interlocks: Track 3 / Unit 6 ceremony and the Command Ledger drift generalization (R-4 orders CCL-adjacent promotion after Track 3).

EPIC 9 — K-Wave Lawverian Program (frame + validation, never a forcing function)
Source: today's Kondratiev thread; `kwave_lawverian_program_prompt.md` written.

* Register three opposition instances + state models + defines entries (Aleksandrov notes for anything authored); no stored phase clock anywhere — 2020s hydration data is the winter.
* Phase 2 promotion: engine wiring with R-PROOF docs + regenerated baselines.
* Correspondence harness vs historical long-wave phase chronology + report template; hydration coordination notes for the ingest workstream (per-county wealth stays with ADR051's open item — don't fork it).

EPIC 10 — Interrogable Field (sibling program; Viable Game consumes, never builds)
Source: `PROMPT_interrogable_field.md` (2026-07-17 early AM thread).

* W-A wiki closure over the landed weather layer: every rendered element inspectable; concept cards; formula terminals with provenance; generated wiki on the `(kind, id, tick?)` scheme; no card is a dead end.
* W-B gravity/mass idiom stays proposal-only — note its mass-grounding question is answered by Epic 2's M_k; cross-link rather than duplicate.
* Resolve its five owner rulings before Phase 1 exit; quarantine intact (R4 forecast overlay, Wave-5 deception, Wave-6, curvature terrain Law-4 HELD).
* Optional harvest from the mock: GEO⇄TOPO projection toggle, flow-species differentiation, double-well basin instrument.

EPIC 11 — Pydantic Modernization
Source: 2026-07-17 early-AM thread.

* Discriminated unions for node/edge payload dispatch + JSON Schema 2020-12 round-trip tests.
* Module-level TypeAdapter bulk hydration from Postgres bytes; strict mode on ledger models (mechanical Loud Failure), lax retained at ingest boundaries.
* 2.12→2.13 upgrade with `qa:regression` as the acceptance gate (serialize_as_any / polymorphic-serialization risk).
* PydanticAI replaces the ADR015 hand-rolled protocol: `output_type` + `output_validator` turns "never fabricate engine data" into a validation invariant; ModelRetry + FallbackModel cover the provider triangle. Rejected (don't revisit): pydantic-graph, Logfire, `model_construct` in the hot loop.

EPIC 12 — Run the Postgres/Django/Data-Architecture Review (yesterday's deliverable)

* Execute the 8-lane parallel-subagent review prompt (schema/migrations; determinism + hash chain; transactions/tick boundaries; per-tick I/O rule; layering; tests/ops/config; Django bridge incl. dual-migration conflict, `v_hex_state_asof` read path, write-boundary enforcement; reference-data/ingest pipeline incl. SQLite read-only + defines discipline).
* Triage findings into a punch list; feed anything structural into Epics 5/6.

EPIC 13 — Infra & Ops Minimal Stack
Source: 7/15 Nix thread (verified: no `flake.nix` in tree).

* Nix flake devshell (~30 lines, devshell-only scope) + `flake.lock` version pinning.
* Nightly `pg_dump` to object storage — the one non-negotiable.
* Reconcile the CI-runner note: the 7/15 plan said pytest-via-uv, but the tree has since settled on poetry (uv.lock deleted). Pick one and record it.
* Server posture decision when convenient: minimal NixOS reusing the flake vs Docker Compose (ruled equivalent).

EPIC 14 — Spatial & Demographic Substrate
Source: 7/3 H3 thread + today's DPD thread (verified: no NLCD, no reproductive-labor module).

* Data-source amendment: add NLCD (and formalize Natural Earth) to the approved list — gates everything below.
* Terrain as behavior gate: keep the small enum, add a typed biocapacity composition vector on hexes (which UseValues extractable, at what stock); rasterize NLCD to R7 by majority fraction; rivers live on edges, not cells.
* ReproductiveLabor / DPD grounding (backlog, explicitly out of current scope): spec the ADR for cohort inflows using Census age structure, CDC life tables, county fertility — closes the "DPD has no birth parameter" gap. Sequence after Epic 4 records D-P-D′ in future-work.

EPIC 15 — Post-1.0 Modularization (parked until 1.0; record now, build later)
Source: today's Rust-modularization thread. Boundary principle ruled: anything sharing the tick-hash contract stays together.

* Pre-1.0-able: execute ADR069 cockpit-submodule extraction (accepted, awaiting green-light); `babylon-data` repo (already deferred-ladder item 4).
* Gates first: the deferred-repo-refactors ladder (reference-DB reproducibility, two-tier contract, poetry build fix).
* Post-1.0: Rustify the deterministic core as one unit (not topology alone); correspondence harness over the 5 `qa:regression` scenarios + michigan-e2e with ULP-vs-semantic divergence classing; declared baseline epoch under Amendment L; storage behind a trait (Postgres server / rusqlite+parquet desktop); Tauri desktop wrap of the existing cockpit; `babylon-engine` extraction only after cutover stabilizes.
* Ruling needed before critical-path ordering: desktop/Steam primary vs web canonical.

Small tasks (single-sitting)

* MELT denominator audit: check whether the melt module computes GDP/L vs value-added(NDP)/L; align code with the TSSI-note definition or document the bias.
* Residual repo hygiene (the 7/10 program is verified mostly executed — doc trees consolidated, uv.lock/package.json gone, baselines slimmed, root allowlist live): audit the 37 still-tracked `reports/` files, fold `setup.cfg` into pyproject, make the one-time `git filter-repo` decision.

Consolidated decision queue (blocking, owner)

1. R-1 hegemonic-mass counting (blocks Epic 2 Phase B).
2. R-7 confirmed sequencing vs Viable Game merges (ruled; hold to it).
3. Viable Game box-ticks: fast-forward-to-epilogue; RED_OGV repair.
4. Shop: hiring as third route? System placement slot.
5. ADR068 §d event-registry widening (interacts with spine quick-win #7's wire whitelist — decide together).
6. `[CRISIS_DETECTED]` consumer replacement (pre-rule for Epic 6).
7. Interrogable Field's five rulings.
8. NLCD/Natural Earth source amendment (gates Epic 14).
9. Desktop/Steam vs web canonical (gates Epic 15 ordering only).
10. R-2…R-6 as their phases arrive.
