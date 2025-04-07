"""
Census API configuration file for querying American Community Survey (ACS) data.

This module provides organized variable definitions and configurations for accessing
Census Bureau data through the census package. Variables are grouped into major
categories (housing, population, economic, social) for easier access.

Usage:
    from census_config import CENSUS_VARS, DEFAULT_YEAR, DEFAULT_SURVEY

    # Get all housing variables
    housing_vars = CENSUS_VARS['housing']

    # Get specific variable code
    total_pop = CENSUS_VARS['population']['total_population']

Note:
    Variable codes (e.g. B01003_001E) correspond to ACS detailed tables.
    Suffix 'E' indicates estimate, 'M' margin of error, 'PE' percentage estimate.
"""

# Housing Variables
HOUSING_VARS = {
    "total_housing_units": {
        "code": "B25001_001E",
        "description": "Total housing units",
    },
    "occupancy": {
        "total": {
            "code": "B25002_001E",
            "description": "Total housing units for occupancy status",
        },
        "occupied": {"code": "B25002_002E", "description": "Occupied housing units"},
        "vacant": {"code": "B25002_003E", "description": "Vacant housing units"},
    },
    "tenure": {
        "total": {
            "code": "B25003_001E",
            "description": "Total housing units for tenure",
        },
        "owner_occupied": {
            "code": "B25003_002E",
            "description": "Owner-occupied housing units",
        },
        "renter_occupied": {
            "code": "B25003_003E",
            "description": "Renter-occupied housing units",
        },
    },
    "value": {
        "median_home_value": {
            "code": "B25077_001E",
            "description": "Median value (dollars) for owner-occupied housing units",
        },
        "median_rent": {
            "code": "B25064_001E",
            "description": "Median gross rent (dollars)",
        },
    },
    "rooms": {
        "median_rooms": {
            "code": "B25018_001E",
            "description": "Median number of rooms per housing unit",
        },
        "total_rooms": {
            "code": "B25017_001E",
            "description": "Total rooms in housing units",
        },
    },
}

# Population Variables
POPULATION_VARS = {
    "total_population": {"code": "B01003_001E", "description": "Total population"},
    "age": {
        "median_age": {"code": "B01002_001E", "description": "Median age (years)"},
        "under_18": {"code": "B09001_001E", "description": "Population under 18 years"},
        "over_65": {
            "code": "B01001_020E",
            "description": "Population 65 years and over",
        },
    },
    "race": {
        "total": {"code": "B02001_001E", "description": "Total population for race"},
        "white_alone": {"code": "B02001_002E", "description": "White alone"},
        "black_alone": {
            "code": "B02001_003E",
            "description": "Black or African American alone",
        },
        "asian_alone": {"code": "B02001_005E", "description": "Asian alone"},
        "hispanic_any_race": {
            "code": "B03003_003E",
            "description": "Hispanic or Latino (of any race)",
        },
    },
    "nativity": {
        "native": {"code": "B05012_002E", "description": "Native born population"},
        "foreign_born": {
            "code": "B05012_003E",
            "description": "Foreign born population",
        },
    },
}

# Economic Variables
ECONOMIC_VARS = {
    "income": {
        "median_household": {
            "code": "B19013_001E",
            "description": "Median household income (dollars)",
        },
        "mean_household": {
            "code": "B19025_001E",
            "description": "Mean household income (dollars)",
        },
        "per_capita": {
            "code": "B19301_001E",
            "description": "Per capita income (dollars)",
        },
    },
    "poverty": {
        "total_poverty": {
            "code": "B17001_002E",
            "description": "Total population below poverty level",
        },
        "poverty_rate": {
            "code": "B17001_002PE",
            "description": "Poverty rate (percentage)",
        },
    },
    "employment": {
        "civilian_labor_force": {
            "code": "B23025_003E",
            "description": "Civilian labor force 16 years and over",
        },
        "employed": {
            "code": "B23025_004E",
            "description": "Employed civilian population 16 years and over",
        },
        "unemployed": {
            "code": "B23025_005E",
            "description": "Unemployed civilian population 16 years and over",
        },
        "unemployment_rate": {
            "code": "B23025_005PE",
            "description": "Unemployment rate (percentage)",
        },
    },
    "commuting": {
        "mean_travel_time": {
            "code": "B08135_001E",
            "description": "Mean travel time to work (minutes)",
        },
        "total_workers": {
            "code": "B08301_001E",
            "description": "Total workers 16 years and over",
        },
    },
}

# Social Variables
SOCIAL_VARS = {
    "education": {
        "total_25_plus": {
            "code": "B15003_001E",
            "description": "Population 25 years and over",
        },
        "bachelors_or_higher": {
            "code": "B15003_022E",
            "description": "Bachelor's degree or higher",
        },
        "graduate_degree": {
            "code": "B15003_023E",
            "description": "Graduate or professional degree",
        },
    },
    "language": {
        "english_only": {"code": "B16001_002E", "description": "Speak only English"},
        "spanish": {"code": "B16001_003E", "description": "Speak Spanish"},
        "asian_pacific": {
            "code": "B16001_004E",
            "description": "Speak Asian and Pacific Island languages",
        },
    },
    "health_insurance": {
        "total_civilian": {
            "code": "B27001_001E",
            "description": "Total civilian noninstitutionalized population",
        },
        "with_insurance": {
            "code": "B27001_002E",
            "description": "With health insurance coverage",
        },
        "no_insurance": {
            "code": "B27001_003E",
            "description": "No health insurance coverage",
        },
    },
    "households": {
        "total_households": {"code": "B11001_001E", "description": "Total households"},
        "family_households": {
            "code": "B11001_002E",
            "description": "Family households",
        },
        "nonfamily_households": {
            "code": "B11001_007E",
            "description": "Nonfamily households",
        },
    },
}

# Group all variable sets
CENSUS_VARS = {
    "housing": HOUSING_VARS,
    "population": POPULATION_VARS,
    "economic": ECONOMIC_VARS,
    "social": SOCIAL_VARS,
}

# Default year for queries (most recent 5-year estimates available)
DEFAULT_YEAR = 2021

# Default survey - using 5-year estimates for most reliable data
DEFAULT_SURVEY = "acs5"

# Geographic levels supported by the Census API
GEO_LEVELS = {
    "us": "nation",
    "state": "state",
    "county": "county",
    "place": "place",
    "tract": "tract",
    "block_group": "block group",
    "zip": "zip code tabulation area",
}

# Geographic hierarchy for drilling down
GEO_HIERARCHY = {
    "state": ["us"],
    "county": ["us", "state"],
    "tract": ["us", "state", "county"],
    "block_group": ["us", "state", "county", "tract"],
    "place": ["us", "state"],
}
