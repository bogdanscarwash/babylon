-- Add the checked canonical H3 shadow surface without moving reader or writer authority.
--
-- The frozen legacy origin contains all 15 public relations below. A truly
-- fresh Rust origin contains none of them, so this migration is deliberately
-- conditional and leaves that lane unchanged. Every shadow remains nullable:
-- the Python writer and legacy text columns stay authoritative until the later
-- one-way cutover.
DO $h3_shadow_keys$
DECLARE
    mapping TEXT[];
BEGIN
    FOREACH mapping SLICE 1 IN ARRAY ARRAY[
        ['dynamic_hex_state', 'cell_id', 'dyn_cell'],
        ['hex_activity', 'cell_id', 'activity_cell'],
        ['hex_cell', 'cell_id', 'hex_cell'],
        ['hex_cell', 'ancestor_r5', 'hex_r5'],
        ['hex_cell', 'ancestor_r6', 'hex_r6'],
        ['hex_latest', 'cell_id', 'latest_cell'],
        ['hex_map', 'cell_id', 'map_cell'],
        ['hex_r8_linear_features_reference', 'cell_id', 'r8_feature_cell'],
        ['hex_r8_reference', 'cell_id', 'r8_cell'],
        ['hex_r8_reference', 'parent_cell_id', 'r8_parent'],
        ['hex_spatial_map', 'cell_id', 'spatial_cell'],
        ['hex_state', 'cell_id', 'state_cell'],
        ['hex_substrate', 'cell_id', 'substrate_cell'],
        ['hex_substrate', 'ancestor_r7', 'substrate_r7'],
        ['hex_terrain_state', 'cell_id', 'terrain_cell'],
        ['immutable_reference_lodes_od_matrix', 'home_cell_id', 'lodes_home'],
        ['immutable_reference_lodes_od_matrix', 'workplace_cell_id', 'lodes_work'],
        ['infrastructure_link_state', 'source_cell_id', 'infra_source'],
        ['infrastructure_link_state', 'target_cell_id', 'infra_target'],
        ['org_snapshot', 'home_cell_id', 'org_home'],
        ['tick_event', 'cell_id', 'event_cell']
    ]
    LOOP
        IF pg_catalog.to_regclass(
            pg_catalog.format('public.%I', mapping[1])
        ) IS NOT NULL THEN
            EXECUTE pg_catalog.format(
                'ALTER TABLE public.%I ADD COLUMN %I BIGINT',
                mapping[1],
                mapping[2]
            );
            EXECUTE pg_catalog.format(
                'ALTER TABLE public.%I ADD CONSTRAINT %I CHECK (%I > 0)',
                mapping[1],
                'ck_h3s_' || mapping[3] || '_pos',
                mapping[2]
            );
            EXECUTE pg_catalog.format(
                'ALTER TABLE public.%I ADD CONSTRAINT %I FOREIGN KEY (%I) '
                'REFERENCES babylon_ref.h3_cell(cell_id) '
                'DEFERRABLE INITIALLY DEFERRED',
                mapping[1],
                'fk_h3s_' || mapping[3],
                mapping[2]
            );
            EXECUTE pg_catalog.format(
                'CREATE INDEX %I ON public.%I (%I) WHERE %I IS NOT NULL',
                'ix_h3s_' || mapping[3],
                mapping[1],
                mapping[2],
                mapping[2]
            );
        END IF;
    END LOOP;
END
$h3_shadow_keys$;
