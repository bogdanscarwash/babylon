#!/usr/bin/env python3
"""Rebuild the small rail observation from captured official service responses.

No network access occurs. Explicit source and output arguments keep acquisition
and canonical generation separate and leave unrelated artifacts untouched.
"""

from __future__ import annotations

import argparse
from pathlib import Path

from verify_ntad_border_rail_v1 import (
    RailEvidenceError,
    bounded_read,
    build_artifact,
    canonical_bytes,
)


def main() -> int:
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("--response", type=Path, required=True)
    parser.add_argument("--layer", type=Path, required=True)
    parser.add_argument("--output", type=Path, required=True)
    args = parser.parse_args()
    try:
        if args.output.resolve() in (args.response.resolve(), args.layer.resolve()):
            raise RailEvidenceError("output must not overwrite a captured source")
        result = canonical_bytes(
            build_artifact(bounded_read(args.response), bounded_read(args.layer))
        )
        args.output.parent.mkdir(parents=True, exist_ok=True)
        args.output.write_bytes(result)
    except (RailEvidenceError, OSError) as error:
        parser.exit(1, f"NTAD rail build refused: {error}\n")
    return 0


if __name__ == "__main__":
    raise SystemExit(main())
