---
bump: minor
---

### Added
- Support for `=` as an equality check operator in expressions (e.g., `1 * (2 / 3) = (1 * 2) / 3` returns `true`)
- Previously, using `=` in an expression would throw `Parse error: Unexpected character '=' at position N`
- Both sides of the equality are evaluated and compared, returning `true` or `false`
