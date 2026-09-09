# Incident: data-drive mount displacement + silent Postgres shadow cluster

**Found:** 2026-07-14, while verifying `feature/17-living-engine` (10 unit tests red with
`FileNotFoundError: data/sqlite/marxist-data-3NF.sqlite`).
**Status: ✅ RESOLVED 2026-07-14 ~17:11 EDT** — owner ran `sudo tools/heal_data_mount.sh
--with-fstab`: drive reclaimed at `/media/user/data`, fstab hardened (nofail), shadow preserved
at `/media/user/babylon-pg.shadow-20260714-1710` plus two SQL dumps on the drive; Postgres came
ready after ~95 s of fsync + automatic crash recovery (the script's old 60 s wait declared a
false failure — now 5 min); real lineage verified live (5,102 `tick_commit` rows, newest
2026-07-13 03:35 UTC) and `mise run data:doctor` fully green.
Forensics: read-only 4-agent sweep, evidence inline below.

**OPERATIONALIZED (2026-07-14):** this failure class is now mechanically caught and healed:

- **`mise run data:doctor`** (`tools/data_doctor.sh`, wired into the `check` gate; clean skip on
  boxes without the drive) — reds loudly on all four facets: displaced mount, missing trove,
  dangling repo symlinks, Postgres bind off-drive. Live efficacy proven against this incident.
- **`sudo tools/heal_data_mount.sh [--with-fstab]`** — the runbook below as one verified command:
  dump shadow → stop container → preserve shadow aside → remount at the rightful mountpoint
  (optional fstab hardening so a reboot can never displace it again) → restart → verify lineage.
- A pre-heal `pg_dumpall` of the shadow already sits on the drive:
  `/media/user/data1/babylon-pg-shadow-backup-20260714-1526.sql`.

## What happened

1. During the 2026-07-13 session-kill window (earlyoom killed the X session; see the claude-mem
   leak incident), the LUKS data drive (`sda1` → `luks-1b2ddee8…`, ext4 **LABEL=data**, 3.6 T)
   was not mounted when Docker came up at **09:37:40**. Docker auto-created the missing bind-mount
   path `/media/user/data/babylon-pg` **on the root filesystem**, and Postgres `initdb`'d a fresh,
   empty cluster into it. No error anywhere — a fully silent failure.
2. When the drive was mounted later (21:30), udisks found `/media/user/data` occupied by that
   leftover root-owned directory and fell back to **`/media/user/data1`**. There is **no fstab /
   crypttab entry** for this drive — the mount is purely session-managed, which is why a stale
   directory could steal the name.

## Consequences (current state)

- **Every repo `data/*` symlink dangles** (29 entries: sqlite, bea, qcew, lodes, tiger,
  natural-earth, …) → 10 reference-DB unit tests FAIL + ~40 skip. Loud, harmless, heals with the
  mount. The trove itself is intact at `/media/user/data1/babylon-data/`.
- **Two divergent Postgres lineages:**
  | | real (encrypted drive) | shadow (root fs, LIVE) |
  |---|---|---|
  | path | `/media/user/data1/babylon-pg` | `/media/user/data/babylon-pg` |
  | born / last write | 2026-07-03 / **2026-07-11 10:43** | 2026-07-13 09:37 / **still growing** |
  | contents | ~8 days of real dev history | 81 MB, 10 `tick_commit` rows of Jul 13–14 test churn |

  The running container (`babylon-pg-isolated`, `restart: unless-stopped`) is on the **throwaway**
  lineage; the valuable one is frozen. A naive mount fix would hide the shadow under the mount.
- **Second loaded gun, not yet fired:** `BABYLON_ARCHIVE_ROOT=/media/user/data/babylon-archives`
  (`.env`) resolves through the same broken path; nothing has called `sim:archive` yet.
- Root cause of the bind-mount exposure is the workstation-local `.env`
  (`BABYLON_PG_DATA=/media/user/data/babylon-pg`) — not a repo bug; `.env.example` is clean.

## Runbook (one sitting; sudo steps marked; do NOT reboot first — `restart: unless-stopped`
would just re-create the shadow)

1. `ls -la /media/user/data/` — confirm only `babylon-pg` is present.
2. Insurance dump of the shadow onto the real drive (no sudo):
   `docker exec babylon-pg-isolated pg_dumpall -U test > /media/user/data1/babylon-pg-shadow-backup-$(date +%Y%m%d-%H%M).sql`
3. `docker compose stop babylon-pg` (NOT `down -v`).
4. **[sudo]** `sudo mv /media/user/data/babylon-pg /media/user/data/babylon-pg.shadow-2026-07-13`
5. `udisksctl unmount -b /dev/mapper/luks-1b2ddee8-005d-4efd-9f82-482168bde87c`
   (fallback **[sudo]** `sudo umount /media/user/data1`).
6. `udisksctl mount -b /dev/mapper/luks-1b2ddee8-005d-4efd-9f82-482168bde87c` — must land at
   `/media/user/data` now that the name is free (fallback **[sudo]** explicit `mount` to that path).
   Optional hardening **[sudo]**: add the fstab entry so a reboot can never displace it again.
7. Verify: `ls -la /media/user/data/babylon-pg` (uid 999, mtime 2026-07-11 10:43) and
   `readlink -f data/sqlite` resolves.
8. `docker compose up -d babylon-pg`; `pg_isready`.
9. Confirm the real lineage: `tick_commit` max(created_at) should reflect history through
   2026-07-11 — not the 10-row Jul 13–14 shadow window.
10. Dispose of `babylon-pg.shadow-2026-07-13` + the step-2 dump at leisure.
11. Re-run `mise run check` — the 10 environmental reds must clear (they are mount collateral;
    the branch's code was blast-radius-verified clean).
