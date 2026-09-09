#!/usr/bin/env python3
"""Phase 0-D declared acquisition — fetch, verify, pin (ruling 18).

The charter (``docs/superpowers/specs/2026-07-30-phase0d-spatial-data-charter.md``)
forbids ad-hoc downloads: every input enters through THIS manifest —
pinned URL + sha256 per file. The maiden run computes and PINS the shas
(``--pin``); every later run VERIFIES against the pinned manifest and
refuses a mismatch loudly (a changed upstream file is a declared
re-acquisition, never a silent drift).

Inputs (the 83 bridge counties = Michigan, FIPS 26):

- TIGER/Line 2023 ``AREAWATER`` per county (the water polygons the land
  mask subtracts) — 83 files.
- TIGER/Line 2023 block groups, statewide (``tl_2023_26_bg.zip``) — the
  population-apportionment geometry.
- Census 2020 DHC P1 population at block-group grain — fetched separately
  via the API in the builder (recorded there; no bulk file).
- LODES8 Michigan WAC ``S000 JT00 2020`` + the geography crosswalk — the
  workplace-density share key at block grain.

Downloads land in the babylon-data trove (never the repo): the fetch is a
LOCAL ceremony; CI consumes the built artifacts only (standing rule).

Usage::

    uv run python tools/phase0d/fetch.py --pin      # maiden: download + write shas
    uv run python tools/phase0d/fetch.py --verify   # later: re-verify everything
"""

from __future__ import annotations

import argparse
import hashlib
import json
import sqlite3
import sys
import time
import urllib.error
import urllib.request
from pathlib import Path

REPO_ROOT = Path(__file__).resolve().parents[2]
SQLITE = REPO_ROOT / "data" / "sqlite" / "marxist-data-3NF.sqlite"
MANIFEST = Path(__file__).resolve().parent / "fetch_manifest.json"
TROVE = Path("/media/user/data/babylon-data")

_TIGER = "https://www2.census.gov/geo/tiger/TIGER2023"
_LODES = "https://lehd.ces.census.gov/data/lodes/LODES8/mi"

#: Fixed upper bound on manifest entries (83 counties + statewide files).
MAX_ENTRIES = 100


def bridge_county_fips() -> list[str]:
    """The 83 res-7 bridge counties, from the reference DB (never hardcoded)."""
    conn = sqlite3.connect(SQLITE)
    try:
        rows = conn.execute(
            """
            SELECT DISTINCT c.fips FROM dim_county c
            JOIN bridge_county_h3 b ON b.county_id = c.county_id
            WHERE b.resolution = 7 ORDER BY c.fips
            """
        ).fetchall()
    finally:
        conn.close()
    return [str(r[0]) for r in rows]


def planned_entries() -> list[dict[str, str]]:
    """The declared acquisition set: (url, trove-relative destination)."""
    entries: list[dict[str, str]] = [
        {
            "url": f"{_TIGER}/BG/tl_2023_26_bg.zip",
            "dest": "tiger/bg/tl_2023_26_bg.zip",
        },
        {
            "url": f"{_LODES}/wac/mi_wac_S000_JT00_2020.csv.gz",
            "dest": "lodes/mi_wac_S000_JT00_2020.csv.gz",
        },
        {
            "url": f"{_LODES}/mi_xwalk.csv.gz",
            "dest": "lodes/mi_xwalk.csv.gz",
        },
        {
            # Census 2020 P.L. 94-171 Michigan (KEYLESS bulk — the API's
            # block-group route returns a Missing Key page, a paid-for
            # lesson). The geo file carries SUMLEV=750 block rows with
            # POP100 + INTPTLAT/INTPTLON: population AND the internal
            # point, no segment join, block grain (finer than the charter's
            # bg plan — a method upgrade, not a deviation).
            "url": (
                "https://www2.census.gov/programs-surveys/decennial/2020/data/"
                "01-Redistricting_File--PL_94-171/Michigan/mi2020.pl.zip"
            ),
            "dest": "census/mi2020.pl.zip",
        },
    ]
    entries.extend(
        {
            "url": f"{_TIGER}/AREAWATER/tl_2023_{fips}_areawater.zip",
            "dest": f"tiger/areawater/tl_2023_{fips}_areawater.zip",
        }
        for fips in bridge_county_fips()
    )
    if len(entries) > MAX_ENTRIES:  # loop-bound sanity (Power-of-10 rule 2)
        msg = f"manifest grew past {MAX_ENTRIES} entries — re-declare the bound"
        raise RuntimeError(msg)
    return entries


