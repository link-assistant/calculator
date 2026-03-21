### Fixed
- Fixed currency conversion failing on page load when expression is loaded from URL. Expressions containing crypto currencies (e.g., TON) or other currencies would show "No exchange rate available" because rates hadn't been fetched yet. The worker now waits for required exchange rates to load before completing the calculation, showing a busy indicator while waiting.

### Added
- Exchange rate caching in localStorage with TTL-based invalidation: central bank rates (ECB, CBR) cached for 12 hours, real-time crypto rates (CoinGecko) cached for 5 minutes. Cached rates are applied immediately on page load for faster initial calculations.
