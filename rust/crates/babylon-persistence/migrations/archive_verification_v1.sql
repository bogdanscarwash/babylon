-- PER-320: verification is a receipt-processing fact, not a page rewrite.
-- A missing consumption caps the contiguous prefix even when later pages
-- have already staged. Only committed markers contribute to either horizon.
CREATE VIEW public.v_archive_verification_v1 AS
SELECT marker.campaign_id,
       MAX(marker.resolve_tick) AS durable_tick,
       COALESCE(
           MIN(marker.resolve_tick) FILTER (WHERE consumed.campaign_id IS NULL) - 1,
           MAX(marker.resolve_tick)
       ) AS processed_tick
FROM babylon_state.tick_commit AS marker
LEFT JOIN babylon_meta.archive_receipt_consumption_v1 AS consumed
  ON consumed.campaign_id = marker.campaign_id
 AND consumed.resolve_tick = marker.resolve_tick
 AND consumed.tick_content_hash = marker.tick_content_hash
GROUP BY marker.campaign_id;

DO $$
BEGIN
    IF EXISTS (SELECT 1 FROM pg_catalog.pg_roles WHERE rolname = 'babylon_reader') THEN
        GRANT SELECT ON public.v_archive_verification_v1 TO babylon_reader;
    END IF;
END
$$;
