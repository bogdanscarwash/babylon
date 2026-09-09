# Spec 116 — The Playability Spine (Program 24, sub-project 1)

**Status:** EXECUTED 2026-07-17 (all 25 tasks landed; acceptance sweep complete,
6/6 gates met — see §Acceptance gate results below) · **Branch:**
`feature/116-playability-spine`
**Parent design:** `docs/superpowers/specs/2026-07-17-viable-game-design.md` (§4)
**ADR:** ADR079 (seeded `proposed` with Task 1's first commit batch; flipped to
`accepted` at Task 25)

## Thesis

Babylon's engine already computes drama that never reaches the player. Before any screen
track (fog, circuit, doctrine) can matter, the base loop must be playable under **any**
information model: a campaign that holds tension across hundreds of ticks, an event rail
that is signal rather than wallpaper, a first session that explains itself, and the nine
evidence-backed wirings that connect existing engine riches to existing UI surfaces.
This is connection-and-framing work — no new dynamical systems.

## Owner ruling 2026-07-17 (post-design, supersedes the design's "520-tick" framing)

> "The endgame: it's **100 years of in-game time**, not a specific condition. The game
> ends when it ends; the scenario it's in is up to player actions and the economy."

At 1 tick = 1 week (defines `timescale.weeks_per_year: 52`), the campaign horizon is
**5200 ticks**. The five outcomes (REVOLUTIONARY_VICTORY, ECOLOGICAL_COLLAPSE,
FASCIST_CONSOLIDATION, RED_OGV, FRAGMENTED_COLLAPSE) become **recognized patterns** the
world is in along the way — never terminators. This executes the 2026-07-14
emergent-endgames ruling (fixed horizon + recognized patterns) which the live app had
not yet implemented ("Endgame Reached — tick 18"). Termination = horizon only, or the
player's own fast-forward-to-epilogue once a pattern locks (FR-116-5).

## Scope (from the parent design, verbatim commitments)

