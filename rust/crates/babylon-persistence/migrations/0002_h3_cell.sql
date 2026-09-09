CREATE TABLE babylon_ref.h3_cell (
    cell_id BIGINT NOT NULL,
    resolution SMALLINT NOT NULL,
    immediate_parent BIGINT,
    ancestor_r4 BIGINT,
    ancestor_r5 BIGINT,
    ancestor_r6 BIGINT,
    ancestor_r7 BIGINT,
    CONSTRAINT h3_cell_pkey PRIMARY KEY (cell_id),
    CONSTRAINT h3_cell_id_positive CHECK (cell_id > 0),
    CONSTRAINT h3_cell_resolution_range CHECK (resolution BETWEEN 0 AND 15),
    CONSTRAINT h3_cell_resolution_matches_id CHECK (
        resolution = ((cell_id >> 52) & 15)::SMALLINT
    ),
    CONSTRAINT h3_cell_immediate_parent_matches CHECK (
        CASE
            WHEN resolution = 0 THEN immediate_parent IS NULL
            WHEN resolution BETWEEN 1 AND 15 THEN
                immediate_parent IS NOT NULL
                AND immediate_parent = (
                    (cell_id & ~(15::BIGINT << 52))
                    | ((resolution - 1)::BIGINT << 52)
                    | ((1::BIGINT << (3 * (16 - resolution))) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r4_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 3 THEN ancestor_r4 IS NULL
            WHEN resolution BETWEEN 4 AND 15 THEN
                ancestor_r4 IS NOT NULL
                AND ancestor_r4 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (4::BIGINT << 52)
                    | ((1::BIGINT << 33) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r5_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 4 THEN ancestor_r5 IS NULL
            WHEN resolution BETWEEN 5 AND 15 THEN
                ancestor_r5 IS NOT NULL
                AND ancestor_r5 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (5::BIGINT << 52)
                    | ((1::BIGINT << 30) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r6_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 5 THEN ancestor_r6 IS NULL
            WHEN resolution BETWEEN 6 AND 15 THEN
                ancestor_r6 IS NOT NULL
                AND ancestor_r6 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (6::BIGINT << 52)
                    | ((1::BIGINT << 27) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_ancestor_r7_matches CHECK (
        CASE
            WHEN resolution BETWEEN 0 AND 6 THEN ancestor_r7 IS NULL
            WHEN resolution BETWEEN 7 AND 15 THEN
                ancestor_r7 IS NOT NULL
                AND ancestor_r7 = (
                    (cell_id & ~(15::BIGINT << 52))
                    | (7::BIGINT << 52)
                    | ((1::BIGINT << 24) - 1)
                )
            ELSE FALSE
        END
    ),
    CONSTRAINT h3_cell_immediate_parent_fkey FOREIGN KEY (immediate_parent)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r4_fkey FOREIGN KEY (ancestor_r4)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r5_fkey FOREIGN KEY (ancestor_r5)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r6_fkey FOREIGN KEY (ancestor_r6)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED,
    CONSTRAINT h3_cell_ancestor_r7_fkey FOREIGN KEY (ancestor_r7)
        REFERENCES babylon_ref.h3_cell(cell_id) DEFERRABLE INITIALLY DEFERRED
);
REVOKE ALL ON TABLE babylon_ref.h3_cell FROM PUBLIC;
