# U5 — engine-train seam contracts (pinned before code)

Status: CONTRACT — written after ADR165 (all Director rulings landed) and before any U5
code. Every sub-unit below implements against these seams; deviations get recorded here
with a dated note, the ADR162 pattern.

Ruling basis: `ai/decisions/ADR165_p26_director_rulings_trade_slate.yaml`. Estimates and
defect inventory: `u4-phi-attribution-options.md` (§7 = the ruled Option C). σ math:
`specs/107-sigma-gradient/spec.md`. Transport: `specs/108-transport-substrate/`
(spec/plan/research/tasks — its own contract set; this doc only pins its cross-seams).

## Sub-units and ordering

| Sub | What | Depends on | Surface |
|---|---|---|---|
| U5a | Core-bloc theory research (Amin/Wallerstein/MIM) → treatment rule | — | one doc |
| U5b | Ricci GVC table re-ingestion (pinned rebuild) | — | data lane |
| U5c | Disjoint taxonomy + Mexico→latin_america split | U5b placement | crosswalk/data |
| U5d | σ-index artifact + σ-composition attribution + ERDI fix | U5a, U5b, U5c | economics/persistence |
| U5e | Transport substrate slice 1 (spec-108 tasks.md) | rulings only | engine/defines/ooda |
| U5f | Tariff/duty levers via Policy/Electoral | U5e's defines regen lands first | defines/policy |
| U5g | `build_vol2_circulation_step` composer | — | game/cli |

Defines regeneration (`tools/generate_defines_config.py` rewrites the whole
`defines.yaml`) is a serialized resource: U5e's `TransportDefines` regen lands before
U5f's `TradePolicyDefines` regen. No two sub-units regenerate concurrently.

## U5d — the σ-composition attribution pipeline (Option C)

### The two-stage shape (determinism red line honored)

σ NEVER computes in-tick. Stage 1 is a build-time artifact; stage 2 is init-time
attribution; the tick path is unchanged (reads `external_nodes_phi` exactly as today).

**Stage 1 — σ-index artifact (generator now unblocked: the drive is mounted).**
`tools/make_sigma_index_artifact.py` fills the already-pinned schema of
`src/babylon/data/reference/sigma_index.parquet` (spec-107's declared generator seam;
the red-phase absence pin in
`tests/unit/domain/economics/sigma/test_contracts_red_phase.py` flips to a
presence+content pin — a deliberate red→green graduation, recorded in the ADR).
- US-side σ per BEA-industry×year from loaded tables (QCEW wage bill/employment, BEA
  I-O). K input: BEA Fixed Assets is STAGED (spec-107 D4), so the artifact uses the
  **program-10 §3 named interim flow proxy `intermediate_inputs / wage_bill`** —
  chosen because it is the proxy the program text itself names (disclosed in the
  artifact metadata and the ADR; supersedable when FAAt3.1ESI lands).
- Composition: z-score standardization + linear weighted sum, weights from
  `GameDefines.sigma` (new category, canonical `(1/3, 1/3, 1/3)`) — the ADR165 D1
  delegated decision. The artifact stamps the weights it was built with.

**Stage 2 — init-time bloc attribution.**
New module `src/babylon/domain/economics/sigma/attribution.py` (pure) +
`_query_external_nodes_phi`'s successor in `persistence/postgres_initialization.py`:
- Bloc σ is **anchored, not measured** (no foreign production data): each of the 8
  nodes maps to its Ricci region (`fact_ricci_unequal_exchange_gvc`, U5b); the bloc's
  raw world-scale coordinate derives from its region's UE flow intensity
  (`value_pct_gdp`, OUTFLOW-positive ⇒ down-gradient), passed through
  `anchor_to_world_scale` with `DistributionStats` computed over the Ricci sample.
  Nearest-vintage rule for campaign years (the four vintages 1995/2000/2007/2009):
  deterministic `max(v ≤ year, else min(v))`.
- Share functional form: `share_i ∝ max(0, σ_US − σ_i) × trade_i`, renormalized to
  Σ = 1.0 (conservation, spec-101 D3). Linear gap (p = 1) — simplest form consistent
  with "value transfer up-gradient" (program-10 §3); any exponent is a declared define
  (`sigma.attribution_gap_exponent`, default 1.0).
