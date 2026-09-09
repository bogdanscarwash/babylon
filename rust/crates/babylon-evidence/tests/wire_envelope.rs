//! Language-neutral behavioral contracts for the canonical T3 wire envelope.

use babylon_evidence::{
    canonical_envelope, decode_envelope, record_digest, Digest32, PayloadCursor, PayloadEncoder,
    RecordDigest, SfsWireError, T3Record,
};
use babylon_kernel::sha256_of;

#[derive(Debug, PartialEq, Eq)]
struct OneByte(u8);

impl T3Record for OneByte {
    const DOMAIN: &'static [u8] = b"babylon.sfs-sample.v1";
    const MAX_PAYLOAD_BYTES: usize = 1;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u8(self.0)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self(cursor.read_u8()?))
    }
}

macro_rules! domain_record {
    ($name:ident, $domain:expr) => {
        #[derive(Debug, PartialEq, Eq)]
        struct $name;

        impl T3Record for $name {
            const DOMAIN: &'static [u8] = $domain;
            const MAX_PAYLOAD_BYTES: usize = 0;
            type Error = SfsWireError;

            fn encode_payload(&self, _out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
                Ok(())
            }

            fn decode_payload(_cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
                Ok(Self)
            }
        }
    };
}

domain_record!(EmptyDomain, b"");
domain_record!(NulDomain, b"babylon.sfs\0bad.v1");
domain_record!(NonAsciiDomain, b"babylon.sfs-\xff.v1");
domain_record!(MaxDomain, &[b'a'; 64]);
domain_record!(OverlongDomain, &[b'a'; 65]);

#[derive(Debug, PartialEq, Eq)]
struct TooMuchPayload(u8);

impl T3Record for TooMuchPayload {
    const DOMAIN: &'static [u8] = b"babylon.too-much.v1";
    const MAX_PAYLOAD_BYTES: usize = 0;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u8(self.0)
    }

    fn decode_payload(_cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self(0))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct AsciiText(String);

impl T3Record for AsciiText {
    const DOMAIN: &'static [u8] = b"babylon.ascii-text.v1";
    const MAX_PAYLOAD_BYTES: usize = 66;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_ascii("algorithm_id", &self.0, 64)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self(cursor.read_ascii("algorithm_id", 64)?))
    }
}

#[derive(Debug, PartialEq, Eq)]
struct NfcText(String);

impl T3Record for NfcText {
    const DOMAIN: &'static [u8] = b"babylon.nfc-text.v1";
    const MAX_PAYLOAD_BYTES: usize = 258;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_nfc_utf8("text", &self.0, 256)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self(cursor.read_nfc_utf8("text", 256)?))
    }
}

#[derive(Debug, PartialEq)]
struct SignedFloat(f64);

impl T3Record for SignedFloat {
    const DOMAIN: &'static [u8] = b"babylon.signed-f64.v1";
    const MAX_PAYLOAD_BYTES: usize = 8;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_finite_f64("value", self.0)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self(cursor.read_finite_f64("value")?))
    }
}

#[derive(Debug, PartialEq)]
struct NonNegativeFloat(f64);

impl T3Record for NonNegativeFloat {
    const DOMAIN: &'static [u8] = b"babylon.non-negative-f64.v1";
    const MAX_PAYLOAD_BYTES: usize = 8;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_finite_non_negative_f64("value", self.0)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self(cursor.read_finite_non_negative_f64("value")?))
    }
}

#[derive(Debug, PartialEq)]
struct PrimitiveRecord {
    byte: u8,
    short: u16,
    word: u32,
    long: u64,
    digest: Digest32,
    signed: f64,
    algorithm: String,
    text: String,
    nested: OneByte,
}

impl T3Record for PrimitiveRecord {
    const DOMAIN: &'static [u8] = b"babylon.primitive-record.v1";
    const MAX_PAYLOAD_BYTES: usize = 512;
    type Error = SfsWireError;

