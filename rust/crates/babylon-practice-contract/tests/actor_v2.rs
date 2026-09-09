use babylon_practice_contract::actor_v2::ActorOrganizationIdV2;

#[test]
fn actor_identity_round_trips_only_its_opaque_bytes() {
    let _: fn([u8; 8]) -> ActorOrganizationIdV2 = ActorOrganizationIdV2::from_bytes;
    let _: fn(ActorOrganizationIdV2) -> [u8; 8] = ActorOrganizationIdV2::to_bytes;
    let bytes = [0x80, 0x01, 0x23, 0x45, 0x67, 0x89, 0xab, 0xcd];

    assert_eq!(ActorOrganizationIdV2::from_bytes(bytes).to_bytes(), bytes);
}

#[test]
fn actor_identity_orders_as_unsigned_big_endian_bytes() {
    let lower = ActorOrganizationIdV2::from_bytes([0x7f, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff, 0xff]);
    let higher =
        ActorOrganizationIdV2::from_bytes([0x80, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00, 0x00]);

    assert!(lower < higher);
}
