# Program 24 — The Archive: Work Orders P2 · P3 · P4

**Status:** P1 Keel MERGED (2026-07-20). This document schedules everything from P2 fan-out through P4 cutover.
**WO numbering:** continues the keel sequence (P1 consumed WO-1…WO-15); P2 begins at WO-16.
**Binding canon:** `project/programs/24-the-archive.md` (charter), `ai/_inbox/tui/20260719archiveinterfacedesign.md` (design canon, S/R rulings), `docs/reference/projection-registry.rst` (II.11), Constitution v2.12.0 Amendments V/W/X/D, `CLAUDE.md` machine-safety.
**Global invariants (every WO):** engine value-drift ZERO — `mise run qa:regression` byte-identical except the single P4 vault ceremony; fixture-first for all view WOs (no DB/engine in view tests); Amendment D untriggered (Lane T = read-only orderings only); agents run scoped venv-shadow tests only (`env PYTHONPATH= .venv/bin/python -m pytest <paths> -q`), NEVER `mise run test:unit`, heavy runs single-flight.
**Extension recipe reference:** the 13-step "new entity-kind page" order in the keel-api digest governs every Lane P WO; do not deviate.

---

## Shared-file discipline (read before Wave 1)

Four files are **append-only zipper-merge points** touched by multiple parallel WOs. Parallel agents MUST keep edits additive; the integrator resolves zippers (MEMORY: "git ZIPPERS both-appended blocks"). The bulk of every WO is collision-free new files.

| Shared file | What each WO appends | Collision class |
|---|---|---|
| `src/babylon/projection/view_models.py` | one `<Kind>View` + widen `ProjectionRecord` union | append-only zipper |
| `src/babylon/projection/registry.py` | one `DeclaredView` const + `REGISTRY` tuple entry (SQL-backed kinds only) | append-only zipper |
| `src/babylon/tui/directives.py` | one `_directive_<name>` method on `BabylonFence` (auto-dispatched) | append-only zipper |
| `src/babylon/tui/app.py` | `KNOWN_ENTITIES` membership + `StatblockProvider` dispatch | **serialized to Wave-2 WO-45** — Wave-1 WOs do NOT touch app.py |

Design rule enforced here: Wave-1 page/widget WOs deliver a **per-kind statblock provider function inside their own module** and register nothing in `app.py`. The single serial WO-45 composes them. This is what makes Wave 1 genuinely parallel.

---

# PHASE P2 — Fan-out lanes

## WAVE 1 — maximally parallel, worktree-isolated, fixture-first

All Wave-1 WOs are **AGENT-SAFE (worktree-parallel)**. Each: branch from `dev`, worktree-isolated, scoped venv-shadow tests only. None touches the engine; none touches `app.py`.

### Lane P — pages (one WO per entity kind)

Every Lane P WO follows the keel-api 13-step recipe and mirrors `projection/county.py` exactly: a `project_<kind>(id, *, graph, world, tick) -> <Kind>View` with a one-producer-per-field docstring table, absence-before-call `None` conversion, a frozen `extra="forbid"` `<Kind>View` with `kind: Literal[...]` discriminator, a `vault/templates/<kind>.md.j2` (fenced-directive-only), a `render_<kind>` + `_REMEDY_BY_FIELD`-equivalent, a `bake_<kind>` on `VaultMaterializer` (path `<kind>/<id>.md`), a `record_<kind>_fixture`/`load_<kind>_fixture` pair, and a snapshot test.

---

**WO-16 · Lane P: State dossier page · [P]**
- **Frame:** County+Nesting frame (state = territory tier). Design canon R7 (Victoria-3 nesting), S1 (vault-as-contract).
- **Deliverables:** `projection/state.py` (`project_state`); `StateView` in `view_models.py` (append); `DeclaredView` for existing `v_state_value_aggregate` already in `REGISTRY` — reuse, no new registry row unless new columns needed; `vault/templates/state.md.j2`; `render_state` in `vault/render.py` (or `vault/render_state.py`); `bake_state` on `materializer.py`; `record_state_fixture`/`load_state_fixture` in `fixtures/recorder.py`; per-kind statblock provider `state_statblocks` in `projection/state.py`.
- **Contract tests:** `tests/unit/projection/test_state.py` (pins one-producer-per-field table, absence→None); `tests/unit/projection/vault/test_render_state.py` (pins statblock rows + absence blocks + frontmatter); `tests/unit/tui/snapshots/test_state_page.py` (pytest-textual-snapshot, fixture-fed).
- **Depends on:** none.
- **Canon:** keel-api extension recipe; registry row `v_state_value_aggregate` (order_by `session_id, tick, state_fips`).

**WO-17 · Lane P: National dossier page · [P]**
- **Frame:** County+Nesting (national tier).
- **Deliverables:** as WO-16 for `national`; reuse existing `v_national_value_aggregate` registry entry; `projection/national.py`, `NationalView`, `national.md.j2`, `render_national`, `bake_national`, fixtures.
- **Contract tests:** `tests/unit/projection/test_national.py`; `tests/unit/projection/vault/test_render_national.py`; `tests/unit/tui/snapshots/test_national_page.py`.
- **Depends on:** none.
- **Canon:** keel-api recipe; `v_national_value_aggregate`.

**WO-18 · Lane P: Organization page · [P]**
- **Frame:** County+Nesting (org dossier). Design canon S1; fog note: orgs carry `ORG_POLITICAL_FIELDS` (internal state political for non-player orgs).
- **Deliverables:** `projection/organization.py`; `OrganizationView`; `organization.md.j2`; `render_organization`; `bake_organization`; fixtures; `org_statblocks`. Field split for fog (material: existence/public activity/territorial presence; political: `consciousness_tendency`, `cohesion`, `cadre_level`) recorded in docstring — actual fog gating is Lane E WO-41 (do NOT wire `apply_fog` here; fixture carries pre-gated data).
- **Contract tests:** `tests/unit/projection/test_organization.py`; `tests/unit/projection/vault/test_render_organization.py`; `tests/unit/tui/snapshots/test_org_page.py`.
- **Depends on:** none.
- **Canon:** keel-api recipe; fog `ORG_POLITICAL_FIELDS` (`fog/filter.py`).

