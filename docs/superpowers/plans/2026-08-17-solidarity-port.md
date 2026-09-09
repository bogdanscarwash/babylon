# Implementation Plan — Solidarity @8.0 Port Train

Target: port `SolidaritySystem` (Material Base @8.0, `src/babylon/engine/systems/solidarity.py`,
203 lines) into a BSL rule pack `rust/crates/babylon-tick/content/rules/solidarity.bsl`.

Authored 2026-08-17 against `dev` @ `17971664`. Upstream archaeology:
`scratchpad/checkpoint-a/archaeology-solidarity.md` (re-verified here; **five corrections** in §3).
Binding law: Constitution v3.1.0 (Amendments AE/AF), **ADR183** (port-as-is; the frozen engine is a
structure/ordering contract, not a byte oracle), ADR172 ruling 5 (no imposed functional forms),
ADR204 (W1/W11 ternary unification), ADR207 (the W10 port handoff precedent), ADR208 R14
(Checkpoint A).

Place in the schedule: Solidarity is one of the **remaining seven** Material Base systems.
ADR208 R14 defines "MATERIAL BASE COMPLETE" as all 13 ported in Rust (6/13 today: vitality @1.0,
territory @2.0, production @3.0, lifecycle @7.0, dispossession @10.0, metabolism @13.0). This train
takes it to 7/13 and is the cheapest remaining one — its language-level blocker is already cleared.

---

## 1. The content-model ruling (the plan's hardest call — READ FIRST)

**RULED: `ideology.class_consciousness` ports to the existing `social-class/revolutionary` field.
No `social-class/class-consciousness` field is minted.**

The digest left this open and flagged it as "the single biggest unknown for a port train's Task 0".
It is now resolved against the tree. The chain, with citations:

### 1.1 What the landed port actually declares

Deffields live in `.bscn` scenarios, **not** in `.bsl` rule packs (`scenario.rs:385-453` dispatch
list). `consciousness-ternary-conformance.bscn:210-214` declares the class consciousness surface as:

```
  ; ---- the ternary surface (FIRST probability deffields in committed content) ----
  (deffield social-class/revolutionary probability intensive)
  (deffield social-class/liberal probability intensive)
  (deffield social-class/fascist probability intensive)
  (deffield social-class/dominant-worldview enum WorldView)
```

There is **no `class-consciousness` field anywhere in the ported estate** (grep-confirmed: the only
`class_consciousness` hits under `rust/` are the Python oracle mirror and a frozen-reference
comment). The estate has moved past the scalar.

### 1.2 The frozen engine's own relation between the scalar and the ternary

This is the citation that makes the ruling a port, not an invention. Frozen
`engine/systems/ideology.py:382-386` — a verbatim code comment in ConsciousnessSystem itself:

```python
# Route agitation through solidarity → class/nation split.
# The ternary router (Spec 043) returns shifts in (revolutionary,
# liberal, fascist). The legacy two-axis IdeologicalProfile maps
#   class_consciousness  ← revolutionary (delta_r)
#   national_identity    ← fascist       (delta_f)
```

and it is implemented at `ideology.py:410`:

```python
new_class = min(1.0, current_profile["class_consciousness"] + delta_r)
```

**The frozen engine identifies `class_consciousness` as the revolutionary axis's accumulator.**
`delta_r` — the revolutionary component of the ternary router's output — is what gets added to
`class_consciousness`. That identification is the frozen engine's, cited, not this plan's.

### 1.3 Why minting a new scalar field would be wrong

- **ADR204 W11 struck the legacy scalar** estate-wide; D146 records "The cc/ni estate and its
  read-time bridge (`projection/aggregation.py:86-98`) are **retired** (ADR204 W1/W11)". Minting
  `social-class/class-consciousness` now resurrects a struck surface.
- It would be a **dead field**: Solidarity would be its only writer and *nothing* would read it —
  the landed consciousness pack reads `social-class/revolutionary`. That is precisely the
  fixture-over-dead-feature failure mode `mise run check:vocabulary` exists to catch.
- **D152 already ported this exact quantity for this exact gate.** D152's register row
  (`bsl-language.rst`, middle column `N/A (a gate read re-pointed at the stored ternary — the W1
  unification re-home, not a BSL construct)`) re-pointed the *activation-threshold read on
  `class_consciousness`* to `social-class/revolutionary`, wording it "the same quantity post-W1
  unification". SolidaritySystem's source-side gate is the **same comparison against the same
  define** (`activation_threshold`, 0.3). Porting the gate to `revolutionary` and the write to a
  different field would be incoherent within one train.

