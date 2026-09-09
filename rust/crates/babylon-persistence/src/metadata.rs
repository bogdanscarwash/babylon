//! Rust-owned typed access to the retained `babylon_meta` client catalog.

use std::collections::BTreeSet;
use std::time::SystemTime;

use postgres::{Config, GenericClient, NoTls, Row};

use crate::foundation::CampaignFoundationV1;
use crate::identity::CampaignId;
use crate::runtime::RustPersistenceRuntimeErrorV2;

/// Closed retained campaign status.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CampaignCatalogStatusV1 {
    /// The campaign remains available to resume.
    Active,
    /// The player abandoned the campaign.
    Abandoned,
}

/// One typed retained `babylon_meta.campaign` row.
#[derive(Debug, Clone, PartialEq, Eq)]
pub struct CampaignCatalogRowV1 {
    campaign_id: CampaignId,
    slug: String,
    engine_version: String,
    defines_hash: String,
    last_tick: u64,
    status: CampaignCatalogStatusV1,
    last_played_at: Option<SystemTime>,
    created_at: SystemTime,
    rng_seed: Option<i64>,
    content_digest: Option<String>,
}

impl CampaignCatalogRowV1 {
    /// Return the stable campaign identity.
    #[must_use]
    pub const fn campaign_id(&self) -> CampaignId {
        self.campaign_id
    }

    /// Borrow the retained user-facing slug.
    #[must_use]
    pub fn slug(&self) -> &str {
        &self.slug
    }

    /// Borrow the engine version recorded at campaign creation.
    #[must_use]
    pub fn engine_version(&self) -> &str {
        &self.engine_version
    }

    /// Borrow the hexadecimal defines digest.
    #[must_use]
    pub fn defines_hash(&self) -> &str {
        &self.defines_hash
    }

    /// Return the last materially acknowledged tick.
    #[must_use]
    pub const fn last_tick(&self) -> u64 {
        self.last_tick
    }

    /// Return the retained campaign status.
    #[must_use]
    pub const fn status(&self) -> CampaignCatalogStatusV1 {
        self.status
    }

    /// Return when material progress was last acknowledged, when any exists.
    #[must_use]
    pub const fn last_played_at(&self) -> Option<SystemTime> {
        self.last_played_at
    }

    /// Return when the retained campaign row was created.
    #[must_use]
    pub const fn created_at(&self) -> SystemTime {
        self.created_at
    }

    /// Return the replay seed when the retained row has been bound to Rust.
    #[must_use]
    pub const fn rng_seed(&self) -> Option<i64> {
        self.rng_seed
    }

    /// Borrow the canonical content-digest JSON when Rust has bound it.
    #[must_use]
    pub fn content_digest(&self) -> Option<&str> {
        self.content_digest.as_deref()
    }
}

macro_rules! navigation_row {
    ($name:ident) => {
        #[doc = "One typed retained navigation row."]
        #[derive(Debug, Clone, PartialEq, Eq)]
        pub struct $name {
            campaign_id: CampaignId,
            position: u32,
            entity_id: String,
        }

        impl $name {
            /// Construct a retained navigation row.
            ///
            /// # Errors
            /// Refuses an empty or NUL-containing entity identity.
            pub fn try_new(
                campaign_id: CampaignId,
                position: u32,
                entity_id: String,
            ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
                if entity_id.is_empty() || entity_id.as_bytes().contains(&0) {
                    return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
                }
                Ok(Self {
                    campaign_id,
                    position,
                    entity_id,
                })
            }

            /// Return the stable campaign identity.
            #[must_use]
            pub const fn campaign_id(&self) -> CampaignId {
                self.campaign_id
            }

            /// Return the exact retained list position.
            #[must_use]
            pub const fn position(&self) -> u32 {
                self.position
            }

            /// Borrow the retained entity identity.
            #[must_use]
            pub fn entity_id(&self) -> &str {
                &self.entity_id
            }
        }
    };
}

navigation_row!(WatchlistRowV1);
navigation_row!(JumplistRowV1);
navigation_row!(BreadcrumbRowV1);

/// Rust-owned accessor for the retained client metadata tier.
///
/// This store never participates in tick hashing. Navigation replacements are
/// transactional and preserve exact zero-based positions.
#[derive(Clone)]
pub struct RetainedMetadataStoreV1 {
    config: Config,
}

impl RetainedMetadataStoreV1 {
    /// Bind the accessor to one `PostgreSQL` target.
    #[must_use]
    pub fn new(config: &Config) -> Self {
        Self {
            config: config.clone(),
        }
    }

