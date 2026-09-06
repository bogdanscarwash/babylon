//! Exact grant dependencies, read in the caller's publication transaction.

use std::collections::BTreeSet;

use postgres::GenericClient;

use super::record::{is_link, parse_page_key, GrantDependency, RevisionRecord};
use super::storage::{signed, unsigned};
use crate::archive::{database, decode, decode_subject_kind};
use crate::{ArchiveAtomValueV1, ArchiveCitationV1, ArchivePageRefV1, SemanticArchiveErrorV1};

pub(super) fn capture(
    client: &mut impl GenericClient,
    record: &RevisionRecord,
) -> Result<Vec<GrantDependency>, SemanticArchiveErrorV1> {
    let mut keys = BTreeSet::new();
    keys.insert((record.subject.clone(), "subject".to_owned()));
    for atom in &record.atoms {
        let subject = if is_link(atom) {
            let ArchiveAtomValueV1::Text(target) = atom.value() else {
                return Err(SemanticArchiveErrorV1::StoredPageMismatch);
            };
            parse_page_key(target)?
        } else {
            record.subject.clone()
        };
        keys.insert((subject, atom.grant_key().to_owned()));
    }
    if keys.len() > 513 {
        return Err(SemanticArchiveErrorV1::CollectionBound);
    }
    let wanted: Vec<_> = keys
        .iter()
        .map(|(subject, key)| format!("{}/{}@{key}", subject.kind().as_str(), subject.id()))
        .collect();
    let campaign = record.source.campaign_id();
    let tick = signed(record.source.tick())?;
    let rows = client
        .query(
            "SELECT g.subject_kind,g.subject_id,g.grant_key,g.granted_tick, \
        g.provenance_source_id,g.provenance_locator FROM babylon_meta.archive_knowledge_grant_v1 g \
        WHERE g.campaign_id=$1 AND g.granted_tick<=$3 \
        AND (g.subject_kind || '/' || g.subject_id || '@' || g.grant_key)=ANY($2::text[]) \
        ORDER BY g.subject_kind,g.subject_id,g.grant_key",
            &[campaign.as_uuid(), &wanted, &tick],
        )
        .map_err(|error| database("capture exact Archive revision grants", &error))?;
    if rows.len() != keys.len() {
        return Err(SemanticArchiveErrorV1::UnknownSubject);
    }
    rows.iter()
        .map(|row| {
            Ok(GrantDependency {
                subject: ArchivePageRefV1::try_new(
                    decode_subject_kind(&decode::<String>(row, 0)?)?,
                    decode(row, 1)?,
                )?,
                key: decode(row, 2)?,
                granted_tick: unsigned(decode(row, 3)?)?,
                citation: ArchiveCitationV1::try_new(decode(row, 4)?, decode(row, 5)?)?,
            })
        })
        .collect()
}
