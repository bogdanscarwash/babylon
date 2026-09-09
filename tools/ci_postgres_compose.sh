#!/usr/bin/env bash
# Apply one runner-sized PostgreSQL Compose contract to start and cleanup.

set -euo pipefail

readonly -a COMPOSE_ARGS=(
  -f docker-compose.yml
  -f docker-compose.ci.yml
)
export BABYLON_PG_DATA=babylon-pg-ci

if (( $# != 1 )); then
  echo "usage: ci_postgres_compose.sh {up|down|config}" >&2
  exit 2
fi

case "$1" in
  up)
    exec docker compose "${COMPOSE_ARGS[@]}" up -d --wait babylon-pg
    ;;
  down)
    exec docker compose "${COMPOSE_ARGS[@]}" down -v
    ;;
  config)
    exec docker compose "${COMPOSE_ARGS[@]}" config
    ;;
  *)
    echo "ci_postgres_compose: unknown command '$1'" >&2
    exit 2
    ;;
esac
