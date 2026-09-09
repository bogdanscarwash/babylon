# T1.1 severity ceremony (conditional) — owner sign-off note

> Ledger label: **"T1.1 severity (conditional)"** (design §7,
> `ai/_inbox/t11-seam-severity-design.md`). Authored by U7 (the T1.1 train's closeout unit)
> once U1 (derived severity), U2 (single-sourced both surfaces), and U6 (the equality
> sentinel) had merged, so the drift table below reflects the real shipped
> `src/babylon/models/event_severity.py::DRIFT_TABLE` — not a projection.

## Why this is a ceremony, and why it is NOT a `tests/baselines/**` ceremony

Severity is a `G∘P` read-only projection (Amendment-S tripwire, design §1): it is derived
from `(event_kind × terminal_proximity)`, consumed by the web bridge and the Archive
Chronicle for **display and autopause only**, and is never read back into physics —
`tests/unit/models/test_event_severity.py`'s grep gate confirms no importer under
`babylon/engine` or `babylon/domain` reads `resolve_severity`. Consequently:

- **`qa:regression` stays byte-identical across the whole T1.1 train.** No tick hash,
  formula, or persisted game-state field changes. This is confirmed structurally (severity
  never enters `WorldState`/the graph/the tick loop), not merely asserted.
- **This is not a `tests/baselines/**` physics ceremony** — no baseline CSV/JSON changed,
  no `Baselines: blessed(<slug>)` commit trailer applies, and `tools/generate_ceremony_message.py`
  was not invoked. The §6.5 provenance-ceremony gate does not fire on this train.
- What *does* change is **owner-facing behavior on the game surface**: which events
  autopause the Archive Chronicle, and which render at which visual salience. That is a
  real, disclosed feature change — the reason this note exists.

## The reconciliation: why a derived rule cannot be a rubber stamp

The pure derivation rule (design §2) is binary for `CROSSING` events — `TERMINAL_ADJACENT`
⇒ `critical`, `INTRA_LEVEL` ⇒ `informational` — there is **no `warning` tier reachable from
a `CROSSING`**. The legacy 47-member hand-tiered snapshot (`_LEGACY_HAND_TIERS`, frozen in
`event_severity.py` purely as reconciliation input) had put 20 members at `warning`; of
those, only the 4 `ACT`-kind members (`state_repression`, `pogrom`, `lockout`,
`vigilantism`) are legitimately `warning` under the pure rule. The other 16 are
`CROSSING`/`PATTERN`-over-`CROSSING` members the rule **mechanically forces** off
`warning`, toward `critical` or `informational`. That reclassification is the drift table
below — every row carries a declared rationale in `event_severity.py::_DRIFT_RATIONALES`
(reproduced here), and `_build_drift_table` raises loudly at import if any drifted member
were ever missing one.

## Drift table (16 of 47 day-one members reclassified)

### Promoted `warning` → `critical` (8) — these NEWLY autopause the Archive Chronicle

| EventType | Rationale |
|---|---|
| `fascist_recruitment` | `CROSSING`/`TERMINAL_ADJACENT`: a node is captured by a fascist faction — a completed hostile transition on the same axis as `red_brown_coup` (already critical). |
| `doctrine_purge_failed` | `CROSSING`/`TERMINAL_ADJACENT`: the org remains trapped in `doctrine_trap_sprung`'s critical condition after a failed escape attempt — the persisting crisis must not be under-signaled. |
| `market_correction` | `CROSSING`/`TERMINAL_ADJACENT`: a fictitious/real divergence exceeding profit-rate serviceability is a completed crisis-axis "snap", materially on par with `economic_crisis`/`superwage_crisis` (both already critical). |
| `bifurcation_threshold` | `CROSSING`/`TERMINAL_ADJACENT`: this IS the George-Jackson bifurcation-axis crossing feeding `power_vacuum`'s branch resolution — an endgame-axis lock. |
| `co_optive_breakdown` | `CROSSING`/`TERMINAL_ADJACENT`: a co-optation failure WITH bifurcation is structurally the same bifurcation-axis event as `power_vacuum`/`revolutionary_offensive`/`fascist_revanchism` (all already critical). |
| `level_transition` | `CROSSING`/`TERMINAL_ADJACENT`: sublating the principal contradiction to a higher level is a major structural/regime-level leap. |
| `pattern_shift` | `PATTERN` inheriting `endgame_reached`'s tier: a recognized-endgame-pattern change is directly endgame-axis content by definition. |
| `red_settler_trap_detected` | `PATTERN` inheriting `bifurcation_threshold`'s tier: detecting this pattern means the RED_OGV terminal-endgame track (settler-socialist trap) is live. |

### Demoted `warning` → `informational` (8) — these are now quieter on the Chronicle

| EventType | Rationale |
|---|---|
| `excessive_force` | `CROSSING`/`INTRA_LEVEL`: the forcing-hazard spark is a reversible precursor — it does not itself lock any terminal axis (`uprising`, already critical, is the completed rupture-adjacent crossing once agitation gates it). |
| `mass_awakening` | `CROSSING`/`INTRA_LEVEL`: a reversible, frequently-recurring per-node consciousness threshold-cross, not itself an endgame-axis lock. |
| `fascist_drift` | `CROSSING`/`INTRA_LEVEL`: an early-stage, reversible per-node drift — `fascist_recruitment` is the completed capture this precedes. |
| `dispossession_cascade` | `CROSSING`/`INTRA_LEVEL`: a recurring milestone marker on a continuous decline, not itself a terminal lock. |
| `organizational_fracture` | `CROSSING`/`INTRA_LEVEL`: an individual, reversible defection that only completes the hostile capture (`red_brown_coup`, already critical) once a majority accumulates. |
| `doctrine_trap_escaped` | `CROSSING`/`INTRA_LEVEL`: the org's positive resolution out of `doctrine_trap_sprung`'s critical condition. |
| `entity_death` | `CROSSING`/`INTRA_LEVEL`: an individual, per-entity starvation event — the aggregate mortality signal (`population_attrition`, already informational) carries the system-level severity. |
| `crisis_phase_transition` | `CROSSING`/`INTRA_LEVEL`: one `EventType` covers all 6 arcs of the `CreditCyclePhase` machine, most of which are routine reversible business-cycle churn — a genuine day-one granularity limitation (the machine's one terminal arc, `STAGNATION`, is under-signaled by this generic event; flagged for a possible future per-arc split, not resolved here). |

**Net tier composition, 47 day-one members:** critical 14 → **22** (+8); warning 20 → **4**
(the 4 `ACT` members only); informational 13 → **21** (+8).

## Autopause / render-tier consequence (the owner-visible feature disclosure)

`compute_autopause_state` (`babylon/tui/chronicle_salience.py`, Program 24 P3 WO-48 — a
NET-NEW mechanic, no pre-existing Archive/TUI autopause to compare against) fires **iff the
tick's events contain at least one `critical`-tier event**; warning/informational events
never autopause regardless of count. Because this is the FIRST Archive Chronicle build to
carry autopause at all, "drift" here does not mean "changed from a shipped behavior" — it
means "the derived table this net-new mechanic keys on is not simply the legacy hand-tier
snapshot verbatim." Concretely, had the Chronicle instead consumed the legacy 47-tier
snapshot unreconciled:

- The 8 **promoted** events above would NOT have autopaused (they read `warning` in the
  legacy snapshot). Under the shipped derivation, they DO.
- The 8 **demoted** events would have rendered at `warning` salience; under the shipped
  derivation they render at `informational` (quieter — subject to whatever
  volume-floor/dedup treatment `chronicle_salience.py` gives informational-tier events).

This is a real behavior choice the owner should confirm, not a bug — the reclassification
is mechanically forced by the pure rule (a `CROSSING` cannot be `warning`), not an arbitrary
per-event judgment call.

## Open owner questions carried into this ceremony (design §9, unresolved by U1–U7)

1. **`calibration_warning.*` / `CALIBRATION_*` kind.** These are the invariant-residual
   family census/§II.7 names as the `ALARM` exemplar. Under the pure rule `ALARM` is
   *always* `critical` — applied naively, every `CALIBRATION_*` member would newly
   autopause on a data-quality notice. `event_severity.py` defaults to classifying them
   **`FLOW`** instead (preserving their current `informational` tier) — a **flagged
   disposition, not a drift** (no rationale row exists for them in `DRIFT_TABLE` because
   their tier did not change) — pending an owner ruling on whether they are true
   invariant-residual `ALARM`s.
2. **The CROSSING-at-warning reclassification itself.** Confirmed here as the drift table
   above; owner sign-off requested specifically on the 8 promotions (new autopause
   triggers) and the 8 demotions (quieter rendering).
3. **`crisis_phase_transition`'s granularity limitation** (drift table above): one
   `EventType` covering all 6 `CreditCyclePhase` arcs, including the one terminal arc
   (`STAGNATION`), is flagged as a candidate for a future per-arc split — not resolved by
   this train.

## Verification trail

- `src/babylon/models/event_severity.py::DRIFT_TABLE` — the generated table this note
  transcribes (16 rows; `_build_drift_table` raises at import on any undeclared drift).
- `tests/unit/models/test_event_severity.py` / `test_event_severity_single_source.py` —
  U1/U6 behavioral pins, including the grep gate proving no engine/domain importer reads
  `resolve_severity`.
- `src/babylon/sentinels/seam_algebra/checks.py::check_severity_single_source` (U6) — the
  standing CI gate keeping the web bridge, the Archive Chronicle, and this generated table
  in lock-step going forward; confirmed green via `mise run check:seam-algebra` when this
  note was authored (U7, this lane).
- `mise run qa:regression` — not run from this read-only lane (machine-safety scoping keeps
  this lane to `mise run test:q`); the byte-identity claim rests on the structural argument
  above (severity never enters `WorldState`/the graph/the tick loop, confirmed by the grep
  gate) plus U2's own acceptance criterion. **Owner/merge-time verification of
  `mise run qa:regression` byte-identical is the one item this note asks the ceremony to
  confirm empirically**, not merely by construction.
