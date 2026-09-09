-- 0040_receipts_vocabulary.sql
-- P25 U12 (ADR139) — L-RECEIPTS: the Third-Worldist ledger's provenance rows.
-- Widens the boundary_flow_register vocabulary CHECKs (0013) for the social
-- wage's supply chain: EXPLOITATION_FLOW (exploited class -> exploiter class,
-- the chain's source), FISCAL_FUNDING (tribute pool -> sovereign fisc, the
-- slice an enactment consumed), SOCIAL_WAGE (sovereign -> class, the
-- delivered unit) — plus the two non-spatial endpoint kinds they need
-- (sovereign, social_class). Append-only discipline (REVOKE, 0013) untouched.
-- Numbered 0040: dev holds 0039_domain_contracts (pg-domain, ADR138) — this
-- lane never mints into a hole another lane owns.
-- Idempotent: DROP IF EXISTS + ADD re-runs clean; widening validates
-- existing rows trivially (old vocabulary is a subset of the new).

ALTER TABLE boundary_flow_register
    DROP CONSTRAINT IF EXISTS boundary_flow_register_source_kind_check;
ALTER TABLE boundary_flow_register
    ADD CONSTRAINT boundary_flow_register_source_kind_check
    CHECK (source_kind IN ('hex', 'county', 'state', 'national', 'external',
                           'sovereign', 'social_class'));

ALTER TABLE boundary_flow_register
    DROP CONSTRAINT IF EXISTS boundary_flow_register_dest_kind_check;
ALTER TABLE boundary_flow_register
    ADD CONSTRAINT boundary_flow_register_dest_kind_check
    CHECK (dest_kind IN ('hex', 'county', 'state', 'national', 'external',
                         'sovereign', 'social_class'));

ALTER TABLE boundary_flow_register
    DROP CONSTRAINT IF EXISTS boundary_flow_register_flow_type_check;
ALTER TABLE boundary_flow_register
    ADD CONSTRAINT boundary_flow_register_flow_type_check
    CHECK (flow_type IN ('trade_edge', 'drain_edge', 'commute_out',
                         'commute_in', 'physical_exchange',
                         'exploitation_flow', 'fiscal_funding',
                         'social_wage'));
