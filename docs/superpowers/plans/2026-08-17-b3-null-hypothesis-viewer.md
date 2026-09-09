# Implementation plan — B3 wave 1, "watch the null hypothesis run"

**Status:** ready to execute (revision 2). Wave 1 only; wave 2+ enumerated in §10 and built by
nobody here.
**Authority:** Amendment AF (`CONSTITUTION.md:683`) clauses (i)/(ii)/(v); GH#617 (Admin Debug
Dashboard, Director 2026-08-17); GH#619's static-first sequencing ruling (memory
`visualization-directives-2026-08-17.md:83-88`); GH#577 (the B3 milestone, whose two *named 3D*
items are explicitly **not** this wave — §0.1); the Game Design Standard
(`docs/superpowers/specs/2026-07-29-game-design-standard-design.md` — **§1 at `:39`** for
"recognizers, never terminators", **§3 at `:65`** for the cadence law, Layout B at `:67-79`,
ruling 11 at `:24`); ADR170 (tension lens), ADR182 R1/R2/R5 (write log, structure-public,
capability-gated depth), **ADR213** (the intrinsic-host train — `SessionId`, D179 session-id
provenance, D178 `node_content_ids`); Constitution III.7 (determinism) and III.11 (loud absence).
**Grounding reports (read before executing):** `b3-charter/client-estate.md`,
`b3-charter/emission-surface.md`, `b3-charter/directive-register.md`, `b3-charter/data-seam.md`.
**Adversarial critique of record:** `b3-charter/plan-critique.md` (4 Critical / 9 Important /
9 Minor). §0.3 records what changed and why; every finding is dispositioned in this text.

**Author:** planning pass, 2026-08-17/18. **Revision 2, 2026-08-18:** re-verified against
`/media/user/data/worktrees/wt-b3` **after** the merge up to dev tip `b5a3268a` (PR #657,
`feature/576-intrinsic-host`). Every `file:line` below was re-read at that tip. Revision 1's
citations against `20d95482` are superseded — do not consult them.

---

## 0. Scope verdict — read this before writing any code

The Director-approved lane is: **make an unattended simulation run engaging.** Five deliverables
(time controls, narrative beats, the carceral arc as the first story, an extensible lens/pane
seam, the declared admin/material-truth framing). Three things must be said out loud first.

### 0.1 This is NOT #577's body of work — charter it as a sibling

`gh issue view 577` scopes B3 as **"Patches the golden snub-nosed monkey as the tutorial guide"**
plus **"the topology view as 3D scenes."** Neither lands in wave 1, and the reasons are ruled, not
preferential:

1. **#577's own sequencing clause (R3):** "B3 waits for nothing on the engine side but comes LAST
   — the 2D game must be playable first. Non-goal: no 3D globe/terrain-first presentation at
   v1.0." A viewer that makes the 2D run watchable is precisely the prerequisite that clause
   names.
2. **Amendment AF (i) licenses exactly two 3D moments** (Patches; the topology view), and
   **#648 — an open `director-gate` issue — is litigating whether the Observatory suite fits
   inside that enumeration.** Building 3D before #648 closes risks doing it twice.
3. **The estate says 3D is not nearly free here.** `babylon-client` is 2D by construction:
   `Camera2d` (`map/camera.rs`), `Mesh2d`/`MeshMaterial2d<ColorMaterial>` (`map/mesh.rs:63-73`),
   zero `StandardMaterial`, zero custom `Material` impls, zero `AssetServer::load` call sites, no
   `.gltf`/`.obj` in the tree. A 3D moment means a second camera, a lighting rig, a model pipeline
   and an asset-licensing decision — a train, not a task. **No 3D this wave.**

**Action (Task 0.1, no code):** open a wave-1 issue under the `Bevy B3 — Client Completion`
milestone — *"B3 wave 1: watch the null hypothesis run — time controls, narrative beats, the
carceral arc"* — and comment on #577 recording that its two named items stay closed as the
capstone, unblocked-but-later, with #648 named as the 3D gate. Comment on #617 naming which of
its ingredients land here (the declared admin surface, the `TickReport` pane) and which do not
(the write-log pane — §8 B4). **Wave 1 does not discharge #617** (§8 B4, Minor 9).

### 0.2 The narrative_hint strings are real authored copy — and they are NOT on the wire

The design brief is right that the packs' `narrative_hint` strings are authored copy. The
emission-surface report is also right that **no emitted payload anywhere in the BSL estate carries
a string** (§2.8 admits no string payload values), and every port dropped its hint explicitly
(`decomposition.bsl:354`, `control-ratio.bsl:163,169,294,341`, `vitality.bsl:90-95`).

Both facts hold at once because the copy lives in the **frozen Python reference**, where it is a
template over the *same numeric payload keys the BSL emit carries*:

```python
# src/babylon/engine/systems/decomposition.py:360-365
"trigger_event": EventType.SUPERWAGE_CRISIS.value,
"narrative_hint": (
    "CLASS DECOMPOSITION: Labor aristocracy collapses. "
    f"{enforcer_pop_gain} become guards/cops. "
    f"{proletariat_pop} fall into the precariat."
),
```

`enforcer_pop_gain` / `proletariat_pop` are exactly `population-transferred-to-enforcer` /
`population-transferred-to-proletariat` in `decomposition.bsl:383-384`'s emit. **Therefore the
narration is a client-side transcription job, of the identical kind `county_tension` already
performed on `tension.py`** (`lens.rs:66` records that precedent in its own doc comment) — not
an engine change, not a new payload type, not a fabrication. §2.2 makes it a design ruling with a
drift guard; §2.3 makes the *causal* half (`trigger_event`, above) a shipped instrument.

### 0.3 What revision 2 changed, and what the merge dissolved

The critique's four Criticals and nine Importants are dispositioned in the sections named below.
Two of its findings changed shape because **the tree moved under it** — the plan's base was an
ancestor of dev; the worktree is now merged to `b5a3268a`:

- **`node_content_ids` now EXISTS** (`babylon-bsl/src/scenario.rs:311`,
  `pub node_content_ids: HashMap<NodeId, String>` on the public `LoadedScenario` that the public
  `load_scenario` returns; threaded to `PreparedRules` at `babylon-tick/src/lib.rs:148`). The
  critique's B2 verification ("`rg node_content_ids rust/` returns zero hits") was true on the old
  tip and is **false now**. Revision 1's hand-transcribed per-story rosters and their transcription
  guard are **deleted from this plan** — §3.2 derives the roster from the loader. Global
  Constraint 6 is untouched: the client reads a public field it already had a call to.
- **A carceral golden now exists — and it pins TICK 1 ONLY.**
  `tick_goldens.rs:709 carceral_arc_conformance_hashes_are_pinned` calls `run_once` (one tick) and
  pins `before`/`after`/`fired = 13`. It is **not** a tick-110 oracle and cannot serve as one.
  C1's disposition (§2.9, Task 7) is built on that fact, not around it.

Everything the critique verified sound is kept verbatim: the §2.2 severity table, the
transcription argument, the B6/#503 citation, the `test_rust_theme_parity.py` precedent, B1's
zero-TERRITORY finding.

### 0.4 What the landed content can actually feed (the honesty budget)

From `b3-charter/emission-surface.md`, re-verified: 12 real `EventType`s exist across the 9 landed
Material Base packs; 3 packs (territory, production, metabolism) emit **nothing** and are visible
only through direct attribute reads. Two scenarios matter:

