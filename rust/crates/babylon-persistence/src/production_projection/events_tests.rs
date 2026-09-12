use super::*;
use babylon_material_circuit::{ArrivalReceipt, DeliveryReceipt, RealizationReceipt};

fn delivery_receipts(order_id: OrderId) -> MaterialTickReceipts {
    MaterialTickReceipts {
        resolve_tick: 2,
        production: Vec::new(),
        dispatches: Vec::new(),
        losses: Vec::new(),
        handling: Vec::new(),
        local_fulfillments: Vec::new(),
        local_transfers: Vec::new(),
        // Two distinct original receipt rows on the same exact principal.
        arrivals: [3, 5]
            .map(|quantity| ArrivalReceipt { order_id, quantity })
            .to_vec(),
        deliveries: vec![DeliveryReceipt {
            order_id,
            quantity: 8,
        }],
        realizations: vec![RealizationReceipt {
            order_id,
            quantity: 8,
        }],
    }
}

#[test]
fn typed_delivery_preserves_original_rows_identifiers_sequence_and_descriptions() {
    let catalog = crate::test_support::catalog();
    let route = &catalog.routes()[0];
    let supplier = catalog.site(&route.supplier_site_key).unwrap();
    let buyer = catalog.site(&route.buyer_site_key).unwrap();
    let good = catalog.good(&route.good_key).unwrap();
    let mut receipts = delivery_receipts(route.order_id());
    let mut events = Vec::new();
    project_events(&catalog, &receipts, [1; 32], &mut events).unwrap();
    receipts.resolve_tick = 3;
    project_events(&catalog, &receipts, [2; 32], &mut events).unwrap();
    assert_eq!(events.len(), 8);
    let expected = [
        ("arrival", ProductionDeliveryStage::Arrival, 3),
        ("arrival", ProductionDeliveryStage::Arrival, 5),
        ("delivery", ProductionDeliveryStage::Delivery, 8),
        (
            "quantity realization",
            ProductionDeliveryStage::QuantityRealization,
            8,
        ),
    ];
    for (index, event) in events.iter().enumerate() {
        let (kind, stage, quantity) = expected[index % 4];
        let digest = if index < 4 { [1; 32] } else { [2; 32] };
        let digest = digest_hex(&digest);
        assert_eq!(event.id, format!("{digest}:{index}"));
        assert_eq!(event.receipt_digest, digest);
        assert_eq!(event.period, if index < 4 { 2 } else { 3 });
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
            Some(ProductionDeliveryEvidence {
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
    let catalog = crate::test_support::catalog();
    let missing = OrderId::from_bytes([0xfa; 32]);
    assert!(!catalog.routes().iter().any(|row| row.order_id() == missing));
    assert_eq!(
        project_events(
            &catalog,
            &delivery_receipts(missing),
            [1; 32],
            &mut Vec::new()
        ),
        Err(ProductionProjectionError::State)
    );
}
