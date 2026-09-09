---
condition: Continue working until the Rust rewrite (hypergraph-rs) is completed
status: paused
iterations: 10
started_at: 2026-07-18T00:00:00-04:00
last_verdict: "not_met — Phase 0+1 DONE (19/19, judge-verified). Phase 2 IN PROGRESS: Tasks 1-6 of 13 complete, all reviewed APPROVE. Paused mid-phase at a clean boundary (owner token limit); all work committed, gates green, no in-flight tasks."
max_iterations: 0
paused_reason: owner token budget; paused mid-Phase-2 at the Task 6 boundary
---

# Goal context

Replaced: "Program 09 — Full-Game Build executed to completion" (active
2026-07-04, 0 iterations, superseded by owner directive 2026-07-18).

## Checkpoint 1 — DONE (verified by judge 2026-07-18)

Phase 0+1: all 19 plan tasks complete and reviewed in worktree
`.claude/worktrees/feature-hypergraph-rs-phase-0-1` (branch
`feature/hypergraph-rs-phase-0-1`). 105 workspace tests (71 core + 34
conformance), 31 XGI ground-truth vectors, divergences D1–D13 registered
(spec §4.7, executable), `mise run rust:check` + `mise run rust:msrv`
(1.85) green, zero `unsafe`. Plan carries the ACTUAL completion record.
Also landed: H1–H9 hardening, conformance harness, rust-toolchain pin
(1.91.1), `rust:*` mise namespace, `.github/workflows/rust.yml`, Bretto
textbook grounding (sources/bretto_hypergraph-theory.pdf + .txt).

## Phase 2 — IN PROGRESS (Tasks 1–6 of 13 DONE, all APPROVE)

Plan: `docs/superpowers/plans/2026-07-18-hypergraph-rs-phase-2.md` (in the
worktree, committed at 1ee40f2d). Worktree HEAD: **f79a42b8**.
Ledger: `.superpowers/sdd/progress.md` (gitignored, in worktree).

| Batch | Commits | Scope | Verdict |
|---|---|---|---|
| T1+2 | 678b7f0a, 586bcb8c | MembershipError family; infallible add_node_to_edge; remove_empty on remove_node(s)_from; remove_edges_from (truncate-at-Err parity); D9 flipped | APPROVE |
| T3+4 | a6568071, 294bc113 | DiHypergraph core (arc-presence direction = tail agent→edge arc / head edge→agent arc; D14) + membership ops/removals (D11-extension) | APPROVE |
| T5+6 | 58234264, f79a42b8 | DiHypergraph bulk/attrs/copy/freeze/Debug/eq (D16) + SimplicialComplex core: add_simplex subface closure, has_simplex (D15, D17) | APPROVE |

State: **139 core / 67 conformance / 55 vectors**, all green. Register now
D1–D17. Key probed truths: dimemberships(n)=(in,out) (head-edges first);
directed "emptied"=both sides empty; SC 4-simplex→11 edges (no singletons),
top-first uid; SC empty simplex CREATED (D1-class); XGI DiHypergraph has no
clear_edges (D16) and its freeze skips membership ops (D12-extension).

## Resume: next move

Dispatch **T7+8 batch** (SDD implementer subagent): SC add_simplices_from /
add_weighted_simplices_from / close (max_order semantics — plan has probes)
+ SC removals (remove_simplex_id supface cascade, remove_node semantics),
copy/freeze/Debug/eq. Then T9+10 (NodeView/EdgeView core + advanced),
T11+12 (DiViews + subhypergraph), T13 solo (spec reconciliation: §3.3
arc-presence, §3.4 cache deferral, filterby→Phase 4 note, D-table order,
plan completion record, `mise run rust:msrv`, plus the carried minors:
T3+4's 5 cosmetic nits, add_simplex E:Default plan-signature drift,
face-mapping canary, T1+2 vector-rename note, bulk-asymmetry sanity).

Process (established, works — do not redesign): plan tasks are
self-contained; dispatch implementer with plan path + task numbers + context
(HEAD, counts, register next-free D18, mirrors: DiHypergraph impl for SC
patterns, hypergraph.rs for Value-only bounded impls); then review-package
script + reviewer subagent (verify independently: re-run `mise run
rust:check`, re-probe XGI claims); ledger; sed-bump iterations. Review
script: `/home/user/.cache/opencode/packages/superpowers@git+https:/github.com/obra/superpowers.git/node_modules/superpowers/skills/subagent-driven-development/scripts/review-package`.
Probe python: `/home/user/projects/game/babylon/.venv/bin/python` (xgi
0.10.2). Gates from worktree root: `mise run rust:check` (every commit),
`rust:msrv` (T13). Commits: conventional + Co-Authored-By trailer.

After Phase 2: **every remaining phase has a planning brief** in
`docs/superpowers/plans/2026-07-18-hypergraph-rs-roadmap-phases-3-11.md`
(committed; per-phase scope, XGI sources, test targets w/ counts, design
pointers, probe targets, task-split suggestions, process recipe). Each
phase's Task 0 = write its task-level plan from that brief using the
Phase 2 plan as template (writing-plans skill), then SDD-execute. Phase
order per spec §10.3: 3 linalg/algorithms → 4 generators/stats/readwrite →
5 layout/viz → 6 convert/dynamics/communities/utils → 7 PyO3 bindings (THE
conformance gate, all 50 XGI test files) → 8 WASM → 9 CLI → (10 React
optional) → 11 Babylon swap (fresh plan, constitutional surface). Optional
housekeeping still open: ai/state.yaml update; branch integration (30+
commits) is a BD decision.
