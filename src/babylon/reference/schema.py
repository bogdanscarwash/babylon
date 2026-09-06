"""3NF Normalized SQLite schema for Marxian economic analysis.

Provides properly normalized dimension and fact tables populated via ETL
from external data sources. Optimized for imperial rent, surplus value,
labor aristocracy, and unequal exchange analysis.

Dimensions (33 tables):
    Geographic: dim_state, dim_county, dim_metro_area, dim_geographic_hierarchy,
                dim_cfs_area, dim_country, dim_import_source
    Bridge: bridge_county_metro, bridge_cfs_county
    Industry: dim_industry, dim_sector, dim_ownership
    Census Codes: dim_income_bracket, dim_employment_status, dim_worker_class,
                  dim_occupation, dim_education_level, dim_housing_tenure,
                  dim_rent_burden, dim_commute_mode, dim_poverty_category
    Energy: dim_energy_table, dim_energy_series
    FRED: dim_wealth_class, dim_asset_category, dim_fred_series
    Commodities: dim_commodity, dim_commodity_metric, dim_sctg_commodity
    Metadata: dim_time, dim_gender, dim_data_source, dim_race
    Coercive: dim_coercive_type
    Employment: dim_employment_area

Facts (34 tables):
    Census (14), QCEW/Productivity (5), Trade (1), Energy (1),
    FRED (5), Commodities (1), Materials (3), Circulatory (3), DOT (1)

Note:
    Census fact tables include time_id and race_id FKs for multi-year and
    race-disaggregated analysis (15 years x 10 race groups).

    The census above is the pre-ADR076 schema. The 2026-07-17 execution
    slate (ADR075/076) amputated 16 tables (incl. the Energy dims/fact
    listed here) and demoted 8 — ``data-catalog.yaml`` is the authoritative
    per-table registry; amputated tables live on as committed CSV artifacts
    registered in ``data-artifacts.yaml``.

    Since the parquet-canonical cutover (ADR098, 2026-07-20),
    ``data-artifacts.yaml`` is the LINEAGE AUTHORITY for the whole estate:
    per-table parquet sources + ``schema.sql`` are canonical (sha-pinned in
    the manifest), and the SQLite reference DB is a deterministic build
    product of ``tools/build_reference_db.py``. These ORM classes describe
    that product for readers; the DDL of record is ``schema.sql``.
"""

from datetime import date, datetime
from decimal import Decimal

from sqlalchemy import (
    CheckConstraint,
    Date,
    Float,
    ForeignKey,
    Index,
    Integer,
    Numeric,
    String,
    Text,
    UniqueConstraint,
    func,
    text,
)
from sqlalchemy.orm import Mapped, mapped_column
from sqlalchemy.types import DateTime

from babylon.reference.database import NormalizedBase

# =============================================================================
# GEOGRAPHIC DIMENSION TABLES
# =============================================================================


class DimState(NormalizedBase):
    """US state dimension - unified from Census, FRED, and Materials."""

    __tablename__ = "dim_state"

    state_id: Mapped[int] = mapped_column(primary_key=True)
    state_fips: Mapped[str] = mapped_column(String(2), unique=True, nullable=False)
    state_name: Mapped[str] = mapped_column(String(100), nullable=False)
    state_abbrev: Mapped[str] = mapped_column(String(2), nullable=False)

    __table_args__ = (Index("idx_state_abbrev", "state_abbrev"),)


class DimCounty(NormalizedBase):
    """US county dimension - primary domestic geographic grain."""

    __tablename__ = "dim_county"

    county_id: Mapped[int] = mapped_column(primary_key=True)
    fips: Mapped[str] = mapped_column(String(5), unique=True, nullable=False)
    state_id: Mapped[int] = mapped_column(ForeignKey("dim_state.state_id"), nullable=False)
    county_fips: Mapped[str] = mapped_column(String(3), nullable=False)
    county_name: Mapped[str] = mapped_column(String(200), nullable=False)
    h3_res4: Mapped[str | None] = mapped_column(String(15))

    __table_args__ = (
        Index("idx_county_state", "state_id"),
        Index("idx_county_name", "county_name"),
    )


class DimCountyGeometry(NormalizedBase):
    """County geometry for spatial operations.

    Stores county boundary polygons and centroids from TIGER shapefiles
    for spatial aggregation (H3 hex mapping) and visualization.

    The geometry_wkt column contains Well-Known Text format polygons
    for counties requiring spatial queries. This is a 1:1 extension
    of DimCounty, not a separate dimension.

    Attributes:
        county_id: FK to DimCounty (also primary key).
        centroid_lat: Latitude of county centroid (WGS84).
        centroid_lon: Longitude of county centroid (WGS84).
        area_sq_km: County land area in square kilometers.
        geometry_wkt: Well-Known Text polygon boundary (optional, large).
    """

    __tablename__ = "dim_county_geometry"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    centroid_lat: Mapped[Decimal] = mapped_column(Numeric(9, 6), nullable=False)
    centroid_lon: Mapped[Decimal] = mapped_column(Numeric(10, 6), nullable=False)
    area_sq_km: Mapped[Decimal | None] = mapped_column(Numeric(12, 4))
    geometry_wkt: Mapped[str | None] = mapped_column(Text)


class DimMetroArea(NormalizedBase):
    """Metropolitan and Micropolitan Statistical Areas.

    Area Types:
        msa: Metropolitan Statistical Area (50K+ urban core)
        micropolitan: Micropolitan Statistical Area (10K-50K urban core)
        csa: Combined Statistical Area (aggregation of adjacent CBSAs)
    """

    __tablename__ = "dim_metro_area"

    metro_area_id: Mapped[int] = mapped_column(primary_key=True)
    geo_id: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    cbsa_code: Mapped[str | None] = mapped_column(String(10))
    metro_name: Mapped[str] = mapped_column(String(200), nullable=False)
    area_type: Mapped[str] = mapped_column(String(20), nullable=False)  # msa, micropolitan, csa

    __table_args__ = (
        Index("idx_metro_cbsa", "cbsa_code"),
        CheckConstraint("area_type IN ('msa', 'micropolitan', 'csa')", name="ck_metro_area_type"),
    )


class BridgeCountyMetro(NormalizedBase):
    """Many-to-many county-metro mapping."""

    __tablename__ = "bridge_county_metro"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    metro_area_id: Mapped[int] = mapped_column(
        ForeignKey("dim_metro_area.metro_area_id"), primary_key=True
    )
    is_principal_city: Mapped[bool] = mapped_column(default=False)


class BridgeCountyH3(NormalizedBase):
    """H3 hexagon to county mapping for spatial aggregation.

    Maps H3 hexagonal cells at a specified resolution to counties,
    enabling efficient spatial joins without runtime polygon operations.

    At resolution 5 (~252 km² hexagons), CONUS requires ~21K cells.
    Each cell maps to one county (the county containing the cell center).

    Attributes:
        h3_index: H3 cell index string (15 chars, primary key).
        county_id: FK to DimCounty containing this cell's centroid.
        resolution: H3 resolution level (default 5).
        coverage_pct: Optional % of cell area within county bounds.
    """

    __tablename__ = "bridge_county_h3"

    h3_index: Mapped[str] = mapped_column(String(15), primary_key=True)
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    resolution: Mapped[int] = mapped_column(default=5)
    coverage_pct: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))

    __table_args__ = (Index("idx_h3_county", "county_id"),)


class DimGeographicHierarchy(NormalizedBase):
    """State-to-county geographic hierarchy with allocation weights.

    Enables disaggregation of state-level external data (e.g., Census CFS)
    to county-level internal schema representation. Each county row contains
    weights for distributing state-level flows proportionally.

    Allocation weight types:
        - population_weight: Share of state population in county
        - employment_weight: Share of state employment in county
        - gdp_weight: Share of state GDP in county (proxy via employment + wages)

    Example usage::

        # Get all counties for California with weights
        SELECT c.county_name, h.population_weight, h.employment_weight
        FROM dim_geographic_hierarchy h
        JOIN dim_county c ON h.county_id = c.county_id
        JOIN dim_state s ON h.state_id = s.state_id
        WHERE s.state_fips = '06'
        ORDER BY h.population_weight DESC
    """

    __tablename__ = "dim_geographic_hierarchy"

    hierarchy_id: Mapped[int] = mapped_column(primary_key=True)
    state_id: Mapped[int] = mapped_column(ForeignKey("dim_state.state_id"), nullable=False)
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    population_weight: Mapped[Decimal] = mapped_column(
        Numeric(10, 8), nullable=False
    )  # 0.0 to 1.0, sum to 1.0 per state
    employment_weight: Mapped[Decimal] = mapped_column(
        Numeric(10, 8), nullable=False
    )  # 0.0 to 1.0, sum to 1.0 per state
    gdp_weight: Mapped[Decimal | None] = mapped_column(
        Numeric(10, 8)
    )  # Optional, derived from employment+wages
    source_year: Mapped[int] = mapped_column(nullable=False)  # Year of weight calculation

    __table_args__ = (
        Index("idx_geo_hierarchy_state", "state_id"),
        Index("idx_geo_hierarchy_county", "county_id"),
        # Ensure unique state-county pairing per year (UniqueConstraint required for ON CONFLICT)
        UniqueConstraint("state_id", "county_id", "source_year", name="uq_geo_hierarchy_unique"),
        CheckConstraint(
            "population_weight >= 0 AND population_weight <= 1",
            name="ck_population_weight_range",
        ),
        CheckConstraint(
            "employment_weight >= 0 AND employment_weight <= 1",
            name="ck_employment_weight_range",
        ),
    )


class DimCFSArea(NormalizedBase):
    """Census Commodity Flow Survey geographic areas (132 FAF zones).

    CFS areas are aggregations of counties used by the Census Bureau for
    commodity flow analysis. Each CFS area maps to one or more counties.
    """

    __tablename__ = "dim_cfs_area"

    cfs_area_id: Mapped[int] = mapped_column(primary_key=True)
    cfs_code: Mapped[str] = mapped_column(String(10), unique=True, nullable=False)
    cfs_name: Mapped[str] = mapped_column(String(200), nullable=False)
    state_id: Mapped[int | None] = mapped_column(ForeignKey("dim_state.state_id"))

    __table_args__ = (Index("idx_cfs_area_state", "state_id"),)


class BridgeCFSCounty(NormalizedBase):
    """Many-to-many CFS area to county mapping with allocation weights."""

    __tablename__ = "bridge_cfs_county"

    cfs_area_id: Mapped[int] = mapped_column(
        ForeignKey("dim_cfs_area.cfs_area_id"), primary_key=True
    )
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    allocation_weight: Mapped[Decimal] = mapped_column(
        Numeric(10, 8), nullable=False
    )  # Share of CFS area in this county


