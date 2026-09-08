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
        checkout,
        {},
        "3.12.14",
        "/opt/python",
        str(checkout / ".venv"),
        str(checkout / "src/babylon/__init__.py"),
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
    faults = environment_faults(
        checkout, environment, version, base_prefix, str(checkout / ".venv"), origin
    )
    assert any(message in fault for fault in faults)


@pytest.mark.parametrize("prefix", ["/other/.venv", "/opt/python"])
def test_foreign_environment_is_rejected_even_with_checkout_imports(
    checkout: Path, prefix: str
) -> None:
    faults = environment_faults(
        checkout,
        {},
        "3.12.14",
        "/opt/python",
        prefix,
        str(checkout / "src/babylon/__init__.py"),
    )
    assert any("checkout-local .venv" in fault for fault in faults)


def test_symlinked_environment_is_rejected(checkout: Path, tmp_path: Path) -> None:
    foreign = tmp_path / "other-venv"
    foreign.mkdir()
    (checkout / ".venv").symlink_to(foreign, target_is_directory=True)
    faults = environment_faults(
        checkout,
        {},
        "3.12.14",
        "/opt/python",
        str(checkout / ".venv"),
        str(checkout / "src/babylon/__init__.py"),
    )
    assert any("checkout-local .venv" in fault for fault in faults)


def test_symlink_remediation_preserves_the_environment_target(checkout: Path) -> None:
    target = checkout / "valuable-environment"
    target.mkdir()
    (checkout / ".venv").symlink_to(target, target_is_directory=True)
    faults = environment_faults(checkout, {}, "3.12.14", "/opt/python", str(target), None)
    assert any("unlink .venv" in fault and "preserving its target" in fault for fault in faults)
    assert target.is_dir()
    assert (checkout / ".venv").is_symlink()
