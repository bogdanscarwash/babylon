# 05 — Catalog Execution: Waves 2–5, Local Play, Doc Hygiene

The catalog of record is `reports/aidocs-vs-code-audit-2026-05-16.md` Part 3
(spec text) + Part 4 (dependency graph). This file adds execution order,
2026-07-02 annotations, and the non-catalog work needed for "the game works
locally". Do NOT duplicate the spec text here — read the audit section when
starting each spec. Process for every spec: `00-mission.md` working
agreements (speckit lifecycle, TDD, full-vision no-MVP).

## Program 09 cross-cut (ratified 2026-07-03 — read `09-program-full-game.md`)

The full-game program runs BESIDE this catalog on four parallel lanes:
`[E: 071→101→102→104→105]` (engine, serialized, owns baselines),
`[W: 090→091→{092∥093}→094→095→103]` (web product per design canon),
`[D: 100 ∥ 098-LODES ∥ 068-slice]` (data), `[O: 096→099]` (Observatory
debug dashboard). It does not reorder this catalog; it feeds it —
092–095 build the surfaces Wave-3 specs animate, and Lane E resumes
Wave 2 (072–074) after the trade window closes. File-ownership law and
sync points: `09` §3.

## Wave order (after 071; interleave 098 throughout)

### Wave 2 — Player Organizational Economy (unblocks the game loop)

- **spec-072 Vanguard Economy System** — the player-org resource loop
  (cadre/sympathizer labor, budget, reputation, heat). Frontend already
  renders `OrgState.vanguard` (wired 2026-07-02 in the web sprint); the
  backend engine economy behind it is this spec.
- **spec-073 Cohesion Mechanic (Iron Law of Oligarchy)** — org-internal
  decay/discipline. Interacts with 071's chauvinism-defection path; build
  after 071 so ORGANIZATIONAL_FRACTURE has both inputs.
- **spec-074 Demographic Crisis & Resolution Pathway Selector** — note: the
  engine's demographic base (population attrition, vitality) is now REAL and
  gated (see `02-engine-truths.md` §2); this spec builds the crisis
  RESOLUTION layer on top.

### Wave 3 — Player Action Verbs (3D Conflict + 3C Information)

- **spec-075 Kinetic Warfare (ATTACK verb)** — critical path. Depends on 071.
- **NEW: Commodity-flow routing ("slime mold", 07 §2 M1)** — per-tick
  min_cost_flow over (hex × SCTG) O-D pairs with conductivity EMA;
  unrouted demand → realization crisis. Percy-requested spec; it is what
  makes 075's edge-severing economically real. Build WITH or immediately
  after 075.
- **NEW: Prose→stance-vector verb input (07 §2 M2)** — out-of-tick LLM
  parser → frozen StanceSchema with coherence-refusal; presets fallback.
  The input model for ALL Wave-3 verb specs; stance = signed intervention
  on an opposition's balance (couples to the dialectics refactor).
- **Verb sub-modes + verb-page UX (07 §2 M3, §3 X4)** — ~20+ interventions
  under the 9 verbs; feedforward projections, unavailable-with-reasons,
  real-number tradeoffs, consciousness-cascade readouts, 3×3 verb grid.
- **spec-076 Fog of War & Intel Layer** — the Intel page (4-variant inspector,
  incl. Communities) is already live against snapshots (2026-07-02); this
  spec adds the epistemic layer (what the player is ALLOWED to see).
- **spec-077 Gramscian Hegemony System** — player narrative warfare. Pairs
  naturally with the crisis-gated consciousness machinery (§4 of
  `02-engine-truths.md`): 077 is how the PLAYER moves visibility/agitation.
  - Its DISPLAY SURFACE is the **Gramscian Wire triptych** (07 §3 X2):
    Corporate Feed / Liberated Signal / Intel Stream, euphemism
    dictionaries, hegemony-driven visibility, diegetic downtime — already
    designed (2026-05-17 artboards). Narrator voice+stack per 07 §3 X1
    (Mao "Oppose Stereotyped Party Writing", tool-calling tone enum).