class DimSCTGCommodity(NormalizedBase):
    """Standard Classification of Transported Goods (SCTG) commodity codes.

    42 two-digit SCTG codes used by Census CFS for commodity classification.
    """

    __tablename__ = "dim_sctg_commodity"

    sctg_id: Mapped[int] = mapped_column(primary_key=True)
    sctg_code: Mapped[str] = mapped_column(String(5), unique=True, nullable=False)
    sctg_name: Mapped[str] = mapped_column(String(200), nullable=False)
    category: Mapped[str | None] = mapped_column(String(50))  # agriculture, mining, manufacturing
    strategic_value: Mapped[str | None] = mapped_column(String(20))  # critical, high, medium, low

    __table_args__ = (
        Index("idx_sctg_category", "category"),
        CheckConstraint(
            "strategic_value IN ('critical', 'high', 'medium', 'low') OR strategic_value IS NULL",
            name="ck_sctg_strategic_value",
        ),
    )


class DimCountry(NormalizedBase):
    """International trade partners with world-system classification."""

    __tablename__ = "dim_country"

    country_id: Mapped[int] = mapped_column(primary_key=True)
    cty_code: Mapped[str] = mapped_column(String(10), unique=True, nullable=False)
    country_name: Mapped[str] = mapped_column(String(200), nullable=False)
    is_region: Mapped[bool] = mapped_column(default=False)
    world_system_tier: Mapped[str | None] = mapped_column(
        String(20)
    )  # core/semi_periphery/periphery

    __table_args__ = (
        Index("idx_country_tier", "world_system_tier"),
        Index("idx_country_name", "country_name"),
        CheckConstraint(
            "world_system_tier IN ('core', 'semi_periphery', 'periphery') OR world_system_tier IS NULL",
            name="ck_world_system_tier",
        ),
    )


class DimImportSource(NormalizedBase):
    """Import source countries with geopolitical classification."""

    __tablename__ = "dim_import_source"

    import_source_id: Mapped[int] = mapped_column(primary_key=True)
    country: Mapped[str] = mapped_column(String(100), nullable=False)
    commodity_count: Mapped[int | None] = mapped_column()
    map_class: Mapped[str | None] = mapped_column(String(50))  # Geopolitical classification


# =============================================================================
# INDUSTRY & OWNERSHIP DIMENSION TABLES
# =============================================================================


class DimIndustry(NormalizedBase):
    """Unified NAICS industry dimension bridging QCEW, Productivity, and FRED."""

    __tablename__ = "dim_industry"

    industry_id: Mapped[int] = mapped_column(primary_key=True)
    naics_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    industry_title: Mapped[str] = mapped_column(String(300), nullable=False)
    naics_level: Mapped[int] = mapped_column(nullable=False)  # 0-6
    parent_naics_code: Mapped[str | None] = mapped_column(String(20))
    sector_code: Mapped[str | None] = mapped_column(String(2))  # 2-digit for rollup

    # Marxian classification
    class_composition: Mapped[str | None] = mapped_column(String(20))

    # Data source flags
    has_productivity_data: Mapped[bool] = mapped_column(default=False)
    has_fred_data: Mapped[bool] = mapped_column(default=False)
    has_qcew_data: Mapped[bool] = mapped_column(default=False)

    __table_args__ = (
        Index("idx_industry_level", "naics_level"),
        Index("idx_industry_sector", "sector_code"),
        Index("idx_industry_class", "class_composition"),
        Index("idx_industry_parent", "parent_naics_code"),
        CheckConstraint(
            "class_composition IN ('goods_producing', 'service_producing', 'circulation', "
            "'government', 'extraction') OR class_composition IS NULL",
            name="ck_class_composition",
        ),
    )


class DimOwnership(NormalizedBase):
    """QCEW ownership types with government/private flags."""

    __tablename__ = "dim_ownership"

    ownership_id: Mapped[int] = mapped_column(primary_key=True)
    own_code: Mapped[str] = mapped_column(String(2), unique=True, nullable=False)
    own_title: Mapped[str] = mapped_column(String(50), nullable=False)
    is_government: Mapped[bool] = mapped_column(nullable=False)
    is_private: Mapped[bool] = mapped_column(nullable=False)


class DimBEAIndustry(NormalizedBase):
    """BEA industry classification (~71 industries).

    The Bureau of Economic Analysis uses its own industry classification scheme
    for GDP-by-industry, input-output, and gross output data. BEA industries
    roughly correspond to 2-3 digit NAICS but with aggregations and splits
    specific to national accounting.

    Hierarchy levels:
        1 = Sector (All industries, Private industries, Government)
        2 = Major industry group (Agriculture, Mining, Manufacturing, etc.)
        3 = Industry (Farms, Oil & Gas, Motor vehicles, etc.)
        4 = Detail industry (where available)

    Note:
        County-level BEA GDP uses a coarser ~20-industry classification.
        National-level BEA data uses the full ~71-industry detail.
        The bridge_naics_bea table maps NAICS codes to BEA industries.
    """

    __tablename__ = "dim_bea_industry"

    bea_industry_id: Mapped[int] = mapped_column(primary_key=True)
    bea_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    industry_name: Mapped[str] = mapped_column(String(200), nullable=False)
    bea_level: Mapped[int] = mapped_column(
        nullable=False
    )  # 1=sector, 2=major, 3=industry, 4=detail
    parent_bea_code: Mapped[str | None] = mapped_column(String(20))
    line_number: Mapped[int | None] = mapped_column()  # BEA table line number for reference

    __table_args__ = (
        Index("idx_bea_industry_level", "bea_level"),
        Index("idx_bea_industry_parent", "parent_bea_code"),
        CheckConstraint("bea_level >= 1 AND bea_level <= 4", name="ck_bea_level_range"),
    )


class BridgeNAICSBEA(NormalizedBase):
    """NAICS to BEA industry concordance.

    Maps NAICS codes from DimIndustry to BEA industry codes in DimBEAIndustry.
    This bridge enables linking QCEW employment data (NAICS-based) to BEA
    gross output and value-added data for surplus value calculations.

    Mapping quality indicators:
        exact: 1:1 NAICS to BEA mapping
        aggregated: Multiple NAICS codes map to one BEA industry
        split: One NAICS code maps to multiple BEA industries (weighted)
        estimated: Approximate mapping based on SIC-era concordances

    Note:
        County-level BEA uses coarser industries than national BEA.
        A NAICS code may map to different BEA codes at different levels.
    """

    __tablename__ = "bridge_naics_bea"

    industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_industry.industry_id"), primary_key=True
    )
    bea_industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_bea_industry.bea_industry_id"), primary_key=True
    )
    mapping_quality: Mapped[str] = mapped_column(String(20), default="exact")
    weight: Mapped[Decimal | None] = mapped_column(Numeric(5, 4))  # For split mappings (sum to 1.0)

    __table_args__ = (
        Index("idx_naics_bea_industry", "industry_id"),
        Index("idx_naics_bea_bea", "bea_industry_id"),
        CheckConstraint(
            "mapping_quality IN ('exact', 'aggregated', 'split', 'estimated')",
            name="ck_mapping_quality",
        ),
    )


# =============================================================================
# CENSUS CODE DIMENSION TABLES (Extracted Labels)
# =============================================================================


class DimIncomeBracket(NormalizedBase):
    """Income distribution brackets from census_income_distribution."""

    __tablename__ = "dim_income_bracket"

    bracket_id: Mapped[int] = mapped_column(primary_key=True)
    bracket_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    bracket_label: Mapped[str] = mapped_column(String(100), nullable=False)
    bracket_min_usd: Mapped[int | None] = mapped_column()
    bracket_max_usd: Mapped[int | None] = mapped_column()  # NULL = open-ended
    bracket_order: Mapped[int] = mapped_column(nullable=False)


class DimEmploymentStatus(NormalizedBase):
    """Employment status categories from census_employment_status."""

    __tablename__ = "dim_employment_status"

    status_id: Mapped[int] = mapped_column(primary_key=True)
    status_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    status_label: Mapped[str] = mapped_column(String(100), nullable=False)
    is_labor_force: Mapped[bool | None] = mapped_column()
    is_employed: Mapped[bool | None] = mapped_column()
    status_order: Mapped[int] = mapped_column(nullable=False)


class DimWorkerClass(NormalizedBase):
    """Class of worker from census_worker_class with Marxian mapping."""

    __tablename__ = "dim_worker_class"

    class_id: Mapped[int] = mapped_column(primary_key=True)
    class_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    class_label: Mapped[str] = mapped_column(String(200), nullable=False)
    marxian_class: Mapped[str | None] = mapped_column(String(20))
    class_order: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        CheckConstraint(
            "marxian_class IN ('proletariat', 'petty_bourgeois', 'state_worker', 'unpaid_labor') "
            "OR marxian_class IS NULL",
            name="ck_marxian_class",
        ),
    )


class DimOccupation(NormalizedBase):
    """Occupation codes from census_occupation with labor type classification."""

    __tablename__ = "dim_occupation"

    occupation_id: Mapped[int] = mapped_column(primary_key=True)
    occupation_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    occupation_label: Mapped[str] = mapped_column(String(300), nullable=False)
    occupation_category: Mapped[str | None] = mapped_column(String(100))
    labor_type: Mapped[str | None] = mapped_column(String(20))
    occupation_order: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_occupation_category", "occupation_category"),
        Index("idx_occupation_labor_type", "labor_type"),
        CheckConstraint(
            "labor_type IN ('productive', 'unproductive', 'reproductive', 'managerial') "
            "OR labor_type IS NULL",
            name="ck_labor_type",
        ),
    )


class DimEducationLevel(NormalizedBase):
    """Education levels from census_education."""

    __tablename__ = "dim_education_level"

    level_id: Mapped[int] = mapped_column(primary_key=True)
    level_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    level_label: Mapped[str] = mapped_column(String(200), nullable=False)
    years_of_schooling: Mapped[int | None] = mapped_column()
    level_order: Mapped[int] = mapped_column(nullable=False)


class DimHousingTenure(NormalizedBase):
    """Housing tenure types (Owner/Renter)."""

    __tablename__ = "dim_housing_tenure"

    tenure_id: Mapped[int] = mapped_column(primary_key=True)
    tenure_type: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    tenure_label: Mapped[str] = mapped_column(String(100), nullable=False)
    is_owner: Mapped[bool] = mapped_column(nullable=False)


