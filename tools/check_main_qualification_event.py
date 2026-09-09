#!/usr/bin/env python3
"""Refuse a main qualification run outside its two sanctioned event shapes."""

from __future__ import annotations

import argparse
import sys


class QualificationEventError(RuntimeError):
    """The workflow event cannot supply trusted release-qualification evidence."""


def validate_event(*, event_name: str, ref: str, base_ref: str) -> None:
    """Accept a pull request to main or a manual proof on exact dev."""
    if event_name == "pull_request":
        if base_ref == "main":
            return
        raise QualificationEventError(
            f"pull_request qualification requires base main, found {base_ref or '<empty>'}"
        )
    if event_name == "workflow_dispatch":
        if ref == "refs/heads/dev":
            return
        raise QualificationEventError(
            f"workflow_dispatch qualification requires refs/heads/dev, found {ref or '<empty>'}"
        )
    raise QualificationEventError(f"unsupported qualification event {event_name or '<empty>'}")


def main() -> int:
    """Validate command-line event coordinates."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--event", required=True)
    parser.add_argument("--ref", required=True)
    parser.add_argument("--base-ref", default="")
    args = parser.parse_args()
    try:
        validate_event(event_name=args.event, ref=args.ref, base_ref=args.base_ref)
    except QualificationEventError as error:
        print(f"main-qualification: REFUSED — {error}", file=sys.stderr)
        return 1
    print(f"main-qualification: accepted {args.event} at {args.ref}")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
