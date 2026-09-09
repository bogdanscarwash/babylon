//! The end-to-end proof the adversarial review demanded: a rule that
//! DECLARES `(intrinsic floor …)` and CALLS `(floor …)` clears content,
//! load, and evaluation through the exact same `run_once`/`run_once_into`
//! the CLI driver (`main.rs`'s two-path invocation) and `babylon-client`'s
//! engine link (`engine_link.rs`) both call — the SAME entry point
//! production uses, not a third parameter only this test exercises (round
//! 1 of this review shipped exactly that gap: `run_once_with_intrinsics`
//! had no production caller at all).
//!
//! The intrinsic declaration lives INSIDE `RULE`, alongside the `(rule …)`
//! form — §2.2: "file boundaries and file names carry no semantics", so an
//! `(intrinsic …)` top-form is ordinary content wherever it appears in the
//! source `run_once` reads, not a side channel.
//!
//! `population = 11`, `rate = 0.5c` (exact in binary64 — no rounding
//! ambiguity), `deaths = floor(population * rate) = floor(5.5) = 5`.

use babylon_bsl::structural_verbs::CollectingSink;
use babylon_graph::memory::MemoryGraph;
use babylon_graph::substrate::{GraphSubstrate, NodeId};
use babylon_tick::{run_once, run_once_into};

const SCENARIO: &str = r#"
(scenario floor-e2e/one-class
  (deffield social-class/population int extensive)
  (deffield social-class/deaths int extensive)
  (defconst economy/rate 0.5c)

  (node core NodeType/SOCIAL_CLASS
    (social-class/population 11)))
"#;

const RULE: &str = r#"
(intrinsic floor :params (real) :returns int :cost 5)
(rule vitality/floor-e2e-count-deaths
  :role mechanic :evidence derived :material-basis "prove the floor intrinsic clears content, load and evaluation"
  :fuel 64
  (bindings
    (binding population :field social-class/population)
    (binding rate :const economy/rate)
    (binding deaths :expr (floor (* population rate))))
  (when (> population 0))
  (effects
    (update-node self social-class/deaths (set deaths))))
"#;

/// The mirror source with the two top-forms in the OTHER order — proves
/// `split_content` really does treat placement as non-semantic rather than
/// happening to work only because the intrinsic decl comes first.
const RULE_INTRINSIC_LAST: &str = r#"
(rule vitality/floor-e2e-count-deaths
  :role mechanic :evidence derived :material-basis "prove the floor intrinsic clears content, load and evaluation"
  :fuel 64
  (bindings
    (binding population :field social-class/population)
    (binding rate :const economy/rate)
    (binding deaths :expr (floor (* population rate))))
  (when (> population 0))
  (effects
    (update-node self social-class/deaths (set deaths))))
(intrinsic floor :params (real) :returns int :cost 5)
"#;

#[test]
fn a_rule_that_declares_and_calls_floor_runs_through_run_once() {
    let mut graph = MemoryGraph::new();
    let mut sink = CollectingSink::default();
    let report = run_once_into(SCENARIO, RULE, &mut graph, &mut sink)
        .expect("a declared, called floor intrinsic must clear the whole seam");

    assert_eq!(report.fired, 1);
    assert_ne!(report.before, report.after, "the rule must move state");

    let deaths = graph
        .node_attribute(NodeId(0), "social-class/deaths")
        .expect("social-class/deaths must have been written");
    assert_eq!(
        deaths, 5.0,
        "floor(11 * 0.5) = floor(5.5) = 5 — a ceiling or round-half-even \
         implementation would write 6"
    );
}

/// §2.2's "file boundaries and file names carry no semantics", proven for
/// FORM ORDER specifically: the intrinsic declaration AFTER the rule form
/// must produce byte-identical hashes to the declaration BEFORE it.
#[test]
fn declaration_order_relative_to_the_rule_does_not_matter() {
    let first = run_once(SCENARIO, RULE).expect("intrinsic-first source");
    let last = run_once(SCENARIO, RULE_INTRINSIC_LAST).expect("intrinsic-last source");
    assert_eq!(first.after, last.after);
    assert_eq!(first.fired, last.fired);
}

#[test]
fn the_declared_call_is_deterministic_across_two_full_runs() {
    let a = run_once(SCENARIO, RULE).expect("first run");
    let b = run_once(SCENARIO, RULE).expect("second run");
    assert_eq!(a.after, b.after);
}

