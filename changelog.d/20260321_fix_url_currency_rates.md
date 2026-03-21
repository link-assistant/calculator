### Fixed
- Fixed currency conversion failing on page load when expression is loaded from URL. Expressions containing currencies (e.g., RUB, TON, USD) would show "No exchange rate available" because rates hadn't been fetched yet. The worker now awaits all exchange rate sources before executing any calculation, ensuring rates are always available on the first attempt.
