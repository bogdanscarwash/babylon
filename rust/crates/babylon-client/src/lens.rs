//! The three live map lenses (Program 28 B2, Phase C — completes B1's
//! never-built Phase C, generalized to three lenses): `county_tension`
//! (ADR170, Task 8), `county_legitimation` (Task 9), and
//! `county_population_trend` (Task 9b — the one that genuinely moves every
//! tick on this demo content, BLOCKER 2 fix). Each lens computes a raw
//! value only; `map/bands.rs` (Task 10) owns the color mapping.

use crate::projection::Projector;
use babylon_graph::substrate::{GraphSubstrate, NodeId};

/// One lens's live reading: a `(fips, value)` cell per county the lens
/// knows about (`None` = no honest data this tick, never a fabricated
/// zero), plus an optional whole-lens absence reason. Shared shape all
/// three lenses return.
pub struct LensReading {
    pub cells: Vec<(String, Option<f64>)>,
    pub absent_reason: Option<String>,
}

/// The live `LensReading`s the map can show, one per row of
/// `crate::map::LENSES` and in the SAME order (B3 wave-1 Task 8, §2.10) —
/// `crate::map::ActiveLens(i)` indexes both in lockstep. Refreshed together
/// on every tick advance via `loop_ui::build_lens_data`, the one place that
/// iterates `LENSES` to fill this. `spawn_engine_session_and_hud` (Task 14)
/// is the ONLY inserter, always with a fully-computed literal, in `Startup`
/// — strictly before `recolor_on_lens_changed` (an `Update` system) can
/// ever run, so no earlier reader can observe a missing resource. No
/// `Default` derive on purpose: nothing may construct this half-built. The
/// inner `Vec` is `pub` (not wrapped behind an accessor) so both in-crate
/// readers (`map::bands`/`map::hud`) and `tests/lens_registry.rs` (an
/// external test crate) can index and measure it directly.
#[derive(bevy::prelude::Resource)]
pub struct CurrentLensData(pub Vec<LensReading>);

/// Bundles what a lens's `compute` function reads — the live graph, the
/// per-fips node roster (empty for a `MapBinding::None` story, the same
/// "no branch needed" shape `loop_ui::build_lens_data`'s own doc comment
/// already established), and each territory's tick-0 population baseline —
/// unifying the three previously-distinct signatures
/// (`county_tension`/`county_legitimation`/`county_population_trend` used
/// to take `&dyn GraphSubstrate`, that plus `node_by_fips`, and that plus
/// `node_by_fips` plus `baseline`, respectively) into the one
/// `fn(&LensInputs<'_>) -> LensReading` `crate::map::LensSpec::compute`
/// needs (B3 wave-1 Task 8, §2.10).
pub struct LensInputs<'a> {
    pub graph: &'a dyn GraphSubstrate,
    pub roster: &'a [(String, NodeId)],
    pub baseline: &'a [(String, f64)],
}

/// Below this, `phi + theta` is the degenerate all-bled-dry limit and `w`
/// collapses to `0.0` — the shared measure kernel's own convention
/// (`babylon.formulas.contradiction.calculate_wealth_asymmetry_gap`,
/// mirrored here for the Rust witness).
const DEGENERATE_EPS: f64 = 1e-9;

