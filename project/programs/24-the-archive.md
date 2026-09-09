# Program 24 — The Archive: wiki-as-gameplay terminal client

**Ratified:** 2026-07-20 — two BD ruling batches in-session: (1) codename + number, the
P4 cutover gate, §6.5 provenance home, git-doctrine adoption items 1–3; (2) the P0 exit
batch — stack ADR099 as drafted, the brief §10 sextet (below), embedding pin deferred to
P1, next-trains order (keel next; Vol I ∥ Vol II alongside when the BD wants the engine
train running).
**Status:** **P1 THE KEEL COMPLETE** (2026-07-20 night, same session as P0: all five
keel items + the county end-to-end exit + git-doctrine side items 1–3). The exit
criterion is pinned by `tests/integration/archive/test_county_e2e.py` — tick →
projection ≡ committed fixture → baked page → rendered → SVG snapshot golden — and
the II.11 TODO is discharged by `docs/reference/projection-registry.rst`. The
tick-commit seam is the runner's `TickCommitObserver` (None-default; qa:regression
byte-identical with it in place). Next: **P2 fan-out lanes.** Three Kitty eyes-on
checks remain owner-verifiable (spike RESULTS.md commands); failure reopens only
ADR099's graphics-lane rows.
**Authority:** this file is the program master. Design canon (binding, incorporated by
reference): `ai/_inbox/tui/20260719archiveinterfacedesign.md` (rulings R1–R8, synthesis
S1–S11), `ai/_inbox/tui/20260719archivestackresearch.md` (stack evidence),
`ai/_inbox/tui/babylonlocalfirstinfrastructure.md` (deployment D1–D3),
`ai/_inbox/tui.md` (seed brief). Roadmap: `project/roadmap.md` §3. Constitutional basis:
Amendments V/W/X (v2.12.0, ADR093). Stack decisions: ADR099 (draft → ratify at P0 exit).

## Vision

Babylon's interface is not a dashboard. It is **the Cadre Council's own files**: a wiki in
a terminal — pages, wikilinks, backlinks, fuzzy switcher, command palette — where every
entity the organization knows is a page and playing the game is reading, navigating, and
acting on documents. The loop: **read the Archive → form a theory → issue verbs through
menus → the engine adjudicates → events revise the Archive.** Between engine and eyes
there is exactly one kind of thing: a **declared projection**. Clients are disposable;
projections are the product.

## Owner rulings (binding)

From the design brief (2026-07-19): **R1** pivot — the React cockpit is superseded as
primary client (Program 12 Phase A data spine survives and feeds this); **R2** wiki is the
gameplay metaphor; **R3** point-and-click AND modal keyboard nav, neither second-class;
**R4** no LLM in the input path, narrator = flavor only, game fully playable/informative
with it off; **R5** the deterministic in-process engine adjudicates, Postgres powers the
read side via declared views; **R6** pages are template-constructed from projections;
**R7** Victoria-3 nested views are canon; **R8** hypergraph shapes render terminal-native.

From the local-first session (2026-07-19): **D1** game-managed embedded Postgres 17
cluster on the player machine (unix socket, no TCP, no docker); **D2** Ollama-first
narrator/embeddings, Workers AI opt-in fallback, mute always legal; **D3** Grafana + SQL
for fleet observability. Baseline platform: Debian 13; WSL2 documented workaround;
native Windows/macOS = 2.0.0.

From tonight (2026-07-20): codename **The Archive**, **Program 24**; the **P4 cutover
gate is RATIFIED** (below); §6.5 provenance discipline lives permanently in CLAUDE.md +
CI; git-doctrine adoption items 1–3 approved as P1-era side tasks.

## Constitutional coverage

- **Amendment V** (LIVE): II.8 transport-generalized client + II.5 narrator-only AI — the
  program's in-process `observe()` client and R4 posture are constitutional, not novel.
- **Amendment W / III.13** (RATIFIED · PENDING CODE): deterministic materialization + the
  golden vault. **This program is the pending code.** Until P4, nothing may describe the
  vault pipeline as existing.
- **Amendment X**: X.7 flake toolchain (implemented); **X.8 local-first
  Periphery/Metropole (RATIFIED · PENDING CODE)** — discharged by the packaging train
  after cutover (local-first doc §8 parcel 3).
- **II.11 follow-up TODO** (subsystem boundary contracts + table-ownership registry): the
  P1 projection registry is the constitution's own mandated spec — write once, cite here.
- **Amendment D must stay untriggered**: Lane T renders read-only *orderings* only; no
  hyperedge-mutation affordances while II.7 is in transition state.
- **Engine untouched**: zero engine-value drift for the program's entire life;
  `qa:regression` byte-identical except the one declared golden-vault seeding ceremony
  (P4). No other ceremony is sanctioned by this charter.

## Phases (exit criteria are observable capabilities)