class DimRentBurden(NormalizedBase):
    """Rent burden brackets from census_rent_burden."""

    __tablename__ = "dim_rent_burden"

    burden_id: Mapped[int] = mapped_column(primary_key=True)
    bracket_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    burden_bracket: Mapped[str] = mapped_column(String(50), nullable=False)
    burden_min_pct: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))
    burden_max_pct: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))
    is_cost_burdened: Mapped[bool | None] = mapped_column()  # >= 30%
    is_severely_burdened: Mapped[bool | None] = mapped_column()  # >= 50%
    bracket_order: Mapped[int] = mapped_column(nullable=False)


class DimPovertyCategory(NormalizedBase):
    """Poverty categories from census_poverty."""

    __tablename__ = "dim_poverty_category"

    category_id: Mapped[int] = mapped_column(primary_key=True)
    category_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    category_label: Mapped[str] = mapped_column(String(200), nullable=False)
    is_below_poverty: Mapped[bool | None] = mapped_column()
    category_order: Mapped[int] = mapped_column(nullable=False)


# =============================================================================
# ENERGY DIMENSION TABLES
# =============================================================================


# =============================================================================
# FRED DIMENSION TABLES
# =============================================================================


class DimWealthClass(NormalizedBase):
    """FRED wealth percentile classes with Babylon mapping."""

    __tablename__ = "dim_wealth_class"

    wealth_class_id: Mapped[int] = mapped_column(primary_key=True)
    percentile_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    percentile_label: Mapped[str] = mapped_column(String(100), nullable=False)
    babylon_class: Mapped[str | None] = mapped_column(String(50))  # Maps to Babylon social class


class DimAssetCategory(NormalizedBase):
    """FRED asset categories with Marxian interpretation."""

    __tablename__ = "dim_asset_category"

    category_id: Mapped[int] = mapped_column(primary_key=True)
    category_code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    category_label: Mapped[str] = mapped_column(String(100), nullable=False)
    marxian_interpretation: Mapped[str | None] = mapped_column(Text)


class DimFredSeries(NormalizedBase):
    """FRED series metadata."""

    __tablename__ = "dim_fred_series"

    series_id: Mapped[int] = mapped_column(primary_key=True)
    series_code: Mapped[str] = mapped_column(String(50), unique=True, nullable=False)
    title: Mapped[str] = mapped_column(String(300), nullable=False)
    units: Mapped[str | None] = mapped_column(String(50))
    frequency: Mapped[str | None] = mapped_column(String(20))
    seasonal_adjustment: Mapped[str | None] = mapped_column(String(50))
    source: Mapped[str | None] = mapped_column(String(100))


# =============================================================================
# COMMODITIES DIMENSION TABLES
# =============================================================================


class DimCommodity(NormalizedBase):
    """Critical commodities with Marxian interpretation."""

    __tablename__ = "dim_commodity"

    commodity_id: Mapped[int] = mapped_column(primary_key=True)
    code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    is_critical: Mapped[bool | None] = mapped_column()
    primary_applications: Mapped[str | None] = mapped_column(Text)
    marxian_interpretation: Mapped[str | None] = mapped_column(Text)


class DimCommodityMetric(NormalizedBase):
    """Commodity metrics with categories."""

    __tablename__ = "dim_commodity_metric"

    metric_id: Mapped[int] = mapped_column(primary_key=True)
    code: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    name: Mapped[str] = mapped_column(String(200), nullable=False)
    units: Mapped[str | None] = mapped_column(String(50))
    category: Mapped[str | None] = mapped_column(String(50))  # Production, Reserves, Price
    marxian_interpretation: Mapped[str | None] = mapped_column(Text)


# =============================================================================
# METADATA DIMENSION TABLES
# =============================================================================


class DimTime(NormalizedBase):
    """Calendar dimension for temporal analysis."""

    __tablename__ = "dim_time"

    time_id: Mapped[int] = mapped_column(primary_key=True)
    year: Mapped[int] = mapped_column(nullable=False)
    month: Mapped[int | None] = mapped_column()  # NULL for annual data
    quarter: Mapped[int | None] = mapped_column()  # 1-4
    is_annual: Mapped[bool] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_time_year", "year"),
        Index("idx_time_annual", "is_annual"),
    )


class DimGender(NormalizedBase):
    """Gender dimension for disaggregated census data."""

    __tablename__ = "dim_gender"

    gender_id: Mapped[int] = mapped_column(primary_key=True)
    gender_code: Mapped[str] = mapped_column(String(10), unique=True, nullable=False)
    gender_label: Mapped[str] = mapped_column(String(20), nullable=False)


class DimDataSource(NormalizedBase):
    """Data source tracking for provenance."""

    __tablename__ = "dim_data_source"

    source_id: Mapped[int] = mapped_column(primary_key=True)
    source_code: Mapped[str] = mapped_column(String(50), unique=True, nullable=False)
    source_name: Mapped[str] = mapped_column(String(200), nullable=False)
    source_url: Mapped[str | None] = mapped_column(String(500))
    description: Mapped[str | None] = mapped_column(Text)
    source_year: Mapped[int | None] = mapped_column()
    source_agency: Mapped[str | None] = mapped_column(String(100))
    coverage_start_year: Mapped[int | None] = mapped_column()
    coverage_end_year: Mapped[int | None] = mapped_column()


class DimRace(NormalizedBase):
    """Race/ethnicity dimension following Census A-I suffix scheme.

    Enables analysis of national oppression and class composition by race,
    critical for understanding material conditions in MLM-TW framework.

    Census Race Codes:
        A = White alone
        B = Black or African American alone
        C = American Indian and Alaska Native alone
        D = Asian alone
        E = Native Hawaiian and Other Pacific Islander alone
        F = Some other race alone
        G = Two or more races
        H = White alone, not Hispanic or Latino
        I = Hispanic or Latino
        T = Total (all races, base table)
    """

    __tablename__ = "dim_race"

    race_id: Mapped[int] = mapped_column(primary_key=True)
    race_code: Mapped[str] = mapped_column(String(1), unique=True, nullable=False)
    race_name: Mapped[str] = mapped_column(String(100), nullable=False)
    race_short_name: Mapped[str] = mapped_column(String(20), nullable=False)
    is_hispanic_ethnicity: Mapped[bool] = mapped_column(default=False)
    is_indigenous: Mapped[bool] = mapped_column(default=False)
    display_order: Mapped[int] = mapped_column(default=0)

    __table_args__ = (
        Index("idx_race_code", "race_code"),
        CheckConstraint(
            "race_code IN ('A', 'B', 'C', 'D', 'E', 'F', 'G', 'H', 'I', 'T')",
            name="ck_race_code_valid",
        ),
    )


# =============================================================================
# INGEST CHECKPOINT TABLE (for resume capability)
# =============================================================================


class IngestCheckpoint(NormalizedBase):
    """Tracks completed data ingestion work units for resume capability.

    Each row represents a completed (source, year, state, table, race) tuple.
    Loaders query this before fetching data to skip completed work.

    Note:
        This table is managed by DataLoader checkpoint helpers, not by
        individual loaders. Use _is_completed(), _mark_completed(), and
        _clear_checkpoints() from DataLoader base class.
    """

    __tablename__ = "ingest_checkpoint"

    checkpoint_id: Mapped[int] = mapped_column(primary_key=True)
    source_code: Mapped[str] = mapped_column(String(50), nullable=False)  # e.g., "census", "qcew"
    year: Mapped[int] = mapped_column(Integer, nullable=False)
    state_fips: Mapped[str] = mapped_column(String(2), nullable=False)
    table_id: Mapped[str] = mapped_column(String(20), nullable=False)  # e.g., "B19001", "B19001A"
    race_code: Mapped[str] = mapped_column(String(1), nullable=False, default="T")  # T, A-I
    row_count: Mapped[int] = mapped_column(Integer, nullable=False, default=0)
    completed_at: Mapped[datetime] = mapped_column(
        DateTime, nullable=False, server_default=func.now()
    )

    __table_args__ = (
        UniqueConstraint(
            "source_code",
            "year",
            "state_fips",
            "table_id",
            "race_code",
            name="uq_ingest_checkpoint",
        ),
        Index("idx_checkpoint_source_year", "source_code", "year"),
    )


# =============================================================================
# COERCIVE INFRASTRUCTURE DIMENSION TABLES
# =============================================================================


class DimCoerciveType(NormalizedBase):
    """Coercive infrastructure type classification.

    Categorizes state power apparatus by function and command chain:
    - Carceral: prisons, jails, detention centers
    - Enforcement: police stations, sheriff offices
    - Military: bases, armories, installations
    """

    __tablename__ = "dim_coercive_type"

    coercive_type_id: Mapped[int] = mapped_column(primary_key=True)
    code: Mapped[str] = mapped_column(String(30), unique=True, nullable=False)
    name: Mapped[str] = mapped_column(String(100), nullable=False)
    category: Mapped[str] = mapped_column(
        String(20), nullable=False
    )  # carceral, enforcement, military
    command_chain: Mapped[str] = mapped_column(
        String(20), nullable=False
    )  # federal, state, local, mixed

    __table_args__ = (
        Index("idx_coercive_category", "category"),
        Index("idx_coercive_command", "command_chain"),
        CheckConstraint(
            "category IN ('carceral', 'enforcement', 'military')",
            name="ck_coercive_category",
        ),
        CheckConstraint(
            "command_chain IN ('federal', 'state', 'local', 'mixed')",
            name="ck_coercive_command_chain",
        ),
    )


# =============================================================================
# CENSUS FACT TABLES
# =============================================================================


class FactCensusIncome(NormalizedBase):
    """Income distribution by county, time, and race."""

    __tablename__ = "fact_census_income"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    bracket_id: Mapped[int] = mapped_column(
        ForeignKey("dim_income_bracket.bracket_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    household_count: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_income_county", "county_id"),
        Index("idx_income_time", "time_id"),
        Index("idx_income_race", "race_id"),
    )


class FactCensusMedianIncome(NormalizedBase):
    """Median income by county, time, and race."""

    __tablename__ = "fact_census_median_income"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    median_income_usd: Mapped[Decimal] = mapped_column(Numeric(10, 2), nullable=False)

    __table_args__ = (
        Index("idx_median_income_time", "time_id"),
        Index("idx_median_income_race", "race_id"),
    )


class FactCensusEmployment(NormalizedBase):
    """Employment status by county, time, and race."""

    __tablename__ = "fact_census_employment"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    status_id: Mapped[int] = mapped_column(
        ForeignKey("dim_employment_status.status_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    person_count: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_employment_time", "time_id"),
        Index("idx_employment_race", "race_id"),
    )


