#!/usr/bin/env bash
# Data-pathway doctor — loudly validates that the data trove and the Postgres
# bind mount actually live on the encrypted data drive, catching the silent
# displacement class of failure (2026-07-13: the drive remounted at
# /media/user/data1, every data/* symlink dangled, and docker initdb'd a shadow
# Postgres cluster onto the root filesystem without a single error anywhere —
# see reports/incident-2026-07-13-data-mount-displacement.md).
#
# Read-only. Exit 0 = healthy (or not the dev box — CI machines have no data
# drive, so absence of the LUKS mapper is a clean skip, not a failure).
# Exit 1 = the pathway is broken; the failure lines say exactly where.
# Heal with: sudo tools/heal_data_mount.sh
set -uo pipefail

MAPPER="${BABYLON_DATA_MAPPER:-/dev/mapper/luks-1b2ddee8-005d-4efd-9f82-482168bde87c}"
EXPECTED_MOUNT="${BABYLON_DATA_MOUNT:-/media/user/data}"
PG_CONTAINER="${BABYLON_PG_CONTAINER:-babylon-pg-isolated}"
REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"

fail=0
bad() { printf 'DATA-DOCTOR FAIL: %s\n' "$*" >&2; fail=1; }
note() { printf 'data-doctor: %s\n' "$*"; }

# Not the dev box (no unlocked data drive) -> clean skip so CI never reds here.
if [ ! -e "$MAPPER" ]; then
  note "mapper $MAPPER absent — not the dev box (or drive locked); skipping."
  exit 0
fi

# 1. The drive must be mounted, and mounted at the expected place.
actual_target="$(findmnt -no TARGET "$MAPPER" 2>/dev/null || true)"
if [ -z "$actual_target" ]; then
  bad "data drive $MAPPER is unlocked but NOT MOUNTED anywhere"
elif [ "$actual_target" != "$EXPECTED_MOUNT" ]; then
  bad "data drive is mounted at '$actual_target', expected '$EXPECTED_MOUNT' — the udisks suffix fallback struck again (heal: sudo tools/heal_data_mount.sh)"
fi

# 2. The trove layout must be reachable through the expected mountpoint.
if [ ! -f "$EXPECTED_MOUNT/babylon-data/sqlite/marxist-data-3NF.sqlite" ]; then
  bad "reference DB not found at $EXPECTED_MOUNT/babylon-data/sqlite/marxist-data-3NF.sqlite"
fi

# 3. The repo's data/ symlink farm must resolve.
if ! readlink -e "$REPO_ROOT/data/sqlite/marxist-data-3NF.sqlite" >/dev/null 2>&1; then
  bad "repo symlink $REPO_ROOT/data/sqlite dangles (reference-DB tests will FAIL, not skip)"
fi

# 3b. data/sqlite must BE a symlink into the trove — a real dir here means a
#     test run auto-created an empty stub DB (sqlite creates-on-connect) and
#     every reference-DB test fails with "no such table". readlink -e alone
#     passes on such a stub, which is exactly how this slipped past on
#     2026-07-15 in a fresh worktree (symlink farm is untracked local state).
if [ -e "$REPO_ROOT/data/sqlite" ] && [ ! -L "$REPO_ROOT/data/sqlite" ]; then
  bad "repo data/sqlite is a real dir/file, not a trove symlink — likely an auto-created stub DB; fix: rm -r data/sqlite && ln -s $EXPECTED_MOUNT/babylon-data/sqlite data/sqlite"
fi

# 4. The Postgres bind source must sit on the data drive — NOT the root fs.
#    (The exact silent failure mode of 2026-07-13: docker auto-creates a missing
#    bind path and Postgres initdb's a divergent shadow cluster, zero errors.)
if command -v docker >/dev/null 2>&1 && docker ps --format '{{.Names}}' 2>/dev/null | grep -qx "$PG_CONTAINER"; then
  bind_src="$(docker inspect "$PG_CONTAINER" --format \
    '{{range .Mounts}}{{if eq .Destination "/var/lib/postgresql/data"}}{{.Source}}{{end}}{{end}}' 2>/dev/null || true)"
  if [ -n "$bind_src" ]; then
    bind_dev="$(findmnt -no SOURCE -T "$bind_src" 2>/dev/null || true)"
    if [ "$bind_dev" != "$MAPPER" ]; then
      bad "Postgres bind source '$bind_src' lives on '$bind_dev', not the data drive — a SHADOW cluster is accumulating on the wrong filesystem"
    fi
  else
    note "container $PG_CONTAINER has no /var/lib/postgresql/data bind (named volume?) — skipping bind check."
  fi
else
  note "container $PG_CONTAINER not running — bind check skipped."
fi

if [ "$fail" -eq 0 ]; then
  note "healthy — drive at $EXPECTED_MOUNT, trove + symlinks resolve, Postgres on the drive."
fi
exit "$fail"