| Story | Content | What moves | What is honestly absent | Validated horizon |
|---|---|---|---|---|
| **counties** (today's default) | `us-counties-lifecycle-demo.bscn` + `vitality.bsl` + `lifecycle.bsl` | `LIFECYCLE_TRANSITION` ×12/tick, `LEGITIMATION_CRISIS`/`_RECOVERY` on edges, `ENTITY_DEATH`; pop-d/p/d′ move every tick | Tension lens (**zero cells resolve** — no pack writes `tick-exploitation-rate`/`tick-total-surplus`, `lens.rs:62-63`); legitimation is const-only after tick 1 | **tick 1** — `tick_goldens.rs:102` pins tick 1; `us_counties_demo.rs:22` advances once. §2.7/I3 |
| **carceral** (this wave's shipped story) | `carceral-arc-conformance.bscn` + `decomposition.bsl` + `control-ratio.bsl` | 4 beats at ticks 1 / 53 / 105 / 106, byte-verified against the frozen mirror | **Zero TERRITORY nodes** — the county map has nothing to paint (§8 B1) | **tick 110** — `carceral_arc_conformance.rs:146` `LAST_TICK` |

Anything needing `ImperialRent`, unequal-exchange flows, Vol II/III series, ecology, or the
Verdict Watch's five progress bars is **wave 2, gated on its producing pack's port** (S6's
lens-rides-the-port policy). §10 lists them; none is built here.

---

## 1. Global constraints

Non-negotiable for every task below.

1. **The viewer reads; it never writes.** The one and only state-advancing call stays
   `EngineSession::advance` → `TickSession::advance` (`engine_link.rs:148-150`), the same
   deterministic path the CLI driver uses. Zero `update_node`/`add_node` in non-test client code
   (`b3-charter/data-seam.md` §2 verified this holds today — it must still hold at merge).
2. **Determinism is unaffected by the clock.** Auto-run changes *how many* ticks execute, never
   *what* a tick computes. The executable form of that claim is Task 2's gate: the state hash at
   tick N is byte-identical whether N was reached by N single steps or by one auto-run batch.
3. **Honest physics (GH#618/M8).** Every on-screen quantity maps 1:1 to a named engine quantity,
   cited by field or payload key at its call site. No decorative motion that implies causation.
   The one animation this wave ships is the **tick heartbeat** (§2.1), which claims nothing about
   any field — it *is* the clock.
4. **Loud absence over fabrication (III.11), in THREE classes not two.** A field the engine did not
   write renders as declared absence; a field the engine wrote as a **declared non-computation**
   renders as `NotComputed` with its reason — never as the numeral it literally holds (§2.6). No
   zeros, no interpolation, ever.
5. **The admin surface is the NAMED exception (GH#617/M12).** Wave 1 renders material truth
   unfogged — legitimate because a no-player observatory has no epistemic state to protect
   (`b3-charter/data-seam.md` §1) — and says so on screen. The seam a later player view filters
   through is built here (§2.6); the filter itself is not.
6. **No engine-crate change in wave 1.** `babylon-bsl`, `babylon-graph`, `babylon-tick` and every
   `.bsl`/`.bscn` file are read-only to this train. Reading a **public** field or calling a
   **public** function of those crates is not a change. If a deliverable seems to need an actual
   edit, it is wave 2 (that is how §8 B4 got there).
7. **No Python-engine change.** `src/babylon/` is the frozen reference. `qa:regression` and
   `qa:vault-regression-ci` must stay byte-identical **because nothing Python moved**. The one
   Python file this train adds is a *test* (the parity guard, §2.2).
8. **TDD, red phase mandatory.** Rust has no `@pytest.mark.red_phase`; a red step is a commit
   whose test fails and whose message says so, immediately followed by its green commit. Commit
   per unit of work via `mise run commit -- "type(scope): msg"`.
9. **Power-of-10 rules.** Explicit typing, no catch-alls, ≤~100 lines per function, **every loop
   statically bounded** (the auto-run catch-up loop's bound is a `const`, §2.1), smallest possible
   scope.
10. **Pedantic clippy + `cargo doc -D warnings` on `babylon-client` from day one** — the crate is
    *not* on the pedantic legs today (`.mise.toml:1645-1649` names only kernel/bsl/graph). Task 1
    puts it there, after Task 0 measures the pre-existing debt. RST-style rustdoc on every public
    item.
11. **Machine safety.** `/media/user/data/worktrees/wt-b3` has its own `rust/target`, so this
    train may build concurrently with other worktrees — but **within this tree, cargo is
    single-flight** (one target dir, one file lock). Never fan out `cargo test` across agents.
12. **Aesthetic law.** §9b role colors come from `palette.rs` (`:14-21`) and nowhere else — the
    crate-wide stray-literal sweep in `tests/unit/render/test_rust_theme_parity.py` fails on any
    new `Color::srgb[_u8]` outside `palette.rs` or a registered exemption. Crimson/gold on
    near-black, no rounded corners, no glow, no blur (memory `percy-aesthetic-ksbc-guix.md:11-19`).
    Iosevka stays unshipped (§8 B11) — do not vendor an unlicensed font.
13. **No campaign verdict, no ending screen, no outcome framing.** The five canonical outcomes and
    the Verdict Watch are Director-reserved and out of wave 1 (§2.5, §8 B14). A system latch is
    rendered as a system latch.

---

## 2. The engineering calls

Each call: the decision, why, and the alternative that was rejected.

### 2.1 The clock — one system, a bounded catch-up loop, a plain resource

**Decision.** A `RunState` resource — `{ running: bool, speed_index: usize, accumulator: f32,
autopause: AutopauseMode }` — plus **exactly one** `Update` system, `advance_ticks`, that replaces
`advance_on_space` (`loop_ui.rs:150-182`) and folds three inputs into one advance path:
single-step (`Space`), auto-run (accumulator vs. the speed table), and run-to-next-beat. Speeds
are a fixed table `const SPEEDS_PER_SECOND: [f32; 5] = [1.0, 2.0, 5.0, 10.0, 25.0]`. The catch-up
loop is `while advanced < MAX_TICKS_PER_FRAME && accumulator >= interval` with
`const MAX_TICKS_PER_FRAME: usize = 8`, and the accumulator is **clamped** to
`MAX_TICKS_PER_FRAME × interval` afterwards so a stalled frame (window drag, breakpoint) can never
fast-forward the sim.

**Defaults are a design decision, stated (Minor 6, and C3's disposition):**
`running = true`, `autopause = OnCritical`, `speed_index = 2` (5 t/s). That IS
**autoplay-until-event**, the mode GDS §3 (`…design.md:65`) names as the default. The run starts
moving and stops itself at the first critical beat; it does not wait for a keypress it never told
anyone about.

Bindings: `Space` step, `P` play/pause, `,`/`.` slower/faster, `B` run-to-next-beat, `N` next
story (§2.5), `Tab` lens (unchanged), `F3` admin panel. All of them appear in the controls legend
on the tick-0 story card and in the HUD footer — a binding not on the legend does not exist.

**Why.** (a) Power-of-10 rule 2 demands a statically provable loop bound, and a `const`-bounded
`while` gives one trivially. (b) Two independent advancing systems in one frame double-advance —
the client-estate survey flags this as the exact hazard, so the fold into one system is a
correctness requirement, not tidiness. (c) `loop_ui.rs:64-96` records two ordering bugs already
found *by test* in this plugin's `.chain()`; keeping the advance inside that same chain means the
existing `.after(advance_on_space)` guards keep working with a rename, instead of being re-derived
against a new schedule. (d) Lens recompute and `LensChanged` fire **once after the batch**, not
per tick — the recolor clears all 360,064 vertices unconditionally (`map/bands.rs:191-194`;
the count is `atlas.rs:664`'s own arithmetic), so per-tick recolor at 25 t/s would multiply the
one genuinely expensive operation by 25 for no visible gain.

**The accumulator arithmetic is a pure function (I4).** `advance_ticks` computes nothing itself:
it calls
```rust
fn ticks_due(accumulator: f32, interval: f32, max: usize) -> (usize, f32)
```
which is unit-tested with **zero Bevy** and stated entirely in tick-domain terms. The only line
in the crate that reads a clock is the one that adds `time.delta_secs()` to `accumulator`.

**Rejected: `FixedUpdate` + `Time<Fixed>`.** Bevy's fixed timestep already implements catch-up,
but it lives in a *different schedule* than the `.chain()` whose ordering this crate has already
paid to get right, and changing speed means mutating a global timestep resource that any future
consumer shares. The estate's own comments make schedule-splitting the expensive mistake here.

**Rejected: Bevy `States<Playing|Paused>` + `run_if`.** No `States` machine exists anywhere in
the crate; introducing one would be a new pattern (client-estate §7 says so explicitly), and pause
must halt *only* the advance — every render/HUD system keeps running — which a bool + early return
expresses exactly and a state-gated schedule expresses clumsily.

**Phase-locked animation (S2, the Entity's heartbeat).** `advance_ticks` publishes
`TickPhase(f32)` = `accumulator / interval` ∈ [0,1). Wave 1's single consumer is a heartbeat on
the HUD tick readout — animation timed by the **sim clock, not the frame clock**, which is the
one structural commitment #639's Entity identity asks of any new chrome. It asserts nothing about
any field, so M8 is satisfied by construction.

**Its degenerate cases are specified, not discovered (Minor 4):** while paused, `TickPhase` holds
its last value and the heartbeat **stops** (an unmoving clock must look unmoving); during a
catch-up frame covering up to `MAX_TICKS_PER_FRAME` ticks, `TickPhase` completes exactly one ramp
and the readout instead shows the batch size (`+8 ticks`) — the honest render of "more happened
than one phase can show". The heartbeat is **three discrete palette steps** (`DIM` → `BONE` →
`GOLD` on the tick glyph), never an alpha fade or a scale bloom — the aesthetic ruling forbids
glow and blur (Global Constraint 12).

### 2.2 The event-feed data path — drain-with-cursor, tick-stamped, severity-ranked, copy transcribed

**Decision.** Four parts.

1. **Tick stamping happens at the advance site.** `CollectingSink`
   (`babylon-bsl/src/structural_verbs.rs:85-88`) is a bare `pub events: Vec<(String, Vec<(String,
   Value)>)>` with **no tick field**, and the client owns the sink instance. So: inside
   `advance_ticks`, immediately after each `session.advance()`, `drain(..)` the sink and push each
   event into a client-owned `BeatLog` stamped with `session.inner.tick()` — which is exactly the
   tick that produced it, because `advance` appends only during its own call.
2. **`BeatLog` is a bounded `VecDeque<Beat>`** (`const BEAT_LOG_CAPACITY: usize = 512`), oldest
   dropped. Draining rather than re-reading also closes the unbounded-memory item tracked on
   **#503** — at 25 t/s the counties story emits 12 `LIFECYCLE_TRANSITION`s per tick, ~18,000 in
   60 seconds, and the current feed re-reads the whole accumulated `Vec` every frame
   (`loop_ui.rs:333-355`).
3. **Prominence comes from the ratified severity taxonomy, transcribed.**
   `src/babylon/models/event_severity.py` derives a tier from `(kind, terminal_proximity)` by a
   pure rule (`derive_severity`, `:229`) and is *already declared* a read-only projection that
   never enters the tick hash (its own module doc, Amendment-S tripwire). Transcribe
   `derive_severity` plus the 12 rows for landed event types (from `SEVERITY_TAXONOMY`, `:285`)
   into `severity.rs`. The resulting ranking is exactly what a beat feed needs and it is not
   invented here:

   | EventType | kind | proximity | tier |
   |---|---|---|---|
   | `TERMINAL_DECISION`, `CONTROL_RATIO_CRISIS`, `CLASS_DECOMPOSITION`, `SUPERWAGE_CRISIS` | CROSSING | TERMINAL_ADJACENT | **critical** |
   | `LEGITIMATION_CRISIS` | CROSSING | TERMINAL_APPROACH | warning |
   | `LEGITIMATION_RECOVERY`, `MASS_AWAKENING`, `ENTITY_DEATH` | CROSSING | INTRA_LEVEL | informational |
   | `LIFECYCLE_TRANSITION`, `CONSCIOUSNESS_TRANSMISSION`, `VALUE_TRANSFER`, `DISPOSSESSION_EVENT` | FLOW | NA | informational (salience floor) |

   Rendering: **critical** = CRIMSON, full sentence, tick-stamped, autopause candidate;
   **warning** = GOLD, full sentence; **informational** = BONE/DIM, and same-tick FLOW events of
   one type **collapse to one line with a count** ("12 territories advanced the D-P-D′ circuit").
4. **The sentence comes from `narration.rs`, transcribed from
   `src/babylon/game/chronicle_adapter.py::_SUMMARY_BUILDERS`** (`:162`), enriched for the four
   carceral beats by the frozen systems' own `narrative_hint` copy (`decomposition.py:189-192,
   361-365`; `control_ratio.py:201-205, 222-232`). Every `{slot}` binds to a **named payload key**;
   a key the payload does not carry renders `{absent}`, never a default (this is a real case:
   `control-ratio.bsl:320-329`'s zero-enforcer branch deliberately omits `actual-ratio`); a key
   the port wrote as a declared non-computation renders through §2.6's `NotComputed` class, never
   as its literal zero. An `EventType` with no verified copy falls through to a generic
   `<EVENT_TYPE> @ <subject>` line — the same discipline `chronicle_adapter._generic_summary`
   (`:556`) already uses, for the same reason.

**Each row carries its own provenance, and that survives the Python deletion (I7).** A
`NarrationSpec` row is
```rust
pub struct NarrationSpec {
    pub event_type: &'static str,
    pub subject_key: Option<&'static str>,   // Minor 2 — declared, never first-match-wins
    pub template: &'static str,
    pub because: Option<&'static str>,       // §2.3 — the transcribed causal chain
    pub source: &'static str,                // "src/babylon/engine/systems/decomposition.py:361-365 @ p27-python-freeze"
}
```
`source` names the frozen file, its line range, **and the freeze tag**. When the Python engine
deletion ceremony lands (memory `python-engine-deletion-endgame`), the strings' provenance is
still readable from the Rust table alone. §9's ADR row records the guard's own lifecycle.

**Minor 2 — the subject key is declared per `EventType`, not guessed.**
`CONSCIOUSNESS_TRANSMISSION` carries both `source-id` and `target-id`; first-match-wins would pick
deterministically but arbitrarily. `subject_key` names the one to render (`target-id` — the node
the transmission acted *on*), next to the template that uses it.

**Minor 3 — aggregate payloads get a declared carrier scope, not "@ n/a".**
`CONTROL_RATIO_CRISIS` and `TERMINAL_DECISION` carry pure aggregates and no `NodeRef` at all
(`control-ratio.bsl:320-329, 366-379`). Their `subject_key` is `None`, and the feed renders them
at **world scope** — `@ <story.title> · world` — with the story card explaining that a
world-scoped beat is a statement about the whole run, not a missing id.

**Why.** The engine has no string payloads and must not grow them; the copy already exists,
already deterministic, already tested Python-side; and the transcription-with-a-parity-guard
pattern is this repo's established answer to an FFI boundary no import can cross
(`tests/unit/render/test_rust_theme_parity.py`'s module doc says exactly this about `palette.rs`).

**The drift guard.** A new Python test, `tests/unit/render/test_rust_narration_parity.py`, parses
`narration.rs` and `severity.rs` and asserts, for every transcribed `EventType`: (a) the severity
row's `(kind, proximity)` equals `SEVERITY_TAXONOMY`'s, (b) the template's slot names are a subset
of the wire keys `EVENT_BUILDERS`' own builder for that type reads — the identical static check
`test_chronicle_adapter.py:280::test_summary_builders_only_read_wire_keys_event_builders_also_reads`
already performs Python-side (it is what caught `CLASS_DECOMPOSITION` reading `original_id`
instead of the wire key `source_class`; see also `:128`). Copy drift then turns the **Python** gate
red.

**Rejected: teach BSL/`CollectingSink` to carry the tick and the narration string.** It is an
engine change to two crates for a presentation concern, it would put authored prose inside content
whose bytes are hashed, and §2.8's no-string-payload rule is a deliberate language boundary. The
client is the right owner; the sink is already the client's.

**Rejected: an LLM/RAG narrator for wave 1.** The Archive's narrator lane exists but is
out-of-process and non-deterministic; a beat feed must be reproducible from `(event, payload)`
alone. `chronicle_adapter`'s own module doc draws this exact line (`:20`, "NO LLM here").

### 2.3 The causal chain is shipped, not deferred (I6 — the pedagogy half of M21)

**Decision.** Every **critical** beat renders a second line — `because: …` — transcribed from the
frozen source, never authored. Four rows, all citations verified at revision 2:

| Beat | `because` line, transcribed | Source |
|---|---|---|
| `SUPERWAGE_CRISIS` | "the labor aristocracy's wealth clears the *approaching, not dying* gate — super-wages can no longer sustain the privileged stratum" | `decomposition.py:189-192`; the gate arithmetic at `carceral-arc-conformance.bscn:34-38` |
| `CLASS_DECOMPOSITION` | "triggered by `superwage_crisis`, {decomposition-delay} ticks earlier" | `decomposition.py:360` (`"trigger_event": EventType.SUPERWAGE_CRISIS.value`), delay at `.bscn:17,137` |
| `CONTROL_RATIO_CRISIS` | "triggered by `class_decomposition`, {control-ratio-delay} ticks earlier — the prisoners exceed what the enforcers can hold" | `control_ratio.py:201-205`; delay at `.bscn:18,138`; the guard chain at `control-ratio.bsl:294` |
| `TERMINAL_DECISION` | "the atomized surplus population cannot resist — average organization {avg-organization} falls short of the revolution threshold {revolution-threshold}" | `control_ratio.py:228-232` (the GENOCIDE branch's own reasoning), thresholds are payload keys |

The `{slot}`s bind to payload keys or to the story's declared delay constants (§2.5) under the
same absence discipline as every other slot. **This is the single cheapest thing in the plan that
serves the standing Director compass** — GDS ruling 11 (`…design.md:24`): pedagogy and engagement
are one criterion, not a trade-off. `carceral_arc_beats.txt` is a *test golden* read by engineers;
it is not, and was never, the pedagogy artifact. The beat card is.

**Rejected: a wiki pane / derivation affordance now.** GDS Layout B's Wiki pane is wave 2 (§10).
A transcribed causal line needs no new surface and no new copy — it rides the beat card the plan
already ships.

### 2.4 Pacing — the dead windows are the deliverable, not a side effect (C3)

**The binding standard.** GDS §3 (`…design.md:65`) pins the cadence law: mid-game
**autoplay-until-event as the default mode**, and three falsifiable gates — *"≤150 autopauses per
century; no 20-tick window with zero decisions; no dead 15-minute stretch of a mid-game session"*.
Wave 1 has no campaign and no player decisions, so the century gates are not measurable here; the
plan states the wave-1 restatement it **can** be judged against, and gates on it:

> **Wave-1 cadence claim:** on every shipped story, **no 20-tick window renders a frozen screen**.
> At least one on-screen instrument changes value on every single tick, and that instrument is a
> named engine quantity.

**How each story meets it, honestly.**

- **carceral** — the world publishes its own schedule as named engine quantities:
  `institution/superwage-crisis-tick` (written by `decomposition.bsl:270-271`),
  `institution/decomposition-fire-tick` (`decomposition.bsl:309`),
  `institution/control-crisis-tick` (`control-ratio.bsl:338`), plus the three
  `carceral/*-delay` defconsts (`carceral-arc-conformance.bscn:137-139`). Therefore
  `superwage-crisis-tick + decomposition-delay − tick` **is** the ticks-to-next-beat, computed
  from a stamped field and a declared constant. §2.5 makes the delays a transcribed, citation-
  carrying field of `Story`; §3.3 makes the countdown a pane. Ticks 2–52 stop being a frozen screen
  and become visible accumulation.
- **counties** — the opposite problem: 12 identical `LIFECYCLE_TRANSITION`s every tick forever.
  §2.2's collapse handles the volume; the **motion** comes from the fact that `pop-d`, `pop-p`,
  `pop-d-prime`, `wealth-d-prime` and `dependency-ratio` genuinely move every tick
  (`lifecycle.bsl:383-387`). So the collapsed FLOW line carries a **magnitude, not just a count** —
  "12 territories advanced the D-P-D′ circuit (Σ|Δpop-d′| = 41.7)" — and the state panel shows the
  per-tick delta beside each value. Both are named engine quantities; nothing is invented.

**The latch is a structural zero and must be read as one (I2 interaction).** Every
`institution/*-tick` latch is **seeded 0** in the `.bscn` (`carceral-arc-conformance.bscn`'s
carrier: "every latch seeded 0"). Reading `superwage-crisis-tick == 0` therefore means *not yet
latched*, never *latched at tick 0*. The countdown gates on the companion flag
(`superwage-crisis-known`, `decomposition-fired-known`, `control-crisis-emitted`), never on the
tick value, and renders `not yet latched` through §2.6's `NotComputed` class until the flag
flips. A countdown that guessed from the zero would be exactly the fabrication III.11 forbids.

**Time compression is a surfaced control, not an implementation detail.** `B` = run-to-next-beat
(beat-to-beat skip) appears in the controls legend with that name, and the HUD shows what it will
skip to when a countdown is live ("`B` → next beat in 39 ticks"). Skipping dead air and making a
run watchable are different jobs; wave 1 does both, and says which is which.

**The executable gate (Task 6.3):** across a headless auto-run of each story to its validated
horizon, assert that no 20-tick window leaves the rendered HUD + state panel + countdown text
byte-identical. Any story that cannot pass it does not ship as a story.

### 2.5 Scenario selection — a compile-time story catalog, chosen by CLI, shown as a card

**Decision.** `story.rs` declares `const STORIES: &[Story]`, each carrying:

```rust
pub struct Story {
    pub id: &'static str,
    pub title: &'static str,
    pub premise: &'static str,              // TRANSCRIBED from the .bscn header — §I1 below
    pub premise_source: &'static str,       // "content/scenarios/carceral-arc-conformance.bscn:11-23"
    pub scenario_src: &'static str,         // include_str!
    pub rule_srcs: &'static [&'static str], // include_str!
    pub session_id: &'static str,           // §3.5 — the scenario's own qname, per D179
    pub map_binding: Option<MapBinding>,    // counties: Fips; carceral: None
    pub arc: Option<StoryArc>,              // { last_tick, beat_count }
    pub validated_horizon: i64,             // §2.7 / I3
    pub delays: &'static [DeclaredConst],   // §2.4's countdown operands, each with its .bscn cite
}
```

**The roster is DERIVED, not transcribed (revision 2).** `EngineSession::start` already calls
`babylon_bsl::scenario::load_scenario` and discards the `LoadedScenario` (`engine_link.rs:71-72`).
It now binds it and reads `loaded.node_content_ids` (`scenario.rs:311`) — a real
`NodeId → content-id` map from the loader itself. `DEMO_FIPS` (`engine_link.rs:37-40`) and its
count assertion (`:78-85`) are **retired**; the FIPS join becomes a filter of the derived map over
`graph.nodes("TERRITORY")`, and the loud assertion generalizes to "the story declares
`MapBinding::Fips`; the scenario minted N TERRITORY nodes; here are the ones whose content ids do
not parse as FIPS" — a *derivation* failure, not a transcription drift. Revision 1's blocker B2 is
dissolved (§0.3).

**Selection:** `--story <id>` parsed with bare `std::env::args()` (precedent:
`babylon-tick/src/main.rs`; no `clap` dependency exists in the workspace), threaded as a
`SelectedStory` **resource inserted before Startup** (Minor 7 — `EngineSession::start` runs inside
`TickLoopPlugin`'s Startup system, `loop_ui.rs:100`, and takes no inputs today). An unknown id
**fails loud** listing the catalog. The three existing headless suites that add `TickLoopPlugin`
(`tick_loop.rs`, `eyes_on_smoke.rs`, `production_render_paths.rs`) each insert
`SelectedStory(counties)` explicitly — their world does not change and neither do their assertions.

**The default experience decision (I8), made:** **`counties` stays the default**, and **carceral
becomes discoverable by construction**. Rationale: the county atlas is the crate's only spatial
instrument and the counties world moves every tick (§2.4), so a first-run viewer who types nothing
still sees the map breathe. Carceral is one keystroke away and *advertised*: the tick-0 story card
prints the **whole catalog** (id, title, one-line premise, beat count), the HUD footer names the
current story and the `N` key, and `N` restarts into the next catalog entry with a fresh
`EngineSession` — which the catalog makes a three-line function.
**Rejected: defaulting to carceral.** It hides the map on first run and makes the crate's most
expensive shipped asset invisible to anyone who types no flag.
**Rejected: leaving carceral behind an unadvertised flag** (revision 1's shape) — the critique is
right that an undiscoverable flag is not a shipped story.

**The tick-0 story card** renders: title, premise (transcribed), `0/N beats`, the catalog, and the
full controls legend. Dismissed by the first advance; recallable with `?`.

**Why a compile-time catalog.** It keeps the `include_str!` model (no asset pipeline exists —
client-estate §8) while making "which world am I watching" a first-class datum the HUD, the beat
feed, the countdown and wave 2's lobby all read from one place.

**Rejected: a Bevy lobby screen now.** GDS §4's ratified IA does name Lobby → play screen, but a
lobby needs a `States` machine, focusable buttons and a UI layout estate that does not exist (every
HUD element today is an absolutely-positioned `Text`, `loop_ui.rs:223-246`). The catalog is the
lobby's data model; wave 2 builds the screen over it without re-deciding anything.

**Rejected: runtime file paths (`--scenario path.bscn`).** It would let the viewer load content
that never passed a conformance gate and quietly become a second source of truth about what the
game contains. The catalog is a curated list; modding is #531, post-1.0, by its own ruling.

**I1 — `premise` is transcribed, and the transcription is proved.** Revision 1 marked it
"(authored copy)", which Amendment AD reserves to the Director. Revision 2 sources it from text
that already exists and was already reviewed:

- **carceral** — `carceral-arc-conformance.bscn:11-23` (the DERIVED TICK SCHEDULE block and its
  outcome sentence, including "GENOCIDE … ip-seed's organization is UNTOUCHED by decomposition's
  intake … only lumpen's pre-existing 200 @ 0.2 contribute anything nonzero").
- **counties** — `us-counties-lifecycle-demo.bscn:1-5` (the world's own one-paragraph
  self-description: twelve real-FIPS counties carrying `lifecycle/dpd-circuit` plus the six
  `vitality/subsistence-and-death` fixture nodes).

`premise_source` carries the exact `file:line-line`. **Task 5.1 makes this executable:** for every
story, strip each premise line's leading `; ` and collapse whitespace, then assert the result is a
substring of the same normalization of the scenario source. An implementer who "improves" the
wording turns the test red. If a header quote cannot be made to render legibly inside the card's
width, **STOP** — escalate the two strings to the Director as a two-line gate (§8 B15); do not
author a replacement.

### 2.6 The admin/player seam — one projector, FOUR provenances, a declared banner

**Decision.** A new `projection.rs`: every panel reads through
`Projector::read(node, field) -> Reading`, where `Reading { value: Option<f64>, provenance:
Provenance }` and

```rust
pub enum Provenance {
    Material,                                   // the engine computed it; show the number
    Absent { reason: &'static str },            // the engine never wrote it; show {absent}
    NotComputed { reason: &'static str },       // I2 — the port declared a non-computation
    Redacted { remedy: &'static str },          // I9 — declared-dead until #593
}
```

Wave 1's projector is `Projector::material()` and constructs `Material` / `Absent` /
`NotComputed` only. The admin surface renders a persistent, declared banner —
`ADMIN · MATERIAL TRUTH · UNFOGGED` — and the state panel, beat feed, countdown and lens compute
all read through the projector.

**I2 — `NotComputed` is the render class for structural zeros, and its members are enumerated.**
A pane or narration slot that printed "desired wages: 0" would state a computed magnitude the
engine never computed — the exact fabrication III.11 forbids, arriving through the one door
revision 1 left open. The wave-1 membership list, verified at revision 2:

| Key | Where | Why it is not a number |
|---|---|---|
| `SUPERWAGE_CRISIS.desired-wages` | `decomposition.bsl:264` | bare `0.0c` — "the frozen mirror's real dollar figures don't compute here; this port emits structural zeros" |
| `SUPERWAGE_CRISIS.available-pool` | `decomposition.bsl:265` | same |
| `ENTITY_DEATH.cause` | `vitality.bsl:90-95` | the discriminant is not on the wire at all; the pack's own comment records it is re-derivable (`drained < death-threshold` ⇒ wealth_threshold, else starvation) — wave 1 does **not** re-derive it, it declares it |
| every `institution/*-tick` latch before its flag flips | `carceral-arc-conformance.bscn` carrier ("every latch seeded 0") | the 0 means *not yet latched* (§2.4) |

Rendering: the field's name, the phrase `not computed by this port`, and the citation — never a
numeral, never a dash that could read as a value. The narration templates for those slots carry
no `{slot}` at all.

**I9 — `Redacted` is registered declared-dead, not tested as live.** Revision 1 shipped a test
that constructed the variant itself and asserted it rendered — precisely the shape CLAUDE.md names
as a recurring defect class (a fixture stamping something production never emits, giving a green
test over a dead feature). Revision 2:

1. The variant stays (ADR182 R2 rules redaction-with-remedy ≠ absence; baking a two-state world in
   would foreclose the ruling).
2. Its match arm exists **because the compiler requires exhaustiveness**, and says so in its own
   comment.
3. The test is inverted: `redacted_is_declared_dead_until_593` **source-scans**
   `babylon-client/src/**` for `Provenance::Redacted` constructions and asserts there are **zero**
   outside the enum definition — the same source-scan shape `test_rust_theme_parity.py` already
   uses on the Rust tree. It counts the construct as dead; it does not imply coverage.
4. ADR-NF (next-free-at-landing; see §9.2) records it as a **declared-dead extension point gated on #593**, in the same register the
   `inert` sentinel family keeps for Python constructs
   (`src/babylon/sentinels/inert/registry.py`'s `DECLARED_PRODUCERS` doctrine — the registry itself
   scans Python production roots and cannot see Rust, which is why the Rust-side scan test is the
   mechanism and the ADR row is the record).

**Why the seam at all.** `b3-charter/data-seam.md` §1 is precise about the defect:
`refresh_state_panel` (`loop_ui.rs:273-307`) and the three lenses call `graph.node_attribute`
**directly**, so "there is no seam to insert a fog filter at; the raw read and the render are the
same call site." Wave 1's whole obligation is to create that seam, so wiring `apply_fog`'s Rust
transcription later is a filter swap rather than a rewrite. The banner discharges #617's "the admin
surface must be the *named* exception, not a forgotten unfogged panel."

**Minor 8 — the `observe()` gap is recorded, not papered over.** M4 calls the client a
"presentation-only viewport over `observe()`-projection shapes"; there is no `observe()` anywhere
in `rust/`. `Projector` reads `graph.node_attribute` directly (as B2 already does). ADR-NF (next-free-at-landing; see §9.2) records
the AF (ii) conformance argument for why a client-side projector is the wave-1 stand-in and what
would have to change if the Rust engine ever grows an `observe()` seam.

**Rejected: an `is_admin: bool` flag on the existing call sites.** That is the shape #617
explicitly warns against — a flag on a path that has no filter is decoration; the seam is the
deliverable.

**Rejected: porting `apply_fog` now.** Fog needs `DoctrineCapability`, which has **zero hits
anywhere under `rust/`** (`b3-charter/data-seam.md` §1) and no player to hide anything from. It is
wave 2+, gated on the player-verb surface (#593).

### 2.7 Validated horizons — auto-run is bounded by what has been proved (I3)

**Decision.** `Story.validated_horizon: i64` is a required field, and auto-run **stops** at it with
a declared banner: *"tick 110 is this story's validated horizon (`carceral_arc_conformance.rs:146`);
beyond it nothing has been proved about this world."* `Space` still single-steps past it, and the
HUD switches to `⚠ beyond validated horizon` — the run is never silently unverified.

Measured values:

- **carceral = 110.** `carceral_arc_conformance.rs:146` `const LAST_TICK: i64 = 110`, and its four
  suites (`the_full_carceral_arc_runs_in_order`, `the_arc_emits_each_event_exactly_once`,
  `the_arc_ends_in_genocide_with_no_organization`, and the post-session class-state cross-check
  against the frozen mirror) all run the full 110.
- **counties = 1 today, and wave 1 raises it by measuring, not by assuming.** The whole committed
  verification is one tick (`tick_goldens.rs:102` pins tick 1; `us_counties_demo.rs:22` advances
  once). Revision 1 casually specified "600 auto-run ticks" and asserted nothing about whether the
  world is numerically sane there. Task 4.3 ships
  `counties_stay_numerically_sane_to_the_validated_horizon`: across `COUNTIES_VALIDATED_HORIZON`
  ticks, every `territory/{pop-d,pop-p,pop-d-prime,wealth-d-prime,dependency-ratio}` and every
  `social-class/{population,wealth}` is **finite and non-negative**, and `dependency-ratio` is
  finite. The constant is **measured at implementation**: start at 600; if the assertion fails at
  tick K, set the horizon to the largest passing round number below K, record the measured failure
  (value, field, tick) in the PR body, and file it — a world that goes insane at tick 340 is news,
  not a reason to lower a number quietly.

**Why not just cap everything at 110.** A horizon is a claim about a world, and the two worlds have
genuinely different evidence. Declaring the number per story, sourced to the test that proves it,
is the honest form; one global cap would hide that counties has almost none.

### 2.8 How tests gate a UI crate — headless real-App assertions + text goldens, never pixels

**Decision.** Three layers, all inside `cargo test --workspace` (which already covers this crate,
`.mise.toml:1644`):

1. **Pure-function unit tests** for narration templates, severity derivation, the speed table and
   `ticks_due` (§2.1) — no Bevy, no clock.
2. **Headless real-`App` assertions** — build an `App` with `MinimalPlugins + AssetPlugin +
   MapPlugin + TickLoopPlugin`, drive **real** `KeyboardInput` messages, and read the **actual
   rendered `Text`/color components**. This is the estate's own hard-won doctrine: four separate
   module docs (`map/mod.rs`, `loop_ui.rs`, `production_render_paths.rs`, `eyes_on_smoke.rs`)
   record a prior finding where a test asserted against a hand-built fixture or a re-implemented
   copy of production logic and stayed green while the real system was gutted (`tick_loop.rs:32-38`
   names the mutation proof for the hash readout; `loop_ui.rs:404-412, 459-464` name two more).
   Every new UI system in this train gets a test at this layer, or it does not land.
   **The keypress must go through the real `KeyboardInput` message pipeline**
   (`tick_loop.rs:9-30`'s `press_key_via_real_event` helper) — a direct `ButtonInput::press()` is
   wiped by `InputPlugin`'s `PreUpdate` clear before any `Update` system sees it.
   **I4 — the clock is virtual, never wall-clock.** Any test that advances time inserts an explicit
   per-update duration (`bevy::time::TimeUpdateStrategy::ManualDuration` — **verify the exact API
   against the pinned Bevy 0.18 at implementation**, `babylon-client/Cargo.toml:21`) and states its
   assertions in **tick-domain terms** ("2 ticks are due after 2.5 intervals of accumulated sim
   time"), never in seconds of wall clock. **No test in this train may read an uncontrolled
   `Time::delta`.** The estate has paid for wall-clock tests once already (memory
   `program-15-gauntlet`: "wall-clock tests = determinism poison") and a flaky test on the gate
   that is supposed to *prove* determinism is the worst possible place to pay again.
3. **A text golden for the carceral arc.** `tests/goldens/carceral_arc_beats.txt` — the rendered
   beat lines for all 110 ticks, compared byte-for-byte. It is deterministic, platform-independent
   and diff-readable. It is a **regression artifact read by engineers**, not the pedagogy artifact
   (§2.3 corrects revision 1 on that point).

**Rejected: screenshot goldens.** They need a GPU/display CI cannot promise, and pixel output
varies with font rasterization and wgpu backend — the guard would be flakier than the thing it
guards, and it would imply a determinism claim weaker than III.7's, which is about the tick hash,
not about pixels. The text golden buys the same regression coverage where the content actually is.

**Note on the ceremony gate.** The golden lives at `rust/crates/babylon-client/tests/goldens/`,
**not** under `tests/baselines/**`, so the §6.5 `Baselines: blessed(...)` trailer is *not* owed and
the pre-push/CI ceremony gate will not fire. That is a deliberate path choice, recorded so nobody
is surprised in either direction; a PR that moves the golden still owes its drift table in the PR
body as plan discipline.

### 2.9 The engine-unchanged proof — a real equality, not a hash that does not exist (C1)

**The defect being fixed.** Revision 1's Task 6 gate read: *"the same run's tick-110 state hash
matches `carceral_arc_conformance.rs`'s."* That file contains **zero** state-hash assertions. And
the golden that landed since — `tick_goldens.rs:709 carceral_arc_conformance_hashes_are_pinned` —
pins **tick 1 only**, via `run_once`. There is no tick-110 oracle anywhere in the estate. A fresh
implementer would have invented one (self-blessing whatever the client produced) or dropped the
gate.

**Decision — three layers, each naming where its oracle value comes from.**

- **G1 — the equality (the real oracle, no golden needed).** Inside
  `babylon-client/tests/carceral_arc_story.rs`, build a fresh independent
  `TickSession::new(story.scenario_src, &joined_rules, HypergraphStore::new(),
  SessionId::new(story.session_id)?)` and advance it 110 times with its own sink. Assert that its
  per-tick `TickReport.after` **sequence** and its final `CanonicalState::state_hash` are
  byte-identical to the client `App`'s. **The oracle is the engine itself, computed live in the
  same test** — no blessed constant, no self-certification. If the client ever perturbs the engine
  (a stray write, a duplicated advance, a re-ordered rule load), this reds immediately.
- **G2 — the pin (the drift alarm).** Record that tick-110 hash as a `const` **in that same
  `babylon-client` test file** — not in `babylon-tick` (Global Constraint 6 stands). Its value is
  **measured at implementation from G1's own independent session**, and its doc comment says so in
  those words, naming G1 as the derivation. **Mutation-proof it** before committing: change one
  hex digit, confirm red, revert; and separately delete one `advance()` call, confirm red, revert.
  Record both in the PR body.
- **G3 — the beat conformance (mirrored from the suite's ACTUAL assertions).** Restate over the
  *rendered feed* exactly what `carceral_arc_conformance.rs` asserts about the engine: the four
  beats at ticks **1 / 53 / 105 / 106** (`:181-228`), **exactly once each and exactly four total**
  (`:236-269`), and `TERMINAL_DECISION`'s `outcome` = `Value::Int(0)`, the numeric GENOCIDE
  encoding (`:271-293`). The suite proves the engine; G3 proves the *viewer* reports the engine.

**One honest caveat, made executable (the C4 interaction).** G1 pins the client and the oracle to
the **same** `SessionId`, so the equality holds by construction. The separate question — whether
the story's session id could change the run at all — is answered by
`the_story_session_id_does_not_yet_move_the_run`: the same 110-tick content under **two different**
`SessionId`s produces identical hashes today, because no landed pack calls `rng-draw`
(`rg rng-draw rust/crates/babylon-tick/content/rules/` → **zero**, verified 2026-08-18). That test
is a witness, not a law: the day a pack lands an `rng-draw` call it goes red, which is the correct
and loud alarm that the session-id choice has become load-bearing. Do **not** delete it then —
re-anchor it.

**Rejected: chartering a `carceral_arc_hashes_are_pinned` tick-110 case in `tick_goldens.rs`.**
It is a `babylon-tick` change and collides head-on with Global Constraint 6. G1 gets the same
assurance with zero engine-crate edits.

### 2.10 The lens seam — a descriptor table, not a five-file match cascade

**Decision.** Replace the closed `ActiveLens` enum + five exhaustive `match` sites with a
`&'static [LensSpec]` registry:

```rust
pub struct LensSpec {
    pub id: &'static str,            // stable, used by tests and the footer
    pub label: &'static str,         // HUD label — no second hand-maintained string
    pub help: &'static str,          // what quantity this paints, by engine field name
    pub compute: fn(&LensInputs<'_>) -> LensReading,
    pub paint: LensPaint,            // CountyFill(fn(Option<f64>) -> Color) today
}
```

`ActiveLens(usize)` indexes it; `Tab` advances `(i + 1) % LENSES.len()`; `CurrentLensData` becomes
a `Vec<LensReading>` parallel to `LENSES`; `LENS_CYCLE_FOOTER` (`map/hud.rs:92`) and `lens_label`
(`:36`) are **derived from the table** instead of hand-maintained. `LensInputs<'a>` bundles
`{ graph, roster, baseline }` so the three current signatures (`lens.rs:89, 199, 251`) unify.

**Why.** Client-estate §4 counts the cost of the status quo: a new lens touches five files' worth
of exhaustive matches (`map/bands.rs:153,158`, `map/mod.rs:61`, `map/hud.rs:38,54`) plus a
hand-written footer constant. With the table it is **one row**. And the safety net gets *stronger*,
not weaker: label/help/paint are fields of the same struct, so a lens cannot exist without them,
whereas today `LENS_CYCLE_FOOTER` can silently go stale. `paint: LensPaint` (one variant,
`CountyFill`) is the declared landing site for #615's flow lens — the one line of forward structure
this wave spends, with a named consumer.

**Rejected: keep the enum, add arms.** Its stated virtue is compiler-enforced exhaustiveness ("no
wraparound bug can hide", `map/mod.rs:44`). That virtue is preserved by two cheap tests (every spec
has a unique non-empty id/label; `Tab` visits every index exactly once per cycle) and the enum's
real cost — five edit sites and a hand-maintained footer — is removed. A wave-2 lens that paints
*edges* (rent flows) cannot be an arm of a county-fill match at all.

### 2.11 Where the map goes when a story has no territory

**Decision.** `map_binding: None` (carceral) hides the county mesh and renders a declared absence
banner: *"this story has no territorial substrate — 6 nodes, 0 territories; the county map is not
applicable."* The roster panel (per-node fields for the story's own nodes) becomes the primary
surface, alongside the beat feed and the countdown pane.

**Why.** Client-estate §6 is the load-bearing finding: `carceral-arc-conformance.bscn` declares
five `SOCIAL_CLASS` + one `INSTITUTION` and **zero `TERRITORY`** nodes, while every map/lens/HUD
path is keyed on `NodeType::TERRITORY` + a FIPS string. Painting 3,222 counties `PANEL` forever
while a story runs elsewhere would be the exact "decorative surface that implies data" III.11
forbids. Hiding it and saying why is the honest render.

---

## 3. Design details worth pinning before code

### 3.1 Module layout after wave 1

```
src/
  lib.rs            + pub mod story, narration, severity, projection, coverage, ui
  engine_link.rs    EngineSession: story-driven; roster DERIVED from node_content_ids
  story.rs          Story/StoryArc/MapBinding/DeclaredConst + STORIES + CLI selection
  narration.rs      NarrationSpec table (template + because + subject_key + source)
  severity.rs       EventKind/TerminalProximity/derive_severity + 12 rows
  projection.rs     Projector/Reading/Provenance (4 variants, §2.6)
  coverage.rs       FieldCoverage table — the M22 ledger, executable (§3.4)
  lens.rs           LensSpec/LensPaint/LensInputs + LENSES registry
  ui/
    mod.rs          UiPlugin: registers the panes below
    time.rs         RunState, ticks_due, advance_ticks, TickPhase, controls readout
    beats.rs        BeatLog, drain/classify, the beat feed + beat card
    countdown.rs    the latch/countdown pressure instrument (§3.3)
    admin.rs        the declared admin banner + TickReport pane (F3)
    story_card.rs   the tick-0 card + catalog + controls legend (recallable with ?)
  loop_ui.rs        shrinks to session lifecycle + hash/tick readouts
```

`ui/` is the pane seam: a new pane is a plugin + a keybinding row, matching the crate's existing
`MapPlugin`/`TickLoopPlugin` idiom rather than inventing a framework.

### 3.2 `EngineSession`, generalized — re-cited against the merged tree (C4)

```rust
pub struct EngineSession {
    pub inner: TickSession<HypergraphStore>,
    pub sink: CollectingSink,
    pub story: &'static Story,
    pub roster: Vec<(String, NodeId)>,           // DERIVED from LoadedScenario::node_content_ids
    pub population_baseline: Vec<(String, f64)>, // territory stories only
}
```

**The constructor, verified at `rust/crates/babylon-tick/src/session.rs:52-70`:**

```rust
pub fn new(
    scenario_src: &str,
    rule_src: &str,
    mut graph: G,
    session: SessionId,
) -> Result<Self, String>
```

Four parameters. `SessionId` comes from `babylon_kernel::SessionId`
(`babylon-kernel/src/clock.rs:12,25`) and its constructor is **fallible** —
`SessionId::new(impl Into<String>) -> Result<Self, EmptySessionId>` — so every call site is
`SessionId::new(story.session_id).expect("catalog ids are non-empty")`, matching the estate's own
idiom at `engine_link.rs:130`, `us_counties_demo.rs:18`, `carceral_arc_conformance.rs:112`.
`new_with_prelude` (`session.rs:82`) takes five; wave 1 needs neither prelude story.

**Roster derivation** replaces revision 1's hand-transcription (§2.5, §0.3):
`load_scenario` (`babylon-bsl/src/scenario.rs:369`) returns `LoadedScenario`, whose
`node_content_ids: HashMap<NodeId, String>` (`:311`) is public. `EngineSession::start` already
performs that load and drops the result (`engine_link.rs:71-72`) — it now binds it. For
`MapBinding::Fips` stories the derived content ids *are* FIPS codes, so `map/bands.rs:192`'s
`atlas.index_of_fips` join is unchanged. The loud startup assertion becomes: every
`graph.nodes("TERRITORY")` id resolves in the derived map, and (for a Fips binding) every resolved
content id is a five-digit FIPS — panicking with the offending ids named. `DEMO_FIPS` and its
count check are deleted.

### 3.3 The two admin/pressure instruments

**The tick-report pane (F3) — the instrument that costs nothing.** `TickReport`
(`babylon-tick/src/lib.rs:26-48`) carries `before`/`after`/`fired`/`per_rule_fired` and is
**computed every tick and thrown away** at the client's call site (`loop_ui.rs:161-163` never binds
the `Ok` value). The admin pane renders it: total subjects fired, and the per-rule breakdown in the
engine's own ascending rule-id byte order. Zero new computation, four named engine quantities.

**Minor 1 — the FLOW cross-check states its invariant.** §2.2's collapsed `LIFECYCLE_TRANSITION`
count is cross-checked against `per_rule_fired["lifecycle/dpd-circuit"]`, but `per_rule_fired` is
per-**rule**, not per-**event-type** (`b3-charter/emission-surface.md` Part 4: *"it tells a caller
'12 territories fired `lifecycle/dpd-circuit`', not '3 counties entered crisis this tick'"*). The
equality holds only because that rule's `LIFECYCLE_TRANSITION` emit is **unconditional and
one-per-subject** (`lifecycle.bsl:388-393`) — while the *same rule* also emits the guarded
`LEGITIMATION_CRISIS`/`_RECOVERY`. The test asserts the equality **and** its doc comment states
that invariant, so the cross-check teaches the real identity rather than a false one. A future rule
that emits `LIFECYCLE_TRANSITION` conditionally breaks the equality, correctly.

**The countdown/pressure pane (§2.4).** For a story declaring `delays`, it renders one row per
pending beat:

```
next beat   CLASS_DECOMPOSITION      in 39 ticks
            superwage-crisis-tick 1  + carceral/decomposition-delay 52  − tick 14
```

Both operands named: the left one a live field read through the projector, the right one a
`DeclaredConst { name, value, source }` transcribed from the `.bscn` with its line cite. Before the
latch flag flips, the row renders `not yet latched` through `Provenance::NotComputed` — never a
guessed countdown from the seeded 0 (§2.4).

### 3.4 The coverage ledger — M22, made executable (I5)

**The obligation.** The register's M22 is a MUST: *"a null-hypothesis viewer that silently omits a
computed field without declaring it is itself a wiring-completeness defect"*, dispositions being
"visual home, or explicitly RULED-ABSENT per GDS §9 W.2". Revision 1's §9 was a thematic
out-of-scope list by *instrument*, not a per-quantity ledger — it did not discharge M22.

**Decision.** `coverage.rs` declares

```rust
pub struct FieldCoverage {
    pub story: &'static str,
    pub field: &'static str,        // "territory/wealth-d-prime"
    pub written_by: &'static str,   // "lifecycle.bsl:386"
    pub home: Home,
}
pub enum Home {
    Panel(&'static str),      // a registered pane id
    Lens(&'static str),       // a registered LensSpec id
    BeatCard(&'static str),   // a registered EventType
    Countdown,
    RuledAbsent { reason: &'static str, future_home: &'static str },
}
```

with **one row per field the wave-1 stories write**. The enumeration is transcribed from the packs'
own reads/writes tables — `decomposition.bsl:53-76`, `control-ratio.bsl:34-52`, plus
`lifecycle.bsl:383-409` and `vitality.bsl:78-89` — so it is a derivation, not a survey. The roster
the implementer must cover, verified at revision 2:

- **counties / lifecycle.bsl:** `territory/{pop-d, pop-p, pop-d-prime, wealth-d-prime,
  dependency-ratio, legitimation-index, legitimation-crisis, transmitted-ideology}`.
- **counties / vitality.bsl:** `social-class/{wealth, active, population}`.
- **carceral / decomposition.bsl:** `social-class/{la-census-population, la-census-wealth,
  la-approaching-flag, la-dying-flag, population, wealth, active}`;
  `institution/{superwage-crisis-known, superwage-crisis-tick, decomposition-fire-tick,
  decomposition-fired-known, decomposition-complete, la-population, la-wealth,
  la-approaching-count, la-dying-count, enforcer-pop-gain, ip-population, enforcer-wealth-gain,
  ip-wealth}`.
- **carceral / control-ratio.bsl:** `social-class/{enforcer-census-population,
  prisoner-census-population, prisoner-census-org-weighted}`;
  `institution/{enforcer-population, prisoner-population, prisoner-org-weighted,
  control-crisis-emitted, control-crisis-tick, terminal-decision-emitted}`.

Roughly 40 rows across two packs — cheap now, and exactly the artifact M22 asks for.

**The gate (Task 9.1):** a test asserts (a) every field named in the transcribed roster appears
**exactly once**, (b) every `Panel`/`Lens`/`BeatCard` home names an id that actually exists in the
shipped registries, and (c) every `RuledAbsent` row carries a non-empty reason *and* a named future
home. ADR-NF (next-free-at-landing; see §9.2) renders the table. Task 9.3's #619 comment then reports at field granularity, which is
what that issue's lettered orphans actually need.

### 3.5 The per-story `SessionId` — the decision, made (C4)

**The obligation.** `TickSession::new` now takes a `SessionId` (§3.2). Choosing one per story is a
determinism decision, and revision 1 did not make it. Today the client hardcodes
`SessionId::new("babylon-client-b2-demo")` (`engine_link.rs:130`) while
`carceral_arc_conformance.rs:112` uses `"run-once"`.

**Decision: each story's `session_id` is its scenario's own qualified name**, transcribed from the
`.bscn`'s `(scenario …)` form:

| Story | `session_id` | Source |
|---|---|---|
| counties | `lifecycle/us-counties-demo` | `us-counties-lifecycle-demo.bscn:61` |
| carceral | `carceral/arc-conformance` | `carceral-arc-conformance.bscn:91` |

Both values were **read out of the `.bscn` files at revision 2**, not derived from the file names —
note that the counties qname is `lifecycle/us-counties-demo`, which is *not* the shape the file
name suggests. Re-read both at implementation rather than reconstructing them.

**Why this and not something else.** D179 (ADR213's session-id provenance row) admits exactly two
conventions for a live run — *"the `ContentDigest` hex **or the scenario id**, minted client-side,
never a UUID or a wall-clock read"* — and ADR213's own follow-on (iii) names adopting that
convention at `babylon-client`'s call site as open work. The scenario id is deterministic, stable
across rebuilds, human-legible in a log line, and unique per story by construction (the catalog's
uniqueness test covers it). The `ContentDigest` hex is the stronger convention for a *campaign*
with mutable content; a curated compile-time catalog of two fixed scenarios does not need it, and
a hex would make the log line unreadable for no gain. **Wave 1 therefore closes ADR213 follow-on
(iii)**, and §9's ADR row says so.

**The placeholder dies with it.** `SessionId::new("babylon-client-b2-demo")` is deleted, not left
beside the new path.

**Consequence for §2.9.** The carceral story's id (`carceral/arc-conformance`) differs from the
conformance suite's (`run-once`). G1 pins both sides to the story's id, so the equality is exact;
`the_story_session_id_does_not_yet_move_the_run` (§2.9) makes the independence executable rather
than assumed.

### 3.6 Autopause and the recognizer presentation (C2)

**`AutopauseMode ∈ { Never, OnCritical }`**, defaulting to `OnCritical` (§2.1). `B`
(run-to-next-beat) sets `running = true` + `OnCritical`, and `advance_ticks` stops the batch the
moment a `critical` beat lands. This is not an invented mechanic: `event_severity.py`'s own docs
frame tiers as autopause pressure ("TERMINAL_APPROACH … the ruling-12 density fix: autopause
pressure drops without silencing a single genuine entry").

**`TERMINAL_DECISION` is a recognizer presentation, not a terminator.** Revision 1 rendered it as
"the terminal end card" that halted the run and declared an outcome. That is wrong twice over, and
both errors are on the ideological line:

1. **GDS §1 (`…design.md:39`) rules the five patterns "*recognizers*, never terminators."** An end
   card that halts and declares an outcome is exactly a terminator surface. Wave 1 correctly defers
   the Verdict Watch; it may not substitute a *harder* framing than the one it deferred.
2. **`TERMINAL_DECISION` is not one of the five canonical outcomes.** It is `control-ratio.bsl`'s
   ADR070-reserved branch (`:341,366-379`). Dressing it as "the end" tells the player a single
   system's latch is the campaign verdict — an ideological framing claim, and the five canonical
   outcomes are Director-reserved (CLAUDE.md §Git & commits, Amendment AD).

**What wave 1 ships instead — the latch card.** On `TERMINAL_DECISION`: autopause (unconditional),
the beat rendered CRIMSON with its transcribed sentence and its `because:` line (§2.3), and a card
that reports **what the latch is**:

```
LATCH  control-ratio/c04-terminal  ·  tick 106
       institution/terminal-decision-emitted  0 → 1
       outcome 0        (this pack's numeric GENOCIDE encoding — control-ratio.bsl:366-379)
       avg-organization 0.0563   revolution-threshold 0.5

This is one system's terminal decision, not a campaign verdict.
The five canonical outcomes and the Verdict Watch are not computed here.
Press P to keep running.
```

Every line is a named engine quantity or a citation. The run **continues** on `P`/`Space` —
nothing is over. No "end", no verdict, no five-outcome vocabulary anywhere on the screen.

**STOP row (§8 B14).** If a terminal/verdict/ending surface is wanted, it is a **director-gate
question**, not a wave-1 engineering call. Anyone who finds themselves designing an end screen
stops and escalates.

---

## 4. Tasks

Each task: **RED step → GREEN step → gate**. Do not proceed past a red gate.

### Task 0 — Governance, measurement, and a verified starting line (no production code) — ~0.15 Mtok

- **0.1** Open the wave-1 issue (§0.1) under the `Bevy B3 — Client Completion` milestone; comment
  on #577 (3D stays the capstone; #648 is its gate) and on #617 (which ingredients land here,
  which do not — and that wave 1 does **not** discharge it, Minor 9).
- **0.2** **Measure the pedantic debt before promising it:** from `rust/`, run
  `cargo clippy -p babylon-client --all-targets --locked -- -D warnings -D clippy::pedantic
  2>&1 | rg '^(warning|error)' | wc -l` and record the count and the top lint families in the
  wave-1 issue. This number decides whether Task 1 is a two-line `.mise.toml` edit or a fix pass.
- **0.3** Run `cargo test -p babylon-tick --locked carceral_arc` **and**
  `cargo test -p babylon-tick --locked carceral_arc_conformance_hashes_are_pinned`, and paste both
  passes into the issue — the four-beat schedule this train ships as a story, and the tick-1
  golden that is **not** the tick-110 oracle (§2.9), verified before either is built on.
- **Gate:** issue open; the pedantic count recorded; both carceral runs green on this tree.

### Task 1 — Put `babylon-client` on the pedantic + doc gate — ~0.3 Mtok

- **1.1 RED.** Add to `.mise.toml`'s `rust:check` run-block, after the `babylon-graph` leg
  (`:1649`):
  ```
  cargo clippy -p babylon-client --all-targets --locked -- -D warnings -D clippy::pedantic
  ```
  **One line only** — `cargo test -p babylon-client --locked` is redundant, since `:1644` already
  runs `cargo test --workspace --locked` (Minor 5). Run `mise run rust:check`. If Task 0.2's count
  is nonzero, this is red **by design** — commit it red with the count in the message.
- **1.2 GREEN.** Fix what it flags. Expected families: `#[must_use]`, missing `# Errors`/`# Panics`
  rustdoc, `cast_precision_loss`/`cast_possible_truncation` (real in tessellation/atlas code),
  `module_name_repetitions`, `doc_markdown`. **Rule:** fix the lint where it is a real defect; where
  a cast is deliberate and bounded, use an inline `#[allow(...)]` **with a one-line cited reason**,
  never a blanket crate-level allow. If one pre-existing file (likely `atlas.rs` at 746 lines or
  `tessellate.rs` at 523) proves disproportionate, a **file-scoped** `#![allow]` with a reason plus
  a follow-up issue is the sanctioned escape — record it in the PR body, do not hide it.
- **1.3 GREEN.** Confirm `RUSTDOCFLAGS='-D warnings' cargo doc` already covers the crate
  (the `--workspace` leg, `.mise.toml:1650`) — it does; no change needed, but state it in the PR
  body so the "doc -D warnings from day one" requirement is evidenced rather than assumed.
- **Gate:** `mise run rust:check` fully green, pedantic leg included. Every `#[allow]` added
  carries a reason.

### Task 2 — The clock, the pacing defaults, and the virtual-time discipline — ~0.7 Mtok

- **2.1 RED — pure arithmetic, no Bevy.** `ticks_due(accumulator, interval, max) -> (usize, f32)`
  tests, all in tick-domain terms: 2.5 intervals accumulated yields exactly 2 ticks and 0.5
  remaining; 30 intervals accumulated yields exactly `max` ticks and a **clamped** remainder
  (no 1,000-tick fast-forward after a stall); a zero/negative accumulator yields 0.
- **2.2 RED — the wired clock.** `tests/time_controls.rs`, all failing, each advancing time via an
  explicit injected per-update duration (§2.8/I4 — never `Time::delta` off a wall clock), each
  keypress driven through the real `KeyboardInput` pipeline:
  1. `RunState`'s **defaults** are `running = true`, `autopause = OnCritical`, `speed_index = 2`
     — autoplay-until-event, per GDS §3 (`…design.md:65`). Assert the defaults directly; they are
     a design decision, not an implementation detail (Minor 6).
  2. `P` toggles `RunState.running`; the HUD control readout text changes to match.
  3. At speed index 0 (1 t/s), 2.5 intervals of injected sim time advance **exactly** 2 ticks.
  4. `,`/`.` move `speed_index` within `[0, SPEEDS_PER_SECOND.len())` and **saturate** at both
     ends (no wraparound — a wrap from 25 t/s to 1 t/s is a usability trap).
  5. `Space` advances exactly one tick **whether paused or running**, and does not double-advance
     on a frame where the auto-run timer also elapsed.
  6. **Determinism:** two `EngineSession`s over the same story — one advanced by 10 discrete
     `Space` presses, one by a single auto-run batch of 10 — produce **byte-identical**
     `state_hash` at tick 10, and identical per-tick hash sequences.
  7. The lens recompute + `LensChanged` fire **once** per frame regardless of how many ticks the
     batch advanced (assert with a message counter).
  8. **Heartbeat degenerate cases (Minor 4):** paused ⇒ `TickPhase` frozen and the readout static;
     a catch-up frame of K > 1 ticks ⇒ exactly one phase ramp and the readout shows `+K ticks`.
- **2.3 GREEN.** `ui/time.rs`: `RunState`, `SPEEDS_PER_SECOND`, `MAX_TICKS_PER_FRAME`, `ticks_due`,
  `TickPhase`, `advance_ticks` (replacing `advance_on_space`), the controls readout entity.
  Re-point `loop_ui.rs`'s `.after(advance_on_space)` guards at the new system name — the two
  ordering fixes documented at `loop_ui.rs:64-96` must survive the rename verbatim.
- **2.4 GREEN.** HUD: `▶ 5 t/s · tick 37` (or `❚❚ paused · tick 37`), plus `· arc 110` when the
  story declares one, plus the story name and the `N` key (§2.5). Heartbeat on the tick readout
  driven by `TickPhase`, three discrete palette steps (§2.1).
- **Gate:** `mise run rust:check` green; `tests/determinism.rs`, `tests/tick_loop.rs` and
  `tests/eyes_on_smoke.rs` still green unmodified except for the system rename; row 6's
  hash-equality assertion passing is the train's determinism evidence.

### Task 3 — The projection seam and the declared admin surface — ~0.6 Mtok

- **3.1 RED.** `tests/projection.rs`: `Projector::material().read(id, "territory/pop-d")` returns
  `Reading { value: Some(_), provenance: Material }`; an unwritten field returns
  `{ None, Absent { .. } }` (never `Some(0.0)`); a key in §2.6's `NotComputed` list renders its
  reason and **contains no digit** (assert on the rendered string); and
  `redacted_is_declared_dead_until_593` **source-scans** `src/**` and asserts zero
  `Provenance::Redacted` constructions outside the enum definition (I9 — the variant is counted
  dead, never fixture-constructed). Plus a headless assertion that the admin banner entity exists
  and reads `ADMIN · MATERIAL TRUTH · UNFOGGED`.
- **3.2 GREEN.** `projection.rs` with the four `Provenance` variants; retarget `refresh_state_panel`
  (`loop_ui.rs:273-307`) and the three lens computes through it. Behavior must not change for
  `Material` reads — the same numbers on screen, through one call site instead of eleven.
- **3.3 GREEN.** `ui/admin.rs`: the banner, `F3` toggling the `TickReport` pane (§3.3) with the
  per-rule breakdown in ascending rule-id byte order, and the raw roster dump for the selected node.
- **Gate:** `rust:check` green; `production_render_paths.rs` still green (it asserts the panel's
  *rendered* text — the refactor is behavior-preserving or the test says otherwise).

### Task 4 — Narrative beats, with the causal chain — ~1.05 Mtok

- **4.1 RED — severity.** `severity.rs` tests: `derive_severity` reproduces all four rule arms
  (ALARM→critical; CROSSING×{ADJACENT,APPROACH,INTRA}→{critical,warning,informational}; FLOW/ACT→
  declared floor, **never critical**); the 12 transcribed rows resolve to §2.2's table; a CROSSING
  row with `proximity = NA` is a **loud error**, not a default (`event_severity.py:229`'s own
  `ValueError` arm).
- **4.2 RED — narration.** `narration.rs` tests: `CLASS_DECOMPOSITION` with the frozen mirror's
  own tick-53 payload renders exactly *"CLASS DECOMPOSITION: Labor aristocracy collapses. 90 become
  guards/cops. 510 fall into the precariat."*; `TERMINAL_DECISION` with `outcome = 0` renders the
  GENOCIDE copy and with `outcome = 1` the REVOLUTION copy (`control_ratio.py:222-232`); a payload
  missing a slot key renders `{absent}` for that slot **and nothing else changes**; a
  `NotComputed` key renders its reason and no numeral; an unknown `EventType` renders the generic
  line. **Plus the `because:` line (§2.3):** each of the four critical rows renders its transcribed
  causal sentence with its delay/threshold slots bound; a beat with no `because` row renders no
  second line at all (never an empty one).
- **4.3 RED — the wired feed, and the horizon.** Headless real-`App`: after N presses on the
  counties story, the rendered `BeatFeedText` contains a tick-stamped `LEGITIMATION_RECOVERY`
  sentence (not `"LEGITIMATION_RECOVERY @ 01013"`); 12 same-tick `LIFECYCLE_TRANSITION`s render as
  **one** collapsed line whose count equals that tick's `per_rule_fired["lifecycle/dpd-circuit"]`
  (with Minor 1's invariant stated in the test's doc comment) and whose magnitude term is a real
  Σ|Δ| over `territory/pop-d-prime`; the `BeatLog` never exceeds `BEAT_LOG_CAPACITY` after a full
  horizon of auto-run ticks; the sink is drained (its `events` length stays bounded — the #503
  memory item, made executable). **And `counties_stay_numerically_sane_to_the_validated_horizon`
  (§2.7/I3):** across `COUNTIES_VALIDATED_HORIZON` ticks every listed field is finite and
  non-negative. Measure the constant; if it fails at tick K, lower the horizon and **record the
  measured failure in the PR body** — do not silently shrink the number.
- **4.4 GREEN.** `severity.rs`, `narration.rs`, `ui/beats.rs` (drain-with-cursor, `BeatLog`,
  collapse rule, severity→color, the panel, the beat card). Retire `refresh_event_feed`
  (`loop_ui.rs:333-355`) and `EVENT_FEED_DEPTH` (`:215`); replace `payload_node_id` (`:312-323`)
  with the **declared** `subject_key` lookup (Minor 2), and render `subject_key: None` beats at
  world scope (Minor 3).
- **4.5 GREEN — autopause and the latch card (§3.6, C2).** `AutopauseMode`, `B` =
  run-to-next-beat, unconditional pause on `TERMINAL_DECISION`, and the **latch card** — never an
  end card, never a verdict, never the five-outcome vocabulary. Test: from tick 0 on the carceral
  story, one `B` press stops at tick 1 (`SUPERWAGE_CRISIS`, critical); a second at 53; a third at
  105; a fourth at 106 and renders the latch card. **Assert the negative too:** the rendered latch
  card contains none of `REVOLUTIONARY_VICTORY`/`FASCIST_CONSOLIDATION`/`RED_OGV`/
  `ECOLOGICAL_COLLAPSE`/`FRAGMENTED_COLLAPSE`/`verdict`/`campaign`, and the run resumes on `P`.
- **4.6 GREEN — the parity guard.** `tests/unit/render/test_rust_narration_parity.py`, modelled
  line-for-line on `test_rust_theme_parity.py`: parse the two Rust tables, assert severity rows
  match `SEVERITY_TAXONOMY` and that every template slot is a wire key `EVENT_BUILDERS` reads for
  that `EventType`. **Mutation-validate it**: change one Rust template slot name, confirm the
  Python test fails, revert.
- **Gate:** `rust:check` green; `mise run check` green (the new Python test runs in `test:unit`);
  the mutation table for 4.6 and the measured counties horizon in the PR body.

### Task 5 — The story catalog — ~0.7 Mtok

- **5.1 RED.** `tests/story.rs`: `STORIES` contains `counties` and `carceral` with unique ids and
  unique `session_id`s; `Story::by_id("nope")` is an `Err` naming the catalog; **the roster is
  derived** — for each story, `load_scenario` + `node_content_ids` resolves every node the
  scenario mints, and for `MapBinding::Fips` every resolved content id is a five-digit FIPS;
  `counties` declares `MapBinding::Fips` and `carceral` declares `None`; every story's
  `validated_horizon` is positive; every `DeclaredConst` names a source that is a real
  `file:line`. **And the premise-provenance test (I1):** each `premise`, normalized (strip leading
  `; `, collapse whitespace), is a substring of the same normalization of its `scenario_src`.
- **5.2 GREEN.** `story.rs` + `EngineSession::start(story)` (§3.2) with the derived roster and the
  `SessionId` from `story.session_id` (§3.5) — delete `DEMO_FIPS`, its count assertion, and the
  `"babylon-client-b2-demo"` placeholder. `--story <id>` via `std::env::args()` in `main.rs`,
  threaded as a `SelectedStory` resource inserted before Startup; unknown id → loud exit listing
  the catalog. Give `tick_loop.rs`, `eyes_on_smoke.rs` and `production_render_paths.rs` an explicit
  `SelectedStory(counties)` (Minor 7).
- **5.3 GREEN.** `ui/story_card.rs`: the tick-0 card (title, premise, `0/N beats`, **the whole
  catalog**, the full controls legend), dismissed on first advance, recallable with `?`; the `N`
  key restarts into the next catalog entry (I8). The beat counter increments only on
  `critical`/`warning` beats belonging to the story's arc.
- **5.4 GREEN — the map's honest absence.** `MapBinding::None` hides the fill/border meshes and
  renders the §2.11 banner. Test: launching `carceral` leaves zero counties painted and the banner
  present — and **no county is painted a stale color** (the FB1 class of bug, one axis over;
  `map/bands.rs:171-190` documents the original).
- **Gate:** `rust:check` green; `eyes_on_smoke.rs` still green on the counties story (the default
  must not change behavior for anyone who types no flag).

### Task 6 — The countdown instrument and the cadence gate — ~0.45 Mtok

- **6.1 RED.** `tests/countdown.rs`: with the carceral story at tick 14, the countdown row for
  `CLASS_DECOMPOSITION` reads 39 ticks and names both operands (`superwage-crisis-tick` = 1 read
  live; `carceral/decomposition-delay` = 52 from the story's `DeclaredConst` with its `.bscn`
  cite). **Before** the latch flag flips, the row renders `not yet latched` via
  `Provenance::NotComputed` and **contains no digit that could read as a countdown** — the seeded-0
  trap (§2.4). After a latch fires, its row retires and the next pending beat's row appears.
- **6.2 RED.** The HUD's `B` hint reads `B → next beat in 39 ticks` when a countdown is live, and
  omits the hint entirely when none is (never `in ? ticks`).
- **6.3 RED — the cadence gate (§2.4).** For **each** shipped story, auto-run headless to the
  story's validated horizon capturing the rendered HUD + state panel + countdown text every tick,
  and assert **no 20-tick window is byte-identical**. This is the executable form of the wave-1
  cadence claim; a story that cannot pass it does not ship.
- **6.4 GREEN.** `ui/countdown.rs` + `Story.delays` transcription. Counties declares no delays; its
  cadence comes from the per-tick deltas (§2.4) — assert that path separately so 6.3 cannot pass
  for counties by accident of some unrelated moving glyph.
- **Gate:** `rust:check` green; 6.3 green on both stories; the per-story cadence evidence
  (the longest identical window found, per story) in the PR body.

### Task 7 — The carceral arc as the first shipped story — ~0.75 Mtok

- **7.1 RED — the golden.** `tests/carceral_arc_story.rs` + `tests/goldens/carceral_arc_beats.txt`
  (empty at red): drive the real `App` on the carceral story for 110 ticks under auto-run, collect
  the rendered beat lines **including the `because:` lines**, compare byte-for-byte.
- **7.2 RED — the arc through the UI (G3, §2.9).** Assertions distinct from `babylon-tick`'s own
  conformance suite (which proves the *engine*; these prove the *viewer*), mirroring that suite's
  actual assertions: the four beats appear in the rendered feed in tick order **1 / 53 / 105 /
  106** (`carceral_arc_conformance.rs:181-228`); **each exactly once and four total**
  (`:236-269`); the tick stamps are the engine's ticks, not frame counts; `TERMINAL_DECISION`
  carries `outcome = Value::Int(0)` (`:271-293`); the latch card renders and the run is paused at
  tick 106 with no verdict vocabulary on screen (Task 4.5's negative assertion, restated
  end-to-end).
- **7.3 RED — the engine-unchanged proof (G1, §2.9).** A fresh independent
  `TickSession::new(story.scenario_src, &joined_rules, HypergraphStore::new(),
  SessionId::new(story.session_id)?)` advanced 110 times produces a per-tick `TickReport.after`
  sequence and a final `state_hash` **byte-identical** to the client `App`'s. Plus
  `the_story_session_id_does_not_yet_move_the_run`: the same content under two different
  `SessionId`s hashes identically today (no landed pack calls `rng-draw`) — a witness that goes
  loudly red when that stops being true.
- **7.4 GREEN.** Wire the story; fill the golden from the first green run and pin it thereafter
  (the `rng.rs:191-192` precedent — a later divergence is a regression, never "the text improved").
  **Pin the tick-110 hash (G2)** as a `const` in this file, its value taken from 7.3's independent
  session, its doc comment naming that derivation. **Mutation-proof both**: flip one hex digit →
  red → revert; delete one `advance()` → red → revert. Record both in the PR body.
- **7.5 GREEN — the roster panel.** For a story with no map, the selected-node panel lists the
  derived roster and shows the story's own published fields through the projector — for the
  carceral world: `social-class/{population, wealth, organization, active}` and the carrier's
  `institution/{enforcer-population, prisoner-population, decomposition-fire-tick, …}`, each
  through its correct `Provenance`. Selection by `↑`/`↓` through the roster (no county to click).
- **Gate:** `rust:check` green; the golden diff empty; **G1's equality green, G2's pin
  mutation-proven, G3's beat conformance green** — the three together are the "the viewer changed
  nothing about the engine" claim, and no step of it cites a hash that does not exist.

### Task 8 — The lens registry — ~0.5 Mtok

- **8.1 RED.** `tests/lens_registry.rs`: `LENSES` ids are unique and non-empty; every spec's
  `help` names at least one engine field string that appears in `lens.rs`; `Tab` visits every index
  exactly once per full cycle then returns to the start; the derived footer string contains every
  lens label; `CurrentLensData.len() == LENSES.len()`.
- **8.2 GREEN.** `LensSpec`/`LensPaint`/`LensInputs`, `LENSES`, `ActiveLens(usize)`; derive
  `lens_label`/`LENS_CYCLE_FOOTER` (`map/hud.rs:36,92`) from the table; collapse the matches at
  `map/bands.rs:153,158`, `map/mod.rs:61`, `map/hud.rs:38,54`, and `loop_ui.rs`.
- **8.3 GREEN.** Keep the **Tension lens in the registry**, whole-lens-absent, with its `help`
  string naming the two fields no landed pack writes (`lens.rs:62-63`,
  `territory/tick-exploitation-rate` and `territory/tick-total-surplus`) and its `absent_reason`
  on screen (`lens.rs:111-115`). It is the honest placeholder for #615, and deleting it would
  erase the reserved field names the economics port is supposed to write.
- **Gate:** `rust:check` green; `eyes_on_smoke.rs`'s Tab-cycle and stale-color tests green
  unmodified in substance (index-based cycling replacing enum cycling is the only diff).

### Task 9 — The coverage ledger, the ADR, closeout — ~0.55 Mtok

- **9.1** `coverage.rs` + its gate (§3.4): the ~40-row `FieldCoverage` table transcribed from the
  four packs' own reads/writes tables, and the test asserting completeness, real homes, and
  reasoned absences. **This is the M22 artifact** — it is a task, not a paragraph.
- **9.2** ADR — **allocate the NEXT FREE number at landing, verified against dev at that
  moment** ("ADR-NF" throughout this plan is that placeholder). Known at planning time:
  `ai/decisions/` on dev tops out at ADR213 (#657), and **ADR214 is already held by the #334
  train's unpushed branch, which lands first** (recorded in this plan's own ledger) — so the
  projected number is ADR215, but the implementer verifies rather than assumes. Plus an
  `ai/decisions/index.yaml` row. It records:
  - the wave-1 scope verdict (§0.1) and what the merge dissolved (§0.3);
  - the engineering calls (§2) each with its rejected alternative;
  - **the per-field coverage table (§3.4/I5)**, rendered in full;
  - the per-story `SessionId` decision (§3.5) — and that it **closes ADR213 follow-on (iii)**;
  - the narration-transcription contract, its parity guard, and **the guard's lifecycle across the
    Python deletion ceremony (I7)**: at that ceremony the Rust tables become the declared SoT and
    `test_rust_narration_parity.py` retires **by named ceremony**, with provenance preserved by
    each `NarrationSpec.source` row's `file:line @ p27-python-freeze` citation;
  - `Provenance::Redacted` as a **declared-dead extension point gated on #593** (I9), with the
    source-scan test named as its counting mechanism;
  - the **`observe()` conformance gap** and the AF (ii) argument for the client-side projector
    (Minor 8);
  - the admin surface as the *named* AF-measured exception (B14), and the #617 divergence
    (always-on with a banner rather than behind a flag — Minor 9).
  `related:` ADR170, ADR182, ADR186 (AF), ADR212, ADR213; `supersedes: []`.
- **9.3** `ai/state.yaml` closing entry. Update the client-status paragraph in `CLAUDE.md`: the
  client is still a viewer, not a game (no player verbs — #593), but it now *runs unattended,
  narrates with its causal chain, paces itself, and ships a story*.
- **9.4** Close the wave-1 issue with evidence: PR links, the golden's four beats quoted, G1/G2/G3
  (§2.9), the mutation tables, the measured counties horizon, the per-story cadence evidence, the
  pedantic-leg diff. Comment on **#503** that the unbounded event-feed memory item is closed by the
  drain (§2.2) with the test that proves it. Comment on **#619** at **field granularity** from the
  coverage ledger — which fields now have a visual home and which are RULED-ABSENT with their named
  future home. Comment on **#576** that ADR213 follow-on (iii) is closed (§3.5).
- **9.5** Revise the estimate against actual (#255 conventions).
- **Gate:** `vale` clean on every edited Markdown; `mise run check:quick`; ADR-NF (next-free-at-landing; see §9.2) present with its
  index row; the coverage-ledger test green.

---

## 5. Gates

### Per-commit
```bash
mise run rust:check     # fmt --check; clippy -D warnings -D cognitive_complexity workspace-wide;
                        # cargo test --workspace; pedantic legs (now incl. babylon-client);
                        # RUSTDOCFLAGS='-D warnings' cargo doc --workspace
```
Single-flight **within this worktree** (one `rust/target`, one file lock). The tree's own target
dir means it does not contend with other worktrees — that exemption is per-tree, not per-machine.

### Per-PR
```bash
mise run check                    # Python lane: lint + format + typecheck + test:unit
mise run qa:regression            # must be BYTE-IDENTICAL — nothing Python moved
mise run qa:vault-regression-ci   # separate estate, same reason
```
Plus, from `rust/`: `cargo test -p babylon-tick --locked` — **every pinned engine hash
byte-identical in every PR, measured against dev tip `b5a3268a` or later**, never against the
plan's old base. No BSL, no `.bscn`, no engine crate is touched by this train, so any hash movement
means something leaked out of the client. Treat movement as a **STOP**, never as expected drift.

### Ceremonies
**None owed.** No file under `tests/baselines/**` is written (§2.8 records why the Rust golden's
path is deliberately outside it). If a Python baseline moves, **STOP** — it means a change escaped
into `src/babylon/`.

### Merge protocol (ADR181)
Per PR: verify every check completed and `headRefOid == headSha`; **harvest the Copilot review**
(wait for it; fix or reply to every inline comment — zero unaddressed is a merge precondition);
merge with `mise run pr:merge -- N`, the only sanctioned path. Branch from `dev`. Never
`--delete-branch` while a child PR is stacked (#193).

---

## 6. PR structure

Three PRs, stacked, each independently green and each shippable on its own.

| PR | Tasks | Branch | Scope | Why separable |
|---|---|---|---|---|
| **A** | 0, 1, 2, 8 | `feature/b3-clock-and-lens-registry` | Pedantic gate for the crate; the run/pause/speed/step clock with autoplay-until-event defaults, the virtual-clock test discipline and the determinism proof; the lens descriptor registry | Pure client architecture on the **existing** demo world — no content change, no new copy, no new ideological surface. Reviewable as "the loop got a clock that runs itself, and the lenses got a table." Delivers a watchable run on its own. |
| **B** | 3, 4 | `feature/b3-projection-seam-and-beats` | The projector with its four provenances (incl. the structural-zero class), the declared admin surface + `TickReport` pane, `severity.rs`, `narration.rs` with the transcribed `because:` chain, the drained tick-stamped `BeatLog`, autopause + the recognizer latch card, the Python parity guard | Fully exercisable on landed counties content (`LIFECYCLE_TRANSITION`, `LEGITIMATION_*`, `ENTITY_DEATH`). The seam and **all the copy** are reviewed before a second story rides on them — and C2's framing change is reviewable in isolation, which is where it belongs. |
| **C** | 5, 6, 7, 9 | `feature/b3-story-catalog-carceral-arc` | The story catalog with derived rosters + per-story `SessionId` + validated horizons + transcribed premises, the countdown/pressure instrument and the cadence gate, the carceral arc wired with its text golden and the three-layer engine-unchanged proof, the coverage ledger, ADR-NF (next-free-at-landing; see §9.2), closeout | The payload, landing on two seams that are already green. Its acceptance is a byte-identical golden, an in-test engine equality, and a cadence gate — not a hash that does not exist. |

Commit scopes: `feat(client)`, `refactor(client)`, `test(client)`, `chore(rust)`, `docs(decisions)`,
`docs(state)`. Conventional commits, `Co-Authored-By` trailer on every one.

---

## 7. Execution order note

Task 6's countdown consumes Task 5's `Story.delays`, and Task 7's gates consume Task 5's
`session_id`. Within PR C the order is **5 → 6 → 7 → 9**. Across PRs, B's `Provenance::NotComputed`
is a hard prerequisite for 6.1's "no digit before the latch" assertion — do not start PR C before
PR B is green.

---

## 8. Blockers and escalations — stated, not planned around

| # | Item | Severity | Disposition |
|---|---|---|---|
| **B1** | **`carceral-arc-conformance.bscn` has ZERO `TERRITORY` nodes** (5 SOCIAL_CLASS + 1 INSTITUTION), while every map/lens/HUD path is keyed on `NodeType::TERRITORY` + FIPS. | Real, **resolved in-plan** | `MapBinding::None` hides the map and declares why (§2.11); the roster panel + beat feed + countdown carry the story. Painting 3,222 counties `PANEL` while a story runs elsewhere would be decorative absence, which III.11 forbids. |
| **B2** | ~~No content-id → `NodeId` map exists.~~ **DISSOLVED at revision 2.** | Closed | `LoadedScenario.node_content_ids` (`babylon-bsl/src/scenario.rs:311`) landed with #657 and is public. §3.2 derives the roster; the hand-transcription and its guard are deleted from this plan (§0.3). |
| **B3** | **No string payload exists on the wire** (§2.8), so no `narrative_hint` reaches a Rust client. | Design constraint, **resolved in-plan** | Client-side transcription of the frozen Python's authored copy (§2.2/§2.3/§0.2) + a Python parity guard, every row carrying its `file:line @ p27-python-freeze` provenance. Minting string payloads in BSL is an engine + language change and is **not** in scope. |
| **B4** | **The write-log pane (#617's named "first genuinely new instrument") needs a `babylon-tick` seam that does not exist.** `EffectExecutor` takes an observer (`structural_verbs.rs`) but `babylon-tick` never threads one. | Scoping honesty | **Wave 2.** Wave 1's admin pane ships the `TickReport` instrument instead — already computed, currently discarded (§3.3). Record on #617 that the write-log pane is engine-seam work, and that **wave 1 does not discharge #617** (Minor 9). |
| **B5** | **The Tension lens resolves ZERO cells on every shipped scenario** — no landed pack writes `territory/tick-exploitation-rate` or `tick-total-surplus` (`lens.rs:62-63`). | Pre-existing, declared | Keep it, whole-lens-absent, with its reserved field names visible in `help` (Task 8.3). Its landing is #615's economics pack. Deleting it would erase the names the port is meant to write; faking it would break the honest-physics rule. |
| **B6** | **Unbounded sink growth becomes real at speed** — 12 `LIFECYCLE_TRANSITION`s/tick × 25 t/s, and the current feed re-reads the whole accumulated `Vec` every frame (`loop_ui.rs:333-355`). Tracked on **#503**. | Real, **resolved in-plan** | Drain-with-cursor into a bounded `BeatLog` (§2.2), with the bound asserted across a full validated horizon of auto-run (Task 4.3). Closes the #503 item with evidence. |
| **B7** | **`babylon-client` is not on the pedantic clippy leg today** (`.mise.toml:1645-1649` names kernel/bsl/graph only); the pre-existing debt is **unmeasured**. | Unknown until Task 0.2 | Measure first, then fix. Inline `#[allow]` with a cited reason for deliberate bounded casts; a file-scoped allow + follow-up issue only if one legacy file (`atlas.rs` 746 lines / `tessellate.rs` 523) proves disproportionate. Never a crate-level blanket allow. |
| **B8** | **No GPU/display in CI** — screenshot goldens are not available and would be flaky if they were. | Constraint, **resolved in-plan** | Headless real-`App` component assertions + a byte-compared text golden (§2.8). This is the estate's own doctrine, arrived at through recorded test-vacuity findings. |
| **B9** | **`LIFECYCLE_TRANSITION` floods any naive feed** (12/tick, unconditional, the highest-volume emit in the estate) — **and the collapsed line would be the same line every tick.** | Design hazard, **resolved in-plan** | Severity-ranked feed with same-tick FLOW collapse whose count is cross-checked against `per_rule_fired` under a stated invariant (§3.3/Minor 1), **plus a magnitude term** (Σ\|Δ`pop-d-prime`\|) so the collapsed line moves every tick — the counties half of §2.4's cadence claim, gated at Task 6.3. |
| **B10** | **ADR176 was not opened in full** by the constraint-register compile; GDS §11 rows Q1 (coarse-cell representation) and Q3 (the unnamed sixth verdict) may or may not still be open. | Pre-existing uncertainty | Wave 1 touches **neither** — no LOD, no Verdict Watch. Any wave-2 task that does must read `ai/decisions/ADR176_director_rulings_batch_gds_dispositions.yaml` first. |
| **B11** | **Iosevka is not installed licensed on this box** (`main.rs:5-9`); the client uses Bevy's built-in font. | Pre-existing, Director-gated | Keep the default font. Do **not** vendor an unlicensed build. The type ladder lands when the Director supplies OFL-licensed files; note it in the closeout so the aesthetic gap is owned, not silently accepted. |
| **B12** | **3D is licensed for exactly two moments** (AF (i)); #648 is an open `director-gate` on widening that enumeration. | Scope guard | No 3D this wave (§0.1). The Entity's organ language appears only as the tick-phase heartbeat — 2D, three discrete palette steps, claiming nothing. |
| **B13** | **Wave 1 ships an unfogged surface** while ADR182 R2 makes `apply_fog` on the player path a hard v1.0 prerequisite. | Declared, not deferred silently | Legitimate today (no player ⇒ no epistemic state to protect, `data-seam.md` §1), **provided** the surface is named (§2.6's banner) and the filter seam exists (`Provenance::Redacted`, declared-dead per I9). ADR-NF (next-free-at-landing; see §9.2) states that the admin surface is the *named* exception AF's player path will be measured against. |
| **B14** | **STOP ROW — any terminal/verdict/ending surface is Director-reserved.** `TERMINAL_DECISION` is `control-ratio.bsl`'s ADR070 branch, not one of the five canonical outcomes; GDS §1 (`…design.md:39`) rules those five "*recognizers*, never terminators". | **Escalation, not a task** | Wave 1 renders the latch as a latch (§3.6) — autopause + a card naming the flag, the tick, the numeric outcome key and its pack citation, with an explicit "this is not a campaign verdict" line and a negative assertion in the test. **If a terminal surface, an ending screen, a Verdict Watch or any five-outcome vocabulary is wanted, STOP and take it to the Director as a director-gate question.** Do not improvise it. |
| **B15** | **STOP ROW — premise copy.** `Story.premise` is transcribed from the `.bscn` headers and proved by a substring test (§2.5/I1). | **Escalation if transcription fails** | If a header quote cannot be made to render legibly, do **not** author a replacement — Amendment AD reserves ideological/pedagogical copy to the Director. Escalate the two strings as a two-line gate. |

**Nothing in this list stops the train.** B1/B3/B6/B8/B9 are resolved by design inside the plan;
B2 is closed; B4/B5/B10/B11/B12 are scope boundaries with named homes; B7 is measured before it is
promised; B13 is a declaration obligation ADR-NF (next-free-at-landing; see §9.2) discharges. **B14 and B15 are live STOP rows** —
they are the two places where an engineer must escalate rather than decide.

---

## 9. Estimate

| Task | Mtok |
|---|---|
| 0 governance + measurement | 0.15 |
| 1 pedantic + doc gate | 0.30 |
| 2 the clock + pacing defaults + virtual-clock discipline | 0.70 |
| 3 projection seam (4 provenances) + admin surface | 0.60 |
| 4 narrative beats (severity + narration + `because` + feed + autopause + latch card + parity guard + horizon) | 1.05 |
| 5 story catalog (derived rosters, session ids, horizons, premises, discoverability) | 0.70 |
| 6 countdown instrument + cadence gate | 0.45 |
| 7 the carceral arc + golden + the three-layer engine-unchanged proof | 0.75 |
| 8 lens registry | 0.50 |
| 9 coverage ledger + ADR-NF (next-free-at-landing; see §9.2) + closeout | 0.55 |
| **Total** | **~5.75** |

Up ~0.9 Mtok from revision 1 (4.85). The increase buys exactly the four Criticals and nine
Importants: pacing (Task 6, new, 0.45), the coverage ledger (folded into Task 9, +0.15), the
causal-chain rows and the validated-horizon assertion (+0.15 on Task 4), the fourth provenance
class (+0.10 on Task 3), and the three-layer C1 proof (+0.05 on Task 7). Roughly two windows @1M;
the #577 charter's "~4 Mtok / 2 windows" remains the right order of magnitude for a
differently-scoped wave. Revise at closeout per #255 conventions.

---

## 10. Out of scope — wave 2 and beyond (one line each)

- **#615 unequal-exchange / imperial-rent flow lens** — lands as the first `LensPaint::Flow` row the
  day the economics BSL port writes `tick-exploitation-rate`/`tick-total-surplus`.
- **#616 three-volumes views** — Vol I/II topology + Sankey in the Topology pane; Vol III ticker and
  scissors charts as Dashboard instruments; presentation-only, zero new mechanics.
- **#617 write-log diff pane** — needs a `WriteObserver` threaded through `babylon-tick::run_tick`
  (B4); engine-seam work, not client work. **Wave 1 does not discharge #617.**
- **Fog / `apply_fog` + `DoctrineCapability` on a player path** — a second transcription job, gated
  on the player-verb surface (#593) and ADR182 R2/R5; `Provenance::Redacted` is its declared-dead
  landing site (I9).
- **The Verdict Watch, the five canonical outcomes, and any ending/verdict surface** — **Director-
  reserved** (B14); GDS §1/§4.
- **#577's two 3D moments** (Patches; topology-as-3D) and the **#638 Observatory suite** — gated on
  #648's amendment question.
- **#639 the Entity as a full visual identity** (dissection mode, organ systems) — wave 1 ships only
  the tick heartbeat.
- **#641 audio / the Entity's voice** — explicitly parked by the Director's 2026-08-11 ruling;
  assets and cue map only.
- **GDS Layout B's full pane set** (Dashboard / Wiki / Topology on keys 1–4, the Watchlist rail, the
  Chronicle/Verdict/Intercepts right rail) — wave 1 builds the pane seam (`ui/`), not the panes; the
  Wiki pane is where §2.3's transcribed `because:` lines eventually grow into derivations.
- **A campaign-scale cadence gate** (GDS §3's ≤150 autopauses/century, no dead 15-minute stretch) —
  unmeasurable without a campaign; wave 1 ships the scoped restatement and its gate (§2.4).
- **Re-deriving `ENTITY_DEATH`'s `cause` discriminant** — `vitality.bsl:90-95` records that it is
  recoverable; wave 1 declares it `NotComputed` rather than re-implementing engine logic in the
  viewer (I2).
- **Ecology / metabolic-rift instrument** — owed per the coverage principle despite GDS's silence
  (#649 is the spec amendment); the coverage ledger (§3.4) is where its absence gets a row.
- **A Bevy lobby screen, modding packs (#531), the BSL REPL (#532) and editor (#533)** — all
  post-wave-1, all with the story catalog (§2.5) as their data model.
