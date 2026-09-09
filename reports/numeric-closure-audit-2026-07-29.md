# Numeric Dependency Closure Audit — Program 27 Phase 0 (Task 5)

**Date:** 2026-07-29
**Scanner:** `tools/numeric_closure_audit.py` (stdlib `ast`, static import-graph
closure over `src/babylon/{engine,domain,formulas,kernel,topology}`)
**Spec anchor:** §6.2 "The numeric annex" (adversarial blocker B4) —
`docs/superpowers/specs/2026-07-28-program-27-refoundation-design.md`

## Method

The scanner walks every `.py` file under the five closure roots, resolves
`import numpy`/`import scipy`/`from numpy import ...`/`from scipy... import
...` aliases per-file, then walks the AST a second time collecting every
`Attribute`/`Name` node whose base resolves to one of those aliases. This is
a **syntactic closure**, not a call-graph reachable-from-`run_tick` trace —
it over-approximates (catches type annotations like `np.ndarray`, not just
calls) but never under-approximates within the five roots, which is the
correct bias for a dependency-closure audit: false positives get filtered by
human review below, false negatives would hide a ruling obligation.

Full raw output (164 hits across 17 files) is not reproduced here; this
report extracts every hit with an actual LAPACK/HiGHS/sparse-BLAS dependency
— i.e. decompositions, solves, and correlation/regression routines — plus
the six spec-named sites verbatim.

## Step 2 verification (six spec-named sites)

Per spec §6.2, six sites are named. All six files appear in the closure;
the exact call sites the scanner finds in each file are listed below (the
spec's line numbers point at the `import` statement or an adjacent line in
some cases — the scanner does not emit import statements as "sites", only
attribute/name usages, so the verified line differs from the spec's cited
line where the spec cited the import line rather than the call line):

| Spec-cited site | File present? | Actual call site(s) found |
|---|---|---|
| `inter_industry.py:253` | yes | `inter_industry.py:253` — `np.linalg.inv` (exact match) |
| `production_chain_rent.py:144` | yes | `production_chain_rent.py:144` — `np.linalg.inv` (exact match) |
| `class_transition.py:74` | yes | `class_transition.py:74` — `np.linalg.eig` (exact match) |
| `curvature.py:26` | yes (file) | import line 25 (`from scipy.optimize import linprog`); call site `curvature.py:225` |
| `circulation.py:27` | yes (file) | import line 26 (`from scipy import sparse`); sparse ops at lines 56/108/110/113–115/123/145–164 |
| `lodes_commute_matrix.py:42` | yes (file) | import line 41 (`import scipy.sparse as sp`); sparse ops at lines 89/95/106/132–133/179–183/201–210 |

All six confirmed present — scanner validated, proceeding per plan.

## Per-site ruling table — six spec-named sites

