use super::*;
use crate::{ArchiveAtomValueV1, ArchiveEvidenceClassV1, ArchiveSubjectKindV1, CampaignId};

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

#[test]
fn required_emission_binds_provenance_search_template_source_and_atom_order() {
    let original = witnessed();
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
    variants.push(changed);
    let mut changed = original.clone();
    changed.atoms.swap(1, 2);
    variants.push(changed);
    for changed in variants {
        assert_eq!(changed.content_sha256, original.content_sha256);
        assert_eq!(
            changed.digest(),
            Err(SemanticArchiveErrorV1::StoredPageMismatch)
        );
    }
    let mut changed = original;
    changed.grants[0].granted_tick = 1;
    assert_ne!(changed.digest().unwrap(), expected);
}

#[test]
fn missing_duplicate_foreign_and_future_membership_refuses() {
    let original = witnessed();
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
    changed.effective_tick = 3;
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
    let mut changed = witnessed();
    changed.grants.remove(0);
    assert_eq!(
        changed.digest(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
}

#[test]
fn scope_cannot_disguise_zero_as_a_commit_or_exceed_storage_domain() {
    let campaign = witnessed().source.campaign_id();
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
}

fn witnessed() -> RevisionRecord {
    use crate::{
        ArchiveKnowledgeGrantV1, ArchiveKnowledgeV1, ArchivePageInputV1, ArchiveSignalV1,
        ArchiveSubjectV1, FogSafeArchiveRendererV1,
    };
    let source = ArchiveReadScopeV2::committed(
        CampaignId::from_uuid(uuid::Uuid::from_bytes([1; 16])),
        2,
        [2; 32],
    )
    .unwrap();
    let subject = ArchiveSubjectV1::try_new(
        ArchiveSubjectKindV1::County,
        "26163".to_owned(),
        "Wayne County".to_owned(),
    )
    .unwrap();
    let citation =
        ArchiveCitationV1::try_new("fixture".to_owned(), "county/26163".to_owned()).unwrap();
    let grants: Vec<_> = ["employment", "subject", "wages"]
        .into_iter()
        .map(|key| GrantDependency {
            subject: subject.page_ref().clone(),
            key: key.to_owned(),
            granted_tick: 0,
            citation: citation.clone(),
        })
        .collect();
    let input = ArchivePageInputV1::try_new(
        subject.clone(),
        source.tick(),
        source.tick_content_hash().unwrap(),
        "A retained question.".to_owned(),
        vec![
            ArchiveSignalV1::try_new(
                "employment".to_owned(),
                "Employment".to_owned(),
                "3".to_owned(),
                citation.clone(),
            )
            .unwrap(),
            ArchiveSignalV1::try_new(
                "wages".to_owned(),
                "Wages".to_owned(),
                "7".to_owned(),
                citation,
            )
            .unwrap(),
        ],
        vec![],
    )
    .unwrap();
    let knowledge = ArchiveKnowledgeV1::try_new(
        grants
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
    RevisionRecord {
        effective_tick: source.tick(),
        atoms: crate::archive::mint_page_atoms(
            source.campaign_id(),
            source.tick(),
            &input,
            &knowledge,
        )
        .unwrap(),
        source,
        subject: subject.page_ref().clone(),
        title: subject.title().to_owned(),
        template_sha256: renderer.template_sha256(),
        content_sha256: page.sha256(),
        markdown: page.markdown().to_owned(),
        search_text: page.search_text().to_owned(),
        provenance_json: serde_json::to_string(page.citations()).unwrap(),
        grants,
        emission,
    }
}

#[test]
fn complete_emission_witness_validates_the_actual_renderer_without_rewriting_retained_bytes() {
    let record = witnessed();
    record.validate().unwrap();
    let mut mismatch = record;
    mismatch.search_text.push('!');
    assert_eq!(
        mismatch.validate(),
        Err(SemanticArchiveErrorV1::StoredPageMismatch)
    );
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
    next.emission = emission;
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
