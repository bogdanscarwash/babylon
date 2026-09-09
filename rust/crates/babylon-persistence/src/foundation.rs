//! Database-free capture of one exact replay campaign foundation.

use std::collections::TryReserveError;

use babylon_bsl::rule_pipeline::split_content;
use babylon_bsl::rules_hash_of;
use babylon_bsl::scenario::{load_scenario, load_scenario_with_prelude};
use babylon_graph::hypergraph_store::HypergraphStore;
use babylon_graph::stable_element::StableElementResolverV1;
use babylon_graph::stable_state::encode_stable_graph_state_v1;
use babylon_kernel::replay::{ReplaySeed, ReplaySessionIdV1};
use babylon_kernel::sha256_of;
use babylon_kernel::tick_content_hash::RefDigestV1;
use babylon_kernel::ContentDigest;
use babylon_tick::replay_session::ReplayTickSession;

use crate::runtime::RustPersistenceRuntimeErrorV2;
use crate::semantic_codec;

/// Persisted, closed content encoding selection. It never depends on source size.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum FoundationContentLayout {
    /// Frozen 65,535-byte source fields.
    V1,
    /// Explicit successor with 1 MiB source fields.
    V2,
}

impl FoundationContentLayout {
    /// Return the exact on-disk layout tag.
    #[must_use]
    pub const fn version(self) -> i16 {
        match self {
            Self::V1 => 1,
            Self::V2 => 2,
        }
    }

