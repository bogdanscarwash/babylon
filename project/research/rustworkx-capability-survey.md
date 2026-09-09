# rustworkx Capability Survey for Babylon

*(Authored 2026-07-14 by a web-research agent walking the full rustworkx.org API tree, grounded
in the repo's actual call sites; every claim cites a doc URL or file:line. Commissioned by the
owner to avoid rewriting what the library already ships and to mine mechanic inspiration. Feeds
Wave-4 topology work; §5's rayon findings feed the determinism-hardening task.)*

## 1. What we already call (grounded inventory)

**Pin:** `pyproject.toml:41` — `rustworkx = "^0.18.0"`; `poetry.lock` locks the exact build
`0.18.0`. Cross-checked against <https://www.rustworkx.org/release_notes.html>: **0.18.0 is the
newest released version** — there is no 0.19+ to catch up to (see §6).

The `rx.*` surface is small and disciplined — almost all of it funneled through two seam modules:

| Seam | Native calls |
|---|---|
| `src/babylon/topology/graph.py` (the `BabylonGraph`/`BabylonUGraph` substrate) | `rx.PyDiGraph`, `rx.PyGraph`, `rx.connected_components`, `rx.weakly_connected_components`, `rx.descendants`, `rx.has_path`, `rx.dijkstra_shortest_paths` |
| `src/babylon/topology/graph_algorithms.py` (the declared "Amendment L seam" — the only place analytics code touches `rx.*` directly) | `rx.number_connected_components`, `rx.weakly_connected_components`, `rx.is_connected`, `rx.articulation_points`, `rx.degree_centrality`, `rx.betweenness_centrality`, `rx.closeness_centrality`, `rx.stoer_wagner_min_cut`, `rx.dijkstra_shortest_path_lengths` |
| `src/babylon/domain/dialectics/instances/connectivity.py:148` | `rx.connected_components` (re-exposed as `pieces()`, the Π₀ functor) |

Everything else (`bifurcation/resilience.py`, `ooda/attention/sparrow.py`,
`domain/organizations/topology.py`, `engine/topology_monitor.py`, `engine/bifurcation_monitor.py`)
consumes these seams rather than importing `rustworkx` directly — most upgrades are single-file.

**What's hand-rolled today** (read line-by-line):

- `domain/bifurcation/resilience.py` — Betti numbers (β₀, β₁ from component count + Euler
  characteristic), structural-equivalence classes (neighbor-set grouping), critical singletons
  (wraps articulation points), critical cutsets (wraps Stoer-Wagner), and a manual "remove
  top-degree nodes, recompute L_max" purge simulation.
- `domain/bifurcation/analysis.py` / `ceiling.py` — pure business logic over graph queries (not
  algorithm duplication).
- `domain/organizations/topology.py` — STAR/HIERARCHY/MESH/CELL classification and key-figure
  structural importance, built from native density/degree/articulation-point primitives combined
  with hand-rolled classification rules.
- `ooda/attention/sparrow.py` — degree-signature equivalence classes, a manual 2×-mean
  betweenness threshold for "singletons," cutsets as articulation-point wrapper.
- `formulas/curvature.py` — fully custom Ollivier-Ricci curvature (scipy `linprog` for
  Wasserstein-1); rustworkx has **no** Ricci-curvature primitive, so this is correctly
  hand-rolled, though its per-pair distance loop (`_graph_distance`, lines 154-177) could batch
  through a native all-pairs function.

## 2. Rewrite-avoidance map

| Hand-rolled code (file:line) | Native rustworkx equivalent | Caveats |
|---|---|---|
| `domain/bifurcation/resilience.py:67-103` `compute_equivalence_classes` (group nodes by identical immediate-neighbor `frozenset`) | [`digraph_maximum_bisimulation`](https://www.rustworkx.org/apiref/rustworkx.digraph_maximum_bisimulation.html) — Paige-Tarjan relational coarsest partition | Bisimulation is *recursive* structural equivalence, strictly more correct than one-hop neighbor-set matching. **Digraph-only** — `BabylonUGraph` needs a symmetric-digraph view (both edge directions). |
| `ooda/attention/sparrow.py:91-114` `_compute_equivalence_classes` (degree + sorted-neighbor-degree signature, capped at 1000 nodes) | Same `digraph_maximum_bisimulation` | Same caveat; also removes the ad hoc `max_nodes = 1000` truncation, itself a silent-correctness risk on large graphs. |
| `domain/organizations/topology.py:131-190` `identify_key_figures` — copies the subgraph, removes each articulation point, recomputes component count, and separately recomputes neighbor-set equality for "has_equivalent" (165-177) | [`biconnected_components`](https://www.rustworkx.org/apiref/rustworkx.biconnected_components.html) for the removal-impact half; `digraph_maximum_bisimulation` for the equivalence half | `biconnected_components` gives the structure directly (which components share a cut vertex) without the O(n) remove-and-recount loop per articulation point. |
| `domain/organizations/topology.py:56-128` `classify_topology` (STAR/HIERARCHY/MESH/CELL via density threshold + max-degree + articulation-point heuristics) | Building blocks already native; no single "classify topology" call exists | Strengthen, don't replace: [`is_bipartite`](https://www.rustworkx.org/api/algorithm_functions/index.html) to validate two-camp splits; `biconnected_components` to distinguish CELL (ring-of-cliques) from HIERARCHY (tree) rigorously. |
| `topology/graph_algorithms.py:61-70` `density()` (inline `m / (n*(n-1))`) | **None** — confirmed absent from the [algorithm index](https://www.rustworkx.org/api/algorithm_functions/index.html) and the [NetworkX-comparison page](https://www.rustworkx.org/networkx.html) | Correctly hand-rolled; nothing to fix. |
| `formulas/curvature.py:154-177` `_graph_distance` — one single-pair Dijkstra call per (i,j) pair inside a double loop | [`distance_matrix`](https://www.rustworkx.org/apiref/rustworkx.distance_matrix.html) (hop-count) or [`floyd_warshall_numpy`](https://www.rustworkx.org/apiref/rustworkx.floyd_warshall_numpy.html) (weighted) — one batched all-pairs call | Efficiency-only rewrite. **Both rayon-threaded above a node-count threshold** — see §5; on the small neighbor-support subgraphs curvature touches (< 10 nodes) they stay single-threaded. |
| `domain/bifurcation/resilience.py:184-252` `compute_purge_resilience` — sort-by-degree (RNG tie-break) → remove top-k → recompute L_max | **No single native equivalent** for targeted-attack simulation | Legitimate hand-rolling. Could be *informed* by `core_number` (k-core) or `group_betweenness_centrality` for smarter-than-degree target selection (§3), but the attack loop stays hand-rolled. |
| `topology/graph.py:449-466` `_component_id_sets`, `dialectics/instances/connectivity.py:122-150` `pieces()` | `connected_components` / `weakly_connected_components` | **Not a gap — the correct mitigation pattern.** Both call the native algorithm then re-sort by insertion position because [`connected_components`](https://www.rustworkx.org/apiref/rustworkx.connected_components.html) returns `list[set[int]]` with **no documented ordering guarantee**. Any new code calling a set-returning rustworkx function must replicate this re-sort. |

## 3. Native capabilities we're NOT using — grouped by Babylon relevance

**Centrality / who matters** ([index §Centrality](https://www.rustworkx.org/api/algorithm_functions/index.html)):

- [`group_betweenness_centrality`](https://www.rustworkx.org/apiref/rustworkx.group_betweenness_centrality.html)
  — betweenness of a *set* → multi-target repression-squad selection: "which 3 cadre, arrested
  together, sever the most communication" — strictly better than the single highest-betweenness
  node.
- [`eigenvector_centrality`](https://www.rustworkx.org/apiref/rustworkx.eigenvector_centrality.html) /
  [`katz_centrality`](https://www.rustworkx.org/apiref/rustworkx.katz_centrality.html) — "important
  because your neighbors are important"; Katz tolerates disconnected/sparse graphs where closeness
  is undefined → Key-Figure "structural power" distinct from brokerage.
- [`pagerank`](https://www.rustworkx.org/apiref/rustworkx.pagerank.html) and
  [`hits`](https://www.rustworkx.org/apiref/rustworkx.hits.html) (dual hub/authority) on the
  TRIBUTE/EXPLOITATION digraph → imperial hierarchy score; HITS is the richer fit (hub =
  extractor, authority = value-producer — a natural Φ-adjacent duality).

**Connectivity / cycles beyond what's used:**

- [`biconnected_components`](https://www.rustworkx.org/apiref/rustworkx.biconnected_components.html),
  [`bridges`](https://www.rustworkx.org/apiref/rustworkx.bridges.html),
  [`chain_decomposition`](https://www.rustworkx.org/apiref/rustworkx.chain_decomposition.html) —
  bridges are *edges* whose loss disconnects → supply-chain/transport chokepoints, complementary
  to vertex articulation points and Stoer-Wagner min-cut.
- [`cycle_basis`](https://www.rustworkx.org/apiref/rustworkx.cycle_basis.html),
  [`simple_cycles`](https://www.rustworkx.org/apiref/rustworkx.simple_cycles.html) /
  [`digraph_find_cycle`](https://www.rustworkx.org/apiref/rustworkx.digraph_find_cycle.html) →
  circuit-of-capital validation: cycle detection on the value-flow digraph flags degenerate closed
  loops — a sanity-check sentinel, not just a mechanic.
- [`core_number`](https://www.rustworkx.org/apiref/rustworkx.core_number.html) (k-core) →
  cadre-density rings: formalizes `TopologyMonitor`'s ad hoc `cadre_density` heuristic
  (`engine/topology_monitor.py:138-179`) into "how many concentric shells of mutual reinforcement
  does this cell sit inside."
- `digraph_maximum_bisimulation` — see §2, direct rewrite target for both equivalence-class
  functions.
- Condensation ([0.17.0 release notes](https://www.rustworkx.org/release_notes.html): quotient
  graph where each node is an SCC) → imperial-hierarchy compression: collapse a
  mutually-reinforcing TRIBUTE/EXPLOITATION SCC into one supernode for a higher-altitude render.
  *(Caveat: verified via changelog only, not a live docstring — confirm the exact signature in a
  REPL before use.)*

**Trees / routing:**

- [`minimum_spanning_tree`](https://www.rustworkx.org/api/algorithm_functions/index.html) /
  [`steiner_tree`](https://www.rustworkx.org/apiref/rustworkx.steiner_tree.html) → organizer
  travel/logistics: MST over territory ADJACENCY weighted by repression/distance; Steiner tree for
  "connect *these* cells at minimum total cost" (proven (2 − 2/t) approximation bound).
- [`max_weight_matching`](https://www.rustworkx.org/apiref/rustworkx.max_weight_matching.html) →
  optimal 1:1 pairing (safehouse-to-family, organizer-to-mentee) maximizing total solidarity
  weight, O(n³).

**Search-with-control:**

- `DFSVisitor`/`BFSVisitor`/`DijkstraVisitor` via
  [`digraph_dfs_search`](https://www.rustworkx.org/apiref/rustworkx.digraph_dfs_search.html) —
  visitors can raise `StopSearch`/`PruneSearch` → State-AI informant search that stops the instant
  a target is found; narrative causality tracer that prunes irrelevant branches.
- [`layers`](https://www.rustworkx.org/apiref/rustworkx.layers.html) / `topological_generations`
  on a DAG → causality/provenance rendering as discrete depth layers for the frontend.
- `immediate_dominators` / `dominance_frontiers` → "which single decision sits on *every* path of
  authority from the center to this territory" — a chain-of-command chokepoint distinct from
  connectivity articulation points (control flow, not just connectivity).

**Matching/isomorphism:**

- [`vf2_mapping`](https://www.rustworkx.org/apiref/rustworkx.vf2_mapping.html) /
  `is_subgraph_isomorphic` → cell-structure detection (State AI pattern-matches a known
  clandestine-cell topology) or trap-pattern detection (honeypot/informant subgraph shapes).

**Matrix/serialization:**

- [`adjacency_matrix`](https://www.rustworkx.org/apiref/rustworkx.adjacency_matrix.html),
  [`distance_matrix`](https://www.rustworkx.org/apiref/rustworkx.distance_matrix.html),
  [`floyd_warshall_numpy`](https://www.rustworkx.org/apiref/rustworkx.floyd_warshall_numpy.html) →
  batch numeric exports feeding numpy formulas directly.
- [`node_link_json`](https://www.rustworkx.org/apiref/rustworkx.node_link_json.html) — no
  hand-rolled node-link/GraphML serializer exists in `src/babylon` today; worth checking whether
  web-bridge JSON payloads could be produced or cross-validated via this native call.

**Layout (engine-side, for the org-network canvas):**

- [`kamada_kawai_layout`](https://www.rustworkx.org/apiref/rustworkx.kamada_kawai_layout.html) —
  **zero randomness, no seed parameter at all**; pure energy minimization — the
  best-determinism-story layout. `circular_layout`/`shell_layout`/`spiral_layout`/
  `bipartite_layout` likewise position-from-structure, no RNG.
- `spring_layout` exists but **requires an explicit seed** (§5) — prefer Kamada-Kawai for
  anything that must hash identically.

**Random graph generators (procedural generation — always-seed-it territory, §5):**

- [`undirected_sbm_random_graph`](https://www.rustworkx.org/apiref/rustworkx.undirected_sbm_random_graph.html)
  (stochastic block model) → procedurally seed a class-stratified initial network (blocks =
  classes, dense within-block / sparse cross-block ties) for new-scenario generation.
- `barabasi_albert_graph`, `random_geometric_graph`, `hyperbolic_random_graph` — see §4.

## 4. New-mechanic inspirations

- **Imperial hierarchy via HITS, not just PageRank** — separate hub (net extractor) and authority
  (net value-producer) scores per node in one call maps onto the Fundamental Theorem's `W_c` vs
  `V_c` distinction more directly than a single PageRank number.
- **Multi-target repression AI via `group_betweenness_centrality`** — a State AI that evaluates
  candidate *arrest sets* against C_B(S), letting "purge 3 mid-level organizers" beat "purge the 1
  obvious leader" when leadership is redundant — fits MLM-TW's emphasis on organizational, not
  individual, vulnerability.
- **k-core vanguard-party detector** — replace `TopologyMonitor`'s liquid/solid phase heuristic
  (`engine/topology_monitor.py:360-387`) with `core_number`: "solid" (vanguard) phase becomes
  formally "the k-core for k ≥ N exists and contains > X% of cadre" — graph-theoretic rather than
  threshold-tuned party consolidation.
- **Circuit-of-capital sentinel via `simple_cycles`/`cycle_basis`** — run cycle detection on the
  value-flow digraph each tick as an invariant check: a cycle with net-zero extraction is a
  modeling bug — a candidate addition to the Sentinels family.
- **Steiner-tree cadre-building** — an OODA "build organization" action computing `steiner_tree`
  over the territory/agent graph: minimum-cost intermediate contacts connecting target unorganized
  workers to the existing network — a concrete, boundedly-approximable recruitment-path mechanic.
- **Bisimulation "structural twins"** — `digraph_maximum_bisimulation` on the COMMAND subgraph
  gives formally rigorous "these key figures are interchangeable" groups — exactly the
  `is_singleton` flag `identify_key_figures` computes ad hoc today, but recursive/transitive,
  catching deeper redundancy (3-way equivalences the one-hop check misses).
- **Hyperbolic routing as clandestine-network resilience** —
  [`hyperbolic_greedy_routing`](https://www.rustworkx.org/api/algorithm_functions/index.html) /
  `hyperbolic_greedy_success_rate` measure decentralized message routing with only local
  information — a literature-grounded proxy for "can this cell structure relay orders without a
  compromised central switchboard."
- **SBM-seeded scenario generation** — `undirected_sbm_random_graph(sizes, probabilities, loops,
  seed=…)` as a deterministic procedural generator for a scenario's initial solidarity network:
  block sizes = class populations, block-probability matrix = a designed solidarity-ceiling table
  — lets `compute_solidarity_ceiling` also *shape the starting graph*.
- **`contract_nodes` for literal org mergers** —
  [`PyDiGraph.contract_nodes`](https://www.rustworkx.org/apiref/rustworkx.PyDiGraph.contract_nodes.html)
  is a one-call primitive for merger graph-surgery — *provided* `weight_combo_fn` is always
  supplied explicitly (§5: omitting it is a documented determinism trap).

## 5. Determinism red flags

Babylon already pins `OMP_NUM_THREADS`/`OPENBLAS_NUM_THREADS`/`MKL_NUM_THREADS`/
`NUMEXPR_NUM_THREADS=1` (`.mise.toml`, `tests/conftest.py`) — but **`RAYON_NUM_THREADS` is pinned
nowhere in the repo**. rustworkx's Rust core uses rayon independently of BLAS/OpenMP, so the
existing thread-cap discipline does **not** cover it.

| Function(s) | Docs say | Risk to Babylon |
|---|---|---|
| [`betweenness_centrality`](https://www.rustworkx.org/dev/apiref/rustworkx.graph_betweenness_centrality.html) / `edge_betweenness_centrality` | Multithreaded when nodes > `parallel_threshold` (default 50); `RAYON_NUM_THREADS` adjusts workers | **Actively called today** (`graph_algorithms.py:84`, `sparrow.py:63`). Any subgraph over 50 nodes runs multi-threaded; float summation across threads is not order-stable → centrality floats (and any downstream hash) not provably bit-identical. **Highest-priority gap.** |
| [`closeness_centrality`](https://www.rustworkx.org/apiref/rustworkx.closeness_centrality.html) / `newman_weighted_closeness_centrality` | Same `parallel_threshold=50` / rayon language | **Actively called today** (`graph_algorithms.py:89`, `sparrow.py:68`). Same risk. |
| [`group_betweenness_centrality`](https://www.rustworkx.org/apiref/rustworkx.group_betweenness_centrality.html) | Same pattern | Not yet called — pin before adopting §4's repression-squad mechanic. |
| [`distance_matrix`](https://www.rustworkx.org/apiref/rustworkx.distance_matrix.html) / [`floyd_warshall_numpy`](https://www.rustworkx.org/apiref/rustworkx.floyd_warshall_numpy.html) | `parallel_threshold=300`, CPU-count pool | Not yet called; pin before the §2 curvature-batching rewrite. |
| `graph_transitivity`/`digraph_transitivity` | Thread pool = CPU count, **no size gate — always parallel** | Not called today. Integer triangle counts are associative, but pin/test before adoption. |
| [`all_pairs_dijkstra_path_lengths`](https://www.rustworkx.org/apiref/rustworkx.all_pairs_dijkstra_path_lengths.html) (and by inference `all_pairs_bellman_ford_*` — unverified) | Thread pool = CPU count, no size gate | Tempting for transport corridors (Program 11); path lengths are float sums — genuine non-associativity risk without `RAYON_NUM_THREADS=1`. |
| [`PyDiGraph.contract_nodes`](https://www.rustworkx.org/apiref/rustworkx.PyDiGraph.contract_nodes.html) with `weight_combo_fn=None` | Parallel-edge weights combined "arbitrarily based on an internal iteration order, subject to change" | Documented arbitrariness, not threading. **Never call without an explicit `weight_combo_fn`.** |
| [`spring_layout`](https://www.rustworkx.org/apiref/rustworkx.spring_layout.html) | Unseeded by default → different sequences per run | Must pass an explicit seed if ever adopted (prefer `kamada_kawai_layout`, which has no RNG at all). |
| All random generators (`undirected_gnp_random_graph`, `undirected_sbm_random_graph`, `barabasi_albert_graph`, `random_geometric_graph`, `hyperbolic_random_graph`, `random_regular_graph`, [`generate_random_path`](https://www.rustworkx.org/apiref/rustworkx.generate_random_path.html)) | `seed=None` default = unseeded | Any procedural-generation feature must thread a deterministic seed (follow the existing correct convention at `engine/topology_monitor.py:483`: `seed=tick`). |
| [`connected_components`](https://www.rustworkx.org/apiref/rustworkx.connected_components.html) / `weakly_connected_components` / `articulation_points` | `list[set[int]]`, no documented ordering | **Already correctly mitigated** by insertion-position re-sorts (`topology/graph.py:449-466`, `connectivity.py:122-150`, `resilience.py:128`) — the required pattern for any new set-returning call. |
| [`CentralityMapping`](https://www.rustworkx.org/apiref/rustworkx.CentralityMapping.html) | Mapping protocol; iteration order undocumented | Two-layer risk with rayon: values can differ by ULP AND `.items()` order is undocumented. `graph_algorithms.py:73-74` (`_centrality_ids`) builds a dict without re-sorting by node id — needs an explicit sort-by-key, since it feeds `SparrowAnalysis.centrality_rankings` (a JSON payload). |
| [`max_weight_matching`](https://www.rustworkx.org/apiref/rustworkx.max_weight_matching.html), [`graph_greedy_color`](https://www.rustworkx.org/apiref/rustworkx.graph_greedy_color.html) | Tie-breaking undocumented | "Verify before trust": pin with a golden test before assuming bit-identical output. |

## 6. Version notes

- **Our pin (`^0.18.0`, locked to `0.18.0`) is the current latest release** — no bump needed or
  possible.
- 0.18.0 already ships several §3/§4 candidates for free: BFS layers, hyperbolic greedy routing,
  random-walk sampling, the group-centrality family — new call sites only, zero dependency change.
- 0.17.0 (inside our pin) is where `digraph_maximum_bisimulation`, `condensation()`, and the
  GraphML serializer landed — all live in the current install.
- The lockfile's cp314-abi3 wheels indicate py3.14 forward-compat, consistent with the nightly
  py3.13 CI leg.
