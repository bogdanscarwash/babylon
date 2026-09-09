# hypergraph-rs Master Roadmap — Phases 3–11 Planning Briefs

> **For agentic workers:** This is the bridge between the design spec's §10.3
> phasing and executable per-phase plans. **Each phase's Task 0 is: write the
> phase plan** using the `writing-plans` skill, with
> `2026-07-18-hypergraph-rs-phase-2.md` as the template (Global Constraints,
> probe discipline, task shape, vector-first TDD). Then execute via
> `subagent-driven-development` (implementer+reviewer batches, ledger
> `.superpowers/sdd/progress.md`). Do NOT skip the per-phase plan — this
> roadmap is a brief, not a plan.

**Precedent artifacts (read first):**
- Spec: `docs/superpowers/specs/2026-07-18-hypergraph-rs-design.md` (design truth; §4.7 divergence register is append-only, executable)
- Phase 0+1 plan + ACTUAL completion record: `docs/superpowers/plans/2026-07-18-hypergraph-rs-phase-0-1.md`
- Phase 2 plan (TEMPLATE — copy its structure): `docs/superpowers/plans/2026-07-18-hypergraph-rs-phase-2.md`
- SDD ledger: `.superpowers/sdd/progress.md` (worktree, gitignored)

**Established implementation patterns (reuse, do not redesign):**
1. Bipartite `StableDiGraph<NodeKind<N,E>, MembershipEdge<M>>` + IndexMap bimaps; public iteration = bimap-filter, insertion-ordered (D5); never `neighbors()` LIFO.
2. Direction = arc presence (DiHypergraph); no flags on MembershipEdge.
3. Errors: `NodeError`/`EdgeError`/`MembershipError` (thiserror); XGI raise/warn → Rust Err, binding translates in Phase 7 (D2 class).
4. Value-only ops (set_*_attributes, weighted SC) live in bounded `impl … Hypergraph<serde_json::Value, serde_json::Value, M>` blocks.
5. uid counter: bump iff `parse::<u64>()` ok (D3/D4/D11); `clear()` resets ≡ new (D10); freeze guards ALL mutators (D12); copy carries frozen (D13).
6. Conformance: probe XGI runtime → vector in `crates/hypergraph-rs/conformance/generate_fixtures.py` → regenerate (`/home/user/projects/game/babylon/.venv/bin/python crates/hypergraph-rs/conformance/generate_fixtures.py` from `hypergraph-rs/`) → replay pinning BOTH truths → unit tests → implement. Gates: `mise run rust:check` (every commit), `mise run rust:msrv` (phase end). New divergences → next D-number (D18 free at Phase 2 end) + row, same commit.
7. Commits: conventional + `Co-Authored-By: opencode <opencode@local>`; batches of 2 tasks; report to `.superpowers/sdd/p<phase>-task-<n>-<m>-report.md`; review-package script at `/home/user/.cache/opencode/packages/superpowers@git+https:/github.com/obra/superpowers.git/node_modules/superpowers/skills/subagent-driven-development/scripts/review-package`.

**XGI oracle:** `/home/user/projects/game/babylon/.venv/lib/python3.13/site-packages/xgi/` (v0.10.2); its test suite at `…/site-packages/tests/` (50 files, listed per phase below). **The 50-file conformance gate is Phase 7's** (needs Python bindings); per-phase gates are the conformance vectors.

---

## Phase 3: linalg + algorithms (spec §10.3, est. 1-2 weeks)

