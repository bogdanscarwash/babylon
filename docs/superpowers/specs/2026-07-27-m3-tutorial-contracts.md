# M3 seam contracts — the Tutorial gate (Tasks 27–29), pinned 2026-07-27

Companion to `docs/superpowers/plans/2026-07-26-ratatui-client.md` (M3 block) and sibling of
`2026-07-27-m2-seam-contracts.md` — same conventions: **field order is load-bearing** (serde_json
`preserve_order` + Python dict-literal order), write verbs return `{"ok": true, ...}` envelopes,
player-reachable refusals never panic, system failures panic loudly (III.11). Pinned from a 6-scout
sweep plus firsthand reads of `game/tutorial.py`, `game/tutorial_runtime.py`, `tui/tutorial_overlay.py`,
`cli/play.py:289-460`, `tui/app.py:1358-1424`, and all 1,172 lines of `tests/unit/tui/test_tutorial_pilot.py`.

## §0 Shared decisions

**The parity thesis.** `WAYNE_OPENING_ARC`'s anchors name TEXTUAL chrome (`binding:ArchiveApp:p`,
`option:watchlist-rail:enter`, panes `1`–`4`). Parity at M3 is **semantic**: each step's *when*
maps to the Rust client's real input via the normative anchor→script table in §5; the *then* is
proven by the same closed predicate vocabulary, evaluated **Python-side only**. Where the Rust
keystroke differs from the prose (`p` vs `P`), the mapping records the deviation; **re-anchoring
the arc's prose to the Rust client's own bindings is M7 content work and a Gate 3 agenda item for
the Director** (the arc IS the BDD spec of record for the Textual client until M7 deletes it).

**Slice discipline (unchanged).** The live overlay in BOTH clients walks
`WAYNE_OPENING_ARC.steps[2:]` (`cli/play.py:314` — the two boot beats are necessarily true once
the shell exists). The **harness** (§5) drives all 24 steps, like the Textual pilot suite does.

**DEFECT FIX (red phase first) — the `VerbIssued` live crash.** Verified twice independently
(firsthand + scout): the arc's mid-slice `VerbIssued` beats (`issue_aid_on_the_proletariat`,
`peek_a_wikilink_with_the_keyboard`) sit INSIDE the `steps[2:]` slice `_tutorial_progress_factory`
hands `TutorialRuntimeProgress`, whose `is_step_complete` raises `AssertionError` on any
`VerbIssued` predicate (`tutorial_runtime.py:141-148`). `TutorialOverlay.check_progress`'s
multi-advance loop therefore CRASHES the live Textual session on the poll where the player
completes `open_the_pinned_row_from_the_watchlist`. No test catches it (overlay tests use
bool-returning fakes — the fixture-shape failure class). Fix, shared by both clients:

- `TutorialRuntimeProgress.__init__` gains keyword-only
  `was_verb_issued: Callable[[str], bool] | None = None`.
- `VerbIssued` dispatch: `None` → the existing loud raise (contract preserved for compositions
  that never hand it verb steps); wired → `return self._was_verb_issued(predicate.verb)` —
  dispatch-proof semantics, exactly `VerbIssued`'s documented meaning.
- Textual wiring: `ArchiveApp` records dispatched names into a `set[str]` — `action_issue_verb`
  records the verb argument (e.g. `"aid"`), `action_peek_wikilink` records `"peek_wikilink"` —
  and `TutorialProgressFactory` widens to pass `was_verb_issued` (6th arg);
  `cli/play.py:_tutorial_progress_factory` threads it into the evaluator.
- Fix the stale `tutorial_runtime.py` module docstring ("its two VerbIssued beats
  (boot_into_lobby, begin_the_operation)" — `begin_the_operation` is `OnPage`, and the arc now has
  THREE `VerbIssued` beats).
- Red-phase test: build the evaluator exactly as `_tutorial_progress_factory` does (real arc
  slice, no `was_verb_issued`), assert `is_step_complete(index_of_issue_aid)` raises — then green
  with the wired callable (False before dispatch, True after).

**Patches (Director directive — content reviewed at Gate 3).** `TutorialStep` gains a REQUIRED
field `patches: str` (min_length=1) — the guide's one dialogue line per step, **data, never a
hardcoded UI string**, BDD-testable like every other field. Rules:

