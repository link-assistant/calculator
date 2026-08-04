---
bump: patch
---

### Fixed
- Recognize the Russian ruble abbreviation `руб` (and `рубл`) as RUB, so expressions like `20000 руб + 120000 руб + 25000 рублей` calculate instead of failing with "cannot add 'руб' and 'RUB'" ([#209](https://github.com/link-assistant/calculator/issues/209)).
- Accept a trailing dot on abbreviated units (`руб.`, `кг.`), which previously raised `Unexpected character '.'`.
