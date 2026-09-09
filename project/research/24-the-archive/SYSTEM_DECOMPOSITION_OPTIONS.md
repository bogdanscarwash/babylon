# System Decomposition Options — Compiled Algebra, Determinism Budget, Postgres Maximalism, Language Modules

**Program 24 (The Archive) / system-decomposition editor synthesis.** v0.1.0 (draft for BD
disposition), 2026-07-21. Constitution v2.15.0. Editor pass over four research sections
(`tmp/sys-decomp/{01-algebra-module, 02-determinism-budget, 03-postgres-maximalism,
04-decomposition-and-nix}.md`, cited below as **[S1]–[S4]**) plus `NORTH_STAR.md` (**[NS]**),
`reports/user-interface-debate.md` Part II (**[UID]**), and
`ai/decisions/ADR106_determinism_boundary.yaml` (**[ADR106]**). Written against the BD position of
2026-07-21 (the seven-point research question: compiled algebra, Rust-for-performance/hardware/
memory-safety, Python-as-glue, Nix as a hard constraint, negotiable determinism, Postgres
maximalism incl. a bespoke extension, and "get this thing playable" as the governor).

**Most of this document touches no v1.0 lane in flight** (Capital Vol I / Vol II, T1.1, T1.2, T4, T7
— [NS §7]) — the declared exception is the two owed determinism-hygiene units in §2.2/§5: ADR106
itself assigns the `os.execv` re-exec to T1.2 (launcher), packaged by T7 (installer env), and the
wall-clock-leak sentinel to T1.1 (spine A). Both sequence *within* those lanes rather than opening new
scope outside them, but they are in-flight-lane work, not lane-free NOW items. It is staged: NOW /
post-Gate-3 / post-1.0. This is a research brief for disposition, **not** a work order, and it is
**not committed**.

### Reference register

| Tag | Source | Verified-against |
|---|---|---|
| **[S1]** | `tmp/sys-decomp/01-algebra-module.md` | `domain/dialectics/{core,instances}/*` read in full; WebFetch on Hackage/GHC/aider/haskell.nix |
| **[S2]** | `tmp/sys-decomp/02-determinism-budget.md` | `tools/regression_test.py`, `conservation_audit.py`, `.mise.toml`, pacing report, HEAD source lines |
| **[S3]** | `tmp/sys-decomp/03-postgres-maximalism.md` | `persistence/*` read; `gh api` on plrust/pgrx/age/h3-pg; `nix eval` on `postgresql17Packages` |
| **[S4]** | `tmp/sys-decomp/04-decomposition-and-nix.md` | import-linter contracts, `flake.nix`, `nix path-info -S`, `gh api` on pgrx/crane/pgGraph/supabase |
| **[NS]** | `NORTH_STAR.md` | read in full (four strata, "rigorous enough", familiarity governor) |
| **[UID]** | `reports/user-interface-debate.md` Part II | OCaml-vs-Rust algebra verdict, BD-reviewed 2026-07-21 |
| **[ADR106]** | `ai/decisions/ADR106_determinism_boundary.yaml` | determinism boundary; econophysics shape-only; owed `os.execv` re-exec; wall-clock leaks |

Editor's confidence marks are carried through from the sections verbatim: **[UNVERIFIED]** and
**[reasoned]** mean the underlying section could not freshly confirm the claim this pass (the
session's WebSearch budget was exhausted; `gh api` / `nix eval` / WebFetch were substituted where a
primary source could be targeted). Those marks are load-bearing and are **not** upgraded here.

---

## 0. Executive summary — the recommended path (ten lines)

1. **Keep the algebra in Python for v1.0.** The properties the BD wants compiler-proven —
   registered coupling endpoints, acyclic nesting, depth ≤ 4, apparatus-XOR — are already fixed in
   Python **source** (`build_default_registry()`/`_DEFAULT_COUPLINGS` are source literals, not data
   loaded from a file), but (a) Python itself has no dependent-type system to check them at compile
   time regardless of authoring shape, and (b) each opposition's `measure` is an engine-coupled Python
   callable that neither serializes to data nor crosses an FFI cleanly — so *no* compiled language
   (Haskell or Rust) can absorb these checks without either rewriting the measures too or freezing the
   catalog and breaking the data-driven Paradox Pattern [S1 §2, central finding].
2. **If a compiled algebra lane is ever opened, it is Rust — never Haskell** — via the proven
   PyO3/maturin pipeline; Haskell fails on interop direction, Nix closure, agent-codegen signal,
   and adds a genuinely new negative (laziness/space-leaks at 2 a.m.) [S1 §§3–5, S4 §b].
3. This **agrees with the [UID] verdict** ("don't adopt OCaml; Python now, Rust-if-ever") and
   **extends it to Haskell**; it **sharpens one nuance** — the first compiled slice is a *new*
   artifact, not the `hypergraph-rs` crate [§1.4 below].
4. **The honest first slice, if the BD wants to scratch the rigor itch now, is a Stratum-2 offline
   catalog-checker CLI** (Python-with-Hypothesis first; Rust or Haskell only as a personal-appetite
   choice, buying nothing over property tests) [S1 §9, NS §2].
5. **Determinism stays tiered, not blanket:** A (seed reproducibility, untouchable) / B
   (byte-identity, by-ruling) / C (presentation, free). The tutorial-BDD transcript and every
   ceremony stand on **Tier B narrator-OFF** — that is the load-bearing surface and it does not
   move [S2 §§1–2, ADR106].
6. **Do NOT relax BLAS=1 for nationwide performance** — measured evidence shows the hot loops are
   O(territory) pure-Python traversal, not BLAS-bound; relaxing it buys nothing and reopens a
   determinism hazard [S2 §§1.4, 3.2].
7. **The real performance lever is post-1.0:** deterministic FIPS-sharded execution of the ~11
   O(territory) systems, then a Rust rewrite of the same — both preserve Tier A unconditionally via
   fixed iteration/merge order, and preserve Tier B via a *new, narrow* cross-implementation
   tolerance leg already pre-authorized by Constitution III.12(b) [S2 §4 items 2–3].
