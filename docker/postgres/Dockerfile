# syntax=docker/dockerfile:1.6

# Isolated test/runtime Postgres for Babylon (spec-087 FR-002).
#
# Base: supported postgis/postgis 17-3.5 Alpine lineage for both stages. Using
# one digest keeps the compiled extensions on the final image's PostgreSQL,
# musl, and PostGIS ABI. Native source installations and CI use this same
# PostgreSQL 17 runtime through Docker Compose.
#
# PIN AND VERIFICATION CONTRACT: the digest-pinned base, checksum-pinned source
# archives, and exact final-stage runtime package revisions are the declared
# shipped inputs. Builder-only APK packages are repository-resolved inside the
# pinned base and affect compiled output, so this does not claim byte-identical
# image rebuilds. Build-time and health behavioral version checks prove the
# required runtime interface. Changing a declared input is a reviewed change:
# record it and re-run the PostgreSQL integration gates and blocking image scan.
#
# Pinned 2026-08-30: tag 17-3.5-alpine, OCI index digest
# sha256:08f4b1e1f4a571008c60272ceb9e0d1f9f8f643792d006b74a35b1bec44c2218
# (linux/amd64 manifest sha256:35d90a44f150ef26c1c734e86e3f888753ca7fd6e1321d85e73f66a7dbffb531,
# Alpine Linux v3.24.1).
FROM postgis/postgis:17-3.5-alpine@sha256:08f4b1e1f4a571008c60272ceb9e0d1f9f8f643792d006b74a35b1bec44c2218 AS extension-builder

# Official release tags resolve to h3-pg ed9a09c834787abb5952318bccef1c4fee119f1d,
# H3 core v4.5.0, and pgvector 159b79aaad5983fb7459c1e3df2897fbb2d11788.
ADD --checksum=sha256:c54c119e1d9a578d5cbcce22f6c66dab2b5a45219fc2b260619807f7f061e53a \
    https://github.com/postgis/h3-pg/archive/refs/tags/v4.5.0.tar.gz \
    /tmp/h3-pg.tar.gz
ADD --checksum=sha256:0da8a392a6ff77e76b60e6a331a49497d0935b6b7b6899da7a3e2786139b0441 \
    https://github.com/uber/h3/archive/refs/tags/v4.5.0.tar.gz \
    /tmp/h3-core.tar.gz
ADD --checksum=sha256:6f88a5cbdde31666f4b6c1a6b75c51dcbeffe58f9a7d2b26e502d5a6e5e14d44 \
    https://github.com/pgvector/pgvector/archive/refs/tags/v0.8.5.tar.gz \
    /tmp/pgvector.tar.gz

RUN set -eux; \
    apk add --no-cache --virtual .extension-build-deps \
        $DOCKER_PG_LLVM_DEPS \
        build-base \
        cmake; \
    mkdir \
        /tmp/extension-install \
        /tmp/h3-core-source \
        /tmp/h3-pg-source \
        /tmp/pgvector-source; \
    tar --extract --gzip --file=/tmp/h3-core.tar.gz \
        --directory=/tmp/h3-core-source --strip-components=1; \
    tar --extract --gzip --file=/tmp/h3-pg.tar.gz \
        --directory=/tmp/h3-pg-source --strip-components=1; \
    tar --extract --gzip --file=/tmp/pgvector.tar.gz \
        --directory=/tmp/pgvector-source --strip-components=1; \
    cmake -S /tmp/h3-pg-source -B /tmp/h3-pg-build \
        -DBUILD_TESTING=OFF \
        -DCMAKE_BUILD_TYPE=Release \
        -DFETCHCONTENT_SOURCE_DIR_H3=/tmp/h3-core-source \
        -DPOSTGRESQL_VERSION=17; \
    cmake --build /tmp/h3-pg-build --parallel 2; \
    DESTDIR=/tmp/extension-install \
        cmake --install /tmp/h3-pg-build --component h3-pg; \
    make -C /tmp/pgvector-source --jobs=2 OPTFLAGS=""; \
    make -C /tmp/pgvector-source \
        DESTDIR=/tmp/extension-install OPTFLAGS="" install; \
    test -f /tmp/extension-install/usr/local/lib/postgresql/h3.so; \
    test -f /tmp/extension-install/usr/local/lib/postgresql/h3_postgis.so; \
    test -f /tmp/extension-install/usr/local/lib/postgresql/vector.so; \
    test -f \
        /tmp/extension-install/usr/local/share/postgresql/extension/h3--4.5.0.sql; \
    test -f \
        /tmp/extension-install/usr/local/share/postgresql/extension/vector--0.8.5.sql

