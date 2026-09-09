# The Adversary Train — design of record (2026-07-22)

> BD directive (verbatim intent): "a tutorial doesn't mean much without an enemy chasing
> you … we also need the CPU algorithm that actually plays against the player too using
> the OODA loops … I want this done prior to the BD gate 3. No finished tutorial until we
> have a CPU system and a publisher for state repression/heat/force."

Gates Gate 3. This is a **wiring train**, not a construction one — the enemy is already
built and merely disconnected (NORTH_STAR §2: the fully-built-but-disconnected disease).
Grounded in the ratified corpus the BD named: `ai/epochs/epoch3/repression-logic.yaml`
(COINTELPRO/legitimacy/Malinovsky spec), Feature 039 "State Apparatus AI" (COMPLETE
2026-03-02, `src/babylon/ooda/state_ai/`, 3407 LOC / 499 tests), the Sparrow
network-vulnerability doctrine (`ai/spec-prompts/enemy-ai/coin.md`,
`ooda/attention/sparrow.py`, Constitution I.21 [RATIFIED · PENDING CODE]), and the RAND COIN
cluster (the three-faction state coalition: Finance-Capital / Security-State /
Settler-Populist). CPU-player architecture already BD-ruled in
`project/research/24-the-archive/PLAYER_INTERFACE_SHELL_design.md` §A/§D: deterministic
in-engine policy, seeded, reproducible, part of the tick hash — "the barred AI is the
narrator, not deterministic policy, so the CPU needs no amendment" (cites Amendment V).

## The core finding (why this is wiring, not building)

`EventType.STATE_REPRESSION` is **triple-dead**, but every consumer is wired and idle:
1. `ooda/action_effects.py::_resolve_repressive` (308-341) computes a `ConsciousnessDelta`
   backfire + sets `events_generated=[STATE_REPRESSION]`, but **never** mutates
   `repression_faced`, **never** stamps `EdgeType.REPRESSION`, **never** publishes. Its
   sibling `_resolve_fascist_verb` (POGROM/VIGILANTISM) does all three.
