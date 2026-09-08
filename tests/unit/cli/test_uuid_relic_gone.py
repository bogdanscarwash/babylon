"""The PyPI `uuid` relic is gone from pyproject; stdlib uuid still works (ADR095 D2)."""

from __future__ import annotations

import tomllib
import uuid
from pathlib import Path


def test_pyproject_does_not_declare_uuid_dependency() -> None:
    data = tomllib.loads(Path("pyproject.toml").read_text(encoding="utf-8"))
    project_deps = data["project"]["dependencies"]
    names = {d.split(">")[0].split("<")[0].split("=")[0].strip() for d in project_deps}
    assert "uuid" not in names, "PyPI uuid relic still declared — it shadows the stdlib module"


def test_stdlib_uuid_still_functions() -> None:
    value = uuid.uuid4()
    assert isinstance(value, uuid.UUID)
