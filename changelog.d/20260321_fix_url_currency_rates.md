### Fixed
- Fixed currency conversion failing on page load when expression is loaded from URL. Expressions containing currencies (e.g., RUB, TON, USD) would show "No exchange rate available" because rates hadn't been fetched yet. The worker now detects which rate sources each expression needs and fetches only those on demand before calculating.
