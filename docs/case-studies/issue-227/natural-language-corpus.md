# Multilingual scientific-quote corpus

[corpus/natural-language-quotes.tsv](corpus/natural-language-quotes.tsv) contains
1,356 expressions in 23 languages, assembled on 2026-10-08 for #232. Every
language has at least 50 distinct expressions and all 12 required constructs.
This extends the 162-row seed from #227 without changing its inputs, sources,
languages or expected results.

## Sources and sampling

| Source group | Rows | Provenance |
| --- | ---: | --- |
| Wikipedia quote | 26 | Original numeric fragments; original attribution retained |
| Wikipedia quote (derived) | 10 | Original conversion requests, with explicit citations added |
| Theorem in words | 40 | Original examples derived from Pythagoras, Kepler, Basel, mass–energy equivalence and Ohm's law |
| Paper phrasing | 86 | Original constructed paper-style expressions; these are not quotations from papers |
| Scientific statement (derived) | 1,150 | 50 localized expressions per language, with known mathematical results and cited definitions |
| Wikipedia sentence | 41 | Numeric passages from actual Wikipedia summaries/article bodies in all 23 languages |
| Open-access paper quote | 3 | Exact excerpts from three explicitly CC BY 4.0 arXiv abstracts |

The [source snapshots](../../../experiments/issue-227/sources/) retain the
Wikipedia text, candidate passages, title, contributors, revision URL, retrieval
date and license. The importer starts with the Wikidata Q2111 language links and
the Wikipedia REST summary API. When a summary has no numerical quantity (nl,
sv, he), it fetches the article body at the summary's revision and selects
numeric passages with unit symbols or names. Unicode `\p{Nd}` recognizes both
Latin and native digits. The committed candidate passages were reviewed for
scientific quantities; historical measurements remain historical statements,
not estimates of the present speed of light. REST text can flatten superscripts
and formulas or retain unusual spacing; quotation rows preserve that text.

The paper snapshots contain the original abstracts, authors, versioned URL and
the explicit CC BY 4.0 link from each arXiv page. The selected excerpts are:

