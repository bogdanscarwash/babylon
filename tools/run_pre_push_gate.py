#!/usr/bin/env python3
"""Run range-scoped pre-push gates without losing deleted paths."""

from __future__ import annotations

import argparse
import os
import subprocess
import sys
from enum import StrEnum
from pathlib import Path
from typing import Final

REPOSITORY_ROOT: Final = Path(__file__).resolve().parents[1]


class Gate(StrEnum):
    """Pre-push gates with repository-level input ownership."""

    RUST_DEV = "rust-dev-gate"
    BSL_REPO_SENTINELS = "bsl-repo-sentinels"


CLASSIFIER_PATH: Final = "tools/run_pre_push_gate.py"
RUST_GATE_PATHS: Final = frozenset(
    {
        ".github/workflows/ci.yml",
        ".mise.toml",
        ".pre-commit-config.yaml",
        CLASSIFIER_PATH,
        "tools/rust_test_report.py",
    }
)
BSL_RETIRED_AUTHORITY_PATHS: Final = frozenset(
    {
        "src/babylon/contracts/practice_contract_v1.py",
        "src/babylon/contracts/practice_contract_v1_generated.py",
        "src/babylon/contracts/relational_territory_dossier_v1.py",
        "src/babylon/contracts/rtd_v1_generated.py",
        "tools/build_detroit_rtd_control.py",
        "tools/generate_practice_contract_types.py",
        "tools/generate_rtd_v1_types.py",
        "tools/sfs_contract_vectors.py",
    }
)
BSL_SENTINEL_PATHS: Final = BSL_RETIRED_AUTHORITY_PATHS | {CLASSIFIER_PATH}
GATE_COMMANDS: Final = {
    Gate.RUST_DEV: ("mise", "run", "rust:check-no-docs", "--", "--dev"),
    Gate.BSL_REPO_SENTINELS: ("mise", "run", "check:bsl-sentinels"),
}


def changed_paths(
    repository: Path,
    from_ref: str,
    to_ref: str,
) -> frozenset[str]:
    """Return the exact push-range paths, including paths deleted at ``to_ref``."""
    command = ("git", "diff", "--name-only", "--no-ext-diff", "-z")
    failures: list[str] = []
    for separator in ("...", ".."):
        completed = subprocess.run(
            [*command, f"{from_ref}{separator}{to_ref}"],
            cwd=repository,
            check=False,
            capture_output=True,
        )
        if completed.returncode == 0:
            return frozenset(os.fsdecode(path) for path in completed.stdout.split(b"\0") if path)
        failures.append(completed.stderr.decode(errors="replace").strip())
    raise RuntimeError("git diff failed for the pre-push range: " + "; ".join(failures))


def gate_applies(gate: Gate, paths: set[str] | frozenset[str]) -> bool:
    """Return whether ``paths`` can alter ``gate`` or its selection contract."""
    if gate is Gate.RUST_DEV:
        return any(
            path.startswith(("rust/", "contracts/", "content/")) or path in RUST_GATE_PATHS
            for path in paths
        )
    return any(path.startswith("ai/decisions/") or path in BSL_SENTINEL_PATHS for path in paths)


def _push_paths_from_environment() -> frozenset[str] | None:
    from_ref = os.environ.get("PRE_COMMIT_FROM_REF")
    to_ref = os.environ.get("PRE_COMMIT_TO_REF")
    if bool(from_ref) != bool(to_ref):
        raise RuntimeError("pre-commit supplied only one push-range endpoint")
    if from_ref is None or to_ref is None:
        return None
    return changed_paths(REPOSITORY_ROOT, from_ref, to_ref)


def main(argv: list[str] | None = None) -> int:
    """Select and run one gate against pre-commit's exact push refs."""
    parser = argparse.ArgumentParser(description=__doc__)
    parser.add_argument("gate", type=Gate, choices=tuple(Gate))
    args = parser.parse_args(argv)

    try:
        paths = _push_paths_from_environment()
    except RuntimeError as error:
        print(f"{args.gate.value}: {error}", file=sys.stderr)
        return 2

    if paths is not None and not gate_applies(args.gate, paths):
        print(f"{args.gate.value}: push range does not touch owned inputs; skipped")
        return 0

    if paths is None:
        print(f"{args.gate.value}: no exact push refs; running conservatively")
    completed = subprocess.run(
        GATE_COMMANDS[args.gate],
        cwd=REPOSITORY_ROOT,
        check=False,
    )
    return completed.returncode


if __name__ == "__main__":
    raise SystemExit(main())
