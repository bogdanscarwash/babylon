# M2 "Playable" — FFI seam contracts (pinned 2026-07-27)

Companion to `2026-07-26-ratatui-client-design.md` §6 and plan Tasks 21–26.
Pinned from a six-scout sweep of the real Python estates (pacing, chronicle
salience, verbs, endgame, watchlist/nav, Textual UX ground truth) — every
shape below cites the source whose field order it inherits. **JSON strings
only cross the FFI; field order is load-bearing** (serde_json
`preserve_order`; see `views/peek.rs` module docs for the M1 precedent).

Every new host method fits the existing PyO3 helpers: zero-arg (`call0`) or
one-JSON-string-arg (`call1`). Multi-parameter verbs take a **single JSON
object argument** — no new call helpers are needed (this supersedes the
design-sketch signatures `pin_watchlist(subject, pinned)` /
`issue_verb(action_id, target_id, target_community)`, which would have
required new 2-arg/void FFI plumbing).

## Envelope convention

Write verbs and tick verbs return an **envelope**, mirroring
`load_campaign` (host.py): `{"ok": true, ...}` on success,
`{"ok": false, "error": "<message>"}` on a *player-reachable* refusal.
Player-reachable refusals (watchlist capacity, verb ineligibility raced, a
`t` press while locked) must NEVER panic the client; system-level failures
(FK violation, TickOrderError, a raising driver after pre-checks) propagate
as exceptions and panic loudly per III.11 — the two classes are handled
differently on purpose. Python catches ONLY the specific exception types
named per method below — never a blanket `except`.

## 0. `load_campaign` ack (M2 addition)

The bind ack gains the session's current tick —
`{"ok": true, "campaign_id": "...", "tick": 300}` — so the HUD's
`T+{tick}` counter is honest for a RESUMED campaign from the first frame
(a zeroed counter over a tick-300 session would be fabricated, III.11).

## 1. Tick controls (Task 21)

### `pacing_state_json()` → (call0)

```json
{"attached": true, "locked": false, "lock_reason": null,
 "awaiting_ack": false, "pause_summary": null, "busy": false}
```

Dict-literal order as above. Mirrors `PacedDriverHandle`
(`tui/app.py:525-582` — primitives only; `PauseNotice` never crosses).
`attached=false` (all else false/null) when no campaign/driver is bound.
Rust pre-checks in Textual's exact order — locked → awaiting_ack → busy —
and renders the same refusal strings (`app.py:2219-2242`); this is the
established pre-check pattern, NOT exception translation. The HUD's PACING
line renders from this same payload (`dashboard_view.py:154-163` states).

### `advance_tick()` → (call0)

```json
{"ok": true, "outcome": {"tick": 43, "paused": false, "chronicle": [ChronicleEvent...]}}
```

or `{"ok": false, "error": "..."}` (no campaign bound; not implemented).
`outcome` key order: **tick, paused, chronicle** — the plan's wire spec and
the `TickOutcome` Protocol declaration order (`tui/app.py:298-328`).
`TickAdvanceResult` is NOT pydantic and its `__slots__` is alphabetical —
the host hand-builds this dict literal; never derive order by introspection
(`session.py:516` vs `:518-527`). `world`/`events`/`autosaved`/
`determinism_hash` are deliberately excluded (the narrower seam).
Python side: delegates to `PacedDriverHandle.advance_once()`. Pacing
refusals should be impossible after the Rust pre-check; if a `PacingError`
still escapes it is a bug and panics loudly (III.11).

### `run_until_paused()` → (call0)

```json
{"ok": true, "outcomes": [TickOutcome, TickOutcome, ...]}
```

An ARRAY of per-tick outcomes mirroring
`PacedDriverHandle.run_until_paused() -> Sequence[TickOutcome]`; Rust
flattens the chronicles client-side exactly as Textual does
(`app.py:2299`). The FFI call BLOCKS until the batch returns — zero
incremental feedback during a run is the Textual ground truth (no spinner,
no streaming; deviating would need a Director ruling). The loop lives in
`PacedTickDriver` (fixed bound `max_ticks=5200`); Rust never loops ticks.

### `acknowledge_pause()` → (call0)

