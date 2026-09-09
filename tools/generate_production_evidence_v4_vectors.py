#!/usr/bin/env python3
"""Generate bounded independent V4 presentation wire vectors with stdlib only.

This tool encodes fixed fixtures, not arbitrary snapshots or economic rules.
Run with --write to replace the fixture; the default only checks exact bytes.
V3's frozen bytes are historical evidence and have no encoder here.
"""

from __future__ import annotations

import argparse
import hashlib
import json
import struct
from dataclasses import asdict, dataclass, replace
from pathlib import Path

DOMAIN = b"babylon.production-observation-evidence.v4\0"
DESTINATION = Path(__file__).resolve().parents[1] / "contracts/fixtures/production_evidence_v4.json"


def number(value: int) -> bytes:
    """Encode an exact unsigned 64-bit value; struct rejects overflow."""
    return struct.pack(">Q", value)


def text(value: str) -> bytes:
    encoded = value.encode("utf-8")
    return number(len(encoded)) + encoded


def optional_text(value: str | None) -> bytes:
    return b"\0" if value is None else b"\1" + text(value)


@dataclass(frozen=True, order=True)
class Completed:
    period: int
    opening_employed: int
    opening_reserve: int
    previous_unretained_hours: int
    current_unretained_hours: int
    retained_hours: int
    target_employed: int
    hires: int
    separations: int

    def values(self) -> tuple[int, ...]:
        return (
            self.period,
            self.opening_employed,
            self.opening_reserve,
            self.previous_unretained_hours,
            self.current_unretained_hours,
            self.retained_hours,
            self.target_employed,
            self.hires,
            self.separations,
        )


@dataclass(frozen=True, order=True)
class Subject:
    scenario: str
    local_name: str


@dataclass(frozen=True)
class Account:
    pool_id: str
    site_id: str
    unit_id: str
    subject: Subject
    hours_per_person: int
    labor_force: int
    employed: int
    reserve: int
    previous_unretained_hours: int
    next_opening_period: int
    next_opening_hours: int
    completed: Completed | None

    def identities(self) -> tuple[str, ...]:
        return (
            self.pool_id,
            self.site_id,
            self.unit_id,
            self.subject.scenario,
            self.subject.local_name,
        )

    def values(self) -> tuple[int, ...]:
        return (
            self.hours_per_person,
            self.labor_force,
            self.employed,
            self.reserve,
            self.previous_unretained_hours,
            self.next_opening_period,
            self.next_opening_hours,
        )

    def key(self) -> tuple[tuple[str, ...], tuple[int, ...], tuple[int, ...]]:
        completed = (0,) if self.completed is None else (1, *self.completed.values())
        return self.identities(), self.values(), completed

    def wire(self) -> bytes:
        completed = (
            b"\0"
            if self.completed is None
            else b"\1" + b"".join(number(value) for value in self.completed.values())
        )
        return (
            b"".join(text(value) for value in self.identities())
            + b"".join(number(value) for value in self.values())
            + completed
        )


def completed_material() -> tuple[dict[str, object], bytes]:
    row: dict[str, object] = {
        "site_id": "b",
        "good_id": "g",
        "unit_id": "u",
        "good": "steel\0sheet",
        "unit": "kg",
        "opening": 2,
        "arrivals": 11,
        "produced": 3,
        "consumed": 5,
        "dispatched": 7,
        "closing": 4,
    }
    wire = b"\1" + number(7) + number(1)
    wire += b"".join(text(value) for value in ("b", "g", "u", "steel\0sheet", "kg"))
    wire += b"".join(number(value) for value in (2, 11, 3, 5, 7, 4))
    return {"period": 7, "rows": [row]}, wire