- LIGO Scientific Collaboration and Virgo Collaboration,
  [Observation of Gravitational Waves from a Binary Black Hole Merger](https://arxiv.org/abs/1602.03837v1),
  abstract: the statement about 90% credible intervals.
- LIGO Scientific Collaboration and Virgo Collaboration,
  [GW170104: Observation of a 50-Solar-Mass Binary Black Hole Coalescence at Redshift 0.2](https://arxiv.org/abs/1706.01812v2),
  abstract: luminosity distance and redshift with asymmetric uncertainties.
- LIGO Scientific Collaboration and Virgo Collaboration,
  [GW170817: Observation of Gravitational Waves from a Binary Neutron Star Inspiral](https://arxiv.org/abs/1710.05832v1),
  abstract: the component-mass range.

The 50 derived expressions per language use six separator cases and four cases
for every other construct. Each template expresses a measurement, calculation,
constant lookup or conversion request. Numbers are substituted systematically;
these are synthetic examples based on source definitions, not measurements or
sentences claimed to occur in those sources. This gives repeated coverage of
each construct without hand-writing 1,150 unrelated rows.

## Language and construct inventory

`lang` is the language of the input text, not the source publication or the
expected display. Use a lowercase ISO 639-1/BCP-47 primary language tag. The
required set is the 13 seed languages plus nl, pl, uk, sv, fa, id, vi, th, he, bn.
The `zh` passages may contain traditional or simplified characters; `pt` does
not imply a particular region. This corpus does not encode a parser locale.

| Language | Rows | Required constructs present |
| --- | ---: | ---: |
| en | 105 | 12/12 |
| ru | 70 | 12/12 |
| de | 65 | 12/12 |
| fr | 64 | 12/12 |
| es | 64 | 12/12 |
| zh | 65 | 12/12 |
| ja | 60 | 12/12 |
| pt | 61 | 12/12 |
| hi | 59 | 12/12 |
| ar | 59 | 12/12 |
| it | 56 | 12/12 |
| ko | 54 | 12/12 |
| tr | 56 | 12/12 |
| nl | 53 | 12/12 |
| pl | 51 | 12/12 |
| uk | 51 | 12/12 |
| sv | 53 | 12/12 |
| fa | 51 | 12/12 |
| id | 52 | 12/12 |
| vi | 52 | 12/12 |
| th | 51 | 12/12 |
| he | 53 | 12/12 |
| bn | 51 | 12/12 |
| **Total** | **1,356** | **276 language/construct pairs** |

| `construct` | Meaning and example cases |
| --- | --- |
| `separators` | Grouped integers and decimals; decimal dot/comma, narrow no-break space, Arabic separators, Indian grouping and native digits |
| `si-prefix` | Measurements in µm, nm, km and mm, including U+00B5 micro sign |
| `scientific-notation` | ×10³, ×10⁸, ×10^23 and e-3 in a frequency statement |
| `uncertainty` | Symmetric ± statements; paper samples also preserve asymmetric TeX uncertainty |
| `percentage-change` | Increase a starting value by a stated percent |
| `fold-change` | Multiply the starting value by a stated factor, including a factor below one |
| `order-of-magnitude` | Multiply the starting value by 10 raised to the stated number of orders |
| `ratio` | Ratios stated in words, including 22 to 7 and 3 to 4 |
| `named-constant` | Localized names for pi, Euler's number e, vacuum light speed and Avogadro's constant |
| `power` | Powers in words, with square, cube and higher exponents |
| `root` | Roots in words, with degrees two through five |
| `unit-conversion` | Micro/nano, kilo/base, milli/base and Celsius/kelvin conversions |

## TSV and note conventions

The five UTF-8, tab-separated columns are `source`, `lang`, `expression`,
`expected`, `note`. Lines starting with `#` are comments. Tabs, newlines and
backslashes inside a field use `\t`, `\n`, `\\`; the existing coverage runner
decodes them. SI symbols remain international even in localized prose.

The `note` field begins with semicolon-separated metadata:

```text
construct=power; kind=derived; license=CC-BY-SA-4.0; https://en.wikipedia.org/wiki/Exponentiation; license-url=https://creativecommons.org/licenses/by-sa/4.0/; adaptation=localized template with synthetic numbers; case=power-1; parameters={"a":"3","b":"2","expected":"9"};
```

- `construct` uses a key from the table above. `other` preserves seed constructs
  outside the required set (trigonometry, means, fractions, dimensional laws,
  etc.) and paper ranges.
- `kind=verbatim` means an exact substring of a committed source snapshot.
  `kind=derived` means a constructed request or numeric substitution. Reserve
  `kind=translated` for a translation that cites the original excerpt and
  records the translation; no newly sampled paper is presented as native prose
  in a language in which it was not published.
- `kind=legacy-quote` preserves the original seed's claimed quotation and
  citation. Its 2026-10-07 source snapshot was not available, so it is not
  silently reclassified as a newly verified quotation.
- Every row includes a source URL, `license` and `license-url`. New quotation
  notes include authors, title and retrieval date; Wikipedia links pin the
  revision and paper links pin the arXiv version. Derived rows record their
  case and unlocalized numeric parameters. Seed notes preserve the old text.
- `expected` is independent of calculator output: percentages use
  `a × (1 + b/100)`, fold changes `a × b`, magnitude changes `a × 10^b`, and
  powers/roots their ordinary positive-real definitions. Scalar results use
  decimal dots and canonical SI units. Pi and e use familiar numerical values;
  light speed and Avogadro's constant use exact SI defining values from NIST.
- An empty `expected` requires `oracle=evaluate-only`. It is used for
  uncertainty, full sentences and ranges, whose eventual display model is not
  yet specified. It checks that evaluation succeeds; it does not verify a
  correct central value, bounds or unit. The existing scalar comparator checks
  rounded numerical results and unit text, not a semantic parse tree.

## Attribution and licensing

Wikipedia contributors' material and adaptations retain
[CC BY-SA 4.0](https://creativecommons.org/licenses/by-sa/4.0/) attribution; it
is not relicensed under this repository's Unlicense. This is the Wikipedia
source allowed by #232, alongside CC BY and public-domain sources. The sampled
paper material retains [CC BY 4.0](https://creativecommons.org/licenses/by/4.0/)
and the named collaborations' attribution. The source-specific licenses apply
to snapshots and corpus material taken or adapted from them.

The NIST references cover [number/unit writing](https://physics.nist.gov/cuu/Units/checklist.html),
[SI prefixes](https://physics.nist.gov/cuu/Units/prefixes.html),
[units and conversions](https://physics.nist.gov/cuu/Units/units.html),
[uncertainty](https://physics.nist.gov/cuu/Uncertainty/index.html) and
[fundamental constants](https://physics.nist.gov/cuu/Constants/index.html).
Their federal-publication status is described in the
[NIST library FAQ](https://www.nist.gov/nist-research-library/library-faqs).
The other derived sources are Wikipedia's definitions of scientific notation,
percentage change, fold change, order of magnitude, ratios, pi, e,
exponentiation and nth roots; their URLs are recorded per row.

## Reproduce and extend

From the repository root, offline regeneration and validation require only
Node.js and committed inputs:

```sh
node experiments/issue-227/build-natural-language-corpus.mjs
node --test scripts/natural-language-corpus.test.mjs
```

To refresh external sources, run the following, review the snapshot diffs and
candidate passages for accurate quantities and provenance, then regenerate:

```sh
node experiments/issue-227/fetch-natural-language-sources.mjs
git diff -- experiments/issue-227/sources
node experiments/issue-227/build-natural-language-corpus.mjs
node --test scripts/*.test.mjs
```

The source fetcher fails if a selected paper lacks its explicit CC BY link or
a required language has no numerical passage. Source fetching stays outside
CI so tests do not depend on a live API. For additional constructed cases,
edit the localized templates and case table rather than the generated TSV.
Keep the seed file intact. Review language-specific wording when extending
the templates; no native-speaker certification is claimed here.

Run the unchanged full coverage checker against the old baseline before
refreshing it. Use a separate output directory so the original is preserved:

```sh
cargo build --release --locked --bin link-calculator
node scripts/competitor-coverage.mjs \
  --baseline docs/case-studies/issue-227/results/coverage-baseline.json \
  --out ci-logs/coverage-expanded --quiet
```

Only after the comparison succeeds, copy the generated baseline and report
from that directory into `docs/case-studies/issue-227/results/`. The report
retains its per-language table for all 23 languages; its English count also
includes the English competitor corpora. The inventory above counts only
scientific natural-language rows.

## Measured result and limits

The release CLI evaluates the 4,249-row full corpus with 656 supported,
268 different, 3,325 unsupported and zero timeouts. No original baseline case
regressed or disappeared. The larger corpus reduces aggregate coverage from
20.9% to 15.4% by exposing additional unsupported inputs; the parser has not
changed. Scientific-only coverage remains small, as expected for the parser
work tracked by #233–#238.

The inventory now meets the per-language and per-construct requirements. It is
a controlled benchmark, not a representative statistical sample of all
scientific literature: derived templates dominate, Wikipedia samples focus on
light speed, and the three paper samples are English gravitational-wave
abstracts. Further domains, native-authored papers in more languages,
decreasing-percent phrases, and independent linguistic review would improve
representativeness. Empty-oracle quotation rows must gain semantic oracles
when the evaluator's sentence/uncertainty result format is defined.