`{"ok": true}` / `{"ok": false, "error": "..."}`. Rust pre-checks
`awaiting_ack` first; success status line: "status: autopause acknowledged
— ready to advance" (`app.py:2311-2324`), and the ack ALSO refreshes the
HUD's PACING feed so the strip never contradicts the status line.

### ChronicleEvent (elements of `outcome.chronicle`)

`model_dump_json()` of `tui/chronicle.py:148-153`, declaration order:

```json
{"tick": 42, "event_type": "uprising", "summary": "...",
 "data": {...open bag, keys vary by EventType, may nest "anchor"...},
 "class_names": null, "org_names": null}
```

Rust must not assume any fixed key set inside `data`.

## 2. Chronicle rail (Task 22)

**Salience ships as DATA the host pre-computes; Rust renders, never ranks**
(design §6 gap ledger). The rules span ticks (dedupe collapses runs across
adjacent bulletins; the autopause banner scans the full 200-row window), so
per-tick payloads cannot carry them — the HOST owns the accumulator
(mirror of `app.py:1663-1694`: append, cap `CHRONICLE_ROW_CEILING=200`)
and exposes the render-ready rail:

### `chronicle_rail_json()` → (call0)

```json
{"autopause_line": "⏸ AUTOPAUSE — THIS CANNOT PASS UNREAD",
 "rows": [
   {"subject": null, "kind": "header", "tick": 847, "severity": null,
    "actor": null, "text": "T0847"},
   {"subject": "organization/org-x", "kind": "event", "tick": 847,
    "severity": "critical", "actor": "The Vanguard", "text": "..."}
 ]}
```

- `autopause_line`: `null` when inactive (absence, never a dimmed row).
- Row order: exactly what `chronicle_stream → chronicle_rows` yields
  (newest-first). Row dict-literal order as above.
- Host pipeline per repaint = Textual's:
  `dedupe_consecutive(apply_volume_floors(history))` then
  `compute_autopause_state` then `chronicle_stream(salient, limit=200)`
  then `chronicle_rows` (`app.py:1655-1661`).
- `kind`: `"header"` (tick header, non-navigable) or `"event"`. There is
  deliberately NO `"quiet"` kind (verify-panel correction): `chronicle_stream`
  never emits an empty bulletin, so a per-tick quiet row would be a dead
  variant — an EMPTY rail is the honest-absence state, rendered client-side.
  `severity` only on events. `actor` is the bold-GOLD prefix (only ~6
  EventTypes ever have one); `text` EXCLUDES it and Rust joins them with
  the Textual `"{actor}: "` separator.
- Honest absence (no session): `{"autopause_line": null, "rows": []}`.
- Colors Rust-side: critical = bold CRIMSON, warning = AMBER, informational
  = BONE, header = bold GOLD, quiet = bold CRIMSON, autopause = bold AMBER.
