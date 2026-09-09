CREATE UNIQUE INDEX h3_reference_membership_origin_key
    ON babylon_ref.h3_reference_membership (ref_digest, cell_id, origin);

CREATE TABLE babylon_ref.reference_product (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    artifact_sha256 BYTEA NOT NULL,
    semantic_sha256 BYTEA,
    row_count BIGINT NOT NULL,
    evidence_class TEXT COLLATE "C" NOT NULL,
    measure_unit TEXT COLLATE "C",
    denominator TEXT COLLATE "C",
    CONSTRAINT reference_product_pkey PRIMARY KEY (ref_digest, product_code),
    CONSTRAINT reference_product_cohort_fkey FOREIGN KEY (ref_digest)
        REFERENCES babylon_ref.h3_reference_cohort(ref_digest) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT reference_product_ref_digest_length CHECK (octet_length(ref_digest) = 32),
    CONSTRAINT reference_product_code_format CHECK (product_code ~ '^[a-z0-9_]+$'),
    CONSTRAINT reference_product_artifact_digest_length CHECK (octet_length(artifact_sha256) = 32),
    CONSTRAINT reference_product_semantic_digest_length
        CHECK (semantic_sha256 IS NULL OR octet_length(semantic_sha256) = 32),
    CONSTRAINT reference_product_row_count_positive CHECK (row_count > 0),
    CONSTRAINT reference_product_evidence_class
        CHECK (evidence_class IN ('Observed', 'Derived')),
    CONSTRAINT reference_product_measure_unit
        CHECK (measure_unit IS NULL OR measure_unit IN ('identity', 'parts_per_million', 'count', 'square_metres')),
    CONSTRAINT reference_product_denominator
        CHECK (denominator IS NULL OR denominator IN ('one_million', 'cell_michigan_land_area_m2'))
);

CREATE TABLE babylon_ref.county_identity (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    county_id BIGINT NOT NULL,
    county_geoid TEXT COLLATE "C" NOT NULL,
    state_id INTEGER NOT NULL,
    county_fips TEXT COLLATE "C" NOT NULL,
    county_name TEXT COLLATE "C" NOT NULL,
    CONSTRAINT county_identity_pkey PRIMARY KEY (ref_digest, county_geoid),
    CONSTRAINT county_identity_county_id_key UNIQUE (ref_digest, county_id),
    CONSTRAINT county_identity_product_fkey FOREIGN KEY (ref_digest, product_code)
        REFERENCES babylon_ref.reference_product(ref_digest, product_code)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_identity_product_code CHECK (product_code = 'dim_county'),
    CONSTRAINT county_identity_county_id_positive CHECK (county_id > 0),
    CONSTRAINT county_identity_geoid_format CHECK (county_geoid ~ '^[0-9]{5}$'),
    CONSTRAINT county_identity_state_id_positive CHECK (state_id > 0),
    CONSTRAINT county_identity_fips_format CHECK (county_fips ~ '^[0-9]{3}$'),
    CONSTRAINT county_identity_geoid_suffix CHECK (right(county_geoid, 3) = county_fips),
    CONSTRAINT county_identity_name_nonempty CHECK (county_name <> '')
);

CREATE TABLE babylon_ref.place_identity (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    place_geoid TEXT COLLATE "C" NOT NULL,
    state_fips TEXT COLLATE "C" NOT NULL,
    place_fips TEXT COLLATE "C" NOT NULL,
    place_ns TEXT COLLATE "C" NOT NULL,
    name TEXT COLLATE "C" NOT NULL,
    name_lsad TEXT COLLATE "C" NOT NULL,
    lsad TEXT COLLATE "C" NOT NULL,
    class_fp TEXT COLLATE "C" NOT NULL,
    principal_city_indicator TEXT COLLATE "C" NOT NULL,
    mtfcc TEXT COLLATE "C" NOT NULL,
    functional_status TEXT COLLATE "C" NOT NULL,
    CONSTRAINT place_identity_pkey PRIMARY KEY (ref_digest, place_geoid),
    CONSTRAINT place_identity_product_fkey FOREIGN KEY (ref_digest, product_code)
        REFERENCES babylon_ref.reference_product(ref_digest, product_code)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT place_identity_product_code CHECK (product_code = 'census_place_identity_mi_2023'),
    CONSTRAINT place_identity_geoid_format CHECK (place_geoid ~ '^[0-9]{7}$'),
    CONSTRAINT place_identity_state CHECK (state_fips = '26'),
    CONSTRAINT place_identity_place_fips_format CHECK (place_fips ~ '^[0-9]{5}$'),
    CONSTRAINT place_identity_geoid_composition CHECK (place_geoid = state_fips || place_fips),
    CONSTRAINT place_identity_ns_format CHECK (place_ns ~ '^[0-9]{8}$'),
    CONSTRAINT place_identity_name_nonempty CHECK (name <> ''),
    CONSTRAINT place_identity_name_lsad_nonempty CHECK (name_lsad <> ''),
    CONSTRAINT place_identity_lsad_format CHECK (lsad ~ '^[0-9]{2}$'),
    CONSTRAINT place_identity_class_format CHECK (class_fp ~ '^[A-Z0-9]{2}$'),
    CONSTRAINT place_identity_principal_city CHECK (principal_city_indicator IN ('N', 'Y')),
    CONSTRAINT place_identity_mtfcc_format CHECK (mtfcc ~ '^[A-Z][0-9]{4}$'),
    CONSTRAINT place_identity_status_format CHECK (functional_status ~ '^[A-Z]$')
);