| # | Site (file:line) | Operation | LAPACK/HiGHS dependency | faer coverage | Proposed ruling | Reasoning |
|---|---|---|---|---|---|---|
| 1 | `domain/economics/tensor_hierarchy/inter_industry.py:253` | `np.linalg.inv` — Leontief inverse `(I − A)⁻¹` | LAPACK `dgetrf`/`dgetri` (LU factor + inverse) | Covered — faer dense LU/solve | (a) same-LAPACK linkage under BLAS=1, **or** (b) faer LU re-derivation with tolerance gate | Dense, well-conditioned Leontief inverse on a small industry-count matrix; faer's dense LU is a direct analogue. Prefer (b) once a tolerance policy exists (III.12(b)); (a) is the interim/fallback if faer LU output diverges beyond tolerance on this specific matrix shape. |
| 2 | `domain/economics/tensor_hierarchy/production_chain_rent.py:144` | `np.linalg.inv` — same Leontief-inverse pattern (production-chain-rent variant) | LAPACK `dgetrf`/`dgetri` | Covered — faer dense LU/solve | (b) tolerance-bounded re-derivation | Same operation family as #1; ruling should track #1's outcome (one ruling, two call sites — recommend the Director rule these as a pair). |
| 3 | `domain/economics/tensor_hierarchy/class_transition.py:74` | `np.linalg.eig` — nonsymmetric eigendecomposition, stationary-distribution extraction (`argmin` of `|λ−1|`, then `real` part, `clip`) | LAPACK `dgeev` (nonsymmetric, complex-valued eigenvalues in general) | **Not covered for nonsymmetric case** — faer's `eig` support is real-Schur/symmetric-favored; general nonsymmetric dense eig with complex eigenvalue output is a gap (per spec's "faer: LU/QR/SVD/eig for dense" — eig coverage exists but nonsymmetric complex-eigenvalue parity with LAPACK `dgeev` needs verification, not assumed) | (a) same-LAPACK linkage, **flagged for faer-coverage verification before Director rules** | This is a stationary-distribution computation (Markov-chain-like), not a raw eigenvalue report — the *chosen* eigenvalue (closest to 1, real part) is stable under decomposition-algorithm choice for a well-behaved transition operator, so (b) may be viable, but LAPACK `dgeev`'s internal QR-algorithm iteration count/pivoting differs from any Rust eig library in ways that could perturb which eigenvalue argmin-selects near degenerate cases. Recommend (a) pending a concrete faer nonsymmetric-eig conformance check. |
| 4 | `formulas/curvature.py:225` (import at `:26`) | `scipy.optimize.linprog` — Ollivier-Ricci curvature via LP (HiGHS solver backend) | HiGHS simplex/interior-point, **not a LAPACK linkage** — no faer analogue since faer does not do LP | **Not covered — no LP solver in faer's scope** | **(c) retirement under III.10 Earn-Its-Keep — flagged likely-(c) per spec §6.2** | Spec explicitly names this the leading III.10-retirement candidate: "a degenerate LP has multiple optima, so this is behavioral drift, not float noise." A degenerate-LP optimum selection is solver-implementation-dependent (which vertex of the optimal face HiGHS vs any other LP solver returns is not determined by the math alone), so no tolerance policy can bound the drift — it is not a floating-point precision problem, it is a **choice among multiple correct answers**. III.10 asks "does this construct earn its keep" — Ollivier-Ricci curvature is a differential-geometry embellishment on the topology substrate, not load-bearing gameplay math per NORTH_STAR's closed-for-v1.0 algebra. **III.10 justification status: UNVERIFIED — no citation found in this audit that Ollivier-Ricci curvature output feeds any player-visible or tick-hash-determining computation; this needs an explicit call-site trace (grep for callers of the function housing this `linprog` call) before the Director can sign off on retirement.** |
| 5 | `domain/economics/substrate/circulation.py:27` (import) | `scipy.sparse` — CSR/COO sparse OD-matrix construction, sparse mat-vec (`od_matrix.T @ v_vec`, `inv_sums @ od_matrix`), `sparse.diags` | Sparse BLAS (SpMV) — no factorization/solve in this file, pure sparse matrix-vector products | Covered — faer has a sparse layer (per spec II.12 three-layer stack: authoring → sparse matrix → operator expression, restated in Rust with faer) | (a) same-linkage or (b) tolerance-bounded re-derivation — **not a retirement candidate** | This is the commuter-flow redistribution matvec (module docstring itself notes "~1000+ hexes accumulates ~1e-9" — i.e. the code already documents float accumulation as a known, bounded hazard). Sparse mat-vec products are deterministic given a fixed traversal order; faer's sparse layer is the designed Rust target per §6.2's own text ("restates in Rust with faer as the default numeric layer"). Recommend (b) with the existing ~1e-9 accumulation bound formalized as the III.12(b) tolerance. |
| 6 | `domain/economics/lodes_commute_matrix.py:42` (import) | `scipy.sparse` — CSR/COO sparse commute-matrix construction (`sp.csr_matrix`, `sp.coo_matrix`, `sp.issparse` validation) | Sparse BLAS — construction/validation only, no solve found in this file | Covered — faer sparse layer | (a)/(b) — **not a retirement candidate** | Pure data-structure construction (builds the LODES commuter-flow sparse matrix from parquet-sourced OD pairs); no decomposition or solve. Lowest-risk of the six — faer's sparse construction is a direct analogue. Recommend (b), tolerance bound TBD (likely exact-integer or near-zero since inputs are counts, not accumulated floats). |

## Additional numeric-closure sites found (beyond the six spec-named)

The closure scan surfaced further numpy/scipy usage in the same roots. Most
are array-shape declarations (`np.ndarray` type hints), zero/ones/eye
allocation, or `np.asarray`/`np.array` conversions with **no LAPACK/HiGHS
dependency** — these need no per-site ruling (they are plain array
bookkeeping, faer-agnostic). The ones below DO carry a solver/statistics
dependency and are flagged for the Director's attention as candidate
additions to the porting contract table (Task 10):

