# Common Gotchas

`tick_commit` marks durability; `MAX(tick)` does not. A failed detached tick
must publish neither graph changes nor receipts. `dynamic_hex_state` is sparse:
read `v_hex_state_asof` through the authenticated observation boundary.

Reference SQLite and Parquet are data preparation artifacts. The production
game writes PostgreSQL through Rust. Retired Python `WorldState`, graph
conversion, and `model_copy` advice is no longer an engine implementation guide.
For retained Pydantic data tools, `model_copy(update=...)` skips validation.

## Postgres Data Volumes Are Lineage-Bound

The container default is `babylon-pg-alpine-c-utf8-v1`. Fresh clusters use
Postgres 17's built-in `C.UTF-8` locale provider and receive an exact lineage
marker only after initialization and every init script complete successfully.
Startup accepts an empty data directory or that exact marked lineage. It rejects
every other nonempty directory before changing ownership, mode, or content.
This is an accidental/recoverability lineage fence, not an adversarial
attestation. A process that can write `PGDATA` can forge both the marker and
cluster metadata. The contract protects operators from accidental physical
reuse across incompatible image/libc lineages; it is not a security boundary.
The injected check is deliberately fail-before-chown. No wrapper or fallback is
allowed to run around the digest-pinned upstream entrypoint.

The retired Debian-lineage `babylon-pg-data` volume remains deliberately outside
the Compose file. Keep it unchanged and never attach it to the Alpine image.
`mise run db:nuke` removes only the current Compose volume. It does not remove
the retired volume or a bind-mounted `BABYLON_PG_DATA` directory.

The repository does not automate physical-volume migration. If an old volume
contains valuable data, keep it unchanged. Plan an offline logical dump/restore
with the original Debian-lineage image (`pg_dump`/`pg_restore` or `pg_dumpall`).
Restore into a fresh current-lineage cluster. Check the restored database before
you retire the old volume. `REINDEX`, collation metadata refresh, and in-place
extension updates do not make cross-libc physical reuse a supported migration
path.

The current census v2 fixtures record observations from the digest-pinned
PostGIS image on Alpine 3.24.1 with PostgreSQL 17.11, PostGIS 3.5.7, H3 and
H3/PostGIS 4.5.0, pgvector 0.8.5, and the built-in `C.UTF-8` locale. The image
contract pins the base and source archive checksums plus final runtime package
revisions; builder packages remain repository-resolved, so the contract makes
no byte-reproducibility claim. The exact lineage marker is
`babylon-postgres-lineage-v1|postgres=17|locale-provider=builtin|locale=C.UTF-8|encoding=UTF8|postgis=3.5.7|h3=4.5.0|h3_postgis=4.5.0|vector=0.8.5`.
The v1 census files remain immutable archives, while v2 is the sole current
adoption and fresh-schema path.

One legacy row changed solely because locale collation reversed the first two
members of the otherwise identical `conservation_audit_log` partition-constraint
array. The archived libc `en_US.utf8` payload orders the hex-hash-length check
before the scale check and hashes to
`30ef0e3b7795606b15a35a2f91bcc40dc60be80f0a000d362bc17c5737ff00e2`.
The built-in `C.UTF-8` payload orders the scale check first and hashes to
`5eff9766641285da9cf078e6633f603a3492f2735b6e5dd6f1c9b1fd07b84b50`.
The checked-in canonical JSON receipts hash to those two fixture digests. The
bounded Rust contract parses both complete payloads, swaps the first two
partition-constraint values, and proves the entire remaining JSON equal. The
DDL is unchanged, and every v2 header records the exact census SQL digest.
