-- The shared developer image's init script configures its test role for
-- asynchronous acknowledgement. Downloadable previews acknowledge durable WAL.
ALTER ROLE test SET synchronous_commit = on;
