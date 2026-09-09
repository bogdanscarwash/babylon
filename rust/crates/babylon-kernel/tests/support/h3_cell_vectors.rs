//! Shared bounded parser for the kernel-owned language-neutral H3 identity vectors.

use std::collections::BTreeSet;

pub(super) const VALID_VECTOR_COUNT: usize = 208;
pub(super) const PENTAGON_VECTOR_COUNT: usize = 12 * 16;
pub(super) const INVALID_RAW_VECTOR_COUNT: usize = 6;
pub(super) const INVALID_SQL_VECTOR_COUNT: usize = 1;
pub(super) const INVALID_TEXT_VECTOR_COUNT: usize = 6;
pub(super) const INVALID_ANCESTOR_VECTOR_COUNT: usize = 2;
const MAX_VECTOR_LINES: usize = 256;

#[derive(Debug)]
pub(super) struct ValidVector {
    pub(super) label: String,
    pub(super) resolution: u8,
    pub(super) text: String,
    pub(super) raw_u64: u64,
    pub(super) sql_i64: i64,
    pub(super) bytes_be: [u8; 8],
    pub(super) immediate_parent: Option<String>,
    pub(super) ancestor_chain_r0_to_self: Vec<String>,
}

impl ValidVector {
    pub(super) fn ancestor_at(&self, resolution: u8) -> Option<&str> {
        self.ancestor_chain_r0_to_self
            .get(usize::from(resolution))
            .map(String::as_str)
    }

    pub(super) fn is_pentagon(&self) -> bool {
        self.label.starts_with("pentagon_")
    }
}

#[derive(Debug)]
pub(super) struct InvalidRawVector {
    pub(super) label: String,
    pub(super) raw_u64: u64,
}

#[derive(Debug)]
pub(super) struct InvalidSqlVector {
    pub(super) label: String,
    pub(super) sql_i64: i64,
}

#[derive(Debug)]
pub(super) struct InvalidTextVector {
    pub(super) label: String,
    pub(super) text: String,
}

#[derive(Debug)]
pub(super) struct InvalidAncestorVector {
    pub(super) label: String,
    pub(super) text: String,
    pub(super) requested_resolution: u8,
}

#[derive(Debug, Default)]
pub(super) struct VectorFixture {
    pub(super) valid: Vec<ValidVector>,
    pub(super) invalid_raw: Vec<InvalidRawVector>,
    pub(super) invalid_sql: Vec<InvalidSqlVector>,
    pub(super) invalid_text: Vec<InvalidTextVector>,
    pub(super) invalid_ancestor: Vec<InvalidAncestorVector>,
}

pub(super) fn load_fixture() -> VectorFixture {
    let source = include_str!("../fixtures/h3_cell_id_vectors_v1.txt");
    let lines = source
        .lines()
        .take(MAX_VECTOR_LINES + 1)
        .collect::<Vec<_>>();
    assert!(
        lines.len() <= MAX_VECTOR_LINES,
        "H3 fixture exceeds the static line bound"
    );

    let mut fixture = VectorFixture::default();
    for line in lines.iter().take(MAX_VECTOR_LINES) {
        if line.is_empty() || line.starts_with('#') {
            continue;
        }
        let fields = line.split('|').take(10).collect::<Vec<_>>();
        match fields.first().copied().unwrap_or_default() {
            "valid" => fixture.valid.push(parse_valid(&fields)),
            "invalid_raw" => fixture.invalid_raw.push(parse_invalid_raw(&fields)),
            "invalid_sql" => fixture.invalid_sql.push(parse_invalid_sql(&fields)),
            "invalid_text" => fixture.invalid_text.push(parse_invalid_text(&fields)),
            "invalid_ancestor" => fixture
                .invalid_ancestor
                .push(parse_invalid_ancestor(&fields)),
            other => panic!("unexpected fixture record kind {other}"),
        }
    }
    assert_fixture_cardinality(&fixture);
    fixture
}

