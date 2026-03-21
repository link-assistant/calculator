### Fixed
- Fixed currency conversion failing on page load when expression is loaded from URL. Expressions containing currencies (e.g., RUB, TON, USD) would show "No exchange rate available" because rates hadn't been fetched yet. The worker now waits for exchange rates to finish loading before completing currency calculations, showing a busy indicator while waiting.