    pub(crate) fn from_persisted(value: i16) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        match value {
            1 => Ok(Self::V1),
            2 => Ok(Self::V2),
            _ => Err(RustPersistenceRuntimeErrorV2::ReplaySource),
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
struct FoundationContentData {
    scenario_source_bytes: Vec<u8>,
    prelude_source_bytes: Option<Vec<u8>>,
    rule_source_bytes: Vec<u8>,
    defines_bytes: Vec<u8>,
    reference_bundle_manifest_bytes: Vec<u8>,
    content_digest: ContentDigest,
    reference_digest: RefDigestV1,
    canonical_bytes: Vec<u8>,
}

impl FoundationContentData {
    fn try_new(
        layout: FoundationContentLayout,
        scenario_source: &str,
        prelude_source: Option<&str>,
        rule_source: &str,
        defines: &[u8],
        reference_manifest: &[u8],
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        let encode = match layout {
            FoundationContentLayout::V1 => semantic_codec::encode_foundation_content,
            FoundationContentLayout::V2 => semantic_codec::encode_foundation_content_v2,
        };
        let canonical_bytes = encode(
            scenario_source,
            prelude_source,
            rule_source,
            defines,
            reference_manifest,
        )?;
        let (_, rules) =
            split_content(rule_source).map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let rule_forms = rules.into_iter().map(|rule| rule.form).collect::<Vec<_>>();
        let content_digest = ContentDigest {
            defines_hash: sha256_of(defines),
            rules_hash: rules_hash_of(&rule_forms)
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?,
        };
        let reference_digest = RefDigestV1::from_bytes(sha256_of(reference_manifest));
        let scenario_source_bytes = copy_bytes(
            "foundation scenario source bytes",
            scenario_source.as_bytes(),
        )?;
        let prelude_source_bytes = prelude_source
            .map(|source| copy_bytes("foundation prelude source bytes", source.as_bytes()))
            .transpose()?;
        let rule_source_bytes = copy_bytes("foundation rule source bytes", rule_source.as_bytes())?;
        let defines_bytes = copy_bytes("foundation defines bytes", defines)?;
        let reference_bundle_manifest_bytes =
            copy_bytes("foundation reference manifest bytes", reference_manifest)?;
        Ok(Self {
            scenario_source_bytes,
            prelude_source_bytes,
            rule_source_bytes,
            defines_bytes,
            reference_bundle_manifest_bytes,
            content_digest,
            reference_digest,
            canonical_bytes,
        })
    }
}

/// Frozen exact mechanics and reference bytes with V1 source bounds.
#[derive(Debug, PartialEq, Eq)]
pub struct FoundationContentBundleV1(FoundationContentData);

/// Exact mechanics and reference bytes with explicit V2 source bounds.
#[derive(Debug, PartialEq, Eq)]
pub struct FoundationContentBundleV2(FoundationContentData);

/// Closed content representation retained by the unchanged outer foundation.
#[derive(Debug, PartialEq, Eq)]
pub enum FoundationContentBundle {
    /// The frozen V1 encoding, including its original field limits.
    V1(FoundationContentBundleV1),
    /// The explicitly selected V2 encoding.
    V2(FoundationContentBundleV2),
}

macro_rules! impl_bundle_constructor {
    ($ty:ident, $layout:ident) => {
        impl $ty {
            /// Copy and validate exact sources using this type's encoding only.
            /// # Errors
            /// Refuses invalid rule source, NUL, field/aggregate bounds or allocation.
            pub fn try_new(
                scenario_source: &str,
                prelude_source: Option<&str>,
                rule_source: &str,
                defines: &[u8],
                reference_manifest: &[u8],
            ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
                FoundationContentData::try_new(
                    FoundationContentLayout::$layout,
                    scenario_source,
                    prelude_source,
                    rule_source,
                    defines,
                    reference_manifest,
                )
                .map(Self)
            }
            const fn data(&self) -> &FoundationContentData {
                &self.0
            }
        }
    };
}
impl_bundle_constructor!(FoundationContentBundleV1, V1);
impl_bundle_constructor!(FoundationContentBundleV2, V2);

impl FoundationContentBundle {
    /// Return the explicitly selected encoding.
    #[must_use]
    pub const fn layout(&self) -> FoundationContentLayout {
        match self {
            Self::V1(_) => FoundationContentLayout::V1,
            Self::V2(_) => FoundationContentLayout::V2,
        }
    }
    const fn data(&self) -> &FoundationContentData {
        match self {
            Self::V1(bundle) => bundle.data(),
            Self::V2(bundle) => bundle.data(),
        }
    }
    pub(crate) fn try_new(
        layout: FoundationContentLayout,
        scenario_source: &str,
        prelude_source: Option<&str>,
        rule_source: &str,
        defines: &[u8],
        reference_manifest: &[u8],
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        match layout {
            FoundationContentLayout::V1 => FoundationContentBundleV1::try_new(
                scenario_source,
                prelude_source,
                rule_source,
                defines,
                reference_manifest,
            )
            .map(Self::V1),
            FoundationContentLayout::V2 => FoundationContentBundleV2::try_new(
                scenario_source,
                prelude_source,
                rule_source,
                defines,
                reference_manifest,
            )
            .map(Self::V2),
        }
    }
}

macro_rules! impl_bundle_accessors {
    ($ty:ident) => {
        impl $ty {
            /// Borrow the exact scenario source bytes.
            #[must_use]
            pub fn scenario_source_bytes(&self) -> &[u8] {
                &self.data().scenario_source_bytes
            }

            /// Borrow the exact optional prelude source bytes.
            #[must_use]
            pub fn prelude_source_bytes(&self) -> Option<&[u8]> {
                self.data().prelude_source_bytes.as_deref()
            }

            /// Borrow the exact rule source bytes.
            #[must_use]
            pub fn rule_source_bytes(&self) -> &[u8] {
                &self.data().rule_source_bytes
            }

            /// Borrow the exact defines artifact bytes.
            #[must_use]
            pub fn defines_bytes(&self) -> &[u8] {
                &self.data().defines_bytes
            }

            /// Borrow the exact reference-bundle manifest bytes.
            #[must_use]
            pub fn reference_bundle_manifest_bytes(&self) -> &[u8] {
                &self.data().reference_bundle_manifest_bytes
            }

            /// Borrow the exact mechanics identity derived from the retained artifacts.
            #[must_use]
            pub const fn content_digest(&self) -> &ContentDigest {
                &self.data().content_digest
            }

            /// Return the exact retained reference-manifest identity.
            #[must_use]
            pub const fn reference_digest(&self) -> RefDigestV1 {
                self.data().reference_digest
            }

            /// Borrow the canonical tagged content-bundle bytes.
            #[must_use]
            pub fn canonical_bytes(&self) -> &[u8] {
                &self.data().canonical_bytes
            }
        }
    };
}
impl_bundle_accessors!(FoundationContentBundleV1);
impl_bundle_accessors!(FoundationContentBundleV2);
impl_bundle_accessors!(FoundationContentBundle);

/// Exact tick-zero sources from which one replay campaign can be reconstructed.
#[derive(Debug, PartialEq, Eq)]
pub struct CampaignFoundationV1 {
    stable_graph_bytes: Vec<u8>,
    world_register_bytes: Vec<u8>,
    resolver_manifest_bytes: Vec<u8>,
    prepared_environment_bytes: Vec<u8>,
    replay_session_identity: ReplaySessionIdV1,
    rng_seed: ReplaySeed,
    content_digest: ContentDigest,
    reference_digest: RefDigestV1,
    content_bundle: FoundationContentBundle,
    canonical_bytes: Vec<u8>,
}

impl CampaignFoundationV1 {
    /// Capture all exact reconstruction sources from one prepared tick-zero session.
    ///
    /// # Errors
    /// Refuses a session after its first executed tick or the first stable
    /// identity, byte-bound, capacity, integer, or allocation failure. This
    /// operation never parses rules or executes a tick.
    pub fn capture(
        session: &ReplayTickSession<HypergraphStore>,
        content_bundle: FoundationContentBundleV1,
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        Self::capture_content(session, FoundationContentBundle::V1(content_bundle))
    }

    /// Capture a tick-zero foundation with explicitly selected V2 content.
    /// # Errors
    /// Refuses the same graph, identity and aggregate-bound errors as `capture`.
    pub fn capture_v2(
        session: &ReplayTickSession<HypergraphStore>,
        content_bundle: FoundationContentBundleV2,
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        Self::capture_content(session, FoundationContentBundle::V2(content_bundle))
    }

    pub(crate) fn capture_content(
        session: &ReplayTickSession<HypergraphStore>,
        content_bundle: FoundationContentBundle,
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        if session.completed_tick() != 0 {
            return Err(RustPersistenceRuntimeErrorV2::FoundationAfterTickZero {
                actual: session.completed_tick(),
            });
        }
        let stable_graph = session
            .stable_graph_state()
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let world_registers = session
            .world_registers()
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let stable_graph_bytes = copy_bytes(
            "campaign foundation stable graph bytes",
            stable_graph.canonical_bytes(),
        )?;
        let world_register_bytes = copy_bytes(
            "campaign foundation world register bytes",
            world_registers.canonical_bytes(),
        )?;
        let resolver_manifest_bytes = copy_bytes(
            "campaign foundation resolver manifest bytes",
            session.resolver_manifest_bytes(),
        )?;
        let prepared_environment_bytes = copy_bytes(
            "campaign foundation prepared environment bytes",
            session.prepared_environment_bytes(),
        )?;
        let replay_session_identity =
            ReplaySessionIdV1::try_from(session.session_identity().as_bytes())
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let rng_seed = session.rng_seed();
        let content_digest = session.content_digest().clone();
        let reference_digest = session.reference_digest();
        if content_bundle.content_digest() != &content_digest
            || content_bundle.reference_digest() != reference_digest
        {
            return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
        }
        Self::verify_bundle_scenario_reproduces_session_graph(session, &content_bundle)?;
        let replay_session_text = std::str::from_utf8(replay_session_identity.as_bytes())
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let canonical_bytes = semantic_codec::encode_foundation(
            &stable_graph_bytes,
            &world_register_bytes,
            &resolver_manifest_bytes,
            &prepared_environment_bytes,
            replay_session_text,
            i64::from_be_bytes(rng_seed.to_be_bytes()),
            &content_digest.defines_hash,
            &content_digest.rules_hash,
            reference_digest.as_bytes(),
            content_bundle.canonical_bytes(),
        )?;
        Ok(Self {
            stable_graph_bytes,
            world_register_bytes,
            resolver_manifest_bytes,
            prepared_environment_bytes,
            replay_session_identity,
            rng_seed,
            content_digest,
            reference_digest,
            content_bundle,
            canonical_bytes,
        })
    }

    /// Refuse a content bundle whose scenario does not reproduce the
    /// session's captured graph.
    ///
    /// The content digest binds defines + rules only, so a caller could
    /// otherwise pair a session built from scenario A with a bundle carrying
    /// scenario B and persist B's declared county mapping over A's live
    /// graph (a later `open` would then fail rebuilding the stored
    /// foundation). Re-loading the bundle's scenario — with its declaration
    /// prelude, exactly as session hydration does — into a disposable graph
    /// and sealing it with its own authored identities reproduces the
    /// session's stable-graph canonical bytes if and only if the bundle
    /// describes the session's world.
    fn verify_bundle_scenario_reproduces_session_graph(
        session: &ReplayTickSession<HypergraphStore>,
        content_bundle: &FoundationContentBundle,
    ) -> Result<(), RustPersistenceRuntimeErrorV2> {
        let scenario = std::str::from_utf8(content_bundle.scenario_source_bytes())
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let prelude = content_bundle
            .prelude_source_bytes()
            .map(std::str::from_utf8)
            .transpose()
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let mut graph = HypergraphStore::new();
        let loaded = match prelude {
            Some(prelude) => load_scenario_with_prelude(prelude, scenario, &mut graph),
            None => load_scenario(scenario, &mut graph),
        }
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let resolver = StableElementResolverV1::seal(
            &graph,
            &loaded.id,
            &loaded.node_content_ids,
            &loaded.hyperedge_content_ids,
        )
        .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let reloaded = encode_stable_graph_state_v1(&graph, &resolver)
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let captured = session
            .stable_graph_state()
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        if reloaded.canonical_bytes() != captured.canonical_bytes() {
            return Err(RustPersistenceRuntimeErrorV2::FoundationScenarioMismatch);
        }
        Ok(())
    }

    #[allow(
        clippy::too_many_arguments,
        reason = "the durable foundation has nine exact replay sources and five content artifacts"
    )]
    pub(crate) fn from_persisted(
        stable_graph_bytes: Vec<u8>,
        world_register_bytes: Vec<u8>,
        resolver_manifest_bytes: Vec<u8>,
        prepared_environment_bytes: Vec<u8>,
        replay_session_identity: &str,
        rng_seed: i64,
        defines_hash: [u8; 32],
        rules_hash: [u8; 32],
        reference_digest: [u8; 32],
        scenario_source: &str,
        prelude_source: Option<&str>,
        rule_source: &str,
        defines_bytes: &[u8],
        reference_manifest: &[u8],
        expected_foundation_sha256: [u8; 32],
        content_layout: FoundationContentLayout,
    ) -> Result<Self, RustPersistenceRuntimeErrorV2> {
        let content_bundle = FoundationContentBundle::try_new(
            content_layout,
            scenario_source,
            prelude_source,
            rule_source,
            defines_bytes,
            reference_manifest,
        )?;
        let content_digest = ContentDigest {
            defines_hash,
            rules_hash,
        };
        let reference_digest = RefDigestV1::from_bytes(reference_digest);
        if content_bundle.content_digest() != &content_digest
            || content_bundle.reference_digest() != reference_digest
        {
            return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
        }
        let replay_session_identity = ReplaySessionIdV1::try_from(replay_session_identity)
            .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?;
        let canonical_bytes = semantic_codec::encode_foundation(
            &stable_graph_bytes,
            &world_register_bytes,
            &resolver_manifest_bytes,
            &prepared_environment_bytes,
            std::str::from_utf8(replay_session_identity.as_bytes())
                .map_err(|_| RustPersistenceRuntimeErrorV2::ReplaySource)?,
            rng_seed,
            &content_digest.defines_hash,
            &content_digest.rules_hash,
            reference_digest.as_bytes(),
            content_bundle.canonical_bytes(),
        )?;
        if sha256_of(&canonical_bytes) != expected_foundation_sha256 {
            return Err(RustPersistenceRuntimeErrorV2::ReplaySource);
        }
        Ok(Self {
            stable_graph_bytes,
            world_register_bytes,
            resolver_manifest_bytes,
            prepared_environment_bytes,
            replay_session_identity,
            rng_seed: ReplaySeed::new(rng_seed),
            content_digest,
            reference_digest,
            content_bundle,
            canonical_bytes,
        })
    }