fn assert_fixture_cardinality(fixture: &VectorFixture) {
    assert_eq!(fixture.valid.len(), VALID_VECTOR_COUNT);
    assert_eq!(
        fixture
            .valid
            .iter()
            .take(VALID_VECTOR_COUNT)
            .filter(|vector| vector.is_pentagon())
            .count(),
        PENTAGON_VECTOR_COUNT
    );
    assert_eq!(fixture.invalid_raw.len(), INVALID_RAW_VECTOR_COUNT);
    assert_eq!(fixture.invalid_sql.len(), INVALID_SQL_VECTOR_COUNT);
    assert_eq!(fixture.invalid_text.len(), INVALID_TEXT_VECTOR_COUNT);
    assert_eq!(
        fixture.invalid_ancestor.len(),
        INVALID_ANCESTOR_VECTOR_COUNT
    );
    assert_unique_valid_identities(fixture);
    assert_unique_invalid_cases(fixture);
    assert_exact_resolution_shape(fixture);
}

fn assert_unique_valid_identities(fixture: &VectorFixture) {
    let mut labels = BTreeSet::new();
    let mut raw_identities = BTreeSet::new();
    let mut text_identities = BTreeSet::new();
    for vector in fixture.valid.iter().take(VALID_VECTOR_COUNT) {
        assert!(
            labels.insert(vector.label.as_str()),
            "duplicate H3 vector label"
        );
        assert!(
            raw_identities.insert(vector.raw_u64),
            "duplicate H3 raw identity"
        );
        assert!(
            text_identities.insert(vector.text.as_str()),
            "duplicate H3 text identity"
        );
    }
    assert_eq!(labels.len(), VALID_VECTOR_COUNT);
    assert_eq!(raw_identities.len(), VALID_VECTOR_COUNT);
    assert_eq!(text_identities.len(), VALID_VECTOR_COUNT);
}

fn assert_unique_invalid_cases(fixture: &VectorFixture) {
    let mut labels = BTreeSet::new();
    let mut raw_identities = BTreeSet::new();
    for vector in fixture.invalid_raw.iter().take(INVALID_RAW_VECTOR_COUNT) {
        assert!(
            labels.insert(vector.label.as_str()),
            "duplicate invalid label"
        );
        assert!(
            raw_identities.insert(vector.raw_u64),
            "duplicate invalid raw identity"
        );
    }
    let mut sql_values = BTreeSet::new();
    for vector in fixture.invalid_sql.iter().take(INVALID_SQL_VECTOR_COUNT) {
        assert!(
            labels.insert(vector.label.as_str()),
            "duplicate invalid label"
        );
        assert!(
            sql_values.insert(vector.sql_i64),
            "duplicate invalid SQL identity"
        );
    }
    let mut text_cases = BTreeSet::new();
    for vector in fixture.invalid_text.iter().take(INVALID_TEXT_VECTOR_COUNT) {
        assert!(
            labels.insert(vector.label.as_str()),
            "duplicate invalid label"
        );
        assert!(
            text_cases.insert(vector.text.as_str()),
            "duplicate invalid text case"
        );
    }
    let mut ancestor_cases = BTreeSet::new();
    for vector in fixture
        .invalid_ancestor
        .iter()
        .take(INVALID_ANCESTOR_VECTOR_COUNT)
    {
        assert!(
            labels.insert(vector.label.as_str()),
            "duplicate invalid label"
        );
        assert!(
            ancestor_cases.insert((vector.text.as_str(), vector.requested_resolution)),
            "duplicate invalid ancestor case"
        );
    }
    assert_eq!(labels.len(), 15);
}