    fn encode_payload(&self, out: &mut PayloadEncoder) -> Result<(), SfsWireError> {
        out.push_u8(self.byte)?;
        out.push_u16(self.short)?;
        out.push_u32(self.word)?;
        out.push_u64(self.long)?;
        out.push_digest(self.digest)?;
        out.push_finite_f64("signed", self.signed)?;
        out.push_ascii("algorithm", &self.algorithm, 64)?;
        out.push_nfc_utf8("text", &self.text, 256)?;
        out.push_complete_envelope(&self.nested)
    }

    fn decode_payload(cursor: &mut PayloadCursor<'_>) -> Result<Self, SfsWireError> {
        Ok(Self {
            byte: cursor.read_u8()?,
            short: cursor.read_u16()?,
            word: cursor.read_u32()?,
            long: cursor.read_u64()?,
            digest: cursor.read_digest()?,
            signed: cursor.read_finite_f64("signed")?,
            algorithm: cursor.read_ascii("algorithm", 64)?,
            text: cursor.read_nfc_utf8("text", 256)?,
            nested: cursor.read_complete_envelope::<OneByte>()?,
        })
    }
}

fn literal_envelope(domain: &[u8], version: u16, payload: &[u8]) -> Vec<u8> {
    let payload_length = u32::try_from(payload.len()).expect("test payload length fits u32");
    let mut bytes = Vec::new();
    bytes.extend_from_slice(domain);
    bytes.push(0);
    bytes.extend_from_slice(&version.to_be_bytes());
    bytes.extend_from_slice(&payload_length.to_be_bytes());
    bytes.extend_from_slice(payload);
    bytes
}

#[test]
fn envelope_bytes_are_domain_nul_version_length_payload() {
    let expected = [
        b"babylon.sfs-sample.v1".as_slice(),
        &[0],
        &[0, 1],
        &[0, 0, 0, 1],
        &[0xaa],
    ]
    .concat();
    assert_eq!(canonical_envelope(&OneByte(0xaa)).unwrap(), expected);
    let digest: RecordDigest = record_digest(&OneByte(0xaa)).unwrap();
    assert_eq!(digest.as_bytes(), &sha256_of(&expected));
    assert_eq!(digest.to_hex().len(), 64);

    let zero = Digest32::from_bytes([0; 32]);
    assert!(zero.is_zero());
    assert_eq!(zero.to_hex(), "0".repeat(64));
}

#[test]
fn decode_refuses_wrong_version_trailing_and_truncated_envelopes() {
    let bytes = canonical_envelope(&OneByte(0xaa)).unwrap();
    assert_eq!(decode_envelope::<OneByte>(&bytes).unwrap(), OneByte(0xaa));
    assert_eq!(
        decode_envelope::<OneByte>(&bytes[..bytes.len() - 1]),
        Err(SfsWireError::TruncatedEnvelope)
    );

    let mut trailing = bytes.clone();
    trailing.push(0);
    assert_eq!(
        decode_envelope::<OneByte>(&trailing),
        Err(SfsWireError::TrailingBytes { count: 1 })
    );

    let mut wrong_domain = bytes.clone();
    wrong_domain[0] = b'B';
    assert_eq!(
        decode_envelope::<OneByte>(&wrong_domain),
        Err(SfsWireError::WrongDomain)
    );

    let mut version_two = bytes;
    let version_index = OneByte::DOMAIN.len() + 1;
    version_two[version_index..version_index + 2].copy_from_slice(&2_u16.to_be_bytes());
    assert_eq!(
        decode_envelope::<OneByte>(&version_two),
        Err(SfsWireError::UnsupportedSchemaVersion { found: 2 })
    );
}