    /// Read one retained campaign row, or report honest absence.
    ///
    /// # Errors
    /// Returns a database or typed-row refusal.
    pub fn campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Option<CampaignCatalogRowV1>, RustPersistenceRuntimeErrorV2> {
        let mut client = self.connect("connect retained campaign reader")?;
        read_campaign_catalog_row_v1(&mut client, campaign_id)
    }

    /// Apply the reversible retained campaign lifecycle state.
    ///
    /// # Errors
    /// Refuses an unknown campaign or database failure.
    pub fn set_campaign_status(
        &self,
        campaign_id: CampaignId,
        status: CampaignCatalogStatusV1,
    ) -> Result<(), RustPersistenceRuntimeErrorV2> {
        let status = match status {
            CampaignCatalogStatusV1::Active => "ACTIVE",
            CampaignCatalogStatusV1::Abandoned => "ABANDONED",
        };
        let mut client = self.connect("connect retained campaign status writer")?;
        let affected = client
            .execute(
                "UPDATE babylon_meta.campaign SET status = $2 WHERE campaign_id = $1::uuid",
                &[campaign_id.as_uuid(), &status],
            )
            .map_err(|error| database("set retained campaign status", &error))?;
        require_one(affected)
    }

    /// Permanently delete one retained campaign and its navigation rows.
    ///
    /// # Errors
    /// Returns a database failure.
    pub fn delete_campaign(
        &self,
        campaign_id: CampaignId,
    ) -> Result<bool, RustPersistenceRuntimeErrorV2> {
        let mut client = self.connect("connect retained campaign deleter")?;
        client
            .execute(
                "DELETE FROM babylon_meta.campaign WHERE campaign_id = $1::uuid",
                &[campaign_id.as_uuid()],
            )
            .map(|affected| affected == 1)
            .map_err(|error| database("delete retained campaign", &error))
    }

