"""The PyPI `uuid` relic is gone from pyproject; stdlib uuid still works (ADR095 D2)."""

from __future__ import annotations

import tomllib
import uuid
from pathlib import Path

from packaging.requirements import Requirement
from packaging.utils import canonicalize_name


def test_pyproject_does_not_declare_uuid_dependency() -> None:
    data = tomllib.loads(Path("pyproject.toml").read_text(encoding="utf-8"))
    requirements = list(data["project"]["dependencies"])
    for extra in data["project"].get("optional-dependencies", {}).values():
        requirements.extend(extra)
    for group in data.get("dependency-groups", {}).values():
        # include-group entries are not requirements; every group is inspected here.
        requirements.extend(requirement for requirement in group if isinstance(requirement, str))
    names = {canonicalize_name(Requirement(requirement).name) for requirement in requirements}
    assert "uuid" not in names, "PyPI uuid relic still declared — it shadows the stdlib module"


def test_stdlib_uuid_still_functions() -> None:
    value = uuid.uuid4()
    assert isinstance(value, uuid.UUID)
