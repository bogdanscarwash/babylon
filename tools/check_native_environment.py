#!/usr/bin/env python3
"""Check the native development interpreter and editable import boundary."""

from __future__ import annotations

import importlib.util
import os
import sys
from collections.abc import Mapping
from pathlib import Path


def environment_faults(
    root: Path,
    environment: Mapping[str, str],
    version: str,
    base_prefix: str,
    module_origin: str | None,
) -> list[str]:
    """Report drift without changing the interpreter, dependency lock, or shell."""
    faults: list[str] = []
    pin = (root / ".python-version").read_text(encoding="utf-8").strip()
    if version != pin:
        faults.append(f"Python {version} differs from pin {pin}; run mise install and uv sync")
    if environment.get("PYTHONPATH"):
        faults.append("PYTHONPATH must be unset; use the project's locked editable environment")
    if base_prefix.startswith("/nix/store/"):
        faults.append("Python uses the retired Nix interpreter; recreate the native .venv")
    if "/nix/store/" in environment.get("LD_LIBRARY_PATH", ""):
        faults.append("LD_LIBRARY_PATH still injects Nix libraries; open a native shell")
    expected_origin = root / "src" / "babylon" / "__init__.py"
    if module_origin is None or Path(module_origin).resolve() != expected_origin.resolve():
        faults.append("babylon imports do not resolve to this checkout; run uv sync --frozen")
    return faults


def main() -> int:
    root = Path(__file__).resolve().parents[1]
    spec = importlib.util.find_spec("babylon")
    faults = environment_faults(
        root,
        os.environ,
        ".".join(str(part) for part in sys.version_info[:3]),
        sys.base_prefix,
        None if spec is None else spec.origin,
    )
    for fault in faults:
        print(f"env-contract: {fault}", file=sys.stderr)
    if not faults:
        print("env-contract: native interpreter and editable imports verified")
    return int(bool(faults))


if __name__ == "__main__":
    raise SystemExit(main())
