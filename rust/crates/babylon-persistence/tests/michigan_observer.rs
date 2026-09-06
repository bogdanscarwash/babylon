use babylon_persistence::{michigan_economy_v1, michigan_observer_foundation_v1};

#[test]
fn michigan_observer_foundation_seeds_all_83_exact_county_baselines() {
    let economy = michigan_economy_v1().expect("pinned QCEW baseline");
    assert_eq!(economy.counties().len(), 83);
    let wayne = economy
        .counties()
        .iter()
        .find(|row| row.county_geoid == "26163")
        .unwrap();
    assert_eq!(wayne.annual_avg_emplvl, 725_504);
    assert_eq!(wayne.total_annual_wages, 55_436_615_328);
    assert_eq!(wayne.annual_avg_wkly_wage, 1469);
    let (session, bundle) = michigan_observer_foundation_v1().expect("observer foundation");
    assert_eq!(session.completed_tick(), 0);
    let source = std::str::from_utf8(bundle.scenario_source_bytes()).unwrap();
    assert_eq!(source.matches("(node county-").count(), 83);
    assert!(source.contains("territory/qcew-average-weekly-wage 1469"));
    assert!(!source.contains("median-wage"));
    assert!(!source.contains("production-total"));
    assert!(bundle.rule_source_bytes().is_empty());
}

#[test]
fn empty_economy_program_commits_a_quiet_week_without_material_changes() {
    use babylon_bsl::identity_codec::StableBslValueV1;
    use babylon_bsl::structural_verbs::CollectingSink;
    use babylon_graph::stable_element::StableElementKeyV1;
    use babylon_practice_contract::ordered_action_v1::OrderedPracticeActionBatchV1;
    let (mut session, _) = michigan_observer_foundation_v1().unwrap();
    let before = session.stable_graph_state().unwrap();
    let actions =
        OrderedPracticeActionBatchV1::empty(session.session_identity().clone(), 1).unwrap();
    let mut sink = CollectingSink::default();
    let report = session
        .advance(&mut sink, &actions)
        .expect("empty program advances the weekly interval");
    assert_eq!(report.report().fired, 0);
    assert_eq!(report.report().considered, 0);
    assert!(report.report().audit_receipts.is_empty());
    assert!(sink.events.is_empty());
    assert_eq!(session.completed_tick(), 1);
    assert_eq!(
        before.digest(),
        session.stable_graph_state().unwrap().digest()
    );

    let counties = michigan_economy_v1().unwrap().counties();
    let territory_rows = report.material_state_rows().territories().rows();
    assert_eq!(territory_rows.len(), 83);
    for (territory, county) in territory_rows.iter().zip(counties) {
        let StableElementKeyV1::Node { local_name, .. } = territory.territory_id() else {
            panic!("county projection lost its node identity");
        };
        assert_eq!(*local_name, format!("county-{}", county.county_geoid));
        let expected = [
            ("county-fips", county.county_geoid.parse::<u64>().unwrap()),
            ("qcew-average-weekly-wage", county.annual_avg_wkly_wage),
            ("qcew-employment", county.annual_avg_emplvl),
            ("qcew-establishments", county.annual_avg_estabs_count),
            ("qcew-total-annual-wages", county.total_annual_wages),
        ]
        .map(|(key, value)| {
            (
                key.to_owned(),
                StableBslValueV1::Int(i64::try_from(value).unwrap()),
            )
        });
        assert_eq!(territory.ordered_fields(), &expected);
    }
}
