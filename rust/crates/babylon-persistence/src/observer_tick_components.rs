//! Additive full-observer access to exact V3 committed component rows.

use babylon_kernel::sha256_of;
use postgres::{Config, GenericClient, NoTls};

use crate::{
    michigan_economy::digest_hex, observer_reader::ObserverEconomyErrorV1,
    validate_legacy_connection_target,
};

const SCHEMA: &str = include_str!("../migrations/observer_tick_components_v1.sql");

pub(crate) const OBSERVER_TICK_COMPONENT_VIEWS_V1: [&str; 24] = [
    "public.v_observer_graph_node_v1",
    "public.v_observer_graph_node_f64_v1",
    "public.v_observer_graph_edge_v1",
    "public.v_observer_graph_hyperedge_v1",
    "public.v_observer_graph_hyperedge_member_v1",
    "public.v_observer_graph_edge_f64_v1",
    "public.v_observer_graph_node_currency_v1",
    "public.v_observer_graph_hyperedge_f64_v1",
    "public.v_observer_world_register_v1",
    "public.v_observer_hex_state_delta_v1",
    "public.v_observer_territory_state_v1",
    "public.v_observer_territory_state_field_v1",
    "public.v_observer_organization_state_v1",
    "public.v_observer_organization_state_field_v1",
    "public.v_observer_organization_territory_v1",
    "public.v_observer_tick_event_v2",
    "public.v_observer_tick_event_field_v2",
    "public.v_observer_tick_choice_receipt_v1",
    "public.v_observer_tick_choice_receipt_branch_v1",
    "public.v_observer_tick_choice_receipt_carrier_element_v1",
    "public.v_observer_checkpoint_manifest",
    "public.v_observer_checkpoint_section_v1",
    "public.v_observer_archive_dirty_receipt_v1",
    "public.v_observer_tick_action_batch_v1",
];

/// Preserve the independent migration identity and original `PostgreSQL` view
/// definitions. Partial installation and later drift are refusals, never repairs.
pub(crate) fn install_observer_tick_components_schema_v1(
    config: &Config,
) -> Result<(), ObserverEconomyErrorV1> {
    validate_legacy_connection_target(config)
        .map_err(|_| ObserverEconomyErrorV1::ConnectionTarget)?;
    let mut client = config
        .connect(NoTls)
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let mut tx = client
        .transaction()
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    tx.query_one(
        "SELECT pg_catalog.pg_advisory_xact_lock($1)",
        &[&crate::SCHEMA_ADVISORY_LOCK_KEY],
    )
    .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let installed: bool = tx
        .query_one(
            "SELECT pg_catalog.to_regclass('public.observer_tick_components_schema_v1') IS NOT NULL",
            &[],
        )
        .map_err(|_| ObserverEconomyErrorV1::Database)?
        .get(0);
    let digest = digest_hex(&sha256_of(SCHEMA.as_bytes()));
    if installed {
        validate_installed(&mut tx, &digest)?;
    } else {
        install_new(&mut tx, &digest)?;
    }
    tx.commit().map_err(|_| ObserverEconomyErrorV1::Database)
}

fn validate_installed(
    tx: &mut impl GenericClient,
    digest: &str,
) -> Result<(), ObserverEconomyErrorV1> {
    let marker = tx
        .query_one(
            "SELECT migration_sha256, view_definitions FROM public.observer_tick_components_schema_v1 WHERE singleton",
            &[],
        )
        .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
    let stored: String = marker
        .try_get(0)
        .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
    let definitions: Vec<String> = marker
        .try_get(1)
        .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?;
    if stored != digest || definitions != view_definitions(tx)? {
        return Err(ObserverEconomyErrorV1::SchemaDrift);
    }
    Ok(())
}

fn install_new(tx: &mut impl GenericClient, digest: &str) -> Result<(), ObserverEconomyErrorV1> {
    for view in OBSERVER_TICK_COMPONENT_VIEWS_V1 {
        let exists: bool = tx
            .query_one("SELECT pg_catalog.to_regclass($1) IS NOT NULL", &[&view])
            .map_err(|_| ObserverEconomyErrorV1::Database)?
            .get(0);
        if exists {
            return Err(ObserverEconomyErrorV1::SchemaDrift);
        }
    }
    tx.batch_execute(SCHEMA)
        .map_err(|_| ObserverEconomyErrorV1::Database)?;
    tx.batch_execute(
        "CREATE TABLE public.observer_tick_components_schema_v1 (singleton boolean PRIMARY KEY CHECK(singleton), migration_sha256 text NOT NULL, view_definitions text[] NOT NULL); REVOKE ALL ON public.observer_tick_components_schema_v1 FROM PUBLIC",
    )
    .map_err(|_| ObserverEconomyErrorV1::Database)?;
    let definitions = view_definitions(tx)?;
    tx.execute(
        "INSERT INTO public.observer_tick_components_schema_v1 VALUES (true,$1,$2)",
        &[&digest, &definitions],
    )
    .map_err(|_| ObserverEconomyErrorV1::Database)?;
    Ok(())
}

fn view_definitions(tx: &mut impl GenericClient) -> Result<Vec<String>, ObserverEconomyErrorV1> {
    OBSERVER_TICK_COMPONENT_VIEWS_V1
        .iter()
        .map(|view| {
            tx.query_one(
                "SELECT pg_catalog.pg_get_viewdef($1::text::regclass, false)",
                &[view],
            )
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)?
            .try_get(0)
            .map_err(|_| ObserverEconomyErrorV1::SchemaDrift)
        })
        .collect()
}
