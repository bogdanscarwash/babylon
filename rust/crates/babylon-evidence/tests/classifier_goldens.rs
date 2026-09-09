//! Golden output classes and exact refusal contracts for T3 classifiers.

use babylon_evidence::{
    classify_persistence, classify_sfs, PersistenceClass, PersistenceClassError, SfsClass,
    SfsClassError,
};

#[test]
fn the_eight_w2_vectors_pin_predicate_order() {
    let cases = [
        (
            &[0.0, 1.0, 2.0, 5.0, 8.0, 10.0, 11.0][..],
            SfsClass::Continuing,
        ),
        (
            &[0.0, 1.0, 2.0, 5.0, 8.0, 8.0, 8.0][..],
            SfsClass::LatePlateau,
        ),
        (
            &[5.0, 5.0, 5.0, 5.0, 5.0, 5.0, 5.0][..],
            SfsClass::FlatPlateau,
        ),
        (&[0.0, 1.0, 2.0, 5.0, 8.0, 6.0, 4.0][..], SfsClass::Reversal),
        (
            &[0.0, 1.0, 2.0, 3.0, 4.0, 5.0, 6.0][..],
            SfsClass::ConstantRate,
        ),
        (&[0.0, 3.0, 6.0, 7.0, 8.0, 10.0, 12.0][..], SfsClass::Other),
        (&[0.0, 1.0, 2.0, 5.0, 8.0, 0.0, 8.0][..], SfsClass::Other),
        (&[0.0, 2.0, 2.0, 4.0, 4.0, 6.0, 6.0][..], SfsClass::Other),
    ];
    for (masses, expected) in cases {
        assert_eq!(classify_sfs(2, masses).unwrap(), expected);
        assert_eq!(expected.code(), expected as u8);
        assert_eq!(SfsClass::from_code(expected.code()), Some(expected));
    }
}

#[test]
fn sfs_class_codes_are_exact_and_closed() {
    let expected = [
        (0, SfsClass::FlatPlateau),
        (1, SfsClass::Reversal),
        (2, SfsClass::Continuing),
        (3, SfsClass::LatePlateau),
        (4, SfsClass::ConstantRate),
        (5, SfsClass::Other),
    ];
    for (code, class) in expected {
        assert_eq!(SfsClass::from_code(code), Some(class));
        assert_eq!(class.code(), code);
    }
    assert_eq!(SfsClass::from_code(6), None);
    assert_eq!(SfsClass::from_code(u8::MAX), None);
}

#[test]
fn classifier_rejects_off_by_one() {
    assert_eq!(
        classify_sfs(1, &[0.0; 4]),
        Err(SfsClassError::InvalidWindowWidth { found: 1 })
    );
    assert_eq!(
        classify_sfs(53, &[0.0; 160]),
        Err(SfsClassError::InvalidWindowWidth { found: 53 })
    );
    assert_eq!(
        classify_sfs(2, &[0.0; 6]),
        Err(SfsClassError::WrongLength {
            expected: 7,
            actual: 6,
        })
    );
    assert_eq!(
        classify_sfs(2, &[0.0; 8]),
        Err(SfsClassError::WrongLength {
            expected: 7,
            actual: 8,
        })
    );
    assert_eq!(classify_sfs(52, &[0.0; 157]), Ok(SfsClass::FlatPlateau));
    assert_eq!(
        classify_sfs(52, &[0.0; 156]),
        Err(SfsClassError::WrongLength {
            expected: 157,
            actual: 156,
        })
    );
}

#[test]
fn sfs_rejects_nonfinite_mass_before_arithmetic() {
    let mut nan = [0.0; 7];
    nan[3] = f64::NAN;
    assert_eq!(
        classify_sfs(2, &nan),
        Err(SfsClassError::NonFiniteMass { index: 3 })
    );
    let mut infinity = [0.0; 7];
    infinity[5] = f64::INFINITY;
    assert_eq!(
        classify_sfs(2, &infinity),
        Err(SfsClassError::NonFiniteMass { index: 5 })
    );
    let overflow_before_nan = [f64::MAX, -f64::MAX, 0.0, f64::NAN, 0.0, 0.0, 0.0];
    assert_eq!(
        classify_sfs(2, &overflow_before_nan),
        Err(SfsClassError::NonFiniteMass { index: 3 })
    );
}