class FactCensusWorkerClass(NormalizedBase):
    """Class of worker by county, gender, time, and race."""

    __tablename__ = "fact_census_worker_class"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    gender_id: Mapped[int] = mapped_column(ForeignKey("dim_gender.gender_id"), primary_key=True)
    class_id: Mapped[int] = mapped_column(ForeignKey("dim_worker_class.class_id"), primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    worker_count: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_worker_class_time", "time_id"),
        Index("idx_worker_class_race", "race_id"),
    )


class FactCensusHours(NormalizedBase):
    """Hours worked by county, gender, time, and race."""

    __tablename__ = "fact_census_hours"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    gender_id: Mapped[int] = mapped_column(ForeignKey("dim_gender.gender_id"), primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    aggregate_hours: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))
    mean_hours: Mapped[Decimal | None] = mapped_column(Numeric(6, 2))

    __table_args__ = (
        Index("idx_hours_time", "time_id"),
        Index("idx_hours_race", "race_id"),
    )


class FactCensusHousing(NormalizedBase):
    """Housing tenure by county, time, and race."""

    __tablename__ = "fact_census_housing"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    tenure_id: Mapped[int] = mapped_column(
        ForeignKey("dim_housing_tenure.tenure_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    household_count: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_housing_time", "time_id"),
        Index("idx_housing_race", "race_id"),
    )


class FactCensusRent(NormalizedBase):
    """Median rent by county, time, and race."""

    __tablename__ = "fact_census_rent"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    median_rent_usd: Mapped[Decimal] = mapped_column(Numeric(8, 2), nullable=False)

    __table_args__ = (
        Index("idx_rent_time", "time_id"),
        Index("idx_rent_race", "race_id"),
    )


class FactCensusRentBurden(NormalizedBase):
    """Rent burden distribution by county, time, and race."""

    __tablename__ = "fact_census_rent_burden"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    burden_id: Mapped[int] = mapped_column(
        ForeignKey("dim_rent_burden.burden_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    household_count: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_rent_burden_time", "time_id"),
        Index("idx_rent_burden_race", "race_id"),
    )


class FactCensusEducation(NormalizedBase):
    """Education attainment by county, time, and race."""

    __tablename__ = "fact_census_education"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    level_id: Mapped[int] = mapped_column(
        ForeignKey("dim_education_level.level_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    person_count: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_education_time", "time_id"),
        Index("idx_education_race", "race_id"),
    )


class FactCensusPoverty(NormalizedBase):
    """Poverty status by county, time, and race."""

    __tablename__ = "fact_census_poverty"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    category_id: Mapped[int] = mapped_column(
        ForeignKey("dim_poverty_category.category_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    person_count: Mapped[int] = mapped_column(nullable=False)

    __table_args__ = (
        Index("idx_poverty_time", "time_id"),
        Index("idx_poverty_race", "race_id"),
    )


class FactCensusIncomeSources(NormalizedBase):
    """Income sources (wage/self-employment/investment) by county, time, and race."""

    __tablename__ = "fact_census_income_sources"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    race_id: Mapped[int] = mapped_column(ForeignKey("dim_race.race_id"), primary_key=True)
    total_households: Mapped[int | None] = mapped_column()
    with_wage_income: Mapped[int | None] = mapped_column()
    with_self_employment_income: Mapped[int | None] = mapped_column()
    with_investment_income: Mapped[int | None] = mapped_column()

    __table_args__ = (
        Index("idx_income_sources_time", "time_id"),
        Index("idx_income_sources_race", "race_id"),
    )


# =============================================================================
# QCEW / PRODUCTIVITY FACT TABLES
# =============================================================================


class FactQcewAnnual(NormalizedBase):
    """QCEW annual employment/wage data.

    Canonical grain (spec-067): county × 6-digit NAICS × ownership × year
    leaves only. Since spec-086, BLS-suppressed cells store imputed
    employment/wage magnitudes (``is_imputed = True``) instead of masking
    zeros; ``disclosure_code`` keeps the raw BLS flag.
    """

    __tablename__ = "fact_qcew_annual"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_industry.industry_id"), primary_key=True
    )
    ownership_id: Mapped[int] = mapped_column(
        ForeignKey("dim_ownership.ownership_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)

    establishments: Mapped[int | None] = mapped_column()
    employment: Mapped[int | None] = mapped_column()
    total_wages_usd: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))
    avg_weekly_wage_usd: Mapped[int | None] = mapped_column()
    avg_annual_pay_usd: Mapped[int | None] = mapped_column()
    lq_employment: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    lq_annual_pay: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    disclosure_code: Mapped[str | None] = mapped_column(String(5))
    is_imputed: Mapped[bool] = mapped_column(nullable=False, server_default=text("0"))

    __table_args__ = (
        Index("idx_qcew_county_time", "county_id", "time_id"),
        Index("idx_qcew_industry_time", "industry_id", "time_id"),
        Index("idx_qcew_ownership", "ownership_id"),
    )


class FactQcewCountyRollup(NormalizedBase):
    """BLS-published county reconciliation constraints (spec-086).

    One row per county × year × ownership holding the published rollup the
    imputed leaves in :class:`FactQcewAnnual` must sum to: the ``own_code``
    ``'0'`` row carries the county Total Covered figure (agglvl 70); rows
    for ``own_code`` 1/2/3/5 carry the per-ownership totals (agglvl 71).
    ``is_imputed`` marks the rare constraint that was itself reconstructed
    via the documented fallback (FR-015); such county-years are flagged
    low-confidence in the load audit report.
    """

    __tablename__ = "fact_qcew_county_rollup"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    ownership_id: Mapped[int] = mapped_column(
        ForeignKey("dim_ownership.ownership_id"), primary_key=True
    )

    establishments: Mapped[int | None] = mapped_column()
    employment: Mapped[int | None] = mapped_column()
    total_wages_usd: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))
    disclosure_code: Mapped[str | None] = mapped_column(String(5))
    is_imputed: Mapped[bool] = mapped_column(nullable=False, server_default=text("0"))


class FactQwiCountyFlow(NormalizedBase):
    """LEHD QWI county labor flows (PER-32).

    One row per county x NAICS (3/4-digit) x ownership x quarter, filtered to
    the all-worker total (sex/age/race/ethnicity/education/firm-age/firm-size
    total categories) from the R2026Q3 Michigan county files. Michigan's LEHD
    coverage spans 2000Q3-2021Q4, so this table serves the 2019/2020 benchmark
    baselines directly and carries no rows for the 2024 thin slice.

    QWI cells are noise-infused and subject to suppression: an empty source
    measure is stored NULL with its QWI status flag preserved in the companion
    ``status_*`` column. Nothing is imputed.

    Columns (QWI mnemonic in parentheses):
        employment_begin / employment_end / employment_stable: employment
            stocks (Emp / EmpEnd / EmpS)
        hires_all / hires_new / separations: worker flows (HirA / HirN / Sep)
        turnover_stable: stable-job turnover rate (TurnOvrS)
        firm_job_gains / firm_job_losses / firm_job_change: firm job flows
            (FrmJbGn / FrmJbLs / FrmJbC; FrmJbC is signed)
        avg_monthly_earnings_stable_usd: mean monthly earnings of stable jobs
            (EarnS) — an observed mean, not a median
        payroll_usd: total quarterly payroll (Payroll); heavily suppressed at
            this grain
    """

    __tablename__ = "fact_qwi_county_flow"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_industry.industry_id"), primary_key=True
    )
    ownership_id: Mapped[int] = mapped_column(
        ForeignKey("dim_ownership.ownership_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)

    employment_begin: Mapped[int | None] = mapped_column()
    employment_end: Mapped[int | None] = mapped_column()
    employment_stable: Mapped[int | None] = mapped_column()
    hires_all: Mapped[int | None] = mapped_column()
    hires_new: Mapped[int | None] = mapped_column()
    separations: Mapped[int | None] = mapped_column()
    turnover_stable: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    firm_job_gains: Mapped[int | None] = mapped_column()
    firm_job_losses: Mapped[int | None] = mapped_column()
    firm_job_change: Mapped[int | None] = mapped_column()
    avg_monthly_earnings_stable_usd: Mapped[int | None] = mapped_column()
    payroll_usd: Mapped[int | None] = mapped_column()

    status_employment_begin: Mapped[int | None] = mapped_column()
    status_employment_end: Mapped[int | None] = mapped_column()
    status_employment_stable: Mapped[int | None] = mapped_column()
    status_hires_all: Mapped[int | None] = mapped_column()
    status_hires_new: Mapped[int | None] = mapped_column()
    status_separations: Mapped[int | None] = mapped_column()
    status_turnover_stable: Mapped[int | None] = mapped_column()
    status_firm_job_gains: Mapped[int | None] = mapped_column()
    status_firm_job_losses: Mapped[int | None] = mapped_column()
    status_firm_job_change: Mapped[int | None] = mapped_column()
    status_avg_monthly_earnings_stable: Mapped[int | None] = mapped_column()
    status_payroll: Mapped[int | None] = mapped_column()

    __table_args__ = (
        Index("idx_qwi_county_time", "county_id", "time_id"),
        Index("idx_qwi_industry_time", "industry_id", "time_id"),
    )


class FactAsmManufacturingAnnual(NormalizedBase):
    """ASM manufacturing general statistics, national x NAICS x year (PER-30).

    One row per NAICS 2017 industry (sector '31-33' plus 3-6 digit
    manufacturing leaves — nested aggregates are distinct published
    observations, each kept) per year, from the 2022 Economic Census
    benchmark vintage (AM2231GS1: 2018-2021 revised). NATIONAL grain: ASM
    publishes no county/state inventory-by-stage product, so county-level
    inventory in Babylon is an explicitly derived quantity (national ratios
    x county QCEW shares), never an observation.

    Carries the inventory STOCKS the estate previously lacked (BEA holds
    value added; NIPA changes-in-private-inventories are flows): materials
    & supplies, work-in-process, finished goods and total, each beginning
    and end of year, in thousands of current USD. Plus shipments, total
    cost of materials, value added, employment, production-worker
    headcount/hours (thousands)/wages and annual payroll — the
    production-worker series are the observed variable-capital split.

    Empty measures are stored NULL with the ASM annotation flag preserved
    in the companion ``flag_*`` column; relative standard errors remain in
    the raw file. Nothing is imputed.
    """

    __tablename__ = "fact_asm_manufacturing_annual"

    industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_industry.industry_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)

    inventories_total_begin_kusd: Mapped[int | None] = mapped_column()
    inventories_total_end_kusd: Mapped[int | None] = mapped_column()
    inventories_materials_begin_kusd: Mapped[int | None] = mapped_column()
    inventories_materials_end_kusd: Mapped[int | None] = mapped_column()
    inventories_wip_begin_kusd: Mapped[int | None] = mapped_column()
    inventories_wip_end_kusd: Mapped[int | None] = mapped_column()
    inventories_finished_begin_kusd: Mapped[int | None] = mapped_column()
    inventories_finished_end_kusd: Mapped[int | None] = mapped_column()
    shipments_kusd: Mapped[int | None] = mapped_column()
    cost_materials_kusd: Mapped[int | None] = mapped_column()
    value_added_kusd: Mapped[int | None] = mapped_column()
    employment: Mapped[int | None] = mapped_column()
    production_workers: Mapped[int | None] = mapped_column()
    production_hours_thousands: Mapped[int | None] = mapped_column()
    payroll_kusd: Mapped[int | None] = mapped_column()
    production_wages_kusd: Mapped[int | None] = mapped_column()

    flag_inventories_total_begin: Mapped[str | None] = mapped_column(String(2))
    flag_inventories_total_end: Mapped[str | None] = mapped_column(String(2))
    flag_inventories_materials_begin: Mapped[str | None] = mapped_column(String(2))
    flag_inventories_materials_end: Mapped[str | None] = mapped_column(String(2))
    flag_inventories_wip_begin: Mapped[str | None] = mapped_column(String(2))
    flag_inventories_wip_end: Mapped[str | None] = mapped_column(String(2))
    flag_inventories_finished_begin: Mapped[str | None] = mapped_column(String(2))
    flag_inventories_finished_end: Mapped[str | None] = mapped_column(String(2))
    flag_shipments: Mapped[str | None] = mapped_column(String(2))
    flag_cost_materials: Mapped[str | None] = mapped_column(String(2))
    flag_value_added: Mapped[str | None] = mapped_column(String(2))
    flag_employment: Mapped[str | None] = mapped_column(String(2))
    flag_production_workers: Mapped[str | None] = mapped_column(String(2))
    flag_production_hours: Mapped[str | None] = mapped_column(String(2))
    flag_payroll: Mapped[str | None] = mapped_column(String(2))
    flag_production_wages: Mapped[str | None] = mapped_column(String(2))

    __table_args__ = (Index("idx_asm_mfg_time", "time_id"),)


class FactProductivityAnnual(NormalizedBase):
    """BLS productivity data for surplus value analysis."""

    __tablename__ = "fact_productivity_annual"

    industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_industry.industry_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)

    # Indices
    labor_productivity_index: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    hours_worked_index: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    hourly_compensation_index: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    unit_labor_costs_index: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    real_output_index: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))
    sectoral_output_index: Mapped[Decimal | None] = mapped_column(Numeric(10, 4))

    # Absolute measures (key for Marxian analysis)
    hours_worked_millions: Mapped[Decimal | None] = mapped_column(Numeric(12, 2))
    employment_thousands: Mapped[Decimal | None] = mapped_column(Numeric(10, 2))
    labor_compensation_millions_usd: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))
    sectoral_output_millions_usd: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))

    __table_args__ = (Index("idx_productivity_time", "time_id"),)


