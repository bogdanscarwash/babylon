"""Derive the committed MIT Election Lab county Republican-vote-share artifact.

Authoring-time tool (ADR049 ratification / P25 U6, ADR132): reads the raw MIT
Election Data and Science Lab "County Presidential Election Returns 2000-2024"
CSV from the babylon-data drive, validates and repairs its county FIPS keys
against the committed census atom (``src/babylon/data/game/
us_county_territories.json``), and writes a small deterministic in-repo CSV
(ADR076 Tier-1: reviewable diff, no LFS) that ``compute_seed_influences``
consumes for the FR-039 FAC_RESTORATIONIST electoral component.

**NEVER invoked by tests or CI** — the CI-no-drive rule (ADR076 owner ruling
2026-07-14) forbids that; like ``make_lodes_tri_county_artifact.py``, this
script runs once on a dev box with the drive mounted, and only its committed
output participates in builds. The matching ``data-artifacts.yaml`` entry is
hand-maintained (the documented EXCEPTION class: no backing sqlite table), and
``tests/unit/tools/test_election_lab_artifact_manifest.py`` is the
regeneration-safety tripwire.

Acquisition provenance (recorded 2026-07-22): the canonical Harvard Dataverse
file (doi:10.7910/DVN/VOQCHQ, dataset v20, file ``countypres_2000-2024.tab``
id 13573089) is guestbook-gated against anonymous API download; the raw CSV
was acquired from a public CC0 mirror and verified by (a) exact MIT schema
match, (b) spot-checks of 11 county-year results against independently known
official returns, (c) the FIPS validation pass below, whose repair/exclusion
census is printed on every run. Raw snapshot:
``/media/user/data/babylon-data/elections/mit_countypres/
countypres_2000-2024_mirror.csv`` (sha256
``1a2323d8d6ebb77c6593a0403aaec680c17f53a86c1664e74dcd58d8e63c3f5a``).

FIPS repair policy (deterministic, applied per election year):

1. ``(state_po, normalized county name)`` is the PRIMARY KEY, not the file's
   ``county_fips`` column: the acquired file's FIPS column is corrupt at the
   individual-row level in several 2024 state blocks (e.g. Sutter CA's Trump
   row carries ``06101`` while its Harris row carries ``06103``; the Arizona
   block is shifted one county slot from La Paz/04012 onward), while the
   name↔votes pairing verifies correct against known official returns.
2. Per (state, name) the vote rows are aggregated: when any row carries
   ``mode == "TOTAL VOTES"`` only those rows are summed (states that report
   per-mode splits alongside a total), otherwise all mode rows are summed.
3. Groups with no 5-digit-numeric raw FIPS at all are EXCLUDED before the
   name join (Rhode Island 10-digit municipality codes — three RI town names
   coincide with county names and would otherwise false-match). Remaining
   groups join (state, name) against the census atom; the atom's FIPS is the
   emitted key; no-match groups are EXCLUDED and counted (Alaska state-house
   districts, the MIT ``36000`` Kansas City sentinel row).
4. Duplicate final FIPS is a loud failure, never a merge. The validation
   census (correct/repaired/excluded counts) prints on every run.

Known upstream caveat (documented, not repaired): MIT reports Kansas City,
MO votes under the ``36000`` sentinel, so Jackson/Clay/Platte/Cass MO county
rows undercount their KC portions.

Usage::

    uv run python tools/make_election_lab_artifact.py            # write
    uv run python tools/make_election_lab_artifact.py --check    # verify only
"""

from __future__ import annotations

import argparse
import csv
import hashlib
import json
import sys
import unicodedata
from collections import defaultdict
from pathlib import Path
from typing import NamedTuple

_REPO_ROOT = Path(__file__).resolve().parents[1]

RAW_CSV = Path(
    "/media/user/data/babylon-data/elections/mit_countypres/countypres_2000-2024_mirror.csv"
)
ATOM_JSON = _REPO_ROOT / "src/babylon/data/game/us_county_territories.json"
ARTIFACT_HOME = _REPO_ROOT / "src/babylon/data/reference/election/mit_countypres_rep_share.csv"

ELECTION_YEAR = 2024
OFFICE = "US PRESIDENT"
REPUBLICAN = "REPUBLICAN"
TOTAL_MODE = "TOTAL VOTES"

HEADER = ["county_fips", "state_po", "county_name", "rep_votes", "total_votes", "rep_vote_share"]


def _norm_fips(raw: str) -> str:
    """Normalize the raw file's float-formatted FIPS ('1001.0' -> '01001')."""
    raw = raw.strip()
    if raw.endswith(".0"):
        raw = raw[:-2]
    return raw.zfill(5) if raw else ""


def _norm_name(name: str) -> str:
    """Space/diacritic/punctuation-insensitive county-name key.

    Strips only the COUNTY/PARISH suffix words — the CITY token is kept so
    independent cities (Richmond city vs Richmond County) stay distinct.
    """
    decomposed = unicodedata.normalize("NFKD", name)
    ascii_name = decomposed.encode("ascii", "ignore").decode("ascii")
    upper = ascii_name.upper()
    cleaned = "".join(ch if ch.isalnum() or ch.isspace() else " " for ch in upper)
    tokens = [t for t in cleaned.split() if t]
    if tokens and tokens[-1] in {"COUNTY", "PARISH"}:
        tokens = tokens[:-1]
    return "".join(tokens)


