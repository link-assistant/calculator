---
bump: patch
---

### Fixed
- Resolve thousands grouping and decimal commas in the lexer before a successfully parsed expression can return a silently wrong result. Support mixed decimal/grouping conventions, Swiss apostrophes, and Indian grouping while retaining function argument commas and numeric-date parsing.
- Preserve measured values with absolute uncertainty (`12,3 ± 4,5`) and German million/billion abbreviations (`1,08 Mrd. km/h`).
- Compare compact competitor result displays such as `300k` numerically and retain the participant-count unit in the corpus expectation.
- Preserve space-grouped fractions and underscore/space grouping, parse radix literals numerically, and report scale overflow without panicking. Document the required fend `1,1` compatibility exception.
