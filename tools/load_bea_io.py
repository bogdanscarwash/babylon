"""CLI entrypoint for the BEA national I-O ingest pipeline (spec-068).

Stages:
    US1: Populate fact_bea_national_industry from Supply-Use XLSX.
    US2: Populate fact_bea_io_coefficient from Make+Use IOUse and TDR XLSX.
    US3: Populate bridge_naics_bea concordance + wire hex_hydrator.
    US4: Validate per-county c/v distribution against Shaikh bands.

Modes:
    --rollback         Truncate fact_bea_national_industry and
                       fact_bea_io_coefficient to empty pre-spec-068 state.
    --dry-run          Parse + validate but do not write to DB.
    --reload-concordance
                       Re-populate bridge_naics_bea from the BEA concordance
                       bundle (default: only on first run).

Per spec-068 SC-007, the full live run targets <15 minutes wallclock for
the 2010-2024 ingest scope. The spec-067 PRAGMA work (WAL, 2 GiB cache,
12 GB mmap) on the reference DB is a prerequisite.

Audit reports land at reports/ingest/bea_io_<timestamp>.{md,json}.

Usage::

    uv run python tools/load_bea_io.py --years 2010-2024
    uv run python tools/load_bea_io.py --rollback
    uv run python tools/load_bea_io.py --dry-run --years 2010-2024
"""

from __future__ import annotations

import argparse
import logging
import sys
import time
from collections.abc import Sequence
from datetime import UTC, datetime
from pathlib import Path

from sqlalchemy import select

from babylon.config.logging_config import setup_logging
from babylon.reference.bea.ingest.audit_report import (
    BEAIngestAuditReport,
    IndustrySnapshot,
)
from babylon.reference.bea.ingest.io_coefficient_writer import (
    upsert_io_coefficient_records,
)
from babylon.reference.bea.ingest.io_matrix_parser import (
    extract_iouse_internal_shares,
    parse_total_req_matrix,
    parse_use_matrix,
)
from babylon.reference.bea.ingest.national_writer import upsert_national_records
from babylon.reference.bea.ingest.schema_migration import ensure_vintage_columns
from babylon.reference.bea.ingest.supply_use_parser import (
    BEAIngestError,
    parse_use_summary,
)
from babylon.reference.bea.ingest.validators import (
    validate_accounting_identity,
    validate_column_sum_identity,
)
from babylon.reference.database import get_normalized_session, normalized_engine
from babylon.reference.schema import DimBEAIndustry, FactBEANationalIndustry


def _parse_years(value: str) -> range:
    """Parse a ``--years`` argument like ``"2010-2024"`` into a ``range``.

    Args:
        value: String of form ``"START-END"`` (inclusive on both ends).

    Returns:
        ``range(START, END + 1)``.

    Raises:
        argparse.ArgumentTypeError: If the value is malformed or START > END.
    """
    try:
        start_str, end_str = value.split("-", 1)
        start, end = int(start_str), int(end_str)
    except ValueError as exc:
        raise argparse.ArgumentTypeError(
            f"--years must be of form 'START-END' (got {value!r})"
        ) from exc
    if start > end:
        raise argparse.ArgumentTypeError(f"--years START ({start}) must be <= END ({end})")
    return range(start, end + 1)


def _build_arg_parser() -> argparse.ArgumentParser:
    """Construct the CLI argument parser for ``load_bea_io.py``."""
    parser = argparse.ArgumentParser(
        prog="load_bea_io",
        description=(
            "Ingest BEA national I-O tables into the reference DB. "
            "Spec-068. See specs/068-bea-national-io-ingest/quickstart.md."
        ),
    )
    parser.add_argument(
        "--years",
        type=_parse_years,
        default=_parse_years("2010-2024"),
        help="Year range to ingest, inclusive (default: 2010-2024).",
    )
    parser.add_argument(
        "--rollback",
        action="store_true",
        help=(
            "Truncate fact_bea_national_industry and fact_bea_io_coefficient "
            "to empty pre-spec-068 state. Does NOT truncate bridge_naics_bea "
            "(shared with spec-025)."
        ),
    )
    parser.add_argument(
        "--dry-run",
        action="store_true",
        help="Parse and validate but do not write to the reference DB.",
    )
    parser.add_argument(
        "--reload-concordance",
        action="store_true",
        help=(
            "Force re-population of bridge_naics_bea from the BEA concordance "
            "bundle (default: only on first run when the table is empty)."
        ),
    )
    parser.add_argument(
        "--log-level",
        choices=("DEBUG", "INFO", "WARNING", "ERROR"),
        default="INFO",
        help="Logging verbosity (default: INFO).",
    )
    return parser


