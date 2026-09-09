"""Reference database configuration (immutable federal statistical data).

The reference database contains normalized 3NF federal statistical data
for initializing simulation state. This data is treated as immutable
after initial load - loaders write once, simulation reads only.

Architecture (ADR030 - Unified SQLite Runtime):
    - ETL/Loading: SQLite (reliable UPSERT, idempotent operations)
    - Reference Data: SQLite (census, QCEW, geography)
    - Simulation State: SQLite (tick-keyed temporal tables per ADR031)

Located at data/sqlite/marxist-data-3NF.sqlite by default. Override with
BABYLON_NORMALIZED_DB_PATH to target an alternate build database.

SQLite was chosen for the entire stack because:
- Reliable on_conflict_do_update (UPSERT) support via SQLAlchemy
- Mature, battle-tested SQLAlchemy dialect
- Predictable constraint enforcement within transactions
- Idempotent loader operations without duplicate key violations
- Single file deployment (no additional database servers)
"""

import os
from collections.abc import Iterator
from contextlib import contextmanager
from pathlib import Path

from sqlalchemy import Engine, create_engine, event
from sqlalchemy.orm import DeclarativeBase, Session, sessionmaker

# Normalized database path - 3NF structure for analytical queries
# From src/babylon/reference/database.py -> repo root is 4 parents up
_REPO_ROOT = Path(__file__).parent.parent.parent.parent

# Legacy source database path (for reference during migration)
SOURCE_DB_PATH = _REPO_ROOT / "data" / "sqlite" / "research.sqlite"
SOURCE_DATABASE_URL = f"sqlite:///{SOURCE_DB_PATH}"


def _resolve_db_path(env_var: str, default_path: Path) -> Path:
    env_value = os.getenv(env_var)
    if not env_value:
        return default_path

    env_path = Path(env_value).expanduser()
    if not env_path.is_absolute():
        env_path = _REPO_ROOT / env_path

    return env_path


NORMALIZED_DB_PATH = _resolve_db_path(
    "BABYLON_NORMALIZED_DB_PATH",
    _REPO_ROOT / "data" / "sqlite" / "marxist-data-3NF.sqlite",
)
NORMALIZED_DATABASE_URL = f"sqlite:///{NORMALIZED_DB_PATH}"


class NormalizedBase(DeclarativeBase):
    """Base class for all normalized 3NF SQLAlchemy models."""

    pass


def get_normalized_engine(echo: bool = False) -> Engine:
    """Create normalized SQLite database engine with FK enforcement and perf PRAGMAs.

    SQLite requires explicit PRAGMAs on every connection. The event listener
    sets:

    * ``foreign_keys = ON`` — FK enforcement is per-connection in SQLite.
    * ``journal_mode = WAL`` — write-ahead logging instead of the default
      DELETE rollback journal. Eliminates the ext4 ``jbd2_log_wait_commit``
      serialization that dominated the spec-067 T036 wallclock (~1:43 hung
      on rollback-journal fsync vs ~8 min via the fast-strategy alternative).
      WAL also allows concurrent readers + a single writer (DELETE mode
      serialized everything). Persistent on the DB file once first set.
    * ``synchronous = NORMAL`` — WAL-safe sync mode that flushes only at
      checkpoints; ``FULL`` would re-introduce per-write fsync stalls and
      is unnecessary for WAL durability (atomicity is preserved either way).
    * ``cache_size = -2097152`` — 2 GiB page cache (negative = KiB).
      Reduces disk reads dramatically for the canonical 520-tick e2e run,
      which re-reads QCEW/BEA reference tables ~166K times.
    * ``mmap_size = 12000000000`` — memory-map up to 12 GB of the DB file
      (more than enough headroom over the current ~8 GB reference DB while
      keeping a safe ceiling on a 32 GB RAM host). The OS page cache
      becomes the de facto cache backend; reads complete without syscall
      overhead.

    Atomicity is preserved (WAL is fully ACID); only the journaling
    mechanism changes. Concurrent readers see a consistent snapshot.

    Args:
        echo: If True, log SQL statements

    Returns:
        Engine: SQLAlchemy engine for the normalized 3NF SQLite database.
    """
    NORMALIZED_DB_PATH.parent.mkdir(parents=True, exist_ok=True)
    engine = create_engine(NORMALIZED_DATABASE_URL, echo=echo)

    @event.listens_for(engine, "connect")
    def set_sqlite_pragma(
        dbapi_conn: object,  # sqlite3.Connection at runtime
        _connection_record: object,
    ) -> None:
        import sqlite3

        if isinstance(dbapi_conn, sqlite3.Connection):
            cursor = dbapi_conn.cursor()
            cursor.execute("PRAGMA foreign_keys=ON")
            cursor.execute("PRAGMA journal_mode=WAL")
            cursor.execute("PRAGMA synchronous=NORMAL")
            cursor.execute("PRAGMA cache_size=-2097152")
            cursor.execute("PRAGMA mmap_size=12000000000")
            cursor.close()

    return engine


