//! Exact revision row persistence, shared by adoption and live publication.

use postgres::{GenericClient, Row};

use super::record::{GrantDependency, RevisionRecord};
use super::{ArchivePublicationOriginV2, ArchiveReadScopeV2};
use crate::archive::{database, decode, decode_digest, decode_stored_atom, decode_subject_kind};
use crate::{ArchiveCitationV1, ArchivePageRefV1, CampaignId, SemanticArchiveErrorV1};

pub(super) const COLUMNS: &str = "campaign_id, subject_kind, subject_id, effective_tick, origin, \
    source_tick, source_content_hash, template_sha256, content_sha256, revision_sha256, \
    title, markdown, search_text, provenance_json, atom_count, grant_count, emission_json";
const ATOM_COLUMNS: &str =
    "atom.campaign_id, atom.subject_kind, atom.subject_id, atom.signal_key, \
    atom.grant_key, atom.evidence_class, atom.value_kind, atom.value_text, atom.value_f64, \
    atom.value_u64, atom.value_bool, atom.provenance_source_id, atom.provenance_locator, \
    atom.valid_tick, atom.atom_id, membership.position";
const KEY_PREDICATE: &str = "campaign_id=$1 AND subject_kind=$2 AND subject_id=$3 \
    AND effective_tick=$4 AND origin=$5";

#[derive(Clone, Copy)]
pub(super) enum ReadAuthority {
    Writer,
    Confined,
}

impl ReadAuthority {
    fn page(self) -> &'static str {
        match self {
            Self::Writer => "babylon_meta.archive_page_revision_v2",
            Self::Confined => "public.v_archive_revision_known_v2",
        }
    }

    fn grant(self) -> &'static str {
        match self {
            Self::Writer => "babylon_meta.archive_revision_grant_v2",
            Self::Confined => "public.v_archive_revision_grant_v2",
        }
    }
}

/// All identifiers are closed internal constants; no caller supplies SQL.
pub(super) fn load(
    client: &mut impl GenericClient,
    record: &RevisionRecord,
    authority: ReadAuthority,
) -> Result<Option<RevisionRecord>, SemanticArchiveErrorV1> {
    let campaign = record.source.campaign_id();
    let tick = signed(record.effective_tick)?;
    let row = client
        .query_opt(
            &format!(
                "SELECT {COLUMNS} FROM {} WHERE {KEY_PREDICATE}",
                authority.page()
            ),
            &[
                campaign.as_uuid(),
                &record.subject.kind().as_str(),
                &record.subject.id(),
                &tick,
                &record.origin.tag(),
            ],
        )
        .map_err(|error| database("read retained Archive revision", &error))?;
    row.as_ref()
        .map(|row| decode_record(client, row, authority))
        .transpose()
}