#[test]
fn sfs_rejects_each_nonfinite_one_operation_result() {
    let delta_overflow = [f64::MAX, -f64::MAX, 0.0, 0.0, 0.0, 0.0, 0.0];
    assert_eq!(
        classify_sfs(2, &delta_overflow),
        Err(SfsClassError::NonFiniteDelta { index: 1 })
    );

    let gain_zero = [
        -f64::MAX,
        0.0,
        f64::MAX,
        f64::MAX,
        f64::MAX,
        f64::MAX,
        f64::MAX,
    ];
    assert_eq!(
        classify_sfs(2, &gain_zero),
        Err(SfsClassError::NonFiniteWindowGain { window: 0 })
    );
    let gain_one = [0.0, 0.0, -f64::MAX, 0.0, f64::MAX, f64::MAX, f64::MAX];
    assert_eq!(
        classify_sfs(2, &gain_one),
        Err(SfsClassError::NonFiniteWindowGain { window: 1 })
    );
    let gain_two = [0.0, 0.0, 0.0, 0.0, -f64::MAX, 0.0, f64::MAX];
    assert_eq!(
        classify_sfs(2, &gain_two),
        Err(SfsClassError::NonFiniteWindowGain { window: 2 })
    );
}

#[test]
fn sfs_normalizes_both_zero_signs_before_bit_predicates() {
    let zeros = [-0.0, 0.0, -0.0, 0.0, -0.0, 0.0, -0.0];
    assert_eq!(classify_sfs(2, &zeros), Ok(SfsClass::FlatPlateau));
}

#[test]
fn classifiers_use_exact_subnormal_values_without_an_epsilon() {
    let subnormal_rate = [
        f64::from_bits(0),
        f64::from_bits(1),
        f64::from_bits(2),
        f64::from_bits(3),
        f64::from_bits(4),
        f64::from_bits(5),
        f64::from_bits(6),
    ];
    assert_eq!(classify_sfs(2, &subnormal_rate), Ok(SfsClass::ConstantRate));
    let positive_subnormal = f64::from_bits(1);
    assert_eq!(
        classify_persistence(
            2,
            &[positive_subnormal, positive_subnormal, positive_subnormal],
        ),
        Ok(PersistenceClass::Persistent)
    );
}

#[test]
fn the_four_persistence_vectors_pin_predicate_order() {
    let cases = [
        (&[2.0, 0.0, 0.0][..], PersistenceClass::Reconverged),
        (&[2.0, 1.0, -1.0][..], PersistenceClass::Reversed),
        (&[2.0, 1.0, 0.5][..], PersistenceClass::Persistent),
        (&[2.0, 0.0, 1.0][..], PersistenceClass::Mixed),
    ];
    for (separations, expected) in cases {
        assert_eq!(classify_persistence(2, separations).unwrap(), expected);
        assert_eq!(expected.code(), expected as u8);
        assert_eq!(PersistenceClass::from_code(expected.code()), Some(expected));
    }
}

#[test]
fn persistence_class_codes_are_exact_and_closed() {
    let expected = [
        (0, PersistenceClass::Reconverged),
        (1, PersistenceClass::Reversed),
        (2, PersistenceClass::Persistent),
        (3, PersistenceClass::Mixed),
    ];
    for (code, class) in expected {
        assert_eq!(PersistenceClass::from_code(code), Some(class));
        assert_eq!(class.code(), code);
    }
    assert_eq!(PersistenceClass::from_code(4), None);
    assert_eq!(PersistenceClass::from_code(u8::MAX), None);
}

#[test]
fn persistence_width_and_checked_length_bounds_are_exact() {
    assert_eq!(
        classify_persistence(1, &[0.0; 2]),
        Err(PersistenceClassError::InvalidPostWidth { found: 1 })
    );
    assert_eq!(
        classify_persistence(53, &[0.0; 54]),
        Err(PersistenceClassError::InvalidPostWidth { found: 53 })
    );
    assert_eq!(
        classify_persistence(2, &[0.0; 2]),
        Err(PersistenceClassError::WrongLength {
            expected: 3,
            actual: 2,
        })
    );
    assert_eq!(
        classify_persistence(2, &[0.0; 4]),
        Err(PersistenceClassError::WrongLength {
            expected: 3,
            actual: 4,
        })
    );
    assert_eq!(
        classify_persistence(52, &[0.0; 53]),
        Ok(PersistenceClass::Reconverged)
    );
}

#[test]
fn persistence_reconvergence_uses_the_last_two_samples() {
    assert_eq!(
        classify_persistence(2, &[0.0, 0.0, 1.0]),
        Ok(PersistenceClass::Mixed)
    );
    assert_eq!(
        classify_persistence(2, &[1.0, 0.0, -0.0]),
        Ok(PersistenceClass::Reconverged)
    );
}

#[test]
fn persistence_rejects_nonfinite_values_and_normalizes_zero() {
    assert_eq!(
        classify_persistence(2, &[2.0, f64::NAN, 0.0]),
        Err(PersistenceClassError::NonFiniteSeparation { index: 1 })
    );
    assert_eq!(
        classify_persistence(2, &[2.0, 1.0, f64::NEG_INFINITY]),
        Err(PersistenceClassError::NonFiniteSeparation { index: 2 })
    );
    assert_eq!(
        classify_persistence(2, &[-0.0, -1.0, 1.0]),
        Ok(PersistenceClass::Mixed)
    );
}
