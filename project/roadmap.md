<!-- vale off -->

# Babylon v4 roadmap mirror

<!-- V4-ROADMAP-MIRROR:START -->

This file explains the accepted gate order. It is not a status tracker. Linear
owns scope, status, priority, dependencies, Horizon, milestones, schedule, and
current work. Use the
[Babylon v1 project](https://linear.app/percy-raskova/project/babylon-v1-playable-political-economy-299b037e7feb)
and its
[Roadmap Charter](https://linear.app/percy-raskova/document/babylon-v4-roadmap-charter-2c67d2898306)
for current portfolio truth.

Target dates are provisional planning anchors. A gate opens only when the prior
gate meets its acceptance contract; elapsed time does not open it.

### G1 — Governance & portfolio

- Target: 2026-09-30
- Delivery roots: PER-5
- Delivery detail: PER-14, PER-15, PER-16, PER-34, and PER-35 remain
  children of the governance root.
- Acceptance: Constitution v4 and the Linear control surface give a fresh
  reader one product purpose, one validation standard, and the next three gates.

### G2 — Executable causality

- Target: 2026-11-30
- Delivery roots: PER-6
- Delivery detail: PER-17 owns executable phase placement; PER-18 owns the
  detached whole-tick transaction and nominal world hash; PER-19 owns
  sequential same-rank composition, role-sensitive effect authority,
  rank-aware refusals, and in-memory causal audit receipts. They remain
  children of the executable-causality root.
- Acceptance: phase anchors determine rule order, a failed tick rolls back its
  working copy, the combined graph-and-register state has one canonical hash,
  governed production rules cannot relabel themselves to escape role authority,
  restricted rule roles cannot author unapproved outcomes, and actual effects
  produce deterministic receipts only after whole-tick success.

### G3 — PostgreSQL, H3 & Archive slice

- Target: 2027-02-28
- Delivery roots: PER-7, PER-12
- Delivery detail: this is a cross-root gate. PER-7 carries the PostgreSQL,
  H3, and Archive foundation; PER-12 carries the first decision-linked Archive
  surface. PER-20 implements the durable boundary; PER-21 through PER-24 remain
  the bounded delivery issues.
- Acceptance: a clean campaign can tick, commit, restart, search only known
  Archive material, and complete a county/place dossier-to-decision-to-
  consequence loop.
- Persistence writer status: implemented_current_V2_only
- PostgreSQL runtime status: PostgreSQL_17_only
- PER-20 durable boundary status: implemented_current
- PER-48 status: Done
- PostgreSQL boundary ADR:
  ADR220_rust_owned_postgresql_persistence_boundary
- Cutover law: the one-way cutover is complete. Rust is the sole live
  game-managed PostgreSQL 17 authority and owns the V2-only durable reader and
  writer. Python retains its declared periphery but has no game-state writer,
  transition reader, migration, DDL, or game-managed connection authority.
- Historical cutover sequencing: before activation, Python was the sole live
  writer, and its game-managed migration and runtime-write paths had to be
  disabled before Rust assumed authority. That requirement is satisfied and
  has no current runtime force.
- Decision surfaces: executable `DecisionSurfaceContract` belongs to PER-24 in
  this gate. It is design law now and planned implementation work here.

### G4 — Productive & distributive circuit

- Target: 2027-04-30
- Delivery roots: PER-10, PER-11, PER-12
- Delivery detail: PER-10 is the primary Political Economy Circuit root;
  PER-11 carries slice-driven system dispositions, and PER-12 carries the
  circuit decision surfaces assigned to this gate. ADR250 R3: one thin
  per-county circuit lands first, visible and testable in the Bevy UI,
  before the full scope.
- Acceptance: county-by-NAICS production, inventories, realization, freight,
  ReserveArmy, and class migration form one visible causal slice, with the
  first thin slice observable in the UI ahead of the full scope.

### G5 — Player agency

- Target: 2027-06-30
- Delivery roots: PER-9
- Delivery detail: Player Agency is the primary root. Archive and fog work may
  support the slice without replacing that root.
- Acceptance: Investigate and one costed intervention create next-week intents,
  causal receipts, and Archive updates, with at least two viable strategies.

### G6 — COVID emergence benchmark

- Target: 2027-10-31
- Delivery roots: PER-8, PER-10
- Delivery detail: ADR250 R2 merges the E0 discipline and the full-circuit
  scope into one benchmark; the historical benchmark and Political Economy
  Circuit roots converge at this gate.
- Acceptance: a 2019 control, a historical shock envelope, and strong- and
  weak-capacity counterfactuals run 104 weekly ticks; shocks enter only as
  governed external-event rows adding allowed pressure, burden, or capacity
  effects; the runs show causal divergence, heterogeneity, hysteresis, and
  counterfactual response through organization money, working capital,
  freight, firms, class movement, and player counterfactuals, with no direct
  outcome writes.

### G7 — Systemic credit & 2008

- Target: 2027-12-31
- Delivery roots: PER-8, PER-10
- Delivery detail: this is a cross-root gate. PER-8 carries the historical
  benchmark, and PER-10 carries banking topology and the credit circuit.
- Acceptance: asset-quality and confidence pressure produces an emergent credit
  contraction and downstream production, employment, class, territory, and
  political effects through the normal action surface.

### G8 — Representative-world v1

- Target: 2028-04-30
- Delivery roots: PER-13
- Acceptance: the proven circuit scales to representative US counties, key
  corridors, selected countries, residual blocs, decision-linked Archive pages,
  and the clean-install, restore, and release gates.

## Portfolio policies

Every frozen Python system eventually receives one disposition: `Port`,
`Adapt`, `Replace`, or `Retire`. Playable causal slices pull those decisions;
finishing all 34 reference systems is not a prerequisite for gameplay.

Kimi-derived data remains Research under PER-43 until a named mechanic names a
field consumer. Only that consumer's required fields enter an ingestion slice.

Base membership identity is current. The attributed-membership payload is
empty, unhashed, unwritten, and unconsumed. It remains planned Research under
PER-44 until a named production mechanic writes, hashes, and consumes it.

Linear records the live dependency graph. A `blocked by` relation is appropriate
only when an API, producer, ontology, executable contract, or Director decision
is missing. Gate order belongs here and in the charter, not in artificial issue
dependencies.

Project #7 status: frozen_migration_input. Project #8 status:
frozen_migration_input. They remain historical evidence while PER-3, PER-4, and
PER-15 complete mapping, import, and redirect audits. Neither is the roadmap.

<!-- V4-ROADMAP-MIRROR:END -->

<!-- vale on -->
