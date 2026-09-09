"""Tests for ``get_reference_session``'s absence guard (task #64).

Founding incident (2026-07-20, G1 nightly): ``SubstrateSystem``'s lattice
build called ``get_reference_session()`` on a runner with no reference
database. ``sqlite3.connect()``/SQLAlchemy's
``create_engine("sqlite:///path")`` both silently CREATE an empty file when
the path is missing -- the runner got a lattice build against a brand-new,
table-less SQLite file instead of a loud "database not found", and the real
failure only surfaced two systems later as a baffling
``no such table: dim_county``.

These tests pin the RED->GREEN fix: ``get_reference_session()`` must raise
``FileNotFoundError`` BEFORE ever opening a connection, and must never create
the missing file as a side effect. Every test that needs to prove the file
was (or was not) touched forces a real connection with a trivial
``SELECT 1`` -- ``create_engine()``/``sessionmaker()`` are both lazy in
SQLAlchemy and never open the underlying file on their own, so a test that
never executes anything would pass "by accident" whether or not the guard
exists.
"""

from __future__ import annotations

import sqlite3
from pathlib import Path

import pytest
from sqlalchemy import text

import babylon.reference.database as db_module
from babylon.reference.database import get_reference_session

pytestmark = pytest.mark.unit


def _point_at(monkeypatch: pytest.MonkeyPatch, db_path: Path) -> None:
    """Point the module at ``db_path`` and clear its lazily-cached globals.

    ``get_reference_session`` resolves its DB through four module globals
    that are each read/created only ONCE and then cached:
    ``NORMALIZED_DB_PATH`` (read by the absence guard itself, at call time),
    ``NORMALIZED_DATABASE_URL`` (the SQLAlchemy URL ``get_normalized_engine``
    builds ``create_engine`` from), and the ``_normalized_engine`` /
    ``_NormalizedSessionLocal`` memoization slots. All four must move
    together, or a stale cached engine from an earlier test/import would
    silently keep pointing at the OLD path while the guard checks the NEW
    one.

    :param monkeypatch: The test's monkeypatch fixture.
    :param db_path: The path to point the module at.
    """
    monkeypatch.setattr(db_module, "NORMALIZED_DB_PATH", db_path)
    monkeypatch.setattr(db_module, "NORMALIZED_DATABASE_URL", f"sqlite:///{db_path}")
    monkeypatch.setattr(db_module, "_normalized_engine", None)
    monkeypatch.setattr(db_module, "_NormalizedSessionLocal", None)


def test_get_reference_session_raises_loudly_on_absent_db(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """No auto-create: an absent reference DB is a loud ``FileNotFoundError``."""
    missing = tmp_path / "does-not-exist.sqlite"
    _point_at(monkeypatch, missing)

    with (
        pytest.raises(FileNotFoundError, match=r"does-not-exist\.sqlite"),
        get_reference_session() as session,
    ):
        session.execute(text("SELECT 1"))


def test_get_reference_session_error_names_the_documented_remedies(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The error names every documented remedy, not just "it's missing"."""
    missing = tmp_path / "does-not-exist.sqlite"
    _point_at(monkeypatch, missing)

    with pytest.raises(FileNotFoundError) as exc_info, get_reference_session() as session:
        session.execute(text("SELECT 1"))

    message = str(exc_info.value)
    assert "mise run data:build-db" in message
    assert "fetch-reference-db" in message
    assert "BABYLON_NORMALIZED_DB_PATH" in message


def test_get_reference_session_does_not_create_the_missing_file(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """The auto-create IS the bug: the guard must fire before any sqlite touch."""
    missing = tmp_path / "does-not-exist.sqlite"
    _point_at(monkeypatch, missing)

    with pytest.raises(FileNotFoundError), get_reference_session() as session:
        session.execute(text("SELECT 1"))

    assert not missing.exists(), (
        "get_reference_session() created the missing DB file as a side effect -- "
        "the guard must run BEFORE any sqlite3/create_engine call"
    )


def test_get_reference_session_opens_fine_when_db_present(
    tmp_path: Path, monkeypatch: pytest.MonkeyPatch
) -> None:
    """A real (even minimal) DB file at the resolved path still opens normally."""
    present = tmp_path / "present.sqlite"
    conn = sqlite3.connect(present)
    conn.execute("CREATE TABLE dim_county (fips TEXT PRIMARY KEY)")
    conn.commit()
    conn.close()

    _point_at(monkeypatch, present)

    with get_reference_session() as session:
        result = session.execute(text("SELECT COUNT(*) FROM dim_county")).scalar()
        assert result == 0