/// The two per-territory stamps `v = s / e` recovers from. **No field
/// registry ports these yet** — no shipped rule pack in this repo writes
/// them (confirmed: `vitality`/`lifecycle` touch only their own
/// `social-class/*`/`territory/*` fields, none of these two) — so this
/// lens is UNCONDITIONALLY absent on every scenario this plan ships,
/// exactly as Task 8's own "Related finding" states. Named here, in
/// kebab-case matching every other `territory/*` field in this crate, so
/// the day an economics BSL port lands these two names are the ones it
/// should write.
///
/// **`pub`, not private (adversarial-panel finding FB7; widened from
/// `pub(crate)` to genuinely-external `pub` in B3 wave-1 Task 8, §2.10,
/// closing the gap between this doc's own long-standing claim below and
/// the visibility keyword that used to fall short of it).** B1 Task 8 Step
/// 2's own instruction — "declare them as const strings in one place that
/// both use" — assumed a real scenario would exist to share this spelling
/// with; none does yet, so these two names currently exist NOWHERE else in
/// the repository (no `.bscn`/`.bsl` file declares or writes them) and are
/// pure invention, at real risk of a future economics-port author
/// independently choosing different spellings. Widened visibility so a
/// future port can `use babylon_client::lens::{TENSION_E_FIELD,
/// TENSION_S_FIELD}` directly instead of re-guessing — the ONE canonical
/// place this plan's own discipline asks for, even though nothing can
/// force a future `.bsl` file to import a Rust const. Also now the same
/// cross-check `tests/lens_registry.rs` uses to prove every
/// `LensSpec::help` names a real field (Task 8's own RED gate). Tracked on
/// #503 (the economics-port follow-up), alongside the `node_by_fips`-shaped
/// fips-mapping gap `county_tension`'s own doc comment already names.
pub const TENSION_E_FIELD: &str = "territory/tick-exploitation-rate";
pub const TENSION_S_FIELD: &str = "territory/tick-total-surplus";

/// The ADR170 `county_extraction` witness, transcribed from
/// `src/babylon/projection/topology/tension.py` unchanged:
/// `phi = v/(v+s)`, `theta = sum(v)/sum(v+s)` (a ratio of sums, never a
/// mean), `w = (phi-theta)/(phi+theta)`, collapsing to `0.0` at
/// `phi+theta <= 1e-9`. A contribution needs BOTH `s > 0` and `e > 0` (the
/// un-hydrated fallback's poisoned zero reads as absence, never as zero
/// tension) — `v = s / e`.
///
/// **Known gap, stated rather than hidden.** Like the other two lenses
/// below (B3 wave-1 Task 8, §2.10 unified all three onto the one
/// `LensInputs` signature), this function receives a full `LensInputs` —
/// but unlike them, it reads only `inputs.graph`, never `inputs.roster`.
/// `GraphSubstrate` has no method recovering a node's FIPS string from its
/// `NodeId` alone (attributes are `f64`-only; §2.9 gives no string-valued
/// attribute a home), so even with a roster in hand this function could not
/// key its cells by fips the way `county_legitimation`/
/// `county_population_trend` do. This function's cell keys are therefore
/// each territory's `NodeId` rendered as a decimal string, NOT a real FIPS
/// code. This is harmless today because the lens is unconditionally absent
/// on this plan's own demo content (no rule pack writes the two fields
/// above) — `map/bands.rs`'s recolor system looks up
/// `atlas.index_of_fips(fips)` per cell and silently skips any key that
/// does not resolve, which every one of these decimal-string keys does. The
/// day a real economics port seeds these fields, whoever wires that content
/// live needs a real fips mapping here too (the same shape
/// `county_legitimation`/`county_population_trend` already carry) —
/// flagged here, not silently worked around.
#[must_use]
pub fn county_tension(inputs: &LensInputs<'_>) -> LensReading {
    let territories = inputs.graph.nodes("TERRITORY");

    // (id, v, new_value) for every territory that actually contributes.
    let mut contributions: Vec<(NodeId, f64, f64)> = Vec::new();
    let projector = Projector::material();
    for &id in &territories {
        // B3 wave-1 Task 3 (plan §2.6): retargeted through the seam —
        // `.value` is `Option<f64>` exactly like the `.ok()` this replaces
        // (neither field is in the projector's `NotComputed` table, so
        // behavior is unchanged: `Material` -> `Some`, `Absent` -> `None`).
        let e = projector.read(inputs.graph, id, TENSION_E_FIELD).value;
        let s = projector.read(inputs.graph, id, TENSION_S_FIELD).value;
        if let (Some(e), Some(s)) = (e, s) {
            if e > 0.0 && s > 0.0 {
                let v = s / e;
                contributions.push((id, v, v + s));
            }
        }
    }

    if contributions.is_empty() {
        return LensReading {
            cells: territories
                .iter()
                .map(|id| (id.0.to_string(), None))
                .collect(),
            absent_reason: Some(
                "no territory carries a positive exploitation-rate/surplus stamp this tick — \
                 no norm exists"
                    .to_owned(),
            ),
        };
    }

    let total_v: f64 = contributions.iter().map(|(_, v, _)| v).sum();
    let total_new_value: f64 = contributions.iter().map(|(_, _, nv)| nv).sum();
    let theta = total_v / total_new_value;

    let phi_by_id: std::collections::HashMap<NodeId, f64> = contributions
        .into_iter()
        .map(|(id, v, nv)| (id, v / nv))
        .collect();

    let cells = territories
        .iter()
        .map(|&id| {
            let key = id.0.to_string();
            match phi_by_id.get(&id) {
                Some(&phi) => {
                    let denom = phi + theta;
                    let w = if denom <= DEGENERATE_EPS {
                        0.0
                    } else {
                        (phi - theta) / denom
                    };
                    (key, Some(w.clamp(-1.0, 1.0)))
                }
                None => (key, None),
            }
        })
        .collect();

    LensReading {
        cells,
        absent_reason: None,
    }
}