- **Core-bloc treatment: per the U5a research doc's concrete rule** (slot pinned here;
  the rule text lands in `u5a-core-bloc-theory.md` and is cited by the implementation —
  the max(0, ·) clamp already zeroes blocs at-or-above the US position; U5a decides
  whether a damped nonzero `w_core` exists and any semi-periphery nuance).
- `trade_i` comes from the **disjoint crosswalk (U5c)** — never the containing blocs.
- ERDI fix (Q7): `_fetch_node_erdi` repaired to read the national Hickel series
  (`scale_type` row matching, national-only disclosed in the docstring); σ remains the
  attribution driver; `erdi_ratio` becomes an honest observational field.

### U5c — disjoint crosswalk contract
- `_NODE_TO_BLOC` is replaced by `_NODE_TO_PARTNERS: dict[str, tuple[int, ...]]` —
  each node maps to a DISJOINT set of `dim_country` ids covering it exactly once
  (no "Europe"⊃"EU", no China double-count via "Asia"+"Pacific Rim").
- Mexico moves to `latin_america` (ADR165 Q3): requires a Mexico country row with real
  Census annual values (U3 loader pattern) and `canada` keyed to actual-Canada rows.
- Denominator law (pinned by test): Σ over the 8 nodes' partner trade ≤ world total,
  and no `dim_country` id appears under two nodes.

## U5e — transport slice 1 (cross-seams only; spec-108 tasks.md governs internals)
- New `TransportDefines` category (corridor capacity/condition/decay, demand-signal
  threshold params, BUILD_INFRASTRUCTURE costs) + damped overhang coupling coefficient
  homed in `CapitalVolumeIIDefines` (ADR165 108-3).
- Flux overlay = demand signal into the sovereign's OODA budget evaluation;
  BUILD_INFRASTRUCTURE is the only mint/repair path (108-2). No autonomous INFORMAL
  minting in slice 1.
- Attack seam: uniform territory splash through the existing
  `_propagate_infrastructure` write (108-4).
- Output for U6: aggregated per-county-pair connectivity coefficient exposed via a
  session-reachable read (feeds the dossier "supply lines" indicator, 108-5).
- Expected qa/vault drift when systems land ⇒ declared §6.5 ceremony, not a surprise.

## U5f — tariff levers (P25↔P26 coupling, ADR165 directive)
- New `TradePolicyDefines` category: per-node tariff rate, import duty, trade-tax
  coefficients (defaults 0.0 ⇒ byte-identical pre-U5f behavior).
- Consumption: the attribution/trade dataflow reads effective trade values as
  `trade_i × (1 − tariff_dampening(rate_i))` at init/re-init; LEGISLATE (Policy
  @17.47) motions can move the rates subject to the reform ceiling; electoral
  outcomes shift them via the existing Policy resolver seam — no new system position.
- Contract: rates are graph/session state once a campaign starts (defines give the
  START values); changes flow through PolicySystem's existing resolution, never a
  direct define mutation mid-campaign.

## U5g — Vol2 composer
- `build_vol2_circulation_step(...)` in `src/babylon/game/vol2.py`: composes the
  LODES tri-county artifact kwargs (`resolve_lodes_hydration_kwargs`) + hex-county
  adjunction (`read_hex_county_adjunction`) into a `Vol2CirculationStep`.
- Honest-absence law: interactive campaigns without hex hydration get `None` (and one
  loud warning), NEVER a vacuous step walking empty adjunctions. `cli/play.py` passes
  the result into `build_interactive_trade_wiring(vol2_step=…)`.
- Closes ADR162's blocking-dependency citation; the seam row already names
  `game/session.py` as supplier.

## U6 phase 2 (M3 covenant lifted — PR #316 merged)
- Textual (default client): trade overview + bloc dossier pages rendered from
  `subject_view("trade", …)` — the phase-1 seam, untouched.
- Rust lane: `trade_view_json` Host-trait method (call1, single-JSON-arg — the M2
  pattern) delegating to the same projection; render per the M2/M3 view conventions.
- The 108-5 connectivity indicator joins the county dossier once U5e's coefficient
  read exists; if U5e lands after, the indicator ships in the same commit as its data
  (no dead UI).

## Gates
Every sub-unit: scoped `mise run test:q` on its surface + ruff + mypy strict before its
commit. Train-final: `mise run check`, `qa:regression`, `qa:vault-regression-ci`;
intentional drift (U5e systems) via declared ceremony. ADRs: one per landed sub-unit
(166+). PR #315 carries the train.
