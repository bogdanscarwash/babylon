# The cue map — every sound's binding contract

The wiring decision has a complete score waiting (#641). Each row binds one asset to the
exact engine vocabulary it fires on, so the (parked) Bevy wiring PR reads this file and
`grep`s nothing else. Bevy wiring itself is PARKED — recorded as a fresh ruling in the
train's ADR (the estate's carriage to Bevy is Amendment AF clause (v)'s standing
obligation; its schedule is not set here).

**Row schema** (machine-checked by `tests/unit/assets/test_cue_map.py`, written
red-first in this train's render/pin step — the checks refuse to pass until every
asset has exactly one row set and every target resolves to its enum):

- `asset` — `sfx/<family>/<name>` or `music/<suite>/<nn>_<name>`
- `bind_kind` — one of `event` / `outcome` / `phase` / `verb` / `ui` / `state`
- `bind_target` — the exact enum member (`EventType.*`, `GameOutcome.*`,
  `TickPartition.*`, `ActionType.*`); for `ui` / `state`, plain English
- `gloss` — what it means, named for a human (the `order_committed` precedent)
- `mix` — ducking/priority intention. Assets ship full-scale; this column IS the mixer
  contract (README: "mix priority between concurrently-firing sounds is the client
  mixer's job")
- `status` — `live` (target exists and fires) / `reserved` (target exists, no resolver
  yet — e.g. `ActionType.STRIKE` per #593) / `gated` (awaiting a named ruling)

**Standing mix contracts** (they live here, not in velocities):

1. **The Φ governor.** `music/entity/01_beast_engine`'s intensity ladder is governed by
   the imperial-rent pool Φ — the beast's loudness IS its feeding rate; disrupting Φ
   audibly starves the machine. State-governed, never random (#641 menu-safe law;
   controller adjudication under the Director's compass delegation, 2026-08-18).
2. **The strike silence.** While `sfx/resistance/resistance_strike` plays, the mixer
   DROPS the beast's pulse (`entity_pulse_*` and `beast_engine`'s percussion bus): the
   beast stops feeding for exactly this long.
3. **Loop points.** `beast_engine` loops at beat 240.0; `the_mask` loops at beat 160.0.
   Both are declared in-file by a `LOOP` marker meta-event; loop the MARKER, not the
   file end.
4. **The organ contract.** `beast_engine`'s channels are the #639 organ map —
   0/1 body+twin, 2 heartbeat, 3 circulation, 4 metabolism, 5 tissue, 6 nervous system,
   7 skeleton, 8 the mask. "Solo organ N" in the client is a mixer operation on bed
   channel N; `music/entity/03_dissection` is the reference recording for which timbre
   is which.
5. **The veil toggle.** Entering the Vol III ticker/dashboard lens crossfades the bed to
   `the_mask`; leaving it crossfades back. The toggle IS the audible lifting of the veil
   (Director-ruled Arm A: the one clean harmony is the deception).

## New material (#641 — families `entity`, `resistance`; suite `entity`)

| asset | bind_kind | bind_target | gloss | mix | status |
| --- | --- | --- | --- | --- | --- |
| sfx/entity/entity_pulse_material | phase | TickPartition.MATERIAL_BASE | the systole — the material base begins computing; ALSO the whole-tick fallback (a client with only the tick edge fires this one alone) | bed layer; never ducks | live |
| sfx/entity/entity_pulse_action | phase | TickPartition.ACTION | the organism acts — OODA dispatch | bed layer; never ducks | live |
| sfx/entity/entity_pulse_consequence | phase | TickPartition.CONSEQUENCE | the diastole — consequences settle, flat | bed layer; never ducks | live |
| sfx/entity/entity_bleed | event | EventType.IMPERIAL_SUBSIDY | one drip of the transfer — value drawn off | accent; ducks under modal focus | live |
| sfx/entity/entity_bleed | event | EventType.RESERVE_ARMY_PRESSURE | wage pressure applied — same drip, same wound | accent; ducks under modal focus | live |
| sfx/entity/entity_dispossession | event | EventType.DISPOSSESSION_EVENT | a severance — through the floor, an octave down | accent | live |
| sfx/entity/entity_dispossession | event | EventType.DISPOSSESSION_CASCADE | the severance cascading | accent | live |
| sfx/entity/entity_capital_strike | event | EventType.CAPITAL_STRIKE | the pulse omits a beat — capital withholds | accent; beast pulse yields the missed slot | live |
| sfx/entity/entity_capital_strike | event | EventType.ECONOMIC_CRISIS | the miss, generalized | accent | live |
| sfx/entity/entity_capital_strike | event | EventType.SUPERWAGE_CRISIS | the pool cannot pay the bribe | accent | live |
| sfx/resistance/resistance_organize | verb | ActionType.RECRUIT | organizing begins — a grid built from nothing | accent | live |
| sfx/resistance/resistance_organize | verb | ActionType.ORGANIZE | same cue — the third, not yet the fifth | accent | live |
| sfx/resistance/resistance_organize | verb | ActionType.FUNDRAISE | the material base of the org | accent | live |
| sfx/resistance/resistance_educate | verb | ActionType.EDUCATE | transmission, breath before tone | accent | live |
| sfx/resistance/resistance_educate | verb | ActionType.PROPAGANDIZE | the louder transmission, same breath | accent | live |
| sfx/resistance/resistance_agitate | verb | ActionType.AGITATE | the accusation that does not stay quiet | accent | live |
| sfx/resistance/resistance_agitate | verb | ActionType.DENOUNCE | the accusation, named | accent | live |
| sfx/resistance/resistance_protest | verb | ActionType.PROTEST | the crowd — more in tune than the beast, less in time | accent | live |
| sfx/resistance/resistance_alliance | verb | ActionType.PROPOSE_ALLIANCE | the earned fifth | accent | live |
| sfx/resistance/resistance_strike | verb | ActionType.STRIKE | the beast stops feeding for exactly this long | PRIORITY; invokes standing contract 2 | reserved (#593: lands as Mobilize sub_mode="strike"; no resolver yet) |
| sfx/resistance/resistance_expropriate | verb | ActionType.EXPROPRIATE | the seizure — up a semitone, short of consonance | accent | live |
| sfx/resistance/resistance_sabotage | verb | ActionType.ATTACK_INFRASTRUCTURE | damage as subtraction | accent | live |
| sfx/resistance/resistance_dual_power | verb | ActionType.BUILD_INFRASTRUCTURE | builds and stays built — fourths, not paradise | accent | live |
| sfx/resistance/resistance_clandestine | verb | ActionType.SURVEIL | disguise — the machine's timbre in the wrong hands | quiet accent; never ducks the bed | live |
| sfx/resistance/resistance_clandestine | verb | ActionType.INFILTRATE | same disguise, deeper | quiet accent | live |
| sfx/resistance/resistance_clandestine | verb | ActionType.COUNTER_INTEL | the counter-watch | quiet accent | live |
| sfx/resistance/resistance_clandestine | verb | ActionType.MAP_NETWORK | the map drawn in the dark | quiet accent | live |
| sfx/resistance/resistance_intercept_capacity | state | verb refused: organizational capacity exhausted (ADR184 — repression and revolutionary action draw one allocation) | drama, not an error — build capacity, come back | accent; interrupts the refused verb's cue | live |
| sfx/resistance/resistance_intercept_doctrine | state | verb refused: DoctrineCapability gate | the line is the obstacle, and it sounds answerable | accent; interrupts | live |
| sfx/resistance/resistance_intercept_repression | event | EventType.STATE_REPRESSION | the machine swallows the voice — cost, not defeat | accent; interrupts | live |
| sfx/resistance/resistance_intercept_repression | event | EventType.EXCESSIVE_FORCE | the swallowing, escalated | accent | live |
| sfx/resistance/resistance_intercept_repression | event | EventType.STATE_SURVEILLANCE | watched — the quiet form of the same cost | accent, attenuated by the mixer | live |
| music/entity/01_beast_engine | state | menu / idle / any lens without modal focus | the bed — the beast never stops feeding | bed; standing contracts 1, 3, 4 | live |
| music/entity/02_tribute_bleed | state | the Φ / tribute / imperial-rent lens | extraction heard as bleeding | bed swap; crossfade from beast_engine | live |
| music/entity/03_dissection | state | the exploded-view / dissection interaction (#639 organ 8) | the organs alone, then reassembled wronger | bed swap; contract 4's reference recording | live |
| music/entity/04_the_mask | state | the Vol III ticker / dashboard lens | the only clean harmony, because the mask lies | bed swap; standing contract 5 | live |

## Dispositions carried, not silently resolved

- `EventType.ENDGAME_REACHED` is a generic dispatch whose payload carries the
  `GameOutcome`; the five `endgame_*` stingers key off the outcome value, not this
  member. Confirm the dispatch site at wiring; recorded, not papered.
- `EventType.RED_OGV_ENDGAME` / `EventType.FRAGMENTED_COLLAPSE_ENDGAME` are distinct
  members that may fire alongside `ENDGAME_REACHED` for those outcomes; the landed
  stinger hints name the `GameOutcome`. Noted, not duplicated.
- **`GameOutcome.UNRESOLVED` is scored by DELIBERATE SILENCE** — the game that does not
  end does not get a chord (controller adjudication under the Director's compass
  delegation, 2026-08-18). This row is design, not a gap.
- `EventType.POLICY_STRUCK` / `EventType.POLICY_PREEMPTED` both plausibly fire
  `stinger_policy_fail`, whose hint names neither. OPEN — disambiguate at wiring.
- `EventType.PATTERN_SHIFT` (spec-116, "which ending is trending") is an ambient signal
  distinct from the terminal stingers and is currently UNSCORED. Open candidate for a
  future ambient cue; not invented tonight.
- **`music/ambient/history_breathing` (legacy) and `music/entity/beast_engine` (#641)
  both claim the menu/idle bed** — history_breathing's own docstring calls itself "the
  menu/idle music the estate never had," and beast_engine's New-material row above binds
  the identical "menu / idle / any lens without modal focus" state. Not reconciled here;
  a legacy-transcription finding, not recorded elsewhere. Wiring must choose one bed (or
  split by lens — e.g. history_breathing for the literal main menu, beast_engine for any
  in-game lens without modal focus) rather than crossfading two idle beds against each
  other.

## Legacy estate (the original 39 SFX + 13 tracks)

| asset | bind_kind | bind_target | gloss | mix | status |
| --- | --- | --- | --- | --- | --- |
| sfx/ui/ui_move | ui | menu/list cursor movement | menu/list cursor movement | accent; menu micro-sound | live |
| sfx/ui/ui_select | ui | item selected / confirmed | item selected / confirmed | accent; menu micro-sound | live |
| sfx/ui/ui_back | ui | escape / back one level | escape / back one level | accent; menu micro-sound | live |
| sfx/ui/ui_deny | ui | invalid / blocked action | invalid / blocked action | accent; menu micro-sound | live |
| sfx/ui/ui_open | ui | panel / overlay opened | panel / overlay opened | accent; menu micro-sound | live |
| sfx/ui/ui_close | ui | panel / overlay closed | panel / overlay closed | accent; menu micro-sound | live |
| sfx/ui/ui_tab | ui | tab / page switch | tab / page switch | accent; menu micro-sound | live |
| sfx/ui/ui_hover | ui | hover / focus acknowledgement | hover / focus acknowledgement | accent; menu micro-sound | live |
| sfx/ui/ui_toggle_on | ui | toggle engaged (map layer, filter, watchlist entry) | toggle engaged (map layer, filter, watchlist entry) | accent; menu micro-sound | live |
| sfx/ui/ui_toggle_off | ui | toggle disengaged | toggle disengaged | accent; menu micro-sound | live |
| sfx/state/tick_advance | state | run_tick complete / turn advance | run_tick complete / turn advance | accent; simulation/meta state | live |
| sfx/state/save_complete | state | game saved | game saved | accent; simulation/meta state | live |
| sfx/state/load_complete | state | save loaded | save loaded | accent; simulation/meta state | live |
| sfx/state/autosave | state | autosave | autosave | accent; simulation/meta state | live |
| sfx/state/state_fault | state | save/load failure or surfaced engine fault | save/load failure or surfaced engine fault | accent; simulation/meta state | live |
| sfx/state/order_committed | event | EventType.ORGANIZATIONAL_ACTION | an order/directive dispatched (OODA) | accent; simulation/meta state | live |
| sfx/state/game_start | state | new game begins | new game begins | accent; simulation/meta state | live |
| sfx/state/game_quit | state | quit to desktop | quit to desktop | accent; simulation/meta state | live |
| sfx/alert/alert_info | state | informational notice posted | informational notice posted | accent; notification ladder rung | live |
| sfx/alert/alert_favorable | state | favorable development posted (relief without resolution) | favorable development posted (relief without resolution) | accent; notification ladder rung | live |
| sfx/alert/alert_warning | state | warning — a threshold is being approached | warning — a threshold is being approached | accent; notification ladder rung | live |
| sfx/alert/alert_critical | state | critical alarm — crisis threshold crossed | critical alarm — crisis threshold crossed | PRIORITY; loudest rung of the ladder | live |
| sfx/alert/event_minor | state | minor simulation event posted | minor simulation event posted | accent; notification ladder rung | live |
| sfx/alert/event_major | state | major simulation event posted | major simulation event posted | accent; notification ladder rung | live |
| sfx/stinger/stinger_rupture | event | EventType.RUPTURE | rupture | accent; dialectical punctuation | live |
| sfx/stinger/stinger_solidarity | event | EventType.SOLIDARITY_SPIKE | solidarity_spike | accent; dialectical punctuation | live |
| sfx/stinger/stinger_solidarity | event | EventType.SOLIDARITY_AWAKENING | solidarity_awakening | accent; dialectical punctuation | live |
| sfx/stinger/stinger_false_solidarity | event | EventType.FASCIST_REVANCHISM | agitation routed to national identity | accent; dialectical punctuation | live |
| sfx/stinger/stinger_atomization | event | EventType.CLASS_DECOMPOSITION | class_decomposition | accent; dialectical punctuation | live |
| sfx/stinger/stinger_repression | event | EventType.STATE_REPRESSION | state_repression | accent; dialectical punctuation | live |
| sfx/stinger/stinger_repression | event | EventType.EXCESSIVE_FORCE | excessive_force | accent; dialectical punctuation | live |
| sfx/stinger/stinger_imperial_rent | event | EventType.SURPLUS_EXTRACTION | surplus_extraction | accent; dialectical punctuation | live |
| sfx/stinger/stinger_imperial_rent | event | EventType.VALUE_TRANSFER | value_transfer | accent; dialectical punctuation | live |
| sfx/stinger/stinger_market_correction | event | EventType.MARKET_CORRECTION | market_correction (P23 — the scissors snapped) | accent; dialectical punctuation | live |
| sfx/stinger/stinger_election | event | EventType.GOVERNMENT_FORMED | ElectoralSystem — government formed | accent; dialectical punctuation | live |
| sfx/stinger/stinger_policy_pass | event | EventType.POLICY_ENACTED | PolicySystem — LEGISLATE success | accent; dialectical punctuation | live |
| sfx/stinger/stinger_policy_fail | event | EventType.POLICY_STRUCK | PolicySystem — LEGISLATE failure | accent; dialectical punctuation | gated (Dispositions: POLICY_STRUCK/POLICY_PREEMPTED ambiguous — disambiguate at wiring) |
| sfx/stinger/stinger_policy_fail | event | EventType.POLICY_PREEMPTED | PolicySystem — LEGISLATE failure | accent; dialectical punctuation | gated (Dispositions: POLICY_STRUCK/POLICY_PREEMPTED ambiguous — disambiguate at wiring) |
| sfx/endgame/endgame_revolutionary_victory | outcome | GameOutcome.REVOLUTIONARY_VICTORY | EventType endgame_reached — REVOLUTIONARY_VICTORY | terminal punctuation | live |
| sfx/endgame/endgame_ecological_collapse | outcome | GameOutcome.ECOLOGICAL_COLLAPSE | EventType endgame_reached — ECOLOGICAL_COLLAPSE | terminal punctuation | live |
| sfx/endgame/endgame_fascist_consolidation | outcome | GameOutcome.FASCIST_CONSOLIDATION | EventType endgame_reached — FASCIST_CONSOLIDATION | terminal punctuation | live |
| sfx/endgame/endgame_red_ogv | outcome | GameOutcome.RED_OGV | EventType endgame_reached — RED_OGV | terminal punctuation | live |
| sfx/endgame/endgame_fragmented_collapse | outcome | GameOutcome.FRAGMENTED_COLLAPSE | EventType endgame_reached — FRAGMENTED_COLLAPSE | terminal punctuation | live |
| music/ambient/01_history_breathing | state | the world map at rest (ambient / menu-idle) | the world map at rest (ambient / menu-idle) | bed; underscores the ambient/menu-idle lens | live |
| music/superstructure/01_the_ballot | state | electoral ritual as bourgeois waltz (superstructure suite) | electoral ritual as bourgeois waltz (superstructure suite) | bed; underscores the superstructure lens | live |
| music/superstructure/02_the_reform_ceiling | state | the PASOK bleed as musical form (superstructure suite) | the PASOK bleed as musical form (superstructure suite) | bed; underscores the superstructure lens | live |
| music/superstructure/03_officeholder | state | capture, note by note (superstructure suite) | capture, note by note (superstructure suite) | bed; underscores the superstructure lens | live |
| music/periphery/01_unequal_exchange | state | the sigma-gradient, audible (periphery suite) | the sigma-gradient, audible (periphery suite) | bed; underscores the periphery lens | live |
| music/periphery/02_superwage | state | comfort on top of the grind (periphery suite) | comfort on top of the grind (periphery suite) | bed; underscores the periphery lens | live |
| music/rift/01_overshoot | state | O = C/B > 1 (rift suite) | O = C/B > 1 (rift suite) | bed; underscores the rift lens | live |
| music/rift/02_the_silent_spring | state | the fading of the living world (rift suite) | the fading of the living world (rift suite) | bed; underscores the rift lens | live |
| music/endgame/01_red_dawn | outcome | GameOutcome.REVOLUTIONARY_VICTORY | REVOLUTIONARY_VICTORY (endgame suite) | terminal theme; leitmotif-linked to the SFX stinger | live |
| music/endgame/02_the_long_winter | outcome | GameOutcome.ECOLOGICAL_COLLAPSE | ECOLOGICAL_COLLAPSE (endgame suite) | terminal theme; leitmotif-linked to the SFX stinger | live |
| music/endgame/03_iron_consolidation | outcome | GameOutcome.FASCIST_CONSOLIDATION | FASCIST_CONSOLIDATION (endgame suite) | terminal theme; leitmotif-linked to the SFX stinger | live |
| music/endgame/04_dual_power | outcome | GameOutcome.RED_OGV | RED_OGV (endgame suite) | terminal theme; leitmotif-linked to the SFX stinger | live |
| music/endgame/05_shattered_map | outcome | GameOutcome.FRAGMENTED_COLLAPSE | FRAGMENTED_COLLAPSE (endgame suite) | terminal theme; leitmotif-linked to the SFX stinger | live |