def _sha256(path: Path) -> str:
    digest = hashlib.sha256()
    with path.open("rb") as fh:
        for chunk in iter(lambda: fh.read(1 << 20), b""):
            digest.update(chunk)
    return digest.hexdigest()


#: Census-server politeness: pause between downloads; bounded retries with
#: backoff on 429/5xx (the maiden run WAS rate-limited at 429 — a paid-for
#: lesson; a persistent failure still fails loudly after the bound).
_INTER_FETCH_SLEEP_S = 3
_RETRY_BACKOFFS_S = (30, 90, 180)


def _download(url: str, dest: Path) -> None:
    dest.parent.mkdir(parents=True, exist_ok=True)
    print(f"  fetch {url}")
    attempts = len(_RETRY_BACKOFFS_S) + 1
    for attempt in range(attempts):  # loop bound: attempts
        try:
            with urllib.request.urlopen(url, timeout=120) as resp, dest.open("wb") as out:  # noqa: S310 - pinned census hosts only
                while True:
                    chunk = resp.read(1 << 20)
                    if not chunk:
                        break
                    out.write(chunk)
            time.sleep(_INTER_FETCH_SLEEP_S)
            return
        except urllib.error.HTTPError as exc:
            retryable = exc.code == 429 or exc.code >= 500
            if not retryable or attempt == attempts - 1:
                raise
            wait = _RETRY_BACKOFFS_S[attempt]
            print(f"    HTTP {exc.code}; backing off {wait}s (attempt {attempt + 1}/{attempts})")
            time.sleep(wait)


def run(*, pin: bool) -> int:
    """Fetch-and-pin (maiden) or verify (steady state). Returns exit code."""
    pinned: dict[str, str] = {}
    if MANIFEST.exists():
        pinned = {e["dest"]: e["sha256"] for e in json.loads(MANIFEST.read_text())["entries"]}
    entries = planned_entries()
    out_entries: list[dict[str, str]] = []
    failures = 0
    for entry in entries:  # loop bound: MAX_ENTRIES
        dest = TROVE / entry["dest"]
        if not dest.exists():
            if not pin:
                print(f"MISSING (run --pin to acquire): {dest}")
                failures += 1
                continue
            _download(entry["url"], dest)
        digest = _sha256(dest)
        expected = pinned.get(entry["dest"])
        if expected is not None and expected != digest:
            print(f"SHA MISMATCH {entry['dest']}: pinned {expected} != actual {digest}")
            failures += 1
            continue
        out_entries.append({**entry, "sha256": digest})
    if failures:
        print(f"{failures} failure(s) — nothing pinned")
        return 1
    if pin:
        MANIFEST.write_text(
            json.dumps(
                {"tiger_vintage": "2023", "lodes": "LODES8", "entries": out_entries}, indent=2
            )
            + "\n"
        )
        print(f"pinned {len(out_entries)} entries -> {MANIFEST}")
    else:
        print(f"verified {len(out_entries)} entries against the pinned manifest")
    return 0


def main(argv: list[str] | None = None) -> int:
    parser = argparse.ArgumentParser(description="Phase 0-D declared acquisition")
    mode = parser.add_mutually_exclusive_group(required=True)
    mode.add_argument("--pin", action="store_true", help="maiden fetch: download + pin shas")
    mode.add_argument("--verify", action="store_true", help="verify the trove against the manifest")
    args = parser.parse_args(argv)
    return run(pin=args.pin)


if __name__ == "__main__":
    sys.exit(main())
