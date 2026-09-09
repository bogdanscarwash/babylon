"""Validate post-spec-068 per-industry c/v against Shaikh (2016) bands (US4).

Spec-068 SC-006 acceptance: for each BEA-summary industry, the
population-weighted mean per-county ``c/v`` falls within ±50 % of
Shaikh's documented empirical band.

This tool reads the current ``fact_bea_national_industry`` table,
computes ``c/v = II_share / VA_share`` per (BEA-industry, year),
averages across years, and validates against the Shaikh band table.

Usage::

    uv run python tools/validate_bea_io_against_shaikh.py
    uv run python tools/validate_bea_io_against_shaikh.py --year 2020
    uv run python tools/validate_bea_io_against_shaikh.py --tolerance 0.3
"""

from __future__ import annotations

import argparse
import logging
import sys
from collections import defaultdict
from collections.abc import Sequence

from sqlalchemy import select

from babylon.config.logging_config import setup_logging
from babylon.reference.bea.shaikh_validator import validate_per_industry_c_v
from babylon.reference.database import get_normalized_session
from babylon.reference.schema import (
    DimBEAIndustry,
    DimTime,
    FactBEANationalIndustry,
)


def _compute_per_industry_c_v(
    session: object,
    year_filter: int | None,
) -> tuple[dict[int, float], dict[int, str]]:
    """Compute c/v per BEA industry from fact_bea_national_industry.

    c/v ≈ intermediate_inputs / value_added.
    Averages across years if ``year_filter`` is None.
    """
    from sqlalchemy.orm import Session as _SessionType

    assert isinstance(session, _SessionType)
    stmt = (
        select(
            FactBEANationalIndustry.bea_industry_id,
            DimBEAIndustry.bea_code,
            FactBEANationalIndustry.intermediate_inputs_millions,
            FactBEANationalIndustry.value_added_millions,
            DimTime.year,
        )
        .join(
            DimBEAIndustry,
            DimBEAIndustry.bea_industry_id == FactBEANationalIndustry.bea_industry_id,
        )
        .join(DimTime, DimTime.time_id == FactBEANationalIndustry.time_id)
    )
    if year_filter is not None:
        stmt = stmt.where(DimTime.year == year_filter)

    cv_samples: dict[int, list[float]] = defaultdict(list)
    bea_code_by_id: dict[int, str] = {}
    for industry_id, bea_code, ii, va, _year in session.execute(stmt).all():
        if ii is None or va is None or va == 0:
            continue
        cv_samples[int(industry_id)].append(float(ii) / float(va))
        bea_code_by_id[int(industry_id)] = bea_code

    return (
        {iid: sum(samples) / len(samples) for iid, samples in cv_samples.items()},
        bea_code_by_id,
    )


def main(argv: Sequence[str] | None = None) -> int:
    """CLI entrypoint."""
    parser = argparse.ArgumentParser(
        prog="validate_bea_io_against_shaikh",
        description=(
            "Validate post-spec-068 per-industry c/v against Shaikh (2016) "
            "empirical bands. SC-006 acceptance gate."
        ),
    )
    parser.add_argument(
        "--year",
        type=int,
        default=None,
        help="Restrict to a single year; default averages across all years.",
    )
    parser.add_argument(
        "--tolerance",
        type=float,
        default=0.5,
        help="Band-widening tolerance fraction (default 0.5 for ±50 %%).",
    )
    args = parser.parse_args(argv)
    setup_logging(default_level="INFO")
    log = logging.getLogger("validate_bea_io_against_shaikh")

    with get_normalized_session() as session:
        c_v_by_industry, bea_code_by_id = _compute_per_industry_c_v(session, args.year)
        violations = validate_per_industry_c_v(
            c_v_by_industry, bea_code_by_id, tolerance_fraction=args.tolerance
        )

    n_industries = len(c_v_by_industry)
    n_violations = len(violations)
    n_pass = n_industries - n_violations
    log.info("Shaikh band validation — spec-068 SC-006")
    log.info("  industries measured: %d", n_industries)
    log.info(
        "  in-band (±%.0f%%): %d (%.1f%%)",
        args.tolerance * 100,
        n_pass,
        100.0 * n_pass / max(n_industries, 1),
    )
    log.info("  out-of-band: %d", n_violations)

    if violations:
        log.info("")
        log.info("Out-of-band industries:")
        for v in violations:
            log.info(
                "  %-10s id=%d  measured=%.3f  band=[%.3f, %.3f]",
                v.bea_code,
                v.bea_industry_id,
                v.measured_c_v,
                v.band_lower,
                v.band_upper,
            )

    sc_006_pass = len(violations) == 0
    log.info("")
    log.info("SC-006: %s", "PASS" if sc_006_pass else "FAIL")
    return 0 if sc_006_pass else 1


if __name__ == "__main__":  # pragma: no cover
    sys.exit(main())
