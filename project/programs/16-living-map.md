# Program 16 — The Living Map

**Ratified:** initiated by Percy 2026-07-11 ("a true frontend complete with playable
maps... less like a corporate dashboard and more like a game... I gave you the end
state, you get us there"). Executed ultracode; all subagents Sonnet 5; Fable
orchestrates and synthesizes. Worktree `worktrees/living-map`, branch
`feature/113-living-map` off dev @ `2d3b2547`.

**Binding owner clarification (mid-planning):** "map-centric" = an actually readable
map corresponding to data, NOT abstract hexes. Hexes are tiles; the visible map encodes
colonial county borders aggregated into states, and those borders get redrawn through
revolution, liberation movements, political collapse, instability.

## Governing documents

- `specs/113-living-map/architecture.md` — structure: map-first shell (4 chrome strata
  over full-bleed DeckGL), InspectionStack + additive `/explain/` endpoint, unified lens
  registry, game chrome, Lane Carto (TIGER county cartography, de jure/de facto claim
  layers), lanes Carto+A→B–F→G with file-ownership boundaries.
- `project/research/16-living-map/DESIGN_BIBLE.md` — design law: five pillars, lens
  taxonomy (default = Imperial Rent Φ), disclosure rules (click-pin, wages-never-naked,
  rebuttal layer), event system (two streams/three channels/two lifetimes), visual
  language (Cold Collapse evolved), voice/lexicon (MIM-derived, with explicit
  non-adoptions), architecture amendments (§9), acceptance gates (§10).
- Research corpus: `project/research/16-living-map/` — 21 reports (books/, games/,
  studios/, data/, mim/, audit/).

## Phases

| Phase | Content | Status |
|-------|---------|--------|
| 0 | Worktree isolation, branch, deps | ✅ 2026-07-11 |
| R | 21-agent research fan-out + Design Bible synthesis | ✅ 2026-07-11 (2.65M tokens, 0 errors) |
| A | Architecture (map-first recomposition, 8 lanes) | ✅ spec committed `012dcea3` |
| I wave 1 | Carto + A(shell) + D(backend) | ✅ `577f62ec`/`0d7b269f`/`ae2c5fb4`; explain verified LIVE vs seeded session |
| I wave 2 | B(lenses+political map) + C(InspectionStack) + E(time/events) + F(outliner/dock) + N(narration) | ✅ one wave commit; 706/706 vitest; narration integrated `0cc2f38a` |
| I wave 3 | SKIN-CHROME + SKIN-MENUS (bible §9b The Installer) + G(consolidation) | ✅ `04c3c93f`; 740/740 vitest; live e2e → Phase V (tailwind-scan env blocker, root-caused) |
| D | ds-sync: new chrome → "Babylon Cockpit" project, graded renders | pending (post-skin) |
| V | Live backend run + screenshots, Playwright vs live, bible §10 gates, wrap-up docs | env READY (db migrated, session 5ad0c6ae seeded) |

**Owner aesthetic ruling (2026-07-11): DESIGN_BIBLE §9b "The Installer"** — Guix-installer
TUI dialog anatomy in the ksbc-new Kitty palette (crimson/gold on `#1a0000`); juice pass
inventory + performance budget in the integration ledger.

## Key decisions (Percy-delegated, recorded)

- Evolve Cold Collapse, never replace (ratified palette). The reskin extends the
  takeovers' existing diegetic language into the shell.
- Default map lens = Imperial Rent Φ choropleth (theory-grounded; see bible §3).
- De jure county/state cartography from TIGER 2024 (3,235 counties, GEOID⇄FIPS join);
  de facto polity claims re-dissolved client-side from immutable county arcs; geometry
  never animates, claims do (≥600ms, cause-named).
- Engine untouched; backend strictly additive (`/explain/`, two `/map/` properties).
- Straight-through execution with durable artifacts committed at each phase boundary.

## Owner items raised

- (to Program 11 owner) `bridge_county_h3` carries res-5/res-7, not the res-8 named in
  Constitution II.13 for the Transport Substrate corridor mesh — discrepancy flagged by
  the Phase-R data survey.
- BLS LAUS county unemployment file in the trove is damaged/needs re-fetch (data survey).