#[test]
fn static_domains_and_record_payload_maximum_are_closed() {
    assert_eq!(
        canonical_envelope(&EmptyDomain),
        Err(SfsWireError::DomainEmpty)
    );
    assert_eq!(
        canonical_envelope(&NulDomain),
        Err(SfsWireError::DomainContainsNul)
    );
    assert_eq!(
        canonical_envelope(&NonAsciiDomain),
        Err(SfsWireError::DomainNonAscii)
    );
    assert_eq!(
        canonical_envelope(&OverlongDomain),
        Err(SfsWireError::DomainTooLong)
    );
    let max_domain = canonical_envelope(&MaxDomain).unwrap();
    assert_eq!(decode_envelope::<MaxDomain>(&max_domain), Ok(MaxDomain));
    assert_eq!(
        decode_envelope::<EmptyDomain>(&[]),
        Err(SfsWireError::DomainEmpty)
    );
    assert_eq!(
        canonical_envelope(&TooMuchPayload(1)),
        Err(SfsWireError::PayloadTooLong {
            limit: 0,
            actual: 1,
        })
    );
    let oversized = literal_envelope(TooMuchPayload::DOMAIN, 1, &[1]);
    assert_eq!(
        decode_envelope::<TooMuchPayload>(&oversized),
        Err(SfsWireError::PayloadTooLong {
            limit: 0,
            actual: 1,
        })
    );
}

#[test]
fn ascii_strings_are_nonempty_bounded_and_ascii() {
    let valid = AsciiText("sha256".to_owned());
    let encoded = canonical_envelope(&valid).unwrap();
    assert_eq!(decode_envelope::<AsciiText>(&encoded).unwrap(), valid);
    let exact_max = AsciiText("a".repeat(64));
    let exact_max_bytes = canonical_envelope(&exact_max).unwrap();
    assert_eq!(
        decode_envelope::<AsciiText>(&exact_max_bytes).unwrap(),
        exact_max
    );
    assert_eq!(
        canonical_envelope(&AsciiText(String::new())),
        Err(SfsWireError::StringEmpty {
            field: "algorithm_id"
        })
    );
    assert_eq!(
        canonical_envelope(&AsciiText("a".repeat(65))),
        Err(SfsWireError::StringTooLong {
            field: "algorithm_id",
            limit: 64,
            actual: 65,
        })
    );
    assert_eq!(
        canonical_envelope(&AsciiText("café".to_owned())),
        Err(SfsWireError::NonAscii {
            field: "algorithm_id"
        })
    );
}

#[test]
fn unicode_17_nfc_accepts_the_full_scalar_witness_corpus() {
    assert_eq!(unicode_normalization::UNICODE_VERSION, (17, 0, 0));
    let witnesses = ["café", "\u{100}", "مرحبا", "漢字", "q\u{307}", "\u{10000}"];
    for witness in witnesses {
        let value = NfcText(witness.to_owned());
        let bytes = canonical_envelope(&value).unwrap();
        assert_eq!(decode_envelope::<NfcText>(&bytes).unwrap(), value);
    }
    let exact_max = NfcText("a".repeat(256));
    let exact_max_bytes = canonical_envelope(&exact_max).unwrap();
    assert_eq!(
        decode_envelope::<NfcText>(&exact_max_bytes).unwrap(),
        exact_max
    );

    assert_eq!(
        canonical_envelope(&NfcText("cafe\u{301}".to_owned())),
        Err(SfsWireError::NonNfc { field: "text" })
    );
    let decomposed = "cafe\u{301}".as_bytes();
    let mut decomposed_payload = Vec::new();
    decomposed_payload.extend_from_slice(
        &u16::try_from(decomposed.len())
            .expect("decomposed witness length fits u16")
            .to_be_bytes(),
    );
    decomposed_payload.extend_from_slice(decomposed);
    let decomposed_envelope = literal_envelope(NfcText::DOMAIN, 1, &decomposed_payload);
    assert_eq!(
        decode_envelope::<NfcText>(&decomposed_envelope),
        Err(SfsWireError::NonNfc { field: "text" })
    );
    assert_eq!(
        canonical_envelope(&NfcText("a".repeat(257))),
        Err(SfsWireError::StringTooLong {
            field: "text",
            limit: 256,
            actual: 257,
        })
    );

    let invalid_utf8 = literal_envelope(NfcText::DOMAIN, 1, &[0, 1, 0xff]);
    assert_eq!(
        decode_envelope::<NfcText>(&invalid_utf8),
        Err(SfsWireError::InvalidUtf8 { field: "text" })
    );
}

