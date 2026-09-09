//! Bounded discrete classifiers over completed synthetic evidence.

/// Exact slow-fast-slow output classes in committed predicate order.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum SfsClass {
    /// Every normalized mass has identical bits.
    FlatPlateau = 0,
    /// The final window gain is negative.
    Reversal = 1,
    /// A nondecreasing trace accelerates and retains a positive late gain.
    Continuing = 2,
    /// A nondecreasing trace accelerates then has exact zero late deltas.
    LatePlateau = 3,
    /// Every delta has the same positive finite bits.
    ConstantRate = 4,
    /// Every other valid trace.
    Other = 5,
}

impl SfsClass {
    /// Returns the exact V1 wire code.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// Maps one closed V1 wire code without a fallback class.
    #[must_use]
    pub const fn from_code(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::FlatPlateau),
            1 => Some(Self::Reversal),
            2 => Some(Self::Continuing),
            3 => Some(Self::LatePlateau),
            4 => Some(Self::ConstantRate),
            5 => Some(Self::Other),
            _ => None,
        }
    }
}

/// Exact control/intervention persistence classes in predicate order.
#[repr(u8)]
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PersistenceClass {
    /// The final two normalized separations are exact positive zero.
    Reconverged = 0,
    /// Initial and final nonzero separations have opposite signs.
    Reversed = 1,
    /// Every separation is nonzero with the initial sign.
    Persistent = 2,
    /// Every other valid separation trace.
    Mixed = 3,
}

impl PersistenceClass {
    /// Returns the exact V1 wire code.
    #[must_use]
    pub const fn code(self) -> u8 {
        self as u8
    }

    /// Maps one closed V1 wire code without a fallback class.
    #[must_use]
    pub const fn from_code(value: u8) -> Option<Self> {
        match value {
            0 => Some(Self::Reconverged),
            1 => Some(Self::Reversed),
            2 => Some(Self::Persistent),
            3 => Some(Self::Mixed),
            _ => None,
        }
    }
}

/// Exact refusals for the SFS output classifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsClassError {
    /// The window width lies outside 2 through 52.
    InvalidWindowWidth { found: u16 },
    /// The trace does not contain exactly `3w + 1` masses.
    WrongLength { expected: usize, actual: usize },
    /// An input mass is NaN or infinite.
    NonFiniteMass { index: usize },
    /// One exact adjacent subtraction produced NaN or infinity.
    NonFiniteDelta { index: usize },
    /// One exact endpoint subtraction produced NaN or infinity.
    NonFiniteWindowGain { window: u8 },
}

/// Exact refusals for the control/intervention persistence classifier.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum PersistenceClassError {
    /// The post width lies outside 2 through 52.
    InvalidPostWidth { found: u16 },
    /// The trace does not contain exactly `P + 1` separations.
    WrongLength { expected: usize, actual: usize },
    /// An input separation is NaN or infinite.
    NonFiniteSeparation { index: usize },
}

