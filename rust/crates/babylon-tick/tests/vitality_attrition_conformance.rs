//! Posture suite for the K=16 wealth-mass carrier (#491 T4, Phase 1 —
//! "the carrier, inert"; design doc §6.2 H1; ADR194 R1) **plus** the T5
//! (Phase 3a) conformance suite for its first real consumer,
//! `vitality/subsistence-clearing` — the dual measure `clearing`/
//! `failing_certain`/`straddle_band` (H2', design doc §6.2; ADR173's
//! P(S|A)).
//!
//! # What this file is, and is not
//!
//! `content/scenarios/vitality-attrition-conformance.bscn` declares and
//! seeds the carrier — sixteen per-class wealth-mass shares, the fifteen
//! grid cuts, the household→person crossing (`η`), and the subsistence
//! horizon (`τ`). T4's own load-time posture tests below (unchanged)
//! still read those raw substrate facts directly through
//! [`babylon_graph::substrate::GraphSubstrate`], never through a rule's
//! binding machinery — Phase 1 supplies the carrier, not the derivation
//! (design doc §12 item 1). **T5 gives the carrier its first real rule**
//! (`content/rules/vitality-attrition.bsl`, replacing the never-firing
//! probe): it fires for the four classes whose guard admits them (core,
//! bourgeoisie, hermit, last-worker), and its ONLY observable channel is
//! `emit` — no `update-node`/`update-edge`/`update-hyperedge` verb
//! appears anywhere in the rule, so the K=16 carrier's own state-hash pin
//! stays byte-identical to T4's measurement even though the rule now
//! fires for real (T5.7: "a binding and a condition, no effect"). The
//! measure-arithmetic tests below therefore read `sink.events`, not
//! post-tick graph state.
//!
//! # File deviation, disclosed (beyond the brief's own "Files:" line)
//!
//! The task brief's own "Files:" line (transcribed verbatim from the
//! plan, `docs/superpowers/plans/2026-08-17-491-rung-ladder.md:1293`)
//! names `content/rules/vitality.bsl` as T5's rule target. That file
//! cannot host `vitality/subsistence-clearing`: it is loaded VERBATIM by
//! `vitality_conformance.rs`'s and `tick_goldens.rs`'s PINNED tests
//! together with `vitality-conformance.bscn`, which declares none of the
//! sixteen `wealth-mass-*` fields, the fifteen `cut-*` defconsts, or
//! `vitality/subsistence-horizon` — an undeclared qname is `E-LOAD-010`
//! UNCONDITIONALLY (`bindings.rs:220`, `:optional` or not), so landing
//! this rule there would break all eighteen pre-existing pins AT LOAD,
//! not drift them. `vitality-attrition.bsl` (this file's own `RULE`
//! const, unchanged path) already declares every construct this rule
//! reads, and its own T4 header already named itself as what "a future
//! task may extend in place or replace outright." See the rule file's own
//! header for the full citation chain.
//!
//! # Unit-system authority
//!
//! Every construct this file names — `wealth-mass-k`, `cut-k`, `η`, `τ` —
//! is DEFINED in `reports/subsistence-unit-reconciliation-2026-08-17.md`
//! (§3 the unit table, §3.1 the crossing, §4 τ, §9 the Aleksandrov trace).
//! This file cites that record; it does not re-derive it.
//!
//! # The five posture legs (§9/T4.1, verbatim)
//!
//! 1. A seeded class reads all 16 masses and they sum to exactly `1.0` in
//!    the stored `f64` (largest-remainder apportionment is I1's law —
//!    this hand-authored fixture's own values are chosen to sum exactly
//!    without needing any apportionment at all: every class's nonzero
//!    entries are binary64-exact dyadic fractions, so no residue exists
//!    to apportion).
//! 2. An UNSEEDED class's mass read is ABSENT (III.11) and the
//!    optional-bind sum is exactly `0.0`.
//! 3. The 15 cuts are monotone non-decreasing and strictly positive.
//! 4. `0.0r` or a negative cut is `E-LEX-027`.
//! 5. (I4, register row) `coefficient`'s `[0,1]` store-boundary domain
//!    (`E-EVAL-020`) already covers every `wealth-mass-*` field by virtue
//!    of its declared type — no new mechanism, cited not re-tested here.

use babylon_bsl::evaluator::Value;
use babylon_bsl::scenario::load_scenario;
use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_kernel::currency::Currency;
use babylon_tick::{hex, run_once, run_once_into};

const SCENARIO: &str = include_str!("../content/scenarios/vitality-attrition-conformance.bscn");
const RULE: &str = include_str!("../content/rules/vitality-attrition.bsl");

/// The 16 mass field qnames, rung 1 through rung 16, in declared order.
const MASS_FIELDS: [&str; 16] = [
    "social-class/wealth-mass-01",
    "social-class/wealth-mass-02",
    "social-class/wealth-mass-03",
    "social-class/wealth-mass-04",
    "social-class/wealth-mass-05",
    "social-class/wealth-mass-06",
    "social-class/wealth-mass-07",
    "social-class/wealth-mass-08",
    "social-class/wealth-mass-09",
    "social-class/wealth-mass-10",
    "social-class/wealth-mass-11",
    "social-class/wealth-mass-12",
    "social-class/wealth-mass-13",
    "social-class/wealth-mass-14",
    "social-class/wealth-mass-15",
    "social-class/wealth-mass-16",
];

/// The 15 cut defconst qnames, in declared order.
const CUT_CONSTS: [&str; 15] = [
    "wealth-sketch/cut-01",
    "wealth-sketch/cut-02",
    "wealth-sketch/cut-03",
    "wealth-sketch/cut-04",
    "wealth-sketch/cut-05",
    "wealth-sketch/cut-06",
    "wealth-sketch/cut-07",
    "wealth-sketch/cut-08",
    "wealth-sketch/cut-09",
    "wealth-sketch/cut-10",
    "wealth-sketch/cut-11",
    "wealth-sketch/cut-12",
    "wealth-sketch/cut-13",
    "wealth-sketch/cut-14",
    "wealth-sketch/cut-15",
];

/// Node ids follow scenario declaration order (`scenario.rs`: "declaration
/// order is the id order") — core=0, bourgeoisie=1, hermit=2,
/// last-worker=3, remnant=4, dissolved=5, calibration=6 (T6's fixture,
/// declared last).
const CORE: u64 = 0;
const BOURGEOISIE: u64 = 1;
const HERMIT: u64 = 2;
const LAST_WORKER: u64 = 3;
const REMNANT: u64 = 4;
const DISSOLVED: u64 = 5;
const CALIBRATION: u64 = 6;

/// The six classes carrying an explicit 16-value mass vector (everyone
/// except `remnant`, the absence fence's own subject).
const SEEDED_CLASSES: [(&str, u64); 6] = [
    ("core", CORE),
    ("bourgeoisie", BOURGEOISIE),
    ("hermit", HERMIT),
    ("last-worker", LAST_WORKER),
    ("dissolved", DISSOLVED),
    ("calibration", CALIBRATION),
];

