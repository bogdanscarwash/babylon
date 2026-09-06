#!/usr/bin/env bash
# Own one disposable PostgreSQL runtime for the destructive PER-20 Rust proof.
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
readonly REPO_ROOT
readonly IMAGE="babylon-per20-legacy-adopter:local"
export CARGO_BUILD_JOBS=4
readonly CARGO_BUILD_JOBS
# Internal handshake for the ignored Rust test. This runner forwards it only
# after it proves exact ownership of the random-canary container below.
readonly TEST_HARNESS_ACK="I_UNDERSTAND_PER20_DROPS_SCRATCH_DATABASES_ROLES_AND_CREATED_BABYLON_INTEL"
readonly LIVE_FOCUS="${BABYLON_LEGACY_ADOPTER_LIVE_FOCUS:-}"
readonly CURRENT_CENSUS_V2_EXPORT_DIR="${BABYLON_CURRENT_CENSUS_V2_EXPORT_DIR:-}"
CANARY="$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
readonly CANARY
readonly CONTAINER="babylon-per20-adopter-${CANARY:0:12}"
readonly RUNTIME_TEMPLATE="per281_runtime_template_${CANARY:0:12}"
VOLUME=""
OWNED=0
RUNTIME_TEMPLATE_CREATED=0

die() {
  printf 'run_rust_legacy_adopter_pg: %s\n' "$*" >&2
  exit 2
}

emit_runtime_logs() {
  [ "$OWNED" -eq 1 ] || return 0
  printf 'PER-20 runtime PostgreSQL log excerpt follows (last 200 lines):\n' >&2
  timeout --signal=TERM --kill-after=2s 10s \
    docker logs --timestamps --tail 200 "$CONTAINER" >&2 || true
}

die_with_runtime_logs() {
  emit_runtime_logs
  die "$@"
}

case "$LIVE_FOCUS" in
  "" | clean_bootstrap | h3_atomicity | installed_mutation | schema_epoch_fresh | schema_epoch_matrix | \
    schema_epoch_rollback | schema_epoch_v5_census | schema_epoch_v6_census | schema_epoch_v7_census | \
    h3_pg_oracle | h3_reference_installer | h3_shadow_backfill | \
    rust_persistence_runtime | archive_worker | reader_role | client_dossier | runtime_census_v2 | pr) ;;
  *) die "unsupported live focus: $LIVE_FOCUS" ;;
esac

