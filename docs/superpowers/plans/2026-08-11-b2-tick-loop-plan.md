# Program 28 B2 — The Tick Loop On Screen: implementation plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to execute this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking.

**Goal:** close Program 28 §7 criterion 3 — a person opens `babylon-client`, sees the county map
B1 rendered, presses a key to advance the tick, and watches real state change: a tick counter, a
deterministic state-hash readout, a live per-county legitimation overlay on the map, a state panel
for the hovered/selected county, and an event feed — all driven by the Rust engine through a new
persistent tick-loop seam running TWO Material Base systems together, never a lookalike.

**Amendment record (2026-08-11, Director interactive ruling batch, this plan's own open
questions).** The first cut of this plan closed with three open questions. The Director ruled all
three, interactively, options quoted verbatim as presented and selected — the same citation
discipline ADR194 uses:

1. **Legitimation lens color mapping: APPROVED, reuse the band palette.** Selected option, quoted:
   *"CRISIS → crimson, UNSTABLE → dim gray, STABLE → gold's absence (panel dark) — reuses the
   ruled four-band vocabulary so the two lenses share one visual language. No new colors enter the
   game."*
2. **Demo content set: MULTI-RULE DRIVER FIRST — the Director OVERRULED this plan's single-pack
   recommendation.** Selected option, quoted: *"Build the multi-rule content-set evolution into B2
   itself so the demo runs vitality+lifecycle together from day one. Bigger B2, later criterion-3
   close, but a richer first demo."*
3. **Audio: DEFERRED out of B2**, per this plan's own recommendation — no change.

Ruling 2 is load-bearing: it reopens what the first cut called "Decision: the demo content set is
the lifecycle rule pack, alone" and inserts a new, genuinely non-trivial engine-lane evolution
(Phase A, Tasks 2–6 below) ahead of the demo-scenario task. This document renumbers and amends
every task list, file structure entry and cross-reference below to carry that decision out. The
three rulings'
full record, with reasoning, sits in the amended "Open questions for the Director" section at the
end — kept, not deleted, per the Documentation philosophy's immutability-of-history discipline;
what changed is that each question now carries its ruling instead of standing open.

**Revision record (2026-08-11, adversarial three-lens verification round on this amended plan).**
A verification panel checked every §2.2/driver/D96/anchor claim in the first amendment TRUE, and
found two blockers, one defect, and one nit, all fixed in this second revision:

- **Blocker 1 — the multi-rule design's "declaration order is execution order" directly
  contradicted an existing normative sentence this plan never read.** `bsl-language.rst:3058-3060`
  (register row D16, §4.2):
  <!-- vale off -->
  *"Rules at the same anchor position evaluate in ascending rule-id byte order [draft ruling —
  Phase 1 review], and their effects apply in that same order. File order and load order are never
  observable."*
  <!-- vale on -->
  Verified by reading the section directly — the sentence matches the citation
  exactly, and directly negates the design the first amendment shipped. **Fixed**: the driver orders
  rules by ascending rule-id BYTE ORDER (D16), not by declaration/concatenation order — see the
  rewritten Execution Order subsection below. Every task and test asserting declaration-order
  behavior now asserts byte order instead; a file-order-INVARIANCE test replaces the order-flip
  test, matching what §4.2 actually promises.
- **Blocker 2 — the "live legitimation overlay" headline was provably constant, and invisible
  under the Director's own palette ruling.** Verified by reading `lifecycle.bsl`'s Block 2 in
  full: `legit-index` is a weighted sum of five `:const` bindings ONLY — no `:field` binding
  anywhere in its computation — so it evaluates to the SAME number for every territory, every
  tick, forever (the pack's own D-1 note already said this; this plan's first cut never followed
  the implication through). Every territory classifies STABLE (0) from tick 2 onward as a result,
  and STABLE renders `PANEL` under ruling 1 — the same color as no data. A player pressing Space
  five times would watch an unchanging dark map. **Fixed**: a new Population Trend lens, keyed to
  the DPD circuit's genuinely per-tick, per-territory population fields (verified against the
  actual formulas — five-tick trajectories worked out below), carries "watch state change." The
  Legitimation lens ships exactly as ruled, with its current-uniform behavior stated plainly
  rather than implied away.
- **Defect — a D-row number collision.** Open PR #500 (director-ruled 2026-08-11, ADR194 R2, the
  Currency-scale operation) mints its OWN `D99` row, verified by diffing its branch against `dev`.
  **Fixed**: this plan names every row number it mints as "the next free register row," resolved
  at execution time, never hard-coded — plus a new task closing the general defect class (a
  register-row uniqueness guard; the existing sync-guard's `re.search` existence checks would pass
  even with two `D99` rows in the file).
- **Nit — a third `rule_id` extractor.** `babylon-bsl` already carries two
  (`canonical_ast.rs:65`, `bound_checker.rs:678`), verified by reading both. **Fixed**: Task 2
  widens `canonical_ast::rule_id`'s visibility and reuses it; no third implementation.

