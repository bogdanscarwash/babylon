#!/usr/bin/env bash
# Validate the current Rust runtime in one owned, disposable PostgreSQL instance.
set -euo pipefail

REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
readonly REPO_ROOT
readonly IMAGE="babylon-postgres-runtime:local"
readonly PREBUILT_IMAGE_ID="${BABYLON_POSTGRES_IMAGE_ID:-}"
export CARGO_BUILD_JOBS=4
readonly CARGO_BUILD_JOBS
# Internal handshake for the ignored Rust test. This runner forwards it only
# after it proves exact ownership of the random-canary container below.
readonly TEST_HARNESS_ACK="I_UNDERSTAND_THIS_DISPOSABLE_RUNTIME_DROPS_ITS_SCRATCH_DATABASES_AND_ROLES"
readonly LIVE_FOCUS="${BABYLON_POSTGRES_LIVE_FOCUS-runtime_smoke}"
CANARY="$(od -An -N16 -tx1 /dev/urandom | tr -d ' \n')"
readonly CANARY
readonly CONTAINER="babylon-runtime-pg-${CANARY:0:12}"
readonly RUNTIME_TEMPLATE="per281_runtime_template_${CANARY:0:12}"
VOLUME=""
OWNED=0
CONTAINER_ID=""
RUNTIME_TEMPLATE_CREATED=0

die() {
  printf 'run_rust_postgres: %s\n' "$*" >&2
  exit 2
}

emit_runtime_logs() {
  [ "$OWNED" -eq 1 ] || return 0
  printf 'Rust runtime PostgreSQL log excerpt follows (last 200 lines):\n' >&2
  timeout --signal=TERM --kill-after=2s 10s \
    docker logs --timestamps --tail 200 "$CONTAINER_ID" >&2 || true
}

die_with_runtime_logs() {
  emit_runtime_logs
  die "$@"
}

if [ "${BABYLON_POSTGRES_IMAGE_ID+x}" = x ] &&
    [[ ! "$PREBUILT_IMAGE_ID" =~ ^sha256:[0-9a-f]{64}$ ]]; then
  die "BABYLON_POSTGRES_IMAGE_ID must be a complete sha256 image ID"
fi

case "$LIVE_FOCUS" in
  runtime_smoke | reference_integrity | runtime | archive | reader | statewide_synthetic | statewide_qualified | client) ;;
  *) die "unsupported live focus: $LIVE_FOCUS" ;;
esac

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
    docker inspect --format '{{.Id}}|{{index .Config.Labels "babylon.disposable_runtime"}}' \
      "$CONTAINER")" || inspect_status="$?"
  [ "$inspect_status" -eq 0 ] || return 1
  local actual_container_id="${identity%%|*}"
  local actual_canary="${identity#*|}"
  [ "$actual_canary" = "$CANARY" ] || return 1
  if [ -n "$expected_container_id" ] &&
      [ "$actual_container_id" != "$expected_container_id" ]; then
    return 1
  fi
  [[ "$actual_container_id" =~ ^[0-9a-f]{64}$ ]] || return 1
  CONTAINER_ID="$actual_container_id"
  OWNED=1
}

# shellcheck disable=SC2329 # Invoked by the EXIT trap after ownership is proved.
cleanup_best_effort() {
  [ "$OWNED" -eq 1 ] || return 0
  timeout --signal=TERM --kill-after=5s 30s \
    docker rm --force --volumes "$CONTAINER_ID" >/dev/null 2>&1 || true
}