/// The other half of leg 2: a name outside `DECLARABLE_INTRINSICS` refuses
/// the WHOLE load, loudly — never a partial admission of the rest.
#[test]
fn declaring_an_uncapped_intrinsic_refuses_the_whole_load() {
    const TANH_RULE: &str = r#"
(intrinsic tanh :params (real) :returns real :cost 40)
(rule vitality/floor-e2e-count-deaths
  :role mechanic :evidence derived :material-basis "x" :fuel 8 (when #t))
"#;
    let err = run_once(SCENARIO, TANH_RULE).unwrap_err();
    assert!(
        err.contains("declarable intrinsic set"),
        "unexpected message: {err}"
    );
}

/// A declared signature that disagrees with the kernel's registration
/// (`E-LOAD-020`) refuses the whole load too — floor's kernel signature is
/// `(real) -> int`, not `(int) -> int`.
#[test]
fn a_mismatched_declared_signature_refuses_the_whole_load() {
    const WRONG_SIGNATURE_RULE: &str = r#"
(intrinsic floor :params (int) :returns int :cost 5)
(rule vitality/floor-e2e-count-deaths
  :role mechanic :evidence derived :material-basis "x" :fuel 8 (when #t))
"#;
    let err = run_once(SCENARIO, WRONG_SIGNATURE_RULE).unwrap_err();
    assert!(err.contains("E-LOAD-020"), "unexpected message: {err}");
}

/// A duplicate intrinsic name across the content set (`E-LOAD-001`) refuses
/// the whole load rather than the last declaration silently winning.
#[test]
fn a_duplicate_intrinsic_declaration_refuses_the_whole_load() {
    const DUPLICATE_RULE: &str = r#"
(intrinsic floor :params (real) :returns int :cost 5)
(intrinsic floor :params (real) :returns int :cost 9)
(rule vitality/floor-e2e-count-deaths
  :role mechanic :evidence derived :material-basis "x" :fuel 8 (when #t))
"#;
    let err = run_once(SCENARIO, DUPLICATE_RULE).unwrap_err();
    assert!(err.contains("E-LOAD-001"), "unexpected message: {err}");
}

/// ADR219 (Director ruling 2026-08-22) resolves this test's former
/// premise: `round-half-even` is no longer "ratified but unlanded" —
/// ADR188 Row 3's housekeeping rider LANDED (D70 resolved), so the name
/// now clears the cap, matches its kernel signature, and DISPATCHES
/// through the same `run_once`/`run_once_into` seam the CLI driver and
/// `babylon-client`'s engine link both call. The property the retired
/// test protected — load pipeline and dispatcher never silently admit or
/// silently no-op an unsupported name — now stands POSITIVELY: every
/// `DECLARABLE_INTRINSICS` member carries a `kernel_signature` row and a
/// dispatch arm in lockstep (`declarations.rs`'s
/// `the_adr219_six_are_declarable_with_their_kernel_signatures` and
/// `intrinsic_host.rs`'s
/// `every_declarable_intrinsic_has_a_signature_row_and_a_dispatch_arm`),
/// and the negative probe lives in
/// `declaring_an_uncapped_intrinsic_refuses_the_whole_load` above, over
/// `tanh` (still outside the cap — ADR188 Row 8).
///
/// The binding's `(round-half-even 0.5c)` ties to the even neighbor —
/// `0.0` — so evaluation exercises the arm on a ruled tie case, and the
/// rule completing proves the dispatch succeeded (the bit-level tie pins
/// themselves are the host's own unit tests').
#[test]
fn round_half_even_now_loads_and_dispatches_through_run_once() {
    const ROUND_HALF_EVEN_RULE: &str = r#"
(intrinsic round-half-even :params (real) :returns real :cost 6)
(rule vitality/floor-e2e-round-half-even
  :role mechanic :evidence derived :material-basis "ADR219: round-half-even dispatches (ADR188 Row 3 landed, D70 resolved)"
  :fuel 64
  (bindings
    (binding population :field social-class/population)
    (binding rate :const economy/rate)
    (binding scaled :expr (round-half-even rate)))
  (when (> population 0))
  (effects
    (update-node self social-class/deaths (set population))))
"#;
    let report = run_once(SCENARIO, ROUND_HALF_EVEN_RULE)
        .expect("round-half-even clears cap, signature, and dispatch under ADR219");
    assert_eq!(report.fired, 1);
}
