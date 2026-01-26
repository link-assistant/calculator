---
bump: patch
---

### Fixed
- Fixed .lino rate file parser to support the new format (`conversion:` as root, `rates:` for data section)
- The parser now correctly handles both the new format and the legacy format (`rates:` as root, `data:` for data section)

### Changed
- Updated `load_rates_from_consolidated_lino()` to support both new and legacy .lino formats
- Updated `parse_consolidated_lino_rates()` WASM binding to support both formats
- Added read-only `currency_db()` accessor to ExpressionParser for testing

### Added
- Added tests for new .lino format parsing
- Added test to verify historical rates can be loaded and retrieved from the database