def _run_rollback_stage(audit_report: BEAIngestAuditReport) -> None:
    """Truncate spec-068 fact tables to empty state (FR-009).

    Populates ``audit_report.rows_inserted`` with negative counts to
    indicate the deletion magnitude. VACUUM is run outside the
    transaction since SQLite forbids VACUUM inside a transaction.
    """
    from sqlalchemy import text

    log = logging.getLogger("load_bea_io.rollback")
    engine = normalized_engine()
    with engine.begin() as conn:
        for table_name in (
            "fact_bea_io_coefficient",  # dependent first
            "fact_bea_national_industry",
        ):
            before = conn.execute(text(f"SELECT COUNT(*) FROM {table_name}")).scalar() or 0
            conn.execute(text(f"DELETE FROM {table_name}"))
            log.info("rollback: truncated %s (rows removed: %d)", table_name, before)
            audit_report.rows_inserted[table_name] = -int(before)
    # VACUUM outside the transaction (SQLite requires this).
    with engine.connect() as conn:
        conn.execute(text("VACUUM"))
    log.info("rollback complete; bridge_naics_bea left intact (shared with spec-025)")


_USE_SUMMARY_DEFAULT_PATH = Path("data/input-output/supply-use/Use_Summary.xlsx")


def _run_us1_stage(
    audit_report: BEAIngestAuditReport,
    years: range,
    dry_run: bool,
) -> None:
    """US1: populate fact_bea_national_industry from Supply-Use XLSX (T026).

    Parses ``Use_Summary.xlsx``, validates FR-002 accounting identity per
    record, and (unless dry_run) UPSERTs into the fact table. Populates
    the SC-001 and SC-003 audit gates.
    """
    log = logging.getLogger("load_bea_io.us1")
    log.info("US1 stage: parsing Use_Summary.xlsx for years %d-%d", years.start, years.stop - 1)

    with get_normalized_session() as session:
        try:
            records = list(parse_use_summary(_USE_SUMMARY_DEFAULT_PATH, years, session))
        except BEAIngestError as exc:
            log.error("US1 stage: parser failed — %s", exc)
            audit_report.sc_001_pass = False
            audit_report.sc_003_pass = False
            raise

        # FR-002 validation per record (always run, even on dry-run).
        violations = [v for r in records for v in [validate_accounting_identity(r)] if v]
        audit_report.accounting_identity_violations.extend(violations)

        if not dry_run:
            stats = upsert_national_records(session, records)
            audit_report.rows_inserted["fact_bea_national_industry"] = stats.rows_inserted
            audit_report.rows_superseded["fact_bea_national_industry"] = stats.rows_superseded
            audit_report.rows_unchanged["fact_bea_national_industry"] = stats.rows_unchanged
            audit_report.vintage_supersessions.extend(stats.supersessions)

            # Top/bottom-10 leaderboards (post-write snapshot).
            _populate_share_leaderboards(session, audit_report)
        else:
            audit_report.rows_inserted["fact_bea_national_industry"] = len(records)
            log.info("US1 stage: dry-run — skipping DB write")

    # SC-001: row count >= 800 (after full ingest, not per-year)
    total_rows = audit_report.rows_inserted.get(
        "fact_bea_national_industry", 0
    ) + audit_report.rows_unchanged.get("fact_bea_national_industry", 0)
    audit_report.sc_001_pass = total_rows >= 800

    # SC-003: 100 % of rows pass FR-002.
    audit_report.sc_003_pass = len(violations) == 0
    log.info(
        "US1 stage: %d records, %d FR-002 violations, sc_001_pass=%s sc_003_pass=%s",
        len(records),
        len(violations),
        audit_report.sc_001_pass,
        audit_report.sc_003_pass,
    )