/// The `lifecycle` rule pack's own encoded classification (its header
/// comment documents this: 0 = STABLE, 1 = UNSTABLE, 2 = CRISIS). `pub`
/// (widened in B3 wave-1 Task 8, §2.10) — `tests/lens_registry.rs` cross-
/// checks every `LensSpec::help` against this and the other field-name
/// consts in this file, the same discipline `TENSION_E_FIELD`'s own doc
/// comment already established.
pub const LEGITIMATION_CRISIS_FIELD: &str = "territory/legitimation-crisis";

/// `territory/legitimation-crisis`'s three closed values. A straight
/// categorical pass-through — no new cut point, no new math (the standing
/// "no imposed functional forms" ruling).
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum LegitimationClass {
    Stable,
    Unstable,
    Crisis,
}

/// A plain three-arm match on the encoded float. The encoding is a CLOSED
/// set the rule pack itself defines — anything else is a loud panic, never
/// a silent fallback.
///
/// # Panics
/// If `raw` is not exactly `0.0`, `1.0`, or `2.0` — `lifecycle.bsl`'s
/// encoding is a closed set with no other legal value.
#[must_use]
// The rule pack's encoding is exactly the three integral floats 0.0/1.0/2.0
// (this doc comment, `lifecycle.bsl`'s header) — an approximate comparison
// would silently reclassify a real wiring bug (any other float) as one of
// the three legitimate classes instead of panicking. Same posture as
// `babylon-bsl::evaluator`'s own exact-comparison arm.
#[allow(clippy::float_cmp)]
pub fn classify(raw: f64) -> LegitimationClass {
    if raw == 0.0 {
        LegitimationClass::Stable
    } else if raw == 1.0 {
        LegitimationClass::Unstable
    } else if raw == 2.0 {
        LegitimationClass::Crisis
    } else {
        panic!("territory/legitimation-crisis read an out-of-encoding value: {raw}")
    }
}

/// Reads `territory/legitimation-crisis` for every `(fips, id)` pair in
/// `inputs.roster` and returns `Some(raw_class_as_f64)` per cell — Task
/// 10's `bands.rs` owns the color mapping, matching the Tension lens's own
/// separation of "compute the value" from "pick the color."
///
/// A `roster` entry naming a `NodeId` this field has never been written on
/// is a WIRING BUG, not an honest absence — unlike Tension's "this county
/// may honestly carry no data," the Phase B demo scenario controls the
/// whole node set and declares this field on every one of its twelve
/// territories. Such an entry panics loudly (III.11) rather than silently
/// reporting `None` — only a FIPS that never appears in `inputs.roster` at
/// all (any of the 3,210 non-demo counties) is the honest "outside the
/// demo, no data this tick" absence, and it never reaches this function in
/// the first place (the caller only ever passes the demo's own roster).
///
/// # Panics
/// If a `roster` entry names a `NodeId` this field was never written on —
/// a wiring bug, not an honest absence (see above).
#[must_use]
pub fn county_legitimation(inputs: &LensInputs<'_>) -> LensReading {
    let projector = Projector::material();
    let cells = inputs
        .roster
        .iter()
        .map(|(fips, id)| {
            // B3 wave-1 Task 3 (plan §2.6): retargeted through the seam.
            // `LEGITIMATION_CRISIS_FIELD` never appears in the projector's
            // `NotComputed` table, so a `None` here means exactly what the
            // old `.unwrap_or_else` panic guarded against — a wiring bug,
            // not an honest absence.
            let reading = projector.read(inputs.graph, *id, LEGITIMATION_CRISIS_FIELD);
            let raw = reading.value.unwrap_or_else(|| {
                panic!(
                    "demo county {fips} (NodeId {id:?}) has no {LEGITIMATION_CRISIS_FIELD} stamp \
                     — this is a wiring bug (the Phase B scenario declares this field on every \
                     territory), not an honest absence: {:?}",
                    reading.provenance
                )
            });
            (fips.clone(), Some(raw))
        })
        .collect();

    LensReading {
        cells,
        absent_reason: None,
    }
}