class FactBEANationalIndustry(NormalizedBase):
    """BEA national industry accounts - gross output, value added, intermediate inputs.

    Stores annual dollar values from BEA GDP-by-industry tables at the national level.
    The fundamental accounting identity holds:
        gross_output = intermediate_inputs + value_added

    This is the core data for deriving constant capital (c) and variable capital (v)
    at the national level. County-level decomposition uses these national ratios
    applied to county employment/wage data from QCEW.

    Columns:
        gross_output_millions: Total production (c + v + s in Marxian terms)
        intermediate_inputs_millions: Constant capital consumed (c)
        value_added_millions: New value created (v + s)

    Note:
        Values are in millions of current dollars (nominal). For real analysis,
        apply GDPDEF deflator from fact_fred_national.
    """

    __tablename__ = "fact_bea_national_industry"

    bea_industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_bea_industry.bea_industry_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)

    # BEA industry accounts (millions of dollars)
    gross_output_millions: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))
    intermediate_inputs_millions: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))
    value_added_millions: Mapped[Decimal | None] = mapped_column(Numeric(15, 2))

    # Spec-068: BEA publication vintage tracking (Clarification Q2 — latest-only
    # supersession policy; NULL permitted for partial-load tolerance during dev).
    vintage_published_date: Mapped[date | None] = mapped_column(Date, nullable=True)

    __table_args__ = (
        Index("idx_bea_national_time", "time_id"),
        Index("idx_bea_national_industry", "bea_industry_id"),
    )


class FactBEACountyGDP(NormalizedBase):
    """BEA county-level GDP by industry.

    Stores annual GDP values from BEA CAGDP2 table at the county level.
    Values are in millions of current dollars (nominal).

    The CAGDP2 table provides GDP by industry for all US counties.
    Industry granularity is coarser than national (~20 industries vs ~70).
    Uses bridge_naics_bea for mapping to national industry ratios.

    Note:
        Some county-industry combinations have suppressed data "(D)" due to
        disclosure limitations. These are stored as NULL.
    """

    __tablename__ = "fact_bea_county_gdp"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    bea_industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_bea_industry.bea_industry_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)

    # GDP in millions of dollars (converted from BEA's thousands)
    gdp_millions: Mapped[Decimal | None] = mapped_column(Numeric(12, 2))

    __table_args__ = (
        Index("idx_bea_county_time", "time_id"),
        Index("idx_bea_county_industry", "bea_industry_id"),
        Index("idx_bea_county_county", "county_id"),
    )


# =============================================================================
# TRADE FACT TABLES
# =============================================================================