    /// Read the exact watchlist order.
    ///
    /// # Errors
    /// Returns a database, position, or entity-identity refusal.
    pub fn watchlist(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<WatchlistRowV1>, RustPersistenceRuntimeErrorV2> {
        let mut client = self.connect("connect retained watchlist reader")?;
        read_navigation_rows(&mut client, campaign_id, NavigationTableV1::Watchlist)
    }

    /// Atomically replace the exact watchlist order.
    ///
    /// # Errors
    /// Refuses an unknown campaign, duplicate entity, invalid identity,
    /// position overflow, or database failure.
    pub fn replace_watchlist(
        &self,
        campaign_id: CampaignId,
        entity_ids: &[String],
    ) -> Result<(), RustPersistenceRuntimeErrorV2> {
        let mut unique = BTreeSet::new();
        if entity_ids.iter().any(|entity_id| !unique.insert(entity_id)) {
            return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
        }
        self.replace_navigation(campaign_id, entity_ids, NavigationTableV1::Watchlist)
    }

    /// Read the exact jumplist order, including legal duplicates.
    ///
    /// # Errors
    /// Returns a database, position, or entity-identity refusal.
    pub fn jumplist(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<JumplistRowV1>, RustPersistenceRuntimeErrorV2> {
        let mut client = self.connect("connect retained jumplist reader")?;
        read_navigation_rows(&mut client, campaign_id, NavigationTableV1::Jumplist)
    }

    /// Atomically replace the exact jumplist order, preserving duplicates.
    ///
    /// # Errors
    /// Refuses an unknown campaign, invalid identity, position overflow, or
    /// database failure.
    pub fn replace_jumplist(
        &self,
        campaign_id: CampaignId,
        entity_ids: &[String],
    ) -> Result<(), RustPersistenceRuntimeErrorV2> {
        self.replace_navigation(campaign_id, entity_ids, NavigationTableV1::Jumplist)
    }

    /// Read the exact breadcrumb order.
    ///
    /// # Errors
    /// Returns a database, position, or entity-identity refusal.
    pub fn breadcrumbs(
        &self,
        campaign_id: CampaignId,
    ) -> Result<Vec<BreadcrumbRowV1>, RustPersistenceRuntimeErrorV2> {
        let mut client = self.connect("connect retained breadcrumb reader")?;
        read_navigation_rows(&mut client, campaign_id, NavigationTableV1::Breadcrumb)
    }

    /// Atomically replace the exact breadcrumb order.
    ///
    /// # Errors
    /// Refuses an unknown campaign, invalid identity, position overflow, or
    /// database failure.
    pub fn replace_breadcrumbs(
        &self,
        campaign_id: CampaignId,
        entity_ids: &[String],
    ) -> Result<(), RustPersistenceRuntimeErrorV2> {
        self.replace_navigation(campaign_id, entity_ids, NavigationTableV1::Breadcrumb)
    }

    fn connect(
        &self,
        operation: &'static str,
    ) -> Result<postgres::Client, RustPersistenceRuntimeErrorV2> {
        self.config
            .connect(NoTls)
            .map_err(|error| RustPersistenceRuntimeErrorV2::postgres(operation, &error))
    }

    fn replace_navigation(
        &self,
        campaign_id: CampaignId,
        entity_ids: &[String],
        table: NavigationTableV1,
    ) -> Result<(), RustPersistenceRuntimeErrorV2> {
        let rows = validate_navigation_input(campaign_id, entity_ids, table)?;
        let mut client = self.connect(table.connect_write_operation())?;
        let mut transaction = client
            .transaction()
            .map_err(|error| database(table.begin_operation(), &error))?;
        if transaction
            .query_opt(
                "SELECT 1 FROM babylon_meta.campaign WHERE campaign_id = $1::uuid FOR KEY SHARE",
                &[campaign_id.as_uuid()],
            )
            .map_err(|error| database("lock retained campaign for navigation replacement", &error))?
            .is_none()
        {
            return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
        }
        transaction
            .execute(table.delete_sql(), &[campaign_id.as_uuid()])
            .map_err(|error| database(table.delete_operation(), &error))?;
        for row in rows {
            let position = i32::try_from(row.position)
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
            transaction
                .execute(
                    table.insert_sql(),
                    &[campaign_id.as_uuid(), &position, &row.entity_id],
                )
                .map_err(|error| database(table.insert_operation(), &error))?;
        }
        transaction
            .commit()
            .map_err(|error| database(table.commit_operation(), &error))
    }
}

#[derive(Clone, Copy)]
enum NavigationTableV1 {
    Watchlist,
    Jumplist,
    Breadcrumb,
}

impl NavigationTableV1 {
    const fn select_sql(self) -> &'static str {
        match self {
            Self::Watchlist => {
                "SELECT position, entity_id FROM babylon_meta.watchlist \
                 WHERE campaign_id = $1::uuid ORDER BY position"
            }
            Self::Jumplist => {
                "SELECT position, entity_id FROM babylon_meta.jumplist \
                 WHERE campaign_id = $1::uuid ORDER BY position"
            }
            Self::Breadcrumb => {
                "SELECT position, entity_id FROM babylon_meta.breadcrumb \
                 WHERE campaign_id = $1::uuid ORDER BY position"
            }
        }
    }

    const fn delete_sql(self) -> &'static str {
        match self {
            Self::Watchlist => "DELETE FROM babylon_meta.watchlist WHERE campaign_id = $1::uuid",
            Self::Jumplist => "DELETE FROM babylon_meta.jumplist WHERE campaign_id = $1::uuid",
            Self::Breadcrumb => "DELETE FROM babylon_meta.breadcrumb WHERE campaign_id = $1::uuid",
        }
    }

    const fn insert_sql(self) -> &'static str {
        match self {
            Self::Watchlist => {
                "INSERT INTO babylon_meta.watchlist (campaign_id, position, entity_id) \
                 VALUES ($1::uuid, $2, $3)"
            }
            Self::Jumplist => {
                "INSERT INTO babylon_meta.jumplist (campaign_id, position, entity_id) \
                 VALUES ($1::uuid, $2, $3)"
            }
            Self::Breadcrumb => {
                "INSERT INTO babylon_meta.breadcrumb (campaign_id, position, entity_id) \
                 VALUES ($1::uuid, $2, $3)"
            }
        }
    }

    const fn connect_write_operation(self) -> &'static str {
        match self {
            Self::Watchlist => "connect retained watchlist writer",
            Self::Jumplist => "connect retained jumplist writer",
            Self::Breadcrumb => "connect retained breadcrumb writer",
        }
    }

    const fn read_operation(self) -> &'static str {
        match self {
            Self::Watchlist => "read retained watchlist",
            Self::Jumplist => "read retained jumplist",
            Self::Breadcrumb => "read retained breadcrumb",
        }
    }

    const fn begin_operation(self) -> &'static str {
        match self {
            Self::Watchlist => "begin retained watchlist replacement",
            Self::Jumplist => "begin retained jumplist replacement",
            Self::Breadcrumb => "begin retained breadcrumb replacement",
        }
    }

    const fn delete_operation(self) -> &'static str {
        match self {
            Self::Watchlist => "clear retained watchlist",
            Self::Jumplist => "clear retained jumplist",
            Self::Breadcrumb => "clear retained breadcrumb",
        }
    }

    const fn insert_operation(self) -> &'static str {
        match self {
            Self::Watchlist => "insert retained watchlist row",
            Self::Jumplist => "insert retained jumplist row",
            Self::Breadcrumb => "insert retained breadcrumb row",
        }
    }

    const fn commit_operation(self) -> &'static str {
        match self {
            Self::Watchlist => "commit retained watchlist replacement",
            Self::Jumplist => "commit retained jumplist replacement",
            Self::Breadcrumb => "commit retained breadcrumb replacement",
        }
    }
}