- **FR-116-1 · Campaign pacing (declared baseline ceremony #1).** Deterministic pacing
  instrument first (headless nationwide run; tick-of-first-recognition per outcome
  pattern + per-axis progress curves), then a single defines-level recalibration until
  the null-play arc holds tension across the 100-year horizon. Convert EndgameDetector
  from adjudicator to pattern recognizer; the campaign horizon (100 years → 5200 ticks)
  lives in defines. Fix the tick-0 "Sovereign Collapse" threshold bug. One ceremony,
  instrument-first-tune-once, per-scenario drift declared (Market Scissors promotion
  pattern).
- **FR-116-2 · Event salience.** (i) Consecutive same-type/same-subject events collapse
  into one card with count and age. (ii) Severity tiers: crimson reserved for genuine
  rupture/endgame proximity; political churn drops to gold/muted. (iii) Autopause fires
  once per distinct critical event — first occurrence or escalation, never repeats.
  Extends the existing `event-tray-mutes` machinery.
- **FR-116-3 · Lobby & briefing.** Generated operation codenames, dates, tick/status,
  delete/archive in the lobby. Game creation lands on a Scenario Briefing: who you are
  (the Cadre Council), the stakes, the five terminal outcomes in plain language with the
  win condition named (`get_journal_objectives` derives these). Curated difficulty/config
  exposure via the existing `CreateGameSerializer` overrides (never raw defines).
- **FR-116-4 · Quick-win wiring set** (each evidence-backed, see §9 of the parent
  design): (1) CausalChainObserver → bridge → wire/journal; (2) five distinct endgame
  epilogues; (3) `preview_action` costs/warnings rendered before submit;
  (4) per-target expected deltas in TargetPicker; (5) serialize the dead `tick_*`
  attributes into CrisisTimeline / BifurcationGauge / Economy surfaces;
  (6) EconomyDashboard phantom chips fixed against real fields; (7) event whitelist
  widened past 44/79 with POGROM/LOCKOUT/VIGILANTISM first-class (own EventType,
  severity, title, geographic anchor); (8) verb dead-ends become disabled-with-reason;
  (9) the permanent "PROFIT no data" chip wired or removed.
- **FR-116-5 · Fast-forward-to-epilogue** (owner box-tick, taken YES): when an outcome
  pattern is locked in, the player may fast-forward to the horizon epilogue instead of
  clicking through doomed ticks.

## Acceptance gates (parent design §4, restated under the fixed-horizon ruling)

1. Null-play nationwide run recognizes no outcome pattern before a calibrated tick
   floor, and the game terminates only at the 100-year horizon (no tick-0 events, no
   early "Endgame Reached").
2. No two consecutive identical event cards.
3. Autopause ≤ 1 per distinct event.
4. Every recognized outcome renders a distinct epilogue at the horizon (or on
   player-accepted fast-forward).
5. Preview visible before every submit.
6. A fresh player reaches their first submitted action unaided (scripted e2e trunk test).

## Acceptance gate results (Task 25 sweep, 2026-07-17)

| # | Gate | Result | Evidence |
|---|---|---|---|
| 1 | Null-play nationwide horizon | MET (with an owner-sanctioned deferral) | `reports/pacing-calibration-2026-07-17.md` — `first_recognition` all-null on the `us` 260-tick sample AND the FULL 5200-tick `wayne_county` run; the full `us` 5200-tick century run was launched then KILLED AND DEFERRED by owner ruling 2026-07-17 (report §4 addendum: "a lot of work to be done before I put that kind of work into it"); Task 2's tick-0 `SOV_EXTERIOR_NULL` exemption test + Tasks 3/4's horizon-only-termination tests confirm no terminator code paths remain |
| 2 | No two consecutive identical event cards | MET | `src/frontend/src/lib/__tests__/eventDedup.test.ts` (unit) + `src/frontend/e2e/first-session.spec.ts`'s live two-tick resolve step (exercises the real dedup output against `wayne_county`'s actual ~160-card-per-tick event volume — no two adjacent cards share a type) |
| 3 | Autopause ≤ 1 per distinct event | MET | `src/frontend/src/store/slices/worldSlice.test.ts`'s "autopause-once" describe block (unit) + `first-session.spec.ts`'s live two-resolve step, handled via `acknowledgeAutopauseIfPresent` |
| 4 | Every recognized outcome renders a distinct epilogue | MET | `tests/unit/web/test_epilogues.py` — 6 outcomes (5 patterns + UNRESOLVED), pairwise-distinct headline+body, "THE BUNKER FAILS" ×4 duplicate proven dead |
| 5 | Preview visible before every submit | MET | `first-session.spec.ts` + `e2e/verb-submit.spec.ts` — `preview-probability`/`verb-cost` testids visible before the submit click resolves |
| 6 | A fresh player reaches their first submitted action unaided | MET | `src/frontend/e2e/first-session.spec.ts` (new, this task) — full lobby → briefing → disabled-with-reason verb grid → preview → submit → two-tick resolve → honest `endgame_progress` trunk, driven against the live stack; 5/5 tests pass |

Full verification battery (Task 25 Step 4): `mise run check` green, `mise run
qa:regression` 5/5 byte-identical, `npm run check` green (frontend
lint+types+vitest). Full authenticated e2e suite: 36/37 non-setup specs green;
one pre-existing, unrelated failure found and reported (not fixed here) — see
ADR079's "Known honest limitations".

## Constraints

- Engine determinism untouched: every change either observes (serialization boundary),
  is the defines-level pacing recalibration executed as declared ceremony #1, or is the
  FR-116-4.7 event-converter widening — which may move event content in the baselines
  and, if it does, is executed as its own declared regeneration (spine ceremony #2,
  with the byte-identical fallback declared instead when no baseline scenario fires a
  newly whitelisted event).
- DEVIATION (planning discovery, 2026-07-17): the parent design §3 reserved "ceremony
  #2" for Track 3 Unit 6; the whitelist widening's converter edit lives in
  `simulation_engine.py` (bridge-side widening is impossible — dropped types never
  reach `WorldState.events`), so its drift cannot ride ceremony #1. Owner-sanction
  item in the PR body; if rejected, the converter widening defers to Track 3 Unit 6's
  ceremony and FR-116-4.7 ships bridge/frontend-side only until then.
- `mise run check` green per unit of work; `qa:regression` byte-identical **except**
  the declared ceremony commits, which regenerate baselines with per-scenario drift
  declared.
- No MVP scoping — the numbered items above are dependency-ordered, not severable.
- Coefficients live in `GameDefines`/`defines.yaml`; copy lives in data, not conditionals.

## Implementation plan

`docs/superpowers/plans/2026-07-17-playability-spine.md` (authored from the 10-scout
recon workflow `wf_cde8ea09-ac8`; task-by-task TDD).