CREATE TABLE babylon_ref.h3_land_fraction (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    cell_id BIGINT NOT NULL,
    membership_origin SMALLINT NOT NULL,
    source_county_geoid TEXT COLLATE "C" NOT NULL,
    land_fraction_ppm INTEGER NOT NULL,
    CONSTRAINT h3_land_fraction_pkey PRIMARY KEY (ref_digest, cell_id),
    CONSTRAINT h3_land_fraction_membership_fkey
        FOREIGN KEY (ref_digest, cell_id, membership_origin)
        REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_land_fraction_product_fkey FOREIGN KEY (ref_digest, product_code)
        REFERENCES babylon_ref.reference_product(ref_digest, product_code)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_land_fraction_county_fkey FOREIGN KEY (ref_digest, source_county_geoid)
        REFERENCES babylon_ref.county_identity(ref_digest, county_geoid)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_land_fraction_product_code CHECK (product_code = 'h3_res7_land_mask'),
    CONSTRAINT h3_land_fraction_cell_positive CHECK (cell_id > 0),
    CONSTRAINT h3_land_fraction_direct_origin CHECK (membership_origin = 1),
    CONSTRAINT h3_land_fraction_range CHECK (land_fraction_ppm BETWEEN 0 AND 1000000)
);

CREATE TABLE babylon_ref.h3_population_count (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    cell_id BIGINT NOT NULL,
    membership_origin SMALLINT NOT NULL,
    population_count BIGINT NOT NULL,
    CONSTRAINT h3_population_count_pkey PRIMARY KEY (ref_digest, cell_id),
    CONSTRAINT h3_population_count_membership_fkey
        FOREIGN KEY (ref_digest, cell_id, membership_origin)
        REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_population_count_product_fkey FOREIGN KEY (ref_digest, product_code)
        REFERENCES babylon_ref.reference_product(ref_digest, product_code)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_population_count_product_code CHECK (product_code = 'h3_res7_population'),
    CONSTRAINT h3_population_count_cell_positive CHECK (cell_id > 0),
    CONSTRAINT h3_population_count_direct_origin CHECK (membership_origin = 1),
    CONSTRAINT h3_population_count_positive CHECK (population_count > 0)
);

CREATE TABLE babylon_ref.h3_workplace_count (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    cell_id BIGINT NOT NULL,
    membership_origin SMALLINT NOT NULL,
    workplace_count BIGINT NOT NULL,
    CONSTRAINT h3_workplace_count_pkey PRIMARY KEY (ref_digest, cell_id),
    CONSTRAINT h3_workplace_count_membership_fkey
        FOREIGN KEY (ref_digest, cell_id, membership_origin)
        REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_workplace_count_product_fkey FOREIGN KEY (ref_digest, product_code)
        REFERENCES babylon_ref.reference_product(ref_digest, product_code)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_workplace_count_product_code CHECK (product_code = 'h3_res7_workplace'),
    CONSTRAINT h3_workplace_count_cell_positive CHECK (cell_id > 0),
    CONSTRAINT h3_workplace_count_direct_origin CHECK (membership_origin = 1),
    CONSTRAINT h3_workplace_count_positive CHECK (workplace_count > 0)
);