- `scenario_name`/`overlay_text` derivations are UNTOUCHED (the no-prose-duplication rendering
  contract); `patches` is NET-NEW prose, never a paraphrase of given/when/then.
- **Patches lines never name keys** — the *when* owns the key; Patches owns the why and the
  encouragement. (Keys differ between clients; Patches must not lie in either.)
- All 24 authored lines are pinned in §8 below and land verbatim in `game/tutorial.py`.
- The RUST overlay renders `Patches: {line}` (AMBER, the golden-fur register of the ksbc palette);
  the TEXTUAL overlay is deliberately untouched — **RECORDED DEVIATION**: Textual is M7-deletion
  bound; Patches is v1.0 (Rust) presentation. The data lives in the shared model so any consumer
  MAY render it; the Textual pilot suite is unaffected (`scenario_name`/`overlay_text` unchanged,
  transcript artifact unchanged).
- The 3D interactive Patches scene is M4 scope (raster lane), NOT M3.

**New host surface (M2 §7 conventions apply — trait default + PyHost forward + RecordingHost arm
+ FFI-fake arm, all four sites or the seam silently lies):**
`tutorial_state_json(view_state_json)` (call1, §1), `new_campaign()` (call0, §2), and
`load_campaign`'s ack gains `home_subject` (§4).

## §1 Task 27 — `tutorial_state_json` + the Rust overlay

### Python: `RustClientHost.tutorial_state_json(view_state_json: str) -> str` (call1)

**RECORDED DEVIATION from the plan sketch (`plan:438` says call0):** the evaluator's `OnPage`/
`PaneShowing` predicates ground on the CLIENT's display state (`current_subject`,
`current_pane` — `tutorial_runtime.py:130,138`), which has no host-side truth (the host cannot
distinguish "read_page for display" from "read_page for refresh"). The client therefore REPORTS
its display state each poll; predicates still evaluate Python-side only.

Argument (Rust-built, field order pinned):

```json
{"subject": "county/26163" | null, "pane": "wiki", "chrome_verbs": ["peek_wikilink"]}
```

- `subject`: the wiki view's current subject (null when none).
- `pane`: `"dashboard" | "map" | "wiki" | "topology"` — the Textual ContentSwitcher ids verbatim (§3).
- `chrome_verbs`: the client's CUMULATIVE chrome-dispatch log (append-once per name). Rust appends
  `"peek_wikilink"` when `K` is pressed while the play chrome exists. Host-side material verbs are
  NOT the client's to report (see the verb log below).

Return envelope (field order pinned; host-owned accumulator):

```json
{"active": false}
{"active": true, "finished": false, "step_index": 3, "total": 22, "step_id": "run_until_autopause",
 "heading": "Step 4/22: Given ..., when ..., then ....", "patches": "…", "body": "GIVEN: …\nWHEN: …\nTHEN: …"}
{"active": true, "finished": true, "step_index": 22, "total": 22, "step_id": null,
 "heading": "Opening arc complete.", "patches": null, "body": "Press Escape to dismiss this tutorial."}
```

- `heading`/`body` are the EXACT Textual overlay strings (`tutorial_overlay.py:233-238`):
  `f"Step {i+1}/{N}: {step.scenario_name}"` / `step.overlay_text`; finished-state strings verbatim.
  The host renders them; Rust NEVER reassembles prose (the U1 no-duplication contract).
- Advance loop = `TutorialOverlay.check_progress` verbatim (`tutorial_overlay.py:221-227`):
  bounded multi-advance through consecutive TRUE predicates, strictly ordered, per poll.
- `{"active": false}` when: no session bound, or the arming heuristic said no (below).
- Evaluator `AssertionError`s propagate (PyHost panics loudly — III.11); with `was_verb_issued`
  wired (§0) no `VerbIssued` raise is reachable.