fn load() -> HypergraphStore {
    let mut graph = HypergraphStore::new();
    load_scenario(SCENARIO, &mut graph).expect("the carrier scenario must load clean");
    graph
}

/// Leg 1 (§9/T4.1): every seeded class's 16 explicit mass values sum to
/// exactly `1.0` in the stored `f64` — no tolerance, matching this crate's
/// own house rule for binary64 values chosen to be exact by construction
/// (`vitality_conformance.rs`'s header explains the same discipline).
#[test]
fn every_seeded_class_reads_all_sixteen_masses_summing_to_exactly_one() {
    let graph = load();
    for (name, id) in SEEDED_CLASSES {
        let mut sum = 0.0_f64;
        for field in MASS_FIELDS {
            let value = graph
                .node_attribute(NodeId(id), field)
                .unwrap_or_else(|e| panic!("{name}: {field} must be seeded: {}", e.message));
            sum += value;
        }
        assert_eq!(sum, 1.0, "{name}: the 16 masses must sum to exactly 1.0");
    }
}

/// Leg 2 (§9/T4.1): the absence fence's own subject. `remnant` seeds NONE
/// of the 16 mass fields — every direct read is `Err` (III.11: absence is
/// a value, not a fabricated zero), and the optional-bind-style sum
/// (each absent field read as `0.0`, exactly the semantics a future
/// `:optional :default 0.0c` BSL binding would apply) is exactly `0.0` —
/// "no distribution", never a fabricated uniform (H1's own citation,
/// `content/rules/consciousness.bsl`'s UNPOSITIONED idiom).
#[test]
fn the_unseeded_class_reads_absent_and_its_optional_bind_sum_is_zero() {
    let graph = load();
    let mut sum = 0.0_f64;
    for field in MASS_FIELDS {
        let result = graph.node_attribute(NodeId(REMNANT), field);
        assert!(
            result.is_err(),
            "remnant: {field} must read ABSENT (III.11), found {result:?}"
        );
        sum += result.unwrap_or(0.0);
    }
    assert_eq!(
        sum, 0.0,
        "remnant: the optional-bind sum over 16 absent fields must be exactly 0.0"
    );
}

/// Leg 3 (§9/T4.1): the 15 cuts are strictly positive and monotone
/// non-decreasing (the check itself is `<=`, tolerating a tie even though
/// this fixture's own values happen to be strictly increasing).
#[test]
fn the_fifteen_cuts_are_strictly_positive_and_monotone_non_decreasing() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the carrier scenario must load clean");
    let mut previous: Option<f64> = None;
    for qname in CUT_CONSTS {
        let Some(Value::Ratio { value, .. }) = loaded.consts.get(qname) else {
            panic!("{qname} must be declared as a Ratio (r-suffixed) defconst");
        };
        let cut = value.get();
        assert!(
            cut > 0.0,
            "{qname}: a cut must be strictly positive, found {cut}"
        );
        if let Some(prev) = previous {
            assert!(
                prev <= cut,
                "{qname}: the grid must be monotone non-decreasing — previous cut {prev} \
                 exceeds this one {cut}"
            );
        }
        previous = Some(cut);
    }
}

/// Leg 4 (§9/T4.1): `0.0r` is `E-LEX-027` — the reader refuses a
/// non-positive `Ratio` literal outright, so a cut of exactly zero cannot
/// even be spelled in the content language (the same gap H2′, design doc
/// §6.2, names for `cut-00`'s implicit zero convention).
///
/// **Observed, not asserted on the numeric code.** `ScenarioError`'s
/// `From<ReadError>` impl (`scenario.rs`) sets `code: None` and carries
/// only the reader's prose message — a read-time (lexer) failure never
/// propagates `LexCode::spec_code()`'s `"E-LEX-027"` string through
/// `load_scenario`'s own error type, unlike a load-time (`ScenarioError`
/// constructed directly, e.g. `E-LOAD-052`) failure, which does. This is
/// existing, pre-this-task behaviour (`reader.rs`'s own unit tests reach
/// the code via a private `lex_err` helper this integration test cannot
/// call), so the assertion below checks the reader's own stable prose
/// (`"(0, ∞)"` / `"strictly positive"`, both drawn verbatim from
/// `classify_ratio`'s doc comment and error text) rather than a code
/// string that does not, in fact, reach this boundary.
#[test]
fn a_zero_ratio_cut_literal_is_refused_at_read_time_e_lex_027() {
    let mut graph = HypergraphStore::new();
    let bad = "(scenario t/zero-cut-probe (defconst wealth-sketch/cut-01 0.0r))";
    let Err(err) = load_scenario(bad, &mut graph) else {
        panic!("a 0.0r Ratio literal must not load");
    };
    let message = err.to_string();
    assert!(
        message.contains("strictly positive") && message.contains("0, \u{221e}"),
        "the refusal must name the Ratio domain's strict-positivity law (E-LEX-027): {message}"
    );
}

/// Leg 4, the negative case — `E-LEX-027` also refuses a negative `Ratio`
/// literal, not only exactly zero. See the sibling test above for why this
/// checks the reader's prose rather than the numeric code string.
#[test]
fn a_negative_ratio_cut_literal_is_refused_at_read_time_e_lex_027() {
    let mut graph = HypergraphStore::new();
    let bad = "(scenario t/negative-cut-probe (defconst wealth-sketch/cut-01 -1r))";
    let Err(err) = load_scenario(bad, &mut graph) else {
        panic!("a -1r Ratio literal must not load");
    };
    let message = err.to_string();
    assert!(
        message.contains("strictly positive") && message.contains("0, \u{221e}"),
        "the refusal must name the Ratio domain's strict-positivity law (E-LEX-027): {message}"
    );
}

/// `η` (household-person-equivalence, DP-7 = A) and `τ`
/// (subsistence-horizon, DP-5 = A now) ship at their RULED values —
/// reconciliation record §3.2/§4.3, sitting posted on #491,
/// 2026-08-18T02:42:38Z.
#[test]
fn household_person_equivalence_and_subsistence_horizon_are_declared_at_their_ruled_values() {
    let mut graph = HypergraphStore::new();
    let loaded = load_scenario(SCENARIO, &mut graph).expect("the carrier scenario must load clean");
    let Some(Value::Ratio { value: eta, .. }) = loaded
        .consts
        .get("wealth-sketch/household-person-equivalence")
    else {
        panic!("household-person-equivalence must be declared as a Ratio defconst");
    };
    assert_eq!(
        eta.get(),
        1.0,
        "DP-7 = A: eta ships at the declared identity value"
    );

    let Some(Value::Ratio { value: tau, .. }) = loaded.consts.get("vitality/subsistence-horizon")
    else {
        panic!("subsistence-horizon must be declared as a Ratio defconst");
    };
    assert_eq!(
        tau.get(),
        1.0,
        "DP-5 = A now: tau ships at the definitional value"
    );
}