- **AMBER (#ff8c00 = Rgb(255,140,0)) is NOT a §9b role token** — do NOT add
  it to `theme.rs` (the cross-language parity guard
  `test_rust_theme_parity.py` fails on unmapped constants there). Declare
  it as a documented module const in the chronicle view, citing
  `tui/theme.py:71-74`.
- `TickOutcome.chronicle` (raw events) crosses in Task 21's envelope per
  the plan's pinned wire shape; the RAIL renders exclusively from
  `chronicle_rail_json()` and the Rust shell reads only `tick`/`paused`
  today (verify-panel correction — the earlier "status/count uses it"
  claim was false).

## 3. Verb plate + dispatch (Task 23)

### `verb_plate_view_json()` → (call0)

`"null"` (no session / no player org) or
`VerbPlateView.model_dump_json()` — the model ALREADY exists
(`projection/verbs/view_models.py:80-95`; VerbRow `:44-78`; VerbPreview
`:14-42`; all frozen, `extra='forbid'`):

```json
{"kind": "verb_plate", "org_id": "...", "tick": 0, "verbs": [
  {"verb": "educate", "eligible": true, "reason": null, "remedy": null,
   "can_afford": true, "afford_note": null,
   "preview": {"estimated_consciousness_delta": 0.01, "estimated_heat_delta": 0.01,
               "action_point_cost": 1.0, "success_probability": 0.7,
               "affected_territory_ids": [], "warnings": []},
   "candidate_target_ids": []},
  ...9 rows...]}
```

Canonical verb order everywhere (F-key zip, render): **educate, reproduce,
attack, mobilize, campaign, aid, investigate, move, negotiate**
(`projection/verbs/preview.py:25-35` dict order). F1–F9 zip positionally.
`investigate` renders as 3 sub-lines sharing ONE row's signals
(`verb_plate.py:64-67`) — an honest documented limitation, do not invent
per-sub-verb eligibility. Ineligible rows show `(reason — remedy)` inline,
never hidden; `can_afford=false` never disables, only annotates.
`reproduce` is self-targeting (`candidate_target_ids` always empty).

### `issue_verb(args_json)` → (call1)

Arg: `{"verb": "educate", "target_id": "sc-x"|null, "target_community": null}`.
Returns `{"ok": true, "turn_id": 17}` or `{"ok": false, "error": "<verbatim>"}`.
Python catches ONLY `RuntimeError`/`ValueError`/`KeyError` (the three types
Textual surfaces verbatim as refusals, `app.py:2326-2387`) — anything else
panics. Rust derives an honest target exactly like `_honest_target_id`
(`app.py:707-739`): the current wiki subject's id-part ONLY if it is a real
member of the row's `candidate_target_ids`; never invented. Dispatch only
queues a turn — effects land at the NEXT tick; success status:
`status: {verb} queued (turn #{id})` (+ afford note if `can_afford` false).

### RECORDED DEVIATION (plan Task 23 test spec)

The plan says the round-trip test asserts "verb decrements remaining
actions". **No such counter exists in production**: `OODAProfile.action_points`
is declared and `ooda/constraints.py::enforce_action_points` implements a
budget, but NOTHING in the live path calls it (`engine/systems/ooda.py` has
only a global `max_actions_total=500`). Writing that assertion would be a
green test over a dead feature (the exact anti-pattern CLAUDE.md's
vocabulary-sentinel section documents). The M2 contract test instead
mirrors the two REAL behaviors `tests/integration/archive/
test_verb_resolution.py` pins: (1) a submitted verb reaches
`turn_resolution.action_phase_results` keyed by org/action_type after the
next tick; (2) an unaffordable submission is refused at `submit_verb`
(ValueError "Cannot afford") and never reaches the queue/engine.

## 4. Endgame HUD (Task 24)

### `endgame_status_json()` → (call0)

`"null"` ONLY when no session is bound (the lobby). Otherwise
`EndgameStatus.model_dump_json()` (`projection/endgame.py:51-57`
declaration order):

```json
{"pattern": null, "outcome": "unresolved", "game_over": false,
 "horizon_tick": 27040, "since_tick": null, "locked": false,
 "axes": {"revolutionary_victory": 0.0, "ecological_collapse": 0.0,
          "fascist_consolidation": 0.0, "red_ogv": 0.0,
          "fragmented_collapse": 0.0}}
```

- Tick 0 of a bound campaign is NOT absence — it is a real all-zero axes
  payload and renders as 5 empty bars, never the absence fence.
- Rust keys the bars by a FIXED id array in `_AXIS_KEYS` order
  (`endgame_detector.py:71-77`) with labels ported (copied, never imported)
  from `dashboard_view.py:36-42`: REVOLUTIONARY VICTORY / ECOLOGICAL
  COLLAPSE / FASCIST CONSOLIDATION / RED OGV / FRAGMENTED COLLAPSE. Never
  iterate the `axes` JSON object for display order.
- `triggered` is DERIVED: `progress >= 1.0` (the detector's own documented
  invariant, `endgame_detector.py:359-371`). `tick_triggered` =
  `since_tick` iff `pattern == <axis id>`, else honest `null` — a matched
  but non-winning axis has NO tracked since-tick by construction.
- Progress can FALL as well as rise (patterns dissolve); `pattern` can flip
  back to null. Render whatever arrives.
- HUD home (M2): a persistent top strip on the play screen —
  `T+{tick}/{horizon_tick}` counter, five compact gauges, PACING state line
  (from `pacing_state_json`). The Market view re-homes the bars at M5/M6.

## 5. Watchlist writes + nav persistence (Task 25)

### `pin_watchlist(args_json)` → (call1)

Arg: `{"subject": "county/26163", "pinned": true}`. Returns
`{"ok": true, "pinned": true}` or `{"ok": false, "error": "..."}`.

RECORDED DEVIATION from the design sketch's `-> None`: the capacity
`ValueError` (`WatchlistState.pin`, cap 20, loud, never LRU) is a
player-reachable outcome Textual renders as a status line
(`app.py:1878-1884`) — `-> None` + panic-on-raise would crash the client
on "watchlist is full", violating III.11's *visible cause* intent and
Textual parity. Python catches ONLY `ValueError`;
`psycopg.errors.ForeignKeyViolation` and everything else propagate → panic
(system-level). Pin/unpin semantics: FIFO append, idempotent both ways,
persist via the SAME `load_watchlist`/`save_watchlist` path M1 reads
(`BabylonMetaStore` already satisfies the Protocol — zero new store code).
On success the rail re-renders from a fresh `watchlist_json()` pull. The
row key stays literally `"subject"` (`watchlist.rs:100,159` hardcodes it).

### `nav_state_json()` → (call0) and `save_nav_state(args_json)` → (call1)

```json
{"jumplist": ["county/26163", "faction/foo"], "breadcrumbs": ["county/26163"]}
```

`save_nav_state` takes the same shape; returns `{"ok": true}` /
`{"ok": false, "error": "..."}`. Persist via `BabylonMetaStore`
`save_jumplist`/`save_breadcrumbs` keyed by the bound session (only
ENTRIES persist; cursor/capacity are reconstructed on restore —
`nav.py:18-21,85-93`). Jumplist/breadcrumb tables allow duplicates
(only watchlist has UNIQUE) — the round-trip test must NOT assert dedup.
The CLIENT dedupes exactly one seam point: a restored trail's trailing
entry equal to the fresh briefing visit is dropped before seeding, or
every resume would grow the persisted jumplist without bound; the client
also caps the saved trail to the newest 20 entries (the Python nav
capacity).

RECORDED DEVIATION from the design sketch ("restored through config_json
at launch"): `config_json` is built BEFORE a campaign is chosen
(`play.py:449-458`, `campaign_id: ""`) and nav state is campaign-scoped —
restore instead rides a post-bind pull (`nav_state_json()` after
`load_campaign`), exactly the watchlist precedent. Save cadence
(verify-panel correction): Back-to-lobby is the SOLE save point — the
chrome is always gone before the lobby can quit, so a quit-path save
could never fire; quitting from inside a campaign passes through the
leave. An empty trail still saves (a cleared trail must be able to clear
the store), and a refused save ack reports on stderr — never silently.
(Textual never wired nav persistence in production at all —
`app.py:1185-1187` uses a throwaway uuid4 + InMemory store — so this is
fresh wiring on both sides, not a port.)

## 6. Play-screen chrome + keys (integration)

Per design §7 global chrome: chronicle rail RIGHT, watchlist rail LEFT,
verb plate BOTTOM, HUD strip TOP, wiki/dossier center. Keys added in M2:
`t` step, `r` run-until-paused, `a` acknowledge, `F1`–`F9` verbs
(canonical zip), `P` (capital) pin/unpin the current wiki subject —
lowercase `p` stays the M1 link-cursor-previous inside WikiView (recorded
divergence from Textual's `p`=pin; collision found by the scout sweep) —
and `p` toggles the highlighted row's pin while the watchlist rail holds
focus (nothing highlighted = a refusal, never a fallback to the
current-dossier pin). Verb rows also dispatch on left-click (their rects
register as `verb:{slot}` pseudo-entities). `Esc` defocuses a rail back to
the center — it never falls through and tears down the campaign. The
focused pane is the ONLY one rendering its selection highlight, and its
rail title carries a `●` marker.

Post-tick refresh fanout (both `t` and `r`), Textual's exact order
(`app.py:2102-2150`): (1) known subjects, (2) HUD/endgame, (3) verb plate,
(4) chronicle rail, (5) watchlist, (6) re-fetch the open wiki page WITHOUT
switching views.

## 7. RecordingHost / fakes

Every new Host method gets a `RecordingHost` arm (transcript
under-recording breaks the Task-28 parity harness) and honest-absence or
loud not-implemented defaults on the trait per the M1 convention, so
milestone-earlier fakes keep compiling.
