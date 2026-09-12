use super::*;

fn foundation() -> MaterialRuntimeFoundation {
    let catalog = crate::michigan_material::MichiganMaterialCatalog::from_defines_toml(
        include_str!("../../../../../content/scenarios/michigan/defines.toml"),
    )
    .unwrap();
    crate::michigan_content::MichiganContentPreset::new_campaign(
        crate::michigan_material::MichiganDeliveryPreset::Standard,
    )
    .create_foundation(&catalog)
    .unwrap()
}

fn runtime() -> DurableMaterialRuntime {
    DurableMaterialRuntime {
        config: Config::new(),
        campaign: CampaignId::from_uuid(uuid::Uuid::nil()),
        session: foundation().into_session().unwrap(),
        tail: None,
        last_receipt: None,
        last_choice_receipts: Vec::new(),
    }
}

fn publish(runtime: &mut DurableMaterialRuntime) -> CommittedTickReceipt {
    let actions = OrderedPracticeActionBatch::empty(
        runtime.session.graph_session().session_identity().clone(),
        runtime.session.completed_tick() + 1,
    )
    .unwrap();
    let candidate = runtime.session.prepare_advance(&actions).unwrap();
    let receipt = CommittedTickReceipt::from_material_candidate(&candidate).unwrap();
    let choices = candidate.graph_report().report().choice_receipts.clone();
    assert_eq!(
        receipt.tick_content_hash(),
        candidate.identity().tick_content_hash()
    );
    assert_eq!(
        receipt.world_before(),
        candidate.identity().prior_world_hash()
    );
    assert_eq!(
        receipt.world_after(),
        candidate.identity().result_world_hash()
    );
    assert_eq!(
        receipt.considered(),
        candidate.graph_report().report().considered
    );
    assert_eq!(receipt.fired(), candidate.graph_report().report().fired);
    let (ack, _) = runtime
        .session
        .commit_prepared_and_publish(&mut CollectingSink::default(), candidate, |_| {
            Ok::<_, std::convert::Infallible>(ReplayCommitDisposition::Committed)
        })
        .unwrap();
    runtime.tail = Some(ack);
    runtime.last_receipt = Some(receipt.clone());
    runtime.last_choice_receipts = choices;
    receipt
}

#[test]
fn diagnostics_bind_the_acknowledged_material_world_and_current_graph() {
    let mut runtime = runtime();
    assert!(runtime.diagnostic_receipt().is_none());
    let before = runtime.observe_current_stable_graph_state().unwrap();
    let receipt = publish(&mut runtime);
    assert_eq!(
        before.digest().as_bytes(),
        &receipt.prior_stable_graph_digest()
    );
    let graph = runtime.observe_committed_graph_state(&receipt).unwrap();
    assert_eq!(
        graph.digest().as_bytes(),
        &receipt.result_stable_graph_digest()
    );
    assert_eq!(
        runtime.session.current_world_hash().unwrap(),
        receipt.world_after()
    );
    assert_eq!(runtime.diagnostic_receipt(), Some(&receipt));
    assert_eq!(
        runtime
            .observe_committed_choice_receipts(&receipt)
            .unwrap()
            .len(),
        receipt.choice_receipt_count()
    );
}

#[test]
fn stale_and_process_absent_details_cannot_be_observed_as_current() {
    let mut runtime = runtime();
    let receipt = publish(&mut runtime);
    // Restart can authenticate the graph without inventing process-local details.
    runtime.last_receipt = None;
    runtime.last_choice_receipts.clear();
    assert!(runtime.observe_committed_graph_state(&receipt).is_ok());
    assert!(runtime.observe_committed_choice_receipts(&receipt).is_err());
    publish(&mut runtime);
    assert!(runtime.observe_committed_graph_state(&receipt).is_err());
    assert!(runtime.observe_committed_choice_receipts(&receipt).is_err());
}