**XGI sources:** `linalg/{hypergraph_matrix,laplacian_matrix,hodge_matrix}.py` (840 LOC); `algorithms/{assortativity,centrality,clustering,connected,properties,shortest_path,simpliciality}.py` (2,521 LOC).
**Test targets (gate-relevant, 74 tests):** `linalg/test_matrix.py` (14); `algorithms/test_assortativity.py` (3), `test_centrality.py` (7), `test_clustering.py` (3), `test_connected.py` (6), `test_properties.py` (23), `test_shortest_path.py` (2), `test_simpliciality.py` (11).
**Design:** spec §4.3 — matrices derived from the bipartite graph via `sprs` (new dep — plan-level note required; check MSRV compat); algorithms delegate to rustworkx-core on the 1-skeleton (`skeleton()`) or bipartite projection. **Dependency:** needs `Hypergraph::skeleton()` / `bipartite_graph()` accessors (spec §4.1 lists them, unimplemented — Phase 3 Task 1 builds them; simpliciality module needs Phase 2's SimplicialComplex).
**Probe targets:** matrix dtypes/ordering (row/col order = insertion order? probe), `order` parameter semantics, dense-vs-sparse return types; which centrality measures are hypergraph-native vs skeleton-delegated; connected-components definition (via skeleton).
**Suggested task split:** matrix constructors (incidence/adjacency/degree) → laplacian + hodge → skeleton/bipartite accessors + connected/shortest_path → properties (largest module, 23 tests) → centrality → clustering/assortativity → simpliciality (SC-dependent) → reconciliation.

## Phase 4: generators + stats + readwrite (spec §10.3, est. 1-2 weeks)

**XGI sources:** `generators/{classic,lattice,random,randomizing,simple,simplicial_complexes,uniform}.py` (2,086 LOC); `stats/{nodestats,edgestats,dinodestats,diedgestats}.py` (2,414 LOC); `readwrite/{json,edgelist,hif,incidence,bipartite,bigg_data,xgi_data}.py` (1,122 LOC).
**Test targets (127 tests):** generators 25 (classic 6, lattice 1, randomizing 2, random 5, simple 2, simplicial_complexes 5, uniform 4); stats 72 (core_stats_functions 52!, nodestats 9, edgestats 2, dinodestats 3, diedgestats 6); readwrite 30 (bigg 3, bipartite_edgelist 5, edgelist 5, hif 5, incidence_matrix 2, json 2, xgi_data 2).
**Design:** spec §4.3/§9.4 — generators seeded `rand_pcg` (new dep); RNG divergence from numpy is EXPECTED (§10.5 risk row): structural-invariant property tests + recorded fixtures, NOT exact-membership equality. Stats = pure functions over attrs (nodestats/edgestats first; di* stats need DiHypergraph — done). Readwrite: serde+serde_json (HIF/JSON), custom parsers (edgelist/BIGG); `xgi_data` gated behind `network` feature (HTTP fetch — consider deferring; tests need network or fixtures).
**Also lands:** `filterby(stat,…)` on views (deferred from Phase 2 — stats machinery exists after this phase; spec §4.2 note).
**Probe targets:** generator seeds/invariants (§9.4 test split: exact-fixture vs property); stats dispatch machinery (`IDStat`, `dispatch_stat` — this is XGI's plugin system; design a Rust trait-object or enum-dispatch equivalent, register divergences); HIF round-trip fidelity; edgelist comment/delimiter handling.
**Suggested task split:** stats core (52-test file is the beast — split nodestats/edgestats/di*) → generators (seeded, per-family tasks) → readwrite json/hif → edgelist/bigg/incidence → filterby-backfill + reconciliation.

## Phase 5: layout + viz (spec §5, est. 1 week)

**XGI sources:** `drawing/{layout,draw,draw_utils}.py` (3,236 LOC — **NOT a direct port**, spec §4.4/§5).
**Test targets (30 tests):** `drawing/test_layout.py` (11 — position assertions, direct port target); `test_draw.py` (16) + `test_draw_utils.py` (3) — **replaced by scene-graph tests** (spec §9.3).
**Design:** spec §5 is prescriptive: layout = pure math (4 strategies: random/circular/spring/bipartite — delegate to rustworkx-core layouts where possible); viz = `SceneGraph` struct + `render_to_svg()`/`render_to_json()` (NO matplotlib in Rust; the Python `draw_hypergraph()` shim is Phase 7's `mpl` feature). Scene-graph schema is the contract for WASM/React consumers (§5.3 — copy the schema verbatim into the phase plan).
**Probe targets:** layout determinism under fixed seed (spring layout RNG); bounding-box/coordinate conventions; test_layout.py's exact assertions.
**Suggested task split:** layout strategies → SceneGraph struct + JSON serializer → SVG renderer → drawing-test replacement (§9.3) + reconciliation.

## Phase 6: convert + dynamics + communities + utils (spec §10.3, est. 3-5 days)

**XGI sources:** `convert/*.py` (1,741 LOC — 12 modules); `dynamics/synchronization.py` (296); `communities/spectral.py` (140); `utils/{utilities,tensor,trie}.py` (1,006).
**Test targets (52 tests):** convert 35 (higher_order_network 11, hyperedges 4, line_graph 4, bipartite_edges/bipartite_graph/hif_dict/hypergraph_dict/incidence/pandas 2 each, encapsulation_dag/graph/k_skeleton 1 each); dynamics 3; communities 0 (!); utils 11.
**Design:** networkx→rustworkx; pandas→`Vec<Record>` (PyO3 converts at boundary, Phase 7; Babylon doesn't use it — consider `to_pandas` deferral with a register row); spectral needs `ndarray`/`sprs` eigendecomposition (dep decision — Phase 3 may have landed sprs already); `dynamics` SIR/SIS… wait — XGI dynamics = synchronization (Kuramoto), probe scope. Utils: `powerset`, `update_uid_counter`, `convert_labels_to_integers`, tensor/trie — port what prior phases actually import (YAGNI audit first).
**Suggested task split:** convert core (dict/edgelist/hif/bipartite — feeds Phase 7 tests) → convert graph-adapters (rustworkx/line-graph/k-skeleton) → dynamics → communities+spectral → utils audit-port → reconciliation.

## Phase 7: Python bindings — THE CONFORMANCE GATE (spec §6, est. 1-2 weeks)

**Scope:** PyO3 bindings for the 164-symbol surface (spec §4.3 mapping table + §4.5 import surface); maturin `pyproject.toml`; auto-generated `hypergraph_rs/__init__.py` mirroring XGI's exactly; `.pyi` stubs; matplotlib shim (`drawing` submodule, `mpl` feature, spec §6.3); **all D-row binding contracts** (spec §4.7 third column — warn-translations, set conversions, dict-merge D6, XGIError shims for D14/D15, SC typo message D-strings, sources/targets aliases, direction-validation order from Phase 2 T4 report).
**Gate:** all 50 XGI test files pass with `import hypergraph_rs as xgi` (spec §9.1-9.2 inventory); the property-based differential test (§4.6/§9.5 — proptest vs real XGI, ADR052 pattern); Babylon `qa:regression` as secondary gate (§9.6).
**Special handling:** drawing tests (§9.3 — scene-graph replacements); generator tests (§9.4 split); xgi_data tests (network — mock or skip-list with register note); `test_import_time.py` (1 test — our import is native, should pass trivially).
**Plan note:** this phase's plan is mostly binding-mechanics per D-row + test-file-by-test-file enablement; organize tasks by XGI test file groups (core 138 → linalg/algorithms 74 → generators/stats/readwrite 127 → drawing 30 → convert/dynamics/utils 49).

## Phase 8: WASM bindings (spec §7, est. 1 week)

`wasm-bindgen` wrappers per §7.2 (JsValue-passing for attrs, typed methods for structure); serde-wasm-bindgen; TS types via wasm-pack; `wasm-pack test` headless; **size budget gate**: <500KB core gzipped, <1.5MB all-features (§7.5 — CI size check); determinism in browser (§7.6 — seed passing). Design is fully prescriptive in §7 — copy §7.1 crate structure + §7.2 binding pattern into the phase plan.

## Phase 9: CLI (spec §8, est. 3-5 days)

clap-derive surface (8 subcommands from §8.1: inspect/render/validate/self-test etc.); smoke tests per command; exit codes (§8.4); cargo-dist prebuilt binaries (§8.5). §8 is prescriptive — copy command surface into plan. The CLI bin is already named `hypergraph` (cargo doc collision #6313 precedent).

## Phase 10: optional React package (§10.3, 2-3 days) — BD decision whether to do at all.

## Phase 11: Babylon swap (spec §6.2, §10.6 — separate effort, post-port)

ADR083; 7-file `s/^import xgi$/import hypergraph_rs as xgi/`; pyproject path dep; `mise run qa:regression` byte-identical; semgrep `import xgi` ban (§6.6); xgi removed. **This is a Babylon-repo change with its own constitutional surface — plan it fresh when Phase 7's gate is green.**

---

## Cross-phase invariants (check at every phase end)

1. `mise run rust:check` + `mise run rust:msrv` green.
2. Divergence register internally consistent (no numbering collisions; every row has a vector).
3. Spec §10.3 checkmarks updated; plan gets an ACTUAL completion record.
4. SDD ledger current; `.opencode/goal.md` resume state accurate.
5. No new dependencies without a plan-level note + MSRV check (Phase 3: sprs; Phase 4: rand_pcg; Phase 6: ndarray?).