/// Classifies one complete bounded slow-fast-slow mass trace.
///
/// # Errors
/// Returns the first exact width, length, mass, delta, or gain refusal.
pub fn classify_sfs(window_width: u16, masses: &[f64]) -> Result<SfsClass, SfsClassError> {
    let (width, expected, second_endpoint, third_endpoint) =
        checked_sfs_shape(window_width, masses.len())?;
    let normalized = normalize_masses(masses, expected)?;
    let delta_count = expected.checked_sub(1).ok_or(SfsClassError::WrongLength {
        expected,
        actual: masses.len(),
    })?;
    let deltas = compute_deltas(&normalized, delta_count)?;
    let gains = [
        checked_gain(normalized[width], normalized[0], 0)?,
        checked_gain(normalized[second_endpoint], normalized[width], 1)?,
        checked_gain(normalized[third_endpoint], normalized[second_endpoint], 2)?,
    ];
    if masses_have_identical_bits(&normalized, expected) {
        return Ok(SfsClass::FlatPlateau);
    }
    if gains[2] < 0.0 {
        return Ok(SfsClass::Reversal);
    }
    let nondecreasing = deltas_are_nonnegative(&deltas, delta_count);
    let middle_positive = count_middle_positive(&deltas, width);
    if nondecreasing
        && gains[0] >= 0.0
        && gains[1] > gains[0]
        && gains[1] > gains[2]
        && gains[2] > 0.0
        && normalized[third_endpoint] > normalized[0]
        && middle_positive >= 2
    {
        return Ok(SfsClass::Continuing);
    }
    if nondecreasing
        && gains[0] >= 0.0
        && gains[1] > gains[0]
        && normalized[third_endpoint] > normalized[0]
        && middle_positive >= 2
        && late_deltas_are_zero(&deltas, width)
    {
        return Ok(SfsClass::LatePlateau);
    }
    if deltas_have_identical_positive_bits(&deltas, delta_count) {
        return Ok(SfsClass::ConstantRate);
    }
    Ok(SfsClass::Other)
}

/// Classifies one complete bounded post-intervention separation trace.
///
/// # Errors
/// Returns the first exact width, length, or non-finite-value refusal.
pub fn classify_persistence(
    post_width: u16,
    separations: &[f64],
) -> Result<PersistenceClass, PersistenceClassError> {
    let expected = checked_persistence_length(post_width, separations.len())?;
    let normalized = normalize_separations(separations, expected)?;
    let final_index = expected
        .checked_sub(1)
        .ok_or(PersistenceClassError::WrongLength {
            expected,
            actual: separations.len(),
        })?;
    let penultimate_index =
        final_index
            .checked_sub(1)
            .ok_or(PersistenceClassError::WrongLength {
                expected,
                actual: separations.len(),
            })?;
    let first = normalized[0];
    let last = normalized[final_index];
    if normalized[penultimate_index].to_bits() == 0 && last.to_bits() == 0 {
        return Ok(PersistenceClass::Reconverged);
    }
    if first.to_bits() != 0
        && last.to_bits() != 0
        && first.is_sign_negative() != last.is_sign_negative()
    {
        return Ok(PersistenceClass::Reversed);
    }
    if all_separations_retain_sign(&normalized, expected) {
        return Ok(PersistenceClass::Persistent);
    }
    Ok(PersistenceClass::Mixed)
}

fn checked_sfs_shape(
    window_width: u16,
    actual: usize,
) -> Result<(usize, usize, usize, usize), SfsClassError> {
    if !(2..=52).contains(&window_width) {
        return Err(SfsClassError::InvalidWindowWidth {
            found: window_width,
        });
    }
    let width = usize::from(window_width);
    let second = width.checked_mul(2).ok_or(SfsClassError::WrongLength {
        expected: usize::MAX,
        actual,
    })?;
    let third = width.checked_mul(3).ok_or(SfsClassError::WrongLength {
        expected: usize::MAX,
        actual,
    })?;
    let expected = third.checked_add(1).ok_or(SfsClassError::WrongLength {
        expected: usize::MAX,
        actual,
    })?;
    if actual != expected {
        return Err(SfsClassError::WrongLength { expected, actual });
    }
    Ok((width, expected, second, third))
}

fn checked_persistence_length(
    post_width: u16,
    actual: usize,
) -> Result<usize, PersistenceClassError> {
    if !(2..=52).contains(&post_width) {
        return Err(PersistenceClassError::InvalidPostWidth { found: post_width });
    }
    let expected =
        usize::from(post_width)
            .checked_add(1)
            .ok_or(PersistenceClassError::WrongLength {
                expected: usize::MAX,
                actual,
            })?;
    if actual != expected {
        return Err(PersistenceClassError::WrongLength { expected, actual });
    }
    Ok(expected)
}