8. **Postgres maximalism is mostly already realized** (window-function as-of views, advisory-lock
   DDL, list-partition purge, pgvector-in-SQL); the menu is scoped to what is *not yet* pushed
   down. One concrete Python-re-derivation anti-pattern is fixable NOW; graph-history is a recursive
   CTE over the **already-dense `edge_snapshot` table** now, Apache AGE only post-1.0 if CTE
   authoring becomes a proven pain point [S3 §§0–1].
9. **A bespoke `pgrx` extension is a verified-healthy, Nix-distributable, precedented path but no
   candidate earns its keep today** — file it as a post-1.0 research seed, not a v1.0 line item
   [S3 §1c, S4 §b Option 3].
10. **Recommended decomposition = Conservative now** (Python engine + the one paused `hypergraph-rs`
    Rust crate, resume-when-funded), with the **Full-vision variant** (algebra crate + `pgrx`
    graph extension) held explicitly as a post-1.0 seed. Rust is a **zero-marginal-Nix-closure**
    addition (already in `devShells.default`, `flake.nix:170-171`); Haskell is a new closure surface
    with no in-repo footprint [S4 §c].

---

## 1. The algebra-module decision — matrix and first slice

### 1.1 The finding that reframes the whole question

Before the matrix: **the BD's worry is misdiagnosed as a type-system gap.** [S1 §2] read the actual
algebra (`core/opposition.py` 620 lines, `core/coupling.py` 225, `core/composition.py`,
`core/galois.py`, `core/cylinder.py`, `instances/catalog.py` 766) and found the "glue" is *not*
under-typed at the layer a compiler helps with:

