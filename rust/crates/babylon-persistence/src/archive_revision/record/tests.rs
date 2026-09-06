use super::*;
use crate::{ArchiveAtomValueV1, ArchiveEvidenceClassV1, ArchiveSubjectKindV1, CampaignId};

const MARKDOWN: &str = "---\nschema: babylon.archive-page.v1\nsubject: county/26163\nverified_tick: 2\ntick_content_hash: 0202020202020202020202020202020202020202020202020202020202020202\n---\n# Wayne County\n\nA retained question.\n\n## Signals\n- **Employment:** 3 — fixture; county/26163\n- **Wages:** 7 — fixture; county/26163\n";

fn atom(record: &RevisionRecord, key: &str, value: &str) -> ArchiveAtomV1 {
    ArchiveAtomV1::try_new(
        record.source.campaign_id(),
        ArchiveAtomSubjectV1::from_page_ref(&record.subject).unwrap(),
        key.to_owned(),
        key.to_owned(),
        record
            .atoms
            .iter()
            .find(|atom| atom.signal_key() == key)
            .map_or(
                ArchiveEvidenceClassV1::Observed,
                ArchiveAtomV1::evidence_class,
            ),
        &ArchiveAtomValueV1::Text(value.to_owned()),
        ArchiveCitationV1::try_new("fixture".to_owned(), "county/26163".to_owned()).unwrap(),
        record.source.tick(),
    )
    .unwrap()
}

fn retained() -> RevisionRecord {
    let mut record = RevisionRecord {
        source: ArchiveReadScopeV2::committed(
            CampaignId::from_uuid(uuid::Uuid::from_bytes([1; 16])),
            2,
            [2; 32],
        )
        .unwrap(),
        subject: ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26163".to_owned())
            .unwrap(),
        effective_tick: 5,
        origin: ArchivePublicationOriginV2::AdoptedHead,
        title: "Wayne County".to_owned(),
        template_sha256: [3; 32],
        content_sha256: Sha256::digest(MARKDOWN.as_bytes()).into(),
        markdown: MARKDOWN.to_owned(),
        search_text: "Wayne County A retained question.".to_owned(),
        provenance_json: r#"[{"source_id":"fixture","locator":"county/26163"}]"#.to_owned(),
        atoms: Vec::new(),
        grants: Vec::new(),
        emission: None,
    };
    record.atoms = vec![
        atom(&record, "subject", &record.title),
        atom(&record, "employment", "3"),
        atom(&record, "wages", "7"),
    ];
    record.grants = ["employment", "subject", "wages"]
        .into_iter()
        .map(|key| GrantDependency {
            subject: record.subject.clone(),
            key: key.to_owned(),
            granted_tick: 0,
            citation: record.atoms[0].citation().clone(),
        })
        .collect();
    record
}

