"""PEP-621 dependency layout and retained operator tooling."""

from __future__ import annotations

import tomllib
from pathlib import Path

DATA = tomllib.loads(Path("pyproject.toml").read_text(encoding="utf-8"))


def _names(specs: list[str]) -> set[str]:
    out: set[str] = set()
    for spec in specs:
        head = spec.split(";")[0].strip()
        for sep in (">", "<", "=", "!", "~", "["):
            head = head.split(sep)[0]
        out.add(head.strip())
    return out


def test_project_dependencies_is_static_list() -> None:
    assert isinstance(DATA["project"]["dependencies"], list)
    assert "dependencies" not in DATA["project"].get("dynamic", [])
    # classifiers went static at the hatchling cutover (2026-07-22):
    # transcribed from the last poetry-core wheel's METADATA
    assert "classifiers" not in DATA["project"].get("dynamic", [])
    assert isinstance(DATA["project"]["classifiers"], list)
    assert "Programming Language :: Python :: 3.12" in DATA["project"]["classifiers"]


def test_no_legacy_poetry_table() -> None:
    assert "poetry" not in DATA.get("tool", {}), "legacy [tool.poetry] table remains"


def test_ops_extra_contains_only_operator_tooling() -> None:
    ops = _names(DATA["project"]["optional-dependencies"]["ops"])
    assert ops == {
        "ansible-dev-tools",
        "rstcheck",
        "doc8",
        "boto3",
    }


def test_retired_web_dependencies_are_absent_from_every_dependency_set() -> None:
    requirements = [
        *DATA["project"]["dependencies"],
        *DATA["project"]["optional-dependencies"]["ops"],
        *DATA["dependency-groups"]["dev"],
        *DATA["dependency-groups"]["docs"],
    ]
    assert _names(requirements).isdisjoint(
        {
            "django",
            "django-cors-headers",
            "django-stubs",
            "djangorestframework",
            "djangorestframework-stubs",
            "gunicorn",
            "pytest-django",
            "pytest-playwright",
        }
    )


def test_ci_xdist_plugin_is_a_direct_dev_dependency() -> None:
    dev = _names(DATA["dependency-groups"]["dev"])
    assert "pytest-xdist" in dev


def test_core_stays_in_default_deps() -> None:
    core = _names(DATA["project"]["dependencies"])
    for pkg in ("pydantic", "typer", "rich", "openai"):
        assert pkg in core
    assert "rustworkx" not in core


def test_build_backend_is_hatchling() -> None:
    # hatchling replaced poetry-core 2026-07-22 (uv completion train); the
    # explicit src-layout mapping lives in [tool.hatch.build.targets.wheel]
    assert DATA["build-system"]["build-backend"] == "hatchling.build"
    assert DATA["tool"]["hatch"]["build"]["targets"]["wheel"]["packages"] == ["src/babylon"]
