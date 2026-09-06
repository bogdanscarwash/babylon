//! Explicit durable content-layout admission without changing frozen foundation DDL.

use babylon_kernel::sha256_of;
use postgres::{Client, GenericClient};

use crate::{CampaignId, FoundationContentLayout};

use crate::runtime::RustPersistenceRuntimeErrorV2;

const SCHEMA: &str = include_str!("../migrations/foundation_content_v2.sql");

/// The caller has already established active authority on this exact connection.
/// Historical layout assignment occurs once, under the foundation writer lock.
pub(crate) fn install_foundation_content_schema_v2(
    client: &mut Client,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let mut tx = client.transaction().map_err(|error| storage(&error))?;
    tx.query_one(
        "SELECT pg_catalog.pg_advisory_xact_lock($1)",
        &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
    )
    .map_err(|error| storage(&error))?;
    let row = tx.query_one(
        "SELECT pg_catalog.to_regclass('babylon_meta.foundation_content_schema_v2') IS NOT NULL, \
         pg_catalog.to_regclass('babylon_state.campaign_foundation_content_layout_v2') IS NOT NULL",
        &[],
    ).map_err(|error| storage(&error))?;
    let marker: bool = row.try_get(0).map_err(|error| storage(&error))?;
    let layout: bool = row.try_get(1).map_err(|error| storage(&error))?;
    let digest = sha256_of(SCHEMA.as_bytes());
    match (marker, layout) {
        (false, false) => {
            tx.batch_execute(
                "LOCK TABLE babylon_state.campaign_foundation IN SHARE ROW EXCLUSIVE MODE",
            )
            .map_err(|error| storage(&error))?;
            tx.batch_execute(SCHEMA).map_err(|error| storage(&error))?;
            tx.execute(
                "INSERT INTO babylon_meta.foundation_content_schema_v2 VALUES (true, $1)",
                &[&&digest[..]],
            )
            .map_err(|error| storage(&error))?;
        }
        (true, true) => {
            let stored: Vec<u8> = tx.query_one(
                "SELECT migration_sha256 FROM babylon_meta.foundation_content_schema_v2 WHERE singleton",
                &[],
            ).map_err(|error| storage(&error))?.try_get(0).map_err(|error| storage(&error))?;
            if stored != digest {
                return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
            }
        }
        _ => return Err(RustPersistenceRuntimeErrorV2::ReplaySource),
    }
    // Required layout presence and the closed version are checked for the requested
    // campaign during hydration. Unrelated unadmitted campaigns cannot poison it.
    tx.commit().map_err(|error| storage(&error))
}

/// Hold the required version row until the surrounding commit or reconciliation
/// finishes. FOR SHARE also blocks non-key version edits, unlike KEY SHARE.
pub(crate) fn lock_foundation_content_layout_v2(
    client: &mut impl GenericClient,
    campaign: CampaignId,
    expected: FoundationContentLayout,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let row = client.query_opt(
        "SELECT content_layout_version FROM babylon_state.campaign_foundation_content_layout_v2 \
         WHERE campaign_id = $1::uuid FOR SHARE", &[campaign.as_uuid()],
    ).map_err(|error| RustPersistenceRuntimeErrorV2::postgres("lock foundation content layout", &error))?
        .ok_or(RustPersistenceRuntimeErrorV2::FoundationAbsent)?;
    let version: i16 = row.try_get(0).map_err(|error| {
        RustPersistenceRuntimeErrorV2::postgres("read locked content layout", &error)
    })?;
    if FoundationContentLayout::from_persisted(version)? != expected {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(())
}

fn storage(error: &postgres::Error) -> RustPersistenceRuntimeErrorV2 {
    RustPersistenceRuntimeErrorV2::postgres("install foundation content layout", error)
}
