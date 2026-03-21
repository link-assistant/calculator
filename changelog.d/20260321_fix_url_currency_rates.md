### Fixed
- Fixed currency conversion failing on page load when expression is loaded from URL. Expressions containing crypto currencies (e.g., TON) or other currencies would show "No exchange rate available" because rates hadn't been fetched yet. The calculator now automatically retries the calculation when exchange rates finish loading.