fn assert_exact_resolution_shape(fixture: &VectorFixture) {
    for resolution in 0..16 {
        let resolution = u8::try_from(resolution).expect("bounded H3 resolution must fit");
        let level = fixture
            .valid
            .iter()
            .take(VALID_VECTOR_COUNT)
            .filter(|vector| vector.resolution == resolution)
            .collect::<Vec<_>>();
        assert_eq!(level.len(), 13, "resolution {resolution} vector count");
        assert_eq!(
            level
                .iter()
                .take(13)
                .filter(|vector| vector.is_pentagon())
                .count(),
            12,
            "resolution {resolution} pentagon count"
        );
        let ordinary = level
            .iter()
            .take(13)
            .filter(|vector| !vector.is_pentagon())
            .take(2)
            .copied()
            .collect::<Vec<_>>();
        assert_eq!(ordinary.len(), 1, "resolution {resolution} ordinary count");
        assert_ordinary_lineage(fixture, ordinary[0], resolution);
    }
}

fn assert_ordinary_lineage(fixture: &VectorFixture, current: &ValidVector, resolution: u8) {
    assert_eq!(current.label, format!("ordinary_r{resolution}"));
    assert_eq!(
        current.ancestor_chain_r0_to_self.len(),
        usize::from(resolution) + 1
    );
    assert_eq!(current.ancestor_at(resolution), Some(current.text.as_str()));
    if resolution == 0 {
        assert_eq!(current.immediate_parent, None);
        return;
    }
    let prior_label = format!("ordinary_r{}", resolution - 1);
    let prior = fixture
        .valid
        .iter()
        .take(VALID_VECTOR_COUNT)
        .find(|vector| vector.label == prior_label)
        .expect("ordinary H3 lineage must contain its prior resolution");
    assert_eq!(
        current.immediate_parent.as_deref(),
        Some(prior.text.as_str())
    );
    assert_eq!(
        &current.ancestor_chain_r0_to_self[..usize::from(resolution)],
        prior.ancestor_chain_r0_to_self.as_slice()
    );
}

fn parse_valid(fields: &[&str]) -> ValidVector {
    assert_eq!(fields.len(), 9);
    ValidVector {
        label: fields[1].to_owned(),
        resolution: fields[2].parse().expect("valid resolution must parse"),
        text: fields[3].to_owned(),
        raw_u64: u64::from_str_radix(fields[4], 16).expect("valid raw identity must parse"),
        sql_i64: fields[5].parse().expect("valid SQL identity must parse"),
        bytes_be: parse_hex_bytes(fields[6]),
        immediate_parent: (!fields[7].is_empty()).then(|| fields[7].to_owned()),
        ancestor_chain_r0_to_self: fields[8].split(',').take(17).map(str::to_owned).collect(),
    }
}

fn parse_invalid_raw(fields: &[&str]) -> InvalidRawVector {
    assert_eq!(fields.len(), 3);
    InvalidRawVector {
        label: fields[1].to_owned(),
        raw_u64: u64::from_str_radix(fields[2], 16).expect("invalid raw identity bytes must parse"),
    }
}

fn parse_invalid_sql(fields: &[&str]) -> InvalidSqlVector {
    assert_eq!(fields.len(), 3);
    InvalidSqlVector {
        label: fields[1].to_owned(),
        sql_i64: fields[2].parse().expect("invalid SQL identity must parse"),
    }
}

fn parse_invalid_text(fields: &[&str]) -> InvalidTextVector {
    assert_eq!(fields.len(), 3);
    InvalidTextVector {
        label: fields[1].to_owned(),
        text: fields[2].to_owned(),
    }
}

fn parse_invalid_ancestor(fields: &[&str]) -> InvalidAncestorVector {
    assert_eq!(fields.len(), 4);
    InvalidAncestorVector {
        label: fields[1].to_owned(),
        text: fields[2].to_owned(),
        requested_resolution: fields[3]
            .parse()
            .expect("invalid ancestor resolution must parse"),
    }
}

fn parse_hex_bytes(raw_hex16: &str) -> [u8; 8] {
    assert_eq!(raw_hex16.len(), 16);
    let mut bytes = [0_u8; 8];
    for (index, byte) in bytes.iter_mut().enumerate().take(8) {
        let start = index * 2;
        *byte = u8::from_str_radix(&raw_hex16[start..start + 2], 16)
            .expect("big-endian vector byte must parse");
    }
    bytes
}
