use super::*;
use babylon_material_circuit::{ArrivalReceiptV1, DeliveryReceiptV1, RealizationReceiptV1};

fn delivery_receipts(order_id: OrderIdV1) -> MaterialTickReceiptsV3 {
    MaterialTickReceiptsV3 {
        resolve_tick: 2,
        production: Vec::new(),
        dispatches: Vec::new(),
        losses: Vec::new(),
        // Two distinct original receipt rows on the same exact principal.
        arrivals: [3, 5]
            .map(|quantity| ArrivalReceiptV1 { order_id, quantity })
            .to_vec(),
        deliveries: vec![DeliveryReceiptV1 {
            order_id,
            quantity: 8,
        }],
        realizations: vec![RealizationReceiptV1 {
            order_id,
            quantity: 8,
        }],
    }
}

#[test]
fn typed_delivery_preserves_original_rows_identifiers_sequence_and_descriptions() {
    let catalog = michigan_material_catalog_v1().unwrap();
    let route = &catalog.routes()[0];
    let supplier = catalog.site(&route.supplier_site_key).unwrap();
    let buyer = catalog.site(&route.buyer_site_key).unwrap();
    let good = catalog.good(&route.good_key).unwrap();
    let mut receipts = delivery_receipts(route.order_id());
    let mut events = Vec::new();
    project_events(catalog, &receipts, [1; 32], &mut events).unwrap();
    receipts.resolve_tick = 3;
    project_events(catalog, &receipts, [2; 32], &mut events).unwrap();
    assert_eq!(events.len(), 8);
    let expected = [
        ("arrival", ProductionDeliveryStageV1::Arrival, 3),
        ("arrival", ProductionDeliveryStageV1::Arrival, 5),
        ("delivery", ProductionDeliveryStageV1::Delivery, 8),
        (
            "quantity realization",
            ProductionDeliveryStageV1::QuantityRealization,
            8,
        ),
    ];
    for (index, event) in events.iter().enumerate() {
        let (kind, stage, quantity) = expected[index % 4];
        let digest = if index < 4 { [1; 32] } else { [2; 32] };
        let digest = digest_hex(&digest);
        assert_eq!(event.id, format!("{digest}:{index}"));
        assert_eq!(event.receipt_digest, digest);
        assert_eq!(event.week, if index < 4 { 2 } else { 3 });
        assert_eq!(event.kind, kind);
        assert_eq!(
            event.subject_site_ids,
            vec![
                digest_hex(&supplier.id().as_bytes()),
                digest_hex(&buyer.id().as_bytes())
            ]
        );
        assert_eq!(
            event.description,
            format!(
                "{} -> {}: {quantity} {} {} {kind}.",
                supplier.label, buyer.label, good.unit_key, good.label
            )
        );
        assert_eq!(
            event.delivery_evidence,
            Some(ProductionDeliveryEvidenceV1 {
                stage,
                order_id: digest_hex(&route.order_id().as_bytes()),
                route_id: digest_hex(&route.id().as_bytes()),
                good_id: digest_hex(&good.id().as_bytes()),
                unit_id: digest_hex(&good.unit_id().as_bytes()),
                quantity,
            })
        );
    }
}

#[test]
fn undisclosed_orders_refuse_typed_delivery_projection() {
    let catalog = michigan_material_catalog_v1().unwrap();
    let missing = OrderIdV1::from_bytes([0xfa; 32]);
    assert!(!catalog.routes().iter().any(|row| row.order_id() == missing));
    assert_eq!(
        project_events(
            catalog,
            &delivery_receipts(missing),
            [1; 32],
            &mut Vec::new()
        ),
        Err(ProductionProjectionErrorV1::State)
    );
}
