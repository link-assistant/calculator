### Changed
- Consolidated currency exchange rate data from 22,147 individual `.lino` files into 41 single files per currency pair
- Changed data format from individual date-based files (`data/currency/{from}/{to}/{date}.lino`) to consolidated files (`data/currency/{from}-{to}.lino`)
- Updated `download_historical_rates.py` script to generate consolidated format
- Added `consolidate_rates.py` script to migrate existing data to new format

### Added
- New `load_rates_from_consolidated_lino()` method in Calculator for loading the consolidated format
- New `parse_consolidated_lino_rates()` WASM binding for parsing consolidated format in web app