/// The DPD circuit's three genuinely per-tick, per-territory population
/// fields — the ones `lifecycle.bsl`'s Block 1 actually writes, unlike
/// Block 2's const-only legitimation math (Task 9's own finding). Summed,
/// they give a territory's total population.
pub const POP_D_FIELD: &str = "territory/pop-d";
pub const POP_P_FIELD: &str = "territory/pop-p";
pub const POP_D_PRIME_FIELD: &str = "territory/pop-d-prime";

/// For each `(fips, id)` in `inputs.roster`, sums `pop-d + pop-p +
/// pop-d-prime` off the LIVE graph and reports its signed delta against
/// `inputs.baseline`'s own entry for that fips — `baseline` is each demo
/// territory's tick-0 total population, captured once by
/// `EngineSession::start` before any `advance()` runs (Task 13). No sign
/// normalization, no size threshold, no clamping — the raw signed delta
/// travels to `map/bands.rs` (Task 10), which does the classification.
///
/// A `fips` present in `inputs.roster` but ABSENT from `inputs.baseline` is
/// a wiring bug (both come from the same `EngineSession::start` call) and
/// panics loudly, never resolving to a silent `None` — the same
/// strictness `county_legitimation` applies to its own roster mismatch
/// case. A FIPS outside BOTH slices (any of the 3,210 non-demo counties)
/// never reaches this function at all — the caller only ever passes the
/// demo's own roster/baseline pair, so that absence is realized by the
/// recolor system simply never touching that county's mesh vertices, not
/// by a cell this function emits.
///
/// # Panics
/// If a roster entry's fips is missing from `inputs.baseline` — a wiring
/// bug, since both come from the same `EngineSession::start` call (see
/// above).
#[must_use]
pub fn county_population_trend(inputs: &LensInputs<'_>) -> LensReading {
    let projector = Projector::material();
    let cells = inputs
        .roster
        .iter()
        .map(|(fips, id)| {
            // B3 wave-1 Task 3 (plan §2.6): retargeted through the seam,
            // `.value.unwrap_or(0.0)` preserving the pre-existing fallback
            // exactly (none of the three fields is in the projector's
            // `NotComputed` table, so behavior is unchanged).
            let pop_d = projector
                .read(inputs.graph, *id, POP_D_FIELD)
                .value
                .unwrap_or(0.0);
            let pop_p = projector
                .read(inputs.graph, *id, POP_P_FIELD)
                .value
                .unwrap_or(0.0);
            let pop_d_prime = projector
                .read(inputs.graph, *id, POP_D_PRIME_FIELD)
                .value
                .unwrap_or(0.0);
            let now = pop_d + pop_p + pop_d_prime;

            let baseline_total = inputs
                .baseline
                .iter()
                .find(|(baseline_fips, _)| baseline_fips == fips)
                .unwrap_or_else(|| {
                    panic!(
                        "demo county {fips} is in the roster but missing from baseline — \
                         this is a wiring bug (both come from the same EngineSession::start \
                         call), not an honest absence"
                    )
                })
                .1;

            (fips.clone(), Some(now - baseline_total))
        })
        .collect();

    LensReading {
        cells,
        absent_reason: None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use babylon_graph::hypergraph_store::HypergraphStore;

    fn territory_with(graph: &mut HypergraphStore, e: f64, s: f64) -> NodeId {
        let id = graph.add_node("TERRITORY").expect("add territory");
        graph.update_node(id, TENSION_E_FIELD, e).expect("stamp e");
        graph.update_node(id, TENSION_S_FIELD, s).expect("stamp s");
        id
    }

    /// B3 wave-1 Task 8 (§2.10): every lens's `compute` now takes one
    /// `&LensInputs<'_>` — the Tension tests below never touch
    /// `roster`/`baseline`, so this helper hands them empty slices.
    fn tension_inputs(graph: &HypergraphStore) -> LensInputs<'_> {
        LensInputs {
            graph,
            roster: &[],
            baseline: &[],
        }
    }

    /// (a) Two clean-stamped territories: `theta` (the ratio of sums) must
    /// differ from the mean of the two `phi`s — the single most likely
    /// transcription slip (using a mean instead of a ratio-of-sums).
    #[test]
    fn theta_is_the_ratio_of_sums_not_the_mean_of_phis() {
        let mut graph = HypergraphStore::new();
        // territory 1: e=2, s=10 -> v=5, new_value=15, phi1=1/3
        let id1 = territory_with(&mut graph, 2.0, 10.0);
        // territory 2: e=10, s=100 -> v=10, new_value=110, phi2=10/110
        let id2 = territory_with(&mut graph, 10.0, 100.0);

        let reading = county_tension(&tension_inputs(&graph));
        assert!(reading.absent_reason.is_none());

        let phi1: f64 = 5.0 / 15.0;
        let phi2: f64 = 10.0 / 110.0;
        let mean_of_phis: f64 = f64::midpoint(phi1, phi2);
        let theta_ratio_of_sums: f64 = (5.0 + 10.0) / (15.0 + 110.0);
        assert!(
            (mean_of_phis - theta_ratio_of_sums).abs() > 1e-6,
            "fixture must make the two candidate thetas differ, or this test cannot \
             distinguish them"
        );

        let w1 = cell_value(&reading, id1);
        let w2 = cell_value(&reading, id2);
        let expected_w1 = {
            let denom = phi1 + theta_ratio_of_sums;
            ((phi1 - theta_ratio_of_sums) / denom).clamp(-1.0, 1.0)
        };
        let expected_w2 = {
            let denom = phi2 + theta_ratio_of_sums;
            ((phi2 - theta_ratio_of_sums) / denom).clamp(-1.0, 1.0)
        };
        assert!(
            (w1 - expected_w1).abs() < 1e-9,
            "got {w1}, want {expected_w1}"
        );
        assert!(
            (w2 - expected_w2).abs() < 1e-9,
            "got {w2}, want {expected_w2}"
        );

        // The mean-of-phis alternative would have produced a DIFFERENT w1 —
        // proving the implementation used the ratio of sums, not the mean.
        let wrong_w1 = {
            let denom = phi1 + mean_of_phis;
            ((phi1 - mean_of_phis) / denom).clamp(-1.0, 1.0)
        };
        assert!((w1 - wrong_w1).abs() > 1e-6);
    }

    /// (b) A net Phi-source (bled) territory scores `w < 0`; a net
    /// Phi-recipient (bribed) territory scores `w > 0`.
    #[test]
    fn bled_scores_negative_and_bribed_scores_positive() {
        let mut graph = HypergraphStore::new();
        // Low phi (wage share far below the norm) -> bled, w < 0.
        let bled = territory_with(&mut graph, 100.0, 1.0); // v=0.01, phi≈0.0099
                                                           // High phi (wage share far above the norm) -> bribed, w > 0.
        let bribed = territory_with(&mut graph, 1.0, 100.0); // v=100, phi≈0.99

        let reading = county_tension(&tension_inputs(&graph));
        assert!(cell_value(&reading, bled) < 0.0);
        assert!(cell_value(&reading, bribed) > 0.0);
    }

    /// (c) A territory with `s > 0, e == 0` contributes nothing to `theta`
    /// and reports `None` for its own cell, while a genuinely-stamped
    /// sibling still resolves.
    #[test]
    fn a_zero_exploitation_rate_with_positive_surplus_contributes_nothing() {
        let mut graph = HypergraphStore::new();
        let good = territory_with(&mut graph, 2.0, 10.0);
        let broken = territory_with(&mut graph, 0.0, 5.0);

        let reading = county_tension(&tension_inputs(&graph));
        assert!(
            reading.absent_reason.is_none(),
            "the good territory still carries data"
        );
        assert!(cell_value_opt(&reading, good).is_some());
        assert!(cell_value_opt(&reading, broken).is_none());
    }

    /// (d) A graph where every territory fails the `s>0 && e>0` gate (the
    /// un-hydrated-fallback shape) yields whole-lens absence and every
    /// cell `None`.
    #[test]
    fn no_data_bearing_territory_yields_whole_lens_absence() {
        let mut graph = HypergraphStore::new();
        territory_with(&mut graph, 0.0, 0.0);
        territory_with(&mut graph, 0.0, 0.0);

        let reading = county_tension(&tension_inputs(&graph));
        assert!(reading.absent_reason.is_some());
        assert_eq!(reading.cells.len(), 2);
        assert!(reading.cells.iter().all(|(_, v)| v.is_none()));
    }

    /// (e) Every returned `w` lands inside `[-1, 1]`, even under an extreme
    /// stamp that would overflow the raw formula without the clamp.
    #[test]
    fn every_w_lands_in_unit_interval() {
        let mut graph = HypergraphStore::new();
        territory_with(&mut graph, 0.0001, 1_000_000.0); // huge v
        territory_with(&mut graph, 1_000_000.0, 0.0001); // tiny v

        let reading = county_tension(&tension_inputs(&graph));
        for (_, w) in &reading.cells {
            if let Some(w) = w {
                assert!((-1.0..=1.0).contains(w), "w {w} out of [-1, 1]");
            }
        }
    }

    fn cell_value_opt(reading: &LensReading, id: NodeId) -> Option<f64> {
        reading
            .cells
            .iter()
            .find(|(k, _)| k == &id.0.to_string())
            .and_then(|(_, v)| *v)
    }

    fn cell_value(reading: &LensReading, id: NodeId) -> f64 {
        cell_value_opt(reading, id).expect("cell must carry a value")
    }

    fn territory_with_crisis_class(graph: &mut HypergraphStore, class: f64) -> NodeId {
        let id = graph.add_node("TERRITORY").expect("add territory");
        graph
            .update_node(id, LEGITIMATION_CRISIS_FIELD, class)
            .expect("stamp legitimation-crisis");
        id
    }

    #[test]
    fn classify_maps_the_three_encoded_values() {
        assert_eq!(classify(0.0), LegitimationClass::Stable);
        assert_eq!(classify(1.0), LegitimationClass::Unstable);
        assert_eq!(classify(2.0), LegitimationClass::Crisis);
    }

    #[test]
    #[should_panic(expected = "out-of-encoding value")]
    fn classify_panics_loudly_on_an_out_of_encoding_value() {
        let _ = classify(3.0);
    }

    #[test]
    fn county_legitimation_reads_back_the_raw_encoded_class_per_fips() {
        let mut graph = HypergraphStore::new();
        let stable = territory_with_crisis_class(&mut graph, 0.0);
        let unstable = territory_with_crisis_class(&mut graph, 1.0);
        let crisis = territory_with_crisis_class(&mut graph, 2.0);
        let node_by_fips = vec![
            ("00001".to_owned(), stable),
            ("00002".to_owned(), unstable),
            ("00003".to_owned(), crisis),
        ];

        let reading = county_legitimation(&LensInputs {
            graph: &graph,
            roster: &node_by_fips,
            baseline: &[],
        });
        assert!(reading.absent_reason.is_none());
        assert_eq!(reading.cells.len(), 3);
        assert_eq!(reading.cells[0], ("00001".to_owned(), Some(0.0)));
        assert_eq!(reading.cells[1], ("00002".to_owned(), Some(1.0)));
        assert_eq!(reading.cells[2], ("00003".to_owned(), Some(2.0)));
        assert_eq!(
            classify(reading.cells[0].1.unwrap()),
            LegitimationClass::Stable
        );
        assert_eq!(
            classify(reading.cells[1].1.unwrap()),
            LegitimationClass::Unstable
        );
        assert_eq!(
            classify(reading.cells[2].1.unwrap()),
            LegitimationClass::Crisis
        );
    }

    /// A `node_by_fips` entry naming a `NodeId` the graph never minted (or
    /// never stamped this field on) is a wiring bug — it must panic loudly,
    /// never resolve to a silent `None`.
    #[test]
    #[should_panic(expected = "wiring bug")]
    fn a_node_by_fips_entry_with_no_matching_stamp_panics_loudly() {
        let graph = HypergraphStore::new();
        let node_by_fips = vec![("99999".to_owned(), NodeId(0))];
        let _ = county_legitimation(&LensInputs {
            graph: &graph,
            roster: &node_by_fips,
            baseline: &[],
        });
    }

    fn territory_with_population(
        graph: &mut HypergraphStore,
        pop_d: f64,
        pop_p: f64,
        pop_d_prime: f64,
    ) -> NodeId {
        let id = graph.add_node("TERRITORY").expect("add territory");
        graph
            .update_node(id, POP_D_FIELD, pop_d)
            .expect("stamp pop-d");
        graph
            .update_node(id, POP_P_FIELD, pop_p)
            .expect("stamp pop-p");
        graph
            .update_node(id, POP_D_PRIME_FIELD, pop_d_prime)
            .expect("stamp pop-d-prime");
        id
    }

    #[test]
    fn a_territory_above_its_baseline_reports_a_positive_delta() {
        let mut graph = HypergraphStore::new();
        let id = territory_with_population(&mut graph, 100.0, 200.0, 50.0); // now = 350
        let node_by_fips = vec![("00001".to_owned(), id)];
        let baseline = vec![("00001".to_owned(), 300.0)];

        let reading = county_population_trend(&LensInputs {
            graph: &graph,
            roster: &node_by_fips,
            baseline: &baseline,
        });
        assert_eq!(reading.cells, vec![("00001".to_owned(), Some(50.0))]);
    }

    #[test]
    fn a_territory_below_its_baseline_reports_a_negative_delta() {
        let mut graph = HypergraphStore::new();
        let id = territory_with_population(&mut graph, 100.0, 100.0, 50.0); // now = 250
        let node_by_fips = vec![("00001".to_owned(), id)];
        let baseline = vec![("00001".to_owned(), 300.0)];

        let reading = county_population_trend(&LensInputs {
            graph: &graph,
            roster: &node_by_fips,
            baseline: &baseline,
        });
        assert_eq!(reading.cells, vec![("00001".to_owned(), Some(-50.0))]);
    }

    #[test]
    fn a_territory_exactly_at_its_baseline_reports_exactly_zero() {
        let mut graph = HypergraphStore::new();
        let id = territory_with_population(&mut graph, 100.0, 100.0, 100.0); // now = 300
        let node_by_fips = vec![("00001".to_owned(), id)];
        let baseline = vec![("00001".to_owned(), 300.0)];

        let reading = county_population_trend(&LensInputs {
            graph: &graph,
            roster: &node_by_fips,
            baseline: &baseline,
        });
        assert_eq!(reading.cells, vec![("00001".to_owned(), Some(0.0))]);
    }

    /// A fips in `node_by_fips` but missing from `baseline` is a wiring bug
    /// (both come from the same `EngineSession::start` call) — panic
    /// loudly, never a silent `None`.
    #[test]
    #[should_panic(expected = "wiring bug")]
    fn a_fips_missing_from_baseline_panics_loudly() {
        let mut graph = HypergraphStore::new();
        let id = territory_with_population(&mut graph, 100.0, 100.0, 100.0);
        let node_by_fips = vec![("00001".to_owned(), id)];
        let baseline: Vec<(String, f64)> = vec![]; // no matching entry
        let _ = county_population_trend(&LensInputs {
            graph: &graph,
            roster: &node_by_fips,
            baseline: &baseline,
        });
    }
}