class FactTradeMonthly(NormalizedBase):
    """Monthly trade flows with countries."""

    __tablename__ = "fact_trade_monthly"

    country_id: Mapped[int] = mapped_column(ForeignKey("dim_country.country_id"), primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    imports_usd_millions: Mapped[Decimal | None] = mapped_column(Numeric(12, 2))
    exports_usd_millions: Mapped[Decimal | None] = mapped_column(Numeric(12, 2))

    __table_args__ = (
        Index("idx_trade_country_time", "country_id", "time_id"),
        Index("idx_trade_time", "time_id"),
    )


class FactCountyExposureByExternal(NormalizedBase):
    """County import-exposure weights per external bloc and year (spec-100).

    The materialized ``county_exposure_by_external`` map named in
    :mod:`babylon.engine.systems.phi_distribution`. For each external bloc and
    annual ``time_id``, the per-county ``weight`` values sum to 1.0 (within
    1e-9); the Φ-distribution system splits a bloc's weekly imperial-rent
    inflow across counties by these weights (it rejects a non-unit sum rather
    than silently renormalize).

    Grounding (Constitution III.8): ``weight`` derives from BEA I-O import
    coefficients (``fact_bea_io_coefficient``, "Noncomparable imports" source)
    times QCEW county industry shares (``fact_qcew_annual`` via the
    ``bridge_naics_bea`` concordance). Because the reference DB has no
    bloc-by-industry trade resolution, the distribution is currently the same
    across blocs; it is stored per-bloc for the consumer contract and forward
    compatibility. Loaded by ``python -m babylon_data.exposure``
    (``mise run data:exposure``).
    """

    __tablename__ = "fact_county_exposure_by_external"

    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    external_country_id: Mapped[int] = mapped_column(
        ForeignKey("dim_country.country_id"), primary_key=True
    )
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    weight: Mapped[float] = mapped_column(Float, nullable=False)

    __table_args__ = (
        Index("idx_county_exposure_bloc_year", "time_id", "external_country_id"),
        Index("idx_county_exposure_county", "county_id"),
    )


class FactBilateralTradeAnnual(NormalizedBase):
    """Bloc-year aggregation of monthly bilateral trade (spec-100).

    Sums ``fact_trade_monthly`` (USD millions) to annual totals per country for
    each annual ``time_id``.

    **See spec-101 (consumer directive)**: this is a USD magnitude. spec-101 MUST
    map ``total_trade_usd_millions`` (or the imports/exports USD columns) to the
    engine's ``ExternalNode.bilateral_trade_value`` (USD). It MUST NOT map this
    table to ``ExternalNode.bilateral_trade_tons`` — that field is tonnage and
    needs FAF freight data (out of scope for spec-100; the spec's original
    "bilateral_trade_tons" name is superseded because the only bilateral trade
    source in the reference DB is monetary). See ``specs/100-county-exposure``
    research R8.
    """

    __tablename__ = "fact_bilateral_trade_annual"

    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    country_id: Mapped[int] = mapped_column(ForeignKey("dim_country.country_id"), primary_key=True)
    imports_usd_millions: Mapped[Decimal | None] = mapped_column(Numeric(18, 2))
    exports_usd_millions: Mapped[Decimal | None] = mapped_column(Numeric(18, 2))
    total_trade_usd_millions: Mapped[Decimal | None] = mapped_column(Numeric(18, 2))

    __table_args__ = (Index("idx_bilateral_trade_time", "time_id"),)


class FactRicciUnequalExchangeGvc(NormalizedBase):
    """Ricci global-value-chain unequal-exchange transfer series (P26 U5b).

    Re-ingestion of the checked-in ``babylon_ricci_final.csv`` artifact
    (region x flow-direction x transfer-type, years 1995/2000/2007/2009) as
    a genuine reference-DB table, grounding the sigma-gradient pipeline's
    ``world_stats`` per Director ruling (``ai/decisions/
    ADR165_p26_director_rulings_trade_slate.yaml`` D2/D3) — a DECLARED
    partial undo of the 2026-07-17 ADR076 R2 amputation. The CSV was, and
    remains, the canonical artifact (``data-artifacts.yaml``'s
    ``babylon_ricci_final`` register-mode entry); this table is a
    build-product mirror populated FROM it by ``tools/ingest_ricci_gvc.py``.

    Deliberately NOT named ``fact_ricci_unequal_exchange`` (the amputated
    table's retired name) — that string is reserved history, and reusing it
    risks confusion with the Postgres RUNTIME table
    ``immutable_reference_ricci_unequal`` (``persistence/sqlite_hydrator.py``
    / ``persistence/postgres_initialization.py``), which despite the
    similarly-shaped name actually hydrates UNRELATED Census
    country/dim data under a legacy label. The two estates are disjoint:
    this is a sqlite reference-DB build-product table; that is a Postgres
    runtime hydration target.

    ``ricci_gvc_id`` is a surrogate PK assigned in the checked-in CSV's own
    row order (deterministic; no combination of the CSV's own columns is a
    natural key for every row — ``edge_id`` repeats across distinct regions
    within the same ``year``/``source_table``, e.g. 1995's EMU and Western
    Europe INFLOW/GVC rows both carry ``edge_id="CORE_INFLOW_GVC"``).
    """

    __tablename__ = "fact_ricci_unequal_exchange_gvc"

    ricci_gvc_id: Mapped[int] = mapped_column(primary_key=True)
    year: Mapped[int] = mapped_column(Integer, nullable=False)
    region_name: Mapped[str] = mapped_column(String(64), nullable=False)
    region_type: Mapped[str] = mapped_column(String(20), nullable=False)
    flow_direction: Mapped[str] = mapped_column(String(10), nullable=False)
    transfer_type: Mapped[str] = mapped_column(String(10), nullable=False)
    value_usd_billions: Mapped[float] = mapped_column(Float, nullable=False)
    value_pct_gdp: Mapped[float | None] = mapped_column(Float)
    signed_value: Mapped[float] = mapped_column(Float, nullable=False)
    gvc_share_of_total: Mapped[float | None] = mapped_column(Float)
    source_table: Mapped[str] = mapped_column(String(32), nullable=False)
    source_priority: Mapped[int] = mapped_column(Integer, nullable=False)
    region_granularity: Mapped[int] = mapped_column(Integer, nullable=False)
    edge_id: Mapped[str] = mapped_column(String(32), nullable=False)

    __table_args__ = (
        Index("idx_ricci_gvc_year", "year"),
        Index("idx_ricci_gvc_region", "region_name"),
        CheckConstraint(
            "region_type IN ('CORE', 'SEMI_PERIPHERY', 'PERIPHERY')",
            name="ck_ricci_gvc_region_type",
        ),
        CheckConstraint(
            "flow_direction IN ('INFLOW', 'OUTFLOW')",
            name="ck_ricci_gvc_flow_direction",
        ),
        CheckConstraint(
            "transfer_type IN ('GVC', 'TOTAL')",
            name="ck_ricci_gvc_transfer_type",
        ),
    )


# =============================================================================
# ENERGY FACT TABLES
# =============================================================================


# =============================================================================
# FRED FACT TABLES
# =============================================================================


class FactFredNational(NormalizedBase):
    """FRED national economic data."""

    __tablename__ = "fact_fred_national"

    series_id: Mapped[int] = mapped_column(
        ForeignKey("dim_fred_series.series_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    value: Mapped[Decimal | None] = mapped_column(Numeric(20, 6))


class FactFredWealthLevels(NormalizedBase):
    """FRED wealth levels by percentile class and asset category."""

    __tablename__ = "fact_fred_wealth_levels"

    series_id: Mapped[int] = mapped_column(
        ForeignKey("dim_fred_series.series_id"), primary_key=True
    )
    wealth_class_id: Mapped[int] = mapped_column(
        ForeignKey("dim_wealth_class.wealth_class_id"), primary_key=True
    )
    category_id: Mapped[int] = mapped_column(
        ForeignKey("dim_asset_category.category_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    value_millions: Mapped[Decimal | None] = mapped_column(Numeric(20, 2))


class FactFredWealthShares(NormalizedBase):
    """FRED wealth shares by percentile class and asset category."""

    __tablename__ = "fact_fred_wealth_shares"

    series_id: Mapped[int] = mapped_column(
        ForeignKey("dim_fred_series.series_id"), primary_key=True
    )
    wealth_class_id: Mapped[int] = mapped_column(
        ForeignKey("dim_wealth_class.wealth_class_id"), primary_key=True
    )
    category_id: Mapped[int] = mapped_column(
        ForeignKey("dim_asset_category.category_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    share_percent: Mapped[Decimal | None] = mapped_column(Numeric(8, 4))


# =============================================================================
# COMMODITIES FACT TABLES
# =============================================================================


class FactCommodityObservation(NormalizedBase):
    """Commodity observation (production, reserves, price, etc.)."""

    __tablename__ = "fact_commodity_observation"

    commodity_id: Mapped[int] = mapped_column(
        ForeignKey("dim_commodity.commodity_id"), primary_key=True
    )
    metric_id: Mapped[int] = mapped_column(
        ForeignKey("dim_commodity_metric.metric_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    value: Mapped[Decimal | None] = mapped_column(Numeric(20, 6))
    value_text: Mapped[str | None] = mapped_column(String(100))  # For non-numeric values


# =============================================================================
# MATERIALS/MINERALS FACT TABLES
# =============================================================================


class FactStateMinerals(NormalizedBase):
    """State mineral production value."""

    __tablename__ = "fact_state_minerals"

    state_id: Mapped[int] = mapped_column(ForeignKey("dim_state.state_id"), primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    value_millions: Mapped[Decimal | None] = mapped_column(Numeric(12, 2))
    rank: Mapped[int | None] = mapped_column()
    percent_total: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))
    principal_commodities: Mapped[str | None] = mapped_column(Text)


class FactMineralProduction(NormalizedBase):
    """National mineral production by type (normalized from mineral_trends)."""

    __tablename__ = "fact_mineral_production"

    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    production_type: Mapped[str] = mapped_column(
        String(50), primary_key=True
    )  # metals, industrial, coal
    value_millions: Mapped[Decimal | None] = mapped_column(Numeric(12, 2))


class FactMineralEmployment(NormalizedBase):
    """Mineral industry employment (normalized from mineral_trends)."""

    __tablename__ = "fact_mineral_employment"

    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)
    sector: Mapped[str] = mapped_column(
        String(50), primary_key=True
    )  # coal, nonfuel, chemicals, etc.
    employment_thousands: Mapped[Decimal | None] = mapped_column(Numeric(10, 2))
    avg_weekly_earnings_usd: Mapped[Decimal | None] = mapped_column(Numeric(10, 2))


# =============================================================================
# CIRCULATORY SYSTEM FACT TABLES
# =============================================================================


class FactCoerciveInfrastructure(NormalizedBase):
    """County-level coercive infrastructure capacity.

    Aggregates facility-level data from HIFLD (prisons, police) and MIRTA
    (military) to county-level counts and capacity metrics.
    """

    __tablename__ = "fact_coercive_infrastructure"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    coercive_type_id: Mapped[int] = mapped_column(
        ForeignKey("dim_coercive_type.coercive_type_id"), primary_key=True
    )
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    facility_count: Mapped[int] = mapped_column(default=0)
    total_capacity: Mapped[int | None] = mapped_column()  # beds, personnel, etc.

    __table_args__ = (
        Index("idx_coercive_county", "county_id"),
        Index("idx_coercive_type_fact", "coercive_type_id"),
    )


class FactBroadbandCoverage(NormalizedBase):
    """County-level broadband coverage metrics from FCC BDC.

    Tracks broadband access at multiple speed tiers for digital divide analysis.
    """

    __tablename__ = "fact_broadband_coverage"

    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), primary_key=True)
    source_id: Mapped[int] = mapped_column(
        ForeignKey("dim_data_source.source_id"), primary_key=True
    )
    pct_25_3: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))  # % with 25/3 Mbps
    pct_100_20: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))  # % with 100/20 Mbps
    pct_1000_100: Mapped[Decimal | None] = mapped_column(Numeric(5, 2))  # % with gigabit
    provider_count: Mapped[int | None] = mapped_column()

    __table_args__ = (Index("idx_broadband_county", "county_id"),)


class FactCommodityFlow(NormalizedBase):
    """State-level commodity flows from Census CFS, allocated to counties.

    Stores origin-destination commodity flow data from the Commodity Flow Survey.
    State-level CFS data is disaggregated to county level using allocation weights
    from DimGeographicHierarchy.

    Flow values are in millions USD (value_millions) and thousands of tons (tons_thousands).
    """

    __tablename__ = "fact_commodity_flow"

    flow_id: Mapped[int] = mapped_column(primary_key=True)
    origin_county_id: Mapped[int] = mapped_column(
        ForeignKey("dim_county.county_id"), nullable=False
    )
    dest_county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    sctg_id: Mapped[int] = mapped_column(ForeignKey("dim_sctg_commodity.sctg_id"), nullable=False)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)
    year: Mapped[int] = mapped_column(nullable=False)
    value_millions: Mapped[Decimal | None] = mapped_column(Numeric(14, 2))  # USD millions
    tons_thousands: Mapped[Decimal | None] = mapped_column(Numeric(14, 2))  # Thousands of tons
    ton_miles_millions: Mapped[Decimal | None] = mapped_column(Numeric(14, 2))  # Millions ton-miles
    mode_code: Mapped[str | None] = mapped_column(String(10))  # Transport mode

    __table_args__ = (
        Index("idx_commodity_flow_origin", "origin_county_id"),
        Index("idx_commodity_flow_dest", "dest_county_id"),
        Index("idx_commodity_flow_sctg", "sctg_id"),
        Index("idx_commodity_flow_year", "year"),
    )


# =============================================================================
# LODES CROSSWALK
# =============================================================================


class FactLodesCommuterFlow(NormalizedBase):
    """County-to-county commuter flow data from LEHD LODES Origin-Destination.

    Aggregates LODES OD (Origin-Destination) block-level data to county pairs.
    Each row represents commuter flows from home county to work county for a year.

    Data Source:
        LEHD LODES8 Origin-Destination files
        https://lehd.ces.census.gov/data/lodes/LODES8/
        Format: {state}_od_main_JT00_{year}.csv.gz

    Key Fields:
        - home_county_id: Where workers LIVE
        - work_county_id: Where workers WORK
        - total_jobs (S000): Total number of jobs
        - Age breakdown: SA01 (29 and under), SA02 (30-54), SA03 (55+)
        - Earnings breakdown: SE01 ($1250/mo or less), SE02 ($1251-$3333/mo),
          SE03 (>$3333/mo)

    Use Cases:
        - Commuter-adjusted throughput: τ_residence vs τ_workplace
        - Identify bedroom communities (high outbound) vs job centers (high inbound)
        - Labor market boundary analysis

    Note:
        Data is pre-aggregated at load time (block→county) for query performance.
        LODES has 18-24 month publication lag; 2022 data typically available mid-2024.
    """

    __tablename__ = "fact_lodes_commuter_flow"

    home_county_id: Mapped[int] = mapped_column(
        ForeignKey("dim_county.county_id"), primary_key=True
    )
    work_county_id: Mapped[int] = mapped_column(
        ForeignKey("dim_county.county_id"), primary_key=True
    )
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), primary_key=True)

    # Total jobs (S000)
    total_jobs: Mapped[int] = mapped_column(nullable=False)

    # Age breakdown (SA01, SA02, SA03)
    jobs_age_29_under: Mapped[int | None] = mapped_column()  # SA01
    jobs_age_30_54: Mapped[int | None] = mapped_column()  # SA02
    jobs_age_55_plus: Mapped[int | None] = mapped_column()  # SA03

    # Earnings breakdown (SE01, SE02, SE03)
    jobs_earn_low: Mapped[int | None] = mapped_column()  # SE01: $1250/mo or less
    jobs_earn_mid: Mapped[int | None] = mapped_column()  # SE02: $1251-$3333/mo
    jobs_earn_high: Mapped[int | None] = mapped_column()  # SE03: >$3333/mo

    __table_args__ = (
        # Query: "Who works in county X?" (inbound commuters)
        Index("idx_lodes_work_time", "work_county_id", "time_id"),
        # Query: "Where do county X residents work?" (outbound commuters)
        Index("idx_lodes_home_time", "home_county_id", "time_id"),
    )