# The census atom's 35013 name is mojibake ("Do?a Ana" — a literal '?'
# replacement character from its own generation pass), so its normalized key
# can never match MIT's "DONA ANA". Alias the mangled key to the real one
# (flagged for a future atom data-hygiene pass; fixing the atom is guarded by
# its content_hash and out of this tool's scope).
_ATOM_KEY_ALIASES: dict[tuple[str, str], tuple[str, str]] = {
    ("NM", "DOAANA"): ("NM", "DONAANA"),
}


def _load_atom() -> tuple[dict[tuple[str, str], str], dict[str, str]]:
    """Load the census atom as (state, name-key) -> fips and fips -> name."""
    payload = json.loads(ATOM_JSON.read_text(encoding="utf-8"))
    by_key: dict[tuple[str, str], str] = {}
    name_by_fips: dict[str, str] = {}
    for county in payload["counties"]:
        key = (county["state_abbrev"], _norm_name(county["county_name"]))
        key = _ATOM_KEY_ALIASES.get(key, key)
        if key in by_key:
            raise SystemExit(f"census atom name-key collision: {key}")
        by_key[key] = county["fips"]
        name_by_fips[county["fips"]] = county["county_name"]
    return by_key, name_by_fips


class _CountyVotes(NamedTuple):
    """One (state, county) group's aggregated returns for a single year."""

    raw_fips_set: frozenset[str]
    county_name: str
    rep_votes: int
    total_votes: int


def _aggregate_year(year: int) -> dict[tuple[str, str], _CountyVotes]:
    """Aggregate candidatevotes per (state_po, name-key) for one year."""
    groups: dict[tuple[str, str], list[dict[str, str]]] = defaultdict(list)
    with RAW_CSV.open(newline="", encoding="utf-8") as handle:
        for row in csv.DictReader(handle):
            if row["year"] != str(year) or row["office"] != OFFICE:
                continue
            groups[(row["state_po"], _norm_name(row["county_name"]))].append(row)

    aggregated: dict[tuple[str, str], _CountyVotes] = {}
    for key, rows in groups.items():
        has_total = any(r["mode"] == TOTAL_MODE for r in rows)
        use = [r for r in rows if r["mode"] == TOTAL_MODE] if has_total else rows
        rep = sum(int(float(r["candidatevotes"] or 0)) for r in use if r["party"] == REPUBLICAN)
        total = sum(int(float(r["candidatevotes"] or 0)) for r in use)
        aggregated[key] = _CountyVotes(
            raw_fips_set=frozenset(_norm_fips(r["county_fips"]) for r in rows),
            county_name=rows[0]["county_name"],
            rep_votes=rep,
            total_votes=total,
        )
    return aggregated


def build_rows() -> tuple[list[list[str]], dict[str, int]]:
    """Build the artifact rows plus the validation census."""
    atom_by_key, name_by_fips = _load_atom()
    aggregated = _aggregate_year(ELECTION_YEAR)

    kept: dict[str, list[str]] = {}
    census = {
        "fips_correct": 0,
        "fips_repaired": 0,
        "excluded_non_county": 0,
        "excluded_unmatched": 0,
    }
    for (state_po, name_key), agg in sorted(aggregated.items()):
        raw_fips_set = agg.raw_fips_set
        county_grain = any(len(f) == 5 and f.isdigit() for f in raw_fips_set)
        if not county_grain:
            # Municipality/district rows (RI 10-digit place codes) are never
            # county data, even when a town name coincides with a county name
            # (Bristol/Newport/Providence RI towns vs their namesake counties).
            census["excluded_non_county"] += 1
            continue
        expected = atom_by_key.get((state_po, name_key))
        if expected is None:
            census["excluded_unmatched"] += 1
            continue
        if raw_fips_set == frozenset({expected}):
            census["fips_correct"] += 1
        else:
            census["fips_repaired"] += 1
        if expected in kept:
            raise SystemExit(f"duplicate final FIPS {expected} ({state_po} {name_key})")
        rep = agg.rep_votes
        total = agg.total_votes
        if total <= 0:
            raise SystemExit(f"non-positive total votes for {expected}")
        share = f"{rep / total:.6f}"
        kept[expected] = [expected, state_po, name_by_fips[expected], str(rep), str(total), share]

    rows = [kept[fips] for fips in sorted(kept)]
    return rows, census


def render(rows: list[list[str]]) -> str:
    """Render the deterministic CSV text (sorted rows, LF endings)."""
    lines = [",".join(HEADER)]
    for row in rows:
        name = row[2]
        cell = f'"{name}"' if ("," in name) else name
        lines.append(",".join([row[0], row[1], cell, row[3], row[4], row[5]]))
    return "\n".join(lines) + "\n"


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--check", action="store_true", help="verify committed artifact only")
    args = parser.parse_args()

    if not RAW_CSV.exists():
        raise SystemExit(
            f"raw MIT CSV not found at {RAW_CSV} — this tool runs only on a dev box "
            "with the babylon-data drive mounted (never in CI)"
        )

    rows, census = build_rows()
    text = render(rows)
    digest = hashlib.sha256(text.encode("utf-8")).hexdigest()
    print(f"rows={len(rows)} sha256={digest}")
    print(f"validation census: {census}")

    if args.check:
        committed = ARTIFACT_HOME.read_text(encoding="utf-8")
        if committed != text:
            print("MISMATCH: committed artifact differs from regeneration", file=sys.stderr)
            return 1
        print("committed artifact verified byte-identical")
        return 0

    ARTIFACT_HOME.parent.mkdir(parents=True, exist_ok=True)
    ARTIFACT_HOME.write_text(text, encoding="utf-8")
    print(f"wrote {ARTIFACT_HOME.relative_to(_REPO_ROOT)}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
