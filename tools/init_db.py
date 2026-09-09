#!/usr/bin/env python3
"""Initialize SQLite reference database with schema.

Creates all tables defined in babylon.reference.schema.

Usage:
    uv run python tools/init_db.py
"""


def init_db() -> None:
    """Create all database tables."""
    from babylon.reference.database import NormalizedBase, init_normalized_db, normalized_engine

    init_normalized_db()

    engine = normalized_engine()
    db_path = str(engine.url).replace("sqlite:///", "")
    print(f"Database initialized: {db_path}")
    print(f"Tables created: {list(NormalizedBase.metadata.tables.keys())}")


def main() -> None:
    """Entry point."""
    init_db()


if __name__ == "__main__":
    main()