- The **closed enumerations** (`CouplingKind`'s 5 variants, `Flavor`'s 2, `composition`'s 3) are
  already exhaustively checked *today* by Python's `Literal` + `match` + `assert_never` under mypy —
  the identical guarantee Haskell's ADTs or Rust's enums buy, for free, with zero new toolchain
  [S1 §2.1, §2.3].
- The **graph-shaped properties** (endpoints registered, nesting acyclic, depth ≤
  `MAX_NESTING_DEPTH = 4`, apparatus-pole XOR) hold over the catalog `build_default_registry()`
  assembles — and that catalog is itself **Python source literals with callable measures**
  (`BoundOpposition(spec=OppositionSpec(key="capital_labor", ...), measure=_capital_labor_measure)`,
  `catalog.py:459-537`; `_DEFAULT_COUPLINGS` is a module-level source tuple, `catalog.py:676`), *not*
  data loaded from a config file at runtime — extended by two more reserved-key volumes not yet bound.
  So the tree **is** compile-time-authored; in principle a dependently-typed language could encode
  these checks at compile time against exactly this shape. What actually blocks moving the check to
  compile time is narrower than "it's runtime data": (a) Python itself has no dependent-type system,
  so the authoring shape is moot for *this* language regardless; and (b) each `measure`/`pole_measure`
  is an **engine-coupled Python callable** — it closes over `GraphInputs` and reads live engine
  state — so it neither serializes to plain data nor crosses a Haskell/Rust FFI boundary without a
  rewrite of the measure itself, which is the real cost a compiled-algebra move would pay. In every
  language the *shape* check is the same bounded DFS/relaxation `opposition.py` already runs — and
  already runs eagerly on every process start, already Power-of-10-bounded, already property-tested
  [S1 §2.3 pull-quote].

So the recoverable gap is **not** "types" — it is *more property tests and exhaustiveness tests
around a checker that already exists*, which is exactly the Fable-5 behavioral-contracts discipline
[CLAUDE.md] and needs no second language [S1 §8 rank 1].

### 1.2 Decision matrix (Haskell / Rust / Stay-Python)

| Axis | **Stay-Python** | **Rust** | **Haskell** |
|---|---|---|---|
| **Static guarantees** | Closed-enum exhaustiveness via `Literal`/`match`/`assert_never` (mypy) — **already shipped**; graph-shape props are runtime DFS (bounded, tested) [S1 §2.1] | Same closed-enum exhaustiveness + **typestate** ("validated-before-use" as a compile error) — one genuinely new static win [S1 §2.2] | Same closed-enum exhaustiveness; GADT/Peano depth-indexing is the one place "ill-formed states don't compile" is literally true for a compile-time-authored tree — Babylon's catalog *is* one (Python source literals), but its callable `measure`s are engine-coupled and don't port/FFI cleanly, so the GADT win would require re-authoring the measures in Haskell too, not just the shape [S1 §2.3] |
| **Python bindings** | N/A (native) | **PyO3 0.29 industrial + in-repo precedent** (`hypergraph-rs/crates/hypergraph-rs-python`); derive-macro auto-marshalling [S1 §3.1, S4 §b] | **No PyO3-equivalent, wrong direction** — `inline-python`/`cpython` embed CPython *into* Haskell; `foreign export` gives a C-ABI symbol + one-shot `hs_init`/`hs_exit` RTS lifecycle + hand-serialized marshalling [S1 §3.2] |
| **Nix** | Zero (already the env) | **Zero marginal closure** — `rustc`/`cargo` already in `devShells.default` (`flake.nix:170-171`); one `Cargo.lock` + `cargoHash` [S4 §c] | **New closure surface** — no GHC/`haskell.nix` anywhere in `flake.nix`/`flake.lock`; `haskell.nix` materialization overhead + a second binary cache (`cache.iog.io`) **[UNVERIFIED figures, reasoned from community record]** [S1 §4] |
| **Maintainability (2 a.m.)** | Highest — the language the core already speaks [NS §6.2] | Compile-time errors, actionable; no GC-timing class; in-repo exemplar to imitate [S1 §5] | **New categorical negative:** laziness/space-leaks (`foldl` thunk blow-ups) are an OOM-in-production failure mode diagnosed by heap profiling — a skill orthogonal to Rust/Python, bad to learn under pressure; strictly worse than strict-by-default OCaml [S1 §5(iii)] |
| **Agent-codegen** | Native | **On Aider Polyglot** (C++/Go/Java/JS/Python/Rust) [S1 §5(ii), verified] | **Absent from Aider Polyglot** — same gap OCaml has, freshly re-confirmed for Haskell this pass [S1 §5(ii)] |
| **Windows (AA disclosure)** | Inherits Python's story | **Tier-1 `x86_64-pc-windows-msvc`** — strongest of any candidate [S1 §7, UID W7] | GHC-via-ghcup native installer **exists** (verified); maturity tier vs Linux/macOS + Hackage Windows long-tail **[UNVERIFIED for 2026]** [S1 §7] |

### 1.3 Verdict (ranked, carried from [S1 §8])

1. **Stay Python for v1.0 / near-term.** The gap is graph-shape rigor over a catalog Python already
   checks eagerly with a bounded/tested checker, whose `measure`s are engine-coupled Python callables;
   the fix is growing the property/exhaustiveness suite — cheaper, zero Nix/FFI/marshalling cost, no
   new language.
2. **If and only when a concrete need materializes (measured performance, or a correctness property
   Python genuinely cannot express even with property tests) — extend Rust via PyO3, never
   Haskell.** Staged post-Gate-3 / post-1.0 per the BD's own directive.
3. **Do not adopt Haskell.** It fails on nearly every axis OCaml already failed on in [UID], **plus**
   the new laziness/space-leak negative — sharpening, not merely repeating, the [NS §6.2] "stack the
   BD cannot personally maintain at 2 a.m." governor.

**Reversibility:** the compiled algebra form does not exist yet; nothing here forecloses adding Rust
later at zero option cost, and Haskell stays reconsiderable *only* for a future construct set that is
**pure data with no engine-coupled callables** — i.e. one where GADT/Peano depth-indexing could check
the shape without also needing to re-host executable measures across an FFI boundary — a scenario
this pass could not find anywhere in the current algebra [S1 §8].

### 1.4 Where this AGREES and DISAGREES with the [UID] verdict — stated explicitly

The BD **deliberately reopened** the compiled-algebra question (the standing context flags [UID]'s
"algebra-stays-Python + no-OCaml" ruling as *input, not gospel*). Honest reconciliation:

- **AGREE (structural conclusion):** [UID] ruled "do not adopt OCaml; keep the Lawverian algebra in
  Python/Pydantic for v1.0; if it must become a compiled typed core later, extend Rust via PyO3."
  This document reaches the **same** conclusion for the same repo-binding reasons (interop, Nix,
  agent-codegen, three-language tax) and **extends** it to Haskell, which was not the language [UID]
  weighed. [NS §6.2] independently states the same verdict as a ratified governor. Three independent
  reads converge — **the convergence is itself the finding** [S1 §0]; do not present this as
  settling a previously-open question, it settles it a *second* time.
- **SHARPEN (reasoning, not conclusion):** [UID] framed the recovery as "capture exhaustiveness as
  property tests." [S1 §2] goes further and shows *why a compiler cannot help here at all* — but the
  reason is narrower than "there is no compile-time tree": the tree **is** compile-time-authored, in
  Python source. The actual blockers are that Python itself has no dependent types to exploit that
  authoring, and that the tree's `measure` callables are engine-coupled Python code that would have to
  be re-hosted, not merely re-typed, to move into Haskell or Rust. So the choice is not "types vs
  tests," it is "tests, in whatever language, because the executable content cannot cross the FFI
  boundary cleanly regardless of what the type system can prove about the shape." This is a stronger
  form of the same claim, and it applies to Rust as much as to Haskell — the PyO3 path avoids the
  FFI-marshalling cost for future *new* Rust code, but does not retroactively make the existing Python
  measures typeable at compile time either.
- **DISAGREE (one nuance):** [UID]'s phrase "**extend the existing Rust `hypergraph-rs` crate**" is
  imprecise and both [S1 §3.1] and [S4 §b] correct it: `hypergraph-rs` is a petgraph/rustworkx-core
  **graph-substrate** crate, *not* a Lawverian-algebra host. What is reused is the **PyO3/maturin/Nix
  pipeline and the precedent**, not the crate as a home for oppositions. So if a compiled algebra
  ever ships, it is a **new** crate (`babylon-algebra`) or — more honestly for the rigor itch — the
  Stratum-2 CLI in §1.5, *not* code hung inside `hypergraph-rs`. [UID] itself already flags this
  caveat in its own text; this document promotes the caveat to the headline.

### 1.5 Recommended first slice (if the BD wants to act now)

Per the brief's own hint — *"it may serve Stratum 2 verification rather than the hot tick path,
which changes the requirements entirely"* — and [NS §2]'s Stratum-2 definition ("sentinel families ·
seam registry · ∂L boundary computation · derivation trees · ceremony gates"):

> **A standalone, offline decision-procedure CLI.** Given a catalog (opposition specs + coupling
> edges as plain data — the shape `build_default_registry()`/`_DEFAULT_COUPLINGS` already are),
> decide: are all coupling endpoints registered? Is the nesting graph acyclic and depth ≤
> `MAX_NESTING_DEPTH`? Does every `contains` edge correspond one-to-one to a nesting binding (both
> directions)? Does every apparatus-flavor spec have a non-community pole B? Exit 0/1, JSON findings
> on stdout — run once per CI/pre-commit, exactly the existing `mise run check:vocabulary` pattern.
> [S1 §9]

This reframing **neutralizes every FFI objection** (one process-start cost, not per-tick; JSON-over-
stdio is adequate; a `dataBuild`-style side devShell, never the shipped-to-players closure). Under
it, all three implementations — **(a) Python + Hypothesis** (zero new toolchain), **(b) a Rust crate
via the PyO3 pipeline**, **(c) a standalone Haskell CLI** — are **roughly equivalent in what they can
prove**, because the thing proven is graph well-formedness over the specs/edges shape exported as
plain data for exactly this CLI, not closed-type exhaustiveness over the full engine catalog (which
still carries Python-callable measures no compiled language changes the type-safety of) [S1 §9].
**Recommendation: (a) Python-with-property-tests.** (b)/(c) buy nothing over it except personal
rigor-appetite — a legitimate but explicitly non-technical justification that must not be conflated
with a capability claim [S1 §9 close].

---

## 2. The determinism tier policy + relaxation menu

### 2.1 What the ceremonies and the tutorial-BDD transcript actually stand on (be honest)

Two columns literally named `determinism_hash` compute unrelated things — internalize this before
touching anything [S2 §1.1, ADR106]:

- `tick_commit.determinism_hash` = `sha256(session_id:tick:rng_seed)` — a **crash-recovery dedup
  key**, depends on no world state, never compared across runs (fresh UUID per run). It is **not** a
  content hash and **not** chained, despite migration `0029`'s comment [S2 §1.1, ADR106 item 3].
- `conservation_audit_log.determinism_hash` = `compute_determinism_hash()` — a **real content hash**
  (canonical sort-keyed JSON), but narrower than full `WorldState` (15 `DynamicHexState` fields) and
  **not currently wired to receive `action_list`** in the live path [S2 §1.1, ADR106 item 3].

**A chained per-tick full-state content hash does not exist** — it is a named, deferred **post-1.0**
unit [ADR106 ruling 12]. No relaxation below may assume a hash-chain fallback exists.

What actually gates drift **today**, and what the tutorial-BDD transcript + golden vault + qa gate
stand on:

- **Dense per-tick CSV goldens** — byte-identical, no tolerance, every tick/entity/edge column
  [S2 §1.2].
- **Golden vault** (`tools/vault_regression.py`, Amendment W) — sha256 per-page manifest,
  byte-identical, two in-process bakes must match each other and the committed manifest [S2 §1.2].
- **The E5b two-process leg** — two OS subprocesses with `PYTHONHASHSEED` stripped, proving the
  engine does **not** secretly depend on dict/set iteration order [S2 §1.2].
- **Runtime cost of the whole gate: 9.632s** (ADR090), over small synthetic scenarios + one
  county-scale fixture — **never nationwide** [S2 §1.2].

The transcript stands on **Tier B, narrator-OFF**. Narrator-ON is non-reproducible **by design**
(`narrative/**` excluded from every verify story) [ADR106 item 4, S2 §4 item 6]. That surface is the
one that must not move.

### 2.2 The tiers

| Tier | Definition | Enforced by | Protects |
|---|---|---|---|
| **A — Seed reproducibility** | same `(seed, defines)` → same emergent history on the *same* impl, forever (save-compat window) | `SimulationConfig.rng_seed`; Postgres `EXCEPT` row-diffs; the `TOLERANCE=1e-5` checkpoint gate | Player trust; the "rewrite test" (III.12) — validated tolerance-bounded, **not** exact bytes [S2 §2] |
| **B — Byte-identity across processes/machines** | bit-for-bit identical output for the *same* impl, no tolerance | dense CSV goldens, vault manifest sha, E5b two-process leg, `defines_hash` hard gate | Catches the ADR089/U9 inert-rate bug class; the vault presentation contract; **the tutorial-BDD transcript** [S2 §2] |
| **C — Presentation/pacing** | non-deterministic by design/exemption; never gated | narrator-ON (Ollama latency), declared `manifest.json` wallclock fields | Nothing — risk is **scope creep** (undeclared Tier-C leaks masquerading as B) [S2 §2] |

**Two Tier-boundary hygiene items are owed and NOT relaxation** — ship them before any menu item, and
note both are in-flight-lane work per ADR106, not lane-free additions: (1) the shipped-entry-point
`os.execv` re-exec (`PYTHONHASHSEED=0` + `OMP_NUM_THREADS`/`OPENBLAS_NUM_THREADS`/`MKL_NUM_THREADS`/
`NUMEXPR_NUM_THREADS=1` — **not** `RAYON_NUM_THREADS`, which ADR106 explicitly excludes from the
re-exec set as a devshell/test-only concern unless a future unit wires rustworkx centrality into the
shipped tick path; today the pin lives only in `tests/conftest.py`/`.mise.toml`/flake devshells, **none
of which ship** — so the player's binary has *no* Tier-A hash-seed pin at all), **T1.2 (launcher)
code, packaged by T7 (installer env)** [ADR106 item 2, S2 §2/§5]; (2) the wall-clock-leak sentinel
closing 3 named persisted-artifact leaks (`metrics.py:209`, `jsonl_recorder.py:90/156/181`), **T1.1
(spine A)** [ADR106 item 3, S2 §2].

### 2.3 The relaxation menu (buys / costs / mitigation / verdict)

| # | What relaxes | Buys | Costs | Mitigation | Verdict |
|---|---|---|---|---|---|
| 1 | Drop `BLAS=1`/`RAYON=1` globally | **Nothing** for the dominant nationwide cost — the ~11 hot systems are O(territory) pure-Python traversal; only dormant `vol2_circulation.py` imports numpy [S2 §3.2] | Reopens the 2026-07-12 oversubscription freeze; defeats FP-reduction-order guard at the one real parallel site (Sparrow rustworkx centrality) [S2 §1.4] | If Sparrow is a measured hotspot, **cache** structural centrality (adjacency is the immutable substrate), don't thread [S2 §4] | **REJECTED — not aimed at the real bottleneck** |
| 2 | FIPS-shard the ~11 O(territory) systems (deterministic partition + merge) | The **actual lever** — this is where measured cost lives (~15.4 s/tick marginal at true 3,153-county scale, extrapolated across an H3-hex→county unit change of unknown transfer — see §2.4) [S2 §3.1–3.3] | Nondeterministic shard interleaving would change `event_bus.publish()` order, which dense goldens pin [S2 §4] | **Fixed FIPS-sorted shard + merge order** (same discipline `_centrality_ids` already uses); preserves Tier A unconditionally, Tier B iff merge order is pinned + tested (a new cheap byte-gate row, not a relaxation) | **CONDITIONAL GO — buildable without relaxing anything; real engineering, not free** |
| 3 | Rust rewrite of the same hot systems (PyO3, `hypergraph-rs` precedent) | 10–50× on per-node Python/dict/object overhead (not FLOPs) at the same fixed FIPS order — **no Tier-A/B ordering-contract change** [S2 §4] | (a) port engineering; (b) Rust may produce different low-order bits (FMA/SIMD) vs numpy even at identical math/order — a genuine **cross-implementation** float question [S2 §4] | **Already constitutionally pre-authorized** — III.12(b) "Float honesty": cross-impl validation is tolerance-bounded checkpoint comparison with a written epsilon derivation; reuse the existing `TOLERANCE=1e-5` machinery + a Rust-vs-Python leg; dense byte-gates stay the within-Python contract, untouched | **RECOMMENDED (post-1.0)** — matches the BD's own instinct; the constitutional mechanism already exists |
| 4 | Formalize the nationwide Tier-B carve-out as a **sentinel-enforced declaration** | Closes the "declared not implicit" gap ADR090 closed for coverage — Amendment R/S already excludes nationwide from byte-identity, but only in prose | Small — one `TierDeclaration` table beside `ScenarioCoverage`/`CoverageGap` | Static-AST-checkable like `check:gate-coverage` | **RECOMMENDED (near-term, cheap)** |
| 5 | Widen tolerance-bounded comparison to more artifacts generally | Cross-machine/libm robustness where byte-identity is over-claimed | Loses sha-ceremony simplicity (a byte diff is unambiguous; a tolerance breach needs a per-artifact derivation) | Apply **narrowly** — only the Rust-vs-Python leg (item 3), not a general downgrade of the cheap, working, U9-catching byte-gates | **NARROW GO — only where item 3 needs it** |
| 6 | Narrator-ON exemption (existing) | Already free — "OFF byte-reproducible, ON not, by design" | None | The wall-clock-leak sentinel (§2.2 owed) keeps the exemption from silently widening | **STATUS QUO — close the 3 leaks, don't reopen scope** |

### 2.4 The nationwide reality (why the tension is not currently live)

Amendment R/S (Constitution IV) **explicitly excludes** the qa:regression micro-scenarios from the
nationwide mandate — they are *determinism contracts, not scale samples* [S2 §1.5]. So the BD's
worry (sacrificing determinism for nationwide playability) is **not a live CI-gate conflict today**;
it becomes one only if a future PR tries to extend byte-identity onto the nationwide artifact
(nothing does; that scenario is itself `[PENDING CODE]`). The only measured wall-clock data point is
`reports/pacing-calibration-2026-07-17.md` (~1,100 H3 **hex** territories ≈ 5.36 s/tick marginal); the
true constitutional scale is **3,153 counties** (`us_county_territories.json`, verified — note the
brief's "~3,191" does not appear in the repo). These are **different spatial partitions** — H3
hexagonal cells vs county polygons — so per-territory cost is not guaranteed to transfer between them;
the **2.87× county/H3-hex count ratio** is a unit change of unknown transfer, not a like-for-like
territory scale-up. Treating it as one anyway, as a rough sanity-check upper bound rather than a
projection, gives **~15.4 s/tick, ~22 h/full-campaign**, **never measured end-to-end** [S2 §3.3].
**First unit of any performance program: run the already-wired `per_system_ms` instrumentation over
the real 3,153-county artifact for a handful of ticks** — get a real per-system attribution before
spending any determinism-relaxation effort [S2 §3.3, §5].

### 2.5 Verdict — recommended tier policy

- **NOW:** keep A and B exactly as scoped (9.6s, U9-proven); ship the two owed hygiene units
  (`os.execv` re-exec, wall-clock sentinel) — they are more urgent than anything on the menu because
  the shipped binary's Tier-A guarantee does not currently reach the player; **do not touch BLAS=1**.
- **Post-Gate-3:** profile at real 3,153-county scale (`per_system_ms`); build item 4 (Tier-B scope
  declaration) — cheap, no relaxation.
- **Post-1.0:** item 2 (deterministic FIPS-sharding) as the engineering fix, feeding item 3 (Rust
  rewrite) as the language fix — both preserve Tier A via fixed order, Tier B via the narrow III.12(b)
  tolerance leg (item 5), never by loosening the within-Python byte gates [S2 §5].
- **Never relax:** seeded-RNG-only randomness (econophysics texts cite **shape only**, never a
  runtime sampler — [ADR106 item 1]); the tolerance-checkpoint gate as the Tier-A cross-impl
  contract; the rule that any new parallel path ships its ordering-contract test **before** merge
  [S2 §5].

---

## 3. Postgres maximalism menu — staged

### 3.1 Ground truth: the "utmost" instinct is already largely realized

Babylon is **not** a thin ORM-over-Postgres [S3 §0]: window-function **as-of views**
(`v_hex_state_asof` reconstructs a dense frame from sparse rows via `LEAD()` interval joins — this is
already stored-procedure-equivalent work in SQL), **advisory-lock DDL safety** (`ensure_ddl_applied`,
motivated by real xdist deadlocks), **list-partition O(1) purge**, **pgvector distance/top-k in SQL**,
**catalog introspection** instead of shadow indexes, and `persist_tick_atomic`'s single-transaction
`ON CONFLICT DO NOTHING` idempotent writes. Extensions loaded today: `postgis`, `vector`, `uuid-ossp`.
The menu is deliberately scoped to what is **not yet** pushed down, not a rewrite [S3 §0].

### 3.2 (a) Stored procs — PL/pgSQL vs PL/Python vs PL/Rust

| Candidate | Verdict | Stage |
|---|---|---|
| The as-of/aggregate views | **Keep as-is** — these ARE the stored-proc layer, no new runtime needed [S3 §1a] | — |
| **Live anti-pattern:** `state_choropleth_cells_from_hex_rows` re-derives in a Python `dict` what `v_state_value_aggregate`/`fetch_state_aggregate` already computes — the module's own docstring says the county tier correctly avoids this [S3 §1a] | **FIX NOW** — swap to `fetch_state_aggregate`; pure win, zero determinism/layering change (post-tick read) | **NOW** |
| PL/pgSQL for genuinely set-based hot paths | **Case-by-case, none identified yet** — reserve for SQL-shaped logic not already a Python loop working around a documented PG locking hazard (e.g. don't collapse `drop_session_partitions`' deliberate per-table transactions) [S3 §1a] | Case-by-case |
| **PL/Python (`plpython3`)** | **Reject as a rule** — runs engine-adjacent Python *inside* the DB process, inverting the "engine computes, DB stores" layering; no candidate needs it [S3 §1a] | Reject |
| **PL/Rust (`plrust`)** | **NO-GO, full stop** — verified dead: last real release v1.2.8 (2024-03), pins `pgrx =0.11.0` (current 0.19.1), caps at `pg16` (no pg17), **not in nixpkgs** [S3 §1a, verified `gh api`/`nix eval`] | Reject permanently |

### 3.3 (b) Graph-in-Postgres — AGE vs recursive CTE, and the time-travel schema (it already exists)

**The line that must not move:** Amendment L — rustworkx is *the* in-memory tick substrate; nothing
here touches it. These candidates answer a **different, read-only, post-hoc** question: "what happened
to the solidarity network historically," over already-committed/hashed history [S3 §1b].

**The "graph time-travel" data model the brief asked to design already exists.** `edge_snapshot`
(`postgres_schema.py:806-834`) is written **every tick, for every edge** — `persist_edge_snapshots`
bulk-inserts `ON CONFLICT DO NOTHING`, and `query_edge_snapshot_history` already reads it back ordered
by tick (the production edge-inspector sparkline). It is the *denser* SCD2 variant (full row per tick,
not a compressed valid-range table) — a fine trade for a local single-player game (bounded by campaign
length × edge count, Parquet-archived + session-purged). The compressed alternative — generalize the
`v_hex_state_asof` `LEAD()`-interval shape to edges — is a known low-risk move if density ever bites
[S3 §1b].

| Candidate | Fit | Stage |
|---|---|---|
| **Recursive CTE over `edge_snapshot`** (zero new deps) | Answers "solidarity network reachable from org X at tick N" today — the historical twin of `organizing_reach`'s live-graph computation; table + indexes already ship | **Post-Gate-3** — one Archive/TUI query helper |
| **Apache AGE (openCypher)** | Verified healthy — v1.7.0/PG17 (2026-02), PG18/19 pre-releases (2026-07); **in nixpkgs** (`postgresql17Packages.age` 1.7.0-rc0), one-line add to `withPackages`. Needs an explicit **materialization** load from `edge_snapshot` (III.13 status: never authoritative, rebuildable). `MATCH (a)-[:SOLIDARITY*1..5]->(b)` is more legible than a deep recursive CTE [S3 §1b] | **Post-1.0** — only if CTE authoring is a *proven* pain point; packaging is already solved |
| `ltree` | Narrow — models label-*paths*, not general graphs; the CTE baseline already covers the need | Not recommended |
| `pg_graphql` | **Reject** — a CRUD GraphQL-API generator, not graph traversal; Babylon has no external API surface [S3 §1b] | Reject |

**Division of labor:** rustworkx computes the tick (in-memory, III.7-bound); Postgres answers
questions *about* committed history, read-only, same epistemic tier as `babylon_meta` — never a hash
input [S3 §1b].

### 3.4 (c) Bespoke `pgrx` extension — feasibility

**`pgrx` is verified healthy** (distinct from dead `plrust`): 4,746 stars, pushed 2026-07-15,
v0.19.1, supports **PG13–18 + PG19beta1**; `cargo-pgrx` in nixpkgs (0.18.1); a compiled extension
rides the same `pg-runtime` `withPackages` distribution already solved for postgis/pgvector [S3 §1c,
S4 §b]. **A directly on-point live precedent exists:** `Evokoa/pgGraph` (547 stars, pushed
2026-07-21 — *today*), "graph database superpowers for existing Postgres," on `pgrx =0.19.1` with a
`rust-overlay` flake that is a near-drop-in template; `supabase/postgres` proves the
`buildPgrxExtension_*` Nix overlay at production scale [S4 §b Option 3]. **But no candidate earns its
keep today** against Constitution III.10 (Earn-Its-Keep): a `c/v/s/k` fold buys nothing (`NUMERIC`
`SUM()` is already order-independent exact-decimal); H3 helpers should use the **pre-built `h3-pg`**
(nixpkgs `postgresql17Packages.h3-pg` 4.2.3, zero Rust authorship) once a concrete SQL-side consumer
is named; dialectics gap-history is a plain `SELECT ... ORDER BY tick` [S3 §1c]. **Verdict:
credible, low-regret, in the back pocket — build ahead of no proven need.**

### 3.5 (d) `NOTIFY`/`LISTEN`

Zero hits in `src/`/`tools/`/`web/` today, and **not clearly needed** — the TUI is turn-based
(refreshes on navigation) and Constitution X.8 commits to a **single embedded process**, so there is
no concurrent-writer/reader scenario to improve on [S3 §1d]. **Speculative future fit** (honestly
flagged): if a detached-headless-runner + separate spectating-TUI is ever chartered, a
`pg_notify('tick_committed', …)` in the existing `tick_commit` insert would cheaply replace polling —
but no such feature is chartered anywhere, so this stays a note, not a roadmap item ("no invented
primitives without amendment") [S3 §1d]. **Stage: NOW nothing / Post-Gate-3 nothing / Post-1.0 only
if a detached-spectator mode is chartered.**

### 3.6 The DO-NOT-MOVE list (permanent, not staged)

**Deterministic tick math stays in Python, never in Postgres** [S3 §2]. The codebase already
documents the hazard in its own words: `compute_determinism_hash`'s docstring — *"the canonicalization
sorts hex rows by h3_index so the hash is order-independent (**Postgres SELECT order is
unspecified**)."* Three concrete arguments: **(1) plan instability** — a SQL-side hash is only as
order-safe as its `ORDER BY`, and parallel plans / index-vs-seqscan flips under `ANALYZE` / cross-
version planner changes are all outside any Babylon pin; **(2) float/serialization fork** — a
`plpgsql`/custom-aggregate hash creates a *second* serialization implementation that must forever
agree with Python's, exactly the III.12(a) anti-pattern; **(3) the correct contrast** —
`ON CONFLICT DO NOTHING` relies on guarantees Postgres *actually makes* (atomicity, constraints); a
hash relies on row *order*, which it explicitly does not [S3 §2]. **Standing guardrail:** any future
stored-proc/AGE/pgrx candidate upstream of `compute_determinism_hash` or any III.7 hash input does not
move into Postgres, full stop.

### 3.7 Staged summary

| Candidate | NOW | Post-Gate-3 | Post-1.0 |
|---|---|---|---|
| Fix `state_choropleth` re-derivation | **Yes** | — | — |
| Audit `projection/` vs 5-view registry | — | Yes | — |
| PL/pgSQL set-based hot paths | Case-by-case | Case-by-case | Case-by-case |
| PL/Python, PL/Rust | Reject | Reject | Reject |
| Recursive-CTE `edge_snapshot` graph history | — | Yes (one helper) | — |
| Apache AGE | — | — | Only if CTE authoring is a proven pain |
| `h3-pg` in `withPackages` | Add once a SQL-side H3 consumer is named | — | — |
| Bespoke `pgrx` extension | No candidate earns its keep | Revisit only if profiling names a bottleneck | Same |
| `NOTIFY`/`LISTEN` | No consumer | No consumer | Only if detached-spectator chartered |
| Tick hash / III.7 inputs | **Never moves — all horizons** | | |

---

## 4. Recommended overall decomposition — per-module language table

### 4.1 Today's enforced boundaries (the seams already exist)

The import-linter's 9 contracts (`pyproject.toml:564-645`) are a mechanically-checked one-way DAG with
`engine` as the unique sink [S4 §a]. The cleanest extraction candidates, ranked by how little a
language swap would require: **`formulas/`** (22 files / 3,947 LOC, provably pure — no
engine/graph/DB imports, only `GameDefines` + numpy/stdlib); **`domain.dialectics.core/`** (8 files /
1,518 LOC, self-documented "pure machinery… No engine imports" — this **is** the BD's algebra layer,
but 3 of 8 files are Pydantic-shaped); **`sentinels/`** (66 files / 15,869 LOC — AST-introspection-
heavy, *wants* to stay in the language it inspects, "leave it"); **`topology` behind `GraphProtocol`**
(already the seam a Rust substrate slots behind — *substrate replacement*, i.e. `hypergraph-rs`, not
new surface) [S4 §a].

**A compiled port of anything upstream of a baseline is automatically a baseline-ceremony event**
(§6.5) because FP reduction order can shift even for a faithful port — not optional process, the
existing gate firing [S4 §a].

### 4.2 Conservative variant (RECOMMENDED for the v1.0 horizon)

| Module | Language | Rationale |
|---|---|---|
| engine, formulas, dialectics, economics, projection, tui, intelligence (glue, providers, pydantic-ai) | **Python** | Stays the engine language [NS §6.2]; the algebra's rigor gap is Python's own lack of dependent types plus engine-coupled measures, not an authoring-shape gap, recovered by property tests not a compiler [§1] |
| `hypergraph-rs` (one paused Rust crate: core + `-python` PyO3 0.29 + `-cli` + `-wasm`) | **Rust** | Already exists, Phases 0–3 done (188 tests), paused mid-Phase-4 with a resume runbook; Phase 11 Babylon-swap never started. **Resume when funded**, additive to `GraphProtocol` [S4 §b Option 1] |
| scipy matrix work, pre-commit, TUI | **Python** | Per the BD's own framing [S4 intro] |

**This is the option [NS §6.2] and [UID] already ratified.** Migration cost ≈ zero beyond resuming a
paused, already-funded plan; nothing in `src/babylon/` imports `hypergraph-rs` yet, so it touches no
test/gate/sentinel today [S4 §b].

### 4.3 Full-vision variant (post-1.0 seed, held explicitly)

| Module | Language | Rationale / cost |
|---|---|---|
| `formulas/` + `dialectics/core/` → new `babylon-algebra` crate, PyO3-bound | **Rust** | ~5,465 LOC of pure math; **but** a real multi-week port + ongoing dual-language tax on the *two most-touched theory packages* (Vol I/II, market scissors, Doctrine Tree all extend these). Buys memory-safety/speed, **not** the categorical-rigor property the BD names — Rust has no GADTs either [S4 §b Option 2, §e]. Every future formula tweak becomes Rust-edit + PyO3-rebuild + baseline ceremony |
| `babylon_graph` `pgrx` extension | **Rust** | The one place a new module solves a *named* (if aspirational) BD pain — "a true graph database we can query locally." Verified, precedented (`Evokoa/pgGraph`), Nix-distributable. **Additive** (new `CREATE EXTENSION`, not a port); cost is a new flake build target + a second determinism surface (SQL functions run outside the BLAS pin — any hash-touching pgrx function needs its own determinism argument from scratch) [S4 §b Option 3] |
| `hypergraph-rs` | **Rust** | Kept unconditionally regardless of the above [S4 §b Option 3] |
| everything else (engine, orchestration, glue, TUI, pydantic-ai) | **Python** | Unchanged in kind [S4 §b Option 3] |

**Frank pains-vs-options finding** [S4 §e]: the "Pydantic jerry-rigging / rigor" complaint is *mostly
aesthetic* unless paired with the cheap Python property-test fix (Rust doesn't deliver the GADT
exhaustiveness the itch is really about); the "sentinel hand-recovery" pain gets **worse** under any
migration (Rust code is invisible to the AST sentinels — a coverage *hole* unless new Rust-side
sentinels are hand-written); **performance is real but currently unmeasured** (no ADR/profile
establishes formulas/dialectics/graph-query as a bottleneck — *get the profile first*); "memory
safety / touches hardware" **names no concrete Babylon module today** (correctly-scoped future-proofing
language); and "a true graph DB locally" is the one genuinely non-aesthetic target — a **speculative
capability** worth the post-1.0 roadmap, not a v1.0 blocker [S4 §e].

### 4.4 In-process vs loopback service (settles the BD's "Python API service" phrasing)

The BD floated "a Haskell module with a Python API service." Two different things hide there [S4 §d]:
**in-process PyO3 bindings** (what `hypergraph-rs` already is — same process, ~ns FFI overhead,
determinism solved as any pure-function call) vs **a loopback service** (a child process behind
RPC/HTTP). The repo has a working precedent for the latter — `LlamaServerSupervisor`
(`intelligence/llama_server.py`): owns a `subprocess.Popen`, loopback-only `127.0.0.1:8737`,
bounded health-poll loop, typed `ProviderUnavailable`, degrades loudly. **Verdict:** for anything on
the tick's hot path → **in-process PyO3, not a service** — a service adds a real determinism tax
(serialization-schema stability, process-warm-ordering, hidden internal state) that the hash-sealed
tick should not pay for what can be a function call. **The loopback pattern stays reserved for what it
already serves** — the optional, degradable, non-tick-blocking narrator side-channel [S4 §d].

### 4.5 Nix composition sketch (verified against this flake)

`uv2nix` (Python env — already how `flake.nix` builds) + `crane` (Rust crates,
`ipetkov/crane` v0.23.4, active) + nixpkgs' `maturinBuildHook` (PyO3 extension modules) **compose as
three overlays over one `pkgs`** — no structural conflict; `Evokoa/pgGraph`'s `rust-overlay` flake and
this repo's `uv2nix` flake are directly composable (both are `overlays` lists over the same `nixpkgs`)
[S4 §c]. Concretely verified: `rustc`/`cargo` are **already** in `devShells.default`
(`flake.nix:170-171`) — **adding Rust is zero marginal Nix-closure cost** for the toolchain (the real
cost is build time + a second `Cargo.lock`/`cargoHash` re-pin discipline, not closure bytes);
`pg-runtime = postgresql_17.withPackages (ps: [postgis pgvector])` (`flake.nix:117`) is exactly the
mechanism a `babylon_graph` pgrx extension or `h3-pg`/`age` would extend — one more `withPackages`
entry; the flake vendors **no GHC/Haskell** in any form [S4 §c]. **Closure-size deltas measured:**
`rustc-1.91.1` 1.53 GiB / `cargo` 1.58 GiB (already present, `nix path-info -S`); `pg-runtime`
marginal fetch 6.1 MiB; a GHC devshell was **[UNVERIFIED]** — no GHC output exists in-repo to price
[S4 §c].

**Windows/AA disclosure line (one line, per Amendment AA discipline):** Rust is Tier-1
`x86_64-pc-windows-msvc`; a `pgrx`/pg-extension build on native Windows is meaningfully harder (no
MSVC-native pgrx precedent surfaced this session) but is squarely a **lane-2, post-1.0, WSL2-shielded**
concern — WSL2 runs the Linux flake unchanged, so none of these options add any **pre-1.0** Windows
obligation [S4 §c].

---

## 5. Staging — NOW / Gate 3 / post-1.0

**NOW (the two determinism-hygiene items advance T1.1/T1.2/T7 in-lane; the rest touches no v1.0
lane):**
- Fix the `state_choropleth_cells_from_hex_rows` Python re-derivation → `fetch_state_aggregate`
  (mechanical, zero-risk, post-tick read, touches no v1.0 lane) [§3.2].
- Ship the two owed determinism-**hygiene** units (not relaxation) — these **advance T1.2/T7** (the
  shipped-entry-point `os.execv` re-exec) **and T1.1** (the wall-clock-leak sentinel) [§2.2],
  sequenced within those lanes rather than new scope outside them, and more urgent than any menu item
  because the shipped binary's Tier-A pin does not currently reach the player [ADR106 items 2–3].
- Build the Tier-B nationwide-scope `TierDeclaration` sentinel (item 4) — cheap, closes a real gap,
  no relaxation [§2.3].
- **(Optional, if the BD wants to act on the rigor itch):** the Stratum-2 catalog-checker CLI in
  Python+Hypothesis [§1.5] — runs in CI/pre-commit, ships nothing to players.
- **Do NOT:** touch BLAS=1; adopt any new language; build any pgrx extension; add unused PG
  extensions.

**Post-Gate-3:**
- Profile at the **real 3,153-county scale** via the already-wired `per_system_ms` (before any perf
  engineering) [§2.4].
- One recursive-CTE Archive/TUI graph-history helper over `edge_snapshot` (no new deps) [§3.3].
- Sweep `projection/` for other "Python re-derives a registered view" anti-patterns [§3.2].

**Post-1.0 (research seeds, none on the critical path):**
- Deterministic FIPS-sharded execution of the ~11 O(territory) systems (item 2), then the Rust
  rewrite of the same (item 3) with the III.12(b) tolerance leg [§2.3, §2.5].
- Resume `hypergraph-rs` Phases 4–11 (substrate swap) when funded [§4.2].
- Full-vision `babylon-algebra` crate and/or `babylon_graph` pgrx extension — **only** on a proven,
  profiled, named need [§4.3].
- Apache AGE — only if recursive-CTE authoring is a demonstrated pain point [§3.3].

---

## 6. Open BD questions (genuinely BD-level rulings only)

1. **Does the rigor itch get scratched now, and in which language?** The honest finding is that
   moving these checks to compile time would require Python to have dependent types (it doesn't) and
   the catalog's engine-coupled measures to be re-hosted across an FFI boundary (they don't move
   cleanly) [§1.1]. If you still want a compiled artifact for personal-rigor reasons, the Stratum-2
   checker CLI [§1.5] is the vehicle — but Python+Hypothesis buys the same proof. **Ruling needed:**
   ship the checker in Python (recommended), or sanction a Rust/Haskell version as an explicitly
   non-technical appetite call? (This is the one place [UID]'s ruling is deliberately reopened; the
   recommendation is that its verdict *stands*, extended to Haskell.)

2. **Is a compiled algebra/perf lane authorized to open post-1.0 at all, and is it Rust-exclusive?**
   [NS §6.2] already names Rust as the sole sanctioned compiled lane; this document concurs and adds
   the Haskell-specific rejection [§1.3]. **Ruling needed:** ratify "Rust-only compiled lane, Haskell
   excluded" as a standing decision (an ADR), closing the reopened question — or keep it open?

3. **Is the "true graph database locally" (pgrx `babylon_graph` extension) a committed post-1.0
   direction or a shelved seed?** It is verified-healthy, precedented, and Nix-distributable, but no
   candidate earns its keep today and it adds a second determinism surface [§3.4, §4.3]. **Ruling
   needed:** file as a post-1.0 research seed (recommended), or charter it now for design?

4. **Is a detached-headless-runner + separate spectating-TUI ever in scope?** This is the *only*
   thing that would justify `NOTIFY`/`LISTEN`, and it would need a constitutional amendment against
   X.8's single-embedded-process commitment [§3.5]. **Ruling needed:** dead idea, or a post-1.0
   possibility worth reserving?

5. **When the post-1.0 Rust-rewrite lever (item 3) lands, is the cross-implementation tolerance leg
   (III.12(b), a *tolerance-bounded* Rust-vs-Python check replacing within-Python byte-identity for
   the ported systems) pre-authorized, or a per-case ceremony?** The constitutional mechanism exists
   [§2.3 item 3]; the question is whether adopting it is a standing grant or needs a ruling each time
   a system is ported. **Ruling needed:** standing grant with a written epsilon-derivation
   requirement (recommended), or case-by-case?

---

*End of brief. Draft for BD disposition — not committed. Every claim above is cited to a section file
([S1]–[S4]), a repo path, or a governance doc ([NS], [UID], [ADR106]); confidence marks
(**[UNVERIFIED]**, **[reasoned]**) are carried through verbatim from the sections and not upgraded.*