| Site (file:line) | Operation | Dependency | faer coverage | Proposed ruling | Reasoning |
|---|---|---|---|---|---|
| `engine/systems/vol2_circulation.py:229-235` | `np.errstate` guard around `np.where(row_sums > 0, v_pre_vec / row_sums, 0.0)`; `year_matrix.matrix.T @ normalized` sparse mat-vec | Sparse BLAS SpMV (same family as circulation.py, site 5) | Covered — faer sparse | (b) tolerance-bounded re-derivation | Same OD-matrix mat-vec pattern as circulation.py; should be ruled alongside site 5, not independently. |
| `domain/economics/throughput/analysis.py:140,211` | `scipy.stats.pearsonr` — Pearson correlation, min-30-sample gate | Not LAPACK; scipy's stats routines are closed-form (no iterative solver) | Not applicable — this is a reporting/analysis correlation, not a tick-path decomposition | **(c) candidate — verify reachability from `run_tick` first** | Module lives under `domain/economics/throughput/` — appears to be an analysis/reporting utility (correlates `tau_lambda` with a class proxy), not obviously on the per-tick critical path. If it is observer/reporting-only, it is out of scope for the numeric annex entirely (§6.2 only covers what's "reachable from `run_tick`") and should be excluded from the ruling table rather than retired. |
| `domain/economics/validation/regression.py:98` | `scipy.stats.linregress` | Not LAPACK; closed-form OLS | Not applicable | **(c) candidate — verify reachability**, same reasoning as above | Path (`domain/economics/validation/`) strongly suggests a test/validation helper, not tick-path production code. |
| `engine/optimization/monte_carlo.py:174` | `scipy.stats.t.ppf` — Student-t critical value for confidence intervals, with a hand-rolled fallback when scipy is absent (`HAS_SCIPY` guard already in the source) | Not LAPACK; closed-form | Not applicable | Likely out of scope — `engine/optimization/` is a sensitivity/what-if analysis tool, not the tick path | The file already guards for scipy's absence with a manual z-score fallback — this is itself evidence the module's authors did not consider scipy a hard dependency, supporting a (c)-adjacent "already optional" disposition rather than a live retirement decision. |
| `engine/optimization/sensitivity.py:381,456` | `np.array(outputs)` feeding `morris_analyze.analyze` (Morris sensitivity method, likely from `SALib`, not scipy/numpy internals) | No LAPACK — array marshalling only | Not applicable | Out of scope | Plain array construction; no solver dependency. Included here only because the closure scan is over-approximate — no ruling needed. |

**Note on scope:** `engine/optimization/` and `domain/economics/{throughput,validation}/`
being included in the closure roots (`engine`, `domain`) but likely NOT
reachable from `run_tick` is a real gap between "static import-graph closure
over five directories" (what the scanner does, and what Step 1 specifies)
and "every numpy/scipy call reachable from `run_tick`" (what spec §6.2
actually asks for). This audit does not attempt a call-graph reachability
trace from `run_tick` — that would require a separate, more expensive
analysis. The six spec-named sites are all confirmed on/near the tick path
(Leontief inverse, class-transition eigendecomposition, Ollivier-Ricci
curvature feeding topology, and the two commuter-flow sparse matrices used
by Vol II circulation). The additional sites above are flagged with an
explicit reachability caveat rather than silently folded into the same
ruling table.

## Summary — proposed rulings by ruling class

- **(a) same-LAPACK linkage (or a-as-fallback):** class_transition.py:74 (`np.linalg.eig`, nonsymmetric — flagged for faer-coverage verification before Director rules a vs b)
- **(b) tolerance-bounded re-derivation:** inter_industry.py:253, production_chain_rent.py:144 (paired Leontief-inverse ruling), circulation.py sparse matvec (+ vol2_circulation.py:229-235 paired), lodes_commute_matrix.py sparse construction
- **(c) retirement under III.10 — flagged likely-(c):** curvature.py:225 (`scipy.optimize.linprog`, Ollivier-Ricci curvature) — **III.10 justification status: UNVERIFIED, needs a caller-reachability trace before Director sign-off**
- **(c) candidates pending reachability verification (not yet ruled, may be out of scope entirely):** throughput/analysis.py (`pearsonr`), validation/regression.py (`linregress`), optimization/monte_carlo.py (`t.ppf`)

## Director sign-off

Rulings are Director-gated. Merging this audit report and its scanner tool
does **not** enact any ruling — each row below is a pending decision this
report exists to inform.

- [ ] `inter_industry.py:253` — `np.linalg.inv` — ruling: ___
- [ ] `production_chain_rent.py:144` — `np.linalg.inv` — ruling: ___
- [ ] `class_transition.py:74` — `np.linalg.eig` — ruling: ___
- [ ] `curvature.py:225` — `scipy.optimize.linprog` — ruling: ___ (flagged likely-(c))
- [ ] `circulation.py` sparse matvec sites — ruling: ___
- [ ] `lodes_commute_matrix.py` sparse construction sites — ruling: ___
- [ ] `vol2_circulation.py:229-235` sparse matvec (pair with circulation.py) — ruling: ___
- [ ] `throughput/analysis.py` `pearsonr` — reachability + ruling: ___
- [ ] `validation/regression.py` `linregress` — reachability + ruling: ___
- [ ] `optimization/monte_carlo.py` `t.ppf` — reachability + ruling: ___
