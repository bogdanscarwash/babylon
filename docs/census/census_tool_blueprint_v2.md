# Census ACS Query Tool Blueprint - Version 2

## Overview
This document outlines the design for a Claude tool that enables querying of Census American Community Survey (ACS) data through the Census API, with specific handling of ACS subject definitions and data quality measures.

## Tool Definition

```xml
<census_acs_query>
<year>Year of ACS data (e.g. 2023)</year>
<variables>Comma-separated list of ACS variables</variables>
<geography>Geographic area specification</geography>
<query_type>raw|tabulate</query_type>
<filters>Optional predicates to filter data</filters>
<weight>Optional statistical weight to apply</weight>
<quality_measures>Optional request for quality measures</quality_measures>
<derived_measures>Optional derived measures to calculate</derived_measures>
</census_acs_query>
```

## Core Components

### 1. Variable Handling

#### Subject Categories
- Housing Variables (e.g. BEDROOMS, ROOMS, TENURE)
- Population Variables (e.g. AGE, SEX, RACE) 
- Economic Variables (e.g. INCOME, EMPLOYMENT)
- Social Variables (e.g. EDUCATION, LANGUAGE)

#### Data Quality Measures
- Coverage Rates
- Response Rates
- Allocation Rates
- Sample Size Information

#### Derived Measures
- Medians
- Means
- Aggregates
- Percentages
- Ratios

### 2. Geography Handling

#### Geographic Hierarchy
- Nation
- State
- County
- Place
- Census Tract
- Block Group

#### Geographic Components
- Urban/Rural
- Metropolitan/Micropolitan
- Inside/Outside Principal Cities

### 3. API Integration

#### Query Construction
```python
def construct_query(params):
    # Validate parameters
    validate_parameters(params)
    
    # Build base URL
    base_url = f"https://api.census.gov/data/{params['year']}/acs/acs1"
    
    # Add query parameters
    query_params = {
        'get': params['variables'],
        'for': params['geography'],
        'key': API_KEY
    }
    
    # Add optional parameters
    if 'filters' in params:
        query_params['predicates'] = params['filters']
        
    return base_url, query_params
```

#### Response Processing
```python
def process_response(response, params):
    """Process Census API response and apply quality checks"""
    
    # Parse JSON response
    data = response.json()
    
    # Apply quality measures if requested
    if 'quality_measures' in params:
        quality_data = get_quality_measures(data, params)
        
    # Calculate derived measures if requested
    if 'derived_measures' in params:
        derived_data = calculate_derived_measures(data, params)
        
    return {
        'data': format_data(data),
        'quality': quality_data if 'quality_measures' in params else None,
        'derived': derived_data if 'derived_measures' in params else None
    }
```

### 4. Data Quality Handling

#### Coverage Rate Calculation
```python
def calculate_coverage_rate(data, independent_estimate):
    """Calculate coverage rate against independent population estimate"""
    acs_estimate = sum(data['population'])
    return (acs_estimate / independent_estimate) * 100
```

#### Response Rate Calculation
```python
def calculate_response_rate(data):
    """Calculate unit response rate"""
    return (data['responses'] / data['sample_size']) * 100
```

#### Allocation Rate Tracking
```python
def track_allocation_rates(data):
    """Track item allocation rates for quality assessment"""
    allocated_items = count_allocated_items(data)
    total_items = count_total_items(data)
    return (allocated_items / total_items) * 100
```

## Error Handling

### Census API Specific Errors
```python
class CensusAPIError(Exception):
    """Base class for Census API errors"""
    pass

class GeographyError(CensusAPIError):
    """Error for invalid geographic specifications"""
    pass
    
class VariableError(CensusAPIError):
    """Error for invalid variable specifications"""
    pass

class QualityError(CensusAPIError):
    """Error for data quality issues"""
    pass
```

### Error Response Format
```json
{
    "error": {
        "code": "ERROR_CODE",
        "message": "Detailed error message",
        "details": {
            "failed_component": "Component that caused error",
            "suggestion": "Suggested fix if applicable"
        }
    }
}
```

## Usage Examples

### Basic Population Query
```python
query = {
    "year": "2023",
    "variables": "B01001_001E", # Total population
    "geography": "state:*",
    "query_type": "raw"
}
```

### Housing Characteristics with Quality Measures
```python
query = {
    "year": "2023",
    "variables": "B25002_001E,B25002_002E", # Housing units, Occupied units
    "geography": "county:*",
    "query_type": "raw",
    "quality_measures": ["coverage_rate", "response_rate"]
}
```

### Income Data with Derived Measures
```python
query = {
    "year": "2023",
    "variables": "B19013_001E", # Median household income
    "geography": "tract:*",
    "query_type": "tabulate",
    "derived_measures": ["median", "aggregate"]
}
```

## Subject Definitions

### Housing Variables
```python
HOUSING_VARIABLES = {
    'UNITS': {
        'description': 'Housing Units',
        'table': 'B25001',
        'categories': ['Total', 'Occupied', 'Vacant']
    },
    'TENURE': {
        'description': 'Housing Tenure',
        'table': 'B25003',
        'categories': ['Owner occupied', 'Renter occupied']
    }
}
```

### Population Variables
```python
POPULATION_VARIABLES = {
    'AGE': {
        'description': 'Age',
        'table': 'B01001',
        'categories': ['Under 5 years', '5-17 years', '18-24 years', '25-44 years', 
                      '45-64 years', '65 years and over']
    },
    'RACE': {
        'description': 'Race',
        'table': 'B02001',
        'categories': ['White alone', 'Black alone', 'American Indian alone', 
                      'Asian alone', 'Pacific Islander alone', 'Other race alone',
                      'Two or more races']
    }
}
```

## Data Quality Thresholds

```python
QUALITY_THRESHOLDS = {
    'coverage_rate': {
        'min': 90.0,
        'max': 110.0,
        'warning_threshold': 95.0
    },
    'response_rate': {
        'min': 85.0,
        'warning_threshold': 90.0
    },
    'allocation_rate': {
        'max': 20.0,
        'warning_threshold': 10.0
    }
}
```

## Best Practices

1. Variable Selection
   - Use detailed tables for granular data
   - Consider allocation rates when selecting variables
   - Check for data quality flags

2. Geography Selection
   - Start with larger geographies for testing
   - Consider margin of error for small areas
   - Check coverage rates for selected areas

3. Data Quality
   - Always check response rates
   - Monitor allocation rates
   - Verify against published tables when possible

4. Performance
   - Batch related queries
   - Cache frequently used data
   - Use appropriate geographic summary levels

## Limitations

1. Data Availability
   - Not all variables available for all geographies
   - Some data suppressed for privacy
   - Vintage differences between geographies

2. Quality Issues
   - Higher margins of error for small geographies
   - Some areas may have low response rates
   - Allocation rates may affect certain variables

3. API Constraints
   - Rate limits
   - Query complexity limits
   - Response size limits

## Future Enhancements

1. Additional Features
   - Support for ACS microdata (PUMS)
   - Time series analysis
   - Cross-survey comparisons

2. Quality Improvements
   - Enhanced validation rules
   - Automated quality checking
   - Improved error messages

3. Performance Optimizations
   - Query optimization
   - Response caching
   - Parallel processing

## References

1. Census API Documentation
2. ACS Technical Documentation
3. Statistical Testing Documentation
4. Geography Program Documentation