if [ "$LIVE_FOCUS" = "runtime_census_v2" ]; then
  [ -n "$CURRENT_CENSUS_V2_EXPORT_DIR" ] ||
    die "runtime_census_v2 requires BABYLON_CURRENT_CENSUS_V2_EXPORT_DIR"
  case "$CURRENT_CENSUS_V2_EXPORT_DIR" in
    /*) ;;
    *) die "current-census-v2 output directory must be absolute" ;;
  esac
  [ -d "$CURRENT_CENSUS_V2_EXPORT_DIR" ] && [ ! -L "$CURRENT_CENSUS_V2_EXPORT_DIR" ] ||
    die "current-census-v2 output directory must be an existing non-symlink directory"
elif [ -n "$CURRENT_CENSUS_V2_EXPORT_DIR" ]; then
  die "current-census-v2 export directory is accepted only with runtime_census_v2 focus"
fi

require_container_absent() {
  local context="$1"
  local inspect_status
  if timeout --signal=TERM --kill-after=2s 10s \
      docker container inspect "$CONTAINER" >/dev/null 2>&1; then
    die "task-owned container is present during $context"
  else
    inspect_status="$?"
  fi
  if [ "$inspect_status" -ne 1 ]; then
    die "task-owned container absence could not be verified during $context"
  fi
}

require_volume_absent() {
  local inspect_status
  if timeout --signal=TERM --kill-after=2s 10s \
      docker volume inspect "$VOLUME" >/dev/null 2>&1; then
    die "task-owned anonymous volume cleanup did not complete"
  else
    inspect_status="$?"
  fi
  if [ "$inspect_status" -ne 1 ]; then
    die "task-owned anonymous volume cleanup could not be verified"
  fi
}

claim_task_container() {
  local expected_container_id="$1"
  local identity
  local inspect_status=0
  identity="$(timeout --signal=TERM --kill-after=2s 10s \
    docker inspect --format '{{.Id}}|{{index .Config.Labels "babylon.per20_disposable"}}' \
      "$CONTAINER")" || inspect_status="$?"
  [ "$inspect_status" -eq 0 ] || return 1
  local actual_container_id="${identity%%|*}"
  local actual_canary="${identity#*|}"
  [ "$actual_canary" = "$CANARY" ] || return 1
  if [ -n "$expected_container_id" ] &&
      [ "$actual_container_id" != "$expected_container_id" ]; then
    return 1
  fi
  OWNED=1
}

# shellcheck disable=SC2329 # Invoked by the EXIT trap after ownership is proved.
cleanup_best_effort() {
  [ "$OWNED" -eq 1 ] || return 0
  timeout --signal=TERM --kill-after=5s 30s \
    docker rm --force --volumes "$CONTAINER" >/dev/null 2>&1 || true
}

cleanup_checked() {
  [ "$OWNED" -eq 1 ] || return 0
  timeout --signal=TERM --kill-after=5s 30s \
    docker rm --force --volumes "$CONTAINER" >/dev/null
  require_container_absent "cleanup"
  if [ -n "$VOLUME" ]; then
    require_volume_absent
  fi
  OWNED=0
}

wait_for_runtime() {
  local deadline=$((SECONDS + 90))
  local host_probe
  local remaining
  for _attempt in {1..90}; do
    remaining=$((deadline - SECONDS))
    if [ "$remaining" -le 0 ]; then
      break
    fi
    if [ "$(timeout --signal=TERM --kill-after=1s "${remaining}s" \
      docker exec "$CONTAINER" psql -qAt -U test -d template1 -c \
        "SELECT pg_catalog.current_setting('babylon.per20_disposable', true) = '$CANARY' \
           AND (SELECT extversion = '3.5.7' FROM pg_catalog.pg_extension WHERE extname = 'postgis') \
           AND (SELECT extversion = '0.8.5' FROM pg_catalog.pg_extension WHERE extname = 'vector')" \
        2>/dev/null || true)" = "t" ]; then
      host_probe="$(timeout --signal=TERM --kill-after=1s 1s \
        env -u PGHOSTADDR -u PGOPTIONS -u PGSERVICE -u PGSERVICEFILE \
          PGPASSWORD=test PGCONNECT_TIMEOUT=1 PGSSLMODE=disable \
        psql -X -w -qAt -F '|' -h 127.0.0.1 -p "$PORT" -U test -d postgres \
          -v ON_ERROR_STOP=1 \
          -c "SELECT 1, pg_catalog.current_setting('babylon.per20_disposable', true)" \
          2>/dev/null || true)"
      if [ "$host_probe" = "1|$CANARY" ]; then
        return 0
      fi
    fi
    if [ "$SECONDS" -lt "$deadline" ]; then
      sleep 1
    fi
  done
  return 1
}

run_schema_epoch_rollback_test() {
  local migration="$1"
  timeout --signal=TERM --kill-after=10s 300s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --lib \
      "schema_epoch::live_rollback_tests::${migration}_rollback_and_ambiguous_commit_reconciliation_are_atomic" \
      --locked -- --nocapture --ignored --exact --test-threads=1
}

create_runtime_template() {
  local bootstrap_dsn="$1"
  local source_observation
  local template_observation
  [ "$bootstrap_dsn" = "postgresql://test:test@127.0.0.1:$PORT/babylon_test" ] ||
    die "runtime template source DSN did not equal the task-owned bootstrap target"
  source_observation="$(timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d babylon_test \
      -v ON_ERROR_STOP=1 \
      -c "SELECT \
            (SELECT pg_catalog.string_agg(ordinal::pg_catalog.text || ':' || \
                     state_tag::pg_catalog.text || ':' || schema_epoch::pg_catalog.text, \
                     ',' ORDER BY ordinal) \
             FROM babylon_meta.persistence_authority_ledger) \
            || '|' || (SELECT pg_catalog.count(*)::pg_catalog.text \
                       FROM babylon_meta.campaign)")" || return
  [ "$source_observation" = "1:1:8,2:2:9|0" ] ||
    die "runtime template source was not clean and Rust-active: $source_observation"
  timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d postgres \
      -v ON_ERROR_STOP=1 \
      -c "CREATE DATABASE \"$RUNTIME_TEMPLATE\" OWNER test TEMPLATE babylon_test" || return
  RUNTIME_TEMPLATE_CREATED=1
  template_observation="$(timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d "$RUNTIME_TEMPLATE" \
      -v ON_ERROR_STOP=1 \
      -c "SELECT \
            (SELECT pg_catalog.string_agg(ordinal::pg_catalog.text || ':' || \
                     state_tag::pg_catalog.text || ':' || schema_epoch::pg_catalog.text, \
                     ',' ORDER BY ordinal) \
             FROM babylon_meta.persistence_authority_ledger) \
            || '|' || (SELECT pg_catalog.count(*)::pg_catalog.text \
                       FROM babylon_meta.campaign)")" || return
  [ "$template_observation" = "1:1:8,2:2:9|0" ] ||
    die "runtime template clone was not clean and Rust-active: $template_observation"
  printf 'PER-281 runtime template ready: database=%s authority=%s\n' \
    "$RUNTIME_TEMPLATE" "$template_observation"
}

verify_runtime_template_and_clone_cleanup() {
  local template_observation
  local clone_count
  template_observation="$(timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d "$RUNTIME_TEMPLATE" \
      -v ON_ERROR_STOP=1 \
      -c "SELECT \
            (SELECT pg_catalog.string_agg(ordinal::pg_catalog.text || ':' || \
                     state_tag::pg_catalog.text || ':' || schema_epoch::pg_catalog.text, \
                     ',' ORDER BY ordinal) \
             FROM babylon_meta.persistence_authority_ledger) \
            || '|' || (SELECT pg_catalog.count(*)::pg_catalog.text \
                       FROM babylon_meta.campaign)")" || return
  [ "$template_observation" = "1:1:8,2:2:9|0" ] || return 1
  clone_count="$(timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d postgres \
      -v ON_ERROR_STOP=1 \
      -c "SELECT pg_catalog.count(*) FROM pg_catalog.pg_database \
          WHERE datname LIKE 'per281_runtime_%' \
            AND datname <> '$RUNTIME_TEMPLATE'")" || return
  [ "$clone_count" = "0" ] || return 1
  printf 'PER-281 runtime template isolated: database=%s authority=%s clones=%s\n' \
    "$RUNTIME_TEMPLATE" "$template_observation" "$clone_count"
}

drop_runtime_template() {
  timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d postgres \
      -v ON_ERROR_STOP=1 \
      -c "DROP DATABASE IF EXISTS \"$RUNTIME_TEMPLATE\" WITH (FORCE)" || return
  RUNTIME_TEMPLATE_CREATED=0
}

# shellcheck disable=SC2329 # Invoked by the INT, TERM, and HUP traps below.
on_signal() {
  local -r status="$1"
  trap - EXIT INT TERM HUP
  if [ "$OWNED" -eq 0 ]; then
    claim_task_container "" || true
  fi
  cleanup_best_effort
  exit "$status"
}

trap cleanup_best_effort EXIT
trap 'on_signal 130' INT
trap 'on_signal 143' TERM
trap 'on_signal 129' HUP

[ "${#CANARY}" -eq 32 ] || die "canary generation failed"
command -v psql >/dev/null 2>&1 || die "psql client is required for host readiness proof"
require_container_absent "startup"

printf 'PER-20 runtime target: image=%s container=%s volume=anonymous port=dynamic-loopback\n' \
  "$IMAGE" "$CONTAINER"
env DOCKER_BUILDKIT=1 \
  timeout --signal=TERM --kill-after=10s 180s \
  docker build --tag "$IMAGE" "$REPO_ROOT/docker/postgres"

run_status=0
created_container_id="$(timeout --signal=TERM --kill-after=5s 30s docker run --detach \
  --name "$CONTAINER" \
  --label "babylon.per20_disposable=$CANARY" \
  --publish 127.0.0.1::5432 \
  --mount type=volume,target=/var/lib/postgresql/data \
  --mount "type=bind,source=$REPO_ROOT/docker/postgres/postgresql.ci.conf,target=/etc/postgresql/postgresql.conf,readonly" \
  --mount "type=bind,source=$REPO_ROOT/docker/postgres/initdb,target=/docker-entrypoint-initdb.d,readonly" \
  --shm-size=1g \
  --env POSTGRES_USER=test \
  --env POSTGRES_PASSWORD=test \
  --env POSTGRES_DB=babylon_test \
  "$IMAGE" postgres \
  -c config_file=/etc/postgresql/postgresql.conf \
  -c "babylon.per20_disposable=$CANARY")" || run_status="$?"
if [ "$run_status" -eq 0 ]; then
  claim_task_container "$created_container_id" ||
    die_with_runtime_logs "created container identity was not proved"
else
  claim_task_container "" || true
  die_with_runtime_logs "task-owned container did not start"
fi

inspect_status=0
VOLUME="$(timeout --signal=TERM --kill-after=2s 10s \
  docker inspect --format '{{range .Mounts}}{{if eq .Destination "/var/lib/postgresql/data"}}{{.Name}}{{end}}{{end}}' "$CONTAINER")" ||
  inspect_status="$?"
[ "$inspect_status" -eq 0 ] && [ -n "$VOLUME" ] ||
  die_with_runtime_logs "anonymous data volume identity was not resolved"
port_status=0
published="$(timeout --signal=TERM --kill-after=2s 10s \
  docker port "$CONTAINER" 5432/tcp)" || port_status="$?"
[ "$port_status" -eq 0 ] || die_with_runtime_logs "dynamic loopback port was not resolved"
readonly PORT="${published##*:}"
case "$PORT" in
  ''|*[!0-9]*) die_with_runtime_logs "dynamic loopback port was not numeric" ;;
esac

wait_for_runtime || die_with_runtime_logs "pinned PostgreSQL runtime was not ready within 90 seconds"

runtime_metadata="$(timeout --signal=TERM --kill-after=2s 10s \
  docker exec "$CONTAINER" psql -X -qAt -U test -d template1 -c \
    "SELECT (pg_catalog.current_setting('server_version_num')::pg_catalog.int4 / 10000)::pg_catalog.text \
      || '|' || pg_catalog.current_setting('server_version')")" ||
  die_with_runtime_logs "PostgreSQL runtime version was not observed"
readonly RUNTIME_MAJOR="${runtime_metadata%%|*}"
readonly RUNTIME_VERSION="${runtime_metadata#*|}"
case "$RUNTIME_MAJOR" in
  ''|*[!0-9]*) die_with_runtime_logs "PostgreSQL runtime major version was not numeric" ;;
esac
[ -n "$RUNTIME_VERSION" ] || die_with_runtime_logs "PostgreSQL runtime version was empty"

printf 'PER-20 runtime ready: container=%s volume=%s port=%s\n' \
  "$CONTAINER" "$VOLUME" "$PORT"
printf 'PER-20 runtime PostgreSQL: major=%s version=%s\n' \
  "$RUNTIME_MAJOR" "$RUNTIME_VERSION"
BOOTSTRAP_DSN="postgresql://test:test@127.0.0.1:$PORT/babylon_test"
readonly BOOTSTRAP_DSN
status=0
if [ "$LIVE_FOCUS" = "clean_bootstrap" ] || [ "$LIVE_FOCUS" = "pr" ]; then
  HOSTILE_OPTIONS_DSN="${BOOTSTRAP_DSN}?options=-c%20search_path%3Dredirected%2Cpublic"
  readonly HOSTILE_OPTIONS_DSN
  before_options_refusal="$(timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d babylon_test \
      -v ON_ERROR_STOP=1 \
      -c "SELECT pg_catalog.count(*)::pg_catalog.text \
          FROM pg_catalog.pg_class AS relation \
          JOIN pg_catalog.pg_namespace AS namespace \
            ON namespace.oid = relation.relnamespace \
          WHERE namespace.nspname NOT IN ('pg_catalog', 'information_schema') \
            AND namespace.nspname NOT LIKE 'pg_toast%'")" || status=$?

  if [ "$status" -eq 0 ]; then
    hostile_options_status=0
    cd "$REPO_ROOT"
    timeout --signal=TERM --kill-after=30s 600s \
      env BABYLON_RUNTIME_DSN="$HOSTILE_OPTIONS_DSN" \
      mise run db:bootstrap || hostile_options_status=$?
    if [ "$hostile_options_status" -eq 0 ]; then
      printf 'options-bearing DSN unexpectedly passed schema preflight\n' >&2
      status=1
    fi
  fi
  if [ "$status" -eq 0 ]; then
    after_options_refusal="$(timeout --signal=TERM --kill-after=2s 10s \
      env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
      psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d babylon_test \
        -v ON_ERROR_STOP=1 \
        -c "SELECT pg_catalog.count(*)::pg_catalog.text \
            FROM pg_catalog.pg_class AS relation \
            JOIN pg_catalog.pg_namespace AS namespace \
              ON namespace.oid = relation.relnamespace \
            WHERE namespace.nspname NOT IN ('pg_catalog', 'information_schema') \
              AND namespace.nspname NOT LIKE 'pg_toast%'")" || status=$?
    if [ "$status" -eq 0 ] && [ "$after_options_refusal" != "$before_options_refusal" ]; then
      printf 'options-bearing DSN changed the fresh database before refusal: before=%s after=%s\n' \
        "$before_options_refusal" "$after_options_refusal" >&2
      status=1
    fi
  fi

  if [ "$status" -eq 0 ]; then
    cd "$REPO_ROOT"
    timeout --signal=TERM --kill-after=30s 600s \
      env BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN" \
      PGHOST="host.invalid" \
      PGHOSTADDR="203.0.113.1" \
      PGPORT="1" \
      PGDATABASE="redirected" \
      PGOPTIONS='-c search_path=redirected,public' \
      PGSERVICE="redirected" \
      PGSERVICEFILE="/nonexistent/babylon-pg-service.conf" \
      PGSYSCONFDIR="/nonexistent/babylon-pg-service.d" \
      mise run db:bootstrap || status=$?
  fi

  if [ "$status" -eq 0 ]; then
    observation="$(timeout --signal=TERM --kill-after=2s 10s \
      env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
      psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d babylon_test \
        -v ON_ERROR_STOP=1 \
        -c "SELECT \
              (SELECT pg_catalog.string_agg(\
                   ordinal::pg_catalog.text || ':' || state_tag::pg_catalog.text || ':' || \
                   schema_epoch::pg_catalog.text, ',' ORDER BY ordinal\
               ) FROM babylon_meta.persistence_authority_ledger) \
              || '|' || (pg_catalog.to_regclass('public.hex_spatial_map') IS NULL)::pg_catalog.text \
              || '|' || (pg_catalog.to_regclass('babylon_state.campaign_foundation') IS NOT NULL)::pg_catalog.text")" || status=$?
    if [ "$status" -eq 0 ] && [ "$observation" != "1:1:8,2:2:9|true|true" ]; then
      printf 'clean bootstrap observation mismatch: expected=1:1:8,2:2:9|true|true actual=%s\n' \
        "$observation" >&2
      status=1
    fi
  fi

  if [ "$status" -eq 0 ] && [ "$LIVE_FOCUS" = "pr" ]; then
    create_runtime_template "$BOOTSTRAP_DSN" || status=$?
  fi

  if [ "$status" -eq 0 ]; then
    timeout --signal=TERM --kill-after=30s 1800s \
      env BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN" \
      mise run qa:michigan-rollover-smoke || status=$?
  fi
fi
if [ "$status" -eq 0 ] &&
    { [ "$LIVE_FOCUS" = "rust_persistence_runtime" ] || [ "$LIVE_FOCUS" = "archive_worker" ] || \
      [ "$LIVE_FOCUS" = "reader_role" ] || [ "$LIVE_FOCUS" = "client_dossier" ]; }; then
  cd "$REPO_ROOT"
  timeout --signal=TERM --kill-after=30s 600s \
    env BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN" \
    mise run db:bootstrap || status=$?
  if [ "$status" -eq 0 ]; then
    create_runtime_template "$BOOTSTRAP_DSN" || status=$?
  fi
fi
cd "$REPO_ROOT/rust"
if [ "$LIVE_FOCUS" = "schema_epoch_rollback" ]; then
  for migration in migration_four migration_five migration_six migration_seven; do
    run_schema_epoch_rollback_test "$migration" || { status=$?; break; }
  done
fi

if [ "$LIVE_FOCUS" = "h3_atomicity" ] || [ "$LIVE_FOCUS" = "pr" ]; then
  timeout --signal=TERM --kill-after=10s 300s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --lib \
      schema_epoch::live_rollback_tests::h3_installer_rollback_and_ambiguous_commit_reconciliation_are_atomic \
      --locked -- --nocapture --ignored --exact --test-threads=1 || status=$?
fi

if [ "$status" -eq 0 ] &&
    { [ "$LIVE_FOCUS" = "rust_persistence_runtime" ] || [ "$LIVE_FOCUS" = "pr" ]; }; then
  env \
    BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
    BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
    BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
    BABYLON_RUNTIME_TEMPLATE_DB="$RUNTIME_TEMPLATE" \
    CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
  cargo test -p babylon-persistence --lib \
    runtime::live_tests::live_ \
    --locked -- --nocapture --ignored --test-threads=1 || status=$?
fi

if [ "$status" -eq 0 ] && [ "$LIVE_FOCUS" = "archive_worker" ]; then
  timeout --signal=TERM --kill-after=10s 900s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      BABYLON_RUNTIME_TEMPLATE_DB="$RUNTIME_TEMPLATE" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --test archive_worker_live --test place_producer_live --test county_producer_live \
      --locked -- --nocapture --ignored --test-threads=1 || status=$?
fi

if [ "$status" -eq 0 ] && [ "$LIVE_FOCUS" = "reader_role" ]; then
  timeout --signal=TERM --kill-after=10s 900s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      BABYLON_RUNTIME_TEMPLATE_DB="$RUNTIME_TEMPLATE" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --test reader_role_live --test observer_material_live \
      --locked -- --nocapture --ignored --test-threads=1 || status=$?
fi

# PER-23 Slice 3 (ADR249 R10-R11): the headless dossier CLI driven as a
# real child process against a confined reader login. The first build of
# the Bevy client stack is heavy, so this leg carries the widest cargo
# envelope of any single focus.
if [ "$status" -eq 0 ] && [ "$LIVE_FOCUS" = "client_dossier" ]; then
  timeout --signal=TERM --kill-after=10s 1200s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      BABYLON_RUNTIME_TEMPLATE_DB="$RUNTIME_TEMPLATE" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-client --test dossier_cli_live \
      --locked -- --nocapture --ignored --test-threads=1 || status=$?
fi

if [ "$status" -eq 0 ] &&
    { [ "$LIVE_FOCUS" = "installed_mutation" ] || [ "$LIVE_FOCUS" = "pr" ]; }; then
  timeout --signal=TERM --kill-after=10s 300s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      BABYLON_LEGACY_ADOPTER_LIVE_FOCUS=installed_mutation \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --test legacy_adopter_postgres --locked -- --nocapture \
      --ignored --test-threads=1 || status=$?
fi

if [ "$status" -eq 0 ] &&
    [ "$LIVE_FOCUS" = "h3_reference_installer" ]; then
  timeout --signal=TERM --kill-after=10s 900s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      BABYLON_LEGACY_ADOPTER_LIVE_FOCUS="$LIVE_FOCUS" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --test legacy_adopter_postgres --locked -- --nocapture \
      --ignored --test-threads=1 || status=$?
fi

if [ "$status" -eq 0 ] &&
    { [ "$LIVE_FOCUS" = "schema_epoch_fresh" ] ||
      [ "$LIVE_FOCUS" = "schema_epoch_matrix" ] ||
      [ "$LIVE_FOCUS" = "schema_epoch_rollback" ] ||
      [ "$LIVE_FOCUS" = "schema_epoch_v5_census" ] ||
      [ "$LIVE_FOCUS" = "schema_epoch_v6_census" ] ||
      [ "$LIVE_FOCUS" = "schema_epoch_v7_census" ] ||
      [ "$LIVE_FOCUS" = "runtime_census_v2" ] ||
      [ "$LIVE_FOCUS" = "h3_pg_oracle" ] ||
      [ "$LIVE_FOCUS" = "h3_shadow_backfill" ]; }; then
  timeout --signal=TERM --kill-after=10s 300s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      BABYLON_LEGACY_ADOPTER_LIVE_FOCUS="$LIVE_FOCUS" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --test legacy_adopter_postgres --locked -- --nocapture \
      --ignored --test-threads=1 || status=$?
fi
if [ "$status" -eq 0 ] && [ -z "$LIVE_FOCUS" ]; then
  timeout --signal=TERM --kill-after=10s 900s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --test legacy_adopter_postgres --locked -- --nocapture \
      --ignored --test-threads=1 || status=$?
fi

if [ "$status" -eq 0 ] && [ -z "$LIVE_FOCUS" ]; then
  timeout --signal=TERM --kill-after=10s 300s \
    env \
      BABYLON_LEGACY_ADOPTER_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_ACK="$TEST_HARNESS_ACK" \
      BABYLON_LEGACY_ADOPTER_DISPOSABLE_CANARY="$CANARY" \
      CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}" \
    cargo test -p babylon-persistence --lib \
      schema_epoch::live_rollback_tests::rollback_and_ambiguous_commit_reconciliation_are_atomic \
      --locked -- --nocapture --ignored --exact --test-threads=1 || status=$?
fi

if [ "$RUNTIME_TEMPLATE_CREATED" -eq 1 ]; then
  template_status=0
  verify_runtime_template_and_clone_cleanup || template_status=$?
  drop_runtime_template || template_status=$?
  if [ "$status" -eq 0 ] && [ "$template_status" -ne 0 ]; then
    status="$template_status"
  fi
fi

if [ "$status" -ne 0 ]; then
  emit_runtime_logs
fi

cleanup_checked
trap - EXIT INT TERM HUP
printf 'PER-20 runtime cleanup verified: container=%s volume=%s\n' "$CONTAINER" "$VOLUME"
exit "$status"