#[test]
fn adoption_preserves_source_markdown_and_atoms_but_has_its_own_effective_identity() {
    let adopted = retained();
    let original = adopted.clone();
    let digest = adopted.digest().unwrap();
    // Independently encoded with Python hashlib + struct.pack, including V1 atom ids.
    assert_eq!(
        digest,
        [
            0x78, 0xf1, 0xf3, 0x41, 0xb7, 0x6f, 0xb7, 0x0c, 0x81, 0x37, 0x24, 0xd9, 0x5c, 0x1f,
            0x85, 0x57, 0xa2, 0x9b, 0x57, 0xf4, 0x62, 0xd2, 0x79, 0x4f, 0x6d, 0x66, 0xf0, 0x1c,
            0x3e, 0xf3, 0xb6, 0xd6
        ]
    );
    assert_eq!(adopted, original);
    assert_eq!(adopted.source.tick(), 2);
    assert_eq!(adopted.effective_tick, 5);
    let mut materialized = adopted.clone();
    materialized.origin = ArchivePublicationOriginV2::Materialized;
    assert_eq!(
        materialized.digest(),
        Err(SemanticArchiveErrorV1::InvalidVerifiedTick)
    );
    materialized.effective_tick = 2;
    assert_eq!(
        materialized.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    assert_eq!(materialized.markdown, adopted.markdown);
    assert_eq!(materialized.content_sha256, adopted.content_sha256);
    assert_eq!(materialized.atoms, adopted.atoms);
}

#[test]
fn digest_covers_provenance_search_template_source_and_order_beyond_markdown_hash() {
    let original = retained();
    let expected = original.digest().unwrap();
    let mut variants = Vec::new();
    let mut changed = original.clone();
    changed.search_text.push('!');
    variants.push(changed);
    let mut changed = original.clone();
    changed.template_sha256[0] ^= 1;
    variants.push(changed);
    let mut changed = original.clone();
    changed.provenance_json.push(' ');
    variants.push(changed);
    let mut changed = original.clone();
    changed.source =
        ArchiveReadScopeV2::committed(changed.source.campaign_id(), 2, [4; 32]).unwrap();
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    let mut changed = original.clone();
    changed.grants[0].granted_tick = 1;
    variants.push(changed);
    for changed in variants {
        assert_eq!(changed.content_sha256, original.content_sha256);
        assert_ne!(changed.digest().unwrap(), expected);
    }
    let mut changed = original;
    changed.atoms.swap(1, 2);
    assert_ne!(changed.digest().unwrap(), expected);
}

#[test]
fn missing_duplicate_foreign_and_future_membership_refuses() {
    let original = retained();
    let mut changed = original.clone();
    changed.atoms.clear();
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::CollectionBound)
    );
    let mut changed = original.clone();
    changed.atoms.push(changed.atoms[1].clone());
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    let mut changed = original.clone();
    changed.source =
        ArchiveReadScopeV2::committed(changed.source.campaign_id(), 3, [2; 32]).unwrap();
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    let mut changed = original.clone();
    changed.subject =
        ArchivePageRefV1::try_new(ArchiveSubjectKindV1::County, "26125".to_owned()).unwrap();
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    let mut changed = original;
    changed.markdown.push('!');
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
    let mut changed = retained();
    changed.grants.remove(0);
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
}