**WO-19 · Lane P: Institution page · [P]** — same shape; `projection/institution.py`, `InstitutionView`, factional-composition sub-model (liberal_technocratic/revanchist_fascist/institutionalist_bonapartist per `InstitutionSerializer`). Tests: `tests/unit/projection/test_institution.py` + render + snapshot. Depends: none.

**WO-20 · Lane P: Sovereign page · [P]** — `projection/sovereign.py`, `SovereignView`; sovereign is the CLAIMS-edge target already resolved in `project_county._single_claimant`. County pages emit `[[sovereign/<id>]]` wikilinks (keel `county.md.j2`) — this WO makes those links resolve to real pages. Tests: `test_sovereign.py` + render + snapshot. Depends: none.

**WO-21 · Lane P: Key figure page · [P]** — `projection/key_figure.py`, `KeyFigureView` (node type `key_figure`). Tests as pattern. Depends: none. Note: verify `key_figure` producer exists in graph before claiming fields (vocabulary sentinel `mise run check:vocabulary`); if the type has no live producer, the WO ships an honest-absence page + flags the dead producer.

**WO-22 · Lane P: Industry page · [P]** — `projection/industry.py`, `IndustryView` (node type `industry`; imperial-rent/Leontief data from `domain/economics/`). Tests as pattern. Depends: none.

**WO-23 · Lane P: Social class page · [P]** — `projection/social_class.py`, `SocialClassView`; reuse `ClassComposition`/`ConsciousnessSimplex` view-models from keel `view_models.py`. Tests as pattern. Depends: none.

**WO-24 · Lane P: Community / hyperedge dossier page · [P]**
- **Frame:** Topology frame. Design canon S9 default idiom ("a hyperedge is a community dossier page: roster, formation tick, overlaps"). **Amendment D: read-only — roster/formation-tick display only, no mutation affordance.**
- **Deliverables:** `projection/community.py` (`project_community` → roster + formation_tick + overlaps); `CommunityView`; `community.md.j2` (roster as `[[social_class/<id>]]` wikilinks = incidence-via-backlinks); `render_community`; `bake_community`; fixtures.
- **Contract tests:** `tests/unit/projection/test_community.py` (roster ordering deterministic); render + snapshot. Snapshot pins that membership renders as backlinks (S9 "backlinks = incidence").
- **Depends on:** none.
- **Canon:** design-canon S9; MEMORY hex/community Lawverian (community NEVER a graph node — read the hyperedge/community as a projection, not a node lookup).

### Lane W — plates / widgets

**WO-25 · Lane W: `peek()` single-renderer · [P]**
- **Frame:** County+Nesting (nested peek). Design canon S7 (one peek mechanism → tooltips, hover preview, transclusion, watchlist rows), R7.
- **Deliverables:** `src/babylon/tui/peek.py` — `peek(entity_view, depth: int) -> RenderableType` producing a compact stat plate from ANY `ProjectionRecord` view-model (dispatch on `.kind`), reused at 4 sizes via `depth`. Keyboard-first; mouse hover supported, never load-bearing. Live-query surface (NOT baked — contrast S3), so it consumes view-models directly, not vault pages.
- **Contract tests:** `tests/unit/tui/test_peek.py` (pins depth→size mapping, dispatch on kind, absence rendering); `tests/unit/tui/snapshots/test_peek_plate.py` (fixture-fed, Wayne 26163 @ T=0847, wages 21.40→19.85).
- **Depends on:** none (dispatches on view-model `.kind`; new kinds register no code here — extension is data-driven).
- **Canon:** S7; §9b newt plate chrome.
- **Open gap flagged:** no depth-limit/recursion-cycle rule in design brief — WO adopts a hard `depth<=3` bound (Power-of-10 rule 2, statically provable) and documents it.

**WO-26 · Lane W: verb plate widget · [P]**
- **Frame:** Verb Plate frame (exact 132×43). Design canon S6, Constitution Article V (nine verbs, atomic, always available), VII.8 feedforward.
- **Deliverables:** `src/babylon/tui/verb_plate.py` — renders the nine Article V verbs legal for a target from a `VerbPlateView` view-model (fixture-fed here; live provider = Lane E WO-38), OODA-gated legality + costs shown, deterministic consequence preview block (`preview_action` output). Investigate's three sub-verbs surface faithfully (not collapsed). newt-dialog chrome. **Emits structured verbs only — no free text, no direct graph mutation (R4).**
- **Contract tests:** `tests/unit/tui/test_verb_plate.py` (all 9 verbs render; ineligible verbs show reason not hidden; costs from view-model); `tests/unit/tui/snapshots/test_verb_plate.py` (fixture: Wayne, tick 0, all 9 eligible via TENANCY — mirrors `verb-submit.spec.ts` corrected assertion).
- **Depends on:** none for the widget (fixture-fed); live data from **WO-38**.
- **Canon:** S6; Article V; VII.8. Modes: Textual MODES for READ/PEEK/VERB (Textual 8.2.8 digest).

