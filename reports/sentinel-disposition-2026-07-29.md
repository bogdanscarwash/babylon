# Sentinel-estate disposition table — Program 27 Phase 0 (§6.3)

**Task:** Program 27 Phase 0, Task 7. **Consumed by:** the Phase-3 port plan
and the Phase-4 cutover gate (spec `docs/superpowers/specs/2026-07-28-
program-27-refoundation-design.md` §6.3, §10). **Cutover blocks on this table
being green** — every `(b)` row is scheduled Phase-3 work, not a follow-on.

## Method

`src/babylon/sentinels/` was read module-by-module: every family's `__init__.py`
docstring (the declared invariant), its `registry.py`/`checks.py` shape, and —
where the docstring named one — its founding incident. LOC is `wc -l` over every
`*.py` file under the family's directory (registry + checks + `__init__.py` +
any extra module). Classification follows spec §6.3's three buckets:

- **(a) subsumed by the type system** — the error class cannot occur in
  idiomatic Rust/BSL; the subsumption argument names the specific type rule.
- **(b) ported** — the invariant is real and language-independent, but nothing
  in Rust's own toolchain (`rustc`, `cargo`, `clippy`) proves it; a
  purpose-built analyzer (syn-based Rust-AST walker, `cargo metadata`, or a
  BSL-AST walker) must re-implement the same static rule against the new
  source.
- **(c) survives as git-level tooling** — the invariant is about the
  development process (commit hygiene, ceremony), not the engine's source
  language; it needs no port because it was never engine-language-specific.

## Repo-count reconciliation (read before the table)

`src/babylon/sentinels/` contains **25** per-invariant directories, not the
spec's stated 24 — verified directly (`find … -maxdepth 1 -type d | wc -l` →
25). The file/LOC totals still match the spec's headline number exactly:
**84 `.py` files / 20,642 lines** = the 25 per-family directories (18,432
lines) **+** the 7 shared-infra modules living directly under
`sentinels/` (`_ast.py`, `base.py`, `dynamic.py`, `exemptions.py`,
`__init__.py`, `report.py`, `_rust.py` — 2,210 lines) that every family
imports but that are not themselves a family (no registry, no declared
invariant — `base.py` is the shared gating/advisory runner, `_ast.py` the
shared AST-helper library, `_rust.py` the Rust-source text-extractor
`tutorial_coverage` uses, `exemptions.py` the one shared `SentinelExemption`
model, `report.py` the shared finding-format renderer, `dynamic.py` shared
dynamic-probe wiring). `18,432 + 2,210 = 20,642` — exact.

Given that, the most likely resolution of the 24-vs-25 count is that the
spec's "24" was written before `fog`'s promotion to a fully separate package,
or simply undercounts by one (`fog/` is genuinely anomalous: unlike every
other family it has **no `checks.py`** — its actual sensor logic lives
entirely in `tools/fog_containment_probe.py`, outside `sentinels/`; only the
exemption registry lives in-package). Rather than force an artificial merge
to hit exactly 24, this table lists **all 25** directories with their real
disposition, and flags `fog` explicitly as the row most likely elided by the
spec's count. Every disposition argument below stands on its own regardless
of which number is "correct" — the Phase-4 gate should track all 25.

## The 24 (+1) families