/// The Currency lane (T3, OQ-J): `wealth`/`s-bio`/`s-class` round-trip
/// through the typed `i128` micro-unit storage exactly, on both a
/// multi-member and a one-member class.
#[test]
fn currency_lane_fields_round_trip_exactly() {
    let graph = load();
    assert_eq!(
        graph
            .node_attribute_currency(NodeId(CORE), "social-class/wealth")
            .expect("core: wealth must be a currency field"),
        Currency::from_micro_units(1_000 * 1_000_000)
    );
    assert_eq!(
        graph
            .node_attribute_currency(NodeId(BOURGEOISIE), "social-class/s-class")
            .expect("bourgeoisie: s-class must be a currency field"),
        Currency::from_micro_units(8 * 1_000_000)
    );
    assert_eq!(
        graph
            .node_attribute_currency(NodeId(LAST_WORKER), "social-class/wealth")
            .expect("last-worker: wealth must be a currency field"),
        Currency::from_micro_units(1_000_000)
    );
}

/// Byte-determinism, the same discipline every other pin in this crate
/// carries: two runs, one post-state hash. **POST-T6 (2026-08-21):**
/// `before == after` no longer holds — the mortality rule's population
/// decrement writes state by design (the pack is no longer emit-only), so
/// the test below asserts `before != after` and pins the exact writes in
/// the T6 suite. `fired == 11` = 5 measure (`core`, `bourgeoisie`,
/// `hermit`, `last-worker`, `calibration` — `remnant`'s mass-sum guard and
/// `dissolved`'s `active = 0` exclude them) + 6 mortality (its guard is
/// active × population only, so `remnant` fires too, its effects
/// inner-guarded away on deaths = 0). See
/// `the_unseeded_class_produces_no_reading_and_the_rule_does_not_fire`
/// below for the absence-fence leg the measure count stands on.
#[test]
fn the_carrier_tick_is_deterministic_and_the_measure_rule_fires_for_five_of_seven_classes() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after, "two runs, one post-state");
    // T6: the pack now WRITES state (the mortality rule's population
    // decrement) — the emit-only `before == after` premise of this test's
    // first four months is retired by design (T6.6), not by accident; the
    // exact post-state is pinned by `the_calibration_point_is_exact_and_
    // last_worker_extincts` and the carrier-hashes golden.
    assert_ne!(
        hex(&a.before),
        hex(&a.after),
        "the mortality rule's writes must move the post-state — a tick that \
         kills nobody here would mean the T6 rule did not fire"
    );
    assert_eq!(
        a.fired, 11,
        "5 (the measure: core, bourgeoisie, hermit, last-worker, \
         calibration; remnant's mass-sum = 0 and dissolved's active = 0 \
         fail ITS guard) + 6 (the mortality rule's guard is active × \
         population only, so remnant fires too — its failing-certain = 0 \
         and the effects' inner guard on deaths > 0 blocks any write or \
         event; dissolved passes neither guard)"
    );
}

// ============================================================================
// T5 (#491, Phase 3a) — the dual measure conformance suite.
//
// `vitality/subsistence-clearing`'s own `:material-basis` and header carry
// the full derivation (H2', the S-7 derivation, the H3 horizon identity,
// the ADR210 R13 level-set citation). This suite verifies FOUR vector
// families (design doc §9/T5.1): (1) measure arithmetic against the
// independent Python oracle, exact equality; (2) boundary conditions; (3)
// monotonicity properties, including ADR202 R2's asserted sign; (4) the
// absence fence.
// ============================================================================

/// The fifteen grid cuts, the SAME values `vitality-attrition-
/// conformance.bscn` declares as `wealth-sketch/cut-01`..`-15` — mirrored
/// here (not read from the scenario) because the boundary/property tests
/// below construct SYNTHETIC `(masses, w_bar, s_stock)` triples the
/// fixture's own four firing classes cannot reach (their ratios are fixed
/// by the scenario).
const SYNTHETIC_CUTS: [f64; 15] = [
    0.18, 0.25, 0.32, 0.40, 0.50, 0.62, 0.75, 0.90, 1.05, 1.22, 1.40, 1.60, 1.85, 2.15, 2.50,
];

/// Mirrors `vitality/subsistence-clearing`'s own STEP algebra exactly
/// (H2', design doc §6.2 — `clearing`'s rungs 2..16 against `cut_{k-1}`,
/// `failing_certain`'s rungs 1..15 against `cut_k`, `>=`-inclusive on the
/// clearing side, strict `<` on the failing side, `straddle_band` the
/// complement against `mass_sum` — review I-1, matching the rule's own
/// `(- mass-sum (+ clearing failing-certain))`, never a stipulated
/// `1.0`). NOT a competing third implementation: it is the rule's OWN
/// closed form, transcribed so the boundary/property/dispersion-sign
/// families below can sweep threshold ratios the fixture's four firing
/// classes do not reach, at a sample density (dozens of points) a real
/// BSL tick per point cannot practically provide. **A transcription
/// error here cannot be caught by this file alone** — review I-3's own
/// finding — which is why `evaluator_reaches_an_exact_cut_boundary_and_a_genuine_straddle`
/// below runs these SAME boundary/straddle claims through the actual
/// evaluator (a scratch scenario + `run_once_into`), independently of
/// this function.
fn clearing_failing_straddle(
    masses: &[f64; 16],
    cuts: &[f64; 15],
    w_bar: f64,
    s_stock: f64,
) -> (f64, f64, f64) {
    let mass_sum: f64 = masses.iter().sum();
    let edges: Vec<f64> = cuts.iter().map(|c| c * w_bar).collect();
    let mut clearing = 0.0_f64;
    for k in 2..=16_usize {
        if edges[k - 2] >= s_stock {
            clearing += masses[k - 1];
        }
    }
    let mut failing_certain = 0.0_f64;
    for k in 1..=15_usize {
        if edges[k - 1] < s_stock {
            failing_certain += masses[k - 1];
        }
    }
    let straddle_band = mass_sum - clearing - failing_certain;
    (clearing, failing_certain, straddle_band)
}