**WO-27 · Lane W: Chronicle plate widget · [P]**
- **Frame:** Chronicle frame. Design canon S8 (tick bulletins as dated pages, daily-note IS the tick; event ledger = browsable stream).
- **Deliverables:** `src/babylon/tui/chronicle.py` — renders a per-tick event stream from a fixture list of events. Reuse actor-resolution prior art conceptually from `web/game/narrator.py` `_subject_from_class_id`/`_subject_from_org_id` (port the helpers, don't re-derive) since `WorldState.events` has no unified actor field. Pagination ceiling 200 rows newest-first (reuse `query_session_events(limit=200)` convention — chronicle-events digest §6). Salience/dedup/autopause are P3 (WO-48) — this widget renders raw stream only.
- **Contract tests:** `tests/unit/tui/test_chronicle.py` (per-tick grouping, actor resolution, empty tick → honest "wire is quiet" per III.11); `tests/unit/tui/snapshots/test_chronicle.py`.
- **Depends on:** none (fixture-fed).
- **Canon:** S8; chronicle-events digest §5 (event→text prior art), §6 (volume/pagination).

**WO-28 · Lane W: command palette Provider · [P]**
- **Frame:** design canon R2 (Obsidian/neovim command palette + fuzzy switcher).
- **Deliverables:** `src/babylon/tui/palette.py` — a Textual `Provider` (search/discover per Textual 8.2.8 digest) exposing navigation + read-only commands over the known-entity set. Fuzzy match. **No verb commands in the palette (R4 keeps verbs in the plate).** Nav-shell integration (jumplist/breadcrumbs) is P3 WO-47.
- **Contract tests:** `tests/unit/tui/test_palette.py` (search ranks known entities; discover lists commands); `tests/unit/tui/snapshots/test_palette.py`.
- **Depends on:** none.
- **Canon:** Textual Provider search/discover; binding precedence focused-widget-first.

**WO-29 · Lane W: statblock / absence / narrative directive hardening · [P]**
- **Frame:** design canon S4 (honest absence), S5 (narrator byline), County+Nesting page anatomy.
- **Deliverables:** contract-test and harden the three keel directives (`_directive_statblock`, `_directive_absence`, `_directive_narrative` in `tui/directives.py`) against real view-model-derived inputs across all Wave-1 kinds. Baked-page path (parse `key: value`) + live path (`StatblockProvider`) both covered. Malformed line → absence Label refusal (III.11). Narrator block byline typography (II.5 provenance-as-typography). **No new directive method here** (those are Lane T WO-30/31/32) — this is robustness + tests only, so no `directives.py` collision with Lane T.
- **Contract tests:** `tests/unit/tui/test_directives_hardening.py` (baked vs live statblock; absence-block remedy text; narrative byline + cache-key surface `cached:{tick}:{model_pin}`); snapshot deltas.
- **Depends on:** none.
- **Canon:** S4/S5; keel `tui/directives.py` `BabylonFence`.

### Lane T — topology surfaces (READ-ONLY orderings only; Amendment D untriggered)

Each Lane T WO adds ONE `_directive_<name>` method to `BabylonFence` (append-only zipper on `directives.py`) plus a pure renderer + ordering provider. **No layout, no force-directed — orderings only, deterministic and snapshot-testable (S9).**

**WO-30 · Lane T: PAOH ordering provider + directive · [P]**
- **Frame:** Topology frame. Design canon S9 PAOH ("needs an ordering, not a layout").
- **Deliverables:** keel already ships `parse_paoh_body`/`render_paoh`/`_directive_paoh` in `directives.py`. This WO adds `projection/topology/paoh.py` — `paoh_ordering(community_views) -> (nodes_in_order, edges_sorted_by_formation_tick)` deriving the ordering from community/hyperedge projection data (formation-tick column order per S9). Wire the ordering into a real page fixture.
- **Contract tests:** `tests/unit/projection/topology/test_paoh.py` (deterministic ordering, formation-tick sort, `ORDER BY` explicit); `tests/unit/tui/snapshots/test_paoh_render.py`.
- **Depends on:** WO-24 (community projection shape).
- **Canon:** S9 PAOH; III.13 (explicit ordering).

**WO-31 · Lane T: Levi / bipartite ego-tree · [P]**
- **Deliverables:** `_directive_egotree` on `BabylonFence`; `tui/topology/egotree.py` renderer + `projection/topology/levi.py` ordering that walks the bipartite (node↔hyperedge) storage structure (S9: "visualization walks the storage structure" — mirrors hypergraph-rs bipartite rep, interim provider = current xgi/rustworkx layer).
- **Contract tests:** `tests/unit/projection/topology/test_levi.py`; `tests/unit/tui/snapshots/test_egotree_render.py`.
- **Depends on:** WO-24.
- **Canon:** S9 Levi/bipartite; R8.

**WO-32 · Lane T: incidence / adjacency matrix · [P]**
- **Deliverables:** `_directive_matrix` on `BabylonFence`; `tui/topology/matrix.py` renderer (cell-art grid) + `projection/topology/incidence.py` ordering. Makes Constitution I.21 centrality/singleton/cutset targeting modes legible (READ-ONLY).
- **Contract tests:** `tests/unit/projection/topology/test_incidence.py`; `tests/unit/tui/snapshots/test_matrix_render.py`.
- **Depends on:** WO-24.
- **Canon:** S9 incidence/adjacency; Constitution I.21 (verify exact clause).

**WO-33 · Lane T: map room — cell-art choropleth (+ TGP raster behind capability flag) · [P]**
- **Frame:** Map Room frame (exact 132×43). Design canon S9 map-room; charter P0 batch tier ruling: **cell-art at EA/state, TGP raster at county-resolution, braille reserved for line-work.**
- **Deliverables:** `tui/map_room.py` — choropleth over level-lattice tiers; cell-art (half-block) floor for EA/state tiers; kitty-graphics-protocol (TGP) raster for county tier **behind a runtime capability flag** (Kitty-detected; default OFF → cell-art fallback). `projection/topology/choropleth.py` reads `v_hex_state_asof` (SPARSE — read the view, never `WHERE tick=N`; MEMORY gotcha) and county aggregates. `_directive_maproom` on `BabylonFence`.
- **Contract tests:** `tests/unit/tui/snapshots/test_map_room_cellart.py` (cell-art floor, capability flag OFF — deterministic, no raster); `tests/unit/projection/topology/test_choropleth.py` (tier→renderer selection; sparse-view read). TGP raster path is NOT snapshot-gated (non-deterministic image bytes) — behind flag, manual Kitty eyes-on check.
- **Depends on:** none (reads existing hex views via fixtures).
- **Canon:** S9 map-room; charter P0 tier batch; `v_hex_state_asof` (spec-089).

### Content lane

**WO-34 · Content: epilogue pages · [P]**
- **Frame:** Epilogue frame. Design canon Export/S11; vault-tick digest §5.
- **Deliverables:** port `web/game/epilogues.py` `EPILOGUES: dict[str, Epilogue]` (engine-agnostic string-keyed data — directly reusable, NOT re-authored) into `projection/vault/epilogues.py` + one vault page per outcome (`epilogue/<outcome>.md`). Copy source-of-truth: the six terminal texts keyed by `GameOutcome` minus `IN_PROGRESS`. **5-vs-6 discrepancy is an OPEN QUESTION** (charter says "five epilogue pages"; `GameOutcome` has 6 non-`IN_PROGRESS` members incl. `UNRESOLVED`) — WO ships all six page shells and flags for owner ruling; the `UNRESOLVED` horizon-check logic home is WO-39/Lane E.
- **Contract tests:** `tests/unit/projection/vault/test_epilogues.py` (each outcome renders verbatim copy — pins the `UNRESOLVED` "THE STRUGGLE CONTINUES…" body exactly as `first-session.spec.ts` did); snapshot.
- **Depends on:** none (data port).
- **Canon:** vault-tick §5 (EndgameDetector 5-axis + GameOutcome 7-member); test-port `first-session.spec.ts` epilogue-text contract.

**WO-35 · Content: briefing dossier page · [P]**
- **Frame:** Lobby/Briefing frame. **NET-NEW design** — design-canon Part 4: "briefing dossier" is NOT in the design brief; closest analogue is the county Dossier shape. Requires owner design authorization (OPEN QUESTION).
- **Deliverables:** port `web/game/codenames.py` `operation_codename(session_id)` (deterministic two-word) + `get_journal_objectives` (5 patterns) into a `projection/briefing.py` + `briefing.md.j2`. Content matches `lobby-briefing.spec.ts` acceptance (codename regex `OPERATION [A-Z]+ [A-Z]+`, 5 pattern rows, win-badge, "100 years" horizon).
- **Contract tests:** `tests/unit/projection/test_briefing.py` (codename determinism, 5 objectives); snapshot.
- **Depends on:** none.
- **Canon:** test-port `lobby-briefing.spec.ts` (NOT in ledger — this WO closes that gap); design-canon Part 4 (net-new flag).

**WO-36 · Content: concept cards · [P]**
- **Frame:** none exists. **NET-NEW design** — design-canon Part 4: no "concept card" idiom in the brief; closest cousin is the peek plate (entity-scoped, not concept-scoped). Requires owner design authorization + a DESIGN_BIBLE page-anatomy section (OPEN QUESTION).
- **Deliverables:** `projection/vault/concept_cards.py` + `concept/<slug>.md` pages for game-concept reference (Fundamental Theorem, Survival Calculus, Metabolic Rift, the nine verbs) sourced from `docs/reference/*.rst`. Absorbs `/explain` (Program 17) + Observatory panes per charter P0 batch ("absorbed as Archive pages").
- **Contract tests:** `tests/unit/projection/vault/test_concept_cards.py`; snapshot.
- **Depends on:** owner design ruling (see OPEN QUESTIONS).
- **Canon:** design-canon Part 4 (net-new); charter P0 batch (`/explain`+Observatory absorption).

## WAVE 1b — parallel, single Wave-1 dependency

**WO-37 · Lane W: watchlist page · [P]**
- **Frame:** design canon S7 ("pinned watchlist = a page of transclusions", watchlist-row peek size).
- **Deliverables:** `tui/watchlist.py` — a page built from `peek(view, depth=0)` rows (watchlist size). Live-query-backed, not baked. Pin/unpin UX + capacity + ordering are §10-unresolved in the brief but charter P0 batch resolved persistence → `babylon_meta` (created P3 WO-46); until then watchlist is session-in-memory with a clear seam to `babylon_meta`.
- **Contract tests:** `tests/unit/tui/test_watchlist.py`; `tests/unit/tui/snapshots/test_watchlist.py`.
- **Depends on:** WO-25 (peek).
- **Canon:** S7; persistence seam → WO-46.

---

## Lane E — engine-adjacent (STRICTLY SERIAL within lane; main-loop-serial)

**AGENT-SAFE? NO — main-loop-serial.** These touch engine seams, determinism, and (WO-44) the runner. Run in the main loop, single-flight, one at a time in listed order. Zero engine value-drift: WO-44's observer is None-default and qa:regression-byte-identical with it in place (proven at keel).

**WO-38 · Lane E: `preview_action` projection + `VerbPlateView` provider · [S]**
- **Deliverables:** port the **pure** helper `_preview_consciousness_delta` (currently `web/game/engine_bridge.py:6999-7111` region) into the projection layer as `projection/verbs/preview.py`, preserving `preview == resolution` math (the same helper the real resolvers use — lane-e-seams §2). Produce `VerbPlateView` (view-model) listing the 9 verbs, OODA-gated legality, costs from verb registry + `GameDefines`, and per-verb preview deltas. Feeds WO-26 live.
- **Contract tests:** `tests/unit/projection/verbs/test_preview.py` — pins `preview == resolution` for `{educate, campaign, aid}` (the per-target-resolver verbs); heuristic verbs pinned to their documented simpler deltas (lane-e-seams §2). Reuse `test_engine_bridge.py::TestExpectedDeltas` assertions as the port target (test-port ledger — currently undispositioned, this WO dispositions it).
- **Depends on:** none within Lane E (first in chain).
- **Canon:** lane-e-seams §2; sentinel `src/babylon/sentinels/seam/registry.py:2295-2329`.

**WO-39 · Lane E: verb submission path · [S]**
- **Deliverables:** TUI-side structured-verb submission that enters the engine's action queue. In the legacy path this is `bridge.submit_action → persistence.submit_turn → game_turn` consumed by `OODASystem` (lane-e-seams §1). For the in-process TUI (design canon S2), build `projection/verbs/submit.py` that writes the action to the runtime queue (Postgres `game_turn` today; `babylon_meta`-adjacent per-campaign DB later) and returns a turn id — **no direct graph mutation, verbs resolve via OODASystem next tick**. Also relocate the `UNRESOLVED` horizon-check (`horizon_tick = campaign_horizon_years * weeks_per_year`, `outcome = pattern or UNRESOLVED`) out of the legacy web bridge into an Archive-side home here (vault-tick §5 requires this so the TUI determines game-over independently of the dead web bridge).
- **Contract tests:** `tests/unit/projection/verbs/test_submit.py` (structured payload shape, affordability gate, one row per (session,tick,org)); `tests/integration/archive/test_verb_resolution.py` (submit → tick → OODASystem folds `player_actions` → `turn_resolution`). Ports `test_engine_bridge.py::TestEngineBridgeActions`/`TestActionInjection`/`TestActionResultPersistence`.
- **Depends on:** WO-38.
- **Canon:** lane-e-seams §1; charter Article V atomicity; vault-tick §5 (UNRESOLVED home).

**WO-40 · Lane E: intel ledger / INVESTIGATE wiring · [S]**
- **Deliverables:** the fog `IntelLedger`/`read_intel`/`ledger_from_events` already live import-pure in `projection/fog/ledger.py`. Wire the write side (`_investigate_field_snapshot`-equivalent: on successful INVESTIGATE, freeze TRUE post-tick values, `field_group = political_field_group(node_type)`) and the read side (`_derive_intel_ledger`-equivalent: fold `action_result` rows deterministically) into the projection/TUI path. Result: INVESTIGATE's three sub-verbs actually open intel; absence blocks that say "Investigate(Territory) to open one" become live affordances (design-canon §5 Jinja example).
- **Contract tests:** `tests/unit/projection/fog/test_investigate_wiring.py` (write freezes true values; read reconstructs deterministically after "restart"); ports `test_engine_bridge.py::TestDeriveIntelLedger`/`TestDeriveIntelLedgerWithoutDjangoDb`/`TestResolveTickPersistsInvestigateSnapshot`/`TestInvestigateFieldSnapshot`.
- **Depends on:** WO-39 (submission produces the `action_result` rows this folds).
- **Canon:** lane-e-seams §3; keel-api `fog/ledger.py`.

**WO-41 · Lane E: veil-tier gating in projection reads · [S]**
- **Deliverables:** unify the read-side gating for TUI projections. Three distinct systems exist (lane-e-seams §4): (a) fog spatial exact/approx/unknown (`apply_fog`), (b) class-vision desert/mud/water (`_apply_class_vision_gate`), (c) doctrine "Veil of Money" tier 0/1/2 (`web/game/veil.py`). Port (a) — already projection-pure — and (c) into `projection/veil.py`, and RESOLVE the documented collision hazard (`fog/filter.py:11-17`: running fog + class-vision on one payload is an unresolved correctness hazard) by defining explicit precedence/sequencing in code + a contract test. Apply to org/territory/inspector projections consumed by Lane P pages.
- **Contract tests:** `tests/unit/projection/test_veil_gating.py` (fog reach-wins precedence; veil tier masks `TIER1_VALUE_RELATION_FIELDS`/`TIER2_SCISSORS_FIELDS`; **the two-gates-on-one-payload case is now well-defined, not hazardous**); ports `test_veil`, `test_engine_bridge.py::TestHexFeaturePropertiesVeilGate`/`TestDerivedEconomyVeilGate`.
- **Depends on:** WO-40.
- **Canon:** lane-e-seams §4; test-port `veil.spec.ts` (3 veil-locked instruments contract).

**WO-42 · Lane E: narrator cache · [S]**
- **Deliverables:** `prose_cache_key(entity_id, tick, model_pin)` exists but is **dead code** (lane-e-seams §5). Wire the narrator attributed-block cache keyed `(entity, tick, model_pin)` (III.6) — async side process writing `{narrative}` blocks into vault pages (design-canon S5, arch diagram line 79). Persist to an Archive store; reuse `NarratorProvider` seam (`babylon/intelligence/providers.py`) + `MuteProvider` (R4: fully informative narrator-OFF). Doctrine-conditioning DEFERRED past v1 (charter P0 batch).
- **Contract tests:** `tests/unit/projection/vault/test_narrator_cache.py` (cache key structure; narrator-OFF page fully informative; `MuteProvider` legal path; model-pin survives deprecation); ports `test_narration_record`/`test_narrative_service`/`test_narrator`.
- **Depends on:** WO-41.
- **Canon:** lane-e-seams §5; design-canon S5; III.6.

**WO-43 · Lane E: epistemic search — FTS over org-known only · [S]**
- **Deliverables:** **zero implementation exists** (lane-e-seams §6 — `fts_columns` is declared metadata only; no `to_tsvector`/`fts5` anywhere). Build (a) an FTS execution layer over the `fts_columns`-declared views (Postgres FTS, `fog = WHERE clause` per ADR099 spike finding) OR an in-process filter given small entity counts; (b) replace `known_target_resolver`'s demo `KNOWN_ENTITIES` input with a **real per-org known-entity set = `organizing_reach()` (spatial) ∪ `_derive_intel_ledger()` observed-node ids (INVESTIGATE history)**. `/` searches ONLY what the org knows; unknown → red links; **no global oracle** (charter P0 batch).
- **Contract tests:** `tests/unit/projection/test_epistemic_search.py` (search returns only reach∪intel entities; fogged entity not surfaced; unknown → redlink); `tests/unit/tui/test_search_resolver.py`.
- **Depends on:** WO-40 (intel ledger), WO-41 (reach/veil).
- **Canon:** lane-e-seams §6; charter P0 batch (FTS-over-known-only); design-canon §10 (search semantics).

**WO-44 · Lane E: CLI vault wiring — P4 PREREQUISITE · [S]**
- **Frame:** vault-tick §2/§4. **This is the hard prerequisite for the P4 ceremony — without it, P4 cannot execute** (vault-tick §4.5).
- **Deliverables:** (1) `SimulationRunConfig.vault_root: Path | None` + `--vault-root` CLI flag (`argparse_cli.py`); (2) in `run()`, construct `CountyTickBaker(VaultMaterializer(vault_root), county_fips=config.scope_fips)` and pass as `tick_commit_observer=` into `_tick_loop` (currently a zero-line no-op — `run()` never wires the observer, vault-tick §2 CRITICAL FINDING); (3) **fix the tick-0 bake gap** — line 1577's bare `bridge.persist_tick(world, 0, ...)` bypasses the observer, so tick-0 state is never baked (vault-tick §2); (4) generalize `CountyTickBaker` → a per-kind baker composition (Lane P kinds now exist); (5) **vault-at-scale**: add content-hash-diff skip in the materializer (skip `commit_page` when rendered Markdown is byte-identical to last-baked) AND batch one-commit-per-tick (all changed pages in a single dulwich commit, not N commits) — national scope is otherwise ~1.64M commits (vault-tick §2/§3). Wire `config.scope_fips` (already resolves national ~3,156 lazily via `scopes.py`).
- **Contract tests:** `tests/unit/engine/headless_runner/test_vault_wiring.py` (run() passes observer; tick-0 baked; content-hash skip; one commit/tick); `tests/integration/archive/test_vault_run_e2e.py` (real `--vault-root` run over `single_county`, byte-identical repo across two independent runs — leans on keel's `test_two_independent_bakes_at_the_same_tick_are_byte_identical_commits`). **qa:regression must stay byte-identical** (observer None-default when `--vault-root` absent).
- **Depends on:** WO-16…WO-24 (per-kind bakers to generalize over), WO-42 (narrator blocks in baked pages).
- **Canon:** vault-tick §1 (dirty-list), §2 (runner gap), §3 (dulwich), §4 (P4 slot-in); Amendment W (III.13).

---

## WAVE 2 — serial integration (unblocks live rendering)

**WO-45 · TUI provider dispatch + app wiring · [S]**
- **Deliverables:** the single WO that edits `tui/app.py`. Generalize `StatblockProvider` from the literal `"county/26163"` branch (`_sample_statblocks`) to a **kind-dispatch registry** consuming every Lane P per-kind provider (WO-16…24). Compose `KNOWN_ENTITIES` from all page WOs' id sets. Register all new directives (already auto-dispatched via `BabylonFence` — verify) and confirm `BabylonMarkdown.BLOCKS` covers wikilink blocks. Replace the sample resolver with the WO-43 epistemic known-set (reach∪intel).
- **Contract tests:** `tests/unit/tui/test_app_dispatch.py` (every kind resolves a statblock provider; unknown kind → absence; known-set = reach∪intel); full-app snapshot across kinds.
- **Depends on:** ALL Lane P (WO-16…24), ALL Lane T directives (WO-30…33), WO-43 (known-set).
- **AGENT-SAFE? NO — serial integrator** (single shared-file edit resolving all Wave-1 additive deliverables).
- **Canon:** keel-api step 10; shared-file discipline table above.

---

# PHASE P3 — Assembly (SERIALIZE; after all P2)

All P3 WOs: **[S], main-loop-serial.** Interleaving rule: at most one engine train + one Archive train; P3 is the Archive train.

**WO-46 · P3: `babylon_meta` epistemic store — CREATE · [S]**
- **Frame:** meta-store digest (LOUD FINDING: `babylon_meta` does NOT exist — zero schema, zero migration, zero model). charter P0 ruling 3.
- **Deliverables:** **CREATE** the store — it must be built, not assumed. Per meta-store digest, scope decision is an OPEN QUESTION (minimal catalog against dev two-DB layout NOW vs pull forward the post-cutover embedded-cluster Periphery parcel). Default recommendation absent owner ruling: minimal `babylon_meta`-shaped tables against the existing runtime DB — campaign catalog (uuid, slug, engine version, defines hash, last tick, last played) + `watchlist`/`jumplist`/`breadcrumb` session-scoped tables. DDL in a new migration + `postgres_schema.py` (the single DDL source-of-truth). **Epistemic tier, NEVER in tick hash** (fog-epistemic-vs-material; MEMORY "always ask who WRITES a store" — the client writes this, not the engine).
- **Contract tests:** `tests/unit/persistence/test_babylon_meta.py` (schema present, idempotent DDL, catalog CRUD, watchlist/jumplist persistence); **assert these tables are absent from the tick determinism hash**.
- **Depends on:** WO-37 (watchlist seam), WO-45.
- **Canon:** meta-store digest; charter P0 ruling 3 (`babylon_meta` mandated); charter Post-cutover Periphery parcel (scope boundary).

**WO-47 · P3: navigation shell · [S]**
- **Deliverables:** jumplist (`Ctrl-O`/`Ctrl-I` vim back-stack), breadcrumbs (trail), fuzzy switcher (wires WO-28 palette Provider). Persist jumplist depth + breadcrumb cross-session state to `babylon_meta` (charter P0 ruling 3 — resolves design-canon §10 open question). Enter follows links; pinned watchlist = a page of transclusions (WO-37).
- **Contract tests:** `tests/unit/tui/test_nav_shell.py` (jumplist push/pop, breadcrumb trail, fuzzy match); `tests/integration/tui/test_nav_persistence.py` (survives restart via `babylon_meta`).
- **Depends on:** WO-46, WO-28, WO-37, WO-45.
- **Canon:** design-canon S7 (jumplist/breadcrumbs), R2 (fuzzy switcher); §10 (persistence).

**WO-48 · P3: Chronicle salience / dedup / autopause · [S]**
- **Frame:** Chronicle frame. **NET-NEW mechanics** — design-canon has NO salience/dedup/autopause spec (design-canon Part 2 explicit gap); reuse legacy prior art.
- **Deliverables:** on WO-27's Chronicle plate: (1) **salience** — reuse `_EVENT_SEVERITY` (47/84 classified: 14 critical/20 warning/13 info; 37 default informational — chronicle-events §6) but FIX the degrade-quiet default to a loud-surface for unclassified new `EventType`s (III.11), and DO NOT copy the frontend's UPPERCASE-key casing bug (chronicle-events §6 known gap — real values are lowercase snake_case); (2) **dedup** — no two CONSECUTIVE cards share `${type}:${subject}` key (test-port `first-session.spec.ts` contract; adjacent same-type/different-subject allowed); (3) **autopause** — wire the reserved `AMBER` autopause token (keel `theme.py`, "NOT wired to any widget yet") to critical-event autopause. Volume floor: narrative capped 1/tick (chronicle-events §6), `ORGANIZATIONAL_ACTION` aggregated to 1/tick (ADR086).
- **Contract tests:** `tests/unit/tui/test_chronicle_salience.py` (severity tiers; consecutive-dedup; autopause fires on critical; unclassified type surfaces loud not buried); snapshot with AMBER autopause indicator.
- **Depends on:** WO-27, WO-46 (autopause state persistence optional).
- **Canon:** chronicle-events §5/§6; test-port `event-popup.spec.ts` + `first-session.spec.ts` dedup contract (both NOT in ledger — this WO + WO-50 close them).

**WO-49 · P3: load / new campaign menu · [S]**
- **Frame:** Lobby frame. Design-canon local-first §3.2 ("honest size of lobby/briefing").
- **Deliverables:** the TUI campaign menu — renders `babylon_meta`'s catalog as a load list; "New" mints a campaign uuid (ADR099 guardrail: DSN-from-config, no hardcoded localhost). Reuses `GameSession` creation path (`PostgresRuntime.create_session`). **No code path opens/lists campaigns from the TUI today** (meta-store §3) — this WO builds it. Row shows codename + `Tick N` + status (ACTIVE/ABANDONED), matching `lobby-briefing.spec.ts` lifecycle (reversible soft-delete, arm-then-confirm delete).
- **Contract tests:** `tests/unit/tui/test_campaign_menu.py` (list from `babylon_meta`; new mints uuid; soft-delete reversible); ports `lobby-briefing.spec.ts` + `test_lobby_lifecycle`.
- **Depends on:** WO-46, WO-35 (briefing dossier), WO-47.
- **Canon:** meta-store §3; local-first §3.2; test-port `lobby-briefing.spec.ts`.

**WO-50 · P3: unaided-first-action Pilot e2e · [S]**
- **Frame:** all frames (spine walkthrough). Cutover gate #2.
- **Deliverables:** port `first-session.spec.ts` (483 lines, the spine acceptance gate — test-port §1, largest gap, NOT in ledger) into a TUI Pilot e2e: unaided player reads a page → forms a theory → issues a verb through the plate → engine adjudicates → Chronicle revises. HARD-asserts (not "if present"): forced first crisis → AUTOPAUSED + critical modal → chronicle takeover; event-dedup; 5 objectives none pinned at 1.00; rigged-horizon → genuine epilogue with verbatim `UNRESOLVED` copy; terminal-state honesty (Step disabled once AUTOPAUSED, `/endgame/` byte-identical first-row-authoritative).
- **Contract tests:** `tests/integration/archive/test_pilot_first_action.py` (the Pilot e2e itself — this IS gate #2). Split per acceptance-gate sub-behavior into ledger rows (test-port §1 recommendation).
- **Depends on:** WO-45, WO-47, WO-48, WO-49, ALL Lane E (WO-38…44).
- **Canon:** test-port §1 `first-session.spec.ts` (gates 2/3/5/6); charter cutover gate #2.

---

# PHASE P4 — Golden vault + cutover gate (LAST; serial)

**WO-51 · P4: golden-vault seeding — THE ONE DECLARED CEREMONY · [S]**
- **Frame:** vault-tick §4; Amendment W (III.13 golden vault); charter "the program's one declared ceremony."
- **Deliverables:** seed golden-vault artifacts across qa:regression scenarios. Concrete slot-in (vault-tick §4.4): run the headless runner with `--vault-root` (WO-44) over `single_county` and `detroit_tri_county` (start there; national is the ~1.64M-commit risk — WO-44's batching/hash-skip must be proven first), produce `tests/baselines/vault/<scenario>/` golden (bundled dulwich `.bundle` OR a manifest of `{path: sha256(content)}` + final HEAD sha). Add a `qa:vault-regression` mise task analogous to `qa:e2e-regression` that runs the pipeline twice independently and asserts **byte-identical commit SHAs** (keel proved this possible).
- **CEREMONY STEPS (exact — this is the sole sanctioned drift in the program's life):**
  1. Generate the golden artifacts via the `--vault-root` run.
  2. Stage them: `git add tests/baselines/vault/`.
  3. `python3 tools/generate_ceremony_message.py --slug archive-golden-vault --summary "P4: seed golden-vault artifacts for single_county + detroit_tri_county across qa:regression"` — it computes the per-file drift table and prints a message the gate accepts by construction.
  4. Commit: pipe straight in — `... | git commit -F -` (or paste). Subject MUST be `test(baselines): seed golden vault (P4)`; body carries the generated drift table; message MUST carry the trailer `Baselines: blessed(archive-golden-vault)`.
  5. Verify: `git log -E --grep '^Baselines: blessed\(' --format='%h %s'` shows the row; `git log --oneline -1` confirms HEAD moved (commit hooks silently abort — MEMORY gotcha; commitizen rejects `ceremony(...)`, use `test(baselines):`).
- **Contract tests:** `qa:vault-regression` byte-gate (the golden itself); `tests/integration/archive/test_vault_golden.py`.
- **Depends on:** WO-44 (CLI wiring — hard prerequisite), WO-51 runs after all pages/narrator exist (WO-16…24, WO-42).
- **Canon:** vault-tick §4; Amendment W; §6.5 provenance-ceremony gate (PR #226 — trailer REQUIRED on every `tests/baselines/**` commit).

**WO-52 · P4: cutover gate #1 — test-port ledger closure · [S]**
- **Deliverables:** close `specs/24-archive/test-port-ledger.md`. Every Playwright behavioral assertion mapped to a projection contract test, a Pilot test, or the golden vault. Fill the ledger gaps identified in test-port §1: add rows for `event-popup.spec.ts`, `first-session.spec.ts`, `lobby-briefing.spec.ts` (3 NOT in ledger); disposition all 54 undispositioned `test_engine_bridge.py` classes (test-port §2) into their mapped lanes.
- **Contract tests:** `tests/unit/archive/test_ledger_closed.py` (asserts every `src/frontend/e2e/*.spec.ts` behavior + every `test_engine_bridge.py` class has a disposition).
- **Depends on:** WO-50 (Pilot), WO-38…44 (Lane E ports), WO-51 (golden vault as a valid disposition target).
- **Canon:** charter cutover gate #1; test-port §0/§1/§2.

**WO-53 · P4: cutover gate #4 — golden-vault byte-gate in CI · [S]**
- **Deliverables:** wire `qa:vault-regression` into `.github/workflows/ci.yml` alongside `qa:regression`/`qa:e2e-regression` (same mise-task invocation pattern devs run). Data-ships-as-artifacts discipline (MEMORY: CI/tests never touch the babylon-data drive).
- **Contract tests:** CI green on the byte-gate; a deliberate one-byte page mutation must RED the gate (mutation-validated per STANDING RULE).
- **Depends on:** WO-51.
- **Canon:** charter cutover gate #4; ADR090 pattern.

**WO-54 · P4: cutover execution — delete frontend, demote web · [S]**
- **Deliverables:** in one commit, `src/frontend/` deleted; `web/` demoted to what is verifiably load-bearing — the thin ingest API (local-first doc; charter). Remove the `web/game/fog/__init__.py` re-export shim (keel-api — "until P4 cutover"). **STAGE as a PR pending BD gate #3** (BD completes a full campaign session — OWNER-GATED; do not block on it).
- **Contract tests:** import-boundary tests green after deletion; `tests/unit/web/test_import_boundary.py` still passes for the surviving ingest API; no dangling `src.frontend` references.
- **Depends on:** WO-52 (gate 1), WO-53 (gate 4), WO-50 (gate 2). Gate #3 = owner-gated → PR staged, not merged, until BD rules.
- **Canon:** charter cutover gate (all 4) + post-cutover parcels (Metropole [P] / Periphery [S-heavy]).

---

# RISKS (top 8)

1. **Shared-file zipper conflicts** (`view_models.py` union, `registry.py` REGISTRY, `directives.py` `BabylonFence` methods, `app.py`) across parallel Wave-1 WOs. → **Mitigation:** strict append-only discipline (per-WO shared-file flag above); `app.py` edits serialized entirely to WO-45; single integrator resolves zippers (known, cheap — both-appended blocks).
2. **Vault-at-scale commit explosion** — national scope × per-tick per-county = ~1.64M dulwich commits (vault-tick §2/§3). → **Mitigation:** WO-44 ships content-hash-diff skip + one-commit-per-tick batching BEFORE P4; P4 golden (WO-51) scoped to `single_county`/`detroit_tri_county` first, national deferred until batching proven.
3. **tick-0 never baked** — `run()`'s bare `persist_tick(world, 0)` bypasses the observer (vault-tick §2). → **Mitigation:** WO-44 explicitly wires the tick-0 persist path; contract test asserts tick-0 page exists.
4. **Two/three overlapping veil-tier systems + documented fog+class-vision collision hazard** (`fog/filter.py:11-17`). → **Mitigation:** WO-41 resolves precedence in code with a contract test that makes the two-gates-on-one-payload case well-defined, not hazardous.
5. **Engine value-drift / qa:regression byte-identity broken by Lane E.** → **Mitigation:** view WOs fixture-first (no engine); WO-44 observer is None-default (byte-identical proven at keel); P4 is the ONLY sanctioned ceremony; every engine-adjacent WO re-runs `qa:regression` byte-identical before merge.
6. **Snapshot flakiness** — cursor blink, thread workers, `NO_COLOR` (Textual 8.2.8 digest). → **Mitigation:** snapshot conftests clear `NO_COLOR`; disable cursor blink in tested widgets; avoid thread workers in snapshot-tested paths; TGP raster (WO-33) NOT snapshot-gated (manual Kitty eyes-on).
7. **`babylon_meta` does not exist** — blocks P3 nav persistence + campaign menu (meta-store LOUD FINDING). → **Mitigation:** WO-46 CREATES it (minimal, dev two-DB scope) as an explicit P3 gate before WO-47/49; scope boundary vs post-cutover embedded cluster surfaced as OPEN QUESTION.
8. **Machine safety** — 12-core/31GB box with other live sessions; parallel agents each spawning pytest stacks tens of GB (CLAUDE.md). → **Mitigation:** Wave-1 agents worktree-isolated, scoped venv-shadow tests only (`env PYTHONPATH= .venv/bin/python -m pytest <paths> -q`), NEVER `mise run test:unit`; Lane E + Wave-2 + P3 + P4 all main-loop-serial single-flight; venv mutations + broad suites single-flight in main loop.

---

# OPEN QUESTIONS (genuine owner decisions only)

1. **Epilogue count: 5 or 6?** Charter says "five epilogue pages"; `GameOutcome` has six non-`IN_PROGRESS` members incl. `UNRESOLVED` (vault-tick §5). Is `UNRESOLVED` a sixth *authored* epilogue page, or a generic "no recognized pattern" default? Blocks WO-34 final content. (Senior ships all six shells; owner rules on the sixth's authorship.)
2. **`babylon_meta` scope for P3.** Minimal catalog against the existing dev two-DB layout NOW (WO-46 default), OR pull forward a slice of the post-cutover "recorded-not-scheduled" Periphery parcel (embedded Postgres 17 cluster, per-campaign DB, template)? The roadmap elides this gap (meta-store summary). Owner scope call — affects WO-46/49 shape and the X.8 timeline.
3. **Net-new design authorization: concept cards (WO-36) + briefing dossier (WO-35).** Neither appears in the design brief (design-canon Part 4) — no ADR-citable basis. No-MVP rule says build all of it, but these need design direction and a new DESIGN_BIBLE "wiki-page anatomy" section (the brief §9.4 flags page-anatomy as not-yet-existing; plate anatomy exists in §9b). Owner: authorize + provide/delegate the design authority for these two surfaces.
4. **Cutover gate #3 scheduling** — BD completes a full campaign session in the TUI (charter gate #3, OWNER-GATED). WO-54's frontend-deletion PR is staged pending this; owner schedules the session. (Not blocking P4's other three gates.)
