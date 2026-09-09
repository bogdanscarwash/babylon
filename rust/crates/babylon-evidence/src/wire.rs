//! Canonical bounded wire envelope and payload primitives for T3 records.

use unicode_normalization::UnicodeNormalization;

use crate::Digest32;

const SCHEMA_VERSION: u16 = 1;
const MAX_DOMAIN_BYTES: usize = 64;
const MAX_NFC_SCALARS: usize = 256;

const _: () = assert!(unicode_normalization::UNICODE_VERSION.0 == 17);
const _: () = assert!(unicode_normalization::UNICODE_VERSION.1 == 0);
const _: () = assert!(unicode_normalization::UNICODE_VERSION.2 == 0);

/// Exact refusals shared by all canonical T3 record families.
#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SfsWireError {
    /// The encoded schema version is not V1.
    UnsupportedSchemaVersion { found: u16 },
    /// The encoded domain differs from the record's exact domain.
    WrongDomain,
    /// The record declares an empty domain.
    DomainEmpty,
    /// The record domain contains its reserved terminator byte.
    DomainContainsNul,
    /// The record domain contains a non-ASCII byte.
    DomainNonAscii,
    /// The record domain exceeds 64 bytes.
    DomainTooLong,
    /// The payload exceeds the record-specific maximum.
    PayloadTooLong { limit: usize, actual: usize },
    /// A declared collection count exceeds its field-specific maximum.
    CountTooLarge {
        field: &'static str,
        limit: usize,
        actual: usize,
    },
    /// A required string is empty.
    StringEmpty { field: &'static str },
    /// A string exceeds its field-specific maximum.
    StringTooLong {
        field: &'static str,
        limit: usize,
        actual: usize,
    },
    /// A field that requires ASCII contains another byte.
    NonAscii { field: &'static str },
    /// A decoded string is not valid UTF-8.
    InvalidUtf8 { field: &'static str },
    /// A Unicode string is not NFC under Unicode 17.0.0.
    NonNfc { field: &'static str },
    /// A sequence contains a duplicate canonical entry.
    DuplicateEntry { field: &'static str },
    /// A sequence is not in its canonical byte order.
    OutOfOrder { field: &'static str },
    /// A decoded closed code is unknown.
    InvalidCode { field: &'static str, value: u8 },
    /// A binary64 field is NaN or infinite.
    NonFinite { field: &'static str },
    /// A non-negative binary64 field is negative.
    Negative { field: &'static str },
    /// A checked size or index calculation overflowed.
    ArithmeticOverflow { field: &'static str },
    /// The envelope or payload ends before a declared value is complete.
    TruncatedEnvelope,
    /// Bytes remain after the complete declared value.
    TrailingBytes { count: usize },
}

/// One record family that has an exact domain and bounded payload.
pub trait T3Record: Sized {
    /// Exact nonempty ASCII domain without its NUL terminator.
    const DOMAIN: &'static [u8];
    /// Maximum encoded payload bytes for this record family.
    const MAX_PAYLOAD_BYTES: usize;
    /// Record-specific error that retains wire refusals as a distinct case.
    type Error: From<SfsWireError>;

    /// Encodes the record-specific payload in canonical field order.
    ///
    /// # Errors
    /// Returns the first exact wire refusal without publishing partial bytes.
    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError>;

    /// Decodes and validates the record-specific payload.
    ///
    /// # Errors
    /// Returns the first exact wire or record-specific semantic refusal.
    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, Self::Error>;
}

/// Private staged payload writer with a fixed record-specific byte ceiling.
#[derive(Debug)]
pub struct PayloadEncoder {
    bytes: Vec<u8>,
    limit: usize,
}

impl PayloadEncoder {
    /// Creates an empty staged payload with an exact maximum size.
    #[must_use]
    pub const fn new(limit: usize) -> Self {
        Self {
            bytes: Vec::new(),
            limit,
        }
    }

    fn reserve(&mut self, additional: usize) -> Result<(), SfsWireError> {
        let actual = self
            .bytes
            .len()
            .checked_add(additional)
            .ok_or(SfsWireError::ArithmeticOverflow { field: "payload" })?;
        if actual > self.limit {
            return Err(SfsWireError::PayloadTooLong {
                limit: self.limit,
                actual,
            });
        }
        self.bytes
            .try_reserve_exact(additional)
            .map_err(|_| SfsWireError::ArithmeticOverflow { field: "payload" })
    }

    fn push_bytes(&mut self, bytes: &[u8]) -> Result<(), SfsWireError> {
        self.reserve(bytes.len())?;
        self.bytes.extend_from_slice(bytes);
        Ok(())
    }

    /// Appends one unsigned byte.
    ///
    /// # Errors
    /// Returns `PayloadTooLong` before the staged payload crosses its ceiling.
    pub fn push_u8(&mut self, value: u8) -> Result<(), SfsWireError> {
        self.push_bytes(&[value])
    }

    /// Appends one big-endian unsigned 16-bit integer.
    ///
    /// # Errors
    /// Returns `PayloadTooLong` before the staged payload crosses its ceiling.
    pub fn push_u16(&mut self, value: u16) -> Result<(), SfsWireError> {
        self.push_bytes(&value.to_be_bytes())
    }

    /// Appends one big-endian unsigned 32-bit integer.
    ///
    /// # Errors
    /// Returns `PayloadTooLong` before the staged payload crosses its ceiling.
    pub fn push_u32(&mut self, value: u32) -> Result<(), SfsWireError> {
        self.push_bytes(&value.to_be_bytes())
    }

    /// Appends one big-endian unsigned 64-bit integer.
    ///
    /// # Errors
    /// Returns `PayloadTooLong` before the staged payload crosses its ceiling.
    pub fn push_u64(&mut self, value: u64) -> Result<(), SfsWireError> {
        self.push_bytes(&value.to_be_bytes())
    }

    /// Appends one exact opaque digest.
    ///
    /// # Errors
    /// Returns `PayloadTooLong` before the staged payload crosses its ceiling.
    pub fn push_digest(&mut self, value: Digest32) -> Result<(), SfsWireError> {
        self.push_bytes(value.as_bytes())
    }

    /// Appends one signed finite big-endian IEEE-754 binary64 value.
    ///
    /// # Errors
    /// Returns `NonFinite` or the exact payload-size refusal.
    pub fn push_finite_f64(&mut self, field: &'static str, value: f64) -> Result<(), SfsWireError> {
        let normalized = normalize_finite(field, value)?;
        self.push_u64(normalized.to_bits())
    }

    /// Appends one finite non-negative big-endian IEEE-754 binary64 value.
    ///
    /// # Errors
    /// Returns `NonFinite`, `Negative`, or the exact payload-size refusal.
    pub fn push_finite_non_negative_f64(
        &mut self,
        field: &'static str,
        value: f64,
    ) -> Result<(), SfsWireError> {
        let normalized = normalize_finite(field, value)?;
        if normalized < 0.0 {
            return Err(SfsWireError::Negative { field });
        }
        self.push_u64(normalized.to_bits())
    }

    /// Appends a nonempty length-prefixed ASCII string.
    ///
    /// # Errors
    /// Returns the exact empty, length, ASCII, or payload-size refusal.
    pub fn push_ascii(
        &mut self,
        field: &'static str,
        value: &str,
        maximum: usize,
    ) -> Result<(), SfsWireError> {
        validate_ascii(field, value, maximum)?;
        let length = u16::try_from(value.len()).map_err(|_| SfsWireError::StringTooLong {
            field,
            limit: maximum.min(usize::from(u16::MAX)),
            actual: value.len(),
        })?;
        self.push_u16(length)?;
        self.push_bytes(value.as_bytes())
    }

    /// Appends a nonempty length-prefixed Unicode-17 NFC string.
    ///
    /// # Errors
    /// Returns the exact empty, length, NFC, or payload-size refusal.
    pub fn push_nfc_utf8(
        &mut self,
        field: &'static str,
        value: &str,
        maximum: usize,
    ) -> Result<(), SfsWireError> {
        validate_nfc(field, value, maximum)?;
        let length = u16::try_from(value.len()).map_err(|_| SfsWireError::StringTooLong {
            field,
            limit: maximum.min(usize::from(u16::MAX)),
            actual: value.len(),
        })?;
        self.push_u16(length)?;
        self.push_bytes(value.as_bytes())
    }

    /// Appends one self-delimiting complete canonical nested envelope.
    ///
    /// # Errors
    /// Returns the nested record's wire refusal or this payload's size refusal.
    pub fn push_complete_envelope<T: T3Record>(&mut self, record: &T) -> Result<(), SfsWireError> {
        let envelope = canonical_envelope(record)?;
        self.push_bytes(&envelope)
    }
}

/// Bounded cursor over one already-sized canonical payload.
#[derive(Debug)]
pub struct PayloadCursor<'a> {
    bytes: &'a [u8],
    index: usize,
}

impl<'a> PayloadCursor<'a> {
    /// Creates a cursor at byte zero.
    #[must_use]
    pub const fn new(bytes: &'a [u8]) -> Self {
        Self { bytes, index: 0 }
    }

    fn take(&mut self, count: usize) -> Result<&'a [u8], SfsWireError> {
        let end = self
            .index
            .checked_add(count)
            .ok_or(SfsWireError::ArithmeticOverflow { field: "cursor" })?;
        let value = self
            .bytes
            .get(self.index..end)
            .ok_or(SfsWireError::TruncatedEnvelope)?;
        self.index = end;
        Ok(value)
    }

    pub(crate) const fn remaining(&self) -> usize {
        self.bytes.len().saturating_sub(self.index)
    }

    fn finish(&self) -> Result<(), SfsWireError> {
        let count = self.remaining();
        if count == 0 {
            Ok(())
        } else {
            Err(SfsWireError::TrailingBytes { count })
        }
    }

    /// Reads one unsigned byte.
    ///
    /// # Errors
    /// Returns `TruncatedEnvelope` on cursor underflow.
    pub fn read_u8(&mut self) -> Result<u8, SfsWireError> {
        self.take(1)?
            .first()
            .copied()
            .ok_or(SfsWireError::TruncatedEnvelope)
    }

    /// Reads one big-endian unsigned 16-bit integer.
    ///
    /// # Errors
    /// Returns `TruncatedEnvelope` on cursor underflow.
    pub fn read_u16(&mut self) -> Result<u16, SfsWireError> {
        let bytes = self
            .take(2)?
            .try_into()
            .map_err(|_| SfsWireError::TruncatedEnvelope)?;
        Ok(u16::from_be_bytes(bytes))
    }

    /// Reads one big-endian unsigned 32-bit integer.
    ///
    /// # Errors
    /// Returns `TruncatedEnvelope` on cursor underflow.
    pub fn read_u32(&mut self) -> Result<u32, SfsWireError> {
        let bytes = self
            .take(4)?
            .try_into()
            .map_err(|_| SfsWireError::TruncatedEnvelope)?;
        Ok(u32::from_be_bytes(bytes))
    }

    /// Reads one big-endian unsigned 64-bit integer.
    ///
    /// # Errors
    /// Returns `TruncatedEnvelope` on cursor underflow.
    pub fn read_u64(&mut self) -> Result<u64, SfsWireError> {
        let bytes = self
            .take(8)?
            .try_into()
            .map_err(|_| SfsWireError::TruncatedEnvelope)?;
        Ok(u64::from_be_bytes(bytes))
    }

    /// Reads one exact opaque digest.
    ///
    /// # Errors
    /// Returns `TruncatedEnvelope` on cursor underflow.
    pub fn read_digest(&mut self) -> Result<Digest32, SfsWireError> {
        let bytes = self
            .take(32)?
            .try_into()
            .map_err(|_| SfsWireError::TruncatedEnvelope)?;
        Ok(Digest32::from_bytes(bytes))
    }

    /// Reads one signed finite big-endian IEEE-754 binary64 value.
    ///
    /// # Errors
    /// Returns `TruncatedEnvelope` or `NonFinite`.
    pub fn read_finite_f64(&mut self, field: &'static str) -> Result<f64, SfsWireError> {
        normalize_finite(field, f64::from_bits(self.read_u64()?))
    }

    /// Reads one finite non-negative big-endian IEEE-754 binary64 value.
    ///
    /// # Errors
    /// Returns `TruncatedEnvelope`, `NonFinite`, or `Negative`.
    pub fn read_finite_non_negative_f64(
        &mut self,
        field: &'static str,
    ) -> Result<f64, SfsWireError> {
        let value = self.read_finite_f64(field)?;
        if value < 0.0 {
            Err(SfsWireError::Negative { field })
        } else {
            Ok(value)
        }
    }

    /// Reads one nonempty length-prefixed ASCII string.
    ///
    /// # Errors
    /// Returns the exact truncation, UTF-8, empty, length, or ASCII refusal.
    pub fn read_ascii(
        &mut self,
        field: &'static str,
        maximum: usize,
    ) -> Result<String, SfsWireError> {
        let length = usize::from(self.read_u16()?);
        validate_declared_string_length(field, length, maximum)?;
        let value = std::str::from_utf8(self.take(length)?)
            .map_err(|_| SfsWireError::InvalidUtf8 { field })?;
        validate_ascii(field, value, maximum)?;
        Ok(value.to_owned())
    }

    /// Reads one nonempty length-prefixed Unicode-17 NFC string.
    ///
    /// # Errors
    /// Returns the exact truncation, UTF-8, empty, length, or NFC refusal.
    pub fn read_nfc_utf8(
        &mut self,
        field: &'static str,
        maximum: usize,
    ) -> Result<String, SfsWireError> {
        let length = usize::from(self.read_u16()?);
        validate_declared_string_length(field, length, maximum)?;
        let value = std::str::from_utf8(self.take(length)?)
            .map_err(|_| SfsWireError::InvalidUtf8 { field })?;
        validate_nfc(field, value, maximum)?;
        Ok(value.to_owned())
    }

    /// Reads and validates one self-delimiting complete nested envelope.
    ///
    /// # Errors
    /// Returns the nested record's exact wire or semantic refusal.
    pub fn read_complete_envelope<T: T3Record>(&mut self) -> Result<T, T::Error> {
        let remaining = self
            .bytes
            .get(self.index..)
            .ok_or(SfsWireError::TruncatedEnvelope)
            .map_err(T::Error::from)?;
        let length = complete_envelope_length::<T>(remaining).map_err(T::Error::from)?;
        let envelope = self.take(length).map_err(T::Error::from)?;
        decode_envelope::<T>(envelope)
    }
}

/// Encodes one record as `domain || NUL || version || length || payload`.
///
/// # Errors
/// Returns the first exact static-domain, payload, string, or numeric refusal.
pub fn canonical_envelope<T: T3Record>(record: &T) -> Result<Vec<u8>, SfsWireError> {
    validate_domain(T::DOMAIN)?;
    let mut payload = PayloadEncoder::new(T::MAX_PAYLOAD_BYTES);
    record.encode_payload(&mut payload)?;
    let payload_length =
        u32::try_from(payload.bytes.len()).map_err(|_| SfsWireError::PayloadTooLong {
            limit: usize::try_from(u32::MAX).unwrap_or(usize::MAX),
            actual: payload.bytes.len(),
        })?;
    let header_length = T::DOMAIN
        .len()
        .checked_add(7)
        .ok_or(SfsWireError::ArithmeticOverflow { field: "envelope" })?;
    let complete_length = header_length
        .checked_add(payload.bytes.len())
        .ok_or(SfsWireError::ArithmeticOverflow { field: "envelope" })?;
    let mut output = Vec::with_capacity(complete_length);
    output.extend_from_slice(T::DOMAIN);
    output.push(0);
    output.extend_from_slice(&SCHEMA_VERSION.to_be_bytes());
    output.extend_from_slice(&payload_length.to_be_bytes());
    output.extend_from_slice(&payload.bytes);
    Ok(output)
}

/// Decodes one exact complete canonical record envelope.
///
/// # Errors
/// Returns the first exact static-domain, envelope, payload, or semantic refusal.
pub fn decode_envelope<T: T3Record>(bytes: &[u8]) -> Result<T, T::Error> {
    validate_domain(T::DOMAIN).map_err(T::Error::from)?;
    let domain_end = find_domain_end(bytes).map_err(T::Error::from)?;
    if bytes.get(..domain_end) != Some(T::DOMAIN) {
        return Err(T::Error::from(SfsWireError::WrongDomain));
    }
    let after_domain = domain_end
        .checked_add(1)
        .ok_or(SfsWireError::ArithmeticOverflow { field: "envelope" })
        .map_err(T::Error::from)?;
    let envelope_tail = bytes
        .get(after_domain..)
        .ok_or(SfsWireError::TruncatedEnvelope)
        .map_err(T::Error::from)?;
    let mut outer = PayloadCursor::new(envelope_tail);
    let version = outer.read_u16().map_err(T::Error::from)?;
    if version != SCHEMA_VERSION {
        return Err(T::Error::from(SfsWireError::UnsupportedSchemaVersion {
            found: version,
        }));
    }
    let payload_length = usize::try_from(outer.read_u32().map_err(T::Error::from)?)
        .map_err(|_| SfsWireError::ArithmeticOverflow { field: "payload" })
        .map_err(T::Error::from)?;
    if payload_length > outer.remaining() {
        return Err(T::Error::from(SfsWireError::TruncatedEnvelope));
    }
    if payload_length > T::MAX_PAYLOAD_BYTES {
        return Err(T::Error::from(SfsWireError::PayloadTooLong {
            limit: T::MAX_PAYLOAD_BYTES,
            actual: payload_length,
        }));
    }
    let payload = outer.take(payload_length).map_err(T::Error::from)?;
    let mut cursor = PayloadCursor::new(payload);
    let record = T::decode_payload(&mut cursor)?;
    cursor.finish().map_err(T::Error::from)?;
    outer.finish().map_err(T::Error::from)?;
    Ok(record)
}

fn validate_domain(domain: &[u8]) -> Result<(), SfsWireError> {
    if domain.is_empty() {
        return Err(SfsWireError::DomainEmpty);
    }
    if domain.len() > MAX_DOMAIN_BYTES {
        return Err(SfsWireError::DomainTooLong);
    }
    for index in 0..64 {
        let Some(byte) = domain.get(index) else {
            break;
        };
        if *byte == 0 {
            return Err(SfsWireError::DomainContainsNul);
        }
        if !byte.is_ascii() {
            return Err(SfsWireError::DomainNonAscii);
        }
    }
    Ok(())
}

fn find_domain_end(bytes: &[u8]) -> Result<usize, SfsWireError> {
    for index in 0..=64 {
        let Some(byte) = bytes.get(index) else {
            return Err(SfsWireError::TruncatedEnvelope);
        };
        if *byte == 0 {
            return Ok(index);
        }
    }
    Err(SfsWireError::WrongDomain)
}

fn complete_envelope_length<T: T3Record>(bytes: &[u8]) -> Result<usize, SfsWireError> {
    validate_domain(T::DOMAIN)?;
    let header_length = T::DOMAIN
        .len()
        .checked_add(7)
        .ok_or(SfsWireError::ArithmeticOverflow { field: "envelope" })?;
    if bytes.len() < header_length {
        return Err(SfsWireError::TruncatedEnvelope);
    }
    if bytes.get(..T::DOMAIN.len()) != Some(T::DOMAIN) || bytes.get(T::DOMAIN.len()) != Some(&0) {
        return Err(SfsWireError::WrongDomain);
    }
    let length_start = T::DOMAIN
        .len()
        .checked_add(3)
        .ok_or(SfsWireError::ArithmeticOverflow { field: "envelope" })?;
    let length_end = length_start
        .checked_add(4)
        .ok_or(SfsWireError::ArithmeticOverflow { field: "envelope" })?;
    let length_bytes = bytes
        .get(length_start..length_end)
        .ok_or(SfsWireError::TruncatedEnvelope)?
        .try_into()
        .map_err(|_| SfsWireError::TruncatedEnvelope)?;
    let payload_length = usize::try_from(u32::from_be_bytes(length_bytes))
        .map_err(|_| SfsWireError::ArithmeticOverflow { field: "payload" })?;
    let complete_length = header_length
        .checked_add(payload_length)
        .ok_or(SfsWireError::ArithmeticOverflow { field: "envelope" })?;
    if complete_length > bytes.len() {
        return Err(SfsWireError::TruncatedEnvelope);
    }
    Ok(complete_length)
}

fn normalize_finite(field: &'static str, value: f64) -> Result<f64, SfsWireError> {
    if !value.is_finite() {
        return Err(SfsWireError::NonFinite { field });
    }
    if value == 0.0 {
        Ok(0.0)
    } else {
        Ok(value)
    }
}

fn validate_declared_string_length(
    field: &'static str,
    actual: usize,
    maximum: usize,
) -> Result<(), SfsWireError> {
    if actual == 0 {
        return Err(SfsWireError::StringEmpty { field });
    }
    let wire_maximum = maximum.min(usize::from(u16::MAX));
    if actual > wire_maximum {
        return Err(SfsWireError::StringTooLong {
            field,
            limit: wire_maximum,
            actual,
        });
    }
    Ok(())
}

fn validate_ascii(field: &'static str, value: &str, maximum: usize) -> Result<(), SfsWireError> {
    validate_declared_string_length(field, value.len(), maximum)?;
    if value.is_ascii() {
        Ok(())
    } else {
        Err(SfsWireError::NonAscii { field })
    }
}

fn validate_nfc(field: &'static str, value: &str, maximum: usize) -> Result<(), SfsWireError> {
    validate_declared_string_length(field, value.len(), maximum)?;
    validate_scalar_count(field, value)?;
    if is_nfc_bounded(value) {
        Ok(())
    } else {
        Err(SfsWireError::NonNfc { field })
    }
}

fn validate_scalar_count(field: &'static str, value: &str) -> Result<(), SfsWireError> {
    let mut scalars = value.chars();
    for _index in 0..256 {
        if scalars.next().is_none() {
            return Ok(());
        }
    }
    if scalars.next().is_some() {
        Err(SfsWireError::StringTooLong {
            field,
            limit: MAX_NFC_SCALARS,
            actual: MAX_NFC_SCALARS + 1,
        })
    } else {
        Ok(())
    }
}

fn is_nfc_bounded(value: &str) -> bool {
    let mut source = value.chars();
    let mut normalized = value.nfc();
    for _index in 0..256 {
        match (source.next(), normalized.next()) {
            (None, None) => return true,
            (Some(left), Some(right)) if left == right => {}
            _ => return false,
        }
    }
    source.next().is_none() && normalized.next().is_none()
}
