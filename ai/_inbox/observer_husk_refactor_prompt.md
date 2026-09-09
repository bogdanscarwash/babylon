# Program Prompt — The Third Role Does Not Exist (Observer/Monitor Husk Excision)

You are executing a refactoring program on **Babylon** (`dev` branch). Branch from `dev`; conventional commits; TDD; never commit to `main`/`dev` directly. The full spec below is the minimum viable plan — no phase-1 cuts, no MVP scoping.

**Mission.** Excise the legacy observer/monitor class hierarchy and re-home every observation concern into one of the three constitutionally stable roles. The codebase currently carries a dead architectural pattern — the runtime watcher — whose math has already been harvested by the engine and bridge and whose laws have already been reborn as sentinels. This program deletes the husk, migrates the last load-bearing knowledge out of it, and promotes the few checks that can name their law.

**Governing ruling (binding).** Under Constitution v2.10.x there are exactly three legal homes for observation, keyed by who consumes the output and when:

1. **Output matters to state** → an engine **system** emitting events/graph attrs, deterministic, baseline-pinned (III.7). Precedent: the Aufhebung signal is the engine-side `LEVEL_TRANSITION` from ContradictionSystem @18, not a monitor hook.
2. **Output is a law about the motion** → a **sentinel**: declared data at layer-0.5 (frozen Pydantic identity declarations, loud at import per III.11), checking logic in the test layer, run out-of-band via `tools/sentinel_check.py` + mise, gating/advisory tiers (III.12 redundant verification). Precedent: `sentinels/conservation/registry.py`.
3. **Output is for eyes** (player, narrator, AI) → a **bridge/`observe()` projection** — a pure read at bridge altitude, reusing engine formulas as pure functions (II.5, II.8). Precedent: `engine_bridge` reusing `calculate_component_metrics`.

A runtime watcher — code hooked into the live run that is neither of the three — is the fourth role, and **the fourth role does not exist**. Amendment S makes observation a coarse-graining of dialectical motion; III.11 kills "watches and might matter"; the shadow→promotion ceremony (ADR077→078) is the lifecycle that replaced it. Nothing in this program may create a new runtime observer, and nothing may survive it in that role.

---

## 0. Read first, in order

1. `CONSTITUTION.md` — II.5 (AI observes, never controls), II.8 (client as presentation layer), III.7 (determinism), III.10 (Earn-Its-Keep), III.11 (Loud Failure), III.12 + VIII.13 (Behavioral Contracts / Spec Trapped in Implementation), VIII.12 (Silent No-Op), Amendment S (apex; statics derived).
2. `ai/decisions/ADR051_lawverian_dialectics_refactor.yaml` — deferred item `topology_monitor_unwired` (the ruling this program completes) and the E0/E2 record of where the monitor program's production half went (Systems 19–21, `LEVEL_TRANSITION`).
3. `ai/decisions/ADR058` + `ADR059` — the src-sweep + fork-reconciliation-ledger pattern: **census → adversarial verification → disposition per item → owner ratifies contested calls → implement**. ADR075's census is the second proof of the pattern; note its lesson: 11 of 41 first-pass orphan claims were overturned on appeal. First-pass orphan detection is not trusted here.
4. `src/babylon/sentinels/base.py` + `sentinels/conservation/registry.py` — the two-tier runner contract and the declared-data-only registry shape every promotion in Phase C must follow.
5. `ai/decisions/ADR068` (sentinels family + endgame wiring), `ADR070` (partition sentinel, shadow-first), `ADR077`/`ADR078` (shadow mechanism + promotion ceremony).
6. Program 14 layering law (`ADR063` + the import-linter contract file): `kernel < models/formulas < topology < domain < persistence < engine`; `intelligence` observes only. Re-homed formulas must move **down** the stack, never sideways within `engine`.
7. The census targets themselves (§2).

---

## 1. Pre-verified census (2026-07-17 snapshot of `dev`)

The following facts were verified by direct clone-and-grep on 2026-07-17. **Re-run every grep before acting on it** — three ADRs landed on this repo *today*; treat this table as a head start, not as truth.