pub(super) fn decode_record(
    client: &mut impl GenericClient,
    row: &Row,
    authority: ReadAuthority,
) -> Result<RevisionRecord, SemanticArchiveErrorV1> {
    let campaign = CampaignId::from_uuid(decode(row, 0)?);
    let subject = ArchivePageRefV1::try_new(
        decode_subject_kind(&decode::<String>(row, 1)?)?,
        decode(row, 2)?,
    )?;
    let mut record = RevisionRecord {
        source: ArchiveReadScopeV2::committed(
            campaign,
            unsigned(decode(row, 5)?)?,
            decode_digest(row, 6)?,
        )?,
        subject,
        effective_tick: unsigned(decode(row, 3)?)?,
        origin: ArchivePublicationOriginV2::from_tag(decode(row, 4)?)?,
        template_sha256: decode_digest(row, 7)?,
        content_sha256: decode_digest(row, 8)?,
        title: decode(row, 10)?,
        markdown: decode(row, 11)?,
        search_text: decode(row, 12)?,
        provenance_json: decode(row, 13)?,
        atoms: Vec::new(),
        grants: Vec::new(),
        emission: decode::<Option<String>>(row, 16)?
            .as_deref()
            .map(super::emission::ArchiveEmissionManifestV2::decode)
            .transpose()?,
    };
    let counts = (decode::<i32>(row, 14)?, decode::<i32>(row, 15)?);
    if !(1..=513).contains(&counts.0) || !(1..=513).contains(&counts.1) {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    read_membership(client, &mut record, authority)?;
    if record.atoms.len()
        != usize::try_from(counts.0).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
        || record.grants.len()
            != usize::try_from(counts.1).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
        || record.digest()? != decode_digest(row, 9)?
    {
        return Err(SemanticArchiveErrorV1::StoredPageMismatch);
    }
    Ok(record)
}

fn read_membership(
    client: &mut impl GenericClient,
    record: &mut RevisionRecord,
    authority: ReadAuthority,
) -> Result<(), SemanticArchiveErrorV1> {
    let campaign = record.source.campaign_id();
    let tick = signed(record.effective_tick)?;
    let atom_query = match authority {
        ReadAuthority::Writer => format!("SELECT {ATOM_COLUMNS} FROM babylon_meta.archive_revision_atom_v2 membership \
            JOIN babylon_meta.archive_atom_v1 atom USING(atom_id) WHERE membership.campaign_id=$1 \
            AND membership.subject_kind=$2 AND membership.subject_id=$3 AND membership.effective_tick=$4 \
            AND membership.origin=$5 ORDER BY membership.position LIMIT 514"),
        ReadAuthority::Confined => format!("SELECT {} FROM public.v_archive_revision_atom_v2 atom \
            WHERE {KEY_PREDICATE} ORDER BY position LIMIT 514", ATOM_COLUMNS.replace("membership.position", "atom.position")),
    };
    let params: &[&(dyn postgres::types::ToSql + Sync)] = &[
        campaign.as_uuid(),
        &record.subject.kind().as_str(),
        &record.subject.id(),
        &tick,
        &record.origin.tag(),
    ];
    let atoms = client
        .query(&atom_query, params)
        .map_err(|error| database("read retained Archive membership", &error))?;
    for (position, row) in atoms.iter().enumerate() {
        if decode::<i32>(row, 15)?
            != i32::try_from(position).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        record.atoms.push(decode_stored_atom(row)?);
    }
    let grants = client
        .query(
            &format!(
                "SELECT grant_subject_kind, grant_subject_id, grant_key, \
        granted_tick, provenance_source_id, provenance_locator, position FROM {} \
        WHERE {KEY_PREDICATE} ORDER BY position LIMIT 514",
                authority.grant()
            ),
            params,
        )
        .map_err(|error| database("read retained Archive grant dependencies", &error))?;
    for (position, row) in grants.iter().enumerate() {
        if decode::<i32>(row, 6)?
            != i32::try_from(position).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?
        {
            return Err(SemanticArchiveErrorV1::StoredPageMismatch);
        }
        record.grants.push(GrantDependency {
            subject: ArchivePageRefV1::try_new(
                decode_subject_kind(&decode::<String>(row, 0)?)?,
                decode(row, 1)?,
            )?,
            key: decode(row, 2)?,
            granted_tick: unsigned(decode(row, 3)?)?,
            citation: ArchiveCitationV1::try_new(decode(row, 4)?, decode(row, 5)?)?,
        });
    }
    Ok(())
}

pub(super) fn insert(
    client: &mut impl GenericClient,
    record: &RevisionRecord,
) -> Result<bool, SemanticArchiveErrorV1> {
    let digest = record.digest()?;
    let campaign = record.source.campaign_id();
    let effective = signed(record.effective_tick)?;
    let source = signed(record.source.tick())?;
    let source_hash = record
        .source
        .tick_content_hash()
        .ok_or(SemanticArchiveErrorV1::InvalidVerifiedTick)?;
    let atoms =
        i32::try_from(record.atoms.len()).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?;
    let grants =
        i32::try_from(record.grants.len()).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?;
    let emission = record
        .emission
        .as_ref()
        .map(super::emission::ArchiveEmissionManifestV2::encode)
        .transpose()?;
    let inserted = client
        .execute(
            &format!(
                "INSERT INTO babylon_meta.archive_page_revision_v2 ({COLUMNS}) \
        VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12,$13,$14,$15,$16,$17) \
        ON CONFLICT (campaign_id,subject_kind,subject_id,effective_tick,origin) DO NOTHING"
            ),
            &[
                campaign.as_uuid(),
                &record.subject.kind().as_str(),
                &record.subject.id(),
                &effective,
                &record.origin.tag(),
                &source,
                &&source_hash[..],
                &&record.template_sha256[..],
                &&record.content_sha256[..],
                &&digest[..],
                &record.title,
                &record.markdown,
                &record.search_text,
                &record.provenance_json,
                &atoms,
                &grants,
                &emission,
            ],
        )
        .map_err(|error| database("insert immutable Archive publication", &error))?
        == 1;
    if inserted {
        insert_membership(client, record)?;
    }
    if load(client, record, ReadAuthority::Writer)?.as_ref() != Some(record) {
        return Err(SemanticArchiveErrorV1::ReceiptConflict);
    }
    Ok(inserted)
}

fn insert_membership(
    client: &mut impl GenericClient,
    record: &RevisionRecord,
) -> Result<(), SemanticArchiveErrorV1> {
    let campaign = record.source.campaign_id();
    let effective = signed(record.effective_tick)?;
    for (position, atom) in record.atoms.iter().enumerate() {
        let position =
            i32::try_from(position).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?;
        client
            .execute(
                "INSERT INTO babylon_meta.archive_revision_atom_v2 \
            (campaign_id,subject_kind,subject_id,effective_tick,origin,position,atom_id) \
            VALUES ($1,$2,$3,$4,$5,$6,$7)",
                &[
                    campaign.as_uuid(),
                    &record.subject.kind().as_str(),
                    &record.subject.id(),
                    &effective,
                    &record.origin.tag(),
                    &position,
                    &&atom.atom_id()[..],
                ],
            )
            .map_err(|error| database("insert immutable Archive membership", &error))?;
    }
    for (position, grant) in record.grants.iter().enumerate() {
        let position =
            i32::try_from(position).map_err(|_| SemanticArchiveErrorV1::CollectionBound)?;
        let granted = signed(grant.granted_tick)?;
        client.execute("INSERT INTO babylon_meta.archive_revision_grant_v2 \
            (campaign_id,subject_kind,subject_id,effective_tick,origin,position,grant_subject_kind, \
            grant_subject_id,grant_key,granted_tick,provenance_source_id,provenance_locator) \
            VALUES ($1,$2,$3,$4,$5,$6,$7,$8,$9,$10,$11,$12)",
            &[campaign.as_uuid(), &record.subject.kind().as_str(), &record.subject.id(), &effective,
            &record.origin.tag(), &position, &grant.subject.kind().as_str(), &grant.subject.id(),
            &grant.key, &granted, &grant.citation.source_id(), &grant.citation.locator()],
        ).map_err(|error| database("insert immutable Archive grant dependency", &error))?;
    }
    Ok(())
}

pub(super) fn unsigned(value: i64) -> Result<u64, SemanticArchiveErrorV1> {
    u64::try_from(value).map_err(|_| SemanticArchiveErrorV1::StoredPageMismatch)
}

pub(super) fn signed(value: u64) -> Result<i64, SemanticArchiveErrorV1> {
    i64::try_from(value).map_err(|_| SemanticArchiveErrorV1::InvalidVerifiedTick)
}