/// Family (1), Measure arithmetic (T5.1(1)) — expected values from the
/// independent Python oracle
/// (`content/scenarios/vitality_attrition_conformance.py`), **exact
/// equality, no tolerance**. Provenance: `PYTHONPATH="$PWD/src" uv run
/// python rust/crates/babylon-tick/content/scenarios/
/// vitality_attrition_conformance.py` (no `PYTHONPATH` actually needed —
/// the oracle imports nothing from `babylon.*`, ADR183/ADR173, see its own
/// header), output on 2026-08-18, verbatim:
///
/// ```text
/// measure vectors (S = s_bio + s_class, ADR210 R13 acquiescence level set; tau=1.0):
///   core         w_bar=10.0 s_stock=2.0 clearing=1.0 failing_certain=0.0 straddle_band=0.0
///   bourgeoisie  w_bar=125.0 s_stock=10.0 clearing=1.0 failing_certain=0.0 straddle_band=0.0
///   hermit       w_bar=100.0 s_stock=2.0 clearing=1.0 failing_certain=0.0 straddle_band=0.0
///   last-worker  w_bar=1.0 s_stock=2.0 clearing=0.0 failing_certain=1.0 straddle_band=0.0
/// ```
///
/// Both sides run IEEE-754 basic operations on binary64 — the same
/// exact-equality discipline `vitality_conformance.rs`'s own header
/// documents (`+ − × ÷` and comparison, correctly rounded, reproducing
/// bit-exactly across implementations, `bsl-language.rst` §4.3); a
/// tolerance here would hide exactly the transcription error it would
/// appear to absorb.
#[test]
fn measure_arithmetic_matches_the_independent_oracle_exactly() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the measure rule must run");

    // T6: the sink now also carries the mortality rule's two
    // POPULATION_ATTRITION events (last-worker, calibration) — filter by
    // type, never by absolute index. Rule-id byte order fires the measure
    // first ("subsistence-clearing" < "subsistence-mortality").
    let measure_events: Vec<&(String, Vec<(String, Value)>)> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "SUBSISTENCE_CLEARANCE_MEASURED")
        .collect();
    assert_eq!(
        measure_events.len(),
        5,
        "exactly the five guard-admitted classes (the calibration fixture          joined the world at T6)"
    );

    // (node id, w_bar $, s_stock $, mass_sum, clearing, failing_certain, straddle_band)
    let expected: [(u64, i128, i128, f64, f64, f64, f64); 5] = [
        (CORE, 10 * 1_000_000, 2 * 1_000_000, 1.0, 1.0, 0.0, 0.0),
        (
            BOURGEOISIE,
            125 * 1_000_000,
            10 * 1_000_000,
            1.0,
            1.0,
            0.0,
            0.0,
        ),
        (HERMIT, 100 * 1_000_000, 2 * 1_000_000, 1.0, 1.0, 0.0, 0.0),
        (LAST_WORKER, 1_000_000, 2 * 1_000_000, 1.0, 0.0, 1.0, 0.0),
        // The T6 calibration fixture (id 6): w_bar = 1.0$ against
        // s_stock = 1.0$ (s_class = 0) — rung 1 wholly failing, nothing
        // straddling (cut-01 × 1.0 = 0.18 < 1.0 by a wide margin).
        (CALIBRATION, 1_000_000, 1_000_000, 1.0, 0.0, 1.0, 0.0),
    ];
    for (i, (id, w_bar_micro, s_stock_micro, mass_sum, clearing, failing_certain, straddle_band)) in
        expected.into_iter().enumerate()
    {
        let payload = &measure_events[i].1;
        let expected_payload = vec![
            ("entity-id".to_owned(), Value::NodeRef(NodeId(id))),
            (
                "w-bar".to_owned(),
                Value::Currency(Currency::from_micro_units(w_bar_micro)),
            ),
            (
                "s-stock".to_owned(),
                Value::Currency(Currency::from_micro_units(s_stock_micro)),
            ),
            ("mass-sum".to_owned(), Value::Real(mass_sum)),
            ("clearing".to_owned(), Value::Real(clearing)),
            ("failing-certain".to_owned(), Value::Real(failing_certain)),
            ("straddle-band".to_owned(), Value::Real(straddle_band)),
        ];
        assert_eq!(payload, &expected_payload, "measurement {i} (node {id})");
        // Review I-1: assert the identity against the EMITTED mass-sum, not
        // a hardcoded 1.0 — a partially-seeded class's short mass-sum would
        // be visible here (the identity would still hold, at a total below
        // 1.0), rather than absorbed into a fabricated straddle-band.
        let emitted_mass_sum = match &payload[3] {
            (label, Value::Real(v)) if label == "mass-sum" => *v,
            other => panic!("expected mass-sum at payload position 3, found {other:?}"),
        };
        assert_eq!(
            clearing + failing_certain + straddle_band,
            emitted_mass_sum,
            "the dual-plus-straddle identity against the EMITTED mass-sum, node {id}"
        );
    }
}

/// Family (2), Boundary (T5.1(2)). `S·τ` below every cut: `clearing = 1 −
/// mass-01` (STEP, rung 1 excluded by construction) and
/// `failing_certain = 0`.
#[test]
fn boundary_s_tau_below_every_cut() {
    // 0.25/0.75 (exact dyadic fractions in binary64) — this crate's own
    // house rule for a hand-authored fixture whose sum must be exact, no
    // rounding artifact from the fixture's OWN chosen values (T4's own
    // `vitality_attrition_conformance.rs` header states the identical
    // discipline).
    let mut masses = [0.0_f64; 16];
    masses[0] = 0.25; // rung 1 — never counted toward clearing
    masses[7] = 0.75; // rung 8
    let w_bar = 1.0;
    let s_stock = 0.10; // below cut-01 * w_bar = 0.18
    let (clearing, failing_certain, straddle_band) =
        clearing_failing_straddle(&masses, &SYNTHETIC_CUTS, w_bar, s_stock);
    assert_eq!(
        clearing,
        1.0 - masses[0],
        "clearing = 1 - mass-01 under STEP"
    );
    assert_eq!(failing_certain, 0.0);
    assert_eq!(straddle_band, masses[0]);
}

/// Family (2), Boundary. `S·τ` at/above every cut: `clearing = 0`.
#[test]
fn boundary_s_tau_at_or_above_every_cut() {
    let mut masses = [0.0_f64; 16];
    masses[0] = 0.25;
    masses[7] = 0.75;
    let w_bar = 1.0;
    let s_stock = 3.0; // above cut-15 * w_bar = 2.50
    let (clearing, failing_certain, straddle_band) =
        clearing_failing_straddle(&masses, &SYNTHETIC_CUTS, w_bar, s_stock);
    assert_eq!(clearing, 0.0, "no rung's lower edge reaches s_stock");
    assert_eq!(
        failing_certain, 1.0,
        "every rung 1..15's upper edge falls short"
    );
    assert_eq!(straddle_band, 0.0);
}

/// Family (2), Boundary. Exactly on a cut — `≥`-inclusive (M-4: a
/// comparison has no rounding mode). `s_stock` sits at EXACTLY
/// `cut-08 * w_bar`: rung 9's lower edge is `cut-08`, so rung 9 clears
/// (`>=`, inclusive); rung 8's upper edge is ALSO `cut-08`, so rung 8 does
/// NOT certainly fail (`<` is strict) — its mass becomes the straddle
/// band, demonstrating both the inclusive boundary and the straddle
/// mechanism at once. 0.5/0.5 (exact in binary64), the same house rule
/// the sibling boundary tests above use.
#[test]
fn boundary_exactly_on_a_cut_is_inclusive_on_the_clearing_side() {
    let mut masses = [0.0_f64; 16];
    masses[7] = 0.5; // rung 8 — straddles
    masses[8] = 0.5; // rung 9 — clears (inclusive lower edge)
    let w_bar = 1.0;
    let s_stock = SYNTHETIC_CUTS[7]; // cut-08 exactly, * w_bar = 0.90
    let (clearing, failing_certain, straddle_band) =
        clearing_failing_straddle(&masses, &SYNTHETIC_CUTS, w_bar, s_stock);
    assert_eq!(clearing, 0.5, "rung 9 clears at the inclusive boundary");
    assert_eq!(
        failing_certain, 0.0,
        "rung 8 does not CERTAINLY fail — < is strict"
    );
    assert_eq!(straddle_band, 0.5, "rung 8's mass is the straddled band");
}