# =============================================================================
# EMPLOYMENT INDUSTRY
# =============================================================================


class DimEmploymentArea(NormalizedBase):
    """Employment area dimension for BLS QCEW by-area data.

    Represents geographic areas at various levels: national, state, county,
    MSA, CSA. Links to state and county dimensions where applicable.
    """

    __tablename__ = "dim_employment_area"

    area_id: Mapped[int] = mapped_column(primary_key=True)
    area_code: Mapped[str] = mapped_column(String(10), nullable=False)
    area_name: Mapped[str] = mapped_column(String(255), nullable=False)
    area_type: Mapped[str] = mapped_column(
        String(20), nullable=False
    )  # national/state/county/msa/csa
    state_id: Mapped[int | None] = mapped_column(ForeignKey("dim_state.state_id"))
    county_id: Mapped[int | None] = mapped_column(ForeignKey("dim_county.county_id"))
    cbsa_code: Mapped[str | None] = mapped_column(String(5))
    csa_code: Mapped[str | None] = mapped_column(String(3))

    __table_args__ = (
        Index("idx_employment_area_code", "area_code"),
        Index("idx_employment_area_type", "area_type"),
    )


# =============================================================================
# DOT HPMS ROAD SEGMENTS
# =============================================================================


class FactHpmsRoadSegment(NormalizedBase):
    """Road segment characteristics from FHWA HPMS.

    Highway Performance Monitoring System data at segment level,
    including traffic (AADT), geometry, and functional classification.
    """

    __tablename__ = "fact_hpms_road_segment"

    segment_id: Mapped[int] = mapped_column(primary_key=True)
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    state_id: Mapped[int] = mapped_column(ForeignKey("dim_state.state_id"), nullable=False)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)
    time_id: Mapped[int | None] = mapped_column(ForeignKey("dim_time.time_id"))

    route_id: Mapped[str | None] = mapped_column(String(100))
    route_number: Mapped[str | None] = mapped_column(String(10))
    route_signing: Mapped[str | None] = mapped_column(String(2))
    route_qualifier: Mapped[str | None] = mapped_column(String(2))
    functional_system: Mapped[int | None] = mapped_column()
    facility_type: Mapped[int | None] = mapped_column()

    aadt: Mapped[int | None] = mapped_column()  # Annual Average Daily Traffic
    aadt_single_unit: Mapped[int | None] = mapped_column()
    aadt_combination: Mapped[int | None] = mapped_column()
    speed_limit: Mapped[int | None] = mapped_column()
    through_lanes: Mapped[int | None] = mapped_column()
    lane_width: Mapped[Decimal | None] = mapped_column(Numeric(6, 2))
    section_length: Mapped[Decimal | None] = mapped_column(Numeric(12, 6))

    nhs: Mapped[int | None] = mapped_column()  # National Highway System
    nhfn: Mapped[int | None] = mapped_column()  # National Highway Freight Network
    urban_id: Mapped[str | None] = mapped_column(String(10))
    year_record: Mapped[int | None] = mapped_column()
    shape_id: Mapped[str | None] = mapped_column(String(50))
    sample_id: Mapped[str | None] = mapped_column(String(50))
    geometry_wkt: Mapped[str | None] = mapped_column(Text)

    __table_args__ = (
        Index("idx_hpms_county", "county_id"),
        Index("idx_hpms_state", "state_id"),
        Index("idx_hpms_year", "year_record"),
    )


# =============================================================================
# ATUS REPRODUCTIVE LABOR TABLES
# =============================================================================


# =============================================================================
# CAPITAL VOLUME I FACT TABLES (Feature 021)
# =============================================================================


class FactBLSUnemploymentDecomposition(NormalizedBase):
    """County-level unemployment decomposition from BLS LAUS.

    Stores detailed labor force status decomposition needed for reserve army
    composition analysis. U-3 approximates floating reserve, (U-6 - U-3)
    approximates latent reserve, PTER approximates stagnant reserve.

    Feature 021: Capital Volume I Production Dynamics (FR-001, FR-014).
    """

    __tablename__ = "fact_bls_unemployment_decomposition"

    fact_id: Mapped[int] = mapped_column(primary_key=True)
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), nullable=False)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)

    labor_force: Mapped[int] = mapped_column(Integer, nullable=False)
    employed: Mapped[int] = mapped_column(Integer, nullable=False)
    unemployed_u3: Mapped[int] = mapped_column(Integer, nullable=False)
    unemployed_u6: Mapped[int] = mapped_column(Integer, nullable=False)
    part_time_economic: Mapped[int] = mapped_column(Integer, nullable=False)
    discouraged: Mapped[int] = mapped_column(Integer, nullable=False)
    marginally_attached: Mapped[int] = mapped_column(Integer, nullable=False)

    __table_args__ = (
        Index("idx_bls_unemp_county", "county_id"),
        Index("idx_bls_unemp_time", "time_id"),
        UniqueConstraint("county_id", "time_id", name="uq_bls_unemp_county_time"),
    )


class FactEvictionLabFiling(NormalizedBase):
    """County-level eviction filings and executions from Eviction Lab.

    Provides eviction activity data for dispossession event tracking and
    territorial dispossession intensity computation.

    Feature 021: Capital Volume I Production Dynamics (FR-005, FR-015).
    """

    __tablename__ = "fact_eviction_lab_filing"

    fact_id: Mapped[int] = mapped_column(primary_key=True)
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), nullable=False)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)

    filings: Mapped[int] = mapped_column(Integer, nullable=False)
    executions: Mapped[int] = mapped_column(Integer, nullable=False)
    filing_rate: Mapped[Decimal] = mapped_column(Numeric(8, 6), nullable=False)
    execution_rate: Mapped[Decimal] = mapped_column(Numeric(8, 6), nullable=False)
    renter_households: Mapped[int] = mapped_column(Integer, nullable=False)

    __table_args__ = (
        Index("idx_eviction_county", "county_id"),
        Index("idx_eviction_time", "time_id"),
        UniqueConstraint("county_id", "time_id", name="uq_eviction_county_time"),
    )


class FactForeclosureRate(NormalizedBase):
    """County-level foreclosure rates from HUD/FRED/state sources.

    Provides foreclosure activity data for dispossession event tracking.
    Critical for the 2008-2012 crisis period in the Detroit metro area.

    Feature 021: Capital Volume I Production Dynamics (FR-005, FR-016).
    """

    __tablename__ = "fact_foreclosure_rate"

    fact_id: Mapped[int] = mapped_column(primary_key=True)
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), nullable=False)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)

    filings: Mapped[int] = mapped_column(Integer, nullable=False)
    completions: Mapped[int] = mapped_column(Integer, nullable=False)
    filing_rate: Mapped[Decimal] = mapped_column(Numeric(8, 6), nullable=False)
    completion_rate: Mapped[Decimal] = mapped_column(Numeric(8, 6), nullable=False)
    mortgaged_units: Mapped[int] = mapped_column(Integer, nullable=False)

    __table_args__ = (
        Index("idx_foreclosure_county", "county_id"),
        Index("idx_foreclosure_time", "time_id"),
        UniqueConstraint("county_id", "time_id", name="uq_foreclosure_county_time"),
    )


class FactCensusInstitutionalOwnership(NormalizedBase):
    """County-level housing institutional ownership from Census ACS.

    Tracks ownership patterns and renter migration for dispossession
    intensity computation and territorial value transfer analysis.

    Feature 021: Capital Volume I Production Dynamics (FR-005, FR-017).
    """

    __tablename__ = "fact_census_institutional_ownership"

    fact_id: Mapped[int] = mapped_column(primary_key=True)
    county_id: Mapped[int] = mapped_column(ForeignKey("dim_county.county_id"), nullable=False)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), nullable=False)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)

    total_units: Mapped[int] = mapped_column(Integer, nullable=False)
    owner_occupied: Mapped[int] = mapped_column(Integer, nullable=False)
    renter_occupied: Mapped[int] = mapped_column(Integer, nullable=False)
    institutional_owned: Mapped[int] = mapped_column(Integer, nullable=False)
    absentee_owned: Mapped[int] = mapped_column(Integer, nullable=False)
    net_migration_renters: Mapped[int] = mapped_column(Integer, nullable=False)

    __table_args__ = (
        Index("idx_inst_ownership_county", "county_id"),
        Index("idx_inst_ownership_time", "time_id"),
        UniqueConstraint("county_id", "time_id", name="uq_inst_ownership_county_time"),
    )


# =============================================================================
# BEA I-O COEFFICIENT TABLES (Feature 025: Tensor Hierarchy)
# =============================================================================


class DimBEAIOTableType(NormalizedBase):
    """BEA input-output table type dimension.

    Classifies which BEA I-O table a coefficient was derived from.
    Supports Use, Make, Supply, and Total Requirements tables.

    Feature 025: Tensor Hierarchy (T009).
    """

    __tablename__ = "dim_bea_io_table_type"

    id: Mapped[int] = mapped_column(primary_key=True)
    table_type: Mapped[str] = mapped_column(String(20), unique=True, nullable=False)
    description: Mapped[str | None] = mapped_column(Text)

    __table_args__ = (
        CheckConstraint(
            "table_type IN ('USE', 'MAKE', 'SUPPLY', 'TOTAL_REQ', 'IMPORT_USE')",
            name="ck_bea_io_table_type_valid",
        ),
    )