    /// Borrow the exact stable graph bytes.
    #[must_use]
    pub fn stable_graph_bytes(&self) -> &[u8] {
        &self.stable_graph_bytes
    }

    /// Borrow the exact tick-zero world-register bytes.
    #[must_use]
    pub fn world_register_bytes(&self) -> &[u8] {
        &self.world_register_bytes
    }

    /// Borrow the exact resolver-manifest bytes.
    #[must_use]
    pub fn resolver_manifest_bytes(&self) -> &[u8] {
        &self.resolver_manifest_bytes
    }

    /// Borrow the exact prepared-environment bytes.
    #[must_use]
    pub fn prepared_environment_bytes(&self) -> &[u8] {
        &self.prepared_environment_bytes
    }

    /// Borrow the exact replay-session namespace.
    #[must_use]
    pub const fn replay_session_identity(&self) -> &ReplaySessionIdV1 {
        &self.replay_session_identity
    }

    /// Return the exact replay seed.
    #[must_use]
    pub const fn rng_seed(&self) -> ReplaySeed {
        self.rng_seed
    }

    /// Borrow the exact mechanics-content identity.
    #[must_use]
    pub const fn content_digest(&self) -> &ContentDigest {
        &self.content_digest
    }

    /// Return the exact reference-data identity.
    #[must_use]
    pub const fn reference_digest(&self) -> RefDigestV1 {
        self.reference_digest
    }

    /// Borrow the exact content bundle.
    #[must_use]
    pub const fn content_bundle(&self) -> &FoundationContentBundle {
        &self.content_bundle
    }

    /// Borrow the canonical complete foundation bytes.
    #[must_use]
    pub fn canonical_bytes(&self) -> &[u8] {
        &self.canonical_bytes
    }
}

fn copy_bytes(
    field: &'static str,
    source: &[u8],
) -> Result<Vec<u8>, RustPersistenceRuntimeErrorV2> {
    let mut bytes = Vec::new();
    bytes
        .try_reserve_exact(source.len())
        .map_err(
            |_: TryReserveError| RustPersistenceRuntimeErrorV2::Allocation {
                field,
                requested: source.len(),
            },
        )?;
    bytes.extend_from_slice(source);
    Ok(bytes)
}
