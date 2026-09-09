//! Pure contracts for the dedicated committed-tick V2 activation pair.

use babylon_persistence::{
    compiled_committed_tick_v2_activation_migrations, compiled_schema_migrations,
};

#[test]
fn activation_pair_is_exact_ordered_and_separate_from_the_ordinary_epoch() {
    let ordinary = compiled_schema_migrations().expect("ordinary migration registry");
    let activation = compiled_committed_tick_v2_activation_migrations()
        .expect("committed-tick V2 activation registry");

    assert_eq!(ordinary.len(), 7);
    assert_eq!(ordinary.last().unwrap().version().as_i64(), 7);
    assert_eq!(activation.len(), 2);
    assert_eq!(activation[0].version().as_i64(), 10);
    assert_eq!(activation[1].version().as_i64(), 11);
    assert_eq!(
        activation[0].sql(),
        include_str!("../migrations/0010_committed_tick_v2_preparation.sql")
    );
    assert_eq!(
        activation[1].sql(),
        include_str!("../migrations/0011_committed_tick_v2_activation.sql")
    );
    assert_eq!(
        activation[0].checksum().as_bytes(),
        &hex_checksum("2d159056ace709ac4e42eb00d7a2fb1b0574705b5095a01ce9043926fcf148b7")
    );
    assert_eq!(
        activation[1].checksum().as_bytes(),
        &hex_checksum("7bd981eefe83d70d178954cb747aeaeac3589985bd42c56fc50822be3c70d066")
    );
}

#[test]
fn activation_sql_leaves_the_runtime_authority_row_as_final_dml() {
    let preparation = include_str!("../migrations/0010_committed_tick_v2_preparation.sql");
    let activation = include_str!("../migrations/0011_committed_tick_v2_activation.sql");

    assert!(preparation.contains("CREATE TABLE babylon_meta.committed_tick_v2_authority_ledger"));
    assert!(!activation.contains("committed_tick_v2_authority_ledger"));
    assert!(!activation.contains("INSERT INTO"));
    assert!(!activation.contains("UPDATE "));
    assert!(!activation.contains("DELETE FROM"));
    assert_eq!(activation.matches("DROP TABLE").count(), 2);
    assert!(
        activation
            .find("DROP TABLE babylon_state.tick_event_field_v1")
            .unwrap()
            < activation
                .find("DROP TABLE babylon_state.tick_event_v1")
                .unwrap()
    );
}

#[test]
fn activation_locks_every_v1_target_before_the_serializable_inventory_snapshot() {
    let runtime = include_str!("../src/runtime.rs");
    let activation = include_str!("../migrations/0011_committed_tick_v2_activation.sql");
    let lock = activation
        .find("LOCK TABLE\n    babylon_state.campaign")
        .expect("activation starts with the closed V1 relation lock set");
    let inventory = activation
        .find("DO $committed_tick_v2_inventory_preflight$")
        .expect("activation inventories only after taking the lock set");

    assert!(lock < inventory);
    assert!(runtime.contains(".batch_execute(SERIALIZABLE_ACTIVATION_SETTINGS_V2)"));
    assert!(!activation[..lock].contains("SELECT "));
}

fn hex_checksum(value: &str) -> [u8; 32] {
    assert_eq!(value.len(), 64);
    let mut output = [0_u8; 32];
    for (index, byte) in output.iter_mut().enumerate() {
        let start = index * 2;
        *byte = u8::from_str_radix(&value[start..start + 2], 16).expect("lowercase hex");
    }
    output
}