**Architecture:** Five phases in five PRs, Phase A now doing two jobs. Phase A opens a
**persistent session** in `babylon-tick` — `TickSession<G>` — AND, ahead of that, widens the
content-set loader to admit **more than one `(rule …)` form**, executed in **ascending rule-id
byte order** (§4.2, register row D16 — never declaration order, which the same section rules
"never observable"), the way `bsl-language.rst` §2.2's grammar always admitted and `babylon-bsl`'s
driver-level loader never implemented. Phase B authors the **demo content**: eighteen subjects
across two node types — twelve real-FIPS territories, each seeded with its OWN distinct population
figures (not three bare repeats of four archetypes) so their DPD trajectories visibly diverge tick
by tick, carrying the already-conformance-tested `lifecycle` rule pack, and six social classes
carrying the already-conformance-tested `vitality` rule pack's own fixture verbatim — so the demo
runs two Material Base systems together from the first tick. Phase C **completes B1's still-unbuilt
Phase C** (`lens.rs`, `map/bands.rs`'s band table, `map/pick.rs`, `map/hud.rs`) but generalizes it
to carry THREE lenses side by side — ADR170's static Tension lens (ported unmodified), the
Director-ruled Legitimation lens (shipped as ruled, though this plan states plainly that it renders
uniformly today — `legit-index` is const-only, verified from the pack's own math), and a new
Population Trend lens that carries "watch state change" for real, keyed to the DPD circuit's
genuinely evolving per-territory population fields — with a lens-picker key cycling all three so
the player can see the difference between "declared once," "moves every tick but not yet
per-territory," and "moves and diverges by county" honestly. Phase D wires the **loop UI**: the
advance-tick input, the tick counter, the hash readout, the state panel, the event feed (now
carrying both packs' events). Phase E resurrects the **file-log sink** the deletion ceremony
retired, proves **determinism** end-to-end, and defines the **eyes-on gate**.

**Tech Stack:** Bevy 0.18.1 (unchanged from B1 — `pan_camera` feature already pinned), the same
`earcutr`/atlas/tessellate stack B1 built, `log4rs` 1.x (`default-features = false`, the exact
feature set the deleted `babylon-tui` crate used) resurrected for the client's own file sink,
`log` 0.4 as the facade that sink listens on — kept fully separate from Bevy's own `tracing`-based
`LogPlugin`, which keeps printing to the console exactly as `DefaultPlugins` already wires it.

**Source spec:** `docs/superpowers/specs/2026-08-10-program-28-bevy-cutover-roadmap-design.md`
(§4 client lane, §7 criterion 3, §8 open questions). **Predecessor plan:**
`docs/superpowers/plans/2026-08-10-b1-county-map-plan.md` (Phase A+B merged as PR #487/#490;
Phase C `lens.rs`/`map/pick.rs`/`map/hud.rs` and the `band_color` function in `map/bands.rs`
**were never built** — only `bands.rs`'s `PANEL` constant landed, pre-positioned there by a B1
adversarial-review fix (F4) specifically so "Task 9 finds one existing declaration to extend, not
a second one to reconcile." This plan is that extension, generalized — see the Sequencing
Decision below.

**Governing rulings this plan carries out and does not reopen:**

- **ADR170** — this plan transcribes the Tension lens formula (`phi = v/(v+s)`,
  `theta = sum(v)/sum(v+s)` as a ratio of sums, `w = (phi-theta)/(phi+theta)`) unchanged, wherever
  it touches it.
- **ADR191 R9/R10/R11** — the Director already settled the Iosevka Nerd Font choice, the
  cartographic insets, and the FOUR-BAND (not continuous-ramp) rendering of the Tension lens; this
  plan reuses R11's band table verbatim rather than re-deriving it, and (per this amendment's
  ruling 1) points the Legitimation lens at the SAME three of its four colors.
- **ADR193** — `babylon-tick::run_once`/`run_once_into` now construct `HypergraphStore`, not
  `MemoryGraph`; `state_hash()` calls `encode_state()`, and ADR193 measures that call QUADRATIC in
  hyperedge count on `HypergraphStore` (22.59 ms at n=2,000 hyperedges; 1.92 s at n=20,000). **This
  plan's own demo scenario mints eighteen nodes (twelve `NodeType/TERRITORY`, six
  `NodeType/SOCIAL_CLASS`) and zero hyperedges** — the cliff does not bite; see the Scale Note
  below for the arithmetic.
- **D16 (§4.2)** —
  <!-- vale off -->
  "Rules at the same anchor position evaluate in ascending rule-id byte order [draft ruling —
  Phase 1 review]… File order and load order are never observable."
  <!-- vale on -->
  This is the GOVERNING rule for the multi-rule
  evolution's execution order (Phase A, Tasks 2–6) — verified by direct reading
  (`bsl-language.rst:3058-3060`) after the first amendment's design contradicted it. The driver
  applies D16 to the whole slice-1 rule set (no anchor-position registry exists to differentiate
  positions across systems yet — Phase 3, not built), not a new ordering law this plan invents.
- **D96 (ADR191 R2)** — "a scenario is a canonical committed artifact and its declaration order is
  part of its identity," strictly for NODE declarations within a scenario. This plan does NOT
  extend D96 to rule declarations (the first amendment's error, corrected here) — D16 already
  governs rule order, on its own textual authority, and needs no analogy borrowed from a different
  register row.
- **Constitution III.7 (determinism)** and **III.11 (Loud Failure)** — the hash display exists
  because the hash IS the honesty proof; a county the demo scenario never minted stays `PANEL`,
  never a fabricated value.
- **R8/R9 (BSL-first porting, escape by proof)** — nothing in this plan adds Rust simulation logic;
  the only Rust code this plan writes is client/UI/seam code, a loader-widening change that makes
  the driver honor grammar §2.2 already admits (not a new primitive), and a factored-out loader
  helper. All simulation content stays in the already-merged `vitality` and `lifecycle` rule packs.
- **No imposed functional forms (2026-07-29 standing ruling)** — the Legitimation lens invents no
  threshold and no formula. It colors counties by the **categorical classification the `lifecycle`
  rule pack already computes and writes** (`territory/legitimation-crisis`: 0 = STABLE, 1 =
  UNSTABLE, 2 = CRISIS), never a newly-derived cut point on the raw index. The new Population Trend
  lens (Task 10, added this revision) reads the same standing: it colors by the SIGN of a
  territory's own total-population change since tick 0 — a strict `>`/`<`/`=` comparison, no
  size threshold invented, no cut point chosen.

## Global Constraints

- Branch from `dev`; conventional commits; every commit ends with
  `Co-Authored-By: Claude Fable 5 <noreply@anthropic.com>`.
- Worktree execution recipe: symlink `.venv` from the main checkout, copy `data/`, commit with
  `PYTHONPATH="$PWD/src"` — this plan is docs-only (no Rust build), but a later executor of THIS
  plan's tasks needs the recipe, so this document carries it for them.
- Gates for the executing agent: `mise run rust:check` for any `rust/` change (clippy
  `-D warnings`, workspace, all targets, locked); `mise run check` for the repo-wide fast gate;
  `mise run qa:regression` and `mise run qa:vault-regression-ci` after any change touching
  `babylon-bsl`/`babylon-tick` — Phase A's Task 1 refactor and Task 4 widening must each move zero
  engine bytes on every EXISTING single-rule content set, and each task's own regression test is
  the first proof of that, not the only one.
- Vale: `vale <file>` on every Markdown page touched, driven to 0.
- **CI reality (unchanged from B1):** `rust-gate` runs on `ubuntu-latest`, compile-time Bevy
  headers only, no display server, no GPU. Every headless test in this plan uses `MinimalPlugins`
  plus `AssetPlugin`, never `DefaultPlugins`, exactly as B1's `tests/map_mesh.rs` and
  `tests/map_camera.rs` already establish.
- **Palette canon** — reuse only already-declared `§9b` tokens (`palette.rs`) and the already-ruled
  ADR170 four-band table (`PANEL`, `CRIMSON`, `DIM`, `GOLD` from `map/bands.rs`); per this
  amendment's ruling 1, the Legitimation lens's three colors are `PANEL` (STABLE — "gold's
  absence"), `DIM` (UNSTABLE), `CRIMSON` (CRISIS), reusing three of the FOUR already-declared
  `map/bands.rs` constants. The new Population Trend lens (this revision) reuses all FOUR —
  `CRIMSON` (declining), `GOLD` (growing), `DIM` (unchanged, unreachable on real float trajectories
  but included for totality), `PANEL` (absence) — the same four tokens ADR191 R11 already declared,
  applied to a third variable. This plan adds no new `Color::srgb_u8` literal anywhere, so
  `test_no_stray_color_literals_outside_palette_or_a_declared_exemption`'s sweep needs no new
  exemption entry.
- **The babylon-bsl surface this plan touches, stated exactly.** Every task reads live state
  through `GraphSubstrate`'s existing trait surface and `CanonicalState`'s existing `state_hash` —
  unchanged (no method added, removed, or re-signatured on either). Two things ARE new, both flagged explicitly, both machinery rather than new
  mathematics or a new primitive (Amendment AE's test): (1) `babylon-tick::TickSession`, additive,
  `run_once`/`run_once_into` keep their exact current signatures; (2)
  `babylon-bsl::rule_pipeline::split_content` widens from "exactly one `(rule …)` top-form" to
  "one or more, EXECUTED IN ASCENDING RULE-ID BYTE ORDER per §4.2/D16, duplicate ids refused" —
  closing a gap between the DRIVER's own historical restriction and what `bsl-language.rst` §2.2's
  grammar (`<top-form>*`) and prose ("Duplicate rule ids… across the content set are
  `E-LOAD-001`") always admitted. See the Multi-Rule Decision section below for the full design
  and why this counts as a driver fix, not a spec change — and why the order is byte order, never
  declaration order (§4.2's own words: "File order and load order are never observable").
- **Scale note (ADR193 arithmetic, worked here so no task has to re-derive it):** the Phase B demo
  scenario mints exactly 18 nodes (12 `NodeType/TERRITORY`, 6 `NodeType/SOCIAL_CLASS`) and declares
  no `(edge …)`/`(hyperedge …)` forms, so `HypergraphStore::encode_state`'s hyperedge half walks
  **zero** hyperedges regardless of the measured quadratic constant — the ADR193 table's smallest
  measured point (n=2,000 hyperedges, 22.59 ms) is already ~5,500x this plan's largest node count.
  `state_hash()` runs twice per `advance()` call (pre/post, per `TickSession::advance`), now
  bracketing TWO `run_tick` calls instead of one, against 18 nodes, ~90 scalar attributes and 0
  hyperedges — still sub-millisecond on the dyadic-half code path both stores share (ADR193:
  "`nodes`/`edges`/`neighbors` show no consistent direction at any scale"). The cliff is a real,
  documented future cost (ADR193's own "3,222-US-county target… plausibly crosses 10,000
  hyperedges once county-level organizational and sector memberships exist") — it belongs to
  whichever later program mints those memberships, not to this one.

---

## Decision: B2 completes B1's Phase C, generalized to three lenses

**B1's own File Structure table already reserves four files for "Phase C — the lens lane":
`lens.rs`, `map/bands.rs`'s `band_color`/recolor-system additions, `map/pick.rs`, `map/hud.rs`.
None of the four landed** — `git log` shows B1 merged only Phase A (#487, the atlas artifact) and
Phase B (#490, atlas reader/tessellation/mesh/camera); `find rust/crates/babylon-client/src` today
shows `atlas.rs`, `engine_link.rs`, `lib.rs`, `main.rs`, `palette.rs`, `tessellate.rs`, and
`map/{bands,camera,mesh,mod}.rs` — `bands.rs` holds only the `PANEL` constant (parked there by a
B1 adversarial-review fix specifically so a later task would extend, not duplicate, its
declaration).

The roadmap spec's own B2 line is: *"tick advance, state panels, event feed — the scenario ->
tick -> hash seam that already runs headless, made visible and playable."* That line does not,
by itself, require the map to recolor from live ticks — the state panel alone could meet "watch
state change." But §7 criterion 3 says *"see the county map… and watch state change"* — read
together with the Director's own aesthetic-and-pedagogy line (*"what game mechanics are both
engaging AND instill education about the correct revolutionary theory?"*), a tick loop whose only
visible effect is a text panel while the map sits inert is a materially weaker demo than one where
watching the county the player has selected shows real state moving. **Stated with the same
honesty the eyes-on gate (Task 18) later carries in full**: on THIS demo content that means one
band-color sign-flip per county family at tick 1 (Task 9b's four archetype families split GOLD/
CRIMSON the moment the first tick runs), and after that, the HUD/state-panel NUMBERS keep moving
every press even though most counties' band color does not flip again — "watch state change" does
not promise a repeated color flip on every single press, only that Something Real is moving and a
player can see it. The map is already built (Phase B); Phase C's reserved files are the ONLY place
the county-indexed lens/hover/recolor/HUD plumbing this needs was ever going to live.

**Decision: this plan builds Phase C's four reserved files as part of its own task list — it does
not wait for a separate Phase C PR, and it does not duplicate Phase C's interfaces alongside new
ones.** Concretely:

1. `lens.rs` carries the ADR170 witness (`county_tension`, ported verbatim from the B1 plan's
   Task 8 spec, corrected for one thing the B1 plan text predates — see below), a second witness
   (`county_legitimation`) reading the field the tick loop writes every tick (Task 9), and a THIRD
   witness (`county_population_trend`, Task 9b, added this revision) reading the DPD circuit's
   genuinely per-tick population fields — the lens that actually carries "watch state change" on
   this demo content.
2. `map/bands.rs` gains B1 Task 9's `band_color` function and four-row table (ADR191 R11,
   unmodified), a second, small three-row table for the Legitimation lens colored per this
   amendment's ruling 1, and a THIRD table for the Population Trend lens (Task 10, sign-only —
   Director-ruled this revision, see the Open Questions section) — all three are pure presentation
   constants, no `GameDefines`/`defines_hash` ceremony, exactly as ADR191 R11 already ruled for the
   first table.
3. `map/pick.rs` and `map/hud.rs` are B1 Task 10's designs, unmodified, except the HUD now also
   names which of the THREE lenses is active — this plan adds that honesty rule because the color
   CRIMSON now carries THREE separate meanings across the three lenses (Tension's "Φ-source, bled,"
   the Legitimation lens's "CRISIS," the Population Trend lens's "declining"), and nothing may let
   a player read one meaning as another.

**One correction to B1's plan text, made explicit so no one silently inherits it wrong:** B1's
Task 8 spec writes `pub fn county_tension(graph: &MemoryGraph) -> TensionLens`. ADR193 (merged the
same day, sequenced textually after B1's plan but landed at the same `dev`-branch tip this plan
reads) swapped the production substrate from `MemoryGraph` to `HypergraphStore` — `run_once_into`
and, after Phase A of this plan, `TickSession`, both hold a `HypergraphStore`. **This plan's
`lens.rs` takes `&dyn GraphSubstrate`, not `&MemoryGraph`** — the trait both stores carry,
matching what the client actually holds. `MemoryGraph` remains only as the differential-test
oracle (ADR193's own consequences section).

## Decision: the demo content set runs `vitality` AND `lifecycle`, together, in rule-id byte order

**Superseded by this amendment.** The first cut of this plan recommended running `lifecycle`
alone, citing a real technical wall: `babylon-bsl::rule_pipeline::split_content` enforces, by
construction (`rule_pipeline.rs:299-308`), exactly one `(rule …)` top-form per content set. The
Director overruled that recommendation (ruling 2, quoted above): B2 builds the multi-rule
evolution now. This section is the design that discharges that ruling, and it replaces the old
"Decision: the demo content set is the lifecycle rule pack, alone" section outright — the
technical-wall description below stays, because it remains the reason the evolution is real
engineering work and not a one-line flag flip.

**Revision note.** This section's FIRST amendment shipped a "declaration order is execution
order" design. A verification round caught it contradicting `bsl-language.rst` §4.2's own text —
see the Execution order subsection below, rewritten in full. Everything else in this section
(the grammar-admits-more-than-one-rule argument, the field/name-collision audit) stood up to
verification unchanged.

### What §2.2 already admits, and what the driver never implemented

`bsl-language.rst` §2.2's own grammar has never limited a content set to one rule:

```text
<file>        ::= <top-form>*
<top-form>    ::= <rule> | <deffield> | <intrinsic-decl> | <manifest> | <metric-decl>
```

and its prose says, in the same section: *"A content set is the union of all files under the
declared content roots. File boundaries and file names carry no semantics… Duplicate rule ids,
duplicate field declarations, duplicate intrinsic declarations… across the content set are
`E-LOAD-001`."* Nothing there says "exactly one" — the grammar's `<top-form>*` is zero-or-more, and
the prose's whole framing (naming DUPLICATE ids as the violation) presupposes a content set can
legally hold two or more rules with DISTINCT ids. The current refusal text —

```text
"a content set needs exactly one (rule …) top-form, found {N}
 (§2.2 — intrinsic declarations do not count; deffield/manifest/metric-decl
 top-forms are not yet split out by this function and would also land here)"
```

— is `babylon-bsl::rule_pipeline::split_content`'s OWN cardinality check; the spec never demanded
it. This plan's evolution makes the driver honor what the grammar already admitted, which is why R8/R9
(BSL-first, escape by proof) and Amendment AE's "mints no new mathematics" test both read this as
machinery, not a new primitive needing a constitutional amendment.

### Execution order: ascending rule-id byte order (§4.2, register row D16) — NOT declaration order

**This subsection replaces the first amendment's design outright.** That design made load/
concatenation order the execution order, reasoning by analogy from D96 (node declaration order is
scenario identity). A verification round found the direct, on-point rule this plan should have
read first: `bsl-language.rst:3058-3060` (§4.2, register row D16):

<!-- vale off -->
> Rules at the same anchor position evaluate in **ascending rule-id byte order** [draft ruling —
> Phase 1 review], and their effects apply in that same order. **File order and load order are
> never observable.**
<!-- vale on -->

That last sentence is the exact negation of the first amendment's design. No honest reading keeps
"declaration order governs" after seeing it — this section corrects course rather than
defending the original error.

**The corrected design.** No anchor-position registry exists yet (see below) to place `vitality`
at position 1 and `lifecycle` at position 7 the way the frozen engine's tick order would — so
every rule in a slice-1 content set is, from the driver's point of view, UNPOSITIONED: it cannot
tell two systems' rules apart by position any more than it could tell two rules at the SAME
position apart. Treating the whole set as if it sat at one shared (unresolved) anchor position and
applying D16's fallback — ascending rule-id byte order — gives the honest, spec-grounded answer
available today, not an invented alternative: **`split_content` collects every `(rule …)`
form (in whatever order the reader encounters them — this remains unchanged from Task 2 below),
and `prepare_rules` SORTS the resulting list by rule-id byte order before returning it.** Every
later stage (`TickSession::advance`, `run_once_into`) simply iterates the already-sorted list, so
sorting happens exactly once, at load time, not per tick.

**Worked for the demo pair.** `"lifecycle/dpd-circuit"` sorts before `"vitality/subsistence-and-
death"` in ascending byte order (`l` = 0x6C < `v` = 0x76) — the REVERSE of the frozen engine's own
tick-position order (Vitality @1, Lifecycle @7). `TickReport.per_rule_fired[0]` holds
`lifecycle`'s entry and `[1]` holds `vitality`'s, throughout this plan — every task and test below
uses that order, corrected from the first amendment's vitality-first assumption.

**Why the reversal is safe for THIS pair, and why the driver must not generalize that safety.**
The two rules' domains are fully disjoint (verified by reading both rules' complete `(bindings …)`
blocks — see the subsection below), so their write-sets never interact and the CANONICAL
(sorted-before-hashing) state after one tick is identical whichever of the two orders ran. That is
a property of THIS pair, not of the mechanism: two rules sharing a node type, or one reading a
field the other writes, would produce genuinely different post-tick state under the two orders,
and D16's byte-order rule is exactly what decides which one is correct — never file order, never
which pack a demo author happened to list first. **This is precisely why §4.2 scopes byte order to
rules "at the same anchor position"**: the moment two systems need to run in a specific ENGINE
order regardless of their rule ids' spelling (an OODA-adjacent example, or any pair where sequence
carries real material meaning), that is the anchor-position registry's job, not byte order's — and
that job stays deferred, with a name, exactly as before this revision.

**Why NOT the formal `:anchor` mechanism (unchanged finding, restated).** `bsl-language.rst` §2.3
already specifies `<anchor> ::= "(" "anchor" ( ":after" | ":before" ) <symbol> ")"` and a default
("a rule with no `<anchor>` belongs to the system named by the first segment of its rule id and
takes that system's declared position"). `mod_anchors.rs`'s own module doc says outright, *"this
module validates the DECLARATION — shape, and the `E-LOAD-002` no-system case. Resolving anchors
into a total order belongs to `babylon-engine`'s anchor-based registry (Phase 3)… deferred with a
name, not silently."* `check_anchor` runs inside `load_rule_form` today (`rule_pipeline.rs:245`)
and stores the result on `LoadedRule.anchor` — but nothing anywhere reads that field for ordering;
no system-position registry exists to resolve `:after`/`:before`, or a system name to a numeric
position, against. Building that registry sits explicitly outside this plan's scope (a Phase 3
BSL-track milestone, not a B2 client-lane task). D16's byte-order rule, by contrast, needs NO
registry — already fully specified and already implementable, which is exactly why it stands as
the correct fallback rather than an interim invented for this plan. The `(anchor …)` forms Task 5 adds
below remain purely declarative — parsed, validated, and inert for ordering, exactly as they are
for every other content set in this repo today; they exist as forward documentation for the day
the registry lands, not as an ordering mechanism this driver reads.

### The two rules' domains are disjoint — a subtlety worth stating precisely

`vitality/subsistence-and-death`'s bindings read/write only `social-class/*` fields and `economy/*`
constants; `lifecycle/dpd-circuit`'s bindings read/write only `territory/*` fields and `lifecycle/*`
constants (both verified by reading each rule's full `(bindings …)` block). Neither rule's subject
type, field reads, or field writes touch the other's. **This is not merely an inference from
reading the bindings by hand — the driver itself computes and enforces it at runtime.**
`tick.rs::subject_type_of` derives a rule's subject type from the SINGLE qname prefix its `:field`
bindings share (loading a rule whose bindings span more than one prefix loudly fails —
<!-- vale off -->
`E-LOAD`-shaped: "the rule's :field bindings span N namespaces… so its subject type is ambiguous"
<!-- vale on -->
); `run_tick` then calls `graph.nodes(&subject_type)` and iterates ONLY that result. Because
`vitality`'s bindings resolve to the single prefix `social-class` and `lifecycle`'s to the
single prefix `territory`, the two rules iterate DISJOINT NODE SETS at the substrate level —
`graph.nodes("SOCIAL_CLASS")` and `graph.nodes("TERRITORY")` — not merely disjoint field
prefixes on a shared set. This is the stronger, load-bearing argument; the bindings-reading
check above is corroborating evidence, not the proof itself. Two consequences follow, and BOTH
matter to how Task 5 builds its conformance test:

1. **The final canonical state hash is order-invariant for THIS pair, specifically**, because
   `CanonicalState::encode_state` sorts every section before hashing (ADR193) and the two rules'
   write-sets never overlap or interact — running lifecycle-then-vitality (byte order) or
   vitality-then-lifecycle (engine order) produces the identical SET of (node, attribute, value)
   triples, and a canonical sort of an identical set hashes identically either way. **This is an
   accident of this specific pair's disjoint domains, not a property of the multi-rule mechanism in
   general** — a future pair that shares a node type or cross-reads a field would NOT enjoy this
   invariance, and the driver must not (and does not) assume it does.
2. **Because of (1), a test that only asserts the final hash would NOT, by itself, prove the
   driver sorts by rule id rather than by file order** — both orderings happen to produce the same
   hash for this pair, so a hash-only test could not distinguish "the driver correctly sorts by
   byte order" from "the driver still (incorrectly) preserves file order" if someone happened to
   concatenate the two `.bsl` files in byte-sorted order already. Task 5's conformance test asserts
   on TWO things as a result: `per_rule_fired`'s own order (must always read
   `["lifecycle/dpd-circuit", "vitality/subsistence-and-death"]`, regardless of which order the
   caller concatenates the two `.bsl` files in — the direct proof the driver sorts, not
   preserves), and, separately, that two content sets built from the SAME two rules in DIFFERENT
   file/concatenation order produce byte-IDENTICAL `TickReport`s — the actual property §4.2
   promises ("file order and load order are never observable"), now a committed test rather than
   an assertion left implicit.

### Field and local-name collisions — checked, none found

The union scenario (Phase B) mints social-class nodes (`vitality`'s fixture) and territory nodes
(`lifecycle`'s fixture) side by side. Checked explicitly, not assumed:

- **`deffield` qnames**: `vitality-conformance.bscn` declares `social-class/{active, population,
  wealth, subsistence-multiplier, s-bio, s-class, inequality}` (7 fields, the 7th unread by the
  rule but seeded for fixture parity). `lifecycle-conformance.bscn` declares `territory/{pop-d,
  pop-p, pop-d-prime, wealth-d-prime, dependency-ratio, legitimation-index, legitimation-crisis,
  transmitted-ideology}` (8 fields). The owning node type prefixes every qname, per the
  language's own convention (§2.9) — the two sets share zero names by construction, not by luck.
- **`defconst` qnames**: `vitality` declares `economy/{base-subsistence, death-threshold}` (2).
  `lifecycle` declares 21 `lifecycle/*`-prefixed constants. Zero overlap.
- **Local node names**: `vitality`'s six fixture nodes carry the names `core`, `bourgeoisie`,
  `hermit`, `last-worker`, `remnant`, `dissolved`. `lifecycle`'s twelve demo nodes carry the names
  `county-<fips>` (Phase B, Task 7). Zero overlap — `load_scenario`'s duplicate-local-name
  check (`scenario.rs`) never fires.
- **`CardinalityCeilings`**: `prepare_rule`'s existing ceiling-building code
  (`scenario.node_types.iter().map(...)`) is already generic over any number of distinct
  `NodeType` members a scenario mints — verified by reading it; no change needed for two node
  types instead of one, and Task 6 keeps this code path unmodified. **Two DIFFERENT string
  conventions coexist in this crate, both correct in their own place, worth naming explicitly so
  no one "fixes" one to match the other:** `CardinalityCeilings::new` deliberately keys its map
  with the PREFIXED form (`format!("NodeType/{member}")` — this is the bound checker's own load-time
  vocabulary, matching how a rule's `<enum-ref>` looks in SOURCE); `GraphSubstrate::nodes()` and
  `node_attribute()` key on the BARE form the substrate actually stores (`"TERRITORY"`, never
  `"NodeType/TERRITORY"` — the BLOCKER this revision fixes at every Rust call site). The two never
  need to agree with each other, because they answer different questions at different layers — one
  is "how many of this enum-ref might a rule legally query," the other is "which nodes does the
  live graph actually hold" — but a reader moving between `prepare_rules` and `lens.rs`/
  `engine_link.rs` should not assume the same string shape carries across that boundary.
- **`systems` registry**: the existing fixed `HashSet` in `prepare_rule`/`prepare_rules`
  already contains both `"vitality"` and `"lifecycle"` (it has since the lifecycle port merged) —
  no change needed there either.

---

## File Structure

| Phase | File | Action | Responsibility |
|---|---|---|---|
| A | `rust/crates/babylon-tick/src/lib.rs` | Edit | Factor `prepare_rule` out of `run_once_into`; later widen to `prepare_rules`; add `pub mod session;` |
| A | `rust/crates/babylon-bsl/src/rule_pipeline.rs` | Edit | `split_content` admits more than one `(rule …)` form, duplicate-id check, reuses `canonical_ast::rule_id` |
| A | `rust/crates/babylon-bsl/src/canonical_ast.rs` | Edit | Widen `rule_id`'s visibility to `pub(crate)` so `rule_pipeline.rs` can reuse it (NIT fix — no third extractor) |
| A | `docs/reference/bsl-language.rst` | Edit | New D-row (next free register number — see the D-row Numbering note) applying D16/§4.2 to the widened driver |
| A | `tests/unit/reference/test_bsl_grammar_sync.py` | Edit | New register-row-uniqueness guard (DEFECT fix — closes the "second `* - D99` row" class) |
| A | `rust/crates/babylon-tick/content/rules/lifecycle.bsl` | Edit | Add `(anchor :after vitality)` — declarative only, inert for ordering today |
| A | `rust/crates/babylon-tick/content/scenarios/vitality-lifecycle-combined-conformance.bscn` | Create | The 10-node conformance fixture (6 vitality + 4 lifecycle, verbatim) |
| A | `rust/crates/babylon-tick/tests/multi_rule_conformance.rs` | Create | Byte-order-sort + file-order-invariance proof (§4.2/D16) |
| A | `rust/crates/babylon-tick/src/session.rs` | Create | `TickSession<G>` — load once, `advance()` many times, now multi-rule |
| B | `rust/crates/babylon-client/tests/print_demo_counties.rs` | Create (throwaway aid) | One-shot atlas print, deleted after use |
| B | `rust/crates/babylon-tick/content/scenarios/us-counties-lifecycle-demo.bscn` | Create | 18-node demo: 12 real-FIPS territories (twelve DISTINCT population seeds) + 6 social classes |
| C | `rust/crates/babylon-client/src/lens.rs` | Create | `county_tension` (ADR170, ported) + `county_legitimation` (new) + `county_population_trend` (new, BLOCKER 2 fix) |
| C | `rust/crates/babylon-client/src/map/bands.rs` | Edit | ADR191 R11's `band_color` (Tension) + legitimation band function (Director ruling 1) + population-trend band function |
| C | `rust/crates/babylon-client/src/map/pick.rs` | Create | Uniform-grid hit test (B1 Task 10's design) |
| C | `rust/crates/babylon-client/src/map/hud.rs` | Create | Hover/selection readout, active-lens label (now 3-way), absence banner |
| C | `rust/crates/babylon-client/src/map/mod.rs` | Edit | Wire the four new modules + 3-way lens-picker input |
| D | `rust/crates/babylon-client/src/engine_link.rs` | Edit | `EngineSession` resource: `TickSession<HypergraphStore>` + `CollectingSink` + FIPS↔`NodeId` map + tick-0 population baseline |
| D | `rust/crates/babylon-client/src/main.rs` | Edit | Advance-tick input, tick counter, hash readout, event feed |
| E | `rust/crates/babylon-client/src/logging.rs` | Create | Resurrected `log4rs` file sink |
| E | `rust/crates/babylon-client/Cargo.toml` | Edit | `log`, `log4rs` deps |
| E | `rust/crates/babylon-client/tests/determinism.rs` | Create | Same-content, same-tick-count ⇒ same hash, end to end |
| E | `rust/crates/babylon-client/tests/eyes_on_smoke.rs` | Create | Headless proxy for the eyes-on gate |

**D-row numbering note (DEFECT fix).** Open PR #500 (ADR194 R2, the Currency-scale operation) also
mints a `D99` register row and has not merged as of this revision. This plan never hard-codes a
row number as a result: Task 3 below reads `docs/reference/bsl-language.rst`'s register table AT
EXECUTION TIME, finds the highest existing `D<N>` row, and mints `D<N+1>` — `D99` if #500 has not
landed yet, `D100` if it has. Every reference in this plan to "the D-row this task adds" means
that resolved number, not a literal "D99."

---

## Phase A — The persistent tick session, and the multi-rule content-set evolution

### Task 1: Factor `prepare_rule` out of `run_once_into`

**Files:**

- Edit: `rust/crates/babylon-tick/src/lib.rs`

**Interfaces:**

- Produces: `pub(crate) struct PreparedRule { loaded: LoadedRule, types: TypeEnv, intrinsics:
  IntrinsicCosts, consts: HashMap<String, Value> }` and `pub(crate) fn prepare_rule<G:
  GraphSubstrate + CanonicalState>(scenario_src: &str, rule_src: &str, graph: &mut G) ->
  Result<PreparedRule, String>` — everything `run_once_into` currently does BEFORE its call to
  `run_tick`.
- Consumes (unchanged): `split_content`, `parse_intrinsic_decls`, `IntrinsicCosts::new`,
  `load_scenario`, `TypeEnv`, `BindingVocabulary`, `CardinalityCeilings`, `LoadContext`,
  `load_rule_form` — every import already at the top of `lib.rs` stays.

**Deliberately single-rule.** This task's `PreparedRule`/`prepare_rule` hold exactly ONE
`LoadedRule`, matching `split_content`'s CURRENT (pre-Task-2) one-rule cardinality — the smallest,
safest first step, proven against the existing single-rule goldens before Task 2 widens the loader
underneath it. Task 4 renames and widens this to `PreparedRules`/`prepare_rules`; this task's job
is only to prove the pure-extraction refactor is behavior-preserving in isolation first.

This is a **pure extraction, zero behavior change**: `run_once_into`'s existing 130-odd lines
split at exactly the point after `let loaded = load_rule_form(...)` — everything before that line
moves into `prepare_rule`, returning the four values it currently holds locally; everything from
`run_tick(...)` onward stays in `run_once_into`, now reading `prepared.loaded`,
`prepared.types`, and so on, instead of local bindings.

- [x] **Step 1: Write the regression-proof test FIRST** (red only in the sense that it must stay
      green through the refactor — there is no new behavior to fail on). Confirm the two existing
      tests already cover this:

```rust
// already in lib.rs's #[cfg(test)] mod tests — read, do not duplicate:
//   run_once_is_deterministic (two-classes.bscn / the fundamental-theorem rule)
// already in tests/engine_link.rs (babylon-client):
//   startup_tick_matches_the_pinned_hash — hex(&report.after) ==
//   "783f651d04d32fffd0109e88423eb7a57b1e0836ed4a9f645d3a8a554e427679"
```

      Run both now, before touching any code: `cargo test -p babylon-tick` and
      `cargo test -p babylon-client --test engine_link` → both PASS. This is the baseline the
      refactor must not move.
- [x] **Step 2: Extract `prepare_rule`.**

```rust
/// Everything `run_once_into` does before running a single tick: parse the
/// intrinsic declarations, load the scenario into `graph`, and load the
/// one `(rule …)` form against the vocabulary/types/ceilings that scenario
/// declared. Shared by `run_once_into` (which still runs exactly tick 1)
/// and, from Task 4 on, `prepare_rules`'s multi-rule successor.
pub(crate) struct PreparedRule {
    pub loaded: LoadedRule,
    pub types: TypeEnv,
    pub intrinsics: IntrinsicCosts,
    pub consts: HashMap<String, Value>,
}

pub(crate) fn prepare_rule<G: GraphSubstrate + CanonicalState>(
    scenario_src: &str,
    rule_src: &str,
    graph: &mut G,
) -> Result<PreparedRule, String> {
    let (intrinsic_forms, rule_form) = split_content(rule_src).map_err(|e| e.to_string())?;
    let declared = parse_intrinsic_decls(&intrinsic_forms).map_err(|e| e.to_string())?;
    let intrinsics = IntrinsicCosts::new(
        declared
            .into_iter()
            .map(|(name, decl)| (name, decl.cost))
            .collect(),
    );

    let scenario = load_scenario(scenario_src, graph).map_err(|e| e.to_string())?;

    let types = TypeEnv {
        fields: scenario.fields.clone(),
        exemptions: &[],
    };
    let vocabulary = BindingVocabulary {
        fields: scenario.fields.keys().cloned().collect(),
        consts: scenario.consts.keys().cloned().collect(),
        metrics: HashSet::new(),
    };
    let ceilings = CardinalityCeilings::new(
        scenario
            .node_types
            .iter()
            .map(|(member, count)| (format!("NodeType/{member}"), *count))
            .collect(),
        HashMap::new(),
    );
    let systems: HashSet<String> = HashSet::from([
        "economics".to_owned(),
        "vitality".to_owned(),
        "consciousness".to_owned(),
        "lifecycle".to_owned(),
    ]);

    let ctx = LoadContext {
        vocabulary: &vocabulary,
        types: &types,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: None,
        rule_file: "rule",
    };
    let loaded = load_rule_form(rule_form, &ctx).map_err(|e| format!("rule rejected: {e}"))?;

    Ok(PreparedRule {
        loaded,
        types,
        intrinsics,
        consts: scenario.consts,
    })
}
```

- [x] **Step 3: Rewrite `run_once_into` to call it.**

```rust
pub fn run_once_into<G: GraphSubstrate + CanonicalState>(
    scenario_src: &str,
    rule_src: &str,
    graph: &mut G,
    sink: &mut CollectingSink,
) -> Result<TickReport, String> {
    let prepared = prepare_rule(scenario_src, rule_src, graph)?;

    let before = graph
        .state_hash()
        .map_err(|e| format!("pre-tick state: {}", e.message))?;

    let outcome = run_tick(
        &prepared.loaded,
        &prepared.types,
        &KernelIntrinsicHost,
        graph,
        sink,
        &prepared.intrinsics,
        &prepared.consts,
        1,
    )
    .map_err(|e| format!("tick failed: {e}"))?;

    let after = graph
        .state_hash()
        .map_err(|e| format!("post-tick state: {}", e.message))?;

    Ok(TickReport {
        before,
        after,
        fired: outcome.fired,
        per_rule_fired: vec![(prepared.loaded_rule_id.clone(), outcome.fired)],
    })
}
```

      Note the pre-tick hash is now taken AFTER `prepare_rule` returns rather than immediately
      after `load_scenario` — this is the same point in program order (nothing between the old
      `load_scenario` call and the old `before` computation touches `graph`), so the hash value is
      identical; only the code that produces it moved. **`per_rule_fired` does not exist yet at
      this step** — Task 4 adds `TickReport.per_rule_fired` and the `loaded_rule_id` field this
      line anticipates; write `run_once_into` WITHOUT that line for this task (`fired:
      outcome.fired` only, matching today's struct), and let Task 4 add the field and this line
      together. Flagged here so the two tasks' interfaces read as one coherent design, not two
      that happen to agree.
- [x] **Step 4:** Run both Step 1 tests again → PASS, byte-identical hash. `mise run rust:check` →
      green. `mise run qa:regression` and `mise run qa:vault-regression-ci` → byte-identical (this
      refactor is inside the engine crate; both gates must stay silent).
- [x] **Step 5: Commit** (`refactor(rust): factor prepare_rule out of run_once_into — zero behavior
      change (B2)`).

### Task 2: Widen `split_content` to admit more than one `(rule …)` form

**Files:**

- Edit: `rust/crates/babylon-bsl/src/rule_pipeline.rs`

**Interfaces:**

- Produces: `pub fn split_content(source: &str) -> Result<(Vec<SExpr>, Vec<(String, SExpr)>),
  LoadError>` — the SAME function name, now returning the intrinsic-decl forms plus a
  **non-empty** `Vec<(String, SExpr)>` of `(rule_id, rule_form)` pairs, one per `(rule …)`
  top-form (in reader-encounter order — Task 4 sorts this into rule-id byte order,
  `split_content` itself makes no ordering claim), duplicate ids refused. **Signature change**:
  the second element of the tuple was `SExpr` (exactly one, un-paired), is now `Vec<(String,
  SExpr)>` (one or more, PRE-PAIRED with each rule's own id) — every caller (`prepare_rule` today,
  `prepare_rules` from Task 4) updates in the same PR. Returning the id alongside the form, rather
  than making the caller re-extract it, is deliberate: `split_content` already computes every
  rule's id internally for the duplicate-id check, so handing it back avoids a second extraction
  downstream AND avoids needing to expose a `rule_id` function outside this crate at all —
  `babylon-tick` (Task 4's crate) never needs to call one.
- **No new `rule_id` function — reuses the crate's existing one, kept crate-internal.**
  `babylon-bsl` already carries TWO rule-id extractors (`canonical_ast.rs:65`, private `fn
  rule_id(expr: &SExpr) -> Result<&str, CasError>`, used by the CAS hashing code; `bound_checker.rs:678`,
  `fn rule_id(items: &[SExpr]) -> String`, a lenient error-reporting helper that never fails) —
  found by reading both before writing a third. `canonical_ast::rule_id` is the right one to
  share: strict (errors rather than guessing), matches the duplicate-id check's own strictness
  need, and already pattern-matches the qname correctly as `Atom::QName` (not `Atom::Symbol` — the
  shape this task's FIRST draft got wrong before this revision caught it against the working
  implementation). This task widens its visibility from private to `pub(crate)` — same-crate only,
  since `split_content` (also this crate) is its only new caller — and reuses it, converting
  `CasError` to `LoadError::Content` at the one call site. No third implementation, and no widening
  to fully `pub` either, since nothing outside `babylon-bsl` needs to call it directly.

- [x] **Step 1: Write the failing tests.** In `rule_pipeline.rs`'s existing `#[cfg(test)] mod
      tests`:

```rust
#[test]
fn split_content_admits_two_rules_in_source_order() {
    let source = r"
(rule a/first :material-basis "x" :fuel 10
  (bindings (binding v :field a/v))
  (effects (update-node self a/v (set v))))
(rule b/second :material-basis "y" :fuel 10
  (bindings (binding v :field b/v))
  (effects (update-node self b/v (set v))))
";
    let (_intrinsics, rules) = split_content(source).expect("two distinct rule ids load");
    assert_eq!(rules.len(), 2);
    assert_eq!(rules[0].0, "a/first");
    assert_eq!(rules[1].0, "b/second");
}

#[test]
fn split_content_still_admits_exactly_one_rule() {
    // The pre-Task-2 shape stays legal — this widening is additive, never
    // a floor raise. Every existing single-rule content set in the repo
    // must keep loading unchanged.
    let source = r#"(rule a/only :material-basis "x" :fuel 10
  (bindings (binding v :field a/v))
  (effects (update-node self a/v (set v))))"#;
    let (_intrinsics, rules) = split_content(source).expect("one rule still loads");
    assert_eq!(rules.len(), 1);
}

#[test]
fn split_content_refuses_zero_rules() {
    let err = split_content("").unwrap_err();
    assert!(err.to_string().contains("found 0"));
}

#[test]
fn a_duplicate_rule_id_across_the_content_set_is_e_load_001() {
    let source = r#"
(rule a/dup :material-basis "x" :fuel 10
  (bindings (binding v :field a/v))
  (effects (update-node self a/v (set v))))
(rule a/dup :material-basis "y" :fuel 10
  (bindings (binding v :field a/v2))
  (effects (update-node self a/v2 (set v))))
"#;
    let err = split_content(source).unwrap_err();
    assert!(err.to_string().contains("E-LOAD-001"));
    assert!(err.to_string().contains("a/dup"));
}
```

- [x] **Step 2:** `cargo test -p babylon-bsl` → FAIL (the current `<[SExpr; 1]>::try_from` cardinality
      check refuses two rules; the return type does not compile against `Vec<SExpr>` callers yet).
- [x] **Step 3: Widen the function.** Replace the current

```rust
match <[SExpr; 1]>::try_from(rule_forms) {
    Ok([rule]) => Ok((intrinsic_forms, rule)),
    Err(rule_forms) => Err(LoadError::Content(format!(
        "a content set needs exactly one (rule …) top-form, found {} …",
        rule_forms.len()
    ))),
}
```

      with:

```rust
if rule_forms.is_empty() {
    return Err(LoadError::Content(
        "a content set needs at least one (rule …) top-form, found 0 \
         (§2.2 — intrinsic declarations do not count; deffield/manifest/metric-decl \
         top-forms are not yet split out by this function and would also land here)"
            .to_owned(),
    ));
}
let mut seen: HashMap<String, ()> = HashMap::with_capacity(rule_forms.len());
let mut paired = Vec::with_capacity(rule_forms.len());
for form in rule_forms {
    let id = crate::canonical_ast::rule_id(&form)
        .map_err(|e| LoadError::Content(e.message))?
        .to_owned();
    if seen.contains_key(&id) {
        return Err(LoadError::Content(format!(
            "E-LOAD-001: duplicate rule id: {id} (§2.2 — rule ids must be \
             content-set-unique, the same duplicate-name discipline \
             parse_intrinsic_decls already enforces for intrinsic \
             declarations)"
        )));
    }
    seen.insert(id.clone(), ());
    paired.push((id, form));
}
Ok((intrinsic_forms, paired))
```

      matching the EXACT `HashMap::contains_key`-before-insert pattern
      `declarations::parse_intrinsic_decls` already uses for duplicate intrinsic names (same file
      family, same §2.2 duplicate-name discipline) — reused, not reinvented, per DRY. In
      `canonical_ast.rs`, widen the existing private extractor's visibility only — no signature
      change, no behavior change, and its own CAS-hashing callers keep compiling unmodified:

```rust
// canonical_ast.rs — was `fn rule_id`, now `pub(crate) fn rule_id`, so
// rule_pipeline.rs (same crate) can reuse it instead of writing a second
// strict extractor. Body unchanged.
pub(crate) fn rule_id(expr: &SExpr) -> Result<&str, CasError> {
    // ... existing body, verbatim ...
}
```

- [x] **Step 4:** `cargo test -p babylon-bsl` → PASS (all four new tests; every EXISTING
      `split_content`/`load_rule_form`/`canonical_ast` test in the crate still green — this is
      additive, not a behavior change for single-rule content or for the CAS hashing code). Update
      `prepare_rule` (Task 1) to destructure the now-`Vec<(String, SExpr)>` second element as
      `rule_forms[0].1.clone()` (still one rule at this point in the plan, and `prepare_rule`
      never needed the id anyway — Task 4 removes the `[0]` indexing and starts consuming the
      paired id when it widens to multi-rule) — a small, mechanical signature-follow, not a
      behavior change.
- [x] **Step 5:** `mise run rust:check` → green (workspace-wide — this crate's callers in
      `babylon-tick` must still compile). `mise run qa:regression` → byte-identical.
- [x] **Step 6: Commit** (`feat(rust): split_content admits more than one (rule …) form, duplicate ids
      refused (B2) — honors §2.2's already-ratified grammar`).

### Task 3: The next-free-register spec row — documenting the widened driver

**Files:**

- Edit: `docs/reference/bsl-language.rst`

**Why this is a task, not a footnote.** The D-row discipline (D80…D98 already in the table) is
this document's own normative-home rule: a workforce reading that changes what the DRIVER accepts,
even when the change is "honor what the grammar already said," gets its own row so a future reader
does not have to reconstruct the reasoning from a Rust doc comment. This follows D97/D98's own
precedent — "a Phase-1-review reading… open to correction, not a Director ruling" — the same
posture this row takes.

**Row number — resolve at execution time, never hard-code.** Open PR #500 (ADR194 R2) also mints a
`D99` row and has not merged as of this plan. Before writing anything, run
`rg -o '\* - D[0-9]+' docs/reference/bsl-language.rst | grep -oE '[0-9]+' | sort -n | tail -1` (or
a similar command) to find the highest existing row number N, and use **D(N+1)**. If #500 has
already merged, that is D100; if not, D99. Every occurrence of "D99" anywhere else in this plan document
(this task, the File Structure table, later cross-references) means this resolved number, not a
literal string — search the plan for the literal text `D99` before executing this task and confirm
none of the OTHER occurrences got hard-coded into committed code or test strings by mistake.

- [x] **Step 1: Add the row** to the D-row list-table (after the current last row, following the
      exact three-column format every row above it uses; the block below uses `D<N>` as a
      placeholder for the resolved number from the paragraph above — substitute it, do not commit
      the literal string `D<N>`):

```rst
   * - D<N>
     - §2.2, §4.2
     - **The content-set loader admits more than one ``(rule …)`` top-form,
       executed in ascending rule-id byte order (register row D16, §4.2),
       duplicate ids refused** — a driver-level fix (Program 28 B2), not a
       spec change. §2.2's grammar (``<top-form>*``) and prose ("Duplicate
       rule ids… across the content set are ``E-LOAD-001``") never limited
       a content set to one rule; ``babylon-bsl::rule_pipeline::
       split_content`` did, by an implementation-level cardinality check
       with no textual basis in this section. This row lifts that check to
       match the grammar it was always supposed to implement, and applies
       an EXISTING ruling to the result — D16 already says "rules at the
       same anchor position evaluate in ascending rule-id byte order… file
       order and load order are never observable"; a slice-1 content set
       carries no anchor-position registry to differentiate positions
       across systems (Phase 3, unbuilt), so every rule in it is
       effectively at one shared, unresolved position, and D16's byte-order
       fallback is what governs. **This row mints no new ordering law** —
       it is an application of D16 to a case D16's own text already
       covers, not a second rule alongside it. ``(anchor …)`` forms stay
       parseable and validated (``check_anchor``, unchanged) but inert for
       ordering under this row, exactly as before it — resolving them into
       a cross-system total order remains Phase 3's job, deferred with a
       name.
       Reference implementation: ``rule_pipeline::split_content``,
       ``canonical_ast::rule_id`` (widened to ``pub(crate)``),
       ``lib::prepare_rules`` (Program 28 B2, `docs/superpowers/plans/
       2026-08-11-b2-tick-loop-plan.md` Phase A Tasks 2–4).
```

- [x] **Step 2: Sync `bsl.ebnf` if it encodes a rule-cardinality constraint.** Grep it for any
      `<file>`/`<top-form>` production carrying an explicit "exactly one rule" note; §2.2's own
      grammar block above (the normative one) never had one, so this step is almost certainly a no-op —
      confirm rather than assume, per the D95/D98 precedent of keeping the appendix and the section
      text in the same commit when they diverge.
- [x] **Step 3:** `vale docs/reference/bsl-language.rst` → 0 (this file already carries a project
      vocabulary; this row's prose should clear it without a new exemption).
- [x] **Step 4: Commit** (`docs(bsl): D<N> — the content-set loader applies D16's byte order to the
      multi-rule case (B2)`, with the resolved number substituted in the message), sequenced right
      after Task 2 since it documents exactly that change.

### Task 3b: The register-row uniqueness guard (DEFECT fix — closes the collision class)

**EXECUTION NOTE (2026-08-11, B2 implementation).** This task is ALREADY SATISFIED by
`723a4c23` (`test(bsl): harden the register sync-guard against duplicate D-row numbers`), an
ancestor of this branch's tip (`cc836fea`) — confirmed via `git merge-base --is-ancestor
723a4c23 HEAD`. That commit lands the same guard this task specifies, in the same file, closing
the exact collision class: a `TestTheDraftRulingRegisterHasNoDuplicateRowNumbers.
test_every_register_row_number_is_unique` scanning every `* - D<n>` row between the "Draft-Ruling
Register" and "See Also" headings and asserting no number repeats (`Counter`-free — a set-based
duplicate check over `>= 90` rows, functionally equivalent to this task's `Counter`-based sketch).
Its own commit message names the SAME incident this task's rationale cites (PR #500 and a
parallel plan both independently reaching for `D99`). Running it confirms it also catches Task
3's own new `D100` row correctly (no duplicate, 28/28 tests in the file pass). No new test was
written — writing a second, near-identical guard in the same file would duplicate rather than
close the gap. Steps below are checked off against that existing guard, not a new one.

**Files:**

- Edit: `tests/unit/reference/test_bsl_grammar_sync.py`

**The exact gap this closes.** The existing sync-guard file only ever asserts a SPECIFIC row
EXISTS (`test_the_register_carries_d92`, `test_d98_is_recorded_in_the_register`, both a
`re.search(r"^\s+\* - D\d+$", ...)` on one fixed number). That style of test cannot catch a
DUPLICATE — `re.search` returns on the FIRST match and never asks whether a second one exists.
Verified directly: if this plan's Task 3 and PR #500 both land a `* - D99` row (the exact
collision this revision's own D-row numbering note exists to avoid, but a FUTURE pair of
concurrent PRs could reproduce the same class of bug), every existing `test_the_register_carries_
d*` test for either row would still PASS, silently. This task adds the general guard so the CLASS
closes, not just this one instance.

- [x] **Step 1: Write the failing test.** Already satisfied by `723a4c23` (see the execution note
      above) — not re-written.

```python
def test_every_register_row_number_is_unique() -> None:
    """No two D-rows may share a number — `re.search`'s existence checks
    above cannot catch this (first-match semantics), so this test walks
    every row instead. Verified as a real gap during Program 28 B2's
    adversarial review: PR #500 and this plan's own D-row task both
    proposed `D99` independently, and every existing per-row test would
    have passed with both landed. See docs/superpowers/plans/
    2026-08-11-b2-tick-loop-plan.md for the incident this test answers.
    """
    body = _read(RST)
    numbers = re.findall(r"^\s+\* - D(\d+)$", body, re.MULTILINE)
    assert numbers, "the register table must contain at least one D-row"
    counts = Counter(numbers)
    duplicates = {n: c for n, c in counts.items() if c > 1}
    assert not duplicates, (
        f"duplicate register row numbers: {duplicates} — each D<N> must be "
        "unique; renumber the later-landed row to the next free number"
    )
```

      `Counter` needs `from collections import Counter` added to the file's existing imports.
- [x] **Step 2:** Run it now, before this plan's own D-row lands — PASS (the register currently has
      no duplicates). This is the regression-proof baseline, not a red phase in the usual TDD
      sense: there is no bug to fix yet, only a gap in coverage to close.
- [x] **Step 3:** `mise run test:q -- tests/unit/reference/test_bsl_grammar_sync.py` → PASS (this
      new test plus every existing test in the file, unmodified).
- [x] **Step 4: Commit** — N/A, no new commit: `723a4c23` already landed this guard (see the
      class (B2)`), landed independently of Task 3 so it protects Task 3's own row from the moment
      it lands, not after.

### Task 4: `prepare_rule` → `prepare_rules`; `run_once_into` runs every rule in order

**Files:**

- Edit: `rust/crates/babylon-tick/src/lib.rs`

**Interfaces:**

- Produces: `pub(crate) struct PreparedRules { rules: Vec<(String, LoadedRule)>, types: TypeEnv,
  intrinsics: IntrinsicCosts, consts: HashMap<String, Value> }` (each entry pairs a rule's own id
  with its `LoadedRule`, SORTED into ascending rule-id BYTE order — §4.2, register row D16, never
  the order `split_content` happened to encounter the forms in) and `pub(crate) fn
  prepare_rules<G: GraphSubstrate + CanonicalState>(scenario_src: &str, rule_src: &str, graph: &mut
  G) -> Result<PreparedRules, String>` — `prepare_rule`'s direct successor, same shape, now
  walking every rule `split_content` returns, loading each, and sorting the result by id before
  returning.
- **`TickReport` gains one field, existing fields UNCHANGED in type** — this is the compatibility
  design this task commits to, checked against every current reader of `.fired`:

```rust
pub struct TickReport {
    pub before: [u8; 32],
    pub after: [u8; 32],
    /// The TOTAL fired-subject count across every rule this tick ran —
    /// unchanged in meaning and type for a single-rule content set (today
    /// every existing caller: `run_once`, the CLI, B0's engine-link probe,
    /// every `*_conformance.rs` test). For a multi-rule tick this is the
    /// SUM across rules — kept a plain `usize` rather than widened to
    /// `Vec<usize>` specifically so `report.fired == N` assertions in
    /// `tests/vitality_conformance.rs`, `tests/lifecycle_conformance.rs`,
    /// `tests/lifecycle_crisis_conformance.rs` and
    /// `tests/floor_intrinsic_e2e.rs` (five call sites, grepped and
    /// confirmed) keep compiling and keep passing unmodified.
    pub fired: usize,
    /// Per-rule detail, in ASCENDING RULE-ID BYTE ORDER (§4.2, register row
    /// D16) — `(rule_id, fired)`. NEVER declaration order or file order;
    /// §4.2 says those "are never observable", and this field's own order
    /// is the driver's proof that it honors that. Length 1 for every
    /// existing single-rule content set (`fired == per_rule_fired[0].1`
    /// always holds); length N for an N-rule content set. This is what
    /// Task 5's conformance test and Phase D's event feed actually need —
    /// a summed `fired` alone cannot tell "5 subjects fired" from
    /// "vitality fired on 3, lifecycle on 2".
    pub per_rule_fired: Vec<(String, usize)>,
}
```

- [x] **Step 1: Write the failing regression tests FIRST**, proving the additive-field design holds
      for every EXISTING single-rule caller before writing the multi-rule path:

```rust
// lib.rs's existing #[cfg(test)] mod tests, extended:
#[test]
fn single_rule_content_still_reports_fired_and_a_one_entry_per_rule_fired() {
    let report = run_once(SCENARIO, RULE).expect("single-rule run");
    assert_eq!(report.per_rule_fired.len(), 1);
    assert_eq!(report.per_rule_fired[0].1, report.fired);
}
```

      Also re-run, unmodified, the five existing `.fired`-reading tests named in the doc comment
      above (`tests/vitality_conformance.rs`, `tests/lifecycle_conformance.rs`,
      `tests/lifecycle_crisis_conformance.rs`, `tests/floor_intrinsic_e2e.rs` ×2 assertions) —
      these must compile and pass with ZERO edits, since `fired`'s type did not change.
- [x] **Step 2:** `cargo test -p babylon-tick -p babylon-bsl` → FAIL (`per_rule_fired` field does
      not exist; `prepare_rules` does not exist).
- [x] **Step 3: Widen `prepare_rule` into `prepare_rules`.**

```rust
pub(crate) struct PreparedRules {
    pub rules: Vec<(String, LoadedRule)>,
    pub types: TypeEnv,
    pub intrinsics: IntrinsicCosts,
    pub consts: HashMap<String, Value>,
}

pub(crate) fn prepare_rules<G: GraphSubstrate + CanonicalState>(
    scenario_src: &str,
    rule_src: &str,
    graph: &mut G,
) -> Result<PreparedRules, String> {
    let (intrinsic_forms, rule_forms) = split_content(rule_src).map_err(|e| e.to_string())?;
    let declared = parse_intrinsic_decls(&intrinsic_forms).map_err(|e| e.to_string())?;
    let intrinsics = IntrinsicCosts::new(
        declared
            .into_iter()
            .map(|(name, decl)| (name, decl.cost))
            .collect(),
    );

    let scenario = load_scenario(scenario_src, graph).map_err(|e| e.to_string())?;

    let types = TypeEnv {
        fields: scenario.fields.clone(),
        exemptions: &[],
    };
    let vocabulary = BindingVocabulary {
        fields: scenario.fields.keys().cloned().collect(),
        consts: scenario.consts.keys().cloned().collect(),
        metrics: HashSet::new(),
    };
    let ceilings = CardinalityCeilings::new(
        scenario
            .node_types
            .iter()
            .map(|(member, count)| (format!("NodeType/{member}"), *count))
            .collect(),
        HashMap::new(),
    );
    let systems: HashSet<String> = HashSet::from([
        "economics".to_owned(),
        "vitality".to_owned(),
        "consciousness".to_owned(),
        "lifecycle".to_owned(),
    ]);

    // ONE shared LoadContext for every rule in the content set — the
    // vocabulary/types/ceilings come from the SCENARIO, not from any one
    // rule, and each rule's own load only reads the subset its bindings
    // reference (verified: no cross-rule interference for vitality +
    // lifecycle, whose bindings are wholly disjoint — see the Multi-Rule
    // Decision section's domain-disjointness note).
    let ctx = LoadContext {
        vocabulary: &vocabulary,
        types: &types,
        ceilings: &ceilings,
        intrinsics: &intrinsics,
        systems: &systems,
        vocabulary_registry: None,
        rule_file: "rule",
    };

    // rule_forms is `Vec<(String, SExpr)>` — each rule's id already paired
    // with its form by split_content (Task 2), so no second extraction
    // here. Loaded in WHATEVER order split_content returned them (reader-
    // encounter order, unspecified) — then SORTED by id, ascending byte
    // order, before returning. This is the one place execution order gets
    // decided (§4.2, register row D16): sorting here, once, at load time,
    // means every later stage (TickSession::advance, run_once_into) just
    // iterates the already-correct order and never re-derives it.
    let mut rules = Vec::with_capacity(rule_forms.len());
    for (id, form) in rule_forms {
        let loaded = load_rule_form(form, &ctx)
            .map_err(|e| format!("rule {id} rejected: {e}"))?;
        rules.push((id, loaded));
    }
    rules.sort_by(|(a, _), (b, _)| a.as_bytes().cmp(b.as_bytes()));

    Ok(PreparedRules {
        rules,
        types,
        intrinsics,
        consts: scenario.consts,
    })
}
```

      No `rule_id` call anywhere in this function — Task 2's `split_content` already hands back
      each rule's id paired with its form, so `babylon-tick` never needs to reach into
      `babylon-bsl`'s internal extractor at all, crate boundary respected by construction.
- [x] **Step 4: Rewrite `run_once_into` to loop.**

```rust
pub fn run_once_into<G: GraphSubstrate + CanonicalState>(
    scenario_src: &str,
    rule_src: &str,
    graph: &mut G,
    sink: &mut CollectingSink,
) -> Result<TickReport, String> {
    let prepared = prepare_rules(scenario_src, rule_src, graph)?;

    let before = graph
        .state_hash()
        .map_err(|e| format!("pre-tick state: {}", e.message))?;

    let mut per_rule_fired = Vec::with_capacity(prepared.rules.len());
    for (id, loaded) in &prepared.rules {
        let outcome = run_tick(
            loaded,
            &prepared.types,
            &KernelIntrinsicHost,
            graph,
            sink,
            &prepared.intrinsics,
            &prepared.consts,
            1,
        )
        .map_err(|e| format!("tick failed in rule {id}: {e}"))?;
        per_rule_fired.push((id.clone(), outcome.fired));
    }
    let fired = per_rule_fired.iter().map(|(_, n)| n).sum();

    let after = graph
        .state_hash()
        .map_err(|e| format!("post-tick state: {}", e.message))?;

    Ok(TickReport {
        before,
        after,
        fired,
        per_rule_fired,
    })
}
```

      Every rule in `prepared.rules` runs to COMPLETION (every matching subject) before the next
      rule starts — never interleaved — against the SAME `graph`, so a later rule sees an EARLIER
      rule's writes from the SAME tick, matching the frozen engine's own in-place, strict-order
      mutation semantics (CLAUDE.md: "Systems mutate the shared graph in-place in strict order…
      each system sees prior systems' mutations") for free, with no new mechanism — this falls out
      of calling `run_tick` sequentially against one `&mut G`. **The ORDER `prepared.rules`
      iterates in is rule-id byte order (Step 3's sort), not the frozen engine's tick-position
      order** — for the demo pair this means `lifecycle` runs before `vitality`, backwards from
      the frozen engine's Vitality-@1-before-Lifecycle-@7 — safe only because the Multi-Rule
      Decision section proved their domains disjoint; this loop does not know that and must not be
      read as if it did.
- [x] **Step 5:** `cargo test -p babylon-tick` → PASS (the new regression test; all five
      externally-grepped `.fired` call sites still green, unmodified). `mise run rust:check` →
      green. `mise run qa:regression` and `mise run qa:vault-regression-ci` → byte-identical (this
      widening must move zero bytes for every EXISTING single-rule content set — the whole point of
      the additive-field design).
- [x] **Step 6: Commit** (`feat(rust): prepare_rules — the multi-rule content-set loader, per-rule
      fired detail (B2)`).

### Task 5: The multi-rule conformance vector — byte-order sort, file-order invariance

**Files:**

- Create: `rust/crates/babylon-tick/content/scenarios/vitality-lifecycle-combined-conformance.bscn`
- Create: `rust/crates/babylon-tick/content/scenarios/vitality_lifecycle_combined_conformance.py`
- Create: `rust/crates/babylon-tick/tests/multi_rule_conformance.rs`
- Edit: `rust/crates/babylon-tick/content/rules/lifecycle.bsl` (one line: `(anchor :after
  vitality)`)

**The point of this task, corrected.** Proves TWO things about the mechanism Tasks 2 and 4 built:
(1) it sorts by rule-id byte order regardless of how the caller concatenates `.bsl` text (§4.2,
D16 — "file order and load order are never observable"), and (2) the sorted result reproduces the
frozen engine's own combined output for the demo pair, DESPITE running in the reverse of the
frozen engine's own tick-position order — safe here only because the two rules' domains are
disjoint (Multi-Rule Decision section), which this task's own Python reference script proves
directly rather than assumes.

- [x] **Step 1: The declarative anchor.** Add `(anchor :after vitality)` to `lifecycle.bsl`'s
      `(rule lifecycle/dpd-circuit …)` form, between its `:fuel` keyword and its `(bindings …)`
      form, matching §2.3's grammar position (`<domain>? <anchor>? <bindings>`). This stays
      INERT for ordering today — Task 4's driver sorts by rule-id byte order (D16), never reads
      `.anchor`, and no test in this plan may assert that it does. The anchor is
      forward-documentation for the eventual Phase 3 anchor-resolution registry, landed now while
      the fact ("lifecycle belongs after vitality in true engine order") is fresh and cheap to
      state, EVEN THOUGH the current driver runs them the opposite way (byte order: lifecycle
      first). Confirm `check_anchor` still accepts the form (`cargo test -p babylon-bsl` — the
      existing `lifecycle.bsl` parse/load tests must stay green; adding a valid, well-formed
      anchor changes nothing else about the rule's load).
- [x] **Step 2: The 10-node combined-conformance scenario.** Union `vitality-conformance.bscn`'s
      six social-class nodes (`core`, `bourgeoisie`, `hermit`, `last-worker`, `remnant`,
      `dissolved`, every field value transcribed byte-for-byte) and `lifecycle-conformance.bscn`'s
      four territory nodes (`core-county`, `growing-county`, `recovering-county`, `young-county`,
      same transcription discipline) into ONE `.bscn` file, combining the `deffield` blocks (7 +
      8 = 15 field declarations) and the `defconst` blocks (2 + 21 = 23 constant declarations) —
      the Multi-Rule Decision section's collision check already confirms zero name overlap in
      either category, so this is a straight concatenation, not a merge requiring judgment calls.
      Name it `vitality-lifecycle-combined-conformance.bscn`; a dedicated, small, ALREADY-PROVEN
      fixture, kept separate from Phase B's larger, real-FIPS-flavored demo scenario — this task's
      job is proving the MECHANISM, Phase B's is building the PLAYABLE world, and conflating them
      would make a mechanism bug harder to isolate from a demo-content bug.
- [x] **Step 3: The combined Python reference script.** `vitality_lifecycle_combined_conformance.py`
      mirrors the calling convention `vitality_conformance.py` and `lifecycle_conformance.py`
      already establish (both exist in this same directory — read them first, match their
      structure, do not invent a new one): build ONE `WorldState`/graph carrying all ten fixture
      nodes (the Step 2 values), call `VitalitySystem().step(state)` FOLLOWED BY
      `LifecycleSystem().step(state)` (the frozen engine's own tick-position order, Vitality @1
      before Lifecycle @7 — the reference stays in TRUE engine order regardless of which order the
      Rust driver runs in, since this script serves as the independent oracle), and ALSO run the two systems in the
      REVERSE order against a second, separately-built copy of the same ten-node state. Print both
      runs' post-tick fields for every node. **This is the step that actually verifies the
      disjoint-domain claim** — the Multi-Rule Decision section asserts it from reading the
      bindings; this script checks it empirically, against the real frozen systems, and the two
      printed outputs must match field-for-field before this task proceeds. If they do not match,
      STOP — the disjoint-domain premise this whole task (and the byte-order-is-safe-here argument)
      rests on would be false, and the plan's design needs to go back to the Multi-Rule Decision
      section rather than paper over the mismatch here.
- [x] **Step 4: Write the failing Rust tests.**

```rust
// tests/multi_rule_conformance.rs
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_tick::run_once_into; // `hex` is not needed here — neither test formats a hash

const SCENARIO: &str =
    include_str!("../content/scenarios/vitality-lifecycle-combined-conformance.bscn");
const VITALITY: &str = include_str!("../content/rules/vitality.bsl");
const LIFECYCLE: &str = include_str!("../content/rules/lifecycle.bsl");

#[test]
fn byte_order_sort_reproduces_the_frozen_engine_despite_running_reversed() {
    // Concatenation order here is arbitrary on purpose (vitality text
    // first) — Step 2 below proves the OTHER concatenation order gives an
    // identical report, which is the actual claim this task makes.
    let rule_src = format!("{VITALITY}\n{LIFECYCLE}");
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, &rule_src, &mut graph, &mut sink).expect("tick");

    // THE ORDER PROOF — ascending rule-id byte order puts lifecycle
    // FIRST ('l' < 'v'), the reverse of the frozen engine's Vitality-@1-
    // before-Lifecycle-@7. Per the Multi-Rule Decision section, the final
    // hash would not, by itself, distinguish "sorts by id" from
    // "preserves file order" for this pair, so per_rule_fired's own
    // order is the load-bearing assertion.
    assert_eq!(report.per_rule_fired.len(), 2);
    assert_eq!(report.per_rule_fired[0].0, "lifecycle/dpd-circuit");
    assert_eq!(report.per_rule_fired[1].0, "vitality/subsistence-and-death");
    // Exact counts pinned from Step 3's printed Python reference —
    // transcribe the real numbers here once the script has run; both
    // vitality-conformance.bscn (5 of 6 subjects pass the guard, per the
    // existing pinned test) and lifecycle-conformance.bscn's own fixture
    // are individually proven, so these counts should match those
    // existing pins exactly, unchanged by union.
    assert_eq!(report.per_rule_fired[0].1, /* lifecycle fired count */ 0);
    assert_eq!(report.per_rule_fired[1].1, /* vitality fired count */ 0);

    // Per-node field values match Step 3's printed Python reference — BOTH
    // halves, transcribed from its printed output. Step 3 proved the two
    // engine-order and reverse-order Python runs agree, so this assertion
    // is legitimate against EITHER printed run.
    // (concrete node_attribute assertions per Step 3's script output)
}

#[test]
fn file_order_is_never_observable_per_section_4_2() {
    // The actual promise §4.2/D16 makes, now a committed test: two content
    // sets built from the SAME two rules in DIFFERENT concatenation order
    // must produce BYTE-IDENTICAL TickReports — not merely the same hash,
    // the same report in full, including per_rule_fired's order (which
    // must be IDENTICAL, not flipped, because the driver sorts rather
    // than preserving file order).
    let forward = format!("{VITALITY}\n{LIFECYCLE}");
    let reversed = format!("{LIFECYCLE}\n{VITALITY}");

    let mut graph_a = HypergraphStore::new();
    let mut sink_a = CollectingSink::default();
    let report_a = run_once_into(SCENARIO, &forward, &mut graph_a, &mut sink_a).expect("tick a");

    let mut graph_b = HypergraphStore::new();
    let mut sink_b = CollectingSink::default();
    let report_b = run_once_into(SCENARIO, &reversed, &mut graph_b, &mut sink_b).expect("tick b");

    assert_eq!(report_a.before, report_b.before);
    assert_eq!(report_a.after, report_b.after);
    assert_eq!(report_a.fired, report_b.fired);
    assert_eq!(
        report_a.per_rule_fired, report_b.per_rule_fired,
        "file/concatenation order must never be observable in the report — §4.2"
    );
}
```

- [x] **Step 5:** Run the Python script, transcribe its printed values into Step 4's placeholder
      assertions and node-attribute checks (never leave a placeholder number in the committed
      test — this step exists precisely to replace them with the real, printed values). `cargo
      test -p babylon-tick --test multi_rule_conformance` → PASS.
- [x] **Step 6:** `mise run rust:check` → green. `mise run qa:regression` → byte-identical (this
      task adds content and a test; it must not move any existing engine byte).
- [x] **Step 7: Commit** (`test(content): multi-rule conformance — byte-order sort reproduces the
      frozen engine, file order proven never observable (B2)`).

### Task 6: `TickSession<G>` — load once, advance many times, now multi-rule

**Files:**

- Create: `rust/crates/babylon-tick/src/session.rs`
- Edit: `rust/crates/babylon-tick/src/lib.rs` (`pub mod session;`, re-export)

**Interfaces:**

- Produces:

```rust
pub struct TickSession<G> { /* private: graph, prepared: PreparedRules, tick */ }

impl<G: GraphSubstrate + CanonicalState> TickSession<G> {
    pub fn new(scenario_src: &str, rule_src: &str, graph: G) -> Result<Self, String>;
    pub fn advance(&mut self, sink: &mut CollectingSink) -> Result<TickReport, String>;
    pub fn tick(&self) -> i64;
    pub fn graph(&self) -> &G;
}
```

  Same four-method public surface as the first cut of this plan — the multi-rule widening is
  entirely internal (`PreparedRule` → `PreparedRules`, one `run_tick` call → a loop). Every later
  task in this plan (Phase C's lens producers, Phase D's UI) reads state through `session.graph()`
  and drives the loop through `session.advance(&mut sink)`, unchanged. This is the ONE
  `babylon-tick` API addition this plan makes on top of the language-crate widening — `run_once`,
  `run_once_into` keep their exact current signatures; `TickReport` gains the additive
  `per_rule_fired` field (Task 4), which every reader of `.fired` ignores without needing to change.

- [x] **Step 1: Write the failing tests.**

```rust
use crate::session::TickSession;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;

const SCENARIO: &str =
    include_str!("../content/scenarios/vitality-lifecycle-combined-conformance.bscn");
const VITALITY: &str = include_str!("../content/rules/vitality.bsl");
const LIFECYCLE: &str = include_str!("../content/rules/lifecycle.bsl");

fn rule_src() -> String {
    format!("{VITALITY}\n{LIFECYCLE}")
}

#[test]
fn advance_numbers_ticks_starting_at_one_over_a_two_rule_session() {
    let mut session =
        TickSession::new(SCENARIO, &rule_src(), HypergraphStore::new()).expect("load");
    assert_eq!(session.tick(), 0);
    let mut sink = CollectingSink::default();
    let r1 = session.advance(&mut sink).expect("tick 1");
    assert_eq!(session.tick(), 1);
    assert_eq!(r1.per_rule_fired.len(), 2);
    session.advance(&mut sink).expect("tick 2");
    assert_eq!(session.tick(), 2);
}

#[test]
fn advance_moves_state_and_each_tick_hashes_differently() {
    let mut session =
        TickSession::new(SCENARIO, &rule_src(), HypergraphStore::new()).expect("load");
    let mut sink = CollectingSink::default();
    let t1 = session.advance(&mut sink).expect("tick 1");
    let t2 = session.advance(&mut sink).expect("tick 2");
    assert_ne!(t1.before, t1.after, "tick 1 must move state");
    assert_eq!(t1.after, t2.before, "tick 2 starts where tick 1 left off");
    assert_ne!(t2.before, t2.after, "tick 2 must move state too — not a one-shot");
}

#[test]
fn two_independent_sessions_over_the_same_content_hash_identically() {
    // The determinism guard this plan's own instructions require, at the
    // babylon-tick level — Phase E's test (tests/determinism.rs in
    // babylon-client) repeats this same property through the client's own
    // seam end to end.
    let mut a = TickSession::new(SCENARIO, &rule_src(), HypergraphStore::new()).expect("load a");
    let mut b = TickSession::new(SCENARIO, &rule_src(), HypergraphStore::new()).expect("load b");
    let mut sink_a = CollectingSink::default();
    let mut sink_b = CollectingSink::default();
    for _ in 0..5 {
        let ra = a.advance(&mut sink_a).expect("a advances");
        let rb = b.advance(&mut sink_b).expect("b advances");
        assert_eq!(ra.after, rb.after, "same content + same tick count must hash identically");
    }
}
```

- [x] **Step 2:** `cargo test -p babylon-tick` → FAIL (`session` module does not exist).
- [x] **Step 3: Write `session.rs`.**

```rust
//! `TickSession` — the persistent load-once, advance-many seam B2 needs,
//! now multi-rule (Phase A, Tasks 2-4). `run_once`/`run_once_into`
//! (`lib.rs`) model one tick end to end and hardcode `run_tick`'s tick
//! argument to `1` for every rule the content set holds; a player-driven
//! loop needs the split this type provides instead: parse and load cost
//! paid ONCE in `new`, the SAME `PreparedRules` and the SAME graph reused
//! by every `advance()` call, every rule in the content set run once per
//! call, in ascending rule-id byte order (§4.2, register row D16 —
//! `prepare_rules` sorts once at load time), with `tick` incremented by
//! this type.

use crate::{prepare_rules, PreparedRules, TickReport};
use babylon_bsl::intrinsic_host::KernelIntrinsicHost;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_bsl::tick::run_tick;
use babylon_graph::state_hash::CanonicalState;
use babylon_graph::substrate::GraphSubstrate;

/// One content set, loaded once, advanced tick by tick against ONE held
/// graph. `G` is caller-supplied (same shape as `run_once_into`) so the
/// caller picks the substrate — production callers pass `HypergraphStore`
/// (ADR193).
pub struct TickSession<G> {
    graph: G,
    prepared: PreparedRules,
    tick: i64,
}

impl<G: GraphSubstrate + CanonicalState> TickSession<G> {
    /// Parse `rule_src` (one or more `(rule …)` forms) and load
    /// `scenario_src` into `graph` once. `prepare_rules` sorts the forms
    /// into ascending rule-id byte order (§4.2, D16) before this returns —
    /// the caller's own concatenation order is never observable.
    ///
    /// # Errors
    /// The same failure modes `run_once_into`'s load half has: an
    /// intrinsic declaration, a scenario load, or a rule load — named to
    /// its own rule id when more than one rule is present.
    pub fn new(scenario_src: &str, rule_src: &str, mut graph: G) -> Result<Self, String> {
        let prepared = prepare_rules(scenario_src, rule_src, &mut graph)?;
        Ok(Self {
            graph,
            prepared,
            tick: 0,
        })
    }

    /// Run one more tick against the held graph: every rule in the
    /// content set, in ASCENDING RULE-ID BYTE ORDER (§4.2, D16 — sorted
    /// once, at load time, by `prepare_rules`), each to completion before
    /// the next starts, against the SAME graph — so a later rule sees an
    /// earlier rule's writes from this same tick (the frozen engine's own
    /// in-place strict-order semantics, inherited for free from calling
    /// `run_tick` sequentially against one `&mut G`). The first call runs
    /// tick 1 (matching `run_once`'s own numbering), the second tick 2,
    /// and so on.
    ///
    /// # Errors
    /// The tick itself (named to its own rule id), or a pre/post
    /// state-hash failure.
    pub fn advance(&mut self, sink: &mut CollectingSink) -> Result<TickReport, String> {
        self.tick += 1;
        let before = self
            .graph
            .state_hash()
            .map_err(|e| format!("pre-tick state: {}", e.message))?;
        let mut per_rule_fired = Vec::with_capacity(self.prepared.rules.len());
        for (id, loaded) in &self.prepared.rules {
            let outcome = run_tick(
                loaded,
                &self.prepared.types,
                &KernelIntrinsicHost,
                &mut self.graph,
                sink,
                &self.prepared.intrinsics,
                &self.prepared.consts,
                self.tick,
            )
            .map_err(|e| format!("tick failed in rule {id}: {e}"))?;
            per_rule_fired.push((id.clone(), outcome.fired));
        }
        let fired = per_rule_fired.iter().map(|(_, n)| n).sum();
        let after = self
            .graph
            .state_hash()
            .map_err(|e| format!("post-tick state: {}", e.message))?;
        Ok(TickReport {
            before,
            after,
            fired,
            per_rule_fired,
        })
    }

    /// The current tick number — 0 before the first `advance()` call.
    #[must_use]
    pub fn tick(&self) -> i64 {
        self.tick
    }

    /// Read-only access to the held graph — the client's map lens and
    /// state panel project live state through this.
    #[must_use]
    pub fn graph(&self) -> &G {
        &self.graph
    }
}
```

      `lib.rs` needs `PreparedRules`/`prepare_rules` visible to `session.rs` (same crate,
      `pub(crate)` already covers this) plus `pub mod session;` and, for the client's convenience,
      `pub use session::TickSession;` alongside the existing `pub use` of `TickReport`.
- [x] **Step 4:** `cargo test -p babylon-tick` → PASS (all three tests above, plus Task 4's
      regression tests and Task 5's conformance tests still green). `mise run rust:check` → green.
- [x] **Step 5: Commit** (`feat(rust): TickSession — persistent load-once/advance-many multi-rule
      tick loop seam (B2)`).

---

## Phase B — The demo content

### Task 7: The eighteen-subject demo world — twelve real-FIPS counties, six social classes

**Files:**

- Create (temporary aid, deleted at Step 4): `rust/crates/babylon-client/tests/print_demo_counties.rs`
- Create: `rust/crates/babylon-tick/content/scenarios/us-counties-lifecycle-demo.bscn`

**The point of this task:** `lifecycle-conformance.bscn`'s four territory nodes
(`core-county`/`growing-county`/`recovering-county`/`young-county`) carry proven-correct fixture
values but carry synthetic local names with no FIPS code, so `CountyAtlas::index_of_fips` cannot
place them on the B1 map. `vitality-conformance.bscn`'s six social-class nodes have no territorial
binding at all — by construction (Multi-Rule Decision section), vitality's contribution to this
demo is INVISIBLE on the map surface itself; it shows up only in the event feed
(`ENTITY_DEATH`-family events) and, if a future task adds a class-scoped panel, in state readouts —
stated here plainly rather than implied, since "richer demo" should not quietly promise a
map-visible effect vitality's own subject type cannot honestly produce. This task re-stamps the
lifecycle archetype value sets onto twelve REAL FIPS codes AND mints the vitality fixture verbatim
alongside them, in ONE scenario, without inventing new numbers this plan's author cannot verify.

**BLOCKER 2 correction — the twelve counties now get TWELVE DISTINCT population seeds, not four
archetypes repeated three times each.** The first cut of this task cycled the four
`lifecycle-conformance.bscn` archetypes verbatim across the twelve FIPS (indices 0-2 all getting
`core-county`'s exact numbers, and so on) — legal, but it means three of the twelve counties would
render and evolve IDENTICALLY, which understates "watch state change" more than necessary and
was never required by anything this task cites. Each of the twelve now takes its archetype's
`pop-d`/`pop-p`/`pop-d-prime` values SCALED by a disclosed, deterministic factor — `{0.95, 1.00,
1.05}` across the three repeats of each archetype, each result rounded to the nearest integer
(the `.bscn` loader accepts only integer literals into `int`-declared fields — verified against
`dispossession.bsl`'s own header note on this exact constraint) — so every county starts from a
distinct, real number while staying visibly the same FAMILY as its archetype (a 5% swing changes
the DPD trajectory's size, never its sign or its rough shape). `wealth-d-prime`,
`dependency-ratio` (a computed field, always seeded `0`) and the legitimation fields stay at their
archetype's exact original values — the initial `legitimation-crisis` seed matters
only for the tick-1 crisis/recovery EDGE detection (Task 5's own `prev-crisis` mechanism), which a
population perturbation has no bearing on.

- [x] **Step 1: Select the twelve FIPS, deterministically, from the committed atlas — never
      guessed.** B1 Task 1 Step 5 sorts the atlas's county table by FIPS ascending before writing,
      so atlas indices `0..12` are the twelve lowest-FIPS counties in the whole 3,222-county
      artifact, whatever they are. Print them:

```rust
// tests/print_demo_counties.rs — a one-shot reporting aid, not a permanent
// test. Delete this file in Step 4 once its output is transcribed below.
use babylon_client::atlas::CountyAtlas;

const ATLAS_BYTES: &[u8] = include_bytes!("../assets/map/county_atlas.bin");

#[test]
#[ignore = "one-shot reporting aid — run manually, transcribe, then this file is deleted"]
fn print_first_twelve() {
    let atlas = CountyAtlas::parse(ATLAS_BYTES).expect("committed atlas parses");
    for i in 0..12 {
        let c = atlas.county(i).expect("index within range");
        println!("{i}: fips={} name={}", c.fips, c.name);
    }
}
```

      Run: `cargo test -p babylon-client --test print_demo_counties -- --ignored --nocapture`.
      Record the twelve printed `(fips, name)` pairs in this task's commit body verbatim — this is
      the only place in this plan a FIPS code is fixed, and it is fixed by running code against the
      committed artifact, not by recall.
- [x] **Step 2: Write the scenario.** ONE `.bscn` file, two node-type halves:
      - **Territory half.** Reuse the `lifecycle-conformance.bscn` header's `deffield` block and
        all 21 `defconst` rows byte-for-byte. Apply the `{0.95, 1.00, 1.05}` scale factors (rounded
        to nearest integer) to each archetype's `pop-d`/`pop-p`/`pop-d-prime` across its three
        repeats — the exact twelve values, computed and verified for this plan:

        | idx | archetype | pop-d | pop-p | pop-d-prime | wealth-d-prime |
        |---|---|---|---|---|---|
        | 0 | core (×0.95) | 2042† | 5748† | 1710 | 10000000 |
        | 1 | core (×1.00) | 2150 | 6050 | 1800 | 10000000 |
        | 2 | core (×1.05) | 2258† | 6352† | 1890 | 10000000 |
        | 3 | growing (×0.95) | 2850 | 4750 | 1425 | 5000000 |
        | 4 | growing (×1.00) | 3000 | 5000 | 1500 | 5000000 |
        | 5 | growing (×1.05) | 3150 | 5250 | 1575 | 5000000 |
        | 6 | recovering (×0.95) | 1900 | 6650 | 1900 | 20000000 |
        | 7 | recovering (×1.00) | 2000 | 7000 | 2000 | 20000000 |
        | 8 | recovering (×1.05) | 2100 | 7350 | 2100 | 20000000 |
        | 9 | young (×0.95) | 3800 | 5225 | 0 | 0 |
        | 10 | young (×1.00) | 4000 | 5500 | 0 | 0 |
        | 11 | young (×1.05) | 4200 | 5775 | 0 | 0 |

        † Four values land on an exact `.5` tie before rounding (`2150×0.95 = 2042.5`,
        `6050×0.95 = 5747.5`, `2150×1.05 = 2257.5`, `6050×1.05 = 6352.5`) — resolved by
        round-half-to-even (banker's rounding, the tool that generated this table used Python 3's
        default `round()`), not round-half-up. Noted explicitly so `2042`/`5748`/`2258`/`6352`
        read as the deliberate, reproducible result of a named rounding rule rather than as
        transcription slips — a different rounding convention would legitimately land one tick off
        `2043`/`5747`/`2257`/`6353` and would still be a correct implementation of THIS table,
        provided the task states which convention it used, as this one now does.

        `dependency-ratio` seeds `0` (computed field) for all twelve; `legitimation-index` seeds `0`
        for all twelve; `legitimation-crisis`/`transmitted-ideology` seed at their ARCHETYPE's
        original value, transcribed exactly from `lifecycle-conformance.bscn` (re-verified against
        the committed file, not recalled): core = `0`/`0` (STABLE), growing = `1`/`0` (UNSTABLE —
        corrected this revision; an earlier draft of this table wrote `0`, which is wrong),
        recovering = `2`/`0` (CRISIS, so it still fires `LEGITIMATION_RECOVERY` on tick 1 for
        indices 6-8 exactly as the unperturbed design did), young = `0`/`0` (STABLE) — the
        population scale factor has no bearing on any of these fields. Name each node `county-<fips>`
        (symbols must start with a lowercase letter — §1's `symbol ::= LOWER (LOWER | DIGIT |
        "-")*` — a bare FIPS like `06037` is not a legal symbol, `county-06037` is).
      - **Social-class half.** Reuse `vitality-conformance.bscn`'s `deffield` block (7 fields,
        `social-class/inequality` included though no rule reads it, for fixture parity with the
        frozen engine per that file's own comment) and its 2 `defconst` rows, and its six nodes'
        values, byte-for-byte, keeping the SAME local names (`core`, `bourgeoisie`, `hermit`,
        `last-worker`, `remnant`, `dissolved` — the Multi-Rule Decision section's collision check
        already confirms these never collide with `county-<fips>`).

```text
(scenario lifecycle/us-counties-demo
  ; --- territory half (lifecycle) ---
  (deffield territory/pop-d int extensive)
  (deffield territory/pop-p int extensive)
  (deffield territory/pop-d-prime int extensive)
  (deffield territory/wealth-d-prime int extensive)
  (deffield territory/dependency-ratio int intensive)
  (deffield territory/legitimation-index int intensive)
  (deffield territory/legitimation-crisis int intensive)
  (deffield territory/transmitted-ideology int intensive)

  ; --- social-class half (vitality) ---
  (deffield social-class/active int extensive)
  (deffield social-class/population int extensive)
  (deffield social-class/wealth int extensive)
  (deffield social-class/subsistence-multiplier int intensive)
  (deffield social-class/s-bio int intensive)
  (deffield social-class/s-class int intensive)
  (deffield social-class/inequality int intensive)

  ; ... all 21 lifecycle/* defconst rows, transcribed verbatim from
  ; lifecycle-conformance.bscn:56-76, same values, same :NNN citations ...
  ; ... both economy/* defconst rows, transcribed verbatim from
  ; vitality-conformance.bscn:43-44 ...

  ; county-<fips[0]>, county-<fips[1]>, county-<fips[2]>: the core-county
  ; archetype (DPDState docstring numbers, PRE-crisis STABLE), scaled
  ; 0.95/1.00/1.05 per Step 2's table so all three are DISTINCT.
  (node county-<fips[0]> NodeType/TERRITORY
    (territory/pop-d 2042) (territory/pop-p 5748) (territory/pop-d-prime 1710)
    (territory/wealth-d-prime 10000000) (territory/dependency-ratio 0)
    (territory/legitimation-index 0) (territory/legitimation-crisis 0)
    (territory/transmitted-ideology 0))
  (node county-<fips[1]> NodeType/TERRITORY
    (territory/pop-d 2150) (territory/pop-p 6050) (territory/pop-d-prime 1800)
    (territory/wealth-d-prime 10000000) (territory/dependency-ratio 0)
    (territory/legitimation-index 0) (territory/legitimation-crisis 0)
    (territory/transmitted-ideology 0))
  ; ... fips[2] (×1.05: 2258/6352/1890) same pattern; the committed file
  ; writes all twelve nodes in full, Step 2's table above is the source.

  ; county-<fips[3..6]>: the growing-county archetype (PRE-crisis UNSTABLE),
  ;   scaled 0.95/1.00/1.05.
  ; county-<fips[6..9]>: the recovering-county archetype (PRE-crisis CRISIS,
  ;   fires LEGITIMATION_RECOVERY on tick 1 under these defconsts, same as
  ;   the conformance scenario, unaffected by the population scale), scaled
  ;   0.95/1.00/1.05.
  ; county-<fips[9..12]>: the young-county archetype (no D' cohort), scaled
  ;   0.95/1.00/1.05.

  ; The six vitality-conformance.bscn nodes, values transcribed verbatim
  ; (fed core, standard-of-living-scaled bourgeoisie, surviving hermit,
  ; starving last-worker, zombie-failsafe remnant, already-dissolved).
  (node core NodeType/SOCIAL_CLASS
    (social-class/active 1) (social-class/population 100) (social-class/wealth 1000)
    (social-class/subsistence-multiplier 1) (social-class/s-bio 1) (social-class/s-class 1)
    (social-class/inequality 0))
  ; ... bourgeoisie, hermit, last-worker, remnant, dissolved, same values ...
  )
```

      Write out all eighteen `(node …)` forms in full — no ellipsis in the committed file, the
      ellipses above are this plan document's abbreviation only.
- [x] **Step 3: A loading test.**

```rust
// rust/crates/babylon-tick/tests/us_counties_demo.rs (new file)
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_tick::TickSession;

const SCENARIO: &str = include_str!("../content/scenarios/us-counties-lifecycle-demo.bscn");
const VITALITY: &str = include_str!("../content/rules/vitality.bsl");
const LIFECYCLE: &str = include_str!("../content/rules/lifecycle.bsl");

#[test]
fn the_demo_scenario_loads_and_ticks_both_packs() {
    let rule_src = format!("{VITALITY}\n{LIFECYCLE}");
    let mut session =
        TickSession::new(SCENARIO, &rule_src, HypergraphStore::new()).expect("load");
    let mut sink = CollectingSink::default();
    let report = session.advance(&mut sink).expect("tick 1");
    assert_ne!(report.before, report.after);
    assert_eq!(report.per_rule_fired.len(), 2);
    // Ascending rule-id byte order (§4.2, D16) — lifecycle before vitality,
    // regardless of the rule_src concatenation order above.
    assert_eq!(report.per_rule_fired[0].0, "lifecycle/dpd-circuit");
    assert_eq!(report.per_rule_fired[1].0, "vitality/subsistence-and-death");
    // The recovering-county archetype fires LEGITIMATION_RECOVERY on tick 1
    // under these defconsts (matching lifecycle-conformance.bscn's own
    // documented behavior) — proves the twelve territory nodes really run
    // lifecycle, not just mint successfully.
    assert!(sink
        .events
        .iter()
        .any(|(name, _)| name == "EventType/LEGITIMATION_RECOVERY"));
}
```

      `cargo test -p babylon-tick --test us_counties_demo` → PASS.
- [x] **Step 4:** Delete `tests/print_demo_counties.rs` — it has finished its job, and its own doc
      comment says so; a stale `#[ignore]`d test that prints fixed array indices against a file
      that could later change underneath is exactly the kind of orphan CLAUDE.md's Surgical
      Changes rule asks an author to clean up when a task's own steps create one.
- [x] **Step 5: Commit** (`feat(content): the eighteen-subject B2 demo world — twelve real-FIPS
      counties + six social classes`), body carrying the Step 1 FIPS/name table.

---

## Phase C — The three-lens map (completes B1 Phase C)

### Task 8: The Tension lens, ported and corrected for `HypergraphStore`

**Files:**

- Create: `rust/crates/babylon-client/src/lens.rs`
- Edit: `rust/crates/babylon-client/src/lib.rs` (`pub mod lens;`)

**Interfaces:**

- Produces:

```rust
pub struct LensReading {
    pub cells: Vec<(String, Option<f64>)>, // (fips, value) — shared shape all three lenses return
    pub absent_reason: Option<String>,
}

/// The three live `LensReading`s the map can show, refreshed together on
/// every tick advance (MEDIUM-HIGH fix — this struct was USED across five
/// call sites in the first cut but never actually DEFINED anywhere; in the
/// real app `Res<CurrentLensData>` would have failed to resolve and Bevy
/// 0.18 would have skipped `recolor_on_lens_changed` with a warn-once,
/// leaving the map permanently uncolored while every test that
/// hand-installed the resource kept passing). Declared here, alongside
/// `LensReading` — `map/bands.rs` (Task 10) reads it via `use
/// crate::lens::CurrentLensData`, never redeclares it. No `Default` derive
/// — `spawn_engine_session_and_hud` (Task 14) is the ONLY inserter, always
/// with a fully-computed literal, and it runs in `Startup` (ordered after
/// `MapPlugin`'s own Startup systems) strictly before `recolor_on_lens_
/// changed` (an `Update` system) can ever run, so no earlier reader can
/// observe a missing or default-empty resource.
#[derive(Resource)]
pub struct CurrentLensData {
    pub tension: LensReading,
    pub legitimation: LensReading,
    pub population_trend: LensReading,
}

pub fn county_tension(graph: &dyn GraphSubstrate) -> LensReading;
```

- [x] **Step 1: Write the failing tests**, hand-building small `HypergraphStore`s (not
      `MemoryGraph` — the Sequencing Decision's correction applies here first): (a) two territories
      with clean stamps where `theta` (computed internally — `LensReading` carries only `w` per
      cell) differs from the mean of the two `phi`s; (b) a bled county scores
      `w < 0`, a bribed county `w > 0`; (c) a territory with `s > 0, e == 0` contributes nothing and
      reports `None`; (d) a graph with zero data-bearing territory nodes yields
      `absent_reason.is_some()` and every cell `None`; (e) every returned `w` lands in `[-1, 1]`.
- [x] **Step 2:** FAIL, then write it — the ADR170 formula transcribed exactly as B1's Task 8
      specified (`phi = v/(v+s)`, `theta = sum(v)/sum(v+s)`, `w = (phi-theta)/(phi+theta)`,
      `phi+theta <= 1e-9` collapses to `0.0`), reading `graph.nodes("TERRITORY")` — the BARE enum
      member string, never `"NodeType/TERRITORY"`: the substrate stores and matches the verb
      layer's own stamped form (`namespace_to_node_type`'s doc comment, `tick.rs`: *"the verb layer
      stamps the enum member verbatim (`(add-node NodeType/SOCIAL_CLASS …)` → `"SOCIAL_CLASS"`)"*
      — confirmed against every live call site in the crate: `structural_verbs.rs:1046`,
      `scenario.rs:776`, `fundamental_theorem_tick.rs:390`, all bare, none prefixed. `NodeType/…`
      is BSL SOURCE syntax (the `<enum-ref>` you write inside a `.bscn`/`.bsl` file); a Rust string
      literal handed to `nodes()` is a RUNTIME key and must match what the substrate actually
      stores — and `graph.node_attribute(id, "...")` through `&dyn GraphSubstrate` rather than a concrete store
      — note this graph, from Task 7 on, ALSO holds six `"SOCIAL_CLASS"` nodes; `nodes()`'s
      own type filter (verified in `memory.rs`/`hypergraph_store.rs`) already excludes them, so
      this task's logic needs no change — confirmed by reading, recorded here rather than assumed.
- [x] **Step 3:** `cargo test -p babylon-client` → PASS.
- [x] **Step 4: Commit** (`feat(client): the ADR170 tension witness over &dyn GraphSubstrate (B2)`).

**Related finding, checked while fixing BLOCKER 2, stated here for the same honesty reason.** Task
7's demo scenario declares only `lifecycle`'s territory fields (`pop-d`/`pop-p`/`pop-d-prime`/
`wealth-d-prime`/`dependency-ratio`/`legitimation-*`/`transmitted-ideology`) — it seeds NO `v`/`s`/
`e`-style economic stamps anywhere, because nothing in this plan's two rule packs writes them, so
`county_tension` returns `absent_reason.is_some()` and every cell `None` on THIS demo,
unconditionally — not merely static like Legitimation, genuinely EMPTY. This is not a new defect:
B1's own Task 12 already named the same fact ("the ADR170 lens is scenario-baked and tick-invariant
under currently-ported content") and built its own declared-fallback path for exactly this case
(the absence banner Task 11 renders). The consequence for THIS plan: **the app must not default
to `ActiveLens::Tension`** — a player's first launch would show the absence banner over an
all-`PANEL` map, which is a worse first impression than any of this plan's other findings and an
easy one to avoid. Task 12 defaults to `ActiveLens::PopulationTrend` instead — see its own note.

### Task 9: The Legitimation lens — live, categorical, zero new math

**Files:**

- Edit: `rust/crates/babylon-client/src/lens.rs`

**Interfaces:**

- Produces: `pub fn county_legitimation(graph: &dyn GraphSubstrate, node_by_fips: &[(String,
  NodeId)]) -> LensReading` and `pub enum LegitimationClass { Stable, Unstable, Crisis }` with
  `pub fn classify(raw: f64) -> LegitimationClass`.

**Why this reads a field instead of deriving one.** The `lifecycle` rule pack already computes and
writes `territory/legitimation-crisis` as an encoded classification (0 = STABLE, 1 = UNSTABLE,
2 = CRISIS — the rule pack's own header comment documents the encoding, quoted in
`lifecycle-conformance.bscn`'s comments). Coloring the map from THIS field, rather than re-deriving
a threshold on the raw `territory/legitimation-index`, adds no new cut point and no new math — this
is a straight categorical pass-through, consistent with the standing "no imposed functional forms"
ruling. **This module produces the raw classification number only** — Task 10's `bands.rs` owns
the color mapping (Director ruling 1), matching the Tension lens's own separation of "compute the
value" from "pick the color."

- [x] **Step 1: Write the failing tests.** A territory whose `legitimation-crisis` reads back
      `0.0`/`1.0`/`2.0` classifies to `Stable`/`Unstable`/`Crisis` respectively; a `node_by_fips`
      entry naming a `NodeId` the graph never minted (a coding error, not a real absence — the
      Phase B scenario controls the whole node set) surfaces as an `Err`, never a silent `None`,
      because unlike Tension's "this county may honestly have no data," a demo-scenario FIPS with
      no matching node is a wiring bug; only FIPS NOT in `node_by_fips` at all are the honest
      "outside the demo, no data this tick" absence.

      **Resolution (adversarial-panel finding FB7, recorded rather than left implicit).** This
      step's own text says the wiring-bug case "surfaces as an `Err`," but the Interfaces block
      three lines up commits `county_legitimation` to a bare `LensReading` return, not
      `Result<LensReading, _>` — a genuine self-contradiction in this plan's own text. The
      implementation resolved it by PANICKING loudly (`.unwrap_or_else(|e| panic!(...))`,
      `lens.rs`) rather than widening the signature to `Result` — defensible under III.11 (Loud
      Failure: a panic is at least as loud as a propagated `Err`, and simpler, since nothing
      upstream of a Bevy `Update` system has anywhere to route a `Result` to), but a design choice
      this plan never explicitly ruled on, so it is recorded here rather than silently taken as
      "what Err must have meant." The SAME panic-on-wiring-bug shape was then reused for Task 9b's
      `county_population_trend` (a fips missing from `baseline`) and for `classify`'s own
      out-of-encoding guard — meaning `refresh_hud` and `refresh_state_panel` (both real `Update`
      systems that run every rendered frame, not merely at load time) now sit downstream of THREE
      implementation-level panic sites (`classify`'s own out-of-encoding match arm,
      `county_legitimation`'s wiring-bug panic, `county_population_trend`'s wiring-bug panic)
      reached through (at least) two separate call sites (`map/hud.rs`'s and `loop_ui.rs`'s own
      `classify`/lens calls) — a real but bounded surface (every one of them requires a
      programming error in `node_by_fips`/`baseline` construction or a `lifecycle.bsl` encoding
      change, never a normal runtime/data condition), not evidence of a design flaw.
- [x] **Step 2:** FAIL, then write it: `classify` is a plain three-arm match on the encoded
      float (`0.0 => Stable`, `1.0 => Unstable`, `2.0 => Crisis`, anything else a loud panic — the
      encoding is a closed set the rule pack itself defines); `county_legitimation` reads
      `territory/legitimation-crisis` for every `(fips, id)` pair in `node_by_fips` and returns
      `Some(raw_class_as_f64)` per cell.
- [x] **Step 3:** `cargo test -p babylon-client` → PASS.
- [x] **Step 4: Commit** (`feat(client): the legitimation lens — live per-tick classification, zero
      new thresholds (B2)`).

**BLOCKER 2 finding, stated here rather than left implied.** This lens's own code is correct and
ships exactly as Director ruling 1 specified. What it displays is a different matter: reading
`lifecycle.bsl`'s Block 2 in full (`legit-index`'s five weighted terms — `home-ownership-rate`,
`healthcare-security`, `retirement-confidence`, `pension-coverage-rate`, `ss-replacement-rate`,
each a `:const`, none a `:field`) shows `legit-index` computes to the SAME value for every
territory, every tick — there is no per-territory or per-tick input anywhere in its formula.
Under the shipped `defines.yaml` values that number is `0.6039`, above
`legitimation-unstable-threshold` (`0.5`), so `new-crisis-class` evaluates to `0` (STABLE) for
every territory from tick 2 onward (tick 1 still fires the recovering-archetype counties'
`LEGITIMATION_RECOVERY` edge, since `prev-crisis` reads the SEEDED value before the rule
overwrites it). Under ruling 1's color mapping (Task 10), STABLE renders `PANEL` — the same color
as no data. **This lens is real, ships as ruled, and will animate the moment `lifecycle.bsl`'s
inputs become per-territory fields (the typed-attribute Phase-2 revision) — but until then, a
player watching ONLY this lens sees an unchanging dark map from tick 2 on.** Task 9b (new
Population Trend lens, below) exists because "watch state change" needs a lens that moves TODAY,
not a promise about a future port.

### Task 9b: The Population Trend lens — the lens that actually moves (BLOCKER 2 fix)

**Files:**

- Edit: `rust/crates/babylon-client/src/lens.rs`

**Interfaces:**

- Produces: `pub fn county_population_trend(graph: &dyn GraphSubstrate, node_by_fips: &[(String,
  NodeId)], baseline: &[(String, f64)]) -> LensReading` — `baseline` is the tick-0 total
  population per demo county, captured once by `EngineSession::start` (Task 13) before any
  `advance()` runs, in the SAME `(fips, value)` shape `node_by_fips` already establishes.

**Why this is the lens that carries "watch state change."** `lifecycle.bsl`'s Block 1 (the DPD
population flow) reads and writes GENUINE per-tick, per-territory `:field`s —
`territory/{pop-d,pop-p,pop-d-prime}` — unlike Block 2's const-only legitimation math (Task 9's
finding). Verified by computing the actual five-tick trajectory from the pack's own formulas
(`births = birth-rate * pop-p`, `d-to-p = rate-d-to-p * pop-d`, `p-to-d-prime = rate-p-to-d-prime *
pop-p`, `deaths = rate-d-prime-to-death * pop-d-prime`, then the three `new-pop-*` updates) against
Task 7's twelve seeded county values:

| county family | total pop, tick 0 | total pop, tick 5 | Δ over 5 ticks |
|---|---|---|---|
| core (×1.00 repeat) | 10,000 | 9,949.5 | −50.5 (−0.51%) |
| growing (×1.00 repeat) | 9,500 | 9,462.2 | −37.8 (−0.40%) |
| recovering (×1.00 repeat) | 11,000 | 10,954.0 | −46.0 (−0.42%) |
| young (×1.00 repeat) | 9,500 | 9,759.6 | **+259.6 (+2.73%)** |

Three of the four archetype families NET-DECLINE (death drain from an already-large D'
cohort outpaces the birth/transition inflow over a short horizon); `young`'s family — seeded with
ZERO D' population — has no death term to speak of yet and NET-GROWS. This is not a coincidence
engineered for the demo: it falls straight out of the pack's own formulas, verified by direct
computation, and it happens to be thematically apt (an aging, established county's population
slowly contracts; a young one grows) without this plan asserting anything about that beyond what
the arithmetic gives. The `±5%` per-county scale factors (Task 7) shift each county's own
size without flipping any family's sign — every core/growing/recovering-family county
declines, every young-family county grows, twelve genuinely distinct trajectories, four groups by
sign-and-shape, not twelve unrelated numbers and not three silent repeats.

- [x] **Step 1: Write the failing tests.** A territory whose current `pop-d + pop-p + pop-d-prime`
      exceeds its `baseline` entry reports a POSITIVE value; below baseline reports NEGATIVE;
      exactly equal (an edge case no real tick reaches, included for totality since `baseline` and
      "now" could coincide before the first `advance()`) reports exactly `0.0`; a FIPS present in
      `node_by_fips` but ABSENT from `baseline` is a wiring bug (both come from the same
      `EngineSession::start` call, Task 13) and surfaces as a loud panic, never a silent `None` —
      the same strictness argument Task 9 makes for its own `node_by_fips` mismatch case; a FIPS
      outside BOTH slices (any of the 3,210 non-demo counties) is the one legitimate `None`.
- [x] **Step 2:** FAIL, then write it: for each `(fips, id)` in `node_by_fips`, read
      `territory/pop-d`, `territory/pop-p`, `territory/pop-d-prime` off `graph` and sum them; look
      up `fips` in `baseline` (linear scan — twelve entries, the same "not worth a `HashMap` at
      this size" call Task 15 already makes for `node_by_fips`); return `Some(now - baseline)` as
      the cell's raw value. **No sign normalization, no size threshold, no clamping** — the
      raw signed delta travels to `map/bands.rs` (Task 10), which does the `> 0.0` / `< 0.0` / `==
      0.0` classification; this module states a number, it does not classify one, matching Task 8
      and Task 9's own division of labor.
- [x] **Step 3:** `cargo test -p babylon-client` → PASS.
- [x] **Step 4: Commit** (`feat(client): the population trend lens — genuinely per-tick, per-county
      state change (B2, BLOCKER 2 fix)`).

### Task 10: Three band tables, one recolor system

**Files:**

- Edit: `rust/crates/babylon-client/src/map/bands.rs`

**Interfaces:**

- Produces: `pub fn tension_band_color(w: Option<f64>) -> Color` (ADR191 R11's table, exactly as
  the B1 plan's Task 9 specified — four rows, `<=` resolution, `PANEL` for absence); `pub fn
  legitimation_band_color(class: Option<f64>) -> Color` (per this amendment's Director ruling 1:
  three rows, `Some(0.0) => PANEL`, `Some(1.0) => DIM`, `Some(2.0) => CRIMSON`, `None => PANEL`);
  `pub fn population_trend_band_color(delta: Option<f64>) -> Color` (NEW this revision, BLOCKER 2
  fix — a strict sign comparison, no invented size threshold: `Some(d) if d > 0.0 => GOLD`
  (growing), `Some(d) if d < 0.0 => CRIMSON` (declining), `Some(0.0) => DIM` (unchanged — no real
  demo tick reaches this, included for totality), `None => PANEL`); plus `#[derive(Resource, Debug,
  Clone, Copy, PartialEq, Eq)] pub enum ActiveLens { Tension, Legitimation, PopulationTrend }`
  (THREE variants, was two — `Debug`/`PartialEq` because Task 12's and Task 18's tests both compare
  and print it directly, `Copy` because every reader takes it by value off a `Res<ActiveLens>`) and
  `#[derive(Event)] pub struct LensChanged;`.

**Director ruling 1, applied exactly.** Selected option, quoted: *"CRISIS → crimson, UNSTABLE →
dim gray, STABLE → gold's absence (panel dark) — reuses the ruled four-band vocabulary so the two
lenses share one visual language. No new colors enter the game."* This REPLACES the first cut's
design (which had invented `GREEN_DARK`/`GOLD` mappings for STABLE/UNSTABLE — neither ships).
**STABLE and "no data this tick" render the SAME color, `PANEL`, on purpose** — a deliberate
INVERSION of the Tension lens's own rule (ADR191 R11, carried unmodified in `tension_band_color`):
"nothing may confuse absence with the neutral band" there means Tension's neutral band is `DIM`,
distinct from `PANEL`. Here, the Director's own ruling makes STABLE deliberately indistinguishable
from absence BY COLOR — "gold's absence" reads as "nothing here needs your attention," which is
thematically apt for a stable county fading into the background. The HUD (Task 11) is, as a
result, the ONLY channel that can tell a player "this county is STABLE" from "this county has no data" —
its literal text label carries weight the color alone cannot, and Task 11's steps make that
explicit. **The new Population Trend lens (Task 9b) is additive to this ruling, not a variance
from it** — it reuses the SAME four `map/bands.rs` tokens (`GOLD` and `DIM` newly put to use here,
`CRIMSON`/`PANEL` shared with the other two lenses), applies them to a THIRD variable, and invents
no fifth color — "no new colors enter the game" holds across all three lenses, not just the one
the ruling named.

- [x] **Step 1: Write the failing tests** for all three band functions — the exact `Srgba` byte
      assertions from the B1 Task 9 spec for `tension_band_color` (CRIMSON at `w <= -0.15`, DIM in
      `(-0.15, 0.15]`, GOLD above, PANEL for `None`; `tension_band_color(Some(0.0)) !=
      tension_band_color(None)`, the Tension lens's OWN non-confusion property, unchanged); for
      `legitimation_band_color`: `Some(0.0)` and `None` BOTH give `PANEL` (the intentional
      equality Director ruling 1 creates — assert them EQUAL, not distinct, and comment why,
      citing this ruling by name so a future reader does not "fix" it back to the first cut's
      design); `Some(1.0)` gives `DIM`; `Some(2.0)` gives `CRIMSON`; for
      `population_trend_band_color`: any positive value gives `GOLD` (spot-check `Some(0.001)` and
      a large value alike — no size cutoff to find the edge of), any negative value gives
      `CRIMSON` (same spot-check shape), `Some(0.0)` gives `DIM`, `None` gives `PANEL`, and
      `population_trend_band_color(Some(0.0)) != population_trend_band_color(None)` (the SAME
      non-confusion property Tension's own table carries — unlike Legitimation, this lens's
      "unchanged" state is NOT meant to look like absence, since a genuinely unchanged county is a
      real, meaningful reading here, not a stand-in for "nothing to report").
- [x] **Step 2:** FAIL, then write all three as `const` tables (or, for `population_trend_band_color`,
      a plain sign match — a two-row table plus its own zero/absence arms would overstate what is
      really an `if`/`else if`/`else`) resolved by the same shape, matching `PANEL`'s existing
      declaration in this file. Neither new function needs a new color constant — both import
      `PANEL`/`DIM`/`CRIMSON`/`GOLD`, all already declared in this file or `crate::palette`.
- [x] **Step 3: The recolor system.** One system, parameterized by `ActiveLens`:

```rust
const ATLAS_BYTES: &[u8] = include_bytes!("../../assets/map/county_atlas.bin");

pub(super) fn recolor_on_lens_changed(
    mut events: EventReader<LensChanged>,
    active: Res<ActiveLens>,
    lens_data: Res<crate::lens::CurrentLensData>, // all THREE LensReading values, refreshed every advance
    surface: Res<MapSurface>,
    mut meshes: ResMut<Assets<Mesh>>,
) {
    if events.read().next().is_none() {
        return;
    }
    let reading = match *active {
        ActiveLens::Tension => &lens_data.tension,
        ActiveLens::Legitimation => &lens_data.legitimation,
        ActiveLens::PopulationTrend => &lens_data.population_trend,
    };
    let color_fn: fn(Option<f64>) -> Color = match *active {
        ActiveLens::Tension => tension_band_color,
        ActiveLens::Legitimation => legitimation_band_color,
        ActiveLens::PopulationTrend => population_trend_band_color,
    };
    // Re-parse the embedded atlas locally rather than reach through a
    // shared `FipsIndex` resource — MEDIUM-HIGH fix: the first cut
    // referenced `Res<FipsIndex>` at three call sites with no task ever
    // defining it, so in the real app this system would never have run at
    // all (Bevy 0.18 skips a system whose resource param cannot resolve,
    // warn-once, no panic — the map would have silently stayed
    // uncolored). `CountyAtlas::parse` is a cheap check-then-decode
    // (Task 4 of the B1 plan's own design) — no tessellation, no
    // allocation beyond the decoded tables — so re-parsing it once per
    // `LensChanged` event (at most once per Space press or Tab press,
    // never per-frame) costs nothing worth caching, and it matches this
    // crate's OWN established convention: `map/camera.rs::spawn_camera`
    // already re-parses the same embedded bytes independently of
    // `map/mesh.rs::spawn_map_surface`'s own parse, specifically to stay
    // free of any same-schedule resource-availability assumption.
    let atlas = crate::atlas::CountyAtlas::parse(ATLAS_BYTES)
        .unwrap_or_else(|e| panic!("county atlas failed to parse: {e}"));
    let Some(mesh) = meshes.get_mut(&surface.fill_mesh) else {
        return;
    };
    let Some(colors) = mesh
        .attribute_mut(Mesh::ATTRIBUTE_COLOR)
        .and_then(|a| a.as_float3_mut())
    else {
        return;
    };
    for (fips, value) in &reading.cells {
        let Some(county_idx) = atlas.index_of_fips(fips) else {
            continue;
        };
        let (start, end) = surface.tessellation.county_vertex_range[county_idx];
        let rgba = color_fn(*value).to_linear().to_f32_array();
        for v in &mut colors[start as usize..end as usize] {
            *v = [rgba[0], rgba[1], rgba[2]];
        }
    }
}
```

      One pass, one buffer, matching B1 Task 9's own "no mesh rebuild" design — this is the same
      recolor shape B1's plan already specified, now parameterized over which lens is active
      instead of fixed to Tension alone, and reading county positions through `atlas.index_of_fips`
      (`atlas.rs`, B1 Task 4) directly rather than a separate index resource.
- [x] **Step 4: Headless test** — `MinimalPlugins` + `AssetPlugin`, install a HAND-BUILT
      `CurrentLensData` with one known Legitimation cell (this test's own job is proving
      `recolor_on_lens_changed`'s LOGIC in isolation, with a fixture the test controls — Task 18
      adds the separate, real-wiring integration test that proves `CurrentLensData` gets populated
      by the real app at all, which this unit-level test cannot and does not claim to prove), set
      `ActiveLens::Legitimation`, fire `LensChanged`, `update()`, assert that county's vertex range
      shows `legitimation_band_color`'s output and every other county's colors held at `PANEL`.
      Add a SPECIFIC regression case: a `Some(0.0)` (STABLE) cell
      and a genuinely-absent cell (a FIPS with no `LensReading` entry at all) produce the SAME
      vertex color — proving the intentional merge, not just the function's return value in
      isolation. Repeat the shape for `ActiveLens::PopulationTrend`: a positive-delta cell shows
      `GOLD`, a negative-delta cell shows `CRIMSON`, and (unlike Legitimation) confirm a
      genuinely-absent cell does NOT match either — `population_trend_band_color`'s own
      non-confusion property, tested at the recolor-system level too, not only the pure function.
- [x] **Step 5: Commit** (`feat(client): three-lens band tables — legitimation reuses PANEL/DIM/
      CRIMSON per Director ruling 1, population trend adds GOLD (B2, completes B1 Phase C Task 9,
      BLOCKER 2 fix)`).

### Task 11: Hover, selection, the active-lens label

**Files:**

- Create: `rust/crates/babylon-client/src/map/pick.rs`, `rust/crates/babylon-client/src/map/hud.rs`

**Interfaces:**

- Produces: `pub struct CountyIndex; pub fn build(atlas: &CountyAtlas) -> CountyIndex; pub fn
  county_at(&self, p: Vec2) -> Option<usize>` (verbatim B1 Task 10 design — uniform grid over
  `world_bounds()`, even-odd ring crossing test, holes inverting membership); `HoveredCounty` and
  `SelectedCounty` resources; the HUD text, now carrying an explicit lens label.

- [x] **Step 1: Write the failing tests** for `county_at` — the same three properties B1 Task 10
      specified: each county's own centroid resolves to itself (floor, not 100%, with exceptions
      listed by FIPS in the test comment); a point in the Gulf of Mexico gives `None`; a point
      inside a county's bounding box but outside its ring gives `None`; the index is identical
      across two builds.
- [x] **Step 2:** FAIL, then write it: a 128x128 uniform grid, bounding-box candidate lists,
      even-odd crossing against the winning candidate's rings.
- [x] **Step 3: Wire the interaction** — `Camera::viewport_to_world_2d` → `county_at` → `HoveredCounty`;
      click promotes to `SelectedCounty`; a GOLD outline at `z = 2.0` over the selection.
- [x] **Step 4: The HUD**, extended past B1 Task 10's spec with the lens label this plan's honesty
      rule adds — and, under the Legitimation lens specifically, carrying MORE weight than usual
      per Task 10's finding that STABLE and absence share a color:

```text
<county name>, <state> (<FIPS>)
Lens: Tension — w = -0.42 (Φ-source, bled)          [if ActiveLens::Tension]
Lens: Legitimation — CRISIS (live, tick 7)          [if ActiveLens::Legitimation, class 2]
Lens: Legitimation — STABLE (live, tick 7)          [class 0 — SAME map color as absence;
                                                       this line is the only place a player
                                                       can tell the two apart]
Lens: Population Trend — +37 since tick 0 (growing) [if ActiveLens::PopulationTrend, delta > 0]
Lens: Population Trend — -19 since tick 0 (declining) [delta < 0]
Lens: Tension — no data this tick                    [absence, any of the three lenses]
```

      Top-left banner whenever the active lens's `absent_reason.is_some()`, in CRIMSON. A
      persistent DIM footer names which lens is ACTIVE and how to cycle (Task 12): "Tab: Tension →
      Legitimation → Population Trend → Tension" — the map must never let a color mean two things
      without saying which one is live, and (per Task 10) a STABLE Legitimation county must never
      let its color alone be mistaken for "no data."
- [x] **Step 5: Headless test** — hovering a known world point sets `HoveredCounty` to the expected
      FIPS, cursor position written directly to the resource (B1 Task 10's own precedent, not
      synthesized window events). Add a case hovering a STABLE demo county and asserting the HUD
      text renders the literal string `"STABLE"` (not merely a color check, since Task 10
      established the color cannot carry this distinction alone), and a case hovering a demo
      county under `ActiveLens::PopulationTrend` and asserting the rendered delta's SIGN matches
      the county's known trajectory direction (Task 9b's table — a `young`-family county must read
      "growing", every other family "declining").
- [x] **Step 6: Commit** (`feat(client): county hover, selection and the active-lens HUD (B2,
      completes B1 Phase C Task 10)`).

### Task 12: Wire `map/mod.rs` — the lens picker

**Files:**

- Edit: `rust/crates/babylon-client/src/map/mod.rs`
- Edit: `rust/crates/babylon-client/src/map/mesh.rs` (widen `spawn_map_surface`'s visibility)

**Housekeeping fix folded in: `spawn_map_surface` needs a `pub` path.** Task 14's `.after(...)`
ordering call references `crate::map::spawn_map_surface`, but B1 landed it as `pub(super) fn
spawn_map_surface` in `mesh.rs` — visible inside `map/`'s own module tree, not to sibling modules
like `loop_ui.rs`. Widen it to `pub fn spawn_map_surface` in `mesh.rs` (a one-word change, no
behavior change) and add it to this task's `pub use mesh::{...}` line below, alongside the three
types B1 already re-exports there.

- [x] **Step 1: Write the failing headless test** — `MinimalPlugins` + `AssetPlugin` +
      `babylon_client::map::MapPlugin`, assert the STARTUP default `ActiveLens` is
      `PopulationTrend` (not `Tension` — see the note below), then press `Tab` three times (write
      directly into `ButtonInput<KeyCode>`, matching the input-resource-mutation pattern this
      plan's tests already use, one `update()` per press), assert `ActiveLens` visits `Tension` →
      `Legitimation` → `PopulationTrend` in that cycle order (a 3-way CYCLE, not a 2-way flip — the
      first cut's design only had two lenses), and a `LensChanged` event fires on every press.
- [x] **Step 2:** FAIL, then add: `mod pick; mod hud;` (new modules from this task); in
      `mesh.rs`, change `pub(super) fn spawn_map_surface` to `pub fn spawn_map_surface`; `pub use
      bands::{ActiveLens, LensChanged}; pub use mesh::spawn_map_surface;` alongside the existing
      `pub use mesh::{MapBorders, MapFill, MapSurface, EXPECTED_VERTEX_COUNT};` and `pub use
      bands::PANEL;` — the same re-export convention B1 already established, extended to the two
      new types and the one function so `crate::map::ActiveLens`/`crate::map::LensChanged`/
      `crate::map::spawn_map_surface` (the paths Task 14's and Task 18's code use) all resolve;
      **`ActiveLens::PopulationTrend` inserted as the `Startup` default resource —
      NOT `Tension`.** Task 8's own finding: this demo scenario seeds no `v`/`s`/`e`-style economic
      fields at all, so `county_tension` returns fully absent on every county, every tick, on this
      content — defaulting to it would open the app on an absence banner over an all-`PANEL` map.
      `PopulationTrend` is the one lens guaranteed to carry real, changing, non-absent data on this
      demo from tick 0. An `Update` system reads `Tab` and CYCLES the active lens
      (`Tension -> Legitimation -> PopulationTrend -> Tension`, a `match` naming all three arms
      explicitly, so no wraparound bug can hide — the STARTING point is
      `PopulationTrend`, and the cycle ORDER keeps matching the test above) plus sends
      `LensChanged`; Task 10's `recolor_on_lens_changed` system registered.
- [x] **Step 3:** `cargo test -p babylon-client` → PASS. `mise run rust:check` → green.
- [x] **Step 4: Commit** (`feat(client): wire the 3-way lens picker into MapPlugin (B2)`). ~~Open the
      Phase C PR (`feat(client): B2 Phase C — the three-lens map, completing B1's Phase C`);
      self-merge on green.~~ **NOT executed as written (adversarial-panel FB6, annotated rather
      than silently checked off):** the executing agent's own standing instructions override this
      step's per-phase-PR/self-merge default — Task 19 opens ONE PR for the whole plan (all five
      phases, 21 commits) and does not merge it (merging goes through the verification + ADR181
      protocol separately). No Phase C PR was opened; this commit landed directly on
      `feat/b2-tick-loop`, same as every other task's commit.

---

## Phase D — The loop UI

### Task 13: `EngineSession` — the client's held tick session

**Files:**

- Edit: `rust/crates/babylon-client/src/engine_link.rs`

**Interfaces:**

- Produces:

```rust
pub struct EngineSession {
    pub inner: TickSession<HypergraphStore>,
    pub sink: CollectingSink,
    pub node_by_fips: Vec<(String, NodeId)>,
    pub population_baseline: Vec<(String, f64)>,
}
impl EngineSession {
    pub fn start() -> Result<Self, String>;
    pub fn advance(&mut self) -> Result<TickReport, String>;
}
```

**`population_baseline` (NEW this revision, BLOCKER 2 fix)** carries each demo territory's tick-0
total population (`pop-d + pop-p + pop-d-prime`, read straight off the graph the moment
`load_scenario` returns, before any `advance()` runs) — Task 9b's Population Trend lens needs a
fixed reference point to measure change AGAINST, and tick 0 (the scenario's own declared, un-ticked
state) is the only point that is honestly "before" every tick this session ever runs. Captured
once, in the SAME shape as `node_by_fips` (`(fips, value)` pairs, same twelve-entry order), never
recomputed.

**Why `node_by_fips` is a plain `Vec`, not a `babylon-bsl` API addition.** `load_scenario`'s local
name -> `NodeId` map is deliberately load-time-only and does not outlive the call (`scenario.rs`'s
own comment). This plan does not widen that API. Instead: the Phase B scenario mints EXACTLY the
twelve `NodeType/TERRITORY` nodes and six `NodeType/SOCIAL_CLASS` nodes (that BSL SOURCE syntax —
the `<enum-ref>` — is what the `.bscn` file itself writes; the RUNTIME string the substrate stores
and matches is the bare enum member, `"TERRITORY"`/`"SOCIAL_CLASS"`, confirmed against
`namespace_to_node_type`'s own doc comment and every live `nodes()` call site in the crate), in
file order, and no others; `GraphSubstrate::nodes("TERRITORY")` — the bare string, NEVER
`"NodeType/TERRITORY"`, which matches nothing and returns an empty `Vec` silently — filters BY
TYPE and returns ascending `NodeId`s among territory nodes only, which equal territory-mint order
because `NodeId` mints as a GLOBAL monotonic counter across the whole scenario (ADR193) and the
type filter preserves relative order within the filtered subset — verified by reading both
`nodes()` implementations (`memory.rs`/`hypergraph_store.rs`: both `.filter(...)` on the type
string THEN `.sort_unstable()` the surviving ids, so the shape holds regardless of which string the
filter matches — only the STRING itself needed the fix this task now carries): interleaving
social-class and territory node declarations in the `.bscn` file changes the ABSOLUTE `NodeId`
values but not their RELATIVE order among same-typed nodes, so this zip is correct regardless of
how the two halves interleave in the file. Zipping `graph.nodes("TERRITORY")` against a `const
DEMO_FIPS: [&str; 12]` array **in the same order as the `.bscn` file's twelve territory
`(node …)` forms** recovers the
fips↔id mapping with no new babylon-bsl surface — fragile only in the sense that editing the
`.bscn` file's territory node order without updating `DEMO_FIPS` would silently mislabel a county,
which Step 2's loud startup assertion turns into an immediate panic instead. **Social-class nodes
get no matching index** — the event feed (Task 15) reads `sink.events` generically and needs no
per-class lookup; a class-scoped state panel sits outside this task's scope (noted, not built).

- [x] **Step 1: Write the failing test.**

```rust
#[test]
fn engine_session_starts_and_the_twelve_fips_resolve_on_the_real_atlas() {
    let session = EngineSession::start().expect("engine session starts");
    assert_eq!(session.node_by_fips.len(), 12);
    let atlas = CountyAtlas::parse(include_bytes!("../assets/map/county_atlas.bin")).unwrap();
    for (fips, _id) in &session.node_by_fips {
        assert!(
            atlas.index_of_fips(fips).is_some(),
            "demo FIPS {fips} must resolve on the committed atlas"
        );
    }
}

#[test]
fn population_baseline_matches_the_seeded_tick_zero_totals() {
    let session = EngineSession::start().expect("engine session starts");
    assert_eq!(session.population_baseline.len(), 12);
    // Task 7's Step 2 table: index 0 (core ×0.95) seeds pop-d=2042,
    // pop-p=5748, pop-d-prime=1710 — total 9,500 exactly. Same fips order
    // as node_by_fips/DEMO_FIPS.
    let (fips0, total0) = &session.population_baseline[0];
    assert_eq!(fips0, &session.node_by_fips[0].0);
    assert!((total0 - 9500.0).abs() < 1e-6, "got {total0}");
}

#[test]
fn engine_session_advance_moves_the_hash_and_runs_both_rules_every_tick() {
    let mut session = EngineSession::start().expect("start");
    let r1 = session.advance().expect("tick 1");
    let r2 = session.advance().expect("tick 2");
    assert_eq!(session.inner.tick(), 2);
    assert_ne!(r1.after, r2.after);
    assert_eq!(r1.per_rule_fired.len(), 2, "both packs run every tick");
}
```

- [x] **Step 2:** FAIL, then write it:

```rust
const SCENARIO: &str =
    include_str!("../../babylon-tick/content/scenarios/us-counties-lifecycle-demo.bscn");
const VITALITY: &str = include_str!("../../babylon-tick/content/rules/vitality.bsl");
const LIFECYCLE: &str = include_str!("../../babylon-tick/content/rules/lifecycle.bsl");

/// Same order as `us-counties-lifecycle-demo.bscn`'s twelve territory
/// `(node …)` forms — Task 7's Step 1 print output, transcribed. A loud
/// startup assertion (below) catches the two arrays ever drifting apart.
const DEMO_FIPS: [&str; 12] = [ /* the twelve FIPS from Task 7 Step 1, in file order */ ];

pub struct EngineSession {
    pub inner: TickSession<HypergraphStore>,
    pub sink: CollectingSink,
    pub node_by_fips: Vec<(String, NodeId)>,
    pub population_baseline: Vec<(String, f64)>,
}

impl EngineSession {
    pub fn start() -> Result<Self, String> {
        let mut graph = HypergraphStore::new();
        // Load through the same load path TickSession uses internally —
        // but we need the territory node ids (and their SEEDED, pre-tick
        // population totals) BEFORE TickSession takes ownership of the
        // graph, so load once here to capture both, then hand a FRESH
        // graph to TickSession::new (it reloads the same scenario, which
        // is deterministic and mints the identical eighteen ids — proven
        // by this task's own Step 1 test, which checks both independently).
        babylon_bsl::scenario::load_scenario(SCENARIO, &mut graph).map_err(|e| e.to_string())?;
        // "TERRITORY" — the bare enum member the substrate actually stores
        // (namespace_to_node_type stamps it verbatim), never "NodeType/TERRITORY".
        let ids = babylon_graph::substrate::GraphSubstrate::nodes(&graph, "TERRITORY");
        if ids.len() != DEMO_FIPS.len() {
            panic!(
                "demo scenario minted {} TERRITORY nodes, DEMO_FIPS names {} — \
                 the array drifted from the .bscn file, fix DEMO_FIPS",
                ids.len(),
                DEMO_FIPS.len()
            );
        }
        let node_by_fips: Vec<(String, NodeId)> = DEMO_FIPS
            .iter()
            .zip(ids.iter())
            .map(|(fips, id)| ((*fips).to_owned(), *id))
            .collect();
        // The tick-0 baseline Task 9b's Population Trend lens measures
        // against — read from THIS graph, before it is discarded, while
        // it still holds only the scenario's seeded (un-ticked) values.
        let population_baseline: Vec<(String, f64)> = node_by_fips
            .iter()
            .map(|(fips, id)| {
                let pop_d = graph.node_attribute(*id, "territory/pop-d").unwrap_or(0.0);
                let pop_p = graph.node_attribute(*id, "territory/pop-p").unwrap_or(0.0);
                let pop_dp = graph
                    .node_attribute(*id, "territory/pop-d-prime")
                    .unwrap_or(0.0);
                (fips.clone(), pop_d + pop_p + pop_dp)
            })
            .collect();

        // Concatenation order below is arbitrary — Phase A's driver sorts
        // by rule-id BYTE ORDER (§4.2, D16) regardless of which text comes
        // first, so this order has no bearing on execution order or on
        // the resulting TickReport (Task 5 proves the file-order
        // invariance directly). Vitality text stays first here only
        // because it reads naturally next to the const list above it, not
        // because it changes anything.
        let rule_src = format!("{VITALITY}\n{LIFECYCLE}");
        let inner = TickSession::new(SCENARIO, &rule_src, HypergraphStore::new())
            .map_err(|e| format!("tick session: {e}"))?;

        Ok(Self {
            inner,
            sink: CollectingSink::default(),
            node_by_fips,
            population_baseline,
        })
    }

    pub fn advance(&mut self) -> Result<TickReport, String> {
        self.inner.advance(&mut self.sink)
    }
}
```

      Note the deliberate double-load (once to recover territory ids and the population baseline,
      once inside `TickSession::new`) rather than widening `TickSession` to expose its internal
      graph mutably before the first `advance` — it costs one extra scenario parse at startup
      (still microseconds against an 18-node scenario) and keeps `TickSession`'s public surface
      exactly the four methods Task 6 specified. `graph.node_attribute` returning `Result<f64,
      GraphError>`, not `Option`, per `GraphSubstrate`'s existing signature — `unwrap_or(0.0)` is
      safe here specifically because Task 7's scenario declares EVERY population field on EVERY
      territory node explicitly (no field is ever unset at mint time), so the error path is
      unreachable for this scenario, not silently papered over for one that might omit a field.
- [x] **Step 3:** `cargo test -p babylon-client` → PASS.
- [x] **Step 4: Commit** (`feat(client): EngineSession — the client's held two-rule TickSession,
      fips↔id map, and tick-0 population baseline (B2)`).

### Task 14: Advance-tick input, tick counter, hash readout

**Files:**

- Edit: `rust/crates/babylon-client/src/main.rs`

- [x] **Step 1: Write the failing headless test.**

```rust
// tests/tick_loop.rs
use bevy::asset::AssetPlugin;
use bevy::prelude::*;

#[test]
fn pressing_space_advances_the_tick_and_updates_the_hash_text() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), InputPlugin));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.update(); // Startup: EngineSession inserted, tick 0

    let counter = app.world().resource::<babylon_client::loop_ui::TickCounter>();
    assert_eq!(counter.0, 0);

    let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
    input.press(KeyCode::Space);
    app.update();

    let counter = app.world().resource::<babylon_client::loop_ui::TickCounter>();
    assert_eq!(counter.0, 1);
}
```

- [x] **Step 2:** FAIL (`loop_ui` module does not exist).
- [x] **Step 3: Write `src/loop_ui.rs`** (new module, `pub mod loop_ui;` in `lib.rs`):

```rust
//! The B2 tick loop's own UI plumbing: Space advances the tick, a text
//! node shows the counter and the deterministic hash — the honesty proof
//! (III.7) rendered where the player can see it move.

use crate::engine_link::EngineSession;
use bevy::prelude::*;

#[derive(Resource, Default)]
pub struct TickCounter(pub i64);

#[derive(Component)]
pub struct HashReadout;

#[derive(Component)]
pub struct TickCounterReadout;

pub struct TickLoopPlugin;

impl Plugin for TickLoopPlugin {
    fn build(&self, app: &mut App) {
        app.insert_resource(TickCounter::default());
        // `.after(map::spawn_map_surface)`: this system fires the FIRST
        // `LensChanged` at tick 0, and `recolor_on_lens_changed` (Task 10)
        // reads the `MapSurface` resource `MapPlugin`'s OWN Startup system
        // creates (Task 10 dropped the separate `FipsIndex` resource this
        // note used to also require — it re-parses the atlas locally
        // instead, so no second Startup dependency remains). Bevy does not
        // order same-schedule systems by plugin-registration order alone —
        // this ordering constraint must be explicit, not implied by
        // `main.rs` listing `MapPlugin` before `TickLoopPlugin`.
        app.add_systems(
            Startup,
            spawn_engine_session_and_hud.after(crate::map::spawn_map_surface),
        );
        app.add_systems(Update, (advance_on_space, refresh_readouts).chain());
    }
}

fn spawn_engine_session_and_hud(
    mut commands: Commands,
    mut lens_changed: EventWriter<crate::map::LensChanged>,
) {
    let session = EngineSession::start()
        .unwrap_or_else(|e| panic!("engine session failed to start: {e}"));
    // Tick 0's own LensReadings — the map must show something correct
    // (or correctly absent) on first launch, before any Space press. The
    // Population Trend reading is `Some(0.0)` (DIM) everywhere at this
    // point, since `population_baseline` IS the tick-0 state — real
    // divergence appears only after the first `advance()`.
    let lens_data = crate::map::CurrentLensData {
        tension: crate::lens::county_tension(session.inner.graph()),
        legitimation: crate::lens::county_legitimation(session.inner.graph(), &session.node_by_fips),
        population_trend: crate::lens::county_population_trend(
            session.inner.graph(),
            &session.node_by_fips,
            &session.population_baseline,
        ),
    };
    commands.insert_resource(lens_data);
    commands.insert_resource(session);
    lens_changed.write(crate::map::LensChanged);
    commands.spawn((
        Text::new("tick 0"),
        TextColor(crate::palette::BONE),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(24),
            right: px(24),
            ..default()
        },
        TickCounterReadout,
    ));
    commands.spawn((
        Text::new("hash: (not yet run)"),
        TextColor(crate::palette::DIM),
        Node {
            position_type: PositionType::Absolute,
            bottom: px(4),
            right: px(24),
            ..default()
        },
        HashReadout,
    ));
}

fn advance_on_space(
    keys: Res<ButtonInput<KeyCode>>,
    mut session: ResMut<EngineSession>,
    mut counter: ResMut<TickCounter>,
    mut lens_data: ResMut<crate::map::CurrentLensData>,
    mut lens_changed: EventWriter<crate::map::LensChanged>,
) {
    if !keys.just_pressed(KeyCode::Space) {
        return;
    }
    session
        .advance()
        .unwrap_or_else(|e| panic!("tick advance failed: {e}"));
    counter.0 = session.inner.tick();
    // Recompute all THREE LensReadings against the POST-tick graph before
    // firing LensChanged — the recolor system (Task 10) only ever reads
    // whatever is already in CurrentLensData when the event fires, so a
    // press that advanced the tick but never refreshed this resource would
    // leave the map showing stale (or, on the very first press, entirely
    // absent) data forever. This is the wiring that makes "watch state
    // change" literally true rather than merely possible.
    lens_data.tension = crate::lens::county_tension(session.inner.graph());
    lens_data.legitimation =
        crate::lens::county_legitimation(session.inner.graph(), &session.node_by_fips);
    lens_data.population_trend = crate::lens::county_population_trend(
        session.inner.graph(),
        &session.node_by_fips,
        &session.population_baseline,
    );
    lens_changed.write(crate::map::LensChanged);
}

fn refresh_readouts(
    counter: Res<TickCounter>,
    session: Res<EngineSession>,
    mut tick_text: Query<&mut Text, (With<TickCounterReadout>, Without<HashReadout>)>,
    mut hash_text: Query<&mut Text, With<HashReadout>>,
) {
    if !counter.is_changed() {
        return;
    }
    if let Ok(mut t) = tick_text.single_mut() {
        t.0 = format!("tick {}", counter.0);
    }
    if let Ok(mut h) = hash_text.single_mut() {
        // The last hash this session computed — sink carries no hash, so
        // read it back off the session's own last report by re-deriving
        // from the graph directly (state_hash is cheap at this scale, see
        // the Global Constraints Scale Note — 18 nodes, 0 hyperedges).
        if let Ok(hash) = babylon_graph::state_hash::CanonicalState::state_hash(session.inner.graph())
        {
            h.0 = format!("hash: {}", babylon_tick::hex(&hash));
        }
    }
}
```

      Wire `TickLoopPlugin` into `main.rs`'s `App::new()` chain, replacing B0/B1's
      `log_engine_link` Startup system (superseded — `EngineSession::start` now IS the engine link,
      and it panics loudly on failure exactly as `log_engine_link` did).
- [x] **Step 4:** `cargo test -p babylon-client --test tick_loop` → PASS. `mise run rust:check` →
      green.
- [ ] (pending Director) **Step 5: Eyes-on:** `cargo run -p babylon-client` — press Space repeatedly, watch the tick
      counter and hash text change every press.
- [x] **Step 6: Commit** (`feat(client): advance-tick input, tick counter, hash readout (B2)`).

### Task 15: The state panel and the event feed

**Files:**

- Edit: `rust/crates/babylon-client/src/loop_ui.rs`

- [x] **Step 1: Write the failing headless test** — after two `advance()` calls with a county
      selected (write `SelectedCounty` directly, matching Task 11's pick-testing precedent), the
      state panel's text contains that county's live `pop-d`/`pop-p`/`pop-d-prime`/
      `legitimation-index` values read straight off the graph (not off the lens, which only carries
      the classification) — proving the panel and the map agree because both read the same graph.
- [x] **Step 2:** FAIL, then write `spawn_state_panel`/`refresh_state_panel`. `SelectedCounty`
      (Task 11) wraps an ATLAS INDEX (`usize`), not a `NodeId` — the map's own vocabulary, matching
      `county_at`'s return type. Resolve the chain explicitly: atlas index -> `atlas.county(idx).fips`
      -> a linear scan of `session.node_by_fips` (twelve entries — a `HashMap` is not worth building
      for this size) for the matching FIPS -> its `NodeId`. A `SelectedCounty`/`HoveredCounty` whose
      atlas index resolves to a FIPS absent from `node_by_fips` (any of the 3,210 non-demo counties)
      renders the panel's honest "no data this tick" text, never a lookup panic — this is the same
      honest-absence shape Task 8's `LensReading` already establishes, applied here to the panel
      instead of the map. For a resolved `id`, read the four fields via
      `session.inner.graph().node_attribute(id, "...")` and render:

```text
<county name> (<fips>)
  pop-d:            2,150
  pop-p:            6,050
  pop-d-prime:      1,800
  legitimation:     STABLE (0)
```

- [x] **Step 3: The event feed — now genuinely two-pack.** A scrolling text list, last 10 entries
      from `session.sink.events`, newest first, rendered as `<EventType> @ <county or n/a>` —
      reusing `CollectingSink`'s already-populated `events: Vec<(String, Vec<(String, Value)>)>`
      with no new sink type. Because `EngineSession` (Task 13) now runs `lifecycle` THEN `vitality`
      every tick (ascending rule-id byte order, §4.2/D16 — see the Multi-Rule Decision section),
      `sink.events` genuinely mixes BOTH packs' emissions, with the `lifecycle` events first each tick:
      `vitality`'s `EventType/ENTITY_DEATH` (its own payload carries `entity-id`, no county
      binding — rendered as `@ n/a`) alongside `lifecycle`'s
      `LIFECYCLE_TRANSITION`/`LEGITIMATION_CRISIS`/`LEGITIMATION_RECOVERY` (county-bound, rendered
      with the county's name where the payload's `entity-id` resolves through `node_by_fips`) —
      this IS the "richer first demo" the Director's ruling asked for, concretely: the feed is
      where the vitality half of the demo becomes visible at all, since Task 7 already established
      it has no map-color counterpart. Bounded to the last 10 by slicing, not by mutating
      `sink.events` (the sink accumulates the WHOLE session's history — acceptable at demo scale, a
      ring buffer is a documented future item if unbounded play sessions become a target, not built
      here).
- [x] **Step 4: Headless test** for the event feed — after an `advance()` that fires
      `LEGITIMATION_RECOVERY` (Task 7's own recovering-county archetype guarantees this on tick 1),
      assert the feed's rendered text contains `"LEGITIMATION_RECOVERY"`. Add a second assertion
      proving the two-pack mix specifically: over enough ticks for the fixture's `last-worker`
      subject to starve (its own conformance fixture already proves this fires within a handful of
      ticks — `vitality-conformance.bscn`'s own comment names it "Starvation"), the feed also
      contains `"ENTITY_DEATH"` — both event families visible in one feed, not merely present in
      the sink.
- [x] **Step 5:** `cargo test -p babylon-client` → PASS.
- [ ] (pending Director) **Step 5 (eyes-on half):** select a county, press Space, watch its panel
      numbers and the event feed both update, and confirm `ENTITY_DEATH` events appear alongside
      the lifecycle events over a longer run — needs a display server this environment does not
      have. (Restructured onto its own unchecked line, adversarial-panel FB6 — the automated half
      above and this human half were previously buried on one `- [x]` line, which a mechanical
      `grep '\- \[ \]'` sweep for pending-Director items would have missed; Task 14 Step 5 already
      used this two-line shape.)
- [x] **Step 6: Commit** (`feat(client): the state panel and event feed — now two packs deep (B2)`).
      ~~Open the Phase D PR (`feat(client): B2 Phase D — the tick loop UI`); self-merge on green.~~
      **NOT executed as written (adversarial-panel FB6, annotated rather than silently checked
      off):** same override as Task 12 Step 4 — Task 19 opens the ONE plan-wide PR; no Phase D PR
      was opened, this commit landed directly on `feat/b2-tick-loop`.

---

## Phase E — Logging, determinism, the eyes-on gate

### Task 16: Resurrect the client file-log sink

**Files:**

- Create: `rust/crates/babylon-client/src/logging.rs`
- Edit: `rust/crates/babylon-client/Cargo.toml`, `rust/crates/babylon-client/src/lib.rs`,
  `rust/crates/babylon-client/src/main.rs`

**Why resurrect rather than reinvent.** CLAUDE.md's client-logging directive: *"the deletion
ceremony retired `rust-client.log` (the Ratatui client's `log4rs` sink); the Bevy client's file
sink lands at milestone B2."* The deleted module (`git show
7d9f0d94^:rust/crates/babylon-tui/src/logging.rs`) carries proven, tested code that already solves
this problem — 10 MB size-triggered rotation, 5 fixed-window archives, file-only (never the
terminal, though Bevy has no alternate screen to corrupt the way the deleted Ratatui client did —
the constraint carries over anyway since a console `log4rs` sink would just duplicate Bevy's own
`tracing`-based `LogPlugin`, not harm it). **`log4rs` listens on the plain `log` facade; Bevy's own
console/dev logging runs on `tracing` via `bevy::log::LogPlugin`** — the two global dispatchers
(`log::set_logger` and `tracing::subscriber::set_global_default`) are independent slots, so both
coexist: Bevy's internals keep printing to the console through `tracing` exactly as
`DefaultPlugins` already wires it, and THIS crate's own `log::debug!`/`log::info!` calls (not
`bevy::log::info!`, which is `tracing`) go to the file sink only.

- [x] **Step 1: Add dependencies**, the exact deleted feature set (`git show
      7d9f0d94^:rust/crates/babylon-tui/Cargo.toml` lines 36-42):

```toml
log = "0.4"
log4rs = { version = "1", default-features = false, features = [
    "rolling_file_appender",
    "compound_policy",
    "size_trigger",
    "fixed_window_roller",
    "pattern_encoder",
] }
```

- [x] **Step 2: Resurrect the module**, transcribed from the deleted file with two changes: the
      sink filename `rust-client.log` → `babylon-client.log` (the retired name stays retired, per
      CLAUDE.md — this is a new client, not a relaunch of the old one) and the module doc's
      "terminal takeover" framing replaced with the Bevy-coexistence framing above.

```rust
//! File-only client logging (Director directive 2026-07-28; resurrected
//! for the Bevy client at Program 28 B2, per CLAUDE.md: "the Bevy client's
//! file sink lands at milestone B2"). Built on `log4rs` — the same crate
//! and the same rotation policy the deleted Ratatui client's
//! `babylon-tui::logging` used, transcribed here rather than reinvented.
//!
//! **Independent of Bevy's own logging.** `bevy::log::LogPlugin` runs on
//! `tracing` and keeps printing to stderr exactly as `DefaultPlugins`
//! wires it — untouched by this module. `log4rs` listens on the separate
//! `log` facade; this crate's OWN `log::debug!`/`log::info!` calls (never
//! `bevy::log::info!`, which is `tracing`) are what land in the file.
//!
//! **No wall-clock in client source.** Timestamps come from `log4rs`'s own
//! pattern encoder inside the appender.

use std::path::Path;
use std::sync::OnceLock;

use log::LevelFilter;
use log4rs::append::rolling_file::policy::compound::roll::fixed_window::FixedWindowRoller;
use log4rs::append::rolling_file::policy::compound::trigger::size::SizeTrigger;
use log4rs::append::rolling_file::policy::compound::CompoundPolicy;
use log4rs::append::rolling_file::RollingFileAppender;
use log4rs::config::{Appender, Config, Root};
use log4rs::encode::pattern::PatternEncoder;

const LOG_MAX_BYTES: u64 = 10 * 1024 * 1024;
const LOG_ARCHIVES: u32 = 5;

static INIT: OnceLock<()> = OnceLock::new();

/// `$XDG_DATA_HOME/babylon/logs` else `~/.local/share/babylon/logs` —
/// mirrors `src/babylon/config/paths.py::player_data_dir()` /
/// `src/babylon/config/base.py::LOG_DIR` exactly, transcribed rather than
/// re-derived (no PyO3 in the play path, Amendment AF, so this cannot call
/// the Python function — it reproduces its two-line rule instead).
#[must_use]
pub fn log_dir() -> std::path::PathBuf {
    log_dir_from(
        std::env::var_os("XDG_DATA_HOME").map(std::path::PathBuf::from),
        std::env::var_os("HOME").map(std::path::PathBuf::from),
    )
}

/// The pure resolution rule behind [`log_dir`], with both environment
/// inputs injected — the test exercises this directly, so it never mutates
/// process-global env vars (cargo runs tests in parallel; a `set_var` in
/// one test races every other test's threads).
fn log_dir_from(
    xdg_data_home: Option<std::path::PathBuf>,
    home: Option<std::path::PathBuf>,
) -> std::path::PathBuf {
    let base = xdg_data_home
        .unwrap_or_else(|| home.expect("HOME must be set").join(".local").join("share"));
    base.join("babylon").join("logs")
}

/// Install the rolling-file logger writing `babylon-client.log` under
/// `log_dir`. `level` is one of `error|warn|info|debug|trace`.
///
/// # Errors
/// A config defect (bad level, non-UTF-8 log dir, log4rs init failure).
pub fn init_file_logging(log_dir: &Path, level: &str) -> Result<(), String> {
    let level_filter = parse_level(level)?;
    if INIT.get().is_some() {
        return Ok(());
    }
    let roll_pattern = log_dir.join("babylon-client.{}.log");
    let roller = FixedWindowRoller::builder()
        .build(
            roll_pattern
                .to_str()
                .ok_or_else(|| format!("log dir is not valid UTF-8: {}", log_dir.display()))?,
            LOG_ARCHIVES,
        )
        .map_err(|e| format!("log roller config: {e}"))?;
    let policy = CompoundPolicy::new(Box::new(SizeTrigger::new(LOG_MAX_BYTES)), Box::new(roller));
    let appender = RollingFileAppender::builder()
        .encoder(Box::new(PatternEncoder::new(
            "{d(%Y-%m-%dT%H:%M:%S%.3f)} [{l}] {t} — {m}{n}",
        )))
        .build(log_dir.join("babylon-client.log"), Box::new(policy))
        .map_err(|e| format!("log appender: {e}"))?;
    let config = Config::builder()
        .appender(Appender::builder().build("file", Box::new(appender)))
        .build(Root::builder().appender("file").build(level_filter))
        .map_err(|e| format!("log config: {e}"))?;
    log4rs::init_config(config).map_err(|e| format!("log init: {e}"))?;
    let _ = INIT.set(());
    install_panic_hook();
    Ok(())
}

fn parse_level(level: &str) -> Result<LevelFilter, String> {
    match level {
        "error" => Ok(LevelFilter::Error),
        "warn" => Ok(LevelFilter::Warn),
        "info" => Ok(LevelFilter::Info),
        "debug" => Ok(LevelFilter::Debug),
        "trace" => Ok(LevelFilter::Trace),
        other => Err(format!(
            "unknown log_level {other:?} (expected error|warn|info|debug|trace)"
        )),
    }
}

fn install_panic_hook() {
    let previous = std::panic::take_hook();
    std::panic::set_hook(Box::new(move |info| {
        log::error!(target: "panic", "client panic: {info}");
        log::logger().flush();
        previous(info);
    }));
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn an_unknown_level_fails_loudly_even_after_init() {
        let err = init_file_logging(Path::new("/nonexistent"), "loudest").unwrap_err();
        assert!(err.contains("unknown log_level"));
    }

    #[test]
    fn init_writes_the_sink_and_reinit_is_a_noop_success() {
        let dir = std::env::temp_dir().join(format!("babylon-client-logtest-{}", std::process::id()));
        std::fs::create_dir_all(&dir).expect("test log dir");
        init_file_logging(&dir, "debug").expect("first init");
        log::debug!(target: "test", "sink probe line");
        log::logger().flush();
        let sink = dir.join("babylon-client.log");
        let written = std::fs::read_to_string(&sink).expect("sink exists");
        assert!(written.contains("sink probe line"));
        init_file_logging(&dir, "debug").expect("re-init is a no-op success");
        std::fs::remove_dir_all(&dir).ok();
    }

    #[test]
    fn log_dir_honors_xdg_data_home() {
        // Injected inputs, no env mutation — parallel-safe by construction
        // (the deleted TUI module's test set XDG_DATA_HOME process-globally
        // and leaned on a single-threaded assumption; not transcribed).
        let xdg = log_dir_from(Some(std::path::PathBuf::from("/tmp/xdg-probe")), None);
        assert_eq!(xdg, std::path::PathBuf::from("/tmp/xdg-probe/babylon/logs"));
        let fallback = log_dir_from(None, Some(std::path::PathBuf::from("/home/probe")));
        assert_eq!(
            fallback,
            std::path::PathBuf::from("/home/probe/.local/share/babylon/logs")
        );
    }
}
```

- [x] **Step 3: Wire it in `main.rs`**, before `App::new()`:

```rust
fn main() {
    let log_dir = babylon_client::logging::log_dir();
    std::fs::create_dir_all(&log_dir).ok();
    if let Err(e) = babylon_client::logging::init_file_logging(&log_dir, "debug") {
        eprintln!("warning: client file logging did not start: {e}");
    }
    log::info!("babylon-client starting (B2 tick loop)");
    App::new()
        // ... unchanged ...
}
```

      A logging failure is a `warning`, not a panic — the game must still be playable with no
      writable log directory (a read-only filesystem, a sandboxed CI runner), which is exactly why
      Step 4's test needs no log directory to exist.
- [x] **Step 4:** `cargo test -p babylon-client --lib logging` → PASS (all three tests). `mise run
      rust:check` → green. `cargo deny check` — `log4rs`/`log` are the same crates the deleted TUI
      already carried, and its `deny.toml`'s `allowlist` already names them; confirm rather than
      assume.
- [x] **Step 5: Commit** (`feat(client): resurrect the log4rs file sink — babylon-client.log
      (B2)`).

### Task 17: End-to-end determinism guard

**Files:**

- Create: `rust/crates/babylon-client/tests/determinism.rs`

**Why this test exists separately from Task 6's `babylon-tick`-level version.** Task 6's test
proves `TickSession` itself is deterministic across a multi-rule content set. This test proves the
SAME property through the client's own composed seam — `EngineSession::start` + repeated
`advance()` — which is the actual path a player's key presses drive, and the one the plan's own
instructions ask to see "as a committed test."

- [x] **Step 1: Write the failing test.**

```rust
use babylon_client::engine_link::EngineSession;

#[test]
fn same_content_same_tick_count_yields_the_same_hash() {
    let mut a = EngineSession::start().expect("session a");
    let mut b = EngineSession::start().expect("session b");
    for tick in 1..=5 {
        let ra = a.advance().expect("a advances");
        let rb = b.advance().expect("b advances");
        assert_eq!(
            ra.after, rb.after,
            "tick {tick}: two independent EngineSessions over the same content must hash identically"
        );
        assert_eq!(
            ra.per_rule_fired, rb.per_rule_fired,
            "tick {tick}: per-rule detail must also match — the order proof, not just the hash"
        );
    }
}

#[test]
fn five_ticks_produce_five_distinct_hashes() {
    // Regression guard against a driver that silently re-runs tick 1 —
    // exactly the bug TickSession's own tick-numbering exists to prevent;
    // this test watches for it at the client's seam too.
    let mut session = EngineSession::start().expect("session");
    let mut hashes = std::collections::HashSet::new();
    for _ in 0..5 {
        let report = session.advance().expect("advance");
        hashes.insert(report.after);
    }
    assert_eq!(hashes.len(), 5, "each tick must produce a distinct state hash");
}
```

- [x] **Step 2:** FAIL until Task 13's `EngineSession` exists (this task can run any time after
      Task 13 — placed last only to sit beside Task 16's logging work in one PR).
- [x] **Step 3:** `cargo test -p babylon-client --test determinism` → PASS.
- [x] **Step 4: Commit** (`test(client): end-to-end determinism guard — same content, same tick
      count, same hash and same per-rule order (B2)`).

### Task 18: The eyes-on gate

**Files:**

- Create: `rust/crates/babylon-client/tests/eyes_on_smoke.rs`
- Edit: `ai/state.yaml`

**Definition (replaces #262, per the roadmap spec §5's board-hygiene note).** A person satisfies
B2's eyes-on gate by:

1. Running `cargo run -p babylon-client`.
2. Seeing the county map render on the DEFAULT `PopulationTrend` lens (Task 12's finding: `Tension`
   defaults would open on an absence banner over an all-`PANEL` map on this demo content, so the
   app does not default there) — the same borders, and `DIM` on all twelve demo counties, everything
   else `PANEL`. **DIM, not GOLD/CRIMSON, at this exact moment** — `population_trend` measures
   change SINCE the tick-0 baseline, and tick 0 IS the baseline (`Some(0.0)` everywhere, `0.0` maps
   to `DIM` per Task 10's own table), so the honest opening view is uniform, not pre-differentiated.
3. Pressing **Space** at least five times, and after each press observing every one of the
   following:
   - the tick counter (bottom-right) increments by exactly one;
   - the hash readout changes to a new hex string every press (never repeats — Task 17 proves this
     is a real property, not a hope);
   - **after the FIRST press**, the twelve demo counties split into GOLD (`young`-family, net
     growth — Task 9b's table) and CRIMSON (the other three families, net decline) — this is the
     moment "watch state change" becomes visible ON THE MAP, not merely in a readout, and every
     press after the first should show the same GOLD/CRIMSON counties growing further apart in
     size (visible in the HUD when hovered, or in the state panel's raw numbers), never
     flipping color family; the Legitimation lens's own map color will NOT change tick to tick on
     this content (Task 9's own finding), so a human running this gate should not expect it to and
     should not read a
     static Legitimation view as a failure;
   - the event feed grows, carrying BOTH event families over a long-enough run —
     `LIFECYCLE_TRANSITION` fires every tick for every county, and `ENTITY_DEATH` fires at least
     once by the tick the fixture's `last-worker` subject starves (Task 15's own conformance
     citation).
4. Pressing **Tab** three times, confirming the active-lens label cycles `PopulationTrend ->
   Tension -> Legitimation -> PopulationTrend` and the map recolors at each step — including
   confirming `Tension`'s own step shows the absence banner over an all-`PANEL` map (the expected,
   honest behavior on this demo content, not a bug) and `Legitimation`'s step shows its own
   colored-but-static band assignment.
5. Confirming the client wrote to `~/.local/share/babylon/logs/babylon-client.log` (or
   `$XDG_DATA_HOME/babylon/logs/`) after the run.

**The CI-safe proxy — everything from step 3 except the human's own eyes**, since no CI runner has
a display server or GPU (the same CI reality every headless test in this plan already works
inside):

```rust
// tests/eyes_on_smoke.rs — the scriptable half of the eyes-on gate. Never
// a replacement for the human pass above (nothing here can see a color on
// screen), but it proves every INPUT-DRIVEN, hash-provable claim that
// pass makes, so a future change that silently breaks the loop reds THIS
// gate in CI before a human ever has to notice by eye.
use babylon_graph::state_hash::CanonicalState; // trait import — .state_hash() below needs it in scope
use bevy::asset::AssetPlugin;
use bevy::prelude::*;
use std::collections::HashSet;

#[test]
fn five_space_presses_advance_five_distinct_ticks_and_fire_both_packs_events() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), InputPlugin));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.update(); // Startup

    let mut hashes = HashSet::new();
    for _ in 0..5 {
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::Space);
        }
        app.update();
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(KeyCode::Space);
        }
        let session = app.world().resource::<babylon_client::engine_link::EngineSession>();
        hashes.insert(session.inner.graph().state_hash().expect("hash"));
    }
    assert_eq!(hashes.len(), 5, "five presses, five distinct hashes");

    let session = app.world().resource::<babylon_client::engine_link::EngineSession>();
    assert!(
        session
            .sink
            .events
            .iter()
            .any(|(name, _)| name == "EventType/LIFECYCLE_TRANSITION"),
        "the event feed must carry lifecycle's own emitted events"
    );
    assert!(
        session
            .sink
            .events
            .iter()
            .any(|(name, _)| name == "EventType/ENTITY_DEATH"),
        "the event feed must carry vitality's own emitted events too — \
         proving both packs actually ran, not just lifecycle"
    );
}

#[test]
fn defaults_to_population_trend_and_tab_cycles_through_all_three() {
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), InputPlugin));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.update();

    // Task 8's finding: Tension has zero data on this demo content, so the
    // app must not default to it.
    assert_eq!(
        *app.world().resource::<babylon_client::map::ActiveLens>(),
        babylon_client::map::ActiveLens::PopulationTrend
    );

    let mut seen = vec![*app.world().resource::<babylon_client::map::ActiveLens>()];
    for _ in 0..3 {
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.press(KeyCode::Tab);
        }
        app.update();
        {
            let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
            input.release(KeyCode::Tab);
        }
        seen.push(*app.world().resource::<babylon_client::map::ActiveLens>());
    }
    use babylon_client::map::ActiveLens::{Legitimation, PopulationTrend, Tension};
    assert_eq!(
        seen,
        vec![PopulationTrend, Tension, Legitimation, PopulationTrend],
        "three presses from the default must visit every lens once and return to start"
    );
}

#[test]
fn a_known_demo_county_actually_recolors_after_a_space_press() {
    // THE test the MEDIUM-HIGH finding asked for: real `TickLoopPlugin` +
    // `MapPlugin` together, no hand-installed `CurrentLensData` (contrast
    // Task 10 Step 4's own test, which deliberately hand-builds a fixture
    // to test `recolor_on_lens_changed`'s LOGIC in isolation — this test
    // proves the real app's WIRING reaches the mesh at all, which a
    // hand-installed resource cannot prove by construction). Before this
    // test existed, every automated check in this plan passed even in a
    // build where `CurrentLensData`/`FipsIndex` never resolved and the map
    // never recolored — this closes that gap.
    let mut app = App::new();
    app.add_plugins((MinimalPlugins, AssetPlugin::default(), InputPlugin));
    app.add_plugins(babylon_client::map::MapPlugin);
    app.add_plugins(babylon_client::loop_ui::TickLoopPlugin);
    app.update(); // Startup — real EngineSession, real CurrentLensData, real MapSurface.

    fn county_zero_colors(app: &App) -> Vec<[f32; 4]> {
        let surface = app.world().resource::<babylon_client::map::MapSurface>();
        let meshes = app.world().resource::<Assets<Mesh>>();
        let mesh = meshes.get(&surface.fill_mesh).expect("fill mesh is registered");
        let (start, end) = surface.tessellation.county_vertex_range[0]; // atlas index 0 = DEMO_FIPS[0]
        match mesh
            .attribute(Mesh::ATTRIBUTE_COLOR)
            .expect("fill mesh carries per-vertex color")
        {
            bevy::mesh::VertexAttributeValues::Float32x4(colors) => {
                colors[start as usize..end as usize].to_vec()
            }
            other => panic!("unexpected color attribute shape: {other:?}"),
        }
    }

    // Tick 0: PopulationTrend is the default lens, and every county reads
    // `Some(0.0)` (now == baseline, nothing has ticked yet) — DIM.
    let before = county_zero_colors(&app);

    {
        let mut input = app.world_mut().resource_mut::<ButtonInput<KeyCode>>();
        input.press(KeyCode::Space);
    }
    app.update();

    // Atlas index 0 is a `core` (×0.95) family county — Task 9b's own
    // table has this family net-DECLINING, so after tick 1 it must read
    // CRIMSON, genuinely different from tick 0's DIM.
    let after = county_zero_colors(&app);

    assert_ne!(
        before, after,
        "the demo county at atlas index 0 must actually recolor after one Space press — \
         if this fails, CurrentLensData is not reaching the mesh even though the tick itself \
         advanced (check that advance_on_space's ResMut<CurrentLensData> param and its three \
         lens.rs calls are wired, and that recolor_on_lens_changed's Res<MapSurface> resolves)"
    );
}
```

- [x] **Step 1:** Write all three tests as shown (the two original plus
      `a_known_demo_county_actually_recolors_after_a_space_press`, the MEDIUM-HIGH fix's automated
      color-change proof), run against Phase C/D's finished code → FAIL until those phases land
      (this task sits last deliberately).
- [x] **Step 2:** Once Phase C and Phase D land, all three PASS. `mise run rust:check` → green.
      The third test is the ONE place in this plan's automated suite that exercises
      `TickLoopPlugin` and `MapPlugin` together with zero hand-installed resources — if it fails
      while the other two pass, the bug is in the real wiring between them, not in either
      plugin's own isolated logic. (It DID fail first: `before == after`, exposed a missing
      `.after(advance_on_space)` ordering constraint on `recolor_on_lens_changed`/`refresh_hud` —
      fixed in `loop_ui.rs`, see that file's own comment.)
- [x] **Step 3:** Closed #262 via a comment citing this plan document (issue was already CLOSED,
      superseded by ADR186 — added the concrete gate definition rather than re-closing). `ai/state.yaml`'s
      B2 entry consolidated into Task 19's own state.yaml step (same file, same edit, once the PR
      number is known) rather than duplicated across both tasks.
- [x] **Step 4: Commit** (`test(client): the B2 eyes-on gate + its CI-safe proxy (B2)`).

### Task 19: Gates, docs, PR

- [x] **Step 1:** `mise run rust:check`'s recipe substituted per the executing agent's standing
      instructions (scoped `cargo test` per crate instead of `cargo test --workspace`, run
      single-flight for machine safety): `cargo fmt --all -- --check`, `cargo clippy --workspace
      --all-targets --locked -- -D warnings`, `RUSTDOCFLAGS='-D warnings' cargo doc --workspace
      --no-deps`, plus the pedantic legs for `babylon-kernel`/`babylon-bsl` — ALL green. `mise run
      check` → green (13833 passed, 49 pre-existing skips, 1 pre-existing xfail, zero Python
      production files touched).
- [x] **Step 2:** `mise run qa:regression` → 11/11 scenarios byte-identical + the two-process
      determinism leg (E5b). `mise run qa:vault-regression-ci` → byte-identical (`single_county`,
      two independent bakes, zero drift). Confirms Phase A's Task 1/Task 4 touches to
      `babylon-bsl`/`babylon-tick` moved nothing, and that Phases C-E (client-only) moved nothing
      either.
- [x] **Step 3:** `cargo test -p babylon-bsl -p babylon-tick -p babylon-client` → every test result
      green (babylon-bsl 463, babylon-tick 82, babylon-client 88 — 73 lib + 15 integration across
      8 test files) — all five phases green together. **Counts corrected (adversarial-panel
      finding FB2)**: this step's own first pass under-counted (461/66/86 across "6" integration
      files) — the true counts, verified by both test-runner totals AND raw `#[test]` attribute
      greps (two independent methods, cross-checked), are the ones above; the gap traces to a
      miscount in commit `f748a5dd`'s own body, propagated from there into this line and into
      `ai/state.yaml` until this fix round caught and corrected both.
- [x] **Step 4:** `ai/state.yaml`'s `recently_completed` list gained a new entry (Program 28 B2,
      citing this plan document and PR #504). GitHub project board's client lane: no existing board
      item found corresponding to "B2 client lane" specifically (searched project 8's item list) —
      left for the Director rather than guessing at board curation; the issue trail (#503 filed,
      #262 commented) carries the record instead. Follow-up issue #503 filed for the three items
      this plan's own sections defer (the Phase 3 anchor-resolution registry, unbounded
      event-feed memory, the economics BSL port/Tension-lens reversion condition) — cited in the PR
      body per the B1 Task 12 precedent.
- [x] **Step 5:** PR #504 opened (`feat(client): B2 — the tick loop on screen, two packs deep`),
      body carrying: the Task 7 Step 1 FIPS table, the pinned multi-rule conformance output (Task
      5), the pinned determinism-guard output, gate evidence, every recorded plan/reality
      mismatch, and a link back to this plan document. The eyes-on human pass is flagged **pending
      Director** in the PR body (no display server in this environment) rather than self-merged —
      merging goes through the verification + ADR181 protocol separately, per the executing
      agent's standing instructions overriding this step's own "self-merge on green" default. NOT
      merged by this agent.

---

## Open questions for the Director

**All three ruled, 2026-08-11, interactive batch — full record.** This document's Amendment
record (top of file) quotes the rulings verbatim and applies them throughout the task list above; this
section keeps each original question's full reasoning intact per the Documentation philosophy's
immutability-of-history discipline, and appends what the ruling actually changed.

1. **Does the new Legitimation lens need its own sign-off, the way ADR191 R11 ruled the Tension
   lens's four bands? — RULED: APPROVED, reuse the band palette.** This plan's original reasoning
   for proceeding without escalating: the Legitimation lens invents no new formula (it colors a
   categorical field the `lifecycle` rule pack already computes), uses only already-declared §9b
   palette tokens, and is additive to — never a replacement for — the Director-ruled Tension lens
   (a Tab key switches between them, both visible, neither hidden). The Director ruled rather than
   letting the recommendation stand by default, selecting: *"CRISIS → crimson, UNSTABLE → dim
   gray, STABLE → gold's absence (panel dark) — reuses the ruled four-band vocabulary so the two
   lenses share one visual language. No new colors enter the game."* Task 10 carries this exactly —
   `legitimation_band_color` maps `{0.0: PANEL, 1.0: DIM, 2.0: CRIMSON}`, replacing this plan's
   first-cut `GREEN_DARK`/`GOLD` invention outright. Task 11 carries the consequence: STABLE and
   "no data" now share a map color on purpose, so the HUD's literal text is the only channel that
   distinguishes them.
2. **Should B2 defer multi-rule-pack sessions, or does "watch state change" implicitly want BOTH
   Material Base systems visible at once? — RULED: MULTI-RULE DRIVER FIRST, this plan's
   recommendation OVERRULED.** This plan's original recommendation was to defer, run `lifecycle`
   alone, and file an issue — reasoning: `lifecycle` alone, of the three merged packs, has a
   subject type matching the map's own unit, so it demonstrates the criterion fully on its own; the
   technical wall (`E-LOAD-001`, `split_content`'s "exactly one rule" cardinality check) is real
   engineering work, not a flag flip. The Director selected the larger option instead, quoted:
   *"Build the multi-rule content-set evolution into B2 itself so the demo runs vitality+lifecycle
   together from day one. Bigger B2, later criterion-3 close, but a richer first demo."* This
   ruling is what reshapes the entire plan above: Phase A gained five tasks (2, 3, 3b, 4, 5) widening
   `split_content`, `prepare_rules`, `TickReport`, and proving a conformance vector against the
   frozen engine's own vitality-then-lifecycle tick order; Phase B's demo scenario grew from
   twelve territory nodes to eighteen (twelve territory + six social-class); Phase D's event feed
   and Phase E's eyes-on gate both now assert on BOTH packs' events, not just the lifecycle rule
   pack's. The Multi-Rule Decision section (above the File Structure table) is the full design this
   ruling required, including the finding — checked, not assumed — that vitality's own subject type
   has no territorial binding, so the "richer demo" ruling 2 asked for is genuinely richer in the
   event feed and the state panel, not on the map surface itself, which only the lifecycle rule
   pack paints.
3. **Audio (SFX/soundtrack, ADR152/153) — RULED: DEFERRED out of B2, per this plan's own
   recommendation.** No change from the first cut: R3's visual scope names "2D map game + panels
   and charts as the primary surface" with no audio obligation; wiring 39 SFX + 13 tracks properly
   is real, separately-scoped work, and B2 already carries five phases (now twenty-one tasks,
   counting Task 3b and Task 9b) of new surface.

**Two more, surfaced and ruled in the third revision round (2026-08-11, same interactive
session).** The BLOCKER 2 fix (the Population Trend lens) made two presentation calls this plan's
author took under ruling 1's own authority ("presentation constants, no ceremony follows") rather
than formally reopening as questions — a verification round flagged that as an overreach: ruling 1
settled the LEGITIMATION lens's palette specifically, not a blanket license for every later lens
this plan invents. This section reopens both explicitly and records them RULED, D97/ADR194
citation discipline (quoted verbatim as presented and selected):

4. **Does the Population Trend lens's GOLD/CRIMSON sign-only mapping need its own sign-off? —
   RULED: APPROVED AS DESIGNED.** This plan's own reasoning for the choice: a strict sign
   comparison invents no size threshold, so it reads as machinery under the same "no imposed
   functional forms" standing this plan already cites for the Legitimation lens, not a new
   presentation ruling requiring escalation. The Director ruled on it directly rather than letting
   that reasoning stand unconfirmed, selecting: *"GOLD = growth, CRIMSON = decline, sign-only (no
   invented threshold — compliant with your no-imposed-forms line). One shared visual vocabulary;
   the lens picker + HUD label carry which meaning is active."* No change to Task 10's
   `population_trend_band_color` — the ruling confirms the design this plan already shipped, and
   records the Director's own reasoning (matching the no-imposed-forms line explicitly, not merely
   by this plan's own say-so) alongside it.
5. **Should the app default to `ActiveLens::PopulationTrend` instead of `Tension`? — RULED:
   APPROVED, WITH A REVERSION CONDITION.** This plan's own reasoning: Task 8's finding that the
   Tension lens has zero data on this demo scenario (no `v`/`s`/`e` economic fields declared
   anywhere) means defaulting to it would open the app on an absence banner, so `PopulationTrend`
   — the one lens guaranteed to carry real, moving data — became the default. The Director ruled:
   *"The app opens on the lens that actually has data and moves; your Tension lens becomes the
   default the moment real economic content lands (recorded as the reversion condition)."*
   **THE REVERSION CONDITION, recorded explicitly per the ruling's own instruction:** the STARTUP
   default returns to `ActiveLens::Tension` the moment a scenario in this content set declares real
   `v`/`s`/`e`-shaped economic fields for its territory nodes AND a rule pack writes them per tick
   (the same "genuinely live, not merely declared" bar Task 9's finding already applies to
   Legitimation) — most plausibly when the deferred economics BSL port (named in Task 19's own
   follow-up list) lands. `PopulationTrend` stays the default until a future task satisfies that
   condition; this is not a standing aesthetic preference, but a fact about which lens the demo
   content can honestly support, and the reversion condition is the plan's own record of when that
   fact changes.
