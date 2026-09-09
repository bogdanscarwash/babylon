-- Additive client-owned declared territory-county mapping; not a persistence-authority epoch.
CREATE TABLE babylon_meta.territory_county_map_schema_v1 (
    contract_id TEXT PRIMARY KEY CHECK (contract_id = 'babylon.territory-county-map-schema.v1')
);

CREATE TABLE babylon_meta.territory_county_map_v1 (
    campaign_id UUID NOT NULL,
    territory_local_name TEXT COLLATE pg_catalog."C" NOT NULL CHECK (
        pg_catalog.octet_length(territory_local_name) BETWEEN 1 AND 256
    ),
    county_geoid TEXT COLLATE pg_catalog."C" NOT NULL CHECK (county_geoid ~ '^[0-9]{5}$'),
    PRIMARY KEY (campaign_id, territory_local_name),
    FOREIGN KEY (campaign_id) REFERENCES babylon_meta.campaign(campaign_id) ON DELETE CASCADE
);

INSERT INTO babylon_meta.territory_county_map_schema_v1 (contract_id)
VALUES ('babylon.territory-county-map-schema.v1');

REVOKE ALL ON TABLE babylon_meta.territory_county_map_schema_v1 FROM PUBLIC;
REVOKE ALL ON TABLE babylon_meta.territory_county_map_v1 FROM PUBLIC;
