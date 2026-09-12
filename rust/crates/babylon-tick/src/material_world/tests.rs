use super::*;

// A merchant's two physical handoffs share the same explicit handling account.
// These bytes exercise the committed receipt boundary independently of its encoder.
fn local_handoff_receipts() -> Vec<u8> {
    let mut bytes = b"babylon.material-tick-receipts.v4\0".to_vec();
    bytes.extend_from_slice(&4_u32.to_be_bytes());
    bytes.extend_from_slice(&1_u64.to_be_bytes());
    for tag in 1..=9_u8 {
        bytes.push(tag);
        bytes.extend_from_slice(&u64::from(matches!(tag, 7 | 8)).to_be_bytes());
        if tag == 7 {
            bytes.extend_from_slice(&[1; 32]); // retailer
            bytes.push(2); // local final demand, not a freight order
            bytes.extend_from_slice(&[2; 32]); // order
            for value in [10_u64, 2, 20, 4] {
                bytes.extend_from_slice(&value.to_be_bytes());
            }
        } else if tag == 8 {
            for key in [2, 1, 3, 4, 5] {
                bytes.extend_from_slice(&[key; 32]);
            }
            bytes.extend_from_slice(&2_u64.to_be_bytes());
        }
    }
    bytes
}

#[test]
fn committed_receipts_admit_explicit_merchant_handling_and_local_fulfillment() {
    use babylon_material_circuit::{OrderId, OutboundOrderId};
    let receipt = decode_material_receipts(&local_handoff_receipts()).unwrap();
    assert_eq!(receipt.resolve_tick, 1);
    assert!(receipt.dispatches.is_empty());
    assert!(receipt.arrivals.is_empty());
    assert_eq!(receipt.handling.len(), 1);
    let work = &receipt.handling[0];
    assert_eq!(
        work.order,
        OutboundOrderId::LocalFinalDemand(OrderId::from_bytes([2; 32]))
    );
    assert_eq!((work.feasible_quantity, work.handled_quantity), (10, 2));
    assert_eq!((work.needed_hours, work.used_hours), (20, 4));
    assert_eq!(receipt.local_fulfillments.len(), 1);
    assert_eq!(receipt.local_fulfillments[0].quantity, 2);
    assert_eq!(receipt.local_fulfillments[0].retailer_site_id, work.site_id);
}

#[test]
fn handling_wire_refuses_unknown_kinds_overdraws_and_false_hours() {
    let canonical = local_handoff_receipts();
    let row = RECEIPT_DOMAIN.len() + 12 + 7 * 9;
    for (offset, replacement) in [
        (row + 32, vec![3]),
        (row + 65 + 8, 11_u64.to_be_bytes().to_vec()),
        (row + 65 + 24, 5_u64.to_be_bytes().to_vec()),
        (row + 65 + 16, 0_u64.to_be_bytes().to_vec()),
    ] {
        let mut changed = canonical.clone();
        changed[offset..offset + replacement.len()].copy_from_slice(&replacement);
        assert_eq!(
            decode_material_receipts(&changed),
            Err(MaterialWorldError::Wire)
        );
    }
}

#[test]
fn receipt_version_and_local_delivery_quantity_are_strict() {
    let canonical = local_handoff_receipts();
    let mut previous_version = canonical.clone();
    let version = RECEIPT_DOMAIN.len();
    previous_version[version..version + 4].copy_from_slice(&3_u32.to_be_bytes());
    assert_eq!(
        decode_material_receipts(&previous_version),
        Err(MaterialWorldError::Wire)
    );
    let mut zero_delivery = canonical;
    let end = zero_delivery.len() - 9; // The empty local-transfer family follows.
    zero_delivery[end - 8..end].copy_from_slice(&0_u64.to_be_bytes());
    assert_eq!(
        decode_material_receipts(&zero_delivery),
        Err(MaterialWorldError::Wire)
    );
}

#[test]
fn local_transfer_receipts_preserve_both_owners_without_freight() {
    use babylon_material_circuit::SiteId;
    let mut bytes = local_handoff_receipts();
    let count = bytes.len() - 8;
    bytes[count..].copy_from_slice(&1_u64.to_be_bytes());
    let row_start = bytes.len();
    for key in [6, 7, 1, 4, 5] {
        bytes.extend_from_slice(&[key; 32]);
    }
    bytes.extend_from_slice(&3_u64.to_be_bytes());
    let receipts = decode_material_receipts(&bytes).unwrap();
    assert_eq!(receipts.local_transfers.len(), 1);
    let transfer = &receipts.local_transfers[0];
    assert_eq!(transfer.supplier_site_id, SiteId::from_bytes([7; 32]));
    assert_eq!(transfer.buyer_site_id, SiteId::from_bytes([1; 32]));
    assert_eq!(transfer.quantity, 3);
    assert!(receipts.dispatches.is_empty());
    assert!(receipts.arrivals.is_empty());

    let mut self_transfer = bytes.clone();
    self_transfer[row_start + 64..row_start + 96].copy_from_slice(&[7; 32]);
    assert_eq!(
        decode_material_receipts(&self_transfer),
        Err(MaterialWorldError::Wire)
    );
    let end = bytes.len();
    bytes[end - 8..].fill(0);
    assert_eq!(
        decode_material_receipts(&bytes),
        Err(MaterialWorldError::Wire)
    );
}
