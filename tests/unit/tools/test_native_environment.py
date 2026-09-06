"""Native development must not inherit an old shell's imports or libraries."""

from pathlib import Path

import pytest
from tools.check_native_environment import environment_faults


@pytest.fixture
def checkout(tmp_path: Path) -> Path:
    (tmp_path / ".python-version").write_text("3.12.14\n", encoding="utf-8")
    return tmp_path


def test_native_editable_environment_is_valid(checkout: Path) -> None:
    assert not environment_faults(
        checkout, {}, "3.12.14", "/opt/python", str(checkout / "src/babylon/__init__.py")
    )


@pytest.mark.parametrize(
    ("environment", "version", "base_prefix", "origin", "message"),
    [
        ({"PYTHONPATH": "/outside"}, "3.12.14", "/opt/python", None, "PYTHONPATH"),
        ({"LD_LIBRARY_PATH": "/nix/store/old/lib"}, "3.12.14", "/opt/python", None, "libraries"),
        ({}, "3.12.14", "/nix/store/old-python", None, "interpreter"),
        ({}, "3.12.13", "/opt/python", None, "differs from pin"),
        ({}, "3.12.14", "/opt/python", "/other/src/babylon/__init__.py", "this checkout"),
    ],
)
def test_environment_drift_is_reported(
    checkout: Path,
    environment: dict[str, str],
    version: str,
    base_prefix: str,
    origin: str | None,
    message: str,
) -> None:
    faults = environment_faults(checkout, environment, version, base_prefix, origin)
    assert any(message in fault for fault in faults)