/// Family (3), Property (T5.1(3)). `clearing` is non-increasing in `S`
/// (holding `w_bar` fixed) and non-decreasing in `w_bar` (holding `S`
/// fixed) — the S-7 derivation's own claim that `clearing` is a
/// complementary CDF, never a curve that can rise as the threshold rises
/// or fall as wealth rises.
#[test]
fn clearing_is_monotone_in_s_and_in_w_bar() {
    let mut masses = [0.0_f64; 16];
    for m in &mut masses {
        *m = 1.0 / 16.0;
    }
    let w_bar = 1.0;
    let s_stocks: Vec<f64> = (0..40).map(|i| 0.05 + f64::from(i) * 0.08).collect();
    let clearings: Vec<f64> = s_stocks
        .iter()
        .map(|&s| clearing_failing_straddle(&masses, &SYNTHETIC_CUTS, w_bar, s).0)
        .collect();
    for pair in clearings.windows(2) {
        assert!(
            pair[1] <= pair[0] + f64::EPSILON,
            "clearing must be non-increasing in S: {pair:?}"
        );
    }

    let s_stock = 1.0;
    let w_bars: Vec<f64> = (1..40).map(f64::from).collect();
    let clearings_w: Vec<f64> = w_bars
        .iter()
        .map(|&w| clearing_failing_straddle(&masses, &SYNTHETIC_CUTS, w, s_stock).0)
        .collect();
    for pair in clearings_w.windows(2) {
        assert!(
            pair[1] >= pair[0] - f64::EPSILON,
            "clearing must be non-decreasing in w_bar: {pair:?}"
        );
    }
}

/// Family (3), Property. ADR202 R2's asserted sign — more intra-class
/// dispersion implies LESS switch-like rupture — **tested HERE ONLY, in
/// this hand-authored fixture, where masses are free** (design doc
/// §9/T5.1(3)).
///
/// **Limitation, stated explicitly (C-6):** in the seeded world A2 gives
/// one shape per county shared by every class, so intra-class dispersion
/// is a county constant and this sign has no class-varying degrees of
/// freedom to be right or wrong about there. The real-data companion is
/// T8a.6; the honest claim is that R2 is CARRIED, not satisfied, by this
/// train (DP-10).
///
/// The proxy for "switch-like": the LARGEST single-step drop in
/// `clearing` across a threshold sweep spanning every cut. A concentrated
/// distribution (all mass in one rung) drops its entire mass in one step
/// when the sweep crosses that rung's boundary — maximally switch-like. A
/// dispersed distribution (mass spread across every rung) sheds a little
/// mass at every boundary — no single step is large.
#[test]
fn adr202_r2_more_dispersion_means_a_less_switch_like_transition() {
    let mut concentrated = [0.0_f64; 16];
    concentrated[7] = 1.0; // all mass in rung 8

    let mut dispersed = [0.0_f64; 16];
    for m in &mut dispersed {
        *m = 1.0 / 16.0; // mass spread evenly across every rung
    }

    let w_bar = 1.0;
    // One sample strictly between every consecutive pair of cuts, plus one
    // below cut-01 and one above cut-15 — sixteen points spanning the
    // whole grid, so a sweep step can land on every rung boundary.
    let mut sweep: Vec<f64> = vec![SYNTHETIC_CUTS[0] - 0.05];
    for pair in SYNTHETIC_CUTS.windows(2) {
        sweep.push((pair[0] + pair[1]) / 2.0);
    }
    sweep.push(SYNTHETIC_CUTS[14] + 0.05);

    let max_step = |masses: &[f64; 16]| -> f64 {
        let clearings: Vec<f64> = sweep
            .iter()
            .map(|&s| clearing_failing_straddle(masses, &SYNTHETIC_CUTS, w_bar, s).0)
            .collect();
        clearings
            .windows(2)
            .map(|p| (p[0] - p[1]).abs())
            .fold(0.0_f64, f64::max)
    };

    let concentrated_max_step = max_step(&concentrated);
    let dispersed_max_step = max_step(&dispersed);
    assert_eq!(
        concentrated_max_step, 1.0,
        "the concentrated fixture sheds its entire mass in one sweep step"
    );
    assert!(
        concentrated_max_step > dispersed_max_step,
        "R2's sign: more dispersion ({dispersed_max_step}) means a LESS \
         switch-like transition than concentration ({concentrated_max_step})"
    );
}

/// Family (4), Absence (T5.1(4)). The unseeded class produces NO reading
/// and the guarded rule does not fire: `remnant` (id 4) never appears
/// among the emitted events, even though it is `active = 1` with
/// `population = 1` — the sum-guard (H1's own citation, the UNPOSITIONED
/// idiom) excludes it on `mass-sum = 0` alone, distinct from `dissolved`'s
/// pre-existing `active = 0` exclusion.
#[test]
fn the_unseeded_class_produces_no_reading_and_the_rule_does_not_fire() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("the measure rule must run");

    // T6: filter to the measure's own events — the mortality rule's two
    // POPULATION_ATTRITION events (last-worker, calibration) share the sink
    // and are the T6 suite's own business above.
    let fired_ids: Vec<NodeId> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "SUBSISTENCE_CLEARANCE_MEASURED")
        .map(|(_, payload)| match payload.first() {
            Some((label, Value::NodeRef(id))) if label == "entity-id" => *id,
            other => panic!("expected an entity-id NodeRef payload item first, found {other:?}"),
        })
        .collect();
    assert!(
        !fired_ids.contains(&NodeId(REMNANT)),
        "remnant must produce no reading: {fired_ids:?}"
    );
    assert!(
        !fired_ids.contains(&NodeId(DISSOLVED)),
        "dissolved must produce no reading: {fired_ids:?}"
    );
    assert_eq!(
        fired_ids.len(),
        5,
        "core, bourgeoisie, hermit, last-worker, calibration (T6's fixture) only"
    );
}

// ============================================================================
// Fix round 1 (task-5-review.md, findings I-1/I-2/I-3): the two tests below
// run ENGINEERED scratch scenarios through the real evaluator. Neither
// touches `vitality-attrition-conformance.bscn` — its own T4 pin would
// move (design doc §5.4 item 5) — so both build a minimal, throwaway
// `.bscn` string instead, the same discipline this file's own
// `a_zero_ratio_cut_literal_is_refused_at_read_time_e_lex_027` test already
// uses.
// ============================================================================