def get_source_engine() -> Engine:
    """Create source database engine (research.sqlite).

    Returns:
        Engine: SQLAlchemy engine for research.sqlite
    """
    return create_engine(SOURCE_DATABASE_URL)


# Initialize engines lazily
_normalized_engine: Engine | None = None
_source_engine: Engine | None = None


def normalized_engine() -> Engine:
    """Get or create the normalized database engine."""
    global _normalized_engine
    if _normalized_engine is None:
        _normalized_engine = get_normalized_engine()
    return _normalized_engine


def source_engine() -> Engine:
    """Get or create the source database engine."""
    global _source_engine
    if _source_engine is None:
        _source_engine = get_source_engine()
    return _source_engine


# Session factories (created lazily)
_NormalizedSessionLocal: sessionmaker[Session] | None = None


def get_normalized_session_factory() -> sessionmaker[Session]:
    """Get or create the normalized session factory."""
    global _NormalizedSessionLocal
    if _NormalizedSessionLocal is None:
        _NormalizedSessionLocal = sessionmaker(
            autocommit=False, autoflush=False, bind=normalized_engine()
        )
    return _NormalizedSessionLocal


def get_normalized_db() -> Iterator[Session]:
    """Get a normalized database session (generator for dependency injection).

    Yields:
        Session: SQLAlchemy database session
    """
    session_factory = get_normalized_session_factory()
    db = session_factory()
    try:
        yield db
    finally:
        db.close()


@contextmanager
def get_normalized_session() -> Iterator[Session]:
    """Get a normalized database session as a context manager.

    This provides write access to the reference database and should
    only be used by loaders during initial data population.

    Usage::

        with get_normalized_session() as session:
            session.add(obj)
            session.commit()

    Yields:
        Session: SQLAlchemy database session with write access.
    """
    session_factory = get_normalized_session_factory()
    session = session_factory()
    try:
        yield session
        session.commit()
    except Exception:
        session.rollback()
        raise
    finally:
        session.close()


@contextmanager
def get_reference_session() -> Iterator[Session]:
    """Get a read-only reference database session.

    This is the preferred API for simulation code to access reference data.
    The session is read-only by convention - no commit is performed.

    Usage::

        from babylon.reference.database import get_reference_session
        from babylon.reference.schema import DimCounty

        with get_reference_session() as session:
            counties = session.query(DimCounty).all()

    Yields:
        Session: SQLAlchemy database session (read-only by convention).

    Raises:
        FileNotFoundError: If ``NORMALIZED_DB_PATH`` does not exist. Unlike
            ``get_normalized_engine``, this entry point never auto-creates the
            reference database (Constitution III.11 / task #64) -- an absent
            file here means the DB was never built or the environment is
            misconfigured, not that a fresh one should be silently opened.
    """
    if not NORMALIZED_DB_PATH.is_file():
        raise FileNotFoundError(
            f"Reference database not found at {NORMALIZED_DB_PATH}. "
            "get_reference_session() is read-only and will not create it. "
            "Remedies: `mise run data:build-db` (rebuild from parquet sources, "
            "ADR098), the CI `fetch-reference-db` action, or set "
            "BABYLON_NORMALIZED_DB_PATH to an existing database."
        )
    session_factory = get_normalized_session_factory()
    session = session_factory()
    try:
        yield session
        # No commit - read-only by convention
    finally:
        session.close()


def init_normalized_db() -> None:
    """Create all normalized database tables."""
    from babylon.reference import schema  # noqa: F401

    NormalizedBase.metadata.create_all(bind=normalized_engine())