FROM postgis/postgis:17-3.5-alpine@sha256:08f4b1e1f4a571008c60272ceb9e0d1f9f8f643792d006b74a35b1bec44c2218

COPY --from=extension-builder \
    /tmp/extension-install/usr/local/lib/postgresql/ /usr/local/lib/postgresql/
COPY --from=extension-builder \
    /tmp/extension-install/usr/local/share/postgresql/extension/ /usr/local/share/postgresql/extension/

# The upstream entrypoint must start as root to repair bind/named-volume
# ownership and migrate numeric UIDs. The build-time transformer fails closed
# on the exact upstream shape, injects the recoverability fence before that
# mutation, and replaces the privilege drop directly. The marker is not a
# security attestation. No runtime wrapper, fallback, or global USER directive
# may bypass the upstream initialization.
RUN --mount=type=bind,source=patch-entrypoint.awk,target=/tmp/patch-entrypoint.awk,readonly \
    set -eux; \
    apk add --no-cache su-exec=0.3-r0 \
        giflib=5.2.2-r2 \
        libcrypto3=3.5.8-r0 \
        libssl3=3.5.8-r0; \
    entrypoint=/usr/local/bin/docker-entrypoint.sh; \
    patched_entrypoint="${entrypoint}.babylon"; \
    awk -f /tmp/patch-entrypoint.awk "$entrypoint" > "$patched_entrypoint"; \
    chmod 0755 "$patched_entrypoint"; \
    mv "$patched_entrypoint" "$entrypoint"; \
    expected_gosu='exec gosu postgres "$BASH_SOURCE" "$@"'; \
    expected_su_exec='exec su-exec postgres "$BASH_SOURCE" "$@"'; \
    expected_lineage='babylon-postgres-lineage-v1|postgres=17|locale-provider=builtin|locale=C.UTF-8|encoding=UTF8|postgis=3.5.7|h3=4.5.0|h3_postgis=4.5.0|vector=0.8.5'; \
    ! grep -Fq "$expected_gosu" "$entrypoint"; \
    test "$(awk -v expected="$expected_su_exec" \
        '{ line=$0; sub(/^[[:space:]]*/, "", line); if (line == expected) count++ } \
         END { print count + 0 }' "$entrypoint")" -eq 1; \
    test "$(grep -Fc "$expected_lineage" "$entrypoint")" -eq 1; \
    test "$(grep -Fc -- '--locale-provider=builtin --builtin-locale=C.UTF-8 --encoding=UTF8' "$entrypoint")" -eq 1; \
    test "$(grep -Fc 'mv "$babylon_lineage_tmp" "$babylon_lineage_marker"' "$entrypoint")" -eq 1; \
    ! grep -Fq 'gosu' "$entrypoint"; \
    rm /usr/local/bin/gosu; \
    test ! -e /usr/local/bin/gosu; \
    ! command -v gosu; \
    su-exec nobody true; \
    test "$(cat /etc/alpine-release)" = '3.24.1'; \
    test "$(postgres --version)" = 'postgres (PostgreSQL) 17.11'; \
    grep -Fx "default_version = '3.5.7'" \
        /usr/local/share/postgresql/extension/postgis.control; \
    grep -Fx "default_version = '4.5.0'" \
        /usr/local/share/postgresql/extension/h3.control; \
    grep -Fx "default_version = '4.5.0'" \
        /usr/local/share/postgresql/extension/h3_postgis.control; \
    grep -Fx "default_version = '0.8.5'" \
        /usr/local/share/postgresql/extension/vector.control