fn normalize_masses(masses: &[f64], expected: usize) -> Result<[f64; 157], SfsClassError> {
    let mut normalized = [0.0; 157];
    for index in 0..157 {
        if index >= expected {
            break;
        }
        let value = masses[index];
        if !value.is_finite() {
            return Err(SfsClassError::NonFiniteMass { index });
        }
        normalized[index] = normalize_zero(value);
    }
    Ok(normalized)
}

fn compute_deltas(masses: &[f64; 157], count: usize) -> Result<[f64; 156], SfsClassError> {
    let mut deltas = [0.0; 156];
    for index in 1..=156 {
        if index > count {
            break;
        }
        let delta = masses[index] - masses[index - 1];
        if !delta.is_finite() {
            return Err(SfsClassError::NonFiniteDelta { index });
        }
        deltas[index - 1] = normalize_zero(delta);
    }
    Ok(deltas)
}

fn checked_gain(right: f64, left: f64, window: u8) -> Result<f64, SfsClassError> {
    let gain = right - left;
    if gain.is_finite() {
        Ok(normalize_zero(gain))
    } else {
        Err(SfsClassError::NonFiniteWindowGain { window })
    }
}

fn normalize_separations(
    separations: &[f64],
    expected: usize,
) -> Result<[f64; 53], PersistenceClassError> {
    let mut normalized = [0.0; 53];
    for index in 0..53 {
        if index >= expected {
            break;
        }
        let value = separations[index];
        if !value.is_finite() {
            return Err(PersistenceClassError::NonFiniteSeparation { index });
        }
        normalized[index] = normalize_zero(value);
    }
    Ok(normalized)
}

fn normalize_zero(value: f64) -> f64 {
    if value == 0.0 {
        0.0
    } else {
        value
    }
}

#[allow(clippy::needless_range_loop)] // The literal bound is part of the classifier contract.
fn masses_have_identical_bits(masses: &[f64; 157], count: usize) -> bool {
    let expected_bits = masses[0].to_bits();
    for index in 0..157 {
        if index >= count {
            break;
        }
        if masses[index].to_bits() != expected_bits {
            return false;
        }
    }
    true
}

#[allow(clippy::needless_range_loop)] // The literal bound is part of the classifier contract.
fn deltas_are_nonnegative(deltas: &[f64; 156], count: usize) -> bool {
    for index in 0..156 {
        if index >= count {
            break;
        }
        if deltas[index] < 0.0 {
            return false;
        }
    }
    true
}

fn count_middle_positive(deltas: &[f64; 156], width: usize) -> usize {
    let mut positive = 0_usize;
    for offset in 0..52 {
        if offset >= width {
            break;
        }
        let index = width + offset;
        if deltas[index] > 0.0 {
            positive += 1;
        }
    }
    positive
}

fn late_deltas_are_zero(deltas: &[f64; 156], width: usize) -> bool {
    for offset in 0..52 {
        if offset >= width {
            break;
        }
        let index = 2 * width + offset;
        if deltas[index].to_bits() != 0 {
            return false;
        }
    }
    true
}

#[allow(clippy::needless_range_loop)] // The literal bound is part of the classifier contract.
fn deltas_have_identical_positive_bits(deltas: &[f64; 156], count: usize) -> bool {
    let expected_bits = deltas[0].to_bits();
    if deltas[0] <= 0.0 {
        return false;
    }
    for index in 0..156 {
        if index >= count {
            break;
        }
        if deltas[index].to_bits() != expected_bits {
            return false;
        }
    }
    true
}

#[allow(clippy::needless_range_loop)] // The literal bound is part of the classifier contract.
fn all_separations_retain_sign(separations: &[f64; 53], count: usize) -> bool {
    let first = separations[0];
    if first.to_bits() == 0 {
        return false;
    }
    let sign = first.is_sign_negative();
    for index in 0..53 {
        if index >= count {
            break;
        }
        if separations[index].to_bits() == 0 || separations[index].is_sign_negative() != sign {
            return false;
        }
    }
    true
}