**P0 — Charter & evidence (serial, in flight).** Deliverables: this charter; the
throwaway spike against the stack research's 8-item falsifiable checklist (scratch dir,
never shipped); stack ADR099 with in-env evidence + the deployment guardrail section
(DSN-from-config, unix-socket support, PG 17 pin, provider precedence
Ollama→Workers-AI→mute, campaign-uuid minting, per-campaign embedding pin). **Exit:** BD
ratifies stack ADR + the brief §10 ruling batch + embedding pin in one sitting.

**P1 — The Keel (serial; everything downstream imports this).**
1. **The Hoist** — transport-neutral projection package realizing the `observe()`
   contract; `web/game/fog/*` and serializer read-model logic move out of Django with
   test-port ledger discipline; Django keeps thin shims. Discharges the II.11 TODO.
2. **Projection registry + contract pattern** — declared SQL views, frozen view-models,
   explicit ORDER BY, FTS columns; Pydantic modernization (TypeAdapter hydration,
   discriminated unions) lands here.
3. **Vault materializer skeleton** — bake at tick commit off spec-089 dirty lists;
   frontmatter stat blocks; staleness stamps; absence blocks; dulwich commits at
   sim-time.
4. **TUI shell** — Textual boot, §9b ksbc theme tokens, `MarkdownFence` directive
   dispatch + wikilink rule, `babylon://` router, snapshot harness.
5. **Fixture recorder** — projection outputs recorded from golden runs so every
   downstream view task runs against fixtures: no DB, no engine.

**Exit:** one entity kind (county) end-to-end — tick → projection → baked page →
rendered → snapshot golden.

**P2 — Fan-out lanes** (parallel; work-order headers + [P]/[S] tags per roadmap §4):
Lane P pages (one task per entity kind); Lane W plates/widgets (peek, verb plate with
`preview_action`, Chronicle, palette Provider, watchlist, statblock/absence/narrative);
Lane T topology (PAOH, Levi ego-tree, incidence/adjacency matrices, map room — cell-art
tier first, TGP raster behind a capability flag; read-only orderings only); Lane E
engine-adjacent, serial (verb submission, intel ledger/INVESTIGATE, veil tier gating,
narrator cache, epistemic search); content lane (concept cards, briefing dossier, five
epilogue pages).

**P3 — Assembly (serialize):** navigation shell (jumplist, breadcrumbs, fuzzy switcher),
salience/dedup/autopause on the Chronicle, load/new campaign menu (the honest size of
"lobby/briefing" per local-first §3.2), unaided-first-action Pilot e2e.

**P4 — Ceremony + cutover.** Golden-vault seeding across the qa:regression scenarios
(**the program's one declared ceremony**). Then the ratified cutover gate:

1. Test-port ledger closed — every Playwright behavioral assertion mapped to a projection
   contract test, a Pilot test, or the golden vault.
2. Unaided-first-action e2e green in the TUI.
3. The BD completes a full campaign session in the TUI.
4. Golden-vault byte-gate green in CI.

Then `src/frontend/` is deleted in one commit; `web/` demotes to what is verifiably
load-bearing (the thin ingest API per the local-first doc §4.1).

**Post-cutover (separate parcels, recorded not scheduled):** Metropole parcel [P]
(ingest API, telemetry schema, blob vault, Grafana boards, key issuance) and Periphery
parcel [S-heavy] (embedded cluster manager, template/catalog, outbox at tick commit,
uploader, Alembic-on-load, packaging) — the latter discharges X.8.

## Interleaving & machine safety

At most **one engine train + one Archive train** after the keel; cross-train surfaces
behind one narrow named helper + a loud behavioral contract. Vol I ∥ Vol II is the
sanctioned engine train (gates green as of tonight). Heavy runs single-flight; parallel
agents read-only or worktree-isolated with scoped `test:q`; fixture-first for all view
tasks.

## P0 exit batch — RESOLVED (BD, 2026-07-20; binding)

1. **Vault slugs:** stable IDs as paths (`county/26163.md`); readable name lives in
   frontmatter + page title. Rename-proof, grep-friendly.
2. **Epistemic `/` search:** FTS over org-known entities only; the unknown surfaces
   *only* as red links from known pages — discovery is Investigate-gated; there is no
   global oracle.
3. **Watchlist / jumplist / breadcrumbs:** persist per-campaign in `babylon_meta` — the
   epistemic store, never in the tick hash (fog ruling).
4. **Narrator doctrine-conditioning:** deferred past v1 (S5 stays optional).
5. **`/explain` + Observatory:** absorbed as Archive pages (concept cards + formula
   terminals).
6. **Map-room tiers:** cell-art choropleths at EA/state tiers; TGP raster for
   county-resolution fills; braille reserved for line-work.

**Embedding pin:** mechanism ratified in ADR099 (per-campaign pin + dimensionality in
campaign metadata, never mixed); the concrete default model is named at P1 with in-env
evidence. **Stack ADR099:** ratified as drafted.

Deferred by design: multiplayer/remote (S2 makes it transport reinsertion); ssh serving;
AAR static-site export builder choice (Sphinx custom domain, export path only);
telemetry payload-schema ADR + beta agreement text (with the Metropole parcel).
