# Reference-DB drop report — batch demotion (ADR075 ruling 1 / ADR076)

Executed: 2026-07-17T01:10:45
Integrity (`PRAGMA quick_check`): ok

| object | kind | rows before drop |
| --- | --- | --- |
| view_energy_consumption | view | view |
| bridge_county_bea_ea | table | rows_before=83 |
| dim_bea_economic_area | table | rows_before=8 |
| fact_ricci_unequal_exchange | table | rows_before=29 |
| fact_energy_annual | table | rows_before=525 |
| dim_energy_series | table | rows_before=20 |
| dim_energy_table | table | rows_before=14 |
| bridge_lodes_block | table | rows_before=1150562 |
| staging_arcgis_feature | table | rows_before=5974 |