def _populate_share_leaderboards(
    session: object,  # SQLAlchemy Session
    audit_report: BEAIngestAuditReport,
) -> None:
    """Compute top-10 / bottom-10 intermediate-inputs share for the audit report.

    Uses a SQL-side division of intermediate_inputs / gross_output,
    excluding rows where either is NULL or GO == 0.
    """
    from sqlalchemy.orm import Session as _SessionType

    assert isinstance(session, _SessionType)
    rows = session.execute(
        select(
            FactBEANationalIndustry.bea_industry_id,
            DimBEAIndustry.industry_name,
            FactBEANationalIndustry.time_id,
            (
                FactBEANationalIndustry.intermediate_inputs_millions
                / FactBEANationalIndustry.gross_output_millions
            ).label("ii_share"),
        )
        .join(
            DimBEAIndustry,
            DimBEAIndustry.bea_industry_id == FactBEANationalIndustry.bea_industry_id,
        )
        .where(FactBEANationalIndustry.gross_output_millions > 0)
        .where(FactBEANationalIndustry.intermediate_inputs_millions.is_not(None))
    ).all()

    if not rows:
        return

    # Sort by share, ascending and descending; collect top/bottom 10.
    sorted_rows = sorted(rows, key=lambda r: float(r.ii_share))
    bottom = sorted_rows[:10]
    top = sorted_rows[-10:][::-1]

    # We also need year for IndustrySnapshot — but the query returned time_id,
    # not year. Map back via dim_time:
    from babylon.reference.schema import DimTime as _DimTime

    time_id_to_year = dict(session.execute(select(_DimTime.time_id, _DimTime.year)).all())

    audit_report.intermediate_inputs_share_top10 = [
        IndustrySnapshot(
            bea_industry_id=r.bea_industry_id,
            bea_industry_name=r.industry_name,
            year=int(time_id_to_year[r.time_id]),
            intermediate_inputs_share=float(r.ii_share),
        )
        for r in top
    ]
    audit_report.intermediate_inputs_share_bottom10 = [
        IndustrySnapshot(
            bea_industry_id=r.bea_industry_id,
            bea_industry_name=r.industry_name,
            year=int(time_id_to_year[r.time_id]),
            intermediate_inputs_share=float(r.ii_share),
        )
        for r in bottom
    ]


_IOUSE_DEFAULT_PATH = Path("data/input-output/make-use/IOUse_Before_Redefinitions_PRO_Summary.xlsx")
_TDR_DEFAULT_PATH = Path("data/input-output/total-domestic-requirements/IxI_TR_Summary.xlsx")


def _run_us2_stage(
    audit_report: BEAIngestAuditReport,
    years: range,
    dry_run: bool,
) -> None:
    """US2: populate fact_bea_io_coefficient from Make+Use + TDR XLSX (T038).

    Wires the USE matrix (a_ij coefficients) + TOTAL_REQ (Leontief inverse
    for cross-validation). Validates FR-004 column-sum identity against
    IOUse's own producer-prices intermediate-inputs share.
    """
    log = logging.getLogger("load_bea_io.us2")
    log.info(
        "US2 stage: parsing IOUse_Summary + TDR for years %d-%d",
        years.start,
        years.stop - 1,
    )

    with get_normalized_session() as session:
        try:
            use_records = list(parse_use_matrix(_IOUSE_DEFAULT_PATH, years, session))
            tdr_records = list(parse_total_req_matrix(_TDR_DEFAULT_PATH, years, session))
            expected_shares = extract_iouse_internal_shares(_IOUSE_DEFAULT_PATH, years, session)
        except BEAIngestError as exc:
            log.error("US2 stage: parser failed — %s", exc)
            audit_report.sc_002_pass = False
            audit_report.sc_004_pass = False
            raise

        # FR-004 column-sum validation (USE table only, IOUse-internal shares).
        violations = validate_column_sum_identity(use_records, expected_shares)
        audit_report.column_sum_identity_violations.extend(violations)

        if not dry_run:
            all_records = use_records + tdr_records
            stats = upsert_io_coefficient_records(session, all_records)
            audit_report.rows_inserted["fact_bea_io_coefficient"] = stats.rows_inserted
            audit_report.rows_superseded["fact_bea_io_coefficient"] = stats.rows_superseded
            audit_report.rows_unchanged["fact_bea_io_coefficient"] = stats.rows_unchanged
            audit_report.vintage_supersessions.extend(stats.supersessions)
        else:
            audit_report.rows_inserted["fact_bea_io_coefficient"] = len(use_records) + len(
                tdr_records
            )
            log.info("US2 stage: dry-run — skipping DB write")

    total_rows = audit_report.rows_inserted.get(
        "fact_bea_io_coefficient", 0
    ) + audit_report.rows_unchanged.get("fact_bea_io_coefficient", 0)
    audit_report.sc_002_pass = total_rows >= 50_000
    audit_report.sc_004_pass = len(violations) == 0
    log.info(
        "US2 stage: %d USE + %d TDR records, %d FR-004 violations, sc_002_pass=%s sc_004_pass=%s",
        len(use_records),
        len(tdr_records),
        len(violations),
        audit_report.sc_002_pass,
        audit_report.sc_004_pass,
    )


