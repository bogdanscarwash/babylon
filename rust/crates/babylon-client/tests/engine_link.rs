use babylon_client::engine_link;
use babylon_tick::hex;

#[test]
fn startup_tick_matches_the_pinned_hash() {
    let report = engine_link::engine_link_probe().expect("tick");
    // The golden from `babylon-tick` on two-classes.bscn + fundamental-theorem.bsl
    // (Task 10 Step 5). If this moves, the ENGINE moved — investigate, never re-pin
    // without a declared ceremony.
    assert_eq!(
        hex(&report.after),
        "783f651d04d32fffd0109e88423eb7a57b1e0836ed4a9f645d3a8a554e427679",
    );
}
