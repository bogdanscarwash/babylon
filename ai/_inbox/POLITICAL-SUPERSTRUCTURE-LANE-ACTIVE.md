# ⚠ ENGINE LANE ACTIVE: Program 25 — The Political Superstructure (electoralism)

**Since:** 2026-07-22 ~13:30 EDT · **Owner:** bg session feb021be · **BD goal (in-session):**
"Expanding the playable interface earlier as we discussed with electoralism and filling in
all of those things that emerged about organizations that we already discussed."

**Branch:** `feature/political-superstructure` (off dev @ e0ece454). This is the ONE engine
train (interleaving rule); the Archive-side train is `feature/interface-refinement`
(bg 756d4549 — p1 shell boot, p2 dashboard-live), and the lanes are disjoint by design.

**Files this lane owns (new unless noted):** `domain/politics/`, `formulas/politics.py`,
`engine/systems/allegiance.py|electoral.py|policy.py`, `config/defines/politics.py`,
`models/entities/organization.py` (party/officeholder fields — shout if you need it),
`models/enums/events.py` (append-only), doctrine-tree content JSON (§3 Electoral Question
fork), catalog append (`political_form`), + the org fill-ins: LEGISLATE resolver,
spec-070 ELECTORAL faction seeding, the carried OrganizationComponent shim/contract-suite
port (roadmap assigns it to "whichever program next touches the Organization entity" — that
is now this one).

**Files this lane will NOT touch:** `cli/play.py`, `src/babylon/tui/**`, `game/session.py`,
`engine/systems/ooda.py` (the felt-actions `_FIRST_CLASS_ACTION_EVENTS` hunk stays with the
interface train's integration units).

Authority: the-electoral-question.md (all 8 §7 rulings RULED — see its annotation; ADR126),
Constitution v2.16.0. Delete this marker when the train's PR merges.