def arrival() -> tuple[dict[str, object], bytes]:
    event: dict[str, object] = {
        "id": "e",
        "period": 7,
        "subject_site_ids": ["b", "a"],
        "kind": "arrival",
        "description": "intact",
        "receipt_digest": "d",
        "delivery_evidence": {
            "stage": "Arrival",
            "order_id": "o",
            "route_id": "r",
            "good_id": "g",
            "unit_id": "u",
            "quantity": 11,
        },
    }
    wire = text("e") + number(7) + number(2) + text("a") + text("b")
    wire += text("arrival") + text("intact") + text("d") + b"\1\1"
    wire += text("o") + text("r") + text("g") + text("u") + number(11)
    return event, wire


def vector(name: str, accounts: list[Account], *, foundation: bool) -> dict[str, object]:
    period = 0 if foundation else 7
    event, event_wire = arrival()
    balance, balance_wire = completed_material()
    snapshot: dict[str, object] = {
        "campaign_id": "c",
        "resolve_tick": period,
        "foundation_digest": "f",
        "tick_content_hash": None if foundation else "t",
        "envelope_digest": None,
        "nominal_world_hash": None if foundation else "w",
        "visibility": "full_observer",
        "counties": [],
        "production": {
            "scenario_label": "s",
            "horizon_period": 16,
            "sites": [],
            "routes": [],
            "freight": [],
            "events": [] if foundation else [event],
            "labor_accounts": [],
            "observed_contexts": [],
            "process_attributions": [],
            "provenance": ["z", "a"],
            "material_balance": None if foundation else balance,
            "staffing_accounts": [asdict(account) for account in accounts],
        },
    }
    wire = DOMAIN + struct.pack(">I", 4) + text("c") + number(period) + text("f")
    wire += optional_text(None if foundation else "t") + optional_text(None)
    wire += optional_text(None if foundation else "w") + b"\0"
    wire += text("s") + number(16) + number(0) * 3  # sites, routes, freight
    wire += number(0) if foundation else number(1) + event_wire
    wire += number(0) * 3  # time accounts, observed contexts, process attributions
    wire += number(2) + text("a") + text("z")
    wire += b"\0" if foundation else balance_wire
    wire += number(len(accounts)) + b"".join(
        account.wire() for account in sorted(accounts, key=Account.key)
    )
    return {
        "name": name,
        "snapshot": snapshot,
        "byte_length": len(wire),
        "canonical_hex": wire.hex(),
        "sha256": hashlib.sha256(wire).hexdigest(),
    }


def fixture_bytes() -> bytes:
    completed = Completed(7, 4, 0, 80, 40, 80, 2, 0, 2)
    account = Account(
        "pool-a",
        "b",
        "labor-hours",
        Subject("fixture/é\0", "workers-a"),
        40,
        4,
        2,
        2,
        40,
        8,
        80,
        completed,
    )
    # Exercise complete-row tie-breaking (None sorts before Some), duplicates,
    # UTF-8 byte lengths, and the codec's full u64 range without economic claims.
    maximum = replace(account, pool_id="pool-z", next_opening_hours=(1 << 64) - 1)
    vectors = [
        vector(
            "completed_multiset",
            [maximum, account, replace(account, completed=None), account],
            foundation=False,
        ),
        vector(
            "foundation",
            [
                replace(
                    account,
                    employed=4,
                    reserve=0,
                    previous_unretained_hours=160,
                    next_opening_period=1,
                    next_opening_hours=160,
                    completed=None,
                )
            ],
            foundation=True,
        ),
    ]
    payload = {
        "schema_version": 4,
        "purpose": "presentation wire evidence, not admitted economic state",
        "vectors": vectors,
    }
    return (json.dumps(payload, ensure_ascii=True, indent=2) + "\n").encode("utf-8")


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--output", type=Path, default=DESTINATION)
    parser.add_argument("--write", action="store_true", help="explicitly regenerate the fixture")
    args = parser.parse_args()
    expected = fixture_bytes()
    if args.write:
        args.output.write_bytes(expected)
    elif not args.output.is_file() or args.output.read_bytes() != expected:
        raise SystemExit(f"V4 wire fixture differs: {args.output}")
    print(f"V4 wire fixture exact: {args.output}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
