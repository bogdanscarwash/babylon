# Program 27 Phase 0 — Estate Inventory + In-Flight v1.0 Stops Disposition

Task 9 of `docs/superpowers/plans/2026-07-29-program-27-phase-0-contracts-and-evidence.md`.
Produces two artifacts named by the spec: (1) the finalized §6.5 engine-periphery
estate inventory, verified against the current codebase; (2) the disposition
table for in-flight v1.0 stops named by spec §13 item 6. No issue is closed by
this document — it is a proposal; closing happens after Director sign-off of
the v3.0.0 amendment (Task 16).

## 1. Estate inventory (finalized §6.5)

Source: `docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`
§6.5. Each row was spot-checked against the code on this date (2026-07-29,
`dev` HEAD `3f4a1eb0`); the **Verified** column records what the checkout
actually shows where it differs from the spec's cited figures. Dispositions
are unchanged — this is a verification pass, not a re-ruling.

| Component | Disposition | Verified |
|---|---|---|
| EventBus (`kernel/event_bus.py`) + EventType (`models/enums/events.py`) | **Port.** Ordering guarantees (registration-order dispatch, append-before-emit, stable-sorted interceptor chain) are already deterministic and load-bearing; preserved byte-for-byte. Single consumption pattern: batch-drain-per-tick. | `event_bus.py` is 288 lines, matches spec exactly. `EventType` is **100** members (`len(list(EventType))`), matching the spec's "100-value" figure and confirming CLAUDE.md's stale "84" note; the enum file itself has grown to 234 lines (spec cited `:30-188`, a stale line anchor — member count is what's load-bearing and it checks out). |
| Three observer mechanisms (legacy SimulationObserver; ad hoc `EndgameDetector.on_tick` direct call; TickCommitObserver → vault baker) | **Consolidate into one hook point.** Porting all three would be porting a bug. | All three still present and distinct in `src/babylon/engine/observers/`; no consolidation done yet (correctly deferred to the Rust port, not Phase 0). |
| SessionRecorder | **Dies.** The real replay substrate is `PerTickTransactionEnvelope` + the commit marker, which ports as a kernel construct. | `session_recorder.py` (222 lines) still lives at `engine/observers/`; unchanged, disposition holds. |
| EndgameDetector (5 outcomes, every-tick re-evaluation, never latching) | **Port as BSL-expressible predicates**; the priority order (RED_OGV > FRAGMENTED_COLLAPSE > ECOLOGICAL_COLLAPSE > FASCIST_CONSOLIDATION > REVOLUTIONARY_VICTORY) becomes **data** asserted by the conformance corpus. | `endgame_detector.py` is **812 lines**, matching the spec exactly. |
| ServiceContainer (~40 `Any`-typed optional slots) | **Not ported 1:1.** Folds into the typed intrinsic table; reproducing it as `Option<Box<dyn Any>>` re-imports the type-erasure problem. | `engine/services.py:151` `class ServiceContainer`. Actual `Any`-typed field count today is **~84**, not ~40 — the container grew since the spec's figure was written (drift, not a disposition change; flagged for the Phase-0 typed-intrinsic-table author as the current baseline to port from, not ~40). |
| TickContext (`extra="allow"` + dict shims) | **Phase-0 census of every stamped key → first-class typed fields.** The escape hatch does not survive. Highest silent-breakage risk found. | `engine/context.py:19` `class TickContext`, `model_config = ConfigDict(extra="allow")` at line 46 — confirmed live. Task 6 of this plan (TickContext stamped-key census) is the execution of this disposition; not re-run here. |
| TickPartition (system declares own partition+position) | **Port as-is** — already-declarative, single source of truth; mods use anchors (§5). | `kernel/tick_partition.py:18` `class TickPartition(StrEnum)` — confirmed live, unchanged. |
| `game/session.py` composition root | Absorbed into `babylon-engine` + `babylon-cli`. | `src/babylon/game/session.py` is **1,897 lines**, matching the spec exactly. |
| CLI: doctor/telemetry/login/self_update/uninstall | **Stays Python** — zero engine coupling verified. | All five confirmed present as standalone modules: `cli/doctor.py`, `cli/telemetry.py`, `cli/login.py`, `cli/self_update.py`, `cli/uninstall.py`. |
| `uuid4()` per-tick correlation id | Replaced by a deterministic per-tick counter (log-only today; the replacement is strictly better). | Confirmed: `simulation_engine.py:195` `correlation_id = str(uuid4())`, log-only per-tick use; also used for session/campaign ids elsewhere (`headless_runner/runner.py:1189`, `tui/campaign_menu.py:191`) which are out of this disposition's scope (identity ids, not tick correlation). |
| Logging estate | The **game process** collapses to one JSONL DEBUG sink (`babylon.log`); the Python observer keeps its own sink; `client-capture.log` retires with the PyO3 boundary. This supersedes the 2026-07-28 three-log Director directive — flagged for Director confirmation (§13 item 7). | Confirmed three-log estate live exactly as CLAUDE.md describes: `config/logging_config.py:306` writes `babylon.log`; `rust/crates/babylon-tui/src/logging.rs` writes `rust-client.log` (log4rs, per PR #341, 2026-07-28); `cli/play.py` writes/rotates `client-capture.log`. No component missing from the spec's row was found. |

No load-bearing component was discovered missing from the §6.5 table during
this pass; the table above is final as written, with the **Verified** column
recording drift in cited figures (EndgameDetector, session.py, EventBus, and
EventType count all match exactly; ServiceContainer's `Any`-slot count has
grown from ~40 to ~84 since the spec was drafted on 2026-07-28).

## 2. In-flight v1.0 stops disposition (spec §13 item 6)

Scope per the plan: "one row per open stop (#262 Gate 3, M4 owner smokes,
ADR109 wiring train, each open Project-8 train)". Enumerated via `gh issue
list --state open` (28 open issues) and `gh project item-list 8 --owner
percy-raskova` (38 items total, 24 in `Todo`, 14 in `Done`). Every `Todo`-status
Project-8 item is a "train" per project convention (`agentic-backlog` label);
rows below cover all 24 open trains plus the two named non-project stops (M4
owner smokes has no GH issue of its own; it is a sub-item of #262's evidence
pack per ADR150).

**Disposition key:** `closed-as-superseded (R2)` — Program 27 R2 (no new
Python engine feature trains) makes this train's remaining work moot, the
issue should close referencing the amendment; `absorbed into P27 Phase N` —
the train's remaining scope becomes Rust-port work inside a named P27 phase,
tracked there instead; `carried (client-side)` — client/tooling/docs work
outside the engine-port boundary, continues unaffected by Program 27.

| Stop | Status | Disposition | Rationale (one sentence) |
|---|---|---|---|
| #262 — BD Gate 3: eyes-on TUI campaign session (owner) | Todo | **carried (client-side)** | Owner eyes-on ruling on the existing Rust TUI, independent of the engine rewrite; the ADR150 M4 owner smokes (field-surface rotate, kitty pixel plate) fold into the same evidence pack and share this disposition. |
| M4 owner smokes (ADR150 §9.21, no standalone issue) | Untracked | **carried (client-side)** | Sub-item of #262's evidence pack, not a separate GH issue; recommend either filing it as a child issue or leaving it inside #262 — no Program-27 action needed either way. |
| #264 — ADR109 wiring-doctrine enforcement train | Todo | **absorbed into P27 Phase N** | Wiring Doctrine governs connecting dormant Python constructs; once the engine is Rust, the doctrine's dataflow/opposition/adjunction/projection/conservation motions re-target the Rust intrinsic/BSL surfaces — the train's remaining Python-side gap ledger becomes input to whichever P27 phase lands the corresponding subsystem, not a standalone Python train. |
| #265 — T6: Tutorial teaches all nine verbs (tutorial IS the BDD suite) | Todo | **carried (client-side)** | Tutorial content and TUI overlay work; the verb surface itself ports (§5 typed structural verbs) but the tutorial script is presentation, unaffected by the engine language. |
| #266 — Activation-drift verification: first 520-tick bake | Todo | **closed-as-superseded (R2)** | A 520-tick bake on the current Python engine is exactly the kind of pre-port validation activity Program 27's parallel-run window (§8, M13) replaces with a cross-implementation drift check; running it standalone on Python now duplicates work the cutover ceremony will do anyway. |
| #267 — Fog/Investigate: wire EpistemicHorizon to the player | Todo | **absorbed into P27 Phase N** | EpistemicHorizon is a Phase-1-shadow system (observes-only) in the current engine; wiring it to the player is new engine-facing feature surface, R2-forbidden in Python — lands as Rust-side work once EpistemicHorizon ports. |
| #268 — Doctrine Tree Unit 6: doctrine→consciousness feedback + branch deepening | Todo | **closed-as-superseded (R2)** | New Doctrine/Consciousness feedback logic is a new Python engine feature train, exactly what R2 forbids; re-charter after the Rust cutover against the ported Doctrine/Consciousness systems. |
| #269 — Material Triad: metabolic calculus → territory train (Amendment AB) | Todo | **closed-as-superseded (R2)** | New engine feature train (metabolic calculus wiring into territory) — forbidden in Python under R2; re-charter post-port. |
| #270 — Balkanization/RED_OGV repair: activate the sovereignty layer (engine half) | Todo | **closed-as-superseded (R2)** | Engine-half repair work is new Python engine logic; forbidden under R2 — re-charter against the ported sovereignty/RED_OGV predicates (already named as BSL-expressible endgame predicates in §6.5). |
| #271 — Divergence Channel Phase-1 (Amendment T, observes-only) | Todo | **closed-as-superseded (R2)** | Observes-only is still new engine code; the Amendment T channel is explicitly "code QUEUED off critical path" per project memory — Program 27 is now the critical path, so this queues behind the cutover rather than landing in Python. |
| #275 — Owed sentinels & truth-status roll-up | Todo | **carried (client-side)** | Sentinel/test-estate bookkeeping across the whole repo, independent of engine language; continues regardless of Program 27, and partly feeds Task 7/8 of this same plan (sentinel-estate and test-estate disposition tables). |
| #284 — Archive shell: kitty raster lane for the Map pane (P9) | Todo | **carried (client-side)** | Pure Rust/Ratatui client rendering work (kitty graphics protocol), no Python engine coupling. |
| #286 — Tutorial: the live STATE_REPRESSION bulletin beat (close the W4 honest gap) | Todo | **carried (client-side)** | Client/tutorial content wiring against an existing engine signal, not new engine logic. |
| #288 — Org estate: player-side organization program (org→org SOLIDARITY verb first) | Todo | **closed-as-superseded (R2)** | New gameplay verb + engine feature train — forbidden under R2; re-charter post-cutover against the typed structural verb surface. |
| #291 — T7: Installer train (nix-bootstrap, agent half) | Todo | **carried (client-side)** | Packaging/installer tooling, orthogonal to the engine's implementation language. |
| #292 — T7: Owner ceremonies (signing keygen, R2 cache + worker, GGUF upload) | Todo | **carried (client-side)** | Owner-only release infrastructure, unaffected by the engine port. |
| #293 — T8: v1.0.0 release ceremony (DoD battery + version ceremony) | Todo | **absorbed into P27 Phase N** | The v1.0.0 release ceremony's Definition-of-Done battery is exactly what Program 27's cutover/freeze-tag machinery (§10, this plan's Tasks 11-17) redefines; the ceremony's content depends on which engine ships as v1.0, so it folds into the freeze-tag phase rather than running independently now. |
| #334 — Phase 0 — national incidence artifact + data program (ADR171) | Todo | **carried (client-side)** | Data/content program (national-incidence overlays, rendering) per ADR171; content and data pipelines are explicitly outside the engine-port boundary (§7 content pipeline covers only `ContentDigest`/defines/BSL, not this data program). |
| #335 — Wiki content architecture: Glossary/State/Flavor namespaces (Director directive) | Todo | **carried (client-side)** | Content/documentation architecture, no engine coupling. |
| #336 — Verb-algebra research seed → proposal (Director-gated) | Todo | **out of scope for this table** | Explicitly a research seed awaiting a Director-gated proposal, not implementation; no engine code has been written against it, so there is nothing for R2 to forbid yet — recommend leaving open as-is; listed here for completeness only. |
| #337 — National axis Phase 2 — engine train (BLOCKED): shadow BoundOppositions + the community seam | Todo | **closed-as-superseded (R2)** | Explicitly an "engine train" per its own title — new Python engine feature work, forbidden under R2; already blocked on the `community_memberships` producer gap (ADR109 W-C, the same gap ADR150 names for the 3D hypergraph) — re-charter post-port. |
| #343 — Program 27 Phase 0 — Contracts & Evidence train | Todo | **n/a (self)** | This is the parent tracking issue for the plan this task belongs to; not a stop, listed for completeness. |
| #280 — hypergraph-rs Phase 9: CLI (render formats) | Open (not in Project 8) | **carried (client-side)** | Standalone Rust CLI tooling for the `hypergraph-rs` sibling project, no Python engine coupling; open but not on the agentic backlog board. |
| #282 — Babylon swap: consume hypergraph_rs behind the XGI surface | Open (not in Project 8), `blocked:dependency` | **absorbed into P27 Phase N** | Consuming `hypergraph-rs` as the graph substrate is exactly the kind of Rust-side dependency decision Program 27's kernel work will make directly; the byte-identical-gate framing this issue names becomes part of the port's own conformance corpus rather than a separate swap-in step. |
| #285 — H3 index BIGINT migration (u64 bitwise invariant grid in Postgres) | Open (not in Project 8) | **carried (client-side)** | A Postgres schema/migration concern; the Postgres runtime and its persistence contract (§7) are explicitly preserved verbatim by Program 27, so this migration proceeds independent of the engine-language rewrite. |
| #287 — S9 lane-extension ruling: 3D spatial embeddings for babylon topology surfaces | Open (not in Project 8), `blocked:owner` | **carried (client-side)** | A rendering/UX ruling for the client's topology surfaces, owner-gated and independent of engine internals. |
| #289 — hypergraph-rs Phase 8 (WASM) — deferred indefinitely | Open (not in Project 8), `paused` | **carried (client-side)** | Already explicitly paused with a recorded resume point in the sibling project; Program 27 doesn't change that status. |
| #290 — hypergraph-rs program closeout — parked advisories + ceremony | Open (not in Project 8) | **carried (client-side)** | Closeout bookkeeping for the sibling project, independent of Program 27. |

**Summary:** of the 24 open Project-8 trains plus the 6 open-but-unlisted
issues examined, **7 are `closed-as-superseded (R2)`** (new-engine-feature
trains that R2 directly forbids: #266, #268, #269, #270, #271, #288, #337),
**4 are `absorbed into P27 Phase N`** (#264, #267, #293, #282), **12 are
`carried (client-side)`** (#262/M4-smokes, #265, #275, #284, #286, #291,
#292, #334, #335, #280, #285, #287, #289, #290 — client, content, docs,
Postgres, and sibling-project work the port doesn't touch), and **2 are
out of scope for this table** (#336 research-seed-only, #343 the parent
tracking issue for this very plan).

No issue is closed by this report. Disposition becomes actionable only after
Director sign-off of the v3.0.0 amendment (Task 16 of this plan); until then
this table is the standing proposal for that sign-off to accept, amend, or
reject line by line.
