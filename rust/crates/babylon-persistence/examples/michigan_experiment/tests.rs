use super::{observe::ProcessRow, run};
use std::collections::BTreeSet;

#[test]
fn matrix_changes_only_stock_and_selected_delivery() {
    let cases = run::cases().unwrap();
    assert_eq!(cases.len(), 4);
    assert_eq!(
        cases.iter().map(|case| case.spec.id).collect::<Vec<_>>(),
        ["standard-0", "delayed-0", "standard-320", "delayed-320"]
    );
    let hashes: BTreeSet<_> = cases
        .iter()
        .map(|case| case.catalog.defines_hash())
        .collect();
    assert_eq!(
        hashes.len(),
        2,
        "preset selection does not alter numeric definitions"
    );
    let reformatted = format!("# irrelevant comment\n{}\n", run::BASELINE);
    let catalog =
        babylon_persistence::michigan_material::MichiganMaterialCatalog::from_defines_toml(
            &reformatted,
        )
        .unwrap();
    assert_eq!(catalog.defines_bytes(), cases[0].catalog.defines_bytes());
    let first = run::experiment_identity(&cases).unwrap();
    assert_eq!(
        run::digest_json(&first).unwrap(),
        run::digest_json(&run::experiment_identity(&run::cases().unwrap()).unwrap()).unwrap()
    );
}

fn process<'a>(result: &'a run::CaseResult, period: usize, key: &str) -> &'a ProcessRow {
    result.rows[period - 1]
        .processes
        .iter()
        .find(|row| row.process == key)
        .unwrap()
}

#[test]
fn delivery_stock_comparison_preserves_controls_and_exposes_staffing_interruption() {
    let results: Vec<_> = run::cases()
        .unwrap()
        .into_iter()
        .map(|case| run::run_case(&case, |_| Ok(())).unwrap())
        .collect();
    let summary = run::summarize(&results).unwrap();
    for (index, (first, complete, metal, panels)) in [
        (5, 6, 600, 0),
        (7, 8, 600, 0),
        (4, 5, 920, 32),
        (4, 7, 920, 32),
    ]
    .into_iter()
    .enumerate()
    {
        let case = &summary["cases"][index];
        assert_eq!(case["first_panel_output_period"], [3, 5, 2, 2][index]);
        assert_eq!(case["first_subassembly_output_period"], first);
        assert_eq!(case["reaches_30_subassemblies_period"], complete);
        assert_eq!(case["final_subassemblies"], 30);
        assert_eq!(case["final_packaged_meal_kg"], 200);
        assert_eq!(case["final_unsold_panels"], panels);
        assert!(results[index]
            .rows
            .iter()
            .all(|row| row.metal_input_equivalent_kg == metal && row.food_kg == 200));
        assert!(results[index].rows[15]
            .routes
            .iter()
            .all(|row| row.in_transit.is_empty()
                && row.cumulative_delivered == row.ordered
                && row.cumulative_lost == 0));
    }
    let arriving = process(&results[0], 2, "panel-forming");
    assert!(arriving.completed_receipt.is_none());
    assert_eq!(arriving.closing_input, 320);
    assert_eq!(arriving.next_opening.as_ref().unwrap().planned_batches, 32);
    assert!(process(&results[2], 1, "panel-forming")
        .completed_receipt
        .is_none());
    assert_eq!(
        process(&results[1], 2, "panel-forming")
            .staffing
            .separations,
        4
    );
    assert_eq!(
        process(&results[3], 3, "panel-forming")
            .staffing
            .separations,
        4
    );
    assert_eq!(
        process(&results[3], 3, "subassembly-making").staffing.hires,
        4
    );
    assert_eq!(
        process(&results[3], 5, "subassembly-making")
            .staffing
            .separations,
        4
    );
    assert_eq!(
        process(&results[3], 6, "subassembly-making").staffing.hires,
        4
    );
    let repeat = run::run_case(&run::cases().unwrap().pop().unwrap(), |_| Ok(())).unwrap();
    assert_eq!(
        serde_json::to_vec(&repeat.rows).unwrap(),
        serde_json::to_vec(&results[3].rows).unwrap()
    );
    let foundations: BTreeSet<_> = results
        .iter()
        .map(|case| case.foundation["foundation_sha256"].as_str().unwrap())
        .collect();
    assert_eq!(foundations.len(), 4);
}

#[test]
fn only_absolute_output_directory_is_accepted() {
    assert!(super::output_argument(["--seed".into(), "319".into()]).is_err());
    assert!(super::output_argument(["--output".into(), "relative".into()]).is_err());
    assert_eq!(
        super::output_argument(["--output".into(), "/tmp/owned-experiment".into()]).unwrap(),
        std::path::PathBuf::from("/tmp/owned-experiment")
    );
}