#[test]
fn nfc_comparison_uses_exactly_256_loop_iterations_then_checks_exhaustion() {
    let source = include_str!("../src/wire.rs");
    let function_start = source
        .find("fn is_nfc_bounded")
        .expect("NFC comparison function exists");
    let function = &source[function_start..];
    let loop_position = function
        .find("for _index in 0..256 {")
        .expect("NFC comparison has the literal 256-iteration bound");
    assert!(!function.contains("for _index in 0..=256 {"));
    let exhaustion_position = function
        .find("source.next().is_none() && normalized.next().is_none()")
        .expect("NFC comparison checks both iterators after the loop");
    assert!(loop_position < exhaustion_position);
}

#[test]
fn finite_binary64_is_big_endian_and_normalizes_either_zero_sign() {
    let signed = SignedFloat(-1.5);
    let expected = literal_envelope(SignedFloat::DOMAIN, 1, &(-1.5_f64).to_bits().to_be_bytes());
    assert_eq!(canonical_envelope(&signed).unwrap(), expected);
    assert_eq!(decode_envelope::<SignedFloat>(&expected).unwrap(), signed);

    let negative_zero = canonical_envelope(&SignedFloat(-0.0)).unwrap();
    assert_eq!(&negative_zero[negative_zero.len() - 8..], &[0; 8]);
    let decoded_zero = decode_envelope::<SignedFloat>(&negative_zero).unwrap();
    assert_eq!(decoded_zero.0.to_bits(), 0);

    assert_eq!(
        canonical_envelope(&NonNegativeFloat(-1.0)),
        Err(SfsWireError::Negative { field: "value" })
    );
    for value in [f64::INFINITY, f64::NEG_INFINITY, f64::NAN] {
        assert_eq!(
            canonical_envelope(&SignedFloat(value)),
            Err(SfsWireError::NonFinite { field: "value" })
        );
        let nonfinite = literal_envelope(SignedFloat::DOMAIN, 1, &value.to_bits().to_be_bytes());
        assert_eq!(
            decode_envelope::<SignedFloat>(&nonfinite),
            Err(SfsWireError::NonFinite { field: "value" })
        );
    }
    let encoded_negative = literal_envelope(
        NonNegativeFloat::DOMAIN,
        1,
        &(-1.0_f64).to_bits().to_be_bytes(),
    );
    assert_eq!(
        decode_envelope::<NonNegativeFloat>(&encoded_negative),
        Err(SfsWireError::Negative { field: "value" })
    );
}

#[test]
fn primitive_helpers_and_complete_nested_envelopes_are_exact_inverses() {
    let value = PrimitiveRecord {
        byte: 0x12,
        short: 0x3456,
        word: 0x789a_bcde,
        long: 0x0123_4567_89ab_cdef,
        digest: Digest32::from_bytes([0x5a; 32]),
        signed: -2.5,
        algorithm: "sha256".to_owned(),
        text: "café".to_owned(),
        nested: OneByte(0xaa),
    };
    let bytes = canonical_envelope(&value).unwrap();
    assert_eq!(decode_envelope::<PrimitiveRecord>(&bytes).unwrap(), value);

    let mut underflow = PayloadCursor::new(&[0; 7]);
    assert_eq!(underflow.read_u64(), Err(SfsWireError::TruncatedEnvelope));
}