2. `engine/systems/ooda.py::_FIRST_CLASS_ACTION_EVENTS` (44-46) = {POGROM, LOCKOUT,
   VIGILANTISM} only — STATE_REPRESSION/STATE_SURVEILLANCE excluded from the publish loop
   (deliberate, documented: "keep their converter-only path so nothing double-delivers if a
   bus publisher lands for them later").
3. The only live graph effect of REPRESS today is `layer3.py::_propagate_heat` bumping
   `heat` → `threat_score` → **zero downstream readers** (write-only dead end).

The CPU **already selects REPRESS** in the live Wayne campaign (`_legacy_wayne.py` seeds
`ORG002` STATE_APPARATUS; `RuleBasedStateAI.select_action` is deterministic — seeded
`random.Random`, faction-objective-weighted, escalation-ladder affinity; no LLM). But there
is a live **player/NPC asymmetry**: heat is bumped for both drivers, while the event tag +
CI-backfire attach to the *player*-issued REPRESS only. So the state acts, but invisibly and
without material consequence.

**Consequence of fixing (1)+(2):** `repression_faced` becomes state-produced → the whole
dialectical cascade activates through existing tested channels — `SurvivalSystem` P(S|R)
denominator (survival.py:126), `ConsciousnessSystem` agitation term + bifurcation routing
(ideology.py:244-383), `StruggleSystem` EXCESSIVE_FORCE spark + `repression_backfire`
(struggle.py:290-317), `FascistFactionSystem` pull (reactionary.py:107-136),
`tick_summary.repression_count`, chronicle (adapter lambda already at chronicle_adapter.py:426),
severity (already ACT/warning, zero drift). **The game loop the consciousness memory
describes — "late-MIM(P) + Φ-disruption loop = THE GAME LOOP" — closes here.**

## The units

**W0 — Unblock #249 (the CI flake).** `test_transcript_names_every_step…` fails CI-only:
`_refresh_breadcrumbs()` (app.py:728) throws `NoMatches('#breadcrumbs')` intermittently
during the briefing-dismiss callback, swallowed by Textual until run_test teardown (a real
production race the transcript test is merely the first to expose under xdist load). Fix:
make the deferred `App._exception` surface loudly + attributably at the step (III.11), AND
determine whether breadcrumbs-absent-mid-transition is a genuine bug or a benign transient —
fix at the real site, do not mask. Breadcrumbs is chrome; the ruling says the transcript
asserts semantic text, not chrome. Unblocks #249 to merge as INFRASTRUCTURE.

**W1 — The Publisher (engine; the BD's "publisher for state repression/heat/force").**
Extend `_resolve_repressive` to mirror `_resolve_fascist_verb`: bump `repression_faced` on
the target + stamp `EdgeType.REPRESSION` (via the existing `_bump_repression_edge`), for
BOTH drivers (closes the player/NPC asymmetry). Add STATE_REPRESSION + STATE_SURVEILLANCE to
the publish path (extend the gate or a symmetric publisher) so a real `Event` reaches the
bus → chronicle + tick_summary. Flip `tick_summary.repression_count` NULL→real-count and
re-pin its 3 guarding tests (now a real publisher EXISTS, so the "no production publisher"
premise is retired — a W-C dataflow motion per ADR109). Declared drift ceremony for
tick_summary; **Wayne tutorial transcript golden regenerates** (real orgs, real REPRESS) —
a transcript regen, not a physics baseline (Wayne ∉ qa:regression 6, which are org-free →
those baselines DO NOT move; verify byte-identical to prove it).

**W2 — The CPU is playing, make it FELT.** Verify `RuleBasedStateAI` actually fires in the
live `GameSession` Wayne campaign (in-engine ≠ in-game — check the composition root runs the
OODA state path, not just that the class exists). Surface state actions in the chronicle so
the player SEES heat rise and reads the REPRESS bulletin. If the live path is dormant despite
the seed, wire it (a W-C motion). Determinism: the state AI's seeded RNG must fold into the
tick's reproducibility (it already takes `rng_seed`; confirm it's sourced deterministically).

**W3 — Sparrow targeting (Constitution I.21 PENDING CODE → LIVE).** `select_repress_target`
currently sorts by `heat × visibility` with `visibility` hardcoded 1.0. Wire the ratified
topological targeting grammar: Surveil→singleton, Infiltrate→cutset, Raid→centrality (the
Sparrow module `ooda/attention/sparrow.py` computes these; they're just not on the decision
path). Makes the enemy TACTICALLY real — it hunts hubs and bridges in the player's SOLIDARITY
graph, per the Sparrow/Krebs doctrine. This is the depth beyond the BD's literal "a CPU + a
publisher" floor; scope-gated — lands if W0-W2 clear with headroom, else its own follow-up.

**W4 — The tutorial learns the enemy.** Extend the Wayne opening arc (T6 step-script data)
with adversary steps: the player watches heat accumulate, reads a state-REPRESS chronicle
bulletin, learns evasion/hardening. The T6 coverage sentinel MECHANICALLY forces any new
binding into the arc (the enemy cannot ship untaught). Regenerate the transcript golden as a
declared artifact. "Finished tutorial" = this post-adversary arc.

## Discipline

Same as the night's five trains: sonnet implements / opus adversarially reviews every unit /
heavy gates single-flight controller-owned / ceremonies declared. W1 is the determinism-
sensitive one — its review must confirm the qa:regression 6 stay byte-identical (org-free by
construction) and that only the declared transcript + tick_summary artifacts move.
Constitution guards the reviewer enforces: deterministic policy only (no LLM in the decision
path — Amendment V/Y), engine adjudicates (the CPU is policy, the narrator only describes),
every new construct traces to a material relation (repression_faced IS the P(S|R)
denominator — the Aleksandrov chain is immediate).