struct NavigationInputRowV1<'a> {
    position: u32,
    entity_id: &'a str,
}

fn validate_navigation_input(
    campaign_id: CampaignId,
    entity_ids: &[String],
    table: NavigationTableV1,
) -> Result<Vec<NavigationInputRowV1<'_>>, RustPersistenceRuntimeErrorV2> {
    let mut rows = Vec::new();
    rows.try_reserve_exact(entity_ids.len()).map_err(|_| {
        RustPersistenceRuntimeErrorV2::Allocation {
            field: "retained navigation rows",
            requested: entity_ids.len(),
        }
    })?;
    for (position, entity_id) in entity_ids.iter().enumerate() {
        let position =
            u32::try_from(position).map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        match table {
            NavigationTableV1::Watchlist => {
                WatchlistRowV1::try_new(campaign_id, position, entity_id.clone())?;
            }
            NavigationTableV1::Jumplist => {
                JumplistRowV1::try_new(campaign_id, position, entity_id.clone())?;
            }
            NavigationTableV1::Breadcrumb => {
                BreadcrumbRowV1::try_new(campaign_id, position, entity_id.clone())?;
            }
        }
        rows.push(NavigationInputRowV1 {
            position,
            entity_id,
        });
    }
    Ok(rows)
}

trait NavigationRowV1: Sized {
    fn try_from_parts(
        campaign_id: CampaignId,
        position: u32,
        entity_id: String,
    ) -> Result<Self, RustPersistenceRuntimeErrorV2>;
}

macro_rules! navigation_row_impl {
    ($name:ident) => {
        impl NavigationRowV1 for $name {
            fn try_from_parts(
                campaign_id: CampaignId,
                position: u32,
                entity_id: String,
            ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
                Self::try_new(campaign_id, position, entity_id)
            }
        }
    };
}

navigation_row_impl!(WatchlistRowV1);
navigation_row_impl!(JumplistRowV1);
navigation_row_impl!(BreadcrumbRowV1);

fn read_navigation_rows<RowType: NavigationRowV1>(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    table: NavigationTableV1,
) -> Result<Vec<RowType>, RustPersistenceRuntimeErrorV2> {
    client
        .query(table.select_sql(), &[campaign_id.as_uuid()])
        .map_err(|error| database(table.read_operation(), &error))?
        .iter()
        .enumerate()
        .map(|(expected, row)| {
            let position: i32 = row
                .try_get(0)
                .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
            if usize::try_from(position).ok() != Some(expected) {
                return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
            }
            RowType::try_from_parts(
                campaign_id,
                u32::try_from(position)
                    .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
                row.try_get(1)
                    .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
            )
        })
        .collect()
}

pub(crate) fn ensure_campaign_catalog_row_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    foundation: &CampaignFoundationV1,
) -> Result<CampaignCatalogRowV1, RustPersistenceRuntimeErrorV2> {
    let defines_hash = hex_digest(&foundation.content_digest().defines_hash)?;
    let rules_hash = hex_digest(&foundation.content_digest().rules_hash)?;
    let content_digest = content_digest_json(&defines_hash, &rules_hash)?;
    let slug = format!("campaign-{}", campaign_id.as_uuid());
    let seed = i64::from_be_bytes(foundation.rng_seed().to_be_bytes());
    client
        .execute(
            "INSERT INTO babylon_meta.campaign \
             (campaign_id, slug, engine_version, defines_hash, last_tick, status, rng_seed, content_digest) \
             VALUES ($1::uuid, $2, $3, $4, 0, 'ACTIVE', $5, $6) \
             ON CONFLICT (campaign_id) DO UPDATE SET \
               rng_seed = COALESCE(babylon_meta.campaign.rng_seed, EXCLUDED.rng_seed), \
               content_digest = COALESCE(babylon_meta.campaign.content_digest, EXCLUDED.content_digest)",
            &[
                campaign_id.as_uuid(),
                &slug,
                &env!("CARGO_PKG_VERSION"),
                &defines_hash,
                &seed,
                &content_digest,
            ],
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("bind retained campaign catalog", &error)
        })?;
    let row = read_campaign_catalog_row_v1(client, campaign_id)?
        .ok_or(RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    if row.defines_hash() != defines_hash
        || row.last_tick() != 0
        || row.rng_seed() != Some(seed)
        || row.content_digest() != Some(content_digest.as_str())
    {
        return Err(RustPersistenceRuntimeErrorV2::CampaignConflict);
    }
    Ok(row)
}

