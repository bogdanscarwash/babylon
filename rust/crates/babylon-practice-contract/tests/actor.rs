use babylon_practice_contract::ActorOrganizationId;

#[test]
fn actor_identity_round_trips_only_its_opaque_bytes() {
    let _: fn([u8; 8]) -> ActorOrganizationId = ActorOrganizationId::from_bytes;
    let _: fn(ActorOrganizationId) -> [u8; 8] = ActorOrganizationId::to_bytes;
    let bytes = [0x80, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd];

    assert_eq!(ActorOrganizationId::from_bytes(bytes).to_bytes(), bytes);
}

#[test]
fn actor_identity_orders_as_unsigned_big_endian_bytes() {
    let lower = ActorOrganizationId::from_bytes([0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    let higher = ActorOrganizationId::from_bytes([0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert!(lower < higher);
}
