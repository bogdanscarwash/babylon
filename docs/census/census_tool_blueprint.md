# Census ACS Query Tool Blueprint

## Overview
This document outlines the design for a Claude tool that enables querying of Census American Community Survey (ACS) data through the Census API.

## Tool Definition

```xml
<census_acs_query>
<year>Year of ACS data (e.g. 2019)</year>
<variables>Comma-separated list of ACS variables (e.g. SEX,DIS,PWGTP)</variables>
<geography>Geographic area specification (e.g. state:11,24,51)</geography>
<query_type>raw|tabulate - Type of query to execute</query_type>
<filters>Optional predicates to filter data (e.g. HICOV=2)</filters>
<weight>Optional statistical weight to apply (e.g. PWGTP)</weight>
</census_acs_query>
```

## Implementation Components

### 1. Query Construction
- Base URL formation: https://api.census.gov/data/[year]/acs/acs1/pums
- Parameter validation and formatting
- URL encoding of special characters
- Query type handling (raw vs tabulated data)

### 2. API Integration
- HTTP request handling
- Authentication if required
- Rate limiting compliance
- Response parsing (JSON)

### 3. Error Handling
- Invalid parameter validation
- API error responses
- Network issues
- Data validation

### 4. Response Formatting
- Raw data format:
```json
{
  "records": [
    {
      "variable1": "value1",
      "variable2": "value2"
    }
  ],
  "metadata": {
    "query_details": {},
    "total_records": 0
  }
}
```

- Tabulated data format:
```json
{
  "data": [
    {
      "category": "value",
      "count": 0,
      "weighted_count": 0
    }
  ],
  "metadata": {
    "query_details": {},
    "weights_applied": ""
  }
}
```

## Usage Examples

### Raw Data Query
```python
# Example raw data query
query = {
  "year": "2019",
  "variables": "SEX,DIS,PWGTP",
  "geography": "state:11,24,51",
  "query_type": "raw",
  "filters": "HICOV=2"
}
```

### Tabulated Query
```python
# Example tabulated query
query = {
  "year": "2019", 
  "variables": "SEX,DIS",
  "geography": "state:11,24,51",
  "query_type": "tabulate",
  "weight": "PWGTP",
  "filters": "HICOV=2"
}
```

## Error Codes

| Code | Description |
|------|-------------|
| 400  | Invalid parameters |
| 401  | Authentication error |
| 404  | Data not found |
| 429  | Rate limit exceeded |
| 500  | Census API error |

## Implementation Steps

1. Parameter Validation
```python
def validate_parameters(params):
    # Validate required fields
    required = ['year', 'variables', 'geography', 'query_type']
    for field in required:
        if field not in params:
            raise ValueError(f"Missing required field: {field}")
            
    # Validate year format
    if not params['year'].isdigit():
        raise ValueError("Year must be numeric")
        
    # Additional validation...
```

2. Query Construction
```python
def construct_query(params):
    base_url = f"https://api.census.gov/data/{params['year']}/acs/acs1/pums"
    
    if params['query_type'] == 'raw':
        query = f"?get={params['variables']}"
    else:
        query = f"?tabulate=weight({params['weight']})"
        
    # Add geography
    query += f"&for={params['geography']}"
    
    # Add filters if present
    if 'filters' in params:
        query += f"&{params['filters']}"
        
    return base_url + query
```

3. Response Processing
```python
def process_response(response, query_type):
    data = response.json()
    
    if query_type == 'raw':
        return {
            "records": format_raw_records(data),
            "metadata": extract_metadata(data)
        }
    else:
        return {
            "data": format_tabulated_data(data),
            "metadata": extract_metadata(data)
        }
```

## Best Practices

1. Cache Commonly Used Data
- Implement caching for frequently accessed data
- Cache variable definitions and geographic codes

2. Rate Limiting
- Implement exponential backoff for retries
- Track API usage to stay within limits

3. Error Handling
- Provide clear error messages
- Include suggestions for fixing common issues
- Log errors for debugging

4. Documentation
- Document all supported variables
- Provide usage examples
- Include common query patterns

## Limitations

1. Data Availability
- Limited to ACS PUMS data
- Geographic coverage restrictions
- Variable availability by year

2. Performance
- Large queries may timeout
- Rate limits may apply
- Response size limitations

3. Functionality
- No direct median calculations
- Limited statistical functions
- No time series in single query

## Future Enhancements

1. Additional Features
- Support for other Census datasets
- Advanced statistical calculations
- Time series analysis

2. Performance Improvements
- Query optimization
- Response caching
- Parallel processing

3. User Experience
- Variable discovery
- Query builder interface
- Result visualization

## References

1. Census API Documentation
2. ACS Technical Documentation
3. PUMS Data Dictionary
4. Geographic Reference Files