#[test]
fn scope_cannot_disguise_zero_as_a_commit_or_exceed_storage_domain() {
    let campaign = retained().source.campaign_id();
    assert_eq!(
        ArchiveReadScopeV2::committed(campaign, 0, [0; 32]),
        Err(SemanticArchiveErrorV1::InvalidVerifiedTick)
    );
    assert_eq!(
        ArchiveReadScopeV2::committed(campaign, u64::MAX, [0; 32]),
        Err(SemanticArchiveErrorV1::InvalidVerifiedTick)
    );
    assert_eq!(
        ArchiveReadScopeV2::foundation(campaign).tick_content_hash(),
        None
    );
    assert_eq!(
        super::super::ArchivePublicationOriginV2::from_tag(2),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
}

fn witnessed() -> RevisionRecord {
    use crate::{
        ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1, ArchivePageInputV1, ArchiveSignalV1,
        ArchiveSubjectV1, FogSafeArchiveRendererV1,
    };
    let mut record = retained();
    let input = ArchivePageInputV1::try_new(
        ArchiveSubjectV1::try_new(
            record.subject.kind(),
            record.subject.id().to_owned(),
            record.title.clone(),
        )
        .unwrap(),
        record.source.tick(),
        record.source.tick_content_hash().unwrap(),
        "A retained question.".to_owned(),
        vec![
            ArchiveSignalV1::try_new(
                "employment".to_owned(),
                "Employment".to_owned(),
                "3".to_owned(),
                record.grants[0].citation.clone(),
            )
            .unwrap(),
            ArchiveSignalV1::try_new(
                "wages".to_owned(),
                "Wages".to_owned(),
                "7".to_owned(),
                record.grants[0].citation.clone(),
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap();
    let knowledge = ArchiveKnowledgeV1::try_new(
        record
            .grants
            .iter()
            .map(|grant| {
                ArchiveKnowledgeGrantV1::try_new(
                    grant.subject.clone(),
                    grant.key.clone(),
                    grant.granted_tick,
                    grant.citation.clone(),
                )
                .unwrap()
            })
            .collect(),
    )
    .unwrap();
    let renderer = FogSafeArchiveRendererV1::new().unwrap();
    let (page, emission) = renderer.render_with_emission(&input, &knowledge).unwrap();
    record.template_sha256 = renderer.template_sha256();
    record.content_sha256 = page.sha256();
    record.markdown = page.markdown().to_owned();
    record.search_text = page.search_text().to_owned();
    record.provenance_json = serde_json::to_string(page.citations()).unwrap();
    record.atoms = crate::archive::mint_page_atoms(
        record.source.campaign_id(),
        record.source.tick(),
        &input,
        &knowledge,
    )
    .unwrap();
    record.emission = Some(emission);
    record
}

#[test]
fn complete_emission_witness_validates_the_actual_renderer_without_rewriting_retained_bytes() {
    let record = witnessed();
    record.validate().unwrap();
    let digest = record.digest().unwrap();
    let mut materialized = record.clone();
    materialized.origin = ArchivePublicationOriginV2::Materialized;
    materialized.effective_tick = record.source.tick();
    assert_ne!(materialized.digest().unwrap(), digest);
    assert_eq!(record.markdown, materialized.markdown);
    let mut opaque = record.clone();
    opaque.emission = None;
    assert_ne!(opaque.digest().unwrap(), digest);
    let mut mismatch = record;
    mismatch.search_text.push('!');
    assert_eq!(
        mismatch.validate(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
}

#[test]
fn ordinary_retained_head_recovers_only_after_exact_forward_proof() {
    let mut record = witnessed();
    let original = record.clone();
    record.emission = None;
    assert_eq!(
        super::super::recovery::recover(&record).unwrap(),
        original.emission
    );
    assert_eq!(record.markdown, original.markdown);
    assert_eq!(record.atoms, original.atoms);
    record.search_text.push('!');
    assert_eq!(super::super::recovery::recover(&record).unwrap(), None);
}

#[test]
fn ambiguous_legal_question_is_retained_opaque_without_inferred_sections_or_labels() {
    let mut record = witnessed();
    let previous = record.emission.as_ref().unwrap();
    let emission = ArchiveEmissionManifestV2::try_new(
        previous.subject_citation().clone(),
        "A question\n## Related\n- [](subject:county/26099)\n## Signals\nStill a question"
            .to_owned(),
        previous.signals().to_vec(),
        previous.links().to_vec(),
    )
    .unwrap();
    let page = crate::FogSafeArchiveRendererV1::new()
        .unwrap()
        .render_emission(
            &crate::ArchiveSubjectV1::try_new(
                record.subject.kind(),
                record.subject.id().to_owned(),
                record.title.clone(),
            )
            .unwrap(),
            record.source.tick(),
            &record.source.tick_content_hash().unwrap(),
            &emission,
        )
        .unwrap();
    record.markdown = page.markdown().to_owned();
    record.search_text = page.search_text().to_owned();
    record.provenance_json = serde_json::to_string(page.citations()).unwrap();
    record.content_sha256 = page.sha256();
    record.emission = Some(emission);
    record.validate().unwrap();
    record.emission = None;
    let opaque = record.clone();
    record.validate().unwrap();
    assert_eq!(super::super::recovery::recover(&record).unwrap(), None);
    assert_eq!(record, opaque);
}

#[test]
fn emitted_field_without_matching_atom_and_grant_cannot_be_a_witness() {
    let mut record = witnessed();
    record.atoms.remove(1);
    record.grants.remove(0);
    assert_eq!(
        record.validate(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
}

fn later(original: &RevisionRecord) -> RevisionRecord {
    let mut record = original.clone();
    record.source = ArchiveReadScopeV2::committed(record.source.campaign_id(), 6, [2; 32]).unwrap();
    record.effective_tick = 6;
    record.origin = ArchivePublicationOriginV2::Materialized;
    record.markdown = record
        .markdown
        .replace("verified_tick: 2", "verified_tick: 6");
    record.content_sha256 = Sha256::digest(record.markdown.as_bytes()).into();
    record.atoms = original
        .atoms
        .iter()
        .map(|old| {
            ArchiveAtomV1::try_new(
                record.source.campaign_id(),
                old.subject().clone(),
                old.signal_key().to_owned(),
                old.grant_key().to_owned(),
                old.evidence_class(),
                old.value(),
                old.citation().clone(),
                6,
            )
            .unwrap()
        })
        .collect();
    record
}

#[test]
fn later_publication_does_not_claim_unchanged_atoms_changed_with_their_source_tick() {
    let previous = witnessed();
    let next = later(&previous);
    assert_ne!(previous.atoms[1].atom_id(), next.atoms[1].atom_id());
    assert!(super::super::changes::between(Some(&previous), &next)
        .unwrap()
        .is_empty());
}

#[test]
fn changed_value_and_removed_assertion_keep_exact_original_atoms_without_zero_fill() {
    let previous = witnessed();
    let mut next = later(&previous);
    let emission = ArchiveEmissionManifestV2::try_new(
        next.grants[0].citation.clone(),
        "A retained question.".to_owned(),
        vec![crate::ArchiveSignalV1::try_new(
            "employment".to_owned(),
            "Employment".to_owned(),
            "5".to_owned(),
            next.grants[0].citation.clone(),
        )
        .unwrap()],
        vec![],
    )
    .unwrap();
    let page = crate::FogSafeArchiveRendererV1::new()
        .unwrap()
        .render_emission(
            &crate::ArchiveSubjectV1::try_new(
                next.subject.kind(),
                next.subject.id().to_owned(),
                next.title.clone(),
            )
            .unwrap(),
            next.source.tick(),
            &next.source.tick_content_hash().unwrap(),
            &emission,
        )
        .unwrap();
    next.markdown = page.markdown().to_owned();
    next.content_sha256 = page.sha256();
    next.search_text = page.search_text().to_owned();
    next.provenance_json = serde_json::to_string(page.citations()).unwrap();
    next.emission = Some(emission);
    next.atoms[1] = atom(&next, "employment", "5");
    next.atoms.pop();
    next.grants.pop();
    let changes = super::super::changes::between(Some(&previous), &next).unwrap();
    assert_eq!(changes.len(), 2);
    assert_eq!(changes[0].signal_key, "employment");
    assert_eq!(changes[0].before.as_ref(), Some(&previous.atoms[1]));
    assert_eq!(changes[0].after.as_ref(), Some(&next.atoms[1]));
    assert_eq!(changes[1].signal_key, "wages");
    assert_eq!(changes[1].before.as_ref(), Some(&previous.atoms[2]));
    assert_eq!(changes[1].after, None);
    assert!(changes.iter().all(|change| change.publication_tick == 6));
}

#[test]
fn changelog_refuses_reverse_duplicate_and_cross_scope_publications() {
    let previous = witnessed();
    let next = later(&previous);
    for (before, after) in [(&next, &previous), (&previous, &previous)] {
        assert_eq!(
            super::super::changes::between(Some(before), after),
            Err(SemanticArchiveErrorV1::StoredPageMismatch)
        );
    }
    let mut foreign = next;
    foreign.source = ArchiveReadScopeV2::committed(
        CampaignId::from_uuid(uuid::Uuid::from_bytes([9; 16])),
        6,
        [2; 32],
    )
    .unwrap();
    assert_eq!(
        super::super::changes::between(Some(&previous), &foreign),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
}