CREATE TABLE babylon_ref.county_h3_land_area (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    cell_id BIGINT NOT NULL,
    membership_origin SMALLINT NOT NULL,
    county_geoid TEXT COLLATE "C" NOT NULL,
    land_area_m2 BIGINT NOT NULL,
    CONSTRAINT county_h3_land_area_pkey PRIMARY KEY (ref_digest, cell_id, county_geoid),
    CONSTRAINT county_h3_land_area_membership_fkey
        FOREIGN KEY (ref_digest, cell_id, membership_origin)
        REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_h3_land_area_product_fkey FOREIGN KEY (ref_digest, product_code)
        REFERENCES babylon_ref.reference_product(ref_digest, product_code)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_h3_land_area_county_fkey FOREIGN KEY (ref_digest, county_geoid)
        REFERENCES babylon_ref.county_identity(ref_digest, county_geoid)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_h3_land_area_product_code
        CHECK (product_code = 'census_county_h3_land_overlap_mi_2023'),
    CONSTRAINT county_h3_land_area_cell_positive CHECK (cell_id > 0),
    CONSTRAINT county_h3_land_area_direct_origin CHECK (membership_origin = 1),
    CONSTRAINT county_h3_land_area_positive CHECK (land_area_m2 > 0)
);

CREATE TABLE babylon_ref.county_place_h3_land_area (
    ref_digest BYTEA NOT NULL,
    product_code TEXT COLLATE "C" NOT NULL,
    cell_id BIGINT NOT NULL,
    membership_origin SMALLINT NOT NULL,
    county_geoid TEXT COLLATE "C" NOT NULL,
    place_geoid TEXT COLLATE "C" NOT NULL,
    place_land_area_m2 BIGINT NOT NULL,
    cell_mi_land_area_m2 BIGINT NOT NULL,
    place_land_area_share_ppb INTEGER NOT NULL,
    CONSTRAINT county_place_h3_land_area_pkey
        PRIMARY KEY (ref_digest, cell_id, county_geoid, place_geoid),
    CONSTRAINT county_place_h3_land_area_membership_fkey
        FOREIGN KEY (ref_digest, cell_id, membership_origin)
        REFERENCES babylon_ref.h3_reference_membership(ref_digest, cell_id, origin)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_place_h3_land_area_product_fkey FOREIGN KEY (ref_digest, product_code)
        REFERENCES babylon_ref.reference_product(ref_digest, product_code)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_place_h3_land_area_county_cell_fkey
        FOREIGN KEY (ref_digest, cell_id, county_geoid)
        REFERENCES babylon_ref.county_h3_land_area(ref_digest, cell_id, county_geoid)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_place_h3_land_area_place_fkey FOREIGN KEY (ref_digest, place_geoid)
        REFERENCES babylon_ref.place_identity(ref_digest, place_geoid)
        DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT county_place_h3_land_area_product_code
        CHECK (product_code = 'census_county_place_h3_land_overlap_mi_2023'),
    CONSTRAINT county_place_h3_land_area_cell_positive CHECK (cell_id > 0),
    CONSTRAINT county_place_h3_land_area_direct_origin CHECK (membership_origin = 1),
    CONSTRAINT county_place_h3_land_area_place_positive CHECK (place_land_area_m2 > 0),
    CONSTRAINT county_place_h3_land_area_denominator_positive CHECK (cell_mi_land_area_m2 > 0),
    CONSTRAINT county_place_h3_land_area_numerator_bound
        CHECK (place_land_area_m2 <= cell_mi_land_area_m2),
    CONSTRAINT county_place_h3_land_area_share_range
        CHECK (place_land_area_share_ppb BETWEEN 0 AND 1000000000),
    CONSTRAINT county_place_h3_land_area_share_formula CHECK (
        place_land_area_share_ppb =
            floor(place_land_area_m2::numeric * 1000000000 / cell_mi_land_area_m2)::bigint
    )
);

CREATE INDEX county_h3_land_area_county_idx
    ON babylon_ref.county_h3_land_area (ref_digest, county_geoid, cell_id);
CREATE INDEX county_place_h3_land_area_place_idx
    ON babylon_ref.county_place_h3_land_area (ref_digest, place_geoid, cell_id, county_geoid);

REVOKE ALL ON TABLE babylon_ref.reference_product FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.county_identity FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.place_identity FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.h3_land_fraction FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.h3_population_count FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.h3_workplace_count FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.county_h3_land_area FROM PUBLIC;
REVOKE ALL ON TABLE babylon_ref.county_place_h3_land_area FROM PUBLIC;