- **spec-078 Repression Logic Full** (ROE tiers, COINTELPRO, Malinovsky,
  snitch budget) — StruggleSystem/state-AI deepening; keep the
  income-circuit hegemony test green when touching StruggleSystem.
- **spec-079 Panopticon Economy (Θ attention budget)** — heat/state-attention
  economy; TerritorySystem heat dynamics already exist as substrate.

### Wave 4 — Strategic Choice

- **spec-080 Doctrine Tree MVP** — depends on 072 (CL/coherence inputs), 070
  (colonial_stance filtering), 071 (NATIONAL_CHAUVINISM tag). Build only after all
  three are merged.
- **spec-081 Warlord Trajectory Branching** — end of the critical path.
  - Ship WITH it: the **chronicle end-screen + Journal objectives**
    (07 §2 M4) — the UX that reconciles "no victory state" with the
    5-outcome enum (outcomes are characters of collapse, not wins).
- **Liberated-zone mechanic** (07 §2 M11) — org→institution transformation
  with a bidirectional spatial boundary the state can dissolve.

### Wave 5 — Cleanup & Loop Closure

- **spec-082 Reproductive Labor Tier-1 + S_imperial explicit accounting** —
  touches the value tensor; coordinate with 098's data slices (ATUS).
- **spec-083 Political Economy of Liquidity** (Fiscal + Fundraising +
  Precarity) — closes the money loop for orgs.
- **NEW: Resource-substrate ledger** (07 §2 M5) — physical units (MWh,
  tons, GPU-hours) feeding `c` via MELT; grounds biocapacity with
  EIA/USGS; physical shocks = discrete crisis resets.

### Waves 6–7

Content/polish waves — read the audit TL;DR + Part 3 tail when Wave 5
closes. The chat-corpus review (07 §§2-3) pre-seeds them; plan details
stay demand-driven:

- **Wave 6 (experience + distribution)**: in-game wiki via composable
  tooltips + term-registry linter (X3 — the onboarding strategy, pairs
  with 085); chronicle/Journal if not shipped with 081 (095 builds the
  surfaces early — `09` §2); visual identity + palette (X5 — RULED
  2026-07-03: AI discretion, "impress me"; **executing NOW as spec-090**
  on the Percy-ratified Cold Collapse canon. CORRECTION: the earlier
  "GOLD=solidarity still binds" note is superseded — Cold Collapse
  assigns solidarity `#5fbf7a` and reserves gold as scarce `rupture`;
  Constitution Article VII's literal "GOLD (action/solidarity)" clause
  therefore needs the amendment 090 drafts and Percy ratifies —
  `09` §1 R-VII);
  accessibility requirements (X8 — colorblind-safe ramps, non-color
  magnitude channel, keyboard nav); audio direction (X7); modding +
  console (M6); Steam distribution (M7 — RULED: web app stays the
  product, Steam = wrapped-web build, PyQt6 binary retired); **production
  deployment track** (M8 — Hetzner+Cloudflare, 12-page spec-kit PDF
  exists; narrator RULED: Workers-AI with LoRA in scope); map viz specs
  (X6 — value-flow arrows lens, BubbleSets hulls, unit iconography,
  sovereignty-overlay rendering); Synopticon surface (X9, with 078/079).
- **Wave 7 (horizon)**: international circulation layer (M9 — its first
  spec set was PULLED FORWARD 2026-07-03 as program-09 specs 100–103
  (owner-scoped: the canonical 8 bloc nodes, boundary-flow activation,
  gamma hydration, scheduled shocks, trade UI surfaces). RULING (`09`
  §1 R-AMEND): non-agentic Layer-0 register-pattern blocs need NO
  constitutional amendment — spec-062 machinery is already sanctioned;
  the amendment trigger is blocs becoming agentic or growing recursive
  internal structure, which stays a Wave-7 decision); scale-out (M10 —
  RULED 2026-07-03: keep it all in one Postgres; federation/columnar
  substrate stays parked).