cleanup_checked() {
  [ "$OWNED" -eq 1 ] || return 0
  timeout --signal=TERM --kill-after=5s 30s \
    docker rm --force --volumes "$CONTAINER_ID" >/dev/null
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
        "SELECT pg_catalog.current_setting('babylon.disposable_runtime', true) = '$CANARY' \
           AND pg_catalog.current_setting('server_version_num')::pg_catalog.int4 / 10000 = 17 \
           AND (SELECT extversion = '3.5.7' FROM pg_catalog.pg_extension WHERE extname = 'postgis') \
           AND (SELECT extversion = '0.8.5' FROM pg_catalog.pg_extension WHERE extname = 'vector')" \
        2>/dev/null || true)" = "t" ]; then
      host_probe="$(timeout --signal=TERM --kill-after=1s 1s \
        env -u PGHOSTADDR -u PGOPTIONS -u PGSERVICE -u PGSERVICEFILE \
          PGPASSWORD=test PGCONNECT_TIMEOUT=1 PGSSLMODE=disable \
        psql -X -w -qAt -F '|' -h 127.0.0.1 -p "$PORT" -U test -d postgres \
          -v ON_ERROR_STOP=1 \
          -c "SELECT 1, pg_catalog.current_setting('babylon.disposable_runtime', true)" \
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

# Each heavy child has its own ceiling. Elapsed receipts measure the five-minute
# warm-runtime target; cold compilation has additional headroom, not a false SLA.
run_phase() {
  local label="$1" limit="$2" started="$SECONDS" status=0
  shift 2
  printf 'Rust PostgreSQL phase start: focus=%s phase=%s limit_seconds=%s\n' "$LIVE_FOCUS" "$label" "$limit"
  timeout --signal=TERM --kill-after=10s "${limit}s" "$@" || status=$?
  printf 'Rust PostgreSQL phase complete: focus=%s phase=%s elapsed_seconds=%s status=%s\n' \
    "$LIVE_FOCUS" "$label" "$((SECONDS - started))" "$status"
  return "$status"
}

runtime_observation() {
  timeout --signal=TERM --kill-after=2s 10s \
    env -u PGHOSTADDR -u PGOPTIONS -u PGSERVICE -u PGSERVICEFILE \
      PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d "$1" \
      -v ON_ERROR_STOP=1 -c "SELECT \
        (SELECT (pg_catalog.count(*) = 1 AND pg_catalog.bool_and(singleton AND pg_catalog.octet_length(schema_sha256) = 32))::pg_catalog.text FROM babylon_meta.current_schema) \
        || '|' || (SELECT pg_catalog.count(*)::pg_catalog.text FROM babylon_meta.campaign) \
        || '|' || (pg_catalog.to_regclass('public.hex_spatial_map') IS NULL)::pg_catalog.text \
        || '|' || (pg_catalog.to_regclass('babylon_state.campaign_foundation') IS NOT NULL)::pg_catalog.text"
}

readonly CLEAN_RUNTIME="true|0|true|true"

create_runtime_template() {
  local observation
  observation="$(runtime_observation babylon_test)" || return
  [ "$observation" = "$CLEAN_RUNTIME" ] || die "runtime template source was not clean and Rust-active: $observation"
  timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d postgres \
      -v ON_ERROR_STOP=1 -c "CREATE DATABASE \"$RUNTIME_TEMPLATE\" OWNER test TEMPLATE babylon_test" || return
  RUNTIME_TEMPLATE_CREATED=1
  observation="$(runtime_observation "$RUNTIME_TEMPLATE")" || return
  [ "$observation" = "$CLEAN_RUNTIME" ] || die "runtime template clone was not clean and Rust-active: $observation"
}

verify_runtime_template_and_clone_cleanup() {
  local observation clone_count
  observation="$(runtime_observation "$RUNTIME_TEMPLATE")" || return
  [ "$observation" = "$CLEAN_RUNTIME" ] || return 1
  clone_count="$(timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d postgres \
      -v ON_ERROR_STOP=1 -c "SELECT pg_catalog.count(*) FROM pg_catalog.pg_database \
          WHERE datname LIKE 'per281_runtime_%' AND datname <> '$RUNTIME_TEMPLATE'")" || return
  [ "$clone_count" = "0" ] || return 1
  printf 'Rust runtime template isolated: database=%s authority=%s clones=%s\n' "$RUNTIME_TEMPLATE" "$observation" "$clone_count"
}

