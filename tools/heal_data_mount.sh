#!/usr/bin/env bash
# One-command heal for the data-mount displacement class of failure
# (reports/incident-2026-07-13-data-mount-displacement.md). Run as root:
#
#   sudo tools/heal_data_mount.sh [--with-fstab]
#
# What it does, in order (each step verified, refuses loudly on surprises):
#   1. dump the shadow Postgres cluster onto the real drive (insurance),
#   2. stop the Postgres container,
#   3. move the root-fs shadow dir aside to /media/user/ (stays visible),
#   4. unmount the drive from its displaced mountpoint,
#   5. mount it at the expected mountpoint (+ optional fstab hardening),
#   6. restart Postgres against the real cluster and verify the lineage.
# The shadow data is never deleted — disposal is the owner's call afterwards.
set -euo pipefail

MAPPER="${BABYLON_DATA_MAPPER:-/dev/mapper/luks-1b2ddee8-005d-4efd-9f82-482168bde87c}"
EXPECTED="${BABYLON_DATA_MOUNT:-/media/user/data}"
PG_CONTAINER="${BABYLON_PG_CONTAINER:-babylon-pg-isolated}"
PG_USER="${BABYLON_PG_USER:-test}"
PG_DB="${BABYLON_PG_DB:-babylon_test}"
REPO_ROOT="$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd)"
WITH_FSTAB=0
[ "${1:-}" = "--with-fstab" ] && WITH_FSTAB=1

die() { printf 'heal_data_mount: %s\n' "$*" >&2; exit 2; }
say() { printf 'heal_data_mount: %s\n' "$*"; }

[ "$(id -u)" -eq 0 ] || die "must run as root: sudo tools/heal_data_mount.sh [--with-fstab]"
[ -e "$MAPPER" ] || die "mapper $MAPPER absent — unlock the drive first (open it in the file manager once)"

actual="$(findmnt -no TARGET "$MAPPER" || true)"
[ -n "$actual" ] || die "drive is unlocked but not mounted anywhere — mount it (anywhere) first"

if [ "$actual" = "$EXPECTED" ]; then
  say "drive already mounted at $EXPECTED — nothing to heal."
  "$REPO_ROOT/tools/data_doctor.sh" || die "doctor still unhappy despite correct mount — investigate its output"
  exit 0
fi
say "drive is at '$actual', reclaiming '$EXPECTED'."
mountpoint -q "$EXPECTED" && die "$EXPECTED is a mountpoint of something else — unexpected, refusing"

ts="$(date +%Y%m%d-%H%M)"

# 1. Insurance dump of the shadow cluster (only possible while it runs).
if docker ps --format '{{.Names}}' | grep -qx "$PG_CONTAINER"; then
  dump="$actual/babylon-pg-shadow-backup-$ts.sql"
  say "dumping shadow cluster to $dump"
  docker exec "$PG_CONTAINER" pg_dumpall -U "$PG_USER" > "$dump" || die "pg_dumpall failed — resolve before touching mounts"
  # 2. Stop the container (docker stop marks it manually-stopped, so
  #    restart:unless-stopped will NOT resurrect it until we docker start).
  say "stopping $PG_CONTAINER"
  docker stop "$PG_CONTAINER" >/dev/null
else
  say "$PG_CONTAINER not running — skipping dump/stop."
fi

# 3. Move the shadow aside to a sibling of the mountpoint (root fs, stays
#    visible after the mount goes over $EXPECTED).
if [ -e "$EXPECTED/babylon-pg" ]; then
  shadow_dest="$(dirname "$EXPECTED")/babylon-pg.shadow-$ts"
  say "preserving shadow cluster at $shadow_dest"
  mv "$EXPECTED/babylon-pg" "$shadow_dest"
fi
leftover="$(find "$EXPECTED" -mindepth 1 -maxdepth 1 2>/dev/null | head -5 || true)"
[ -z "$leftover" ] || die "unexpected content still in $EXPECTED: $leftover — resolve manually, then re-run"

# 4. Unmount from the displaced location.
say "unmounting $actual"
umount "$actual" || die "umount $actual failed (busy?) — close whatever uses it: lsof +D $actual"

# 5. Mount at the rightful, label-derived mountpoint.
mkdir -p "$EXPECTED"
say "mounting $MAPPER at $EXPECTED"
mount "$MAPPER" "$EXPECTED" || die "mount failed"
rmdir "$actual" 2>/dev/null || true

if [ "$WITH_FSTAB" -eq 1 ]; then
  if ! grep -q "[[:space:]]${EXPECTED}[[:space:]]" /etc/fstab; then
    say "hardening: adding fstab entry (nofail — boot never blocks on the locked drive)"
    printf '%s %s ext4 defaults,nofail,x-systemd.device-timeout=10 0 2\n' "$MAPPER" "$EXPECTED" >> /etc/fstab
    systemctl daemon-reload
  else
    say "fstab already has an entry for $EXPECTED — leaving it alone."
  fi
fi

# 6. Bring Postgres back on the REAL cluster and verify the lineage.
if docker ps -a --format '{{.Names}}' | grep -qx "$PG_CONTAINER"; then
  say "starting $PG_CONTAINER on the real cluster"
  docker start "$PG_CONTAINER" >/dev/null
  # Cold start of a large real cluster can fsync the data dir for 90+ s and
  # then run crash recovery (observed 2026-07-14: ~95 s before "ready to
  # accept connections") — allow up to 5 minutes before declaring failure.
  tries=0
  until docker exec "$PG_CONTAINER" pg_isready -U "$PG_USER" -d "$PG_DB" >/dev/null 2>&1; do
    tries=$((tries + 1))
    [ "$tries" -ge 150 ] && die "postgres not ready after 150 checks (~5 min) — inspect: docker logs $PG_CONTAINER"
    sleep 2
  done
  say "postgres ready; tick_commit lineage now reads:"
  docker exec "$PG_CONTAINER" psql -U "$PG_USER" -d "$PG_DB" -tc \
    "SELECT count(*) || ' commits, newest ' || coalesce(max(created_at_utc)::text,'-') FROM tick_commit;" || true
fi

say "running the doctor for the final verdict:"
"$REPO_ROOT/tools/data_doctor.sh"
say "HEALED. Shadow preserved (dir + SQL dump) — dispose when satisfied. Now run: mise run check"