def _run_us3_stage(
    audit_report: BEAIngestAuditReport,
    years: range,
    reload_concordance: bool,  # noqa: ARG001
    dry_run: bool,  # noqa: ARG001
) -> None:
    """US3: concordance coverage + stale-share fallback summary (SC-008).

    Computes the ``StaleShareFallbackSummary`` — the employment-weighted
    fraction of QCEW employment that falls back to ``GLOBAL_FALLBACK_SHARE``
    because the mapped BEA industry has no ``fact_bea_national_industry``
    data within the 5-year forward-fill window. SC-008 requires this to be
    < 1% of total QCEW employment.

    The concordance (``bridge_naics_bea``) is populated by the spec-025
    loader and is shared with spec-068; it is NOT re-populated here unless
    ``--reload-concordance`` is passed (deferred to a future T054 follow-up).
    The hex_hydrator wiring (T056-T057) is in ``hex_hydrator.py`` +
    ``postgres_initialization.py``, not in the ingest CLI.
    """
    log = logging.getLogger("load_bea_io.us3")
    log.info("US3 stage: computing stale-share fallback summary for SC-008")

    from babylon.reference.bea.ingest.stale_share_summary import (
        compute_stale_share_fallback_summary,
    )

    with get_normalized_session() as session:
        audit_report.stale_share_fallback_summary = compute_stale_share_fallback_summary(
            session, years
        )

    summary = audit_report.stale_share_fallback_summary
    assert summary is not None  # just set above
    log.info(
        "US3 stage: total_lookups=%d forward_filled=%d global_default=%d "
        "affected_employment_fraction=%.6f → sc_008_pass=%s",
        summary.total_county_year_lookups,
        summary.forward_filled_lookups,
        summary.global_default_lookups,
        summary.affected_employment_fraction,
        summary.affected_employment_fraction < 0.01,
    )


def _finalize_audit_report(audit_report: BEAIngestAuditReport) -> None:
    """Compute final SC pass/fail gates and the SC-007 wallclock judgement.

    SC-005 (stddev c/v >= 0.2) is post-hoc — set externally after running
    ``mise run sim:e2e-michigan``. SC-008 is computed here from the
    ``stale_share_fallback_summary`` populated in US3.
    """
    audit_report.sc_007_wallclock_seconds = audit_report.duration_seconds
    audit_report.sc_007_pass = audit_report.duration_seconds < 900.0  # 15 minutes
    if audit_report.stale_share_fallback_summary is not None:
        audit_report.sc_008_pass = (
            audit_report.stale_share_fallback_summary.affected_employment_fraction < 0.01
        )


def main(argv: Sequence[str] | None = None) -> int:
    """CLI entrypoint.

    Returns:
        Process exit code (0 = success, non-zero = failure).
    """
    parser = _build_arg_parser()
    args = parser.parse_args(argv)
    setup_logging(default_level=args.log_level)
    log = logging.getLogger("load_bea_io")
    log.info(
        "spec-068 BEA I-O ingest starting (years=%d-%d, rollback=%s, dry_run=%s)",
        args.years.start,
        args.years.stop - 1,
        args.rollback,
        args.dry_run,
    )

    start_time = time.monotonic()
    audit_report = BEAIngestAuditReport(
        timestamp=datetime.now(UTC),
        sim_years_in_scope=tuple(args.years),
        dry_run=args.dry_run,
    )

    # Schema migration (T009) — idempotent ALTER TABLE ADD COLUMN.
    if not args.dry_run:
        engine = normalized_engine()
        migration_result = ensure_vintage_columns(engine)
        for table, was_added in migration_result.items():
            log.info(
                "schema migration: %s vintage_published_date %s",
                table,
                "ADDED" if was_added else "already present",
            )

    if args.rollback:
        _run_rollback_stage(audit_report)
    else:
        _run_us1_stage(audit_report, args.years, args.dry_run)
        _run_us2_stage(audit_report, args.years, args.dry_run)
        _run_us3_stage(audit_report, args.years, args.reload_concordance, args.dry_run)
        # US4 is operator-driven via tools/validate_bea_io_against_shaikh.py.

    audit_report.duration_seconds = time.monotonic() - start_time
    _finalize_audit_report(audit_report)
    json_path, md_path = audit_report.write_to_disk()
    log.info("audit report written: %s + %s", json_path, md_path)

    return 0


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