## "Game works locally" — remaining non-catalog items

Verified state 2026-07-02: backend web tests 246/246, frontend 310/310,
TopBar + Intel on live snapshots, canonical engine alive end-to-end.
Remaining, in rough order of player-visible value:

1. **Unsupported verbs**: `investigate`, `move`, `negotiate` return
   UNSUPPORTED from the engine bridge — they need engine-side handlers.
   Most of these belong INSIDE catalog specs (076 investigate/intel, 075
   move/attack, 077/negotiate) — implement them there, not as one-offs.
   The authoritative verb surface is the 9-verb, 16-route frontend vision
   in `docs/agents/babylon-frontend-reset-prompt.md` (Educate, Aid, Attack,
   Mobilize, Campaign, Move, Investigate, Reproduce, Negotiate — player
   acts through Organizations only, per the Constitution).
1. **Django `accounts` app has no `migrations/` directory** — generate the
   initial migration (`mise run web:migrate` after `makemigrations accounts`)
   and commit it.
1. **Dashboard endpoints** (`get_economy`, `get_edges`, `get_state_apparatus`,
   `get_journal`, `get_alerts` return `{}` in EngineBridge): descoped
   2026-07-02 WITH EVIDENCE — zero frontend consumers exist. Build each one
   WITH the page that consumes it. *Update 2026-07-03: the consumers are
   now SCHEDULED (program 09): `get_journal`/`get_alerts` → 092,
   `get_economy` → 093, trade sections → 103, Observatory endpoints →
   096/099. The rule stands — nothing ships without its page — but
   "descoped" no longer means "unplanned".*
1. **Playtest pass**: after 072 (vanguard economy) + one Wave-3 verb, do a
   real seeded-game playthrough (`mise run web:dev`, new game, ~50 ticks) and
   file what breaks as issues — that list, not intuition, drives the next
   local-play sprint.

## Doc hygiene backlog (audit Part 5 + 2026-07-02 additions)

- Audit Part 5 lists the OBSOLETE-banner and update targets; epochs/ banners
  and roadmap-authority pointers were done 2026-07-02 (commit `108a6422`).
- `ai/decisions/index.yaml` was backfilled ADR030–040/048/049/050;
  keep it current with every new ADR.
- Keep `ai/state.yaml` version-bumped per significant merge.
- Keep THIS kit's `01-state-of-the-world.md` current — it is the first thing
  a fresh agent reads.

## Standing verification loop (run after every engine-touching spec)

```bash
mise run check                     # lint + format + typecheck + unit
poetry run pytest tests/integration/test_bridge_income_circuit.py -q
mise run qa:e2e-regression         # 5-tick gate incl. population liveness
# and before merging anything that changes canonical dynamics:
mise run sim:e2e-bg                # daemonized 520-tick canonical (~1-2 h); watch: mise run sim:status
# then verify: terminal_state.counties_with_population == counties_alive == 83
```

> **STORAGE LIFECYCLE (updated 2026-07-03, specs 087–089/ADR053)**: the
> old 7 GB/run problem is SOLVED (delta persistence + partitioning —
> canonical Michigan now writes ~455k hex rows, ~100 MB-class). The
> standing rule: after every canonical run, archive it —
> `mise run sim:archive -- archive --session <id>` (Parquet+zstd →
> verify → instant DROP-PARTITION purge; DuckDB reads the archive).
> `mise run sim:status` shows liveness; `qa:storage-budget` gates
> rows/tick regressions; `clean:docker` still flushes leaked test
> containers. Hex history reads go through `v_hex_state_asof` — raw
> `dynamic_hex_state` is sparse by design.