| # | Family | LOC | Invariant (from docstring) | Disposition | Subsumption argument / port target |
|---|---|---|---|---|---|
| 1 | `absence` | 722 | Every sqlite connect call site carries a cited disposition (readonly/write); backslide + staleness checked. | **(b) ported** | A mandatory typed connection-mode enum narrows the bug (compiler forces *some* disposition to be named) but doesn't subsume it: whether the *declared* disposition still matches the *actual* open mode (backslide) and whether a registered file still has a live connect call (staleness) are facts about call-site text, not types. Port target: syn-based Rust-AST walker re-implementing the same three rules (registered/backslide/staleness) over `rusqlite`/`sqlx` connect call sites. |
| 2 | `aggregation` | 433 | (i) fog-masked partial-coverage rollups: all-`None` inputs must produce an honest `None`, never a fabricated `0.0`. (ii) intensive-mean scanner: a rate/ratio/share must never be averaged unweighted across space (ADVISORY). | **(a) subsumed** | Directly named in spec §6.3: "the extensive/intensive lexicon becomes real types." `Extensive<T>` exposes only `.sum()`; `Intensive<T>` exposes only `.weighted_mean(weights)` — no bare `.mean()` exists to call by mistake. The fog-masked-`None` half is subsumed the same way: aggregation over `Option<T>` via `?`/`Iterator::sum::<Option<_>>()` propagates `None` on any masked input; there is no `default=0.0` fallback to write accidentally. |
| 3 | `assumptions` | 290 | Every declared economics simplification (flat employment default, national-series-as-county-proxy, etc.) cites a real file; static existence-only check. | **(c) survives as git-level tooling** | The check is `Path.exists()` over a cited-file registry — it never parses code structure, never depends on the engine's language, and (like the baseline-ceremony gate) is really a documentation/governance artifact, not an engine invariant. It ports trivially as a language-agnostic script (the registry itself, currently Python literals, could become a YAML ledger read by any tool) and needs no BSL/Rust-AST analyzer. |
| 4 | `conservation` | 176 | Per-tick accounting identities over a dense trace: `economic_columns_finite` (no NaN/inf) and `imperial_rent_pool_depletion` (non-negative, bounded, non-increasing). | **(b) ported** | A numeric property over a runtime trace, not a static-source claim — no type rule can prove "this column is finite for all 520 ticks" or "this reserve never increases." Port target: a Rust proptest/property-law suite (per the behavioral-contracts doctrine — this IS one of the "golden baseline/property law" artifacts CLAUDE.md names as the durable spec) walking the dense trace the same way `tests/unit/sentinels/test_conservation.py` does today. |
| 5 | `coupling` | 408 | Declared dependency edges (`_DEFAULT_COUPLINGS`) must match real code dependencies, both directions (declared-but-absent, present-but-undeclared). | **(b) ported** | A dependency-graph reachability claim over source text; `cargo`'s crate-dependency graph sees crate boundaries, not BSL-rule-to-BSL-rule read/write coupling inside one crate. Port target: syn-based (or BSL-AST) dependency scanner reading which rule reads which opposition's published symbol, diffed against the declared catalog — same two-directional check, new source language. |
| 6 | `coverage` (incl. `catalog`/`db_probe`) | 1,576 | Every reference-data-dependent computation names its source class in a declared map (`DATA_REQUIREMENTS`), and each declared source class must exist; separately, `data-catalog.yaml`'s per-table registry (Program 21) must resolve to real extractors/consumers. | **(b) ported** (static coherence half); **unaffected / stays Python** (DB-probe half) | The static "declared source class actually exists at its module path" check is a straightforward syn/`cargo metadata` symbol-existence proof once the source classes live in Rust/BSL. The DB-probe / nightly-parquet-subset half doesn't need a Rust port at all: the reference-data build pipeline (parquet → sqlite) is explicitly staying Python per the Constitution's Ledger design — only the *consumer* of that data moves language. |
| 7 | `dangling` | 719 | A dynamic `getattr(x, "name", default)` call site must resolve to a real member of `x`'s declared type/protocol. | **(a) subsumed** | The canonical case: Rust has no `getattr`-by-string, no duck typing. `x.name` where `name` isn't a real field of `x`'s type is a **compile error**, full stop — the entire error class (a call site referencing a member that was renamed/removed elsewhere) cannot be constructed in idiomatic Rust. |
| 8 | `defines_passthrough` | 435 | A formulas-layer call must thread the run's live `defines` through; an optional `defines` param silently falls back to schema defaults when omitted at the call site. | **(a) subsumed** | Make the parameter non-optional (no `Option<&Defines>` with a `Default` fallback — a plain, required `&Defines` argument). The compiler then rejects any call site that omits it; there is no way to "forget" the argument and silently get schema defaults instead of the run's live coefficients. |
| 9 | `domain_sync` | 817 | PostgreSQL `CREATE DOMAIN … CHECK` clauses (numeric + format domains) must not drift from their single source of truth (`babylon.models.types` bounds / registry format specs). | **(b) ported** | The DB schema is not itself Rust code — this stays a cross-language sync problem (SQL migration vs. the language-side type bounds) regardless of engine language. Port target: an xtask/build-script sensor that re-derives the expected `CHECK` predicate from the Rust newtype bounds (a `const`/attribute the sensor reads via `syn`) and diffs it against the committed migration's parsed `CHECK` body — same shape, new source reader. |
| 10 | `fog` | 67 | Every `POLITICAL_FIELDS`/`ORG_POLITICAL_FIELDS` member on an out-of-reach node must be masked to `None` (Hypothesis property test; only the exemption list lives in-package, the actual check lives in `tools/fog_containment_probe.py`). | **(b) ported** | A runtime property (hundreds of generated `(field-subset, value, node-id)` cases) over the fog-filter function, not a static-source claim. Port target: a Rust proptest generating the same shaped cases against the Rust fog-mask function; the declared exemption list ports as plain data. *(Most likely the row the spec's "24" count elides — see reconciliation note above; it is the only family with no `checks.py` of its own.)* |
| 11 | `formula_registration` | 529 | A formula registered in `FormulaRegistry` must be referenced by real production code *outside* its own registration line (the `inert` family's blind spot: registration itself counts as a reference under `inert`'s rule). | **(b) ported** | Same reachability shape as `inert`, with one file explicitly excluded from counting as a reference — a syn/BSL-AST reachability scan over the crate with the registry module carved out, mirroring today's exact exclusion. |
| 12 | `gate_coverage` | 247 | The `qa:regression` scenario estate must declare (statically) and prove (dynamically) complete coverage over every engine System — the U9 "gate ran green over a dead feature" incident. | **(b) ported** | Needs to enumerate the real system list from source (today: AST over `simulation_engine.py`'s `_DEFAULT_SYSTEMS`) and cross-reference the scenario-coverage declarations. Port target: `cargo metadata`/syn read of the Rust system-registry/tick-order construct, paired with a dynamic probe (mirrors the existing static/`-truth` dynamic split). |
| 13 | `inert` | 898 | A declared store/producer must be reachable from real production code (three rules: writer has a caller; producer has a reference; no undeclared accumulator-shaped class exists). | **(b) ported** | Explicitly the spec's own worked example of what a compiler *cannot* do: "inert-detection must cross the BSL content boundary (cargo's `dead_code` lint cannot see a rule no content file references)" — a store can satisfy `dead_code` (it has *a* caller, e.g. from a test or a BSL content file the compiler treats as opaque data) while still being unreachable from the real 34-system tick. Port target: syn-based Rust-AST walker + a BSL-content reachability walker (reads BSL rule files as the analyzer's own second source, since `rustc` doesn't). |
| 14 | `liveness` | 450 | A declared producer/output must have a declared, real reader (`correct-but-inert` / `computed-but-never-consumed`; ADVISORY only). | **(b) ported** | Same reachability shape as `inert`/`unconsumed`, already generalized once by `seam_algebra`'s ambient-graph/live-subgraph construction — ports as an instance of that same BSL-AST-driven live-subgraph builder rather than a bespoke second walker. |
| 15 | `masked_arithmetic` | 506 | Arithmetic on a fog-masked field must be explicitly guarded (`dict.get(key, default)`'s `default` never fires on a *present*-but-`None` key). | **(a) subsumed** | The bug shape requires a dict with a sentinel default silently substituting for an already-present `None`. In Rust the field is `Option<T>`; there is no arithmetic operator on `Option<T>` without an explicit `.unwrap_or(..)`/`match`/`?` — the compiler forces the guard to exist at every site, or the code doesn't compile. |
| 16 | `partition` | 320 | The seeded-vs-derived class-evidence crosswalk (cell vocabulary → `SocialRole`) stays a single source of truth (Program 19, ADR070). | **(a) subsumed** | Same shape as `vocabulary`'s type half: a closed Rust enum for "cells" matched with an **exhaustive** `match` (no wildcard `_ =>` arm permitted in the crosswalk function) — adding a cell without adding its role mapping is a compile error, not a runtime AST finding. |
| 17 | `roundtrip` | 191 | `WorldState.from_graph(state.to_graph())` conserves a declared set of core node fields byte-for-byte (the tick-52 FIPS-drop incident). | **(a) subsumed** | The bug requires two independent representations (a Pydantic model and a dict-shaped graph payload) that can drift. If the Rust graph node *is* the canonical struct (single `#[derive(Serialize, Deserialize)]` type, no separate dict intermediate), the round trip is structurally total for every field by construction; the "known-lossy transient fields" become explicit `#[serde(skip)]` annotations at the field, self-documenting in the type instead of tracked in an external registry. |
| 18 | `seam` | 4,247 | The Seam Observatory: every player-observable quantity is registered (`SEAM_REGISTRY`) and covered by three sensors — continuity, dead-liveness, false-provenance — across the engine → bridge → frontend boundary. | **(b) ported** | The single largest family by LOC and the spec's own "durable seam" (`observe()` projection contract survives the client rewrite even though the client doesn't). Port target: re-target all three sensors at the new boundary (engine → BSL `observe()` projection → Rust client), reading the registry + Rust/BSL-AST instead of the Python bridge. Largest single Phase-3 item. |
| 19 | `seam_algebra` | 2,046 | The unified co-Heyting-boundary computation (ambient graph *G*, live subgraph *L*, seam = ∂L) generalizing `inert`/`unconsumed`/`coupling`/`liveness`/`vocabulary`/`dangling`'s shared "declared construct reached by real production code" claim into one graph. | **(b) ported** | Second-largest family by LOC; since several of the six siblings it generalizes are individually subsumed by the type system (`dangling`, partially `vocabulary`), the ported remainder is a leaner ambient-graph/live-subgraph builder over whichever siblings still need dynamic reachability (post-subsumption) — one BSL-AST graph builder reused across those, rather than six duplicated walkers. |
| 20 | `superstructure` | 284 | Only a register's declared owner file may write it (I-ORD: superstructure acts on the base only through the next tick, ADR135). | **(a) subsumed** | Rust's module-privacy system enforces exactly this: a setter visible only as `pub(in path::to::owner_module)` makes a write from any other module a **compile error**, not a runtime AST-scan finding — the "declared owner set" becomes the visibility annotation itself. |
| 21 | `surface` | 243 | A package's public surface (`__all__`) must not drift from a separately pinned baseline frozenset (the scoped-test blind spot: a symbol export unpinned by any single scoped run). | **(a) subsumed** | The bug requires two independent lists (the export list and its baseline copy) that can fall out of sync. Rust has no equivalent second list: visibility (`pub`) is declared once, at the item's definition, not synced by hand into a separate `__all__`-like baseline — the drift shape this sentinel exists to catch cannot occur. (A `cargo public-api`-style semver-surface diff remains available as optional (c) git-level tooling if API-stability tracking is wanted later, but it is not required to close this bug class.) |
| 22 | `synthetic` | 488 | Every sanctioned mock/fallback data source names itself AND its guard in a declared registry; both symbols must actually exist. | **(b) ported** | Same existence-proof shape as `coupling`/`inert`: syn-based Rust-AST scan confirming both the cited source symbol and its cited guard symbol are still defined at their declared paths. |
| 23 | `tutorial_coverage` | 481 | Every `(surface, key)` keybar hint in the Rust client must be exercised by an authored tutorial step or carry a cited exemption; the option-universe itself gates on a non-zero floor. | **(b) ported** | Already half-ported today: this family reads `rust/crates/babylon-tui/src/views/keybar.rs` as **text** via the shared `_rust.py` extractor (the M7 cutover's own workaround for "no Rust toolchain in the sentinel lane"). Port target: replace the line-oriented regex/state-machine text reader with a real `syn`-based (or BSL-AST) parse of the same keybar declarations — same floor-gated logic, no more text-scraping. |
| 24 | `unconsumed` | 493 | A computed value can have a real production *caller* (satisfying `inert`) while the *value it returns* is written once and read by nothing. | **(b) ported** | Same blind spot as `inert`'s in Rust terms: a struct field can be constructed and even read *once* (satisfying `dead_code`) while never being read by real downstream production logic — no compiler lint proves "was this value's *result* ever consumed," only "was the *symbol* ever referenced." Port target: BSL-AST/syn reachability scan for a declared field with zero non-test read sites, same registry-driven shape. |
| 25 | `vocabulary` | 1,366 | The graph node-type vocabulary AND node/edge *shape* stay closed — six rules (a–f): no invented type strings; every queried type has a producer; every stamped attribute is real declared shape; edge-source-type fabrication; read-side of the shape rule; FIPS/H3-grain inversion. | **SPLIT: (a) + (b)** | Spec's own headline example: "closed enums subsume vocabulary's type half but NOT its reachability half." Rule (a) (no invented type strings) is **(a) subsumed** — `NodeType` as a closed Rust enum makes stamping an invented variant a compile error. Rules (b)–(f) (every *queried* type/attribute/edge-source has a real *producer* in production code — a reachability claim, not a legality claim) are **(b) ported** — a legal enum variant can still be one nothing in production ever constructs; only a syn/BSL-AST producer-reachability walker (identical in spirit to `inert`'s) can see that. |

## Summary counts

| Disposition | Count | Families |
|---|---|---|
| **(a) subsumed** (pure) | 8 | `aggregation`, `dangling`, `defines_passthrough`, `masked_arithmetic`, `partition`, `roundtrip`, `superstructure`, `surface` |
| **(a) subsumed** (one half of a split family) | 1 | `vocabulary` (type half only) |
| **(b) ported** (pure) | 14 | `absence`, `conservation`, `coupling`, `coverage` (static half), `domain_sync`, `fog`, `formula_registration`, `gate_coverage`, `inert`, `liveness`, `seam`, `seam_algebra`, `synthetic`, `tutorial_coverage`, `unconsumed` |
| **(b) ported** (one half of a split family) | 1 | `vocabulary` (reachability half, rules b–f) |
| **(c) survives as git-level tooling** | 1 | `assumptions` |
| **Unaffected / stays with the Python data pipeline** | 1 (sub-scope) | `coverage`'s DB-probe half — the reference-DB ETL is explicitly Python-side per the Constitution, independent of this rewrite |

25 families total (LOC: 18,432 across the 25 directories + 2,210 shared infra
= the spec's 20,642 / 84 files). **8 pure + 1 half = ~8.5 of 25 close for
free** once the type system carries the invariant; **14 pure + 1 half = ~14.5
of 25** are real, language-independent claims that need a purpose-built Rust/
BSL analyzer in Phase 3; **1 of 25** (`assumptions`) was never an engine
concern and needs no port at all.

## Phase-3 work estimate per (b) family

Ordered by LOC (a rough proxy for port effort — the existing Python
implementation is the closest available spec for the ported behavior):

| Family | LOC | Estimated Phase-3 effort |
|---|---|---|
| `seam` | 4,247 | Large — 3 sensors × new boundary (`observe()`/Rust client), largest single item; needs a mutation-validated proof per sensor per spec §6.3. |
| `seam_algebra` | 2,046 | Large — ambient/live-graph builder, but shrinks once its type-subsumed siblings (`dangling`, `vocabulary`'s type half) drop out of its live-set scope. |
| `coverage` (static half) | ~900 (of 1,576; catalog/db_probe partly out-of-scope) | Medium — symbol-existence proof over the new source tree; the YAML catalog itself doesn't move. |
| `inert` | 898 | Medium — three rules, BSL-content-boundary crossing is the hard part (spec's own named hard case). |
| `domain_sync` | 817 | Medium — cross-language (SQL ⟷ Rust-type-bounds) re-derivation, same shape as today. |
| `absence` | 722 | Medium — three static rules over `rusqlite`/`sqlx` call sites. |
| `unconsumed` | 493 | Small-medium — one static rule, registry-driven. |
| `synthetic` | 488 | Small-medium — one static rule, registry-driven. |
| `tutorial_coverage` | 481 | Small — mostly a `syn` swap-in for the existing regex text-reader; logic already proven. |
| `liveness` | 450 | Small — folds into `seam_algebra`'s builder rather than standing alone. |
| `formula_registration` | 529 | Small-medium — one exclusion-aware reachability scan. |
| `gate_coverage` | 247 | Small — system-list enumeration + cross-reference, both sides already declarative. |
| `coupling` | 408 | Small-medium — bidirectional dependency-edge scan. |
| `conservation` | 176 | Small — proptest port of two numeric identities over a dense trace. |
| `fog` | 67 (package) | Small — proptest port of the existing Hypothesis property; logic unchanged. |
| `vocabulary` (rules b–f only) | ~900 (of 1,366; rule (a) subsumed) | Medium — five reachability rules, same registry-driven shape. |

**No family is scheduled to block cutover by default** — per spec §6.3, a
family whose port slips either blocks cutover or degrades to a declared,
Director-signed exemption row; this table is the input Phase 3 executes
against, not the sign-off itself (the sign-off column is intentionally absent
here — it is populated during Phase 3 port work).