| Artifact | Verified state | Provisional disposition |
|---|---|---|
| `engine/simulation/_legacy.py` | The entire `Simulation` class + observer registration (`observers=`, `add_observer`). Only tool client: `tools/vertical_slice.py`. | DELETE (Phase B) |
| `engine/observer.py` (`SimulationObserver` Protocol) | Consumed by `_legacy.py`, the `observers/` package, `observer_adapter.py`. **Not** required by the endgame path (see below). | DELETE (Phase B) |
| `engine/observer_adapter.py` (`ProtocolObserverAdapter`) | Used only by `_legacy.py` + its own test. | DELETE (Phase B) |
| `engine/observers/economic.py` (`EconomyMonitor`) | Sprint-3.1 vintage. Detects >20% `imperial_rent_pool` drop, logs `[CRISIS_DETECTED]` for AI narrative. Law half subsumed by conservation identity #2 (pool non-increasing/bounded); narrative half is a log-scraping seam. | DELETE after §3.2 decision rule |
| `engine/observers/causal.py` (`CausalChainObserver`) | No non-test client. | DELETE; record deferred intelligence-layer seam (§3.3) |
| `engine/observers/metrics.py`, `session_recorder.py`, `jsonl_recorder.py` | No current-path clients found except a reference in `persistence/protocols.py` (reconcile it). Golden/dense-trace tooling is independent (`tools/regression_test.DenseTrace`). | DELETE after §3.4 decision rule |
| `engine/observers/endgame_detector.py` (`EndgameDetector`) | **LIVE.** Wired via `headless_runner` → `bridge.set_endgame_detector` (dotted-path config); `bridge.poll_endgame` calls the **bespoke** `detector.check(world, tick)` — *not* protocol methods. Endgame is ruled to move toward emergent/fixed-horizon patterns; do not invest. | KEEP functioning; SEVER protocol inheritance; RELOCATE out of `observers/` (Phase B) |
| `engine/topology_monitor.py` (`TopologyMonitor`) | Class has **no runner call site** (stated in `contradiction.py`'s own docstring). Its formula `calculate_component_metrics` is imported by `web/game/engine_bridge.py` as a pure read. | HARVEST formulas (Phase A), then DELETE class; checks → Phase C candidates |
| `engine/bifurcation_monitor.py` (`BifurcationMonitor`) | Feature-033 standalone monitor; no non-test instantiation found in `src/`/`web/`/`tools/`. Frontend `BifurcationGauge.tsx` and `domain/bifurcation/types.py` reference the *name* — verify whether the gauge is fed by bridge/domain reads independent of this class. | HARVEST anything pure, then DELETE — **contingent on the gauge audit** (§3.5) |
| `tools/vertical_slice.py` | Legacy tool; **live mise task at `.mise.toml:789`**. | DELETE tool + task together (Phase B) — never leave a broken task |
| `sentinels/` | Seven families on dev: synthetic, seam, coverage, roundtrip, dynamic, partition, conservation. Conservation = instance #3, two identities (finiteness; imperial-rent reserve). One production import from the package: `cell_name` (vocabulary, legal). | The target substrate for Phase C — extend registries only, no new machinery |

Census obligation: run the ADR075 pattern. One pass produces orphan/wired claims per artifact; an **adversarial second pass attempts to refute every DELETE claim** (grep for dynamic imports, dotted-path config strings, mise/CI references, frontend references, doc references, `getattr` dispatch). Any DELETE claim that survives refutation proceeds; any that falls is a **discovery** — stop, reclassify, and if the wiring is surprising, escalate per §6.

---

## 2. Phase A — Harvest before you burn

Move every **pure formula** currently trapped in monitor modules down to its correct layer, then repoint importers.

- `calculate_component_metrics` (and any sibling pure functions in `topology_monitor.py` / `bifurcation_monitor.py`): if it operates on the graph → `babylon/topology/`; if on scalars/arrays only → `babylon/formulas/`. Decide per function by signature, not by origin module.
- Repoint `web/game/engine_bridge.py` (and anything the refutation pass finds) to the new home. Preserve the bridge-altitude honest-null wrappers exactly as they are — the engine-internal variant's bare-0.0 behavior vs the bridge's `None` guard is deliberate (III.11); do not "unify" them.
- Update the import-linter contract for the moves.
- **Gate A:** `mise run qa:regression` — 5/5 byte-identical, unmodified baselines. A pure re-homing that moves a byte is a discovery, not noise: stop and find the wire.

## 3. Phase B — Sever and delete

Deletion order: sever the live dependency first, then delete leaves inward.

**3.1 Sever EndgameDetector.** Remove its `SimulationObserver` inheritance (the polling path uses only `.check(world, tick)` — verified, but re-verify `poll_endgame` first). Relocate the module out of `engine/observers/` (e.g. `engine/endgame/` or alongside the headless-runner bridge — pick the smallest move that lets `observers/` die). Semantics byte-frozen: same dotted-path config surface, same `check` signature, same events. This program does not touch endgame behavior; the emergent/fixed-horizon program owns that future.

**3.2 EconomyMonitor decision rule.** Grep the intelligence/narrator layer, `web/`, and log-consuming tooling for `CRISIS_DETECTED`. If any live path consumes the marker: replace with a proper mechanism *before* deletion — either an engine `EventType` emitted by the owning economic system (threshold as a `defines.yaml` coefficient, never hardcoded) or a bridge-altitude read — owner's call which, escalate with the evidence. If nothing consumes it: delete, and record in the ledger that the crisis-drop *law* is subsumed by conservation identity #2 and the crisis-drop *narrative event* was consumer-less (VIII.12 requires this record — a deleted check must be either subsumed or documented, never silently gone).

**3.3 CausalChainObserver.** Delete. Record the deferred seam honestly: if the intelligence layer later wants causal chains, that is a read over the **event stream** (pgvector Archive / `observe()` projections), not an engine hook. One paragraph in the ledger; no code.

**3.4 Recorders + metrics.** Reconcile the `persistence/protocols.py` reference (likely a stale protocol import — fix or remove). Then apply the rule: golden/dense-trace production is already independent (`tools/regression_test`), so the recorders are legacy → delete. Exception trigger: if any workflow doc, mise task, or CI job references session/JSONL recording, stop and escalate — recorded sessions might be someone's III.12 behavioral artifact.

**3.5 BifurcationMonitor.** Audit the `BifurcationGauge.tsx` data path end-to-end (frontend → web bridge → engine/domain). If the gauge is fed by `domain/bifurcation/` + bridge reads independent of the monitor class (expected): harvest any pure functions per Phase A, delete the class. If the monitor class turns out to be in the feed path: it is *state-or-eyes-bearing* misfiled as a watcher — reclassify per the governing ruling (bridge read or system output), migrate, then delete. Either way the class does not survive as a monitor.

**3.6 The husk proper.** Delete `_legacy.py`, `observer.py`, `observer_adapter.py`, the remaining `observers/` modules and package `__init__` re-exports, `engine/__init__` exports of the protocol, `tools/vertical_slice.py`, and the `.mise.toml:789` task in the same commit. Grep for dangling references (docs, AGENTS.md, comments citing the observer pattern) and fix them.

**3.7 III.12 knowledge-migration audit — per deletion, before deletion.** For every test file that dies with its class, answer in the ledger: does this test pin load-bearing behavioral knowledge whose *only* home is this test (VIII.13)? If yes, migrate the knowledge to a durable artifact first — a sentinel identity, a property law, a baseline annotation, or a predicate spec — then delete. If no, say so and delete. The ledger entry is mandatory either way; "the tests were green so I removed them" is exactly the failure mode III.12 exists to prevent.

**Gate B:** 5/5 byte-identical, unmodified baselines; full test suite green; import-linter green; `mise tasks` runs clean (no orphaned tasks); the migration ledger complete.

## 4. Phase C — Earn-its-keep promotions (gated, additive, advisory-only)

Promote monitor-corpus checks to sentinels **only** where a LAW can be named (III.10). Every promotion follows the conservation-registry shape exactly: declared frozen data at layer-0.5, checking logic in `tests/unit/sentinels/`, registered in `tools/sentinel_check.py` + a mise task, **advisory tier** on arrival. No new sentinel *machinery* — no lifecycle FSMs, no gating ledgers, no generalized frameworks; the rule-of-three discipline from ADR072 holds.

Candidates, with their laws:

- **Topology-invariant cross-check.** β₀ of the SOLIDARITY subgraph, independently computed, must agree with the connectivity cylinder's Π₀/atomization reading. This is III.12(c) redundant verification of a shipped computation — the strongest candidate, and it finally gives the original TDT observational program (Betti tracking) its constitutional home: a *law about* topology rather than a watcher of it.
- **Phase-transition claims from the topology_monitor test corpus.** Anything the §3.7 audit flags as load-bearing (giant-component thresholds, component-metric invariants) lands here as declared identities rather than dying with the tests.
- **Deferred, recorded, not built:** the Sparrow prediction (cutset targeting reduces β₁ before β₀) — blocked on I.21 `[PENDING CODE]` wiring; the κ/curvature program — fully unwired, no shipped incident history. One ledger paragraph each.

**Gate C:** additions are advisory + out-of-band → 5/5 byte-identical still holds with unmodified baselines. Each shipped identity names its law and its Aleksandrov chain in the registry docstring, conservation-style.

## 5. Phase D — Governance

- **ADR-079** (verify next free number by directory listing — 078 was current at census): title along the lines of "Observer/monitor husk excision: the three-role observation ruling, formula re-homing, sentinel promotions, endgame severance." Record the governing ruling, the census-with-refutation results, every disposition, the III.12 migration ledger, and the deferred seams (causal chains, Sparrow law, κ). Closes ADR051's `topology_monitor_unwired` deferred item — say so explicitly.
- Program doc under `project/programs/` (verify next free number — 22 was highest merged at census; 23 is market scissors; likely 24).
- Update the import-linter contract, and any architecture docs that still describe the observer pattern as current.

## 6. Forbidden moves and escalation

Forbidden: creating any new runtime-observer role or protocol; deleting a check without subsumption-or-record (VIII.12); disarming anything silently; touching endgame semantics; hardcoding a threshold that belongs in `defines.yaml`; building speculative sentinel frameworks; regenerating any baseline (this program has no legitimate baseline-moving change — if you think you found one, you found a wire instead).

Escalate to the owner, with evidence, and stop the affected item: (a) any DELETE claim overturned by the refutation pass in a *surprising* way — the ADR075 lesson says expect some; (b) the `[CRISIS_DETECTED]` consumer question if a live consumer exists (§3.2); (c) any workflow dependence on the recorders (§3.4); (d) anything that would require touching a baseline.

**Definition of done:** the word "observer" appears in `src/babylon/engine/` only in the severed endgame module's history and the ADR; every observation concern in the tree is attributable to exactly one of the three roles; gates A–C green; ADR-079 and the ledger merged with the program branch.
