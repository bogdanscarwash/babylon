from census import Census
from us import states
import os
from dotenv import load_dotenv
from src.babylon.census.data_dictionary import CensusDataDictionary

# Load environment variables and initialize Census API client
load_dotenv()
c = Census(os.environ['CENSUS_API_KEY'])

# Initialize our data dictionary
census_dict = CensusDataDictionary()

def get_table_metadata(table_ids):
    """Get metadata about the Census tables we're querying."""
    metadata = {}
    for full_id in table_ids:
        # Strip the _XXXE suffix to get base table ID
        base_id = full_id.split('_')[0]
        info = census_dict.get_table_info(base_id)
        if info:
            metadata[full_id] = {
                'title': info['Table Title'],
                'universe': info['Table Universe'],
                'product_type': info['Data Product Type']
            }
    return metadata

def query_census_data(variables, geo_filter, estimate_type='acs5'):
    """
    Query Census data with additional context.
    
    Args:
        variables: List of Census variable IDs
        geo_filter: Dictionary of geographic filters
        estimate_type: Type of estimate (acs5, acs1, etc.)
    """
    # Add NAME to variables to get geographic names
    if 'NAME' not in variables:
        variables = ['NAME'] + variables
    
    # Get metadata about the variables we're querying
    metadata = get_table_metadata(variables)
    
    # Make the Census API call
    census_client = getattr(c, estimate_type)
    results = census_client.get(variables, geo_filter)
    
    print("\nQuery Information:")
    print("Variables being queried:")
    for var in variables:
        if var in metadata:
            print(f"\n{var}:")
            print(f"  Table: {metadata[var]['title']}")
            print(f"  Universe: {metadata[var]['universe']}")
            print(f"  Product Type: {metadata[var]['product_type']}")
    
    print("\nResults:")
    return results

def example_queries():
    """Example Census queries with context."""
    # Original query: Housing units built 1940 to 1949
    print("\nExample 1: Housing Age Data")
    results = query_census_data(
        ['B25034_010E'],
        {'for': f'state:{states.MD.fips}'}
    )
    print(results)
    
    # Additional example: Occupancy status
    print("\nExample 2: Housing Occupancy Data")
    results = query_census_data(
        ['B25002_001E', 'B25002_002E', 'B25002_003E'],
        {'for': f'state:{states.MD.fips}'}
    )
    print(results)

if __name__ == '__main__':
    # Show some available housing-related tables
    print("Available Housing-Related Tables:")
    housing_tables = census_dict.search_tables('housing')
    print(housing_tables[['Table ID', 'Table Title']].head())
    print("\n" + "="*80 + "\n")
    
    # Run example queries
    example_queries()
