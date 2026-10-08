# Competitor documentation inventory (issues #227 and #231)

Fetched on 2026-10-08. This table inventories the documentation portion of
issue #227, including the additional products named in #231. Machine-readable
page URLs, fetch dates, content hashes, image audits and extraction counts are in
[documentation-sources.json](documentation-sources.json).

| Product | Reference pages | Extracted page examples | Unique corpus rows | Extractor or audit |
| --- | ---: | ---: | ---: | --- |
| [Apple Math Notes](https://support.apple.com/guide/ipad/solve-math-with-math-notes-ipadeb38d0f8/ipados) | 4 | 10 | 15 | [apple](../../../experiments/issue-227/extract-apple-examples.mjs) |
| [Calca](https://calca.io/reference) | 2 | 201 | 209 | [calca](../../../experiments/issue-227/extract-calca-examples.mjs) |
| [DuckDuckGo](https://duckduckgo.com/duckduckgo-help-pages/features/instant-answers-and-other-features) | 1 | 0 | 0 | No typed examples in official help |
| [Frink](https://frinklang.org/frinkdocs.html) | 1 | 1247 | 1252 | [frink](../../../experiments/issue-227/extract-frink-examples.mjs) |
| [Google](https://support.google.com/websearch/answer/3284611?hl=en) | 1 | 7 | 16 | [google](../../../experiments/issue-227/extract-google-examples.mjs) |
| [Hurmet](https://hurmet.org/manual.html) | 1 | 460 | 460 | [hurmet](../../../experiments/issue-227/extract-hurmet-examples.mjs) |
| [Numi](https://github.com/nikolaeu/numi/wiki) | 1 | 77 | 100 | [numi](../../../experiments/issue-227/extract-numi-examples.mjs) |
| [Parsify](https://parsify.notion.site/Getting-started-be7132e43e844bd88fe2ad48918b43d7) | 2 | 36 | 36 | [parsify](../../../experiments/issue-227/extract-parsify-examples.mjs) |
| [Raycast](https://manual.raycast.com/calculator) | 1 | 51 | 58 | [raycast](../../../experiments/issue-227/extract-raycast-examples.mjs) |
| [Samsung Calculator](https://www.samsung.com/us/support/answer/ANS10002547/) | 1 | 0 | 0 | No typed examples in official help |
| [Soulver](https://documentation.soulver.app/documentation/exporting.md) | 67 | 385 | 383 | [soulver](../../../experiments/issue-227/extract-soulver-examples.mjs) |
| [SpeedCrunch](https://www.speedcrunch.org/reference/basic.html) | 10 | 385 | 372 | [speedcrunch](../../../experiments/issue-227/extract-speedcrunch-examples.mjs) |
| [Windows Calculator](https://support.microsoft.com/en-gb/windows/apps/use-the-calculator-in-windows) | 1 | 0 | 0 | No typed examples in official help |
| [Wolfram|Alpha](https://www.wolframalpha.com/examples/mathematics) | 359 | 6167 | 3087 | [wolfram](../../../experiments/issue-227/extract-wolfram-examples.mjs) |
| [Xiaomi Calculator](https://www.mi.com/uk/support/article/KA-06518/) | 1 | 0 | 0 | No typed examples in official help |

The extracted count includes an expression repeated on different pages. Unique
corpus rows also include preserved historical seed examples, so either count can
be larger. Zero means that the audited official help contains no typed examples;
see [documentation-corpus.md](documentation-corpus.md) for evidence and boundaries.

Hurmet and SpeedCrunch publish open-source implementations, but their manuals
belong to this documentation corpus. The existing
[open-source-tests.tsv](corpus/open-source-tests.tsv) separately imports test
cases from fend, libqalculate, Numbat, Rink and math.js. The natural-language
references remain in [natural-language-quotes.tsv](corpus/natural-language-quotes.tsv).

The earlier inventory and research are in [parent PR #228](https://github.com/link-assistant/calculator/pull/228).
Current executable classifications are in [coverage-report.md](results/coverage-report.md).
