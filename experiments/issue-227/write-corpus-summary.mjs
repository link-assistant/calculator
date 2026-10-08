#!/usr/bin/env node
// Keep the inventory's documentation counts aligned with the fetched-source audit.
import { readFileSync, writeFileSync } from 'node:fs';
const root = new URL('../../docs/case-studies/issue-227/', import.meta.url);
const audit = JSON.parse(readFileSync(new URL('documentation-sources.json', root), 'utf8'));
const dates = [...new Set(audit.sources.flatMap((source) => source.pages.map((page) => page.fetched)))].sort();
const rows = audit.sources.map((source) => `| [${source.product}](${source.pages[0].url}) | ${source.pages.length} | ${source.extractedRows} | ${source.rows} | ${source.zeroExamples ? 'No typed examples in official help' : `[${source.id}](../../../${source.extractor})`} |`);
writeFileSync(new URL('competitor-inventory.md', root), `# Competitor documentation inventory (issues #227 and #231)

Fetched on ${dates.join(', ')}. This table inventories the documentation portion of
issue #227, including the additional products named in #231. Machine-readable
page URLs, fetch dates, content hashes, image audits and extraction counts are in
[documentation-sources.json](documentation-sources.json).

| Product | Reference pages | Extracted page examples | Unique corpus rows | Extractor or audit |
| --- | ---: | ---: | ---: | --- |
${rows.join('\n')}

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
`);
