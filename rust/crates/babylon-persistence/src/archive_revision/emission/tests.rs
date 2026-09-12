use super::*;
use crate::{ArchiveKnowledge, ArchiveKnowledgeGrant, ArchiveSubjectKind, FogSafeArchiveRenderer};

fn county(id: &str) -> ArchivePageRef {
    ArchivePageRef::try_new(ArchiveSubjectKind::County, id.to_owned()).unwrap()
}

fn citation() -> ArchiveCitation {
    ArchiveCitation::try_new("source\nline".to_owned(), "locator\nline".to_owned()).unwrap()
}

fn manifest() -> ArchiveEmissionManifest {
    ArchiveEmissionManifest::try_new(
        citation(),
        "A question\n## Signals\n## Related\nwith retained text".to_owned(),
        vec![ArchiveSignal::try_new(
            "employment".to_owned(),
            "Workers\nreading".to_owned(),
            "3\nper scope".to_owned(),
            citation(),
        )
        .unwrap()],
        vec![
            ArchiveEmissionLink::try_new(
                county("26125"),
                Some("District [North]\n](subject:county/26099)".to_owned()),
            )
            .unwrap(),
            ArchiveEmissionLink::try_new(county("26099"), None).unwrap(),
        ],
    )
    .unwrap()
}

#[test]
fn manifest_roundtrips_lawful_multiline_and_delimiter_text_without_parsing_markdown() {
    let manifest = manifest();
    let encoded = manifest.encode().unwrap();
    assert_eq!(ArchiveEmissionManifest::decode(&encoded).unwrap(), manifest);
    assert_eq!(manifest.links()[1].known_label(), None);
    let input = manifest
        .atom_input(
            ArchiveSubject::try_new(
                ArchiveSubjectKind::County,
                "26163".to_owned(),
                "Wayne\nCounty".to_owned(),
            )
            .unwrap(),
            2,
            [2; 32],
        )
        .unwrap();
    let knowledge = ArchiveKnowledge::try_new(vec![
        ArchiveKnowledgeGrant::try_new(county("26163"), "subject".to_owned(), 0, citation())
            .unwrap(),
        ArchiveKnowledgeGrant::try_new(county("26163"), "employment".to_owned(), 0, citation())
            .unwrap(),
        ArchiveKnowledgeGrant::try_new(county("26125"), "subject".to_owned(), 0, citation())
            .unwrap(),
    ])
    .unwrap();
    // The existing V1 renderer accepts these exact strings. A successor must too.
    let page = FogSafeArchiveRenderer::new()
        .unwrap()
        .render(&input, &knowledge)
        .unwrap();
    assert!(page.markdown().contains("# Wayne\nCounty"));
    assert!(page.markdown().contains("Workers\nreading"));
    assert!(page
        .markdown()
        .contains("District [North]\n](subject:county/26099)"));
    assert_eq!(input.links().len(), 1);
    assert_eq!(input.links()[0].target(), &county("26125"));
}

#[test]
fn strict_codec_refuses_unknown_layout_fields_noncanonical_bytes_and_duplicate_targets() {
    let manifest = manifest();
    let encoded = manifest.encode().unwrap();
    for invalid in [
        format!(" {encoded}"),
        encoded.replace("\"layout_version\":2", "\"layout_version\":3"),
        encoded.replacen('{', "{\"unrecognized\":true,", 1),
    ] {
        assert!(ArchiveEmissionManifest::decode(&invalid).is_err());
    }
    let duplicate = vec![manifest.links()[0].clone(), manifest.links()[0].clone()];
    assert_eq!(
        ArchiveEmissionManifest::try_new(citation(), "question".to_owned(), vec![], duplicate),
        Err(SemanticArchiveError::DuplicateKey)
    );
}

#[test]
fn every_ordered_emission_field_changes_its_canonical_bytes_and_hidden_label_is_absent() {
    let original = manifest();
    let expected = original.encode().unwrap();
    for change in 0..7 {
        let mut next = original.clone();
        match change {
            0 => {
                next.subject_citation =
                    ArchiveCitation::try_new("changed".to_owned(), "loc".to_owned()).unwrap();
            }
            1 => next.question.push('!'),
            2 => {
                next.signals[0] = ArchiveSignal::try_new(
                    "employment".to_owned(),
                    "changed".to_owned(),
                    "3".to_owned(),
                    citation(),
                )
                .unwrap();
            }
            3 => next.signals.clear(),
            4 => next.links[0].known_label = Some("changed".to_owned()),
            5 => next.links[0].target = county("26001"),
            6 => next.links.swap(0, 1),
            _ => unreachable!(),
        }
        assert_ne!(next.encode().unwrap(), expected);
    }
    assert!(expected.contains("\"known_label\":null"));
    assert!(!expected.contains("invented unknown label"));
}
