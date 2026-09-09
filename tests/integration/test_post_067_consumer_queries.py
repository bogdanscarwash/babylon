"""Reference-data checks for normalized QCEW leaves after spec-067.

These checks retain the Wayne County annual-wage and Michigan county-year
invariants independently of the retired Python runtime.
"""

from __future__ import annotations

from collections.abc import Iterator

import pytest
from sqlalchemy import text
from sqlalchemy.orm import Session

from babylon.reference.database import get_reference_session


@pytest.fixture
def post_067_session() -> Iterator[Session]:
    """Reference DB session AFTER the spec-067 migration has been applied.

    Skip during fixture setup if the reference DB still contains rollup rows.
    """

    with get_reference_session() as session:
        rollups_remaining = (
            session.execute(
                text(
                    "SELECT COUNT(*) FROM fact_qcew_annual fq "
                    "JOIN dim_industry i ON fq.industry_id = i.industry_id "
                    "JOIN dim_ownership o ON fq.ownership_id = o.ownership_id "
                    "WHERE NOT (i.naics_level = 6 AND o.own_code != '0')"
                )
            ).scalar()
            or 0
        )
        if rollups_remaining > 0:
            pytest.skip(
                f"reference DB still has {rollups_remaining:,} rollup rows; "
                "run `mise exec -- uv run --frozen python tools/normalize_qcew_rollups.py --apply` first"
            )
        yield session


# T038 — Wayne County 2010 via the post-067 SUM-of-leaves path.
@pytest.mark.requires_reference_db
def test_post_067_wayne_2010_has_positive_total_wages(
    post_067_session: Session,
) -> None:
    """Canonical Wayne County leaves have a nonzero annual wage sum."""

    actual_wages = post_067_session.execute(
        text(
            "SELECT SUM(fq.total_wages_usd) "
            "FROM fact_qcew_annual fq "
            "JOIN dim_county c ON fq.county_id = c.county_id "
            "JOIN dim_time t ON fq.time_id = t.time_id "
            "WHERE c.fips = '26163' AND t.year = 2010"
        )
    ).scalar()
    assert actual_wages is not None, "no QCEW data for Wayne 2010 post-067"
    assert float(actual_wages) > 0, "post-067 Wayne 2010 total_wages SUM is zero"


# T039 — Per-county-year statistical floor (SC-007 within QCEW-suppression bound).
@pytest.mark.requires_reference_db
@pytest.mark.xfail(
    strict=False,
    reason="dim_county carries MI balance-of-state pseudo-county 26999 with zero"
    " fact_qcew_annual rows in the trove itself (SQL-verified 2026-07-11; the"
    " ci-data-v1 subset mirrors that absence exactly) — data-load gap,"
    " spec-086/097/098 remediation; owner item 2026-07-11",
)
def test_post_067_michigan_county_years_have_non_zero_employment(
    post_067_session: Session,
) -> None:
    """Every Michigan county-year (2010-2024) returns a non-zero employment
    SUM post-067. This is a structural integrity test — the migration
    should never leave a Michigan county-year with zero canonical leaves.
    """

    zero_county_years = post_067_session.execute(
        text(
            "SELECT c.fips, t.year "
            "FROM dim_county c CROSS JOIN dim_time t "
            "WHERE c.fips LIKE '26%' AND t.is_annual = 1 "
            "  AND t.year BETWEEN 2010 AND 2024 "
            "  AND NOT EXISTS ("
            "    SELECT 1 FROM fact_qcew_annual fq "
            "    WHERE fq.county_id = c.county_id AND fq.time_id = t.time_id"
            "  )"
        )
    ).all()
    assert len(zero_county_years) == 0, (
        f"{len(zero_county_years)} Michigan county-years have zero post-067 rows: "
        f"{zero_county_years[:5]}"
    )