### 1.4 What this ruling does NOT claim — the honest boundary

It does **not** claim scalar cc and the ternary are interchangeable, and it deliberately does not
unify them beyond what §1.2 cites. Two consequences must be recorded, not papered over:

1. **`TernaryConsciousness` is a different model with a different bridge.**
   `models/entities/consciousness.py:51` is the *community/organization* ternary; its
   `collective_identity` property (`:157-165`) equals `r` and is documented as "semantically
   identical to the old `CommunityConsciousness.collective_identity`", and
   `_derive_ternary_from_legacy` (`:225-271`) bridges from `collective_identity`, **not** from
   `IdeologicalProfile.class_consciousness`. Do not cite that bridge for this ruling; §1.2 is the
   citation. (It is also 75/25-style redistribution mathematics that must NOT be transcribed here.)
2. **The simplex opens between @8.0 and @17.0.** SolidaritySystem's frozen write is an
   unconstrained `[0,1]`-clamped scalar write. Writing it into `revolutionary` does **not** preserve
   `r + l + f = 1`. Ported, that means a window from position 8 to position 17 in which a class's
   ternary is off-simplex. This is tolerable and precedented, for three verified reasons:
   - **No other landed pack reads the ternary.** Grep-confirmed: `social-class/(revolutionary|
     liberal|fascist|dominant-worldview)` appears in exactly one rules file, `consciousness.bsl`.
     Nothing in positions 8–17 observes the open window today.
   - **`consciousness/p6-route`'s verbatim `normalize_to_simplex` closure heals it the same tick**
     (`consciousness.bsl:323-330`: `total`, then `r2/l2/f2` renormalize when `total > 1+eps` and
     give `l` the slack when `total < 1-eps`). D154 already records "Same-tick closure heal
     observed (D116; tv-tie-all-true)".
   - **Nothing enforces the simplex at the store.** `probability` deffields range-check each field
     to `[0,1]` independently (`E-EVAL-020`); there is no sum invariant in the substrate.

   This still gets an explicit D-record (§7, D-row 4) **and** a Director-gate question (§4.1),
   because *whether* solidarity transmission should inflate `r` pending closure or displace
   `l`/`f` is a theory question about how solidarity works, not a mechanical one — and displacing
   would require inventing redistribution mathematics the frozen engine does not have (forbidden).
   Port-as-is proceeds on the inflate path; the question is filed non-blocking.

---

## 2. The design, fully determined by the tree

### 2.1 ONE rule, subject `SOCIAL_CLASS`

Frozen iterates all SOLIDARITY edges regardless of endpoint type, then reads
`class_consciousness_from_node(src_attrs)`. **Organization sources are always skipped** — verified:
`Organization.ideology` is a **`str`** field ("Marxism-Leninism", `organization.py:389,395`), so
`class_consciousness_from_node` falls through its `isinstance(ideology, dict)` check and returns
`0.0` (`node_access.py:31-36`); `0.0 <= 0.3` ⟹ `continue`, every tick, always. So unlike
`consciousness.bsl` (which needs both `p2-org-solidarity-push` and `p3-class-solidarity-push`),
**this port needs exactly one rule with one subject type.** Record as a verified narrowing.

### 2.2 The push idiom is mandatory — `(edges ...)` would be a bug

`for-each` iterates a query result **within one subject's effect list**; the engine loops the
subject population (`tick.rs:8-18`). A rule body written `(for-each (edges EdgeType/SOLIDARITY) …)`
therefore runs **once per social-class subject**, processing every edge N times and multiplying
every write by the class count. (`edge-write-lane-e2e.bscn` proves the point from the other side:
it needs a `social-class/shape 1` discriminator on a single `writer` node precisely to make an
iterate-all-edges rule fire exactly once — a fixture hack, not production content.)

**The digest is wrong on this point** (§3, correction 1). The rule uses the D136 push idiom, exactly
mirroring `consciousness.bsl:243-245`:

```
(for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS) …)
```

Each edge is pushed exactly once by its unique source. `:out` matches frozen's
`source_id → target_id` direction.

### 2.3 `set`, not `add` — and the clamp is load-bearing, not decoration

Verified semantics: `PendingWrite` carries an op + operand; `apply_pending_write`
(`structural_verbs.rs:1027-1047`) reads `current` from the graph **at apply time** and computes
`current + write.operand` for `Add`. So `add` *accumulates* across multiple pushes to one target;
`set` clobbers (last-write-wins in subject order).

