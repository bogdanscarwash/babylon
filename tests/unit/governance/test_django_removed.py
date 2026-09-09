"""Executable contract for the PER-258 Django estate removal."""

from __future__ import annotations

import subprocess
import tomllib
from pathlib import Path
from typing import Final

import pytest
from packaging.requirements import Requirement
from packaging.utils import canonicalize_name

pytestmark = pytest.mark.unit

_ROOT: Final[Path] = Path(__file__).resolve().parents[3]
_PYPROJECT: Final[Path] = _ROOT / "pyproject.toml"
_WORKFLOW_ROOT: Final[Path] = _ROOT / ".github" / "workflows"
_ACTION_ROOT: Final[Path] = _ROOT / ".github" / "actions"
_ACTIVE_AUTOMATION: Final[tuple[Path, ...]] = (_ROOT / ".mise.toml",) + tuple(
    sorted((*_WORKFLOW_ROOT.glob("*.y*ml"), *_ACTION_ROOT.rglob("*.y*ml")))
)
_RETIRED_PATHS: Final[tuple[Path, ...]] = (
    _ROOT / "web",
    _ROOT / "tests" / "unit" / "web",
    _ROOT / "tests" / "integration" / "web",
    _ROOT / "tests" / "unit" / "observatory",
    _ROOT / "tests" / "unit" / "test_contract_parity.py",
    _ROOT / "tests" / "scripts" / "quickstart_walkthrough.sh",
    _ROOT / "tests" / "scripts" / "systemd_smoke_test.sh",
    _ROOT / "tests" / "scripts" / "perf_verify_resolve_latency.sh",
)
_RETIRED_DISTRIBUTIONS: Final[frozenset[str]] = frozenset(
    {
        "django",
        "django-cors-headers",
        "django-stubs",
        "djangorestframework",
        "djangorestframework-stubs",
        "gunicorn",
        "pytest-django",
    }
)
_RETIRED_AUTOMATION_TERMS: Final[tuple[str, ...]] = (
    "DJANGO_SETTINGS_MODULE",
    "babylon_web",
    "pytest-django",
    "mypy_django_plugin",
    "--extra server",
    "GeoDjango",
)


def _distribution_name(requirement: str) -> str:
    """Return the canonical distribution name from a valid PEP 508 requirement."""
    return canonicalize_name(Requirement(requirement).name)


def _is_tracked(path: Path) -> bool:
    """Return whether Git tracks the retired file or anything beneath it."""
    relative = path.relative_to(_ROOT)
    result = subprocess.run(  # noqa: S603
        ["git", "ls-files", "--error-unmatch", "--", relative.as_posix()],  # noqa: S607
        cwd=_ROOT,
        capture_output=True,
        text=True,
        check=False,
    )
    if result.returncode not in {0, 1}:
        raise RuntimeError(result.stderr.strip() or "git ls-files failed")
    return result.returncode == 0


@pytest.mark.parametrize(
    ("requirement", "expected"),
    (
        ("Django~=5.2", "django"),
        ("django_cors_headers>=4", "django-cors-headers"),
    ),
)
def test_distribution_name_parses_and_canonicalizes_pep_508(
    requirement: str, expected: str
) -> None:
    assert _distribution_name(requirement) == expected


def test_active_automation_inventory_covers_every_yaml_manifest() -> None:
    expected = (_ROOT / ".mise.toml",) + tuple(
        sorted(
            (*_WORKFLOW_ROOT.glob("*.y*ml"), *_ACTION_ROOT.rglob("*.y*ml")),
        )
    )

    assert expected == _ACTIVE_AUTOMATION


def test_retired_inventory_covers_every_removed_django_script() -> None:
    expected = {
        _ROOT / "tests" / "scripts" / "quickstart_walkthrough.sh",
        _ROOT / "tests" / "scripts" / "systemd_smoke_test.sh",
        _ROOT / "tests" / "scripts" / "perf_verify_resolve_latency.sh",
    }

    assert expected <= set(_RETIRED_PATHS)


def test_retired_django_paths_are_absent() -> None:
    present = [str(path.relative_to(_ROOT)) for path in _RETIRED_PATHS if _is_tracked(path)]
    assert present == [], f"retired Django/web paths remain: {present}"


def test_python_dependencies_exclude_the_django_stack() -> None:
    config = tomllib.loads(_PYPROJECT.read_text(encoding="utf-8"))
    requirements = list(config["project"]["dependencies"])
    for group in config["project"].get("optional-dependencies", {}).values():
        requirements.extend(group)
    for group in config.get("dependency-groups", {}).values():
        requirements.extend(group)
    names = {_distribution_name(requirement) for requirement in requirements}
    assert names.isdisjoint(_RETIRED_DISTRIBUTIONS), names & _RETIRED_DISTRIBUTIONS


def test_python_tooling_has_no_django_bootstrap() -> None:
    config = tomllib.loads(_PYPROJECT.read_text(encoding="utf-8"))
    pytest_config = config["tool"]["pytest"]["ini_options"]
    assert "DJANGO_SETTINGS_MODULE" not in pytest_config
    assert "web" not in pytest_config["pythonpath"]
    mypy_config = config["tool"]["mypy"]
    assert "web" not in mypy_config["mypy_path"].split(":")
    assert "mypy_django_plugin.main" not in mypy_config["plugins"]
    assert "django-stubs" not in config["tool"]


@pytest.mark.parametrize("path", _ACTIVE_AUTOMATION)
def test_active_automation_does_not_bootstrap_django(path: Path) -> None:
    text = path.read_text(encoding="utf-8")
    present = [term for term in _RETIRED_AUTOMATION_TERMS if term in text]
    assert present == [], f"{path.relative_to(_ROOT)} retains Django automation: {present}"