/// One scratch-scenario node: `(name, active, population, wealth$, s_bio$,
/// s_class$, masses)` — all-integer Currency amounts, so no
/// literal-formatting risk.
type ScratchNode<'a> = (&'a str, i64, i64, i64, i64, i64, [f64; 16]);

/// Builds a minimal scratch scenario declaring exactly the constructs
/// `vitality/subsistence-clearing` reads, with one `NodeType/SOCIAL_CLASS`
/// node per [`ScratchNode`]. Cuts are the SAME fifteen values as the
/// committed carrier (`SYNTHETIC_CUTS`) and `tau` is the same ruled
/// `1.0r`, so a hand-derived expected vector transfers directly. Only
/// NONZERO masses are written — the rest ride the rule's own `:optional
/// :default 0.0c` idiom, exercising the absence fence here too.
fn scratch_scenario(nodes: &[ScratchNode<'_>]) -> String {
    let mut s = String::from("(scenario t/subsistence-clearing-probe\n");
    s.push_str("  (deffield social-class/active int extensive)\n");
    s.push_str("  (deffield social-class/population int extensive)\n");
    s.push_str("  (deffield social-class/wealth currency extensive)\n");
    s.push_str("  (deffield social-class/s-bio currency intensive)\n");
    s.push_str("  (deffield social-class/s-class currency intensive)\n");
    for field in MASS_FIELDS {
        s.push_str(&format!("  (deffield {field} coefficient intensive)\n"));
    }
    for (i, qname) in CUT_CONSTS.iter().enumerate() {
        s.push_str(&format!("  (defconst {qname} {:?}r)\n", SYNTHETIC_CUTS[i]));
    }
    s.push_str("  (defconst vitality/subsistence-horizon 1.0r)\n");
    // T6: the mortality rule's own derived defconst — same value the shipped
    // scenario declares (the calibration derivation lives there, D198).
    s.push_str("  (defconst vitality/kappa 1.0c)\n");
    for (name, active, population, wealth, s_bio, s_class, masses) in nodes {
        s.push_str(&format!(
            "  (node {name} NodeType/SOCIAL_CLASS\n    \
             (social-class/active {active})\n    \
             (social-class/population {population})\n    \
             (social-class/wealth {wealth}$)\n    \
             (social-class/s-bio {s_bio}$)\n    \
             (social-class/s-class {s_class}$)\n"
        ));
        for (i, field) in MASS_FIELDS.iter().enumerate() {
            if masses[i] != 0.0 {
                s.push_str(&format!("    ({field} {:?}c)\n", masses[i]));
            }
        }
        s.push_str("  )\n");
    }
    s.push_str(")\n");
    s
}

/// Review I-2: `:expr` bindings resolve for EVERY subject BEFORE the
/// `when` guard runs (`tick.rs`'s `collect_pass` order — bindings first,
/// guard second), so an UNGUARDED `(/ wealth population-int)` would abort
/// the WHOLE TICK for a `population = 0` class (`E-EVAL-012`, division by
/// zero) even though the guard's own `(> population 0)` would have
/// excluded that class from firing — the guard never gets a chance to
/// run. A negative population trips `floor`'s own `E-EVAL-039` one step
/// earlier. The fix nests `if` around both `population-int` and `w-bar`
/// (never a clamp), making both bindings TOTAL. This proves the tick
/// SURVIVES both cases and that neither class fires.
#[test]
fn a_zero_or_negative_population_class_does_not_abort_the_tick() {
    let scenario = scratch_scenario(&[
        ("zero-pop", 1, 0, 100, 1, 1, [0.0_f64; 16]),
        ("negative-pop", 1, -1, 100, 1, 1, [0.0_f64; 16]),
    ]);
    let report = run_once(&scenario, RULE).expect(
        "population <= 0 must not abort the tick — population-int/w-bar are now total functions",
    );
    assert_eq!(
        report.fired, 0,
        "neither class passes the guard's (> population 0)"
    );
}

/// Review I-3: every boundary/property/dispersion-sign test above runs a
/// Rust TRANSCRIPTION of the rule (`clearing_failing_straddle`), and the
/// only four vectors that reach the real evaluator
/// (`measure_arithmetic_matches_the_independent_oracle_exactly`) are all
/// degenerate — `clearing`/`failing_certain` each land in `{0, 1}` and
/// `straddle_band` is always exactly `0` for the committed fixture's four
/// firing classes — so a mutation that flipped every `>=` to `>` in the
/// rule would leave every prior test green (verified: see this test's own
/// closing mutation check). This test runs THREE boundary/fabrication
/// claims through the REAL evaluator via an engineered scratch scenario.
///
/// `on-cut`: `w_bar = $10`, `s_stock = $9 = cut-08 * w_bar` EXACTLY — the
/// `>=`-inclusive boundary law (M-4, "a comparison has no rounding mode").
/// `mass-08 = 0.5` straddles (its own upper edge, `cut-08`, is NOT
/// strictly less than `s_stock`, so it does not certainly fail either);
/// `mass-09 = 0.5` clears (its lower edge is ALSO `cut-08`, `>=` inclusive).
///
/// `off-cut`: `w_bar = $10`, `s_stock = $3`, strictly between
/// `cut-02 * w_bar = $2.5` and `cut-03 * w_bar = $3.2` — a GENUINE
/// straddle with no cut coincidence. `mass-01 = 0.125` fails;
/// `mass-03 = 0.625` straddles; `mass-16 = 0.25` clears — exact dyadic
/// fractions (binary64-exact, this crate's own house rule for a
/// hand-authored fixture whose sum must be exact, `1/8 + 5/8 + 2/8 = 1`).
///
/// `partial-seed` (review round 2, I-1 STILL OPEN on test coverage: every
/// mass vector above — the fixture, `on-cut`, `off-cut`, both property
/// sweeps — sums to exactly `1.0`, so `clearing + failing + straddle ==
/// mass-sum` holds TAUTOLOGICALLY under BOTH the fixed `mass-sum`
/// complement and the OLD, buggy stipulated-`1.0c` complement — a
/// regression back to the bug would leave every prior vector green). Only
/// THREE of sixteen masses are seeded (rungs 1, 3, 16 — the same shape as
/// `off-cut`, same `s_stock = $3`), and they sum to `0.75`, not `1.0` — the
/// other thirteen ride the `:optional :default 0.0c` idiom genuinely, not
/// as a fixture convenience. Hand-derivation, the SAME rungs `off-cut`
/// exercises so only the magnitudes differ: `mass-01 = 0.125` fails
/// (`f-01` reads `edge-01 = $1.8 < s_stock = $3`, TRUE); `mass-03 = 0.375`
/// straddles (`c-03` reads `edge-02 = $2.5 >= $3`, FALSE, so it does not
/// clear; `f-03` reads `edge-03 = $3.2 < $3`, FALSE, so it does not
/// certainly fail either); `mass-16 = 0.25` clears (`c-16` reads
/// `edge-15 = $25 >= $3`, TRUE). Neither `clearing` nor `failing_certain`
/// depends on `mass-03`'s own magnitude — rung 3 straddles regardless of
/// how much mass it carries — so `clearing = 0.25`, `failing_certain =
/// 0.125` **identically to `off-cut`**; only `mass-sum` (`0.75`, not
/// `1.0`) and therefore `straddle-band` differ. **Under the FIXED formula**
/// (`straddle-band = mass-sum -
/// clearing - failing-certain`): `straddle-band = 0.75 - 0.25 - 0.125 =
/// 0.375`, exactly `mass-03`'s own seeded value — correct. **Under the
/// OLD, buggy formula** (`straddle-band = 1.0 - clearing -
/// failing-certain`): `straddle-band = 1.0 - 0.25 - 0.125 = 0.625` — WRONG
/// by exactly `0.25`, the thirteen UNSEEDED rungs' combined absence,
/// silently reported as if it were resolved, measured mass. That `0.25`
/// gap is the fabrication I-1 exists to close, and it is invisible on
/// every mass-sums-to-1 vector by construction.
///
/// Hand-derived (not oracle-transcribed — the Python oracle's own
/// `SUBJECTS` list is the committed fixture only), then cross-checked
/// against `clearing_failing_straddle` BEFORE the rule ever runs, so a
/// disagreement between the hand derivation and the Rust mirror is caught
/// here rather than the two implementations sharing one blind spot.
#[test]
fn evaluator_reaches_an_exact_cut_boundary_and_a_genuine_straddle() {
    let mut on_cut_masses = [0.0_f64; 16];
    on_cut_masses[7] = 0.5; // rung 8 -- straddles
    on_cut_masses[8] = 0.5; // rung 9 -- clears at the inclusive boundary

    let mut off_cut_masses = [0.0_f64; 16];
    off_cut_masses[0] = 0.125; // rung 1 -- fails
    off_cut_masses[2] = 0.625; // rung 3 -- straddles, no cut coincidence
    off_cut_masses[15] = 0.25; // rung 16 -- clears

    // Review round 2, I-1: the SAME shape as `off-cut`, but only 0.75 of
    // mass is seeded (rungs 1/3/16 = 0.125/0.375/0.25) -- thirteen rungs
    // are genuinely absent, not a fixture convenience.
    let mut partial_seed_masses = [0.0_f64; 16];
    partial_seed_masses[0] = 0.125; // rung 1 -- fails
    partial_seed_masses[2] = 0.375; // rung 3 -- straddles, no cut coincidence
    partial_seed_masses[15] = 0.25; // rung 16 -- clears

    let (on_cut_clearing, on_cut_failing, on_cut_straddle) =
        clearing_failing_straddle(&on_cut_masses, &SYNTHETIC_CUTS, 10.0, 9.0);
    assert_eq!(
        (on_cut_clearing, on_cut_failing, on_cut_straddle),
        (0.5, 0.0, 0.5),
        "hand derivation vs. the Rust mirror must agree before the rule ever runs"
    );
    let (off_cut_clearing, off_cut_failing, off_cut_straddle) =
        clearing_failing_straddle(&off_cut_masses, &SYNTHETIC_CUTS, 10.0, 3.0);
    assert_eq!(
        (off_cut_clearing, off_cut_failing, off_cut_straddle),
        (0.25, 0.125, 0.625),
        "hand derivation vs. the Rust mirror must agree before the rule ever runs"
    );
    let (partial_seed_clearing, partial_seed_failing, partial_seed_straddle) =
        clearing_failing_straddle(&partial_seed_masses, &SYNTHETIC_CUTS, 10.0, 3.0);
    assert_eq!(
        (
            partial_seed_clearing,
            partial_seed_failing,
            partial_seed_straddle
        ),
        (0.25, 0.125, 0.375),
        "hand derivation vs. the Rust mirror must agree before the rule ever runs \
         (straddle-band = mass-sum 0.75 - clearing 0.25 - failing 0.125 = 0.375, \
         NOT 1.0 - 0.25 - 0.125 = 0.625 -- that 0.625 is what the OLD, buggy \
         stipulated-1.0c formula would have reported)"
    );

    let scenario = scratch_scenario(&[
        ("on-cut", 1, 1, 10, 1, 8, on_cut_masses),
        ("off-cut", 1, 1, 10, 1, 2, off_cut_masses),
        ("partial-seed", 1, 1, 10, 1, 2, partial_seed_masses),
    ]);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(&scenario, RULE, &mut graph, &mut sink)
        .expect("the engineered scratch scenario must tick clean");
    assert_eq!(
        sink.events.len(),
        3,
        "all three engineered classes pass the guard (mass-sum > 0 for all, \
         including partial-seed's 0.75)"
    );

    let s_stock_micro: [i128; 3] = [9 * 1_000_000, 3 * 1_000_000, 3 * 1_000_000];
    let mass_sum: [f64; 3] = [1.0, 1.0, 0.75];
    let expected: [(u64, f64, f64, f64); 3] = [
        (0, on_cut_clearing, on_cut_failing, on_cut_straddle),
        (1, off_cut_clearing, off_cut_failing, off_cut_straddle),
        (
            2,
            partial_seed_clearing,
            partial_seed_failing,
            partial_seed_straddle,
        ),
    ];
    for (i, (id, clearing, failing_certain, straddle_band)) in expected.into_iter().enumerate() {
        let payload = &sink.events[i].1;
        let expected_payload = vec![
            ("entity-id".to_owned(), Value::NodeRef(NodeId(id))),
            (
                "w-bar".to_owned(),
                Value::Currency(Currency::from_micro_units(10 * 1_000_000)),
            ),
            (
                "s-stock".to_owned(),
                Value::Currency(Currency::from_micro_units(s_stock_micro[i])),
            ),
            ("mass-sum".to_owned(), Value::Real(mass_sum[i])),
            ("clearing".to_owned(), Value::Real(clearing)),
            ("failing-certain".to_owned(), Value::Real(failing_certain)),
            ("straddle-band".to_owned(), Value::Real(straddle_band)),
        ];
        assert_eq!(
            payload, &expected_payload,
            "engineered class {i} (node {id})"
        );
    }
}

// ============================================================================
// T6 (#491, Phase 3b) — Grinding Attrition: the population-write rule.
//
// `vitality/subsistence-mortality` (content/rules/vitality-attrition.bsl):
// `deaths = floor(population × failing-certain × κ)` under DP-6 = B (the
// driver is `failing-certain`, H2′'s dual — D199 records the departure from
// OQ-H's `failing = 1 − clearing`). κ is the DERIVED-not-picked `.bscn`
// defconst `vitality/kappa` (ADR210 R14), derived at the scenario's named
// calibration fixture (design doc §3.5's three requirements; D198 carries
// the derivation + the divergence-surface exhibit):
//
//   fixture `calibration`: the frozen engine's own canonical total-
//   attrition conformance point — coverage_ratio 1.0, inequality 0.8
//   (tests/unit/formulas/test_vitality.py's
//   test_coverage_below_threshold_causes_attrition, rate clamped 1.0) — the
//   one point where frozen and ported forms agree in SEMANTICS ("everyone
//   certainly failing dies this tick"), not merely magnitude. s_class = 0$
//   so the frozen and R13 level sets coincide at the reference; all mass in
//   rung 1, failing WHOLLY (cut-01 × w-bar = 0.18 < s-stock = 1.0$), so
//   failing-certain = 1.0 exactly by construction.
//   κ = frozen-rate₀ / failing-certain₀ = 1.0 / 1.0 = 1.0c.
//
// Structure contract (transcribed, ADR183 — engine/systems/vitality.py:
// 114-131): deaths reduce population and never wealth; the decrement is
// floored; the two continue guards. The frozen loop's post-drain re-read
// does NOT transcribe: BSL's C4 pre-state law reads tick-entry state and
// W2's same-tick refusals forbid an in-tick read of the drain's writes —
// recorded as a named divergence in D198, with the pack-internal note that
// the drain rule (`vitality/subsistence-and-death`) is not co-loaded by
// this scenario at all, and byte-order rule-id sorting would fire it first
// in any co-load anyway.
// ============================================================================

/// T6.1's exact-deaths legs against the shipped scenario: the calibration
/// fixture's exact fit (100 → 0, the frozen engine's 100 at the same
/// point), last-worker's one-death extinction path (driver = 1, pop = 1),
/// and the negative legs (core/bourgeoisie/hermit/dissolved/remnant emit
/// nothing). Events are filtered by type, never by absolute sink index —
/// the measure rule's own events share the sink.
#[test]
fn the_calibration_point_is_exact_and_last_worker_extincts() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("the carrier world must tick clean");

    let attrition: Vec<&(String, Vec<(String, Value)>)> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "POPULATION_ATTRITION")
        .collect();
    assert_eq!(
        attrition.len(),
        2,
        "exactly last-worker (driver 1, pop 1) and calibration (the exact \
         fit) cross one whole member; every other class's driver is 0"
    );

    // Rule firing order within the pack is ascending rule-id byte order
    // (§4.2/D16) and subject order is node-declaration order, so the
    // last-worker event precedes calibration's.
    let expected: [(u64, i64, f64); 2] = [(3, 1, 0.0), (6, 100, 0.0)];
    for (i, (id, deaths, remaining)) in expected.into_iter().enumerate() {
        let payload = &attrition[i].1;
        let get = |key: &str| -> &Value {
            payload
                .iter()
                .find(|(k, _)| k == key)
                .map(|(_, v)| v)
                .unwrap_or_else(|| panic!("payload missing {key}"))
        };
        assert_eq!(get("entity-id"), &Value::NodeRef(NodeId(id)));
        assert_eq!(get("deaths"), &Value::Int(deaths));
        assert_eq!(get("remaining-population"), &Value::Real(remaining));
        assert_eq!(get("failing-certain"), &Value::Real(1.0));
        assert_eq!(get("attrition-rate"), &Value::Real(1.0));
        // The frozen structure contract: deaths reduce population and NEVER
        // wealth (vitality.py:114-131 — the poor die with 0 wealth; wealth
        // is not reduced when people die).
        assert_eq!(
            graph.node_attribute(NodeId(id), "social-class/population"),
            Ok(remaining)
        );
    }
    assert_eq!(
        graph
            .node_attribute_currency(NodeId(6), "social-class/wealth")
            .expect("seeded currency reads back through its own lane"),
        Currency::from_micro_units(100 * 1_000_000),
        "calibration's wealth is untouched by its extinction"
    );
}

