#!/usr/bin/env node
// Refresh licensed source snapshots; review candidates before selecting new rows.
import { mkdirSync, writeFileSync } from 'node:fs';
import { fileURLToPath } from 'node:url';
import { locales } from './natural-language-locales.mjs';

const directory = fileURLToPath(new URL('./sources/', import.meta.url));
mkdirSync(directory, { recursive: true });
const fetched = new Date().toISOString().slice(0, 10);
const headers = { 'User-Agent': 'link-calculator-corpus/1.0 (https://github.com/link-assistant/calculator/issues/232)' };
async function get(url, json = false) {
  const response = await fetch(url, { headers });
  if (!response.ok) throw new Error(`${url}: HTTP ${response.status}`);
  return json ? response.json() : response.text();
}
const save = (name, data) => writeFileSync(`${directory}/${name}.json`, `${JSON.stringify(data, null, 2)}\n`);

const entity = await get('https://www.wikidata.org/wiki/Special:EntityData/Q2111.json', true);
for (const lang of Object.keys(locales)) {
  const title = entity.entities.Q2111.sitelinks[`${lang}wiki`].title;
  const api = `https://${lang}.wikipedia.org/api/rest_v1/page/summary/${encodeURIComponent(title)}`;
  const summary = await get(api, true);
  if (!summary.extract || !summary.revision) throw new Error(`${lang}: missing summary/revision`);
  const numericSentences = (text) => [...new Intl.Segmenter(lang, { granularity: 'sentence' }).segment(text)]
    .map(({ segment }) => segment.trim()).filter((sentence) => /\p{Nd}/u.test(sentence));
  let extract = summary.extract;
  let sentences = numericSentences(extract);
  if (sentences.length === 0) {
    // Some introductions omit the numerical value; sample the article body.
    const query = new URLSearchParams({ action: 'query', prop: 'extracts', explaintext: '1',
      format: 'json', titles: title, revids: summary.revision });
    // Query by pinned revision so the citation and extracted text stay aligned.
    query.delete('titles');
    const article = await get(`https://${lang}.wikipedia.org/w/api.php?${query}`, true);
    const body = Object.values(article.query.pages)[0].extract;
    sentences = numericSentences(body).filter((sentence) =>
      /\p{Nd}[\p{Nd}\s.,٬٫]*\s*(?:m\/s|km|meter|m\b|מטר|קילומטר)/u.test(sentence)).slice(0, 3);
    extract = sentences.join('\n');
  }
  if (sentences.length === 0) throw new Error(`${lang}: no numeric sentence; review another source`);
  save(`wikipedia-${lang}`, {
    lang, title: summary.title, authors: 'Wikipedia contributors', fetched,
    url: summary.content_urls.desktop.page,
    revisionUrl: `https://${lang}.wikipedia.org/w/index.php?oldid=${summary.revision}`,
    license: 'CC-BY-SA-4.0', licenseUrl: 'https://creativecommons.org/licenses/by-sa/4.0/',
    extract, candidates: sentences,
  });
  console.log(`${lang}: ${sentences.length} numeric Wikipedia sentences`);
}

// Each abstract has an explicit CC BY link on its own arXiv landing page.
for (const id of ['1602.03837v1', '1706.01812v2', '1710.05832v1']) {
  const url = `https://arxiv.org/abs/${id}`;
  const html = await get(url);
  const license = html.match(/href="(https?:\/\/creativecommons\.org\/licenses\/by\/4\.0\/)"/);
  if (!license) throw new Error(`${id}: expected explicit CC BY 4.0 license`);
  // arXiv's citation metadata contains the abstract including its original TeX.
  const decode = (value) => value.replace(/&quot;/g, '"').replace(/&amp;/g, '&').replace(/&#39;/g, "'").replace(/&lt;/g, '<').replace(/&gt;/g, '>');
  const abstract = html.match(/name="citation_abstract" content="([^"]*)"/);
  const title = html.match(/name="citation_title" content="([^"]*)"/);
  if (!abstract || !title) throw new Error(`${id}: missing citation metadata`);
  save(`arxiv-${id}`, {
    lang: 'en', title: decode(title[1]), authors: 'LIGO Scientific Collaboration and Virgo Collaboration',
    url, fetched, license: 'CC-BY-4.0', licenseUrl: license[1].replace('http:', 'https:'),
    extract: decode(abstract[1]),
  });
  console.log(`${id}: licensed paper abstract`);
}