**Constructor seam (mirrors `ArchiveApp`'s, same import-layering reason — `babylon.tui` must not
import `babylon.engine`, so the composition root hands the steps in):** `RustClientHost` gains
keyword-only `tutorial_steps: Sequence[TutorialStepView] | None = None` and
`tutorial_progress_factory: TutorialProgressFactory | None = None` (the SAME widened factory
protocol as §0). `cli/play.py:_run_rust_client` threads `_tutorial_steps()` and
`_tutorial_progress_factory(tutorial_enabled, steps)` — the identical objects the Textual path
gets. Arming happens at `bind_session` time: the factory's own tri-state heuristic
(`True`/`False` wins; `None` → `campaign.tick == 0`, `play.py:380`) decides; a `None` evaluator →
`{"active": false}` forever for that campaign. The host's verb log and the accumulator reset on
every `bind_session` (M2 precedent: `_chronicle_history`).

**Host verb log:** `RustClientHost` records dispatch-proof names: `issue_verb(...)` records the
verb string on METHOD ENTRY (dispatch reached the host — outcome-independent, matching
`VerbIssued`'s "proves dispatch, never the outcome"), `new_campaign()` records `"new_campaign"`.
`was_verb_issued(name)` = `name in host_log ∪ chrome_verbs` (the latest poll's `chrome_verbs`).

### Rust: `views/tutorial.rs` — `TutorialOverlayView`

- `update_from_json(&mut self, json)` → parse into `{active, finished, step_index, total, step_id,
  heading, patches, body}` with serde; parse failure → `parse_failed = true`, rendered as the loud
  CRIMSON `▌ tutorial UNREADABLE — malformed host data` strip — NEVER conflated with inactive.
- `render(frame, area)`: a TOP STRIP over the play area (the Textual overlay is `dock: top`,
  `max-height: 40%` — `tutorial_overlay.py:157-167` — NOT a centered popup; the plan's
  `tui-popup` sketch is superseded: **RECORDED DEVIATION** — `tui-popup` is absent from
  `Cargo.lock` (offline build cannot fetch it) and the Textual original is a top dock anyway).
  Hand-rolled per the two existing precedents (`palette.rs:179-186`, `app.rs:403-409`):
  `Clear` + bordered `Block` titled `Tutorial`, height `min(needed, 40%)`, full width; lines:
  heading (GOLD bold), `Patches: {line}` (AMBER — declared like chronicle's, theme-parity-exempt
  the same way), body lines (BONE). Finished state renders its two strings the same way.
- Z-order: base view → **tutorial strip** → palette → peek (transient user-invoked overlays stay
  on top of the ambient strip).
- **Poll discipline:** pre-fetched OUTSIDE `terminal.draw()` (the `peek_json` idiom,
  `app.rs:322-339`), gated on `chrome.is_some() && !tutorial_dismissed` — the HOST is the arming
  authority (`{"active": false}` renders nothing). `AppConfig.tutorial_enabled` stays a parsed
  passthrough; `_run_rust_client` sets it to `tutorial_enabled is not False` ("possibly on") and
  Rust ALSO gates polling on it as a seam-crossing saver — both layers honest, host decides.
- **Esc**: while the strip is VISIBLE (active, not dismissed — including finished) and the palette
  is closed, Esc dismisses the tutorial for the session (`tutorial_dismissed = true` on `App`,
  client-local — **RECORDED DEVIATION**: Textual keeps `dismissed` on the widget; observable
  semantics identical (permanent for the session, `check_progress` stops). Precedence: palette
  intercept → tutorial dismiss → rail defocus → view (`Esc` rail-defocus is shadowed only while
  the strip shows; `Tab`/`3` still move focus).
- Snapshot goldens: new `tests/tutorial_view.rs` + insta snaps (active step with Patches line,
  finished, UNREADABLE, and the 40%-clamp on a tall body).
- **12-call pins unchanged:** every existing fixture passes `tutorial_enabled: false`, so no
  tutorial call appears in `app_shell.rs:112-128` / `test_rust_client_ffi.py:122-135`. New M3
  tests set `true` and pin their own call orders.

## §2 The lobby mint — `new_campaign()` (call0) + lobby `n`

- Python: `RustClientHost.new_campaign()` constructs (lazily, once) a
  `CampaignMenu(self._catalog, engine_version=self._engine_version, defines_hash=self._defines_hash)`
  — the SAME mint path the Textual lobby drives (`campaign_menu.py:250-269`) — calls
  `new_campaign()`, records `"new_campaign"` in the verb log, returns
  `{"ok": true, "campaign_id": "<uuid>", "codename": "<operation codename>"}` (field order
  pinned). Catalog failures are system failures: they RAISE (PyHost panics) — no `ok: false`
  branch exists here by design.
- Rust lobby: `n` → `host.new_campaign()` → on ok, re-pull `lobby_catalog_json`, highlight the
  minted row (matched by `campaign_id`), status `minted Operation {codename} — press Enter to load`.
  M2's `NewCampaign → loud status refusal` arm is REPLACED (update `play_flow.rs`'s pin — a
  deliberate seam change, M2-style).

## §3 Panes — the center switcher (`1`/`2`/`3`/`4`)

The four `PaneShowing` steps name the Textual hybrid shell's panes. Parity requires a REAL pane
model, and Textual's own P1 precedent (honest `{absence}` fences until later programs wired data)
is the honest M3 shape:

- `PlayChrome` gains `pane: Pane` (`Dashboard | Map | Wiki | Topology`, default `Wiki`), with the
  wire ids `"dashboard"/"map"/"wiki"/"topology"` verbatim.
- Global keys (chrome only): `1`→dashboard, `2`→map, `3`→wiki, `4`→topology — switch the pane AND
  return focus to Center. **`3`'s M2 focus-only meaning is subsumed** (deliberate seam change;
  update the M2 key tests).
- Center region renders: `Pane::Wiki` → the wiki view (unchanged); the other three render an
  honest absence fence, one line each, CRIMSON `▌` prefix:
  `▌ dashboard pane — not yet ported (M4/M5 land this surface); press '3' for the wiki` (same
  wording with `map`/`topology` swapped in). Rails/HUD/verb plate/status are pane-independent.
- **Every subject-open navigation forces `Pane::Wiki`** (rail Enter, palette pick, wikilink
  Enter, briefing-begin) — the Textual `navigate-pane-couple` behavior `OnPage`'s
  `current_pane() == "wiki"` clause depends on.
- `view_state.pane` reports the current pane id verbatim.

## §4 The briefing begin — `home_subject`

- `load_campaign`'s ack gains a field: `{"ok": true, "campaign_id": …, "tick": …,
  "home_subject": "county/26163"}` — sourced from `babylon.tui.app._SAMPLE_SUBJECT` (import the
  constant; single source, ruling 3 "Wayne stays in lobby"). Update the three ack pins
  (`test_host_contract.py`, `test_play.py`, `play_flow.rs` fixtures) — additive.
- Rust: in the wiki view with play chrome, `Enter` with NO link under the cursor while the
  current subject starts with `briefing/` navigates to `home_subject` (jumplist-visited like any
  navigation, pane→wiki). This is the composition-level affordance Textual implements as
  `BriefingScreen`'s Enter→dismiss→`_navigate(_SAMPLE_SUBJECT)` (`app.py:1358-1376`); the baked
  briefing page itself carries ZERO wikilinks (verified: `briefing.md.j2`, 49 lines), so an
  Enter-on-link mapping is impossible — this rule is the honest port.
- `K` with no peek target (no link under cursor): status shows the honest refusal mirroring
  Textual's `action_peek_wikilink` (`no wikilinks to peek` substring — pin the exact Rust string
  as `status: no wikilinks to peek on this page`), AND the press appends `"peek_wikilink"` to
  `chrome_verbs` (dispatch proof, §1).

## §5 Task 28 — the headless parity harness (`tests/unit/tui/test_tutorial_pilot_rs.py`)

**Composition (mirrors `test_tutorial_pilot.py:147-347` + `play.py`'s M3 wiring — real engine,
real vault, fake Postgres only):** `_InMemoryGameStore` mirrored verbatim (the pilot's own
mirror-not-import precedent, cited in a comment); EMPTY `InMemoryCampaignCatalog`
(`boot_into_lobby`'s given); in-memory watchlist/nav persistence fakes (mirror
`test_rust_host_m2.py`'s); a `_loader` mirroring the pilot's (`create_new_campaign` over
`WayneCountyScenario()` + `vault_page_source`/`vault_known_subjects` + `bake_briefing`,
`narrator=None`); `_driver_factory` with the documented cast; `RustClientHost(catalog,
defines_hash="d"*16, engine_version="m3-tutorial-pilot-rs", campaign_loader=…, driver_factory=…,
watchlist_persistence=…, nav_persistence=…, tutorial_steps=_tutorial_steps(),
tutorial_progress_factory=_tutorial_progress_factory(True, steps))` — the play.py objects
themselves where importable. Mint determinism: `mock.patch("babylon.tui.campaign_menu.uuid4",
return_value=UUID(int=99))` (the pilot's own `_PINNED_CAMPAIGN_ID` trick).

**Headless size:** `AppConfig` gains `headless_size: (u16, u16)` (serde default `[80, 24]` — all
existing fixtures unchanged); the harness passes `[120, 50]`, the pilot's own `_PILOT_SIZE`, so
the frame-text content checks assert against an un-clipped viewport.

**The normative anchor→script mapping** (drives ONE `babylon_tui.run` call; per-arc-step frame
spans recorded by construction):

| arc step (anchor) | Rust script | note |
|---|---|---|
| `boot_into_lobby` (`binding:LobbyScreen:n`) | `n` | §2 mint |
| *bridging: load the minted row* | `enter` | pilot's `_load_the_minted_campaign` analog |
| `begin_the_operation` (`binding:BriefingScreen:enter`) | `enter` | §4 rule |
| `read_the_*` / `page:` anchors | *(empty)* | pure-read steps, pilot precedent |
| `advance_a_tick` (`t`) | `t` | |
| `run_until_autopause` (`r`) | `r` | the HONEST-GAP no-op refusal (M2 string `autopause pending (…) — press 'a' to acknowledge`); `PausePending` still holds |
| `acknowledge_the_pause` (`a`) | `a` | |
| `palette_to_*` (`palette:<subject>`) | `/`, *type the subject id*, `enter` | type the FULL subject id — deterministic single match; **deviation from the pilot** (which posts `EntityNavigated` directly): the Rust palette is driven for real |
| `jump_back_to_wayne` (`ctrl+o`) | *Rust wiki's real back key* | pin from `wiki.rs` at implementation; record prose deviation if it differs |
| `jump_forward/back_with_brackets` (`]`/`[`) | `]` / `[` if the wiki binds them; else its real keys | same |
| `learn_the_*_pane` (`2`/`3`/`4`/`1`) | `2`/`3`/`4`/`1` | §3 |
| `pin_…` (`binding:ArchiveApp:p`) | `P` | **RECORDED DEVIATION**: Rust pin is `P` (`p` = link cursor, M2 ruling) |
| `open_the_pinned_row…` (`option:watchlist-rail:enter`) | `tab`×k, `enter` | k = the ChromeFocus cycle distance, pinned from `app.rs` |
| `issue_aid…` (`binding:ArchiveApp:f6`) | `f6` | |
| `peek_…` (`binding:ArchiveApp:K`) | `K` | §4 refusal + `chrome_verbs` |
| `open_the_chronicle_rails…` (`option:chronicle-rail:enter`) | `tab`×k, `enter` | no navigable row at tick 2 → cursor `None`, Enter no-op; `OnPage(C001)` holds — the same honest floor as Textual's disabled AMBER row |

**Assertions (the pilot's tiers, ported):**

1. **In-order completion** — the host adapter keeps a `completion_log` of `(step_id, poll_ordinal)`;
   the harness asserts every slice step id appears exactly once, in arc order (in-order by
   construction of the accumulator — the assertion pins the construction), and that each step's
   completion poll falls at-or-after the frame where its mapped input ran (the input CAUSED it).
2. **VerbIssued dispatch-proof** — host verb log contains `new_campaign` and `aid`;
   `chrome_verbs` contains `peek_wikilink`. **RECORDED IMPROVEMENT over the pilot:** no
   `mock.patch` spies — the host IS the dispatch seam.
3. **Extra content checks, verbatim ports** (`_EXTRA_CONTENT_CHECK_BY_STEP_ID`): frame text at the
   right spans contains `class_composition.labor_aristocracy` + `26163`; `wage_balance\s+-?\d+\.\d+`
   and `labor_aristocracy_verdict\s+(True|False)`; `org_type`+`state_apparatus` + `heat` regex;
   `repression_faced` regex; the aid queue status + `store.submitted_turns[-1]` is
   `verb="aid", target_id="C001"`; the peek refusal substring `no wikilinks to peek`.
4. **Transcript golden** — `tests/unit/tui/transcripts/wayne_opening_arc.json` (the plan's exact
   path), committed, regenerate-freely (NOT `tests/baselines/**` — no ceremony):
   `{"arc_id", "steps": [{"index", "id", "scenario_name", "completed_poll", "frame"}]}` +
   a two-independent-runs byte-identical determinism test (the pilot's own doctrine) and a
   regenerate path (`BABYLON_REGEN_TRANSCRIPT=1`).

## §6 Task 29 — the gate run

Full arc green through the harness; transcript blessed (committed); ADR150 gains the status note
(parity PROVEN at M3, cutover unblocked) once green. **BD Gate 3 (#262) is the Director's own
combined content+client ceremony — STOP and hand over when the harness is green.** Gate 3 agenda
items surfaced by this contract: (a) the Patches lines (§8) — content review; (b) the M7
arc-re-anchoring question (prose names Textual keys); (c) the tutorial-coverage sentinel's Rust
counterpart (plan Task 46 — the Rust `Esc` dismiss inherits the Textual escape exemption's
reasoning until then).

## §7 RecordingHost / fakes parity (M2 §7 extended)

Every new method lands at ALL FOUR sites: `host.rs` trait (defaults: `tutorial_state_json` →
`{"active": false}`; `new_campaign` → the loud not-implemented `{"ok": false, "error": …}`
envelope), `babylon-tui-python/src/lib.rs` (call1/call0 forwards), `app.rs` `RecordingHost`
(record + delegate — the silent-omission trap), `test_rust_client_ffi.py`'s `_FakeHost`.
`load_campaign` ack pins updated in all three fixture homes (§4).

## §8 The Patches lines (Director content — Gate 3 reviews)

Authored per §0's rules (never name keys; never paraphrase given/when/then; the guide's voice —
warm, concrete, a golden snub-nosed monkey who takes the material seriously):

| step id | patches |
|---|---|
| boot_into_lobby | Hi — I'm Patches! That lobby's empty. Let's mint our very first campaign together. |
| begin_the_operation | There's our briefing. Read the stakes, then begin the operation — I'll be right beside you. |
| read_the_county_dossier | This is Wayne County — real numbers, not stage props. Look around the statblock before we touch anything. |
| advance_a_tick | Ready? Advance one tick and a whole week of history turns over. |
| run_until_autopause | Now let the weeks roll — the engine stops us the instant something critical fires. |
| acknowledge_the_pause | Something fired! Acknowledge it — the world waits until we've read the wire. |
| palette_to_the_economy_dossier | The command palette can jump us anywhere the Archive knows. Let's visit the national economy. |
| read_the_theorem_verdict | The big one: wages against value. This verdict is the engine's own math — nobody's opinion. |
| jump_back_to_wayne | Lost? Never. The jumplist remembers every page we've walked. |
| jump_forward_with_brackets | It walks forward too — back to the economy page we just left. |
| jump_back_with_brackets | And back again. Forward, back — the trail is always ours. |
| palette_to_the_state_apparatus_dossier | Time you met the other side. The Detroit police are already watching this county. |
| read_the_state_apparatus_dossier | See the heat number? A real adversary keeping real accounts — not narrative color. |
| palette_to_the_repression_ledger | Now the people that heat lands on: the Detroit proletariat — our people. |
| read_the_repression_ledger | repression_faced is the weight our organizing has to outrun. Remember this number. |
| learn_the_map_pane | The shell has more rooms. This one's the map — terrain matters. |
| learn_the_wiki_pane | And back to the wiki, where every dossier lives. |
| learn_the_topology_pane | The topology room — the web of relations we'll be rewiring. |
| learn_the_dashboard_pane | The dashboard — the campaign's vital signs at a glance. |
| pin_the_proletariat_to_the_watchlist | Pin the proletariat to the watchlist — the people we came for stay in sight. |
| open_the_pinned_row_from_the_watchlist | See? One pin, and they're a doorway we can open any time. |
| issue_aid_on_the_proletariat | Now our first real act: Aid. Not a gesture — a material write on the world. |
| peek_a_wikilink_with_the_keyboard | Peek is how we preview links without leaving — though this page honestly has none yet. |
| open_the_chronicle_rails_highlighted_row | The chronicle keeps the record. Today its top row is just the pause marker — but soon, the events will carry names. |

## §9 Integration addendum (pinned post-port, 2026-07-27)

Findings from single-flight integration + the real-vault harness run — each now IS the contract:

1. **Statblock-fence fill — attempted, then REMOVED (verify-panel R9).** The integration pass
   first added a host-side fill for empty `{statblock}` fences; the panel then proved it inert
   everywhere: every baked page carries its rows INLINE from the vault template loop (III.13),
   and the only empty-fence case (the tick-0 county, every field fogged) yields zero rows, so
   the fill extended nothing on any page. It was deleted as dead code with a false origin story.
   The TRUE statblock story: baked rows render identically in both clients; the empty fence IS
   the honest tick-0 epistemic state, accompanied by the page's own absence fences (item 2).
2. **The county check pins the LIVE surface (deviation from the pilot).** The pilot's
   `class_composition.labor_aristocracy` row is FIXTURE-FED on Textual (the dossier statblock
   seam is "not-yet-live", `app.py`'s own docstring; the live `CountyView` at tick 0 is
   epistemically unattributed — fog). The harness asserts FIPS + the live fence + the epistemic
   absence fence naming the Census remedy. **Gate 3 agenda:** the Textual pilot's "not a
   fixture" check is satisfied by a fixture.
3. **The M2 Esc rail-defocus arm did not exist** (the M2 contract's prose promised it; the rails'
   doc comments asserted it) — Esc on a focused rail fell through to wiki-Back. Shipped here,
   after the tutorial-dismiss arm per §1's precedence chain, with a regression test.
4. **Autopause-refusal summary capped at 48 chars** so the actionable `— press 'a' to acknowledge`
   tail survives a real multi-event tick summary (an uncapped real Wayne summary pushed it past
   the line edge). Shape unchanged; short summaries byte-identical.
5. **`was_verb_issued(name)` is a public host METHOD** (harness tier-2 read surface), the same
   union the evaluator seam reads: `verb_log ∪ latest chrome_verbs`.
6. **`verb_log` does NOT reset on `bind_session`** (contract §0's reset prose yields to §5's
   concrete assertion): `new_campaign` is dispatched from the lobby BEFORE the first bind. Known
   wart: a second fresh campaign in one process inherits dispatch history — narrow (arming
   heuristic gates resumed campaigns; the verify panel weighs the fix).
7. **`TutorialStepSource`** — host-local 4-member structural protocol (WO-37 idiom) instead of
   widening the overlay's 2-member `TutorialStepView`; the three type-ignores died.

## §10 Verify-panel outcome (3 Opus lenses, 30 findings, 2026-07-27)

All blockers/majors remediated same-session; the full list lives in the panel transcript. The
load-bearing ones, now contract: the strip **reserves rows** (Textual dock semantics — it had
overlaid and destroyed the HUD/pane fences, certified by the first golden); strip/dismissal state
resets on leaving the campaign; headings/Patches **wrap** (`Wrap {trim:false}`, `height_for`);
the autopause refusal leads with the instruction (`autopause pending — press 'a' to acknowledge
({summary})`, replacing §9 item 4's cap); the evaluator reads a **session-scoped** verb log reset
on bind (the lifetime `verb_log` remains the harness surface — fixes the reachable
second-campaign false-complete); `cmd_issue_verb` records the chrome verb BEFORE its refusal
ladder (keypress-dispatch parity with Textual); Esc can never dismiss the UNREADABLE strip; the
tutorial polls only on state-changing events (and the pin predicate reads a bind-hydrated
write-through cache, never the store per poll); tier-1 is the REAL causal check with two named
early-completion exemptions (`run_until_autopause`, `open_the_chronicle_rails_highlighted_row` —
gate-condition semantics, pinned so a third early completion goes red); the harness pins HUD
visibility and the three pane fences at their teaching beats; CI/mise lost every reference to the
deleted Django paths (including the silently-inert advisory Postgres leg).

**Gate 3 agenda (Director):** the §8 Patches lines; the M7 arc-re-anchoring question (prose names
Textual keys); the Textual pilot's fixture-fed county check (its docstring now says so honestly);
step `run_until_autopause`'s heading+Patches line being unreachable in play (same-poll
multi-advance — identical in Textual); Esc's precedence (tutorial-dismiss shadows rail-defocus
and wiki-Back while the strip shows).