/// T6.1's floor legs, through the real evaluator on scratch worlds: a
/// fractional product below one member floors to zero (no write, no event),
/// and a product landing exactly on one member is one death (the floor
/// boundary is INCLUSIVE at the integer).
#[test]
fn the_deaths_floor_zero_and_boundary_legs() {
    // failing = 0.5 by construction (mass split rung-1/rung-16 across the
    // s-stock line at w-bar = 1.0, s-stock = 1.0$: rung 1's upper edge
    // 0.18 < 1.0 fails, rung 16 is open above).
    let fractional = scratch_scenario(&[("fractional", 1, 1, 1, 1, 0, {
        let mut m = [0.0_f64; 16];
        m[0] = 0.5;
        m[15] = 0.5;
        m
    })]);
    let boundary = scratch_scenario(&[("boundary", 1, 2, 2, 1, 0, {
        let mut m = [0.0_f64; 16];
        m[0] = 0.5;
        m[15] = 0.5;
        m
    })]);

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(&fractional, RULE, &mut graph, &mut sink)
        .expect("the fractional scratch world must tick clean");
    assert!(
        sink.events
            .iter()
            .all(|(ty, _)| ty != "POPULATION_ATTRITION"),
        "pop 1 × failing 0.5 × κ 1.0 = 0.5 floors to zero — no event"
    );
    assert_eq!(
        graph.node_attribute(NodeId(0), "social-class/population"),
        Ok(1.0),
        "no write either"
    );

    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(&boundary, RULE, &mut graph, &mut sink)
        .expect("the boundary scratch world must tick clean");
    let events: Vec<_> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "POPULATION_ATTRITION")
        .collect();
    assert_eq!(events.len(), 1, "pop 2 × failing 0.5 × κ 1.0 = exactly 1");
    assert_eq!(
        graph.node_attribute(NodeId(0), "social-class/population"),
        Ok(1.0)
    );
}

/// T6.1's guard legs under the mortality rule specifically: a never-seeded
/// class (every mass read absent) and an inactive class produce no event
/// and no write, matching the measure rule's own absence-fence behavior.
#[test]
fn the_two_continue_guards_and_the_absence_fence_hold_under_the_mortality_rule() {
    let mut graph = HypergraphStore::new();
    let mut sink = CollectingSink::default();
    run_once_into(SCENARIO, RULE, &mut graph, &mut sink).expect("clean tick");
    let emitters: Vec<u64> = sink
        .events
        .iter()
        .filter(|(ty, _)| ty == "POPULATION_ATTRITION")
        .filter_map(|(_, payload)| {
            payload.iter().find_map(|(k, v)| {
                (k == "entity-id").then_some(v).and_then(|v| match v {
                    Value::NodeRef(NodeId(id)) => Some(*id),
                    _ => None,
                })
            })
        })
        .collect();
    for excluded in [0_u64, 1, 2, 4, 5] {
        assert!(
            !emitters.contains(&excluded),
            "node {excluded} must not emit (core/bourgeoisie/hermit: driver 0; \
             remnant: absence fence; dissolved: active = 0)"
        );
    }
}