class FactBEAIOCoefficient(NormalizedBase):
    """BEA I-O direct requirements coefficients by year and industry pair.

    Stores A[i,j]: value of industry i output required per dollar of industry j
    output, from BEA Use table (direct requirements). One row per (year, table_type,
    source_industry, target_industry) combination.

    Feature 025: Tensor Hierarchy (T009, T019).
    """

    __tablename__ = "fact_bea_io_coefficient"

    id: Mapped[int] = mapped_column(primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), nullable=False)
    table_type_id: Mapped[int] = mapped_column(
        ForeignKey("dim_bea_io_table_type.id"), nullable=False
    )
    source_industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_bea_industry.bea_industry_id"), nullable=False
    )
    target_industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_bea_industry.bea_industry_id"), nullable=False
    )
    coefficient: Mapped[float] = mapped_column(Float, nullable=False)

    # Spec-068: BEA publication vintage tracking (Clarification Q2).
    vintage_published_date: Mapped[date | None] = mapped_column(Date, nullable=True)

    __table_args__ = (
        Index("idx_bea_io_coeff_time", "time_id"),
        Index("idx_bea_io_coeff_table_type", "table_type_id"),
        Index("idx_bea_io_coeff_source", "source_industry_id"),
        Index("idx_bea_io_coeff_target", "target_industry_id"),
        UniqueConstraint(
            "time_id",
            "table_type_id",
            "source_industry_id",
            "target_industry_id",
            name="uq_bea_io_coeff",
        ),
    )


class FactFAFCommodityFlow(NormalizedBase):
    """BTS FAF commodity flows at CFS Area resolution.

    Stores origin-destination flows from the Bureau of Transportation Statistics
    Freight Analysis Framework (FAF5). Uses CFS Area FKs (not county FKs) to
    preserve FAF's native ~130-area geographic resolution.

    Distinct from fact_commodity_flow (county-level Census CFS data).

    Feature 025: Tensor Hierarchy (T009, T036).
    """

    __tablename__ = "fact_faf_commodity_flow"

    flow_id: Mapped[int] = mapped_column(primary_key=True)
    origin_cfs_area_id: Mapped[int] = mapped_column(
        ForeignKey("dim_cfs_area.cfs_area_id"), nullable=False
    )
    dest_cfs_area_id: Mapped[int] = mapped_column(
        ForeignKey("dim_cfs_area.cfs_area_id"), nullable=False
    )
    sctg_id: Mapped[int] = mapped_column(ForeignKey("dim_sctg_commodity.sctg_id"), nullable=False)
    source_id: Mapped[int] = mapped_column(ForeignKey("dim_data_source.source_id"), nullable=False)
    year: Mapped[int] = mapped_column(nullable=False)
    value_millions: Mapped[Decimal | None] = mapped_column(Numeric(14, 2))  # USD millions
    tons_thousands: Mapped[Decimal | None] = mapped_column(Numeric(14, 2))  # Thousands of tons
    ton_miles_millions: Mapped[Decimal | None] = mapped_column(Numeric(14, 2))  # Million ton-miles
    mode_code: Mapped[str | None] = mapped_column(String(10))  # Transport mode

    __table_args__ = (
        Index("idx_faf_flow_origin", "origin_cfs_area_id"),
        Index("idx_faf_flow_dest", "dest_cfs_area_id"),
        Index("idx_faf_flow_sctg", "sctg_id"),
        Index("idx_faf_flow_year", "year"),
    )


class FactHickelERDIAnnual(NormalizedBase):
    """Hickel/Sullivan/Zoomkawala (2021) ERDI annual time series 1960–2017.

    Spec 057: ingested from ``/media/user/data/babylon-data/babylon_hickel_final.csv``
    via ``tools/ingest/hickel_erdi.py``. Provides the periphery-wage signal
    (``erdi`` column) consumed by ``DefaultPeripheryLaborCoefficientsSource``,
    AND the calibration target (``annual_drain_usd_billions``) consumed by
    ``test_imperial_rent_calibration.py`` for SC-004.

    Distinct from the retired ``fact_hickel_drain`` resource-flow decomposition
    table (amputated 2026-07-17 per ADR075 ruling 1, A14). This table stores
    the annual ERDI / drain time series at the national-aggregate level.

    Constitutional III.4 status: ``Hickel_HSZ_Drain`` data source listed in
    ``data-catalog.yaml`` v2.6.2 under ``International Trade``, class ``Fixture``
    (per Spec 057 / R9 PATCH-level amendment).
    """

    __tablename__ = "fact_hickel_erdi_annual"

    id: Mapped[int] = mapped_column(primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), nullable=False)
    scale_type: Mapped[str] = mapped_column(String(40), nullable=False)
    """Hickel methodology selector. Observed values in source CSV:
    'Extensive' (raw dollar drain; pre-2009 era), 'Intensive' (ERDI-corrected;
    post-2009), 'Intensive_China_Inflection' (post-2017 China-shift variant).
    """

    erdi: Mapped[float] = mapped_column(Float, nullable=False)
    """Exchange Rate Distortion Index — market_exchange_rate / PPP_exchange_rate.
    Used as the periphery-wage gap proxy by Spec 057 R1 (broadcast uniformly
    across BEA Summary industries for v1)."""

    annual_drain_usd_billions: Mapped[float] = mapped_column(Float, nullable=False)
    """Aggregate Global South → North drain, USD billions. Used as Spec 057
    SC-004 calibration anchor (year-resolved comparison vs computed national
    total phi_hour · employment-hours)."""

    alpha: Mapped[float | None] = mapped_column(Float)
    core_gain_per_capita_usd: Mapped[float | None] = mapped_column(Float)
    is_anchor_year: Mapped[bool] = mapped_column(nullable=False, default=False)
    china_inflection: Mapped[bool] = mapped_column(nullable=False, default=False)
    cumulative_drain: Mapped[float | None] = mapped_column(Float)
    source: Mapped[str] = mapped_column(String(255), nullable=False)
    """Provenance tag — e.g., 'Hickel_Sullivan_Zoomkawala_2021'."""

    __table_args__ = (
        Index("idx_hickel_erdi_time", "time_id"),
        Index("idx_hickel_erdi_scale_type", "scale_type"),
        # Composite uniqueness: at most one row per (time_id, scale_type)
        Index("idx_hickel_erdi_time_scale", "time_id", "scale_type", unique=True),
    )


class FactBEAFinalDemandAnnual(NormalizedBase):
    """BEA Use Table — per-industry "Total Final Uses (GDP)" annual time series.

    Spec 057: ingested from
    ``/media/user/data/babylon-data/input-output/make-use/IOUse_Before_Redefinitions_PRO_Summary.xlsx``
    via ``tools/ingest/bea_final_demand.py``. Provides the final-demand
    vector ``y`` consumed by ``DefaultFinalDemandSource`` and (downstream)
    by :func:`babylon.domain.economics.tensor_hierarchy.production_chain_rent.ProductionChainRentCalculator.calculate`.

    Constitutional III.4: the BEA Use Table source is covered by the existing
    BEA_GDP / BEA_TiVA catalog entries (under ``Federal Economic``); no
    additional III.4 amendment required.
    """

    __tablename__ = "fact_bea_final_demand_annual"

    id: Mapped[int] = mapped_column(primary_key=True)
    time_id: Mapped[int] = mapped_column(ForeignKey("dim_time.time_id"), nullable=False)
    bea_industry_id: Mapped[int] = mapped_column(
        ForeignKey("dim_bea_industry.bea_industry_id"), nullable=False
    )
    total_final_uses_millions: Mapped[float] = mapped_column(Float, nullable=False)
    """The "Total Final Uses (GDP)" column from the BEA Use Table for this
    (year, industry). Sum of: PCE + Private Fixed Investment + Change in
    Private Inventories + Government Consumption Expenditures + Net Exports.
    Units: millions of dollars."""

    __table_args__ = (
        Index("idx_bea_fd_time", "time_id"),
        Index("idx_bea_fd_industry", "bea_industry_id"),
        Index(
            "idx_bea_fd_time_industry",
            "time_id",
            "bea_industry_id",
            unique=True,
        ),
    )


# =============================================================================
# EXPORTS
# =============================================================================

__all__ = [
    # Dimensions - Geographic
    "DimState",
    "DimCounty",
    "DimMetroArea",
    "BridgeCountyMetro",
    "DimGeographicHierarchy",
    "DimCFSArea",
    "BridgeCFSCounty",
    "DimSCTGCommodity",
    "DimCountry",
    "DimImportSource",
    # Dimensions - Industry
    "DimIndustry",
    "DimOwnership",
    # Dimensions - BEA Industry
    "DimBEAIndustry",
    "BridgeNAICSBEA",
    # Dimensions - Census Codes
    "DimIncomeBracket",
    "DimEmploymentStatus",
    "DimWorkerClass",
    "DimOccupation",
    "DimEducationLevel",
    "DimHousingTenure",
    "DimRentBurden",
    "DimPovertyCategory",
    # Dimensions - Energy
    # Dimensions - FRED
    "DimWealthClass",
    "DimAssetCategory",
    "DimFredSeries",
    # Dimensions - Commodities
    "DimCommodity",
    "DimCommodityMetric",
    # Dimensions - Metadata
    "DimTime",
    "DimGender",
    "DimDataSource",
    "DimRace",
    # Ingest Tracking
    "IngestCheckpoint",
    # Dimensions - Coercive Infrastructure
    "DimCoerciveType",
    # Facts - Census
    "FactCensusIncome",
    "FactCensusMedianIncome",
    "FactCensusEmployment",
    "FactCensusWorkerClass",
    "FactCensusHours",
    "FactCensusHousing",
    "FactCensusRent",
    "FactCensusRentBurden",
    "FactCensusEducation",
    "FactCensusPoverty",
    "FactCensusIncomeSources",
    # Facts - QCEW/Productivity
    "FactQcewAnnual",
    "FactProductivityAnnual",
    # Facts - BEA
    "FactBEANationalIndustry",
    "FactBEACountyGDP",
    # Facts - Trade
    "FactTradeMonthly",
    # Facts - Energy
    # Facts - FRED
    "FactFredNational",
    "FactFredWealthLevels",
    "FactFredWealthShares",
    # Facts - Commodities
    "FactCommodityObservation",
    # Facts - Materials
    "FactStateMinerals",
    "FactMineralProduction",
    "FactMineralEmployment",
    # Facts - Circulatory System
    "FactCoerciveInfrastructure",
    "FactBroadbandCoverage",
    "FactCommodityFlow",
    # LODES Crosswalk
    # Employment Industry
    "DimEmploymentArea",
    # DOT HPMS
    "FactHpmsRoadSegment",
    # ATUS Reproductive Labor
    # Capital Volume I (Feature 021)
    "FactBLSUnemploymentDecomposition",
    "FactEvictionLabFiling",
    "FactForeclosureRate",
    "FactCensusInstitutionalOwnership",
    # Tensor Hierarchy (Feature 025)
    "DimBEAIOTableType",
    "FactBEAIOCoefficient",
    "FactFAFCommodityFlow",
    "FactBEAFinalDemandAnnual",
    "FactHickelERDIAnnual",
]