pub(crate) fn advance_campaign_catalog_tick_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
    predecessor: i64,
    resolve_tick: i64,
) -> Result<(), RustPersistenceRuntimeErrorV2> {
    let affected = client
        .execute(
            "UPDATE babylon_meta.campaign SET last_tick = $2, last_played_at = pg_catalog.clock_timestamp() \
             WHERE campaign_id = $1::uuid AND last_tick = $3",
            &[campaign_id.as_uuid(), &resolve_tick, &predecessor],
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("advance retained campaign catalog", &error)
        })?;
    if affected == 1 {
        Ok(())
    } else {
        Err(RustPersistenceRuntimeErrorV2::CampaignConflict)
    }
}

pub(crate) fn read_campaign_catalog_row_v1(
    client: &mut impl GenericClient,
    campaign_id: CampaignId,
) -> Result<Option<CampaignCatalogRowV1>, RustPersistenceRuntimeErrorV2> {
    client
        .query_opt(
            "SELECT slug, engine_version, defines_hash, last_tick, status, last_played_at, \
                    created_at, rng_seed, content_digest \
             FROM babylon_meta.campaign WHERE campaign_id = $1::uuid",
            &[campaign_id.as_uuid()],
        )
        .map_err(|error| {
            RustPersistenceRuntimeErrorV2::postgres("read retained campaign catalog", &error)
        })?
        .map(|row| decode_campaign_catalog_row(campaign_id, &row))
        .transpose()
}

fn decode_campaign_catalog_row(
    campaign_id: CampaignId,
    row: &Row,
) -> Result<CampaignCatalogRowV1, RustPersistenceRuntimeErrorV2> {
    let last_tick: i64 = row
        .try_get(3)
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    let status: String = row
        .try_get(4)
        .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?;
    Ok(CampaignCatalogRowV1 {
        campaign_id,
        slug: row
            .try_get(0)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        engine_version: row
            .try_get(1)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        defines_hash: row
            .try_get(2)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        last_tick: u64::try_from(last_tick)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        status: match status.as_str() {
            "ACTIVE" => CampaignCatalogStatusV1::Active,
            "ABANDONED" => CampaignCatalogStatusV1::Abandoned,
            _ => return Err(RustPersistenceRuntimeErrorV2::CampaignConflict),
        },
        last_played_at: row
            .try_get(5)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        created_at: row
            .try_get(6)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        rng_seed: row
            .try_get(7)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
        content_digest: row
            .try_get(8)
            .map_err(|_| RustPersistenceRuntimeErrorV2::CampaignConflict)?,
    })
}

fn require_one(affected: u64) -> Result<(), RustPersistenceRuntimeErrorV2> {
    if affected == 1 {
        Ok(())
    } else {
        Err(RustPersistenceRuntimeErrorV2::CampaignConflict)
    }
}

fn database(operation: &'static str, error: &postgres::Error) -> RustPersistenceRuntimeErrorV2 {
    RustPersistenceRuntimeErrorV2::postgres(operation, error)
}

fn hex_digest(bytes: &[u8; 32]) -> Result<String, RustPersistenceRuntimeErrorV2> {
    const HEX: &[u8; 16] = b"0123456789abcdef";
    let mut output = String::new();
    output
        .try_reserve_exact(64)
        .map_err(|_| RustPersistenceRuntimeErrorV2::Allocation {
            field: "campaign catalog digest",
            requested: 64,
        })?;
    for byte in bytes {
        output.push(char::from(HEX[usize::from(byte >> 4)]));
        output.push(char::from(HEX[usize::from(byte & 0x0f)]));
    }
    Ok(output)
}

fn content_digest_json(
    defines_hash: &str,
    rules_hash: &str,
) -> Result<String, RustPersistenceRuntimeErrorV2> {
    let capacity = 164;
    let mut output = String::new();
    output
        .try_reserve_exact(capacity)
        .map_err(|_| RustPersistenceRuntimeErrorV2::Allocation {
            field: "campaign catalog content digest",
            requested: capacity,
        })?;
    output.push_str("{\"defines_hash\":\"");
    output.push_str(defines_hash);
    output.push_str("\",\"rules_hash\":\"");
    output.push_str(rules_hash);
    output.push_str("\"}");
    Ok(output)
}
