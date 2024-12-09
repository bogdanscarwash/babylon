import pandas as pd
from pathlib import Path
from typing import List, Dict, Optional
import re

class CensusDataDictionary:
    def __init__(self, excel_path: str = None):
        if excel_path is None:
            # Default to the config location
            excel_path = Path(__file__).parent.parent / "config" / "2022_DataProductList.xlsx"
        
        self.df = pd.read_excel(excel_path)
        # Clean up column names for easier access
        self.df.columns = [col.split('\n')[0].strip() for col in self.df.columns]
    
    def search_tables(self, keyword: str) -> pd.DataFrame:
        """Search for tables containing the keyword in their title or universe."""
        pattern = re.compile(keyword, re.IGNORECASE)
        mask = (
            self.df['Table Title'].str.contains(pattern, na=False) |
            self.df['Table Universe'].str.contains(pattern, na=False)
        )
        return self.df[mask]
    
    def get_table_info(self, table_id: str) -> Dict:
        """Get detailed information about a specific table."""
        table = self.df[self.df['Table ID'] == table_id]
        if len(table) == 0:
            return None
        
        return table.iloc[0].to_dict()
    
    def get_available_geographies(self, table_id: str, estimate_type: str = '5-Year') -> str:
        """Get geography restrictions for a table."""
        table = self.df[self.df['Table ID'] == table_id]
        if len(table) == 0:
            return None
        
        col = f'{estimate_type} Geography Restrictions'
        return table.iloc[0][col]
    
    def list_data_product_types(self) -> List[str]:
        """Get all available data product types."""
        return sorted(self.df['Data Product Type'].unique())
    
    def get_tables_by_product_type(self, product_type: str) -> pd.DataFrame:
        """Get all tables of a specific product type."""
        return self.df[self.df['Data Product Type'] == product_type]

def example_usage():
    # Create dictionary instance
    dictionary = CensusDataDictionary()
    
    # Example: Search for housing-related tables
    housing_tables = dictionary.search_tables('housing')
    print("\nHousing-related tables:")
    print(housing_tables[['Table ID', 'Table Title']].head())
    
    # Example: Get detailed info about the table from test.py (B25034_010E)
    table_base = 'B25034'  # Base table ID
    table_info = dictionary.get_table_info(table_base)
    if table_info:
        print(f"\nInformation about table {table_base}:")
        print(f"Title: {table_info['Table Title']}")
        print(f"Universe: {table_info['Table Universe']}")
        print(f"Data Product Type: {table_info['Data Product Type']}")
        print(f"5-Year Geography Restrictions: {table_info['5-Year Geography Restrictions']}")

if __name__ == '__main__':
    example_usage()