`add` is nonetheless **wrong here**, for a hard reason: `social-class/revolutionary` is declared
`probability` — a unit-interval type — and a store landing outside `[0,1]` is **`E-EVAL-020`, a
tick-fatal range violation, NEVER a clamp** (`structural_verbs.rs:1690`: "outside the target
field's declared domain is `E-EVAL-020` — never a clamp"). The frozen engine clamps
(`solidarity.py:164-165`: `max(0.0, min(1.0, target + delta))`). A clamp is expressible only on a
computed result, i.e. via `set`. **Transcribing the frozen clamp verbatim is what keeps the write
lawful** — omit it and the rule is tick-fatal on any transmission that would overshoot 1.0.

There is no `min`/`max` scalar intrinsic (only fold aggregates) and **no `abs`** — grep-confirmed.
Both must be `if`-expressed, exactly as `consciousness.bsl` does throughout
(`(if (< x 1) x (- 1 0c))`, `dispossession.bsl:361-364`'s "the same trick recurs at every clamp").

### 2.4 The accepted trade-off, stated plainly

Single-rule `set` preserves **per-edge events** (frozen emits one `CONSCIOUSNESS_TRANSMISSION` per
applied edge) and the **verbatim per-transmission clamp**, at the cost of last-write-wins when two
sources target one class. The alternative — a two-rule split staging deltas into an `(add)`
accumulator, then applying the clamp once — would restore cumulative accumulation but **collapse N
per-edge events into one per target**, breaking the event contract. Under ADR183 port-as-is, the
verbs and events transcribe faithfully and the multi-inbound divergence is D-recorded. **This is
the core design decision of the train; do not silently re-litigate it during implementation.**

### 2.5 Reads a neighbour's field — a first for the estate

The delta needs the *target's* current value, so the rule uses
`(field-of it social-class/revolutionary)`. `field-of` over a `NodeRef` is served
(`evaluator.rs:1278-1292`, slice 1). **This makes Solidarity the first rule pack to read another
node's field**, which makes the collect-then-apply pre-state semantics observable for the first
time — exactly what `tick.rs:52-55` warned was latent: "Verified byte-neutral for every rule pack
landed at the time of the repair — none reads another node's field, so the divergence was
unobservable **until a rule does**." That sentence is this train's headline D-record.

### 2.6 Rule sketch (shape only — the implementer authors the real text)

```
(rule solidarity/p0-transmit
  :material-basis "…"
  :fuel <computed, see Task 2 step 4>
  (bindings
    (binding active :field social-class/active :optional :default 1)
    (binding r :field social-class/revolutionary :optional :default 0.0p)
    (binding threshold  :const solidarity/activation-threshold)
    (binding negligible :const solidarity/negligible-transmission)
    (binding awakening  :const solidarity/mass-awakening-threshold))
  (when (and (= active 1) (> r threshold)))
  (effects
    (for-each (neighbors self EdgeType/SOLIDARITY :out NodeType/SOCIAL_CLASS)
      (guard (and (= (field-of it social-class/active) 1)
                  (> (field-of (edge-between EdgeType/SOLIDARITY self it) solidarity/strength) 0))
        (guard <|delta| >= negligible>
          (update-node it social-class/revolutionary (set <clamp01 (target + delta)>))
          (emit EventType/CONSCIOUSNESS_TRANSMISSION …)
          (guard <old < awakening AND new >= awakening>
            (emit EventType/MASS_AWAKENING …)))))))
```

`delta = strength * (r − target_r)`. Nested `guard` is served — it recurses through
`execute_item`/`collect_items` and takes `1..n` effect items
(`structural_verbs.rs:407-423, 820-832`). `emit` inside `guard` inside `for-each` is precedented
(`dispossession.bsl:399-405`).

**Ordering:** rules run in ascending **rule-id byte order** (`lib.rs:310`), not by frozen system
position; and each conformance test `include_str!`s exactly the pack(s) it wants
(`run_once(scenario_src, rule_src)`, `lib.rs:72`). So the @8.0 position is a **header-comment
convention only** and no cross-pack ordering arises in this train. Follow the landed convention:
document `Material Base @8.0` in the pack header, citing `solidarity.py:91` and
`simulation_engine.py:298`.

### 2.7 Coefficients (verified against `defines.yaml:182-187`)

| defconst | value | frozen source |
|---|---|---|
| `solidarity/activation-threshold` | `0.3c` | `activation_threshold`, `config/defines/consciousness.py:23-28` |
| `solidarity/mass-awakening-threshold` | `0.6c` | `mass_awakening_threshold`, `:29-34` |
| `solidarity/negligible-transmission` | `0.01c` | `negligible_transmission`, `:35-39` |

`scaling_factor` (0.5) and `superwage_impact` (1.0) are declared on the same `SolidarityDefines`
model but have **zero call sites** in `solidarity.py` — do not declare them.

---

## 3. Corrections to the archaeology digest (verified; each changes a task)

1. **§6/§9's "it can stay a direct `for-each (edges EdgeType/SOLIDARITY)` transcription" is WRONG.**
   That shape runs once per subject and multiplies every write by the class count (§2.2). The push
   idiom is mandatory.
2. **§9's "Conformance-world needs … `edge-write-lane-e2e.bscn` or
   `consciousness-ternary-conformance.bscn` … could seed a new Solidarity-specific `.bscn` by
   copy-adapt" understates the work, and the D-record-1 rule-count guess is settled at ONE**
   (§2.1 — org sources are provably always skipped, verified via `Organization.ideology: str`).
3. **§9's event caveat is stale in the part that matters.** "`TickReport` still has no event-log
   carrier (WS1/#502), so emits are expressible but **unpinnable** by a conformance golden" — the
   *golden-hash* half is true (`tick_goldens.rs` covers nodes/attributes/edges/hyperedges, never the
   event log) but the *conformance* half is false: `CollectingSink`
   (`structural_verbs.rs:76-94`) captures `Vec<(String, Vec<(String, Value)>)>`, and
   `vitality_conformance.rs:191-219` pins event type **and full ordered payload** with exact
   `Value::NodeRef`/`Value::Real` equality. **Both events get pinned in this train** (Task 3), not
   deferred.
4. **§7's D-record 2 is right in its conclusion but not its reasoning.** It attributes
   last-write-wins to collect-then-apply alone; in fact `add` *would* accumulate at apply time
   (§2.3). What forces `set` — and therefore the divergence — is the unit-interval `E-EVAL-020`
   store law plus the frozen clamp. The D-record must say so, or a future reader will "fix" it to
   `add` and get a tick-fatal overshoot.
5. **§7's "only the port work itself remains undone" misses a required Rust change.**
   `"solidarity"` is **absent** from the `systems` HashSet (`babylon-tick/src/lib.rs:221-271`); a
   rule under that namespace fails `E-LOAD-002`. Unlike `consciousness.bsl` ("this pack changes no
   Rust source"), this pack needs a one-line registration. The neighbouring `"social-class"` entry's
   comment confirms the train was anticipated: *"NOT a system port: T2 ships no Solidarity content
   (Solidarity's PORT is a separate Wave C train)"*.

---

## 4. Blockers and gated items

**No blocker stops this train.** The Phase-1 inventory's named blocker (BSL query-evaluation
Slice 2) is cleared: `SERVED_QUERY_HEADS = ["nodes", "neighbors", "edges"]`
(`evaluator.rs:546`), `eval_edge_between` and `field_of_edge` are implemented and exercised by
`consciousness.bsl`. Everything the rule needs — `neighbors`, `edge-between`, `field-of` over both
`NodeRef` and `EdgeRef`, nested `guard`, `emit` with payload, `(edge-attr ...)` seeding — is served.
No `ai/decisions/ADR202`/`ADR208` ruling mentions solidarity (grep-confirmed).

Three items to raise rather than plan around:

### 4.1 Director-gate (non-blocking, file as an issue; do NOT block the train)
Does periphery→core solidarity transmission **inflate** the revolutionary share pending
`p6-route`'s same-tick simplex closure (this plan's port-as-is path, §1.4), or should it
**displace** liberal/fascist share so the simplex never opens? The latter requires redistribution
mathematics the frozen engine does not have — inventing it is forbidden by ADR172 ruling 5 and by
this train's port-as-is mandate. Ideological surface (bifurcation/consciousness theory) ⟹ Director's
reserved line per CLAUDE.md. File it; proceed on inflate.

### 4.2 Task-1 spike: `solidarity/` const-namespace collision risk
`solidarity/strength` is an **implicit** edge-field qname (D32 — declaring it is `E-LOAD-001`, and
`load_edge_attr` refuses any `/strength`-suffixed field outright, `scenario.rs:1500-1600`). This
plan also proposes `solidarity/activation-threshold` etc. as **defconsts**. `load_defconst`
(`scenario.rs:517-545`) parses a qname into a separate `consts` map with no namespace validation,
and rules resolve `:const` and `:field` through different lookups — so the two should not collide.
**Verify by loading, not by reasoning** (Task 1 step 2). If any `E-LOAD-*` fires, fall back to
reusing the landed `consciousness/`-prefixed const names (`consciousness/
solidarity-activation-threshold`, `consciousness/negligible-transmission` already exist at
`consciousness-ternary-conformance.bscn:236-237`) and record the reason.

### 4.3 Task-2 spike: fuel bound under inline repetition
There is **no per-iteration binding form** — `bindings` are per-subject, so every per-target
quantity (`(field-of it …)`, `(field-of (edge-between …) …)`, and `delta` itself) repeats inline
several times (§2.6). `:fuel` is per-subject and the bound checker proves the rule fits against
scenario-derived `ceilings` (built from node/edge type counts, `lib.rs:200-220`), so the static
bound scales with seeded edge count. Compute the declared `:fuel` from the checker's own error
message rather than guessing. If the bound proves impractical, **stop and report** — do not
silently restructure into the two-rule staging split, which breaks the per-edge event contract
(§2.4).

---

## 5. Global constraints (binding on every task)

- **ADR183 port-as-is.** Transcribe the frozen computation, gates, clamp and events. No
  refactors, no new mathematics, no imposed functional forms. Where the port must diverge, it is a
  D-record, not a redesign.
- **TDD, red phase first.** Every task below opens with a failing test. Never write rule content
  before the assertion that pins it.
- **The oracle is the dual implementation, not frozen floats** (ADR183 + D146 precedent). The
  Python mirror is authoritative for expected values; the frozen engine is the structure/ordering
  contract.
- **Zero drift on existing pins.** New `tick_goldens.rs` pins are **additive** — a new
  `#[test] fn solidarity_conformance_hashes_are_pinned()` with its own `include_str!` consts. Each
  pin is self-contained (no shared array), so touching an existing hash is a red STOP.
  Pins are **measured, never derived** (`tick_goldens.rs:21-23`): run once, read `hex(&report.…)`
  back, paste.
- **`tests/baselines/**` must not change.** This train adds Rust content only; it touches no Python
  engine path. If `qa:regression` or `qa:vault-regression-ci` moves, **STOP** — that is evidence of
  an unintended coupling, not a ceremony to perform.
- **No `EventType` vocabulary work needed.** `EventType` is an opted-out kind in production content
  (kind-checking stays inert when undeclared — `lib.rs:563-592`), so
  `EventType/CONSCIOUSNESS_TRANSMISSION` and `EventType/MASS_AWAKENING` load without declaration.
  Emit names are bare (`"ENTITY_DEATH"`, not `"EventType/ENTITY_DEATH"`) at the sink.
- **Machine safety.** Single-flight heavy runs. Never fan out parallel pytest/cargo.
- **Commit per task** with `mise run commit -- "type(scope): msg"`; verify HEAD moved.

---

## 6. Tasks

Each task is one implementer dispatch. Gates are cumulative: a task is done when its own gate is
green.

### Task 1 — Register the namespace and declare the conformance world

**Goal:** a loadable `solidarity` namespace and a `.bscn` world whose graph shape produces every
witness the later tasks assert on.

1. **RED.** Create `rust/crates/babylon-tick/tests/solidarity_conformance.rs` with the `run()`
   helper copied verbatim from `vitality_conformance.rs:94-99` (`MemoryGraph` + `CollectingSink` +
   `run_once_into`), plus one test asserting the world's node/edge counts and that the scenario
   loads. It must fail (no `.bscn`, no `.bsl`).
2. **Spike §4.2 first.** Add `"solidarity".to_owned()` to the `systems` HashSet
   (`rust/crates/babylon-tick/src/lib.rs:221-271`), with a comment in the established style of the
   `"production"`/`"metabolism"` entries citing this train. Then confirm a trivial
   `solidarity/`-namespaced rule with a `solidarity/`-namespaced defconst loads. Record the verdict
   (and any fallback per §4.2) in the `.bscn` header.
3. Author `rust/crates/babylon-tick/content/scenarios/solidarity-conformance.bscn`. Build it
   **fresh** (copy the *header discipline* and declaration idiom from
   `consciousness-ternary-conformance.bscn`; neither candidate world is reusable as-is — see §8).
   Declare:
   - `(defvocabulary NodeType (SOCIAL_CLASS))`, `(defvocabulary EdgeType (SOLIDARITY))`
   - `(deffield social-class/revolutionary probability intensive)` — **the same qname and type the
     landed pack uses**; deffield registries are scenario-local, so no collision.
   - `(deffield social-class/active int intensive)` — the 0/1 latch convention
     (`vitality-conformance.bscn:20-22`; `field_of_node` returns `Value::Real` unconditionally, so
     `(= (field-of it social-class/active) 1)` is the only expressible liveness read).
   - the three defconsts of §2.7.
   - **Do NOT deffield `solidarity/strength`** — it stays implicit (D32); seed it via the
     `(edge EdgeType/SOLIDARITY <from> <to> <strength>)` 4th slot.
   - The four witness groups of §8.
4. **GREEN.** Counts assert.

**Gate:** `cd rust && cargo test -p babylon-tick --test solidarity_conformance --locked`
**Commit:** `feat(tick): register the solidarity namespace + declare the conformance world (#<N>)`

### Task 2 — The transmission rule: gates, delta, clamp, the write

**Goal:** `solidarity/p0-transmit` producing the exact post-tick `revolutionary` value on every
witness target.

1. **RED.** Extend `solidarity_conformance.rs` with per-witness assertions on
   `social-class/revolutionary` after one tick — values hand-computed from
   `delta = strength * (source_r − target_r)`, `new = clamp01(target_r + delta)`, and pinned as Rust
   literals. Cover: the plain transmission, each of the three skip gates (`strength <= 0`,
   `source_r <= 0.3`, `|delta| < 0.01`), the inactive-source and inactive-target skips, and the
   clamp's upper bound.
2. Author `rust/crates/babylon-tick/content/rules/solidarity.bsl` per §2.6 — one rule. Header
   follows the landed convention: frozen source + `Material Base @8.0` (citing `solidarity.py:91`,
   `simulation_engine.py:298`), the D-row pointer lines, and the §2.4 trade-off recorded in prose.
   Every `:material-basis` cites the frozen file:line it transcribes.
   - Transcribe the formula's own dead-in-practice guard copy
     (`formulas/solidarity.py:10-36`) port-as-is, per the digest's §4.
   - `if`-express the clamp and `abs` (§2.3).
3. **GREEN.** Values assert.
4. **Fuel (§4.3):** set `:fuel` from the bound checker's error message, not a guess. Report if
   impractical; do not restructure.

**Gate:** `cd rust && cargo test -p babylon-tick --test solidarity_conformance --locked`
**Commit:** `feat(tick): solidarity transmission rule — diffusion into the revolutionary share (#<N>)`

### Task 3 — The two event emits, with payloads pinned

**Goal:** `CONSCIOUSNESS_TRANSMISSION` on every applied transmission; `MASS_AWAKENING` only on an
upward crossing of `0.6`.

1. **RED.** Assert `sink.events` — length, ordering, and **full ordered payload** with exact
   `Value::NodeRef`/`Value::Real` equality, in the shape of `vitality_conformance.rs:191-219`.
   - `CONSCIOUSNESS_TRANSMISSION` payload mirrors `solidarity.py:177-185`: `source-id`, `target-id`,
     `delta`, `solidarity-strength`, `source-consciousness`, `old-target-consciousness`,
     `new-target-consciousness` (BSL kebab-case of the frozen keys).
   - `MASS_AWAKENING` payload mirrors `solidarity.py:195-200`: `target-id`, `old-consciousness`,
     `new-consciousness`, `triggering-source`.
   - Pin the **negative** case too: a transmission that raises consciousness but does not cross
     `0.6` emits exactly one event.
2. Add both emits to the rule. The awakening condition transcribes the frozen **chained
   comparison** `old_consciousness < mass_awakening_threshold <= new_consciousness`
   (`solidarity.py:190`) as two ANDed inequalities — note the asymmetry (`<` then `<=`) and get it
   right; it is the difference between firing and not firing exactly at `0.6`.
3. **GREEN.**

**Gate:** `cd rust && cargo test -p babylon-tick --test solidarity_conformance --locked`
**Commit:** `feat(tick): consciousness-transmission + mass-awakening emits (#<N>)`

### Task 4 — Dual-implementation oracle and the golden pins

**Goal:** the bit-exact Python mirror plus additive hash pins.

1. Author `rust/crates/babylon-tick/content/scenarios/solidarity_conformance.py` — a
   **standalone, dependency-free** script (no pytest, imported by nothing) that mirrors the rule's
   binding order term-for-term over a literal `WORLD` dict matching the `.bscn`, and prints a
   fired-count table plus per-node values using `repr()`. Template:
   `content/scenarios/vitality_conformance.py` (191 lines) — the simpler of the two; the
   consciousness mirror (578 lines) is the heavyweight pattern.
2. Run it and paste its **exact stdout** into `solidarity_conformance.rs`'s `//!` module doc, with
   the regen command documented verbatim in the doc comment:
   `uv run python rust/crates/babylon-tick/content/scenarios/solidarity_conformance.py`.
   Reconcile every Rust literal against that block; any mismatch is a real defect in one of the two
   implementations — find it, do not adjust the assertion to match.
3. Add `solidarity_conformance_hashes_are_pinned()` to
   `rust/crates/babylon-tick/tests/tick_goldens.rs`: new `include_str!` consts, `run_once`,
   `assert_eq!` on `hex(&report.before)` / `hex(&report.after)` / `report.fired`, each with a
   message saying what a move would mean. **Measure the hashes by running once and reading them
   back.** Do not touch any existing pin.

**Gate:** `cd rust && cargo test -p babylon-tick --locked` (the full crate — proves the 8 existing
pins are byte-identical alongside the new one)
**Commit:** `test(tick): dual-implementation oracle + golden pins for the solidarity pack (#<N>)`

### Task 5 — Records and close-out

**Goal:** the permanent record. Per ADR207's precedent, all records land in the **last commit batch
of the last PR**, never split across PRs.

1. **D-records** in `docs/reference/bsl-language.rst` (the Draft-Ruling Register list-table opens at
   line 4717; three columns `# / Section / Ruling`, widths `8 30 62`). Highest row today is
   **D156** — allocate the next free numbers **at PR-open time**, per the file's own rule
   ("is the next free number at the time of allocation"). Rows to file (§7).
2. **ADR** — next free is **ADR209**. Mirror ADR207's structure: `status`, `date`, `title`,
   `context`, `decision` (numbered sections: scope landed with file:line evidence; Director-gate
   rulings; controller rulings; the D-row roster; **corrections to the digest's framing** — §3's
   five; chartered follow-ons), `consequences`, `supersedes: []`, `related:`. Add the
   `ai/decisions/index.yaml` entry in the exact tail format (`title` single-quoted full sentence,
   `status: accepted`, `date`, `file`).
3. **`ai/state.yaml`** — insert one `current_focus` list item at the top (most-recent-first,
   around line 3659) naming issue #, PR #s, ADR209, a dense one-paragraph summary, and a closing
   `Gates: …` sentence naming every gate and its verdict. Update the Material Base tally to
   **7/13** and note Checkpoint A (ADR208 R14) remains held.
4. **File the follow-ons:** the §4.1 Director-gate issue, and a note on the §2.5 first-observable
   collect-then-apply divergence for WS4's D-record adjudication ceremony (#502).
5. Run `vale` on every prose file touched.

**Gate:** the full definition-of-done set (§9).
**Commits:** `docs(bsl): land D<NNN>-D<NNN> register rows (#<N>)` ·
`docs(decisions): ADR209 — the Solidarity @8.0 port train handoff (#<N>)` ·
`docs(state): Solidarity port closing entry (#<N>, ADR209)`

---

## 7. D-records to file (Task 5)

Numbers allocated at PR-open time; D156 is the current high-water mark.

1. **The scalar→ternary re-point.** `ideology.class_consciousness` ports to
   `social-class/revolutionary`; no scalar field is minted. Cite `ideology.py:382-386` (the frozen
   engine's own `class_consciousness ← revolutionary (delta_r)` mapping) and `:410`, plus ADR204
   W1/W11, D146's cc/ni retirement, and D152's precedent re-point of the same gate. Middle column:
   `N/A (a write re-pointed at the stored ternary — the W1 unification re-home, not a BSL
   construct)`.
2. **Multi-inbound-edge last-write-wins** — the genuine behavioural divergence. Frozen applies each
   delta sequentially against the previous write; the port collects all writes against the same
   pre-tick graph (`tick.rs:41-52`) and `set` makes the last one win in subject order. **Record the
   reason correctly (§3 correction 4):** `add` would accumulate at apply time, but the frozen clamp
   plus the `probability` field's `E-EVAL-020` store law (never a clamp) force `set`. Quantify with
   the frozen oracle `TestSolidaritySystemEdgeCases::test_multiple_solidarity_edges`
   (`tests/unit/engine/systems/test_solidarity_system.py:347-392`): two 0.3-strength edges from
   sources at 0.9 and 0.8 into a target at 0.1 — frozen yields **0.478** (0.1 → 0.34 → 0.478);
   the port yields **0.31** (deltas 0.24 and 0.21 both computed against 0.1; last wins).
3. **First rule to read another node's field.** Solidarity makes the collect-then-apply divergence
   observable for the first time; cite `tick.rs:41-55`'s own "unobservable until a rule does".
4. **The open simplex window @8.0 → @17.0** (§1.4 item 2), with the three verified mitigations and
   the §4.1 Director-gate reference.
5. **Org-sourced solidarity edges are provably inert for this system** (§2.1) — verified via
   `Organization.ideology: str` ⟹ `class_consciousness_from_node` returns `0.0` ⟹ the 0.3 gate
   always skips. Hence one rule, not two.
6. **Target liveness must be seeded.** Frozen defaults absent `active` to `True`
   (`solidarity.py:127-130`); `(field-of it social-class/active)` on an unwritten attribute is an
   honest-null error (§3.5), so the ported content must seed `active` on every class. Narrowing, not
   a divergence in behaviour on declared content. (The subject-side read keeps frozen's permissive
   default via `:optional :default 1`.)
7. **`scaling_factor` / `superwage_impact` not declared** — declared on `SolidarityDefines` with
   zero call sites in `solidarity.py`.

---

## 8. The conformance world (Task 1 detail)

**Neither candidate world is reusable.** `edge-write-lane-e2e.bscn` (4 nodes, 2 SOLIDARITY edges)
declares no consciousness surface at all and uses a `social-class/shape` discriminator this port
must not imitate (§2.2). `consciousness-ternary-conformance.bscn` has the right *shape* (classes,
3 SOLIDARITY edges at 0.4p/0.5p/0.9p) but its values are tuned for the routing law, and reusing it
would entangle two packs' fixtures. Build fresh; copy the header discipline and declaration idiom.

Witness groups required (the digest's §8 clone trap is the thing to avoid — the `debs`/`bernie_valve`
electoral goldens seed `solidarity_strength=0.4` but their targets are construction-time clones of
their sources, so `delta == 0.0` exactly and the negligible gate skips every edge at tick 0):

| # | Witness | Shape |
|---|---|---|
| 1 | plain transmission | source `revolutionary` above `0.3`, target below, **differing** — nonzero delta |
| 2 | `MASS_AWAKENING` crossing | target starting just below `0.6`, delta pushing it to/past `0.6`; plus a sibling that rises but stays below (the negative case), and one landing exactly at `0.6` (the `<=` arm) |
| 3 | the three skip gates | `strength = 0`; source at/below `0.3`; a source/target pair whose gap makes `|delta| < 0.01` |
| 4 | multi-inbound divergence | two sources → one target, mirroring the frozen fixture's shape (§7 row 2) so the D-record's 0.478-vs-0.31 numbers are *executed*, not asserted in prose |

Plus an inactive source and an inactive target (the `active = 0` skips), and a clamp witness whose
`target + delta` would exceed `1.0` — the case that would be tick-fatal without the verbatim clamp
(§2.3). Every strength literal should be an exact dyadic rational so expected values are exact in
binary64 (the `edge-write-lane-e2e.bscn` discipline: `0.5`, `0.25`, `0.125`).

---

## 9. PR structure and definition of done

Mirroring ADR207's two-PR split (declaration/read surface, then the law plus all records):

- **PR A — Tasks 1–2.** `feat(tick): Solidarity @8.0 port — namespace, conformance world, and the
  transmission rule (#<N>)`
- **PR B — Tasks 3–5.** `feat(tick): Solidarity @8.0 port — events, oracle, and the permanent
  records (#<N>, ADR209)`

**Issue:** none exists (`gh issue list --search solidarity --state all` — grep-confirmed clean).
Open one first, titled per the landed Wave-A precedent (`#565` "Wave A: Production @3.0 port
train", `#566`): **`Wave C: Solidarity @8.0 port train`**, citing umbrella `#557` and ADR208 R14.

**Per-task gate:** the scoped cargo test named in each task.

**Final gate, before each merge:**

```bash
mise run rust:check              # fmt + clippy -D warnings -D cognitive_complexity + workspace tests + doc
mise run check                   # lint + format + typecheck + test:unit
mise run qa:regression           # MUST be byte-identical — a move is a red STOP, not a ceremony
mise run qa:vault-regression-ci  # single_county + org_probe, byte-identical
```

`tests/baselines/**` must be untouched, so **no §6.5 ceremony is owed**. If one appears to be, stop
and investigate the coupling.

**Merge:** per ADR181 — verify every check completed and `headRefOid` == the green run's `headSha`;
**harvest the Copilot review** (every inline comment gets a fix or a reply; zero unaddressed is a
merge precondition); then merge with `mise run pr:merge -- <N>`, the one sanctioned path. Never
`gh pr merge --auto`.
