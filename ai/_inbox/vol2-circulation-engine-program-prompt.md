> **CONTROLLER NOTICE 2026-07-21 (binding, read before your next unit): ADR NUMBER REALLOCATION.**
> The mainline minted ADR108 (Amendment AA, constitutionally referenced) after this worktree forked,
> and both volume lanes independently claimed 108/109 — three-way collision. Your lane's allocated
> block is **ADR120–129**. REQUIRED as a small standalone commit before your next unit: rename ADR108_vol2_u1_activation_audit→ADR120_..., ADR109_vol2_u2_lodes_data_path→ADR121_...; next free is 122.
> Update each file's id: field, your worktree's ai/decisions/index.yaml keys/file fields, and rg for
> any ADR108/ADR109 references your lane wrote (comments, docstrings, EXCEPTION notes, tests) —
> repoint them. Leave git history alone. Never mint outside your block.

# PROGRAM PROMPT — Volume II: The Circulation Engine

**Authored:** 2026-07-19 by the gap-fix controller session, at `fix/null-play-coupling` @ `3e7bff05`,
from a dedicated read-only reconnaissance sweep. **Every file:line anchor below was verified on that
tree — re-verify at program start; lines drift.**

**Mission (in the Vol III program's own words, adapted):** connect the fully-built-but-disconnected
Volume II circulation estate — the LODES commuter circulation step, the reproduction schemas, the
turnover/fixed-circulating/inventory calculators, the conservation invariants — to the running
engine; replace every stub-fed input with the real calculator; bind the circulation oppositions the
coupling graph already reserves slots for; and ship a sentinel for every error class this
investigation discovered. Wiring, patching, monitoring — the same rigor pass Volume III got.

Run this as a full program: brainstorm → spec (owner review) → plan (U-chain, per-unit review) →
subagent-driven execution → one declared ceremony. Mirror the Vol III conventions (§7).

---

## 1. Sequencing preconditions (HARD GATES — do not start before all three)

1. The Viable-Game gap-fix PR (`fix/null-play-coupling` → dev) has merged.
2. The Vol III program (`refactor/vol3-money-scissors`) has finished its remaining units (U9.7+,
   U7/U8 sentinel classes) and merged with its ceremony. Vol II shares its rooms —
   `domain/economics/tick/`, `GraphInputs`, the dialectics catalog, the seam registry, the defines
   schema — and inherits its sentinels and opposition-binding conventions. Branching before it
   merges buys zipper-merge conflicts for nothing.
3. Strongly preferred: the parquet reference-pipeline program (task #46, spec approved) has cut
   over. The raw LODES OD data Vol II needs currently lives ONLY on the drive
   (`/media/user/data/babylon-data/lodes/od/*.csv.gz` — confirmed present) and has **no
   data-catalog.yaml or data-artifacts.yaml entry at all**. The standing owner rule is *CI/tests
   never touch the babylon-data drive; data ships as deterministic artifacts* — so the Vol II data
   path is a parquet-pipeline citizen, not a drive read.

Also open ADR numbering note: the Vol III plan RESERVES ADR082/ADR083 (they do not exist as files
yet). Check `ai/decisions/index.yaml` for the true next free numbers at authoring time.

## 2. The estate — what exists, and exactly how it is disconnected (verified)

### 2a. `Vol2CirculationStep` — dormant TWO levels deep
`src/babylon/engine/systems/vol2_circulation.py` (306 lines, spec-063): LODES CSR matrix-vector
circulation as ImperialRentSystem sub-stage 5c. Full pipeline is real: hex `v` snapshot →
row-normalize (FR-011 zero-row guard) → sparse mat-vec → paired `COMMUTE_OUT`+`TRADE_EDGE`
boundary rows into `BoundaryFlowRegister` → FR-010 conservation check
(`CirculationConservationViolation`) → `update_node` write-back. Deterministic by construction.

- **Level 1:** the engine gate `ImperialRentSystem._invoke_vol2_circulation_if_wired`
  (`engine/systems/economic.py:157-190`, called at `:85`) requires context keys `vol2_step`,
  `boundary_flow_register`, `session_id`, `simulated_year` (guard at `:174-179`). **Nothing in
  `src/` or `web/` ever sets `vol2_step`.** (Historical note: older docs call this key `il_step`
  and the file `il_circulation.py` — both renamed.)
- **Level 2:** even the LODES matrix hydration never runs. `LODESCommuteMatrixLoader`
  (`domain/economics/lodes_commute_matrix.py`, Postgres round-trip to
  `immutable_reference_lodes_od_matrix`, migration 0016) is production-instantiated exactly once
  (`persistence/postgres_initialization.py:881`), gated on four kwargs
  (`postgres_initialization.py:657-679`, hydration block `:869-902`) that the ONE production
  caller (`headless_runner/runner.py:1094`) never supplies. Pure wiring gap, not a data gap.
- 11 test files construct the step directly and call `.step()` — the spec-063 direct-invocation
  suite (e.g. `tests/integration/test_circulation_determinism.py`,
  `test_paired_cross_border_emission.py`, `tests/property/circulation/test_v_conservation.py`).
  The tests are real; the runtime is unreachable.

### 2b. THE HEX RECONCILIATION (design question #1 — owner-adjacent)
`Vol2CirculationStep.step()` iterates `NodeType.HEX` graph nodes — but **no production code ever
stamps a hex node** (`sentinels/vocabulary/registry.py:90` allowlists `{"hex", "community"}`, and
its comment at `:78-83` names `Vol2CirculationStep` as one of the constructs "iterating an empty
set at runtime"; dated owner-deferred exemption, 2026-07-18 Task 1b). The standing owner ruling
(hex/community Lawverian disposition, 2026-07-18) is: **do NOT stamp hex nodes** — hex res-7 is
immutable substrate; bind ScaleAdjunction; give SubstrateSystem real dynamics; territories are
COUNTY-keyed. So the step AS WRITTEN can never be lit without violating that ruling. The program
must reconcile: refactor the step to operate on the substrate hex-state store
(`v_hex_state_asof` / spec-089 delta persistence) or on county-keyed territories with the
ScaleAdjunction binding — NOT by stamping hex nodes. This interacts with standing task #39
(hex/scale: county-key territories + bind ScaleAdjunction); decide at spec time whether #39 is a
precondition, a merged unit, or explicitly out of scope. Surface this in the spec for the owner.

### 2c. Reproduction schemas — worse than inert: the live consumer is fed a lying stub
`domain/economics/circulation/reproduction.py` (206 lines, Feature 023): `check_simple_reproduction`
(`:71`, the `I(v+s) = IIc` law), `check_extended_reproduction` (`:119`), `compute_disproportionality`
(`:161`), `combine_departments_ii` (`:39`). **Zero production callers.** And the one production site
that CONSTRUCTS their output types does not call them —
`domain/economics/tick/system/__init__.py:1360-1370` hardcodes
`ReproductionBalance(condition_met=True, gap=0.0, interpretation="Default reproduction balance")`
and a same-shaped `ReproductionAnalysis`, then feeds them to `assess_circulation_crisis(...)` at
`:1374` — **a live crisis assessor consuming fabricated always-balanced inputs**. Same pattern:
`DepreciationFundState` built directly (`:1347-1358`; `update_depreciation_fund` from
`fixed_circulating.py:64` never called — no cross-tick accumulation), and `advance_circuit()` never
called (only `initialize_circuit_state`, `:1326`). The gate for all of it:
`if capital_stock <= 0: return county` (`:1308-1310`).

Also zero-production-caller: `fixed_circulating.py` (146L), `turnover.py` free functions (257L —
but the `DefaultTurnoverProfileSource` CLASS is live via `factory.py:617` →
`tick/system/__init__.py:1315`), `inventory.py` (95L), a second unrelated
`tensor_hierarchy/reproduction.py` (NoDataSentinel stub), and the whole Feature-026 substrate
family (`substrate/circulation.py`, `substrate/conservation.py` — `DefaultConservationChecker`
with Σ(c+v+s) and hierarchical-aggregation invariants, test-only; `substrate/hex_graph_bridge.py`
is call-site-live but gated on `services.hex_grid`, never constructed).

### 2d. Web/headless asymmetry (design question #2)
The Phase-5.2 defines-gated wiring plan (`project/execution/briefs/feat-vol2-vol3-service-wiring.md`
— `ServiceWiringDefines`, batches A–D, "Batch C requires Batch A for county capital_stock > 0")
was **never executed** (zero hits for any of its symbols in `src/`). Its goal was later achieved a
different way, but UNEVENLY: `web/game/engine_bridge.py` `_bridge_economics_overrides`
(`:7775-7934`; Task 20b at `:7873-7898`, Task 21b at `:7900-7932`) wires
`create_circulation_services` + `create_vol1_services` + `create_financial_services` for web
sessions; the canonical headless runner (`headless_runner/runner.py:911-1049`,
`_build_economics_overrides`) wires gamma/melt/Leontief/tensor/financial but **NOT**
circulation/vol1. The canonical run exercises less economics than the web game. The seam registry
already books this precisely: Group C/D rows are `DECLARED_CONDITIONAL` on
`_CIRCULATION_LIVE`/`_FINANCIAL_LIVE` (`sentinels/seam/registry.py:470-935`, liveness condition
`:546-551`). The program should close the parity gap (no-compromise default) or get an explicit
owner ruling that the asymmetry is deliberate.

### 2e. Deliberately out of scope (record, don't build)
- Slime-mold conductivity (the emergent half of Constitution II.13): **zero implementation**
  (`project/programs/11-transport-substrate.md:58`); spec-063 explicitly deferred it. Keep it
  deferred — separate program.
- `src/babylon/infrastructure/` res-8 corridor substrate: built + tested, engine-orphaned
  (`11-transport-substrate.md:54`). Audit-note it; wiring it is transport-program work.
- Group C/D `tick_ground_rent` holdout (`_DefaultCountyRentalAdapter` returns None — honest
  absence) belongs to the Vol III/housing line, not here.

## 3. Data path (deterministic-artifact rule applies)
- `fact_lodes_commuter_flow` (DB) is ALIVE but feeds the throughput calculator, NOT this step —
  the step reads raw OD `.csv.gz` off `lodes_root`. Raw OD files have NO catalog entry (a Data
  Constitution gap — fix it in this program: register the source + ship the OD matrix as a
  hash-stamped deterministic artifact or Postgres-hydrated reference table per the parquet
  pipeline's conventions; `bridge_lodes_block` in data-artifacts.yaml — 1.15M-row crosswalk,
  generated-but-unconsumed — is adjacent precedent).
- Persistence path already exists end-to-end: `PerTickTransactionEnvelope.boundary_register_rows`
  (`envelope.py:78`), table `boundary_flow_register` (migration 0013, partitioned 0026, views
  0015/0030). `BoundaryFlowRegister` itself is LIVE infrastructure (Φ-distribution writes real
  rows today) — only the circulation rows are missing.
- `ConservationAuditor` (`persistence/conservation_audit.py`, LIVE every tick) name-checks
  `Vol2CirculationStep.step` in its docstring (`:246`) as the FR-010 invariant it would audit —
  register the invariant for real when the step lights up.

## 4. Dialectics binding surface (the payoff)
`domain/dialectics/instances/catalog.py`: six bound oppositions today (module docstring at `:1`
stale — still says "five"; fix in passing). **`_DEFAULT_COUPLINGS` (`:378-394`) already reserves
two dead slots named for this program:**
`Coupling(source="circulation", target="realization", kind="transforms")` and
`Coupling(source="reproduction", target="disproportionality", kind="transforms")` — silently
skipped by `build_default_coupling_graph()` (`:397-429`) because no opposition registers those
keys. Bind oppositions with EXACTLY those keys and the couplings light up with zero coupling-graph
changes.

Mechanics to mirror:
- Shadow-first: `BoundOpposition(..., shadow=True)` (`core/opposition.py:255-272`) — read every
  tick, excluded from `step()`/principal ranking, routed to `shadow_opposition_states`
  (`contradiction.py:71,204-208`). Promotion = flip the flag + byte-identical baseline ceremony +
  ADR (the `price_value` ADR077→ADR078 precedent). Note ADR072 (Divergence Channel) is still
  `proposed`/Amendment T awaiting BD — the shadow FIELD is shipped ADR077 machinery; don't block
  on ADR072.
- `GraphInputs` (`catalog.py:90-131`) has 8 fields, none circulation. Mirror the `market_balance`
  pattern exactly (`contradiction.py:284-290`): a system stamps a graph attr; `_build_graph_inputs`
  reads it and derives the input via a defines-owned scale; the catalog stays defines-free. New
  fields as `float | None = None` (Vol III convention: `rentier_share`/`debt_ratio`/etc.).
- Measures: the Vol II laws are ratios with natural zero points — disproportionality
  (`compute_disproportionality`) and realization gap (`detect_realization_crisis`) are the obvious
  measure sources. All new coefficients in defines (see §5), never inline.

## 5. Defines
No `capital_vol2.py` exists; the only circulation-adjacent define is
`economy_basic.enable_border_commute_synthesis` (`:485`). Create `capital_vol2.py` →
`CapitalVolumeIIDefines` mirroring `capital_vol3.py`'s conventions — frozen BaseModel, every field's
docstring names its exact consumer `file:function`, `gt=0.0` when the field is a live divisor, and
NO field lands unread (the `MobilizeDefines` orphan and the pre-U5 `debt_spiral_threshold` "NOT YET
READ" phase are the two recorded anti-patterns). Regenerate `defines.yaml` via
`tools/generate_defines_config.py`; `test_constants_sync.py` guards.

## 6. Sentinel units (standing rule: every discovered error class ships a gate, mutation-validated)
This reconnaissance discovered classes NO current family (13: aggregation, conservation, coverage,
dangling, fog, inert, masked_arithmetic, partition, roundtrip, seam, synthetic, unconsumed,
vocabulary) covers:
1. **Gated dormancy** — a construct that is referenced, imported, and direct-invocation-tested, so
   `inert`/`unconsumed` see a satisfied reference, but whose ONLY production call site is gated on
   a context key / kwarg chain nothing ever satisfies (`vol2_step`; the four LODES kwargs). The
   `inert` family's own scope docstring (`sentinels/inert/checks.py:1-60`) confirms it does not
   model this. Ship a registry-of-runtime-gates sentinel: every declared gate names its satisfier,
   and a gate with no production satisfier reds.
2. **Stub-fed liveness** — a live consumer fed hardcoded neutral inputs where a real calculator
   exists (`ReproductionBalance(condition_met=True, ...)` feeding `assess_circulation_crisis`).
   The function runs green every tick and lies. Gate: production construction of a result-type
   that has a designated calculator must go through the calculator (or carry a cited exemption).
3. On resolution of §2b, retire or narrow the `"hex"` entry in `UNSTAMPED_QUERY_ALLOWLIST` — the
   exemption should not outlive the defect it documents.

## 7. Program shape (mirror the Vol III plan conventions)
Model: `docs/superpowers/plans/2026-07-18-vol3-money-scissors.md` ("Connect the
fully-built-but-disconnected... bind... oppositions, and ship the... sentinel classes this
investigation discovered"). Adopt: U-numbered units with sub-units and per-unit spec-compliance
review; a **binding interface contract block** at the top of the plan pinning exact names (attr
keys, GraphInputs fields, opposition keys, defines category, ADR numbers verified free at
authoring time) BEFORE any code; house-rules block (TDD red phase, determinism, zero inline
coefficients, layering/lint:imports, honest absence); plan-splice amendments when facts change
mid-program; ONE declared ceremony at the end.

Suggested unit skeleton (the spec phase owns the final cut):
- **U1 Activation audit** — re-verify this document's §2 on the then-current dev; produce the
  definitive dormancy map (the Vol III discovery-84795 equivalent); resolve/spec the §2b hex
  reconciliation and §2d parity questions with the owner.
- **U2 Data path** — LODES OD as catalog-registered deterministic artifact; hydration wired into
  session init; CI-no-drive honored.
- **U3 Truth in the tick** — replace the §2c stubs with the real calculators
  (`check_simple_reproduction`, `update_depreciation_fund` with real deterministic cross-tick
  accumulation, `advance_circuit`); `assess_circulation_crisis` fed honestly.
- **U4 Light the step** — `Vol2CirculationStep` reconciled per U1's ruling, context wired,
  conservation invariant registered with `ConservationAuditor`.
- **U5 Parity** — headless runner wires circulation/vol1 services (or documented owner ruling).
- **U6 Oppositions** — shadow-bind `circulation`/`realization` and/or
  `reproduction`/`disproportionality` keys; GraphInputs fields; couplings light up.
- **U7 Sentinels** — §6's classes, red/green/red mutation-validated, in the umbrella + CLI.
- **U8 Defines sweep** — `capital_vol2.py`, every field read.
- **U9 Monitoring** — seam-registry rows for every new emission; observability surface
  (timeseries/inspector) veil-gated per the Veil policy where value-axis.
- **Ceremony** — single baseline pass + drift table + ADR. Determine drift EMPIRICALLY: golden
  scenarios may be no-ops for parts of this (no hex nodes, `capital_stock` gates) — an all-zero
  drift table with the reason stated is a valid ceremony (precedent: the Unit-6 doctrine ceremony).

## 8. Verification battery
Per-unit scoped tests + the existing spec-063 direct-invocation suites (they become the
activation-truth tests once the gate lights); property tests
(`tests/property/circulation/test_v_conservation.py`) must hold under the wired path;
`mise run qa:regression` byte-identical until the declared ceremony; `mise run check` +
`check:sentinels` green; the ConservationAuditor invariant live in the canonical run.

## 9. Known cross-program touchpoints
- Task #39 (county-key territories + ScaleAdjunction) — §2b.
- Task #41 (international trade) — `BoundaryEdgeKind`/external-node machinery is shared; avoid
  divergent conventions.
- Parquet program (#46) — §1/§3.
- Vol III merge — shared files; branch only after it lands.

---

## Parallel-execution addendum (2026-07-20)

Owner directive: Vol I and Vol II run IN PARALLEL as one workflow. The protocol —
branch model, the catalog.py contract commit (reserved opposition keys / GraphInputs
fields / couplings for BOTH programs), the tick-pipeline method-ownership partition,
the build-once assignments (gated-dormancy sentinel; headless-runner parity fix for
BOTH service families), single-flight heavy tests, per-volume ceremonies, merge order
— is §10 of the Vol I prompt (`ai/_inbox/vol1-value-production-program-prompt.md`).
That section supersedes this prompt's §1.2-style "branch only after the other merges"
sequencing BETWEEN the two volumes (the Vol III and parquet gates stand unchanged).
Facts drift note: since authoring, the catalog reached TEN oppositions / 12 GraphInputs
fields (Vol III landed) — this prompt's §4 counts are stale; the Vol I prompt's §4 and
its recon carry the current enumeration.