drop_runtime_template() {
  timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d postgres \
      -v ON_ERROR_STOP=1 -c "DROP DATABASE IF EXISTS \"$RUNTIME_TEMPLATE\" WITH (FORCE)" || return
  RUNTIME_TEMPLATE_CREATED=0
}

fresh_relation_count() {
  timeout --signal=TERM --kill-after=2s 10s \
    env PGPASSWORD=test PGCONNECT_TIMEOUT=2 PGSSLMODE=disable \
    psql -X -w -qAt -h 127.0.0.1 -p "$PORT" -U test -d babylon_test \
      -v ON_ERROR_STOP=1 -c "SELECT pg_catalog.count(*) FROM pg_catalog.pg_class AS relation \
        JOIN pg_catalog.pg_namespace AS namespace ON namespace.oid = relation.relnamespace \
        WHERE namespace.nspname NOT IN ('pg_catalog', 'information_schema') \
          AND namespace.nspname NOT LIKE 'pg_toast%'"
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

printf 'Rust runtime target: image=%s container=%s volume=anonymous port=dynamic-loopback\n' \
  "$IMAGE" "$CONTAINER"
if [ -z "$PREBUILT_IMAGE_ID" ]; then
  env DOCKER_BUILDKIT=1 \
    timeout --signal=TERM --kill-after=10s 180s \
    docker build --tag "$IMAGE" "$REPO_ROOT/docker/postgres"
fi
IMAGE_ID="$(timeout --signal=TERM --kill-after=2s 10s \
  docker image inspect --format '{{.Id}}' "$IMAGE")" ||
  die "PostgreSQL image tag could not be inspected"
readonly IMAGE_ID
[[ "$IMAGE_ID" =~ ^sha256:[0-9a-f]{64}$ ]] ||
  die "PostgreSQL image tag did not resolve to a complete sha256 image ID"
if [ -n "$PREBUILT_IMAGE_ID" ] && [ "$IMAGE_ID" != "$PREBUILT_IMAGE_ID" ]; then
  die "prebuilt PostgreSQL image ID does not match the expected tag"
fi

run_status=0
created_container_id="$(timeout --signal=TERM --kill-after=5s 30s docker run --detach \
  --name "$CONTAINER" \
  --label "babylon.disposable_runtime=$CANARY" \
  --publish 127.0.0.1::5432 \
  --mount type=volume,target=/var/lib/postgresql/data \
  --mount "type=bind,source=$REPO_ROOT/docker/postgres/postgresql.ci.conf,target=/etc/postgresql/postgresql.conf,readonly" \
  --mount "type=bind,source=$REPO_ROOT/docker/postgres/initdb,target=/docker-entrypoint-initdb.d,readonly" \
  --shm-size=1g \
  --env POSTGRES_USER=test \
  --env POSTGRES_PASSWORD=test \
  --env POSTGRES_DB=babylon_test \
  "$IMAGE_ID" postgres \
  -c config_file=/etc/postgresql/postgresql.conf \
  -c "babylon.disposable_runtime=$CANARY")" || run_status="$?"
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
[[ "$published" =~ ^127\.0\.0\.1:([0-9]{1,5})$ ]] ||
  die_with_runtime_logs "published PostgreSQL target was not one IPv4 loopback port"
readonly PORT="${BASH_REMATCH[1]}"
[ "$((10#$PORT))" -ge 1 ] && [ "$((10#$PORT))" -le 65535 ] ||
  die_with_runtime_logs "dynamic loopback port was outside the valid range"

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
[ "$RUNTIME_MAJOR" = "17" ] || die_with_runtime_logs "PostgreSQL runtime major did not equal pinned version 17"
[ -n "$RUNTIME_VERSION" ] || die_with_runtime_logs "PostgreSQL runtime version was empty"

printf 'Rust runtime ready: container=%s volume=%s port=%s\n' \
  "$CONTAINER" "$VOLUME" "$PORT"
printf 'Rust runtime PostgreSQL: major=%s version=%s\n' \
  "$RUNTIME_MAJOR" "$RUNTIME_VERSION"
BOOTSTRAP_DSN="postgresql://test:test@127.0.0.1:$PORT/babylon_test"
readonly BOOTSTRAP_DSN
# Explicit environment admission reaches only tests confined to this container.
export BABYLON_POSTGRES_TEST_DSN="postgresql://test:test@127.0.0.1:$PORT/postgres"
export BABYLON_POSTGRES_DISPOSABLE_ACK="$TEST_HARNESS_ACK"
export BABYLON_POSTGRES_DISPOSABLE_CANARY="$CANARY"
export CARGO_TARGET_DIR="${CARGO_TARGET_DIR:-$REPO_ROOT/rust/target}"
export PATH="$CARGO_TARGET_DIR/debug:$PATH"
# Suppress inherited libpq service/target redirection for every host probe.
unset PGHOST PGHOSTADDR PGPORT PGDATABASE PGOPTIONS PGSERVICE PGSERVICEFILE PGSYSCONFDIR
status=0
if [ "$LIVE_FOCUS" != reference_integrity ]; then
  cd "$REPO_ROOT/rust"
  run_phase runtime_build 600 cargo build -p babylon-persistence --bin babylon-runtime --locked || status=$?
  if [ "$status" -eq 0 ] && [ "$LIVE_FOCUS" = runtime_smoke ]; then
    before="$(fresh_relation_count)" || status=$?
    if [ "$status" -eq 0 ]; then
      hostile_status=0
      run_phase hostile_dsn 30 env \
        BABYLON_RUNTIME_DSN="${BOOTSTRAP_DSN}?options=-c%20search_path%3Dredirected%2Cpublic" \
        babylon-runtime bootstrap || hostile_status=$?
      [ "$hostile_status" -eq 1 ] || { printf 'options-bearing DSN did not produce an ordinary refusal: status=%s\n' "$hostile_status" >&2; status=1; }
      after="$(fresh_relation_count)" || status=$?
      [ "$after" = "$before" ] || { printf 'options-bearing DSN changed the fresh database before refusal\n' >&2; status=1; }
    fi
  fi
  if [ "$status" -eq 0 ]; then
    cd "$REPO_ROOT"
    run_phase fresh_bootstrap 180 env BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN" \
      PGHOST=host.invalid PGHOSTADDR=203.0.113.1 PGPORT=1 PGDATABASE=redirected \
      PGOPTIONS='-c search_path=redirected,public' PGSERVICE=redirected \
      PGSERVICEFILE=/nonexistent/babylon-pg-service.conf PGSYSCONFDIR=/nonexistent/babylon-pg-service.d \
      mise run db:bootstrap || status=$?
  fi
  if [ "$status" -eq 0 ]; then
    observation="$(runtime_observation babylon_test)" || status=$?
    [ "$observation" = "$CLEAN_RUNTIME" ] || { printf 'fresh bootstrap authority mismatch: %s\n' "$observation" >&2; status=1; }
  fi
  if [ "$status" -eq 0 ] && [ "$LIVE_FOCUS" != runtime_smoke ]; then
    create_runtime_template || status=$?
    export BABYLON_RUNTIME_TEMPLATE_DB="$RUNTIME_TEMPLATE"
  fi
fi

if [ "$status" -eq 0 ]; then
  cd "$REPO_ROOT/rust"
  case "$LIVE_FOCUS" in
    runtime_smoke)
      cd "$REPO_ROOT"
      run_phase michigan_rollover 600 env BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN" mise run qa:michigan-rollover-smoke || status=$?
      ;;
    reference_integrity)
      run_phase reference_integrity 900 cargo test -p babylon-persistence --lib \
        current_schema::live_tests:: --locked -- --nocapture --ignored --test-threads=1 || status=$?
      if [ "$status" -eq 0 ]; then
        run_phase reference_catalog 600 cargo test -p babylon-persistence --test reference_integrity \
          --locked -- --nocapture --ignored --test-threads=1 || status=$?
      fi
      ;;
    runtime)
      run_phase runtime 600 cargo test -p babylon-persistence --lib \
        runtime::live_tests::live_ --locked -- --nocapture --ignored --test-threads=1 || status=$?
      if [ "$status" -eq 0 ]; then
        run_phase material_writer_bounds 180 env BABYLON_RUNTIME_DSN="$BOOTSTRAP_DSN" \
          cargo test -p babylon-persistence --lib \
          material_runtime::writer_bounds_tests::live_bounded_writer_verifies_authority_and_timeouts_in_read_only_transaction \
          --locked -- --nocapture --ignored --exact --test-threads=1 || status=$?
      fi
      ;;
    archive)
      # Current material fixtures commit real ticks. Bound each independent
      # acceptance group so one slow group cannot hide an unfinished later one.
      run_phase archive_worker 600 cargo test -p babylon-persistence --lib \
        archive_revision::worker::live_tests:: --locked -- --nocapture --ignored \
        --skip ::bounds:: --skip ::revisions:: --skip ::wakeup:: --test-threads=1 || status=$?
      for archive_group in bounds revisions wakeup; do
        [ "$status" -eq 0 ] || break
        run_phase "archive_$archive_group" 600 cargo test -p babylon-persistence --lib \
          "archive_revision::worker::live_tests::$archive_group::" \
          --locked -- --nocapture --ignored --test-threads=1 || status=$?
      done
      for archive_producer in place_producer_live county_producer_live; do
        [ "$status" -eq 0 ] || break
        run_phase "$archive_producer" 600 cargo test -p babylon-persistence \
          --test "$archive_producer" --locked -- --nocapture --ignored --test-threads=1 || status=$?
      done
      ;;
    reader)
      for reader_suite in reader_role_live observer_material_live; do
        # Reader tests share role/environment state; observer tests own independent clones.
        reader_threads=1
        [ "$reader_suite" != observer_material_live ] || reader_threads=4
        run_phase "$reader_suite" 600 cargo test -p babylon-persistence --test "$reader_suite" \
          --locked -- --nocapture --ignored --skip statewide:: --skip statewide_qualified:: --test-threads="$reader_threads" || status=$?
        [ "$status" -eq 0 ] || break
      done
      # The full synthetic campaign gets its own deadline after ordinary readers.
      ;&
    statewide_synthetic)
      if [ "$status" -eq 0 ]; then
        run_phase statewide_synthetic 600 cargo test -p babylon-persistence --test observer_material_live \
          statewide:: --locked -- --nocapture --ignored --test-threads=1 || status=$?
      fi
      ;;
    statewide_qualified)
      # Actual-source four-preset qualification is separate from routine reader checks.
      run_phase statewide_qualified 3600 cargo test -p babylon-persistence --test observer_material_live \
        statewide_qualified:: --locked -- --nocapture --ignored --test-threads=1 || status=$?
      ;;
    client)
      run_phase client 900 cargo test -p babylon-client --test dossier_cli_live \
        --locked -- --nocapture --ignored --test-threads=1 || status=$?
      ;;
  esac
fi

if [ "$RUNTIME_TEMPLATE_CREATED" -eq 1 ]; then
  template_status=0
  verify_runtime_template_and_clone_cleanup || template_status=$?
  drop_runtime_template || template_status=$?
  if [ "$status" -eq 0 ] && [ "$template_status" -ne 0 ]; then status="$template_status"; fi
fi
[ "$status" -eq 0 ] || emit_runtime_logs
cleanup_checked
trap - EXIT INT TERM HUP
printf 'Rust PostgreSQL cleanup verified: container=%s volume=%s elapsed_seconds=%s status=%s\n' \
  "$CONTAINER" "$VOLUME" "$SECONDS" "$status"
exit "$status"
