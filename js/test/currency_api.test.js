import { afterEach, describe, expect, test, vi } from 'vitest';
import {
  API_SOURCE,
  CBR_API_SOURCE,
  CurrencyApiError,
  extractXmlText,
  fetchCbrRates,
  fetchHistoricalRates,
  fetchRate,
  parseCbrXml,
  parseFrankfurterJson,
  ratesToExchangeInfo,
} from '../src/currency_api.js';
import { Err, expect as rustExpect, isOk, unwrapResult } from '../src/rust/core.js';

describe('currency_api', () => {
  test('test_api_source_constant', () => {
    // API source should be ECB via Frankfurter
    expect(API_SOURCE).toBe('frankfurter.dev (ECB)');
  });

  test('test_rates_to_exchange_info', () => {
    const rates = new Map([
      ['eur', 0.92],
      ['gbp', 0.79],
    ]);

    const result = ratesToExchangeInfo('USD', '2026-01-25', rates);

    expect(result.length).toBe(2);

    // Find EUR rate
    const eurRate = rustExpect(result.find(([, to]) => to === 'EUR') ?? null, 'EUR rate should exist');
    expect(eurRate[0]).toBe('USD');
    expect(eurRate[1]).toBe('EUR');
    expect(eurRate[2].rate).toBe(0.92);
    expect(eurRate[2].source).toBe(API_SOURCE);
    expect(eurRate[2].date).toBe('2026-01-25');
  });

  test('test_currency_api_error_display', () => {
    let e = CurrencyApiError.NetworkError('timeout');
    expect(e.toString()).toContain('Network error');

    e = CurrencyApiError.ParseError('invalid json');
    expect(e.toString()).toContain('Parse error');

    e = CurrencyApiError.RateNotFound({ from: 'USD', to: 'XYZ' });
    expect(e.toString()).toContain('USD/XYZ');
  });

  test('test_cbr_api_source_constant', () => {
    expect(CBR_API_SOURCE).toBe('cbr.ru (Central Bank of Russia)');
  });

  test('test_parse_cbr_xml_basic', () => {
    const xml = `<?xml version="1.0" encoding="windows-1251"?>
<ValCurs Date="25.02.2026" name="Foreign Currency Market">
  <Valute ID="R01235">
    <NumCode>840</NumCode>
    <CharCode>USD</CharCode>
    <Nominal>1</Nominal>
    <Name>Доллар США</Name>
    <Value>76,6342</Value>
    <VunitRate>76,6342</VunitRate>
  </Valute>
  <Valute ID="R01239">
    <NumCode>978</NumCode>
    <CharCode>EUR</CharCode>
    <Nominal>1</Nominal>
    <Name>Евро</Name>
    <Value>90,5821</Value>
    <VunitRate>90,5821</VunitRate>
  </Valute>
  <Valute ID="R01270">
    <NumCode>356</NumCode>
    <CharCode>INR</CharCode>
    <Nominal>100</Nominal>
    <Name>Индийская рупия</Name>
    <Value>84,2448</Value>
    <VunitRate>0,842448</VunitRate>
  </Valute>
</ValCurs>`;

    expect(isOk(() => parseCbrXml(xml)), 'Should parse CBR XML successfully').toBe(true);

    const [date, rates] = unwrapResult(() => parseCbrXml(xml));
    expect(date).toBe('2026-02-25');

    // USD rate: 1 USD = 76.6342 RUB
    const usdRate = rustExpect(rates.get('usd') ?? null, 'USD rate should exist');
    expect(Math.abs(usdRate - 76.6342) < 0.001, 'USD rate should be ~76.6342').toBe(true);

    // EUR rate: 1 EUR = 90.5821 RUB
    const eurRate = rustExpect(rates.get('eur') ?? null, 'EUR rate should exist');
    expect(Math.abs(eurRate - 90.5821) < 0.001, 'EUR rate should be ~90.5821').toBe(true);

    // INR rate: 100 INR = 84.2448 RUB -> 1 INR = 0.842448 RUB
    const inrRate = rustExpect(rates.get('inr') ?? null, 'INR rate should exist');
    expect(Math.abs(inrRate - 0.842448) < 0.001, 'INR rate should be ~0.842448 (100 INR = 84.2448 RUB)').toBe(true);
  });

  test('test_parse_cbr_xml_date_conversion', () => {
    const xml = `<ValCurs Date="01.03.2026" name="Foreign Currency Market">
  <Valute ID="R01235">
    <CharCode>USD</CharCode>
    <Nominal>1</Nominal>
    <Value>76,0000</Value>
  </Valute>
</ValCurs>`;

    const [date] = unwrapResult(() => parseCbrXml(xml));
    expect(date, 'Date should be converted from dd.mm.yyyy to yyyy-mm-dd').toBe('2026-03-01');
  });

  test('test_parse_cbr_xml_missing_date', () => {
    const xml = `<ValCurs name="Foreign Currency Market">
  <Valute ID="R01235">
    <CharCode>USD</CharCode>
    <Nominal>1</Nominal>
    <Value>76,0000</Value>
  </Valute>
</ValCurs>`;

    expect(isOk(() => parseCbrXml(xml)), 'Should fail without Date attribute').toBe(false);
  });

  test('test_parse_cbr_xml_nominal_division', () => {
    // Test that nominal > 1 is handled correctly (e.g., 100 JPY = X RUB)
    const xml = `<ValCurs Date="25.02.2026" name="Foreign Currency Market">
  <Valute ID="R01820">
    <CharCode>JPY</CharCode>
    <Nominal>100</Nominal>
    <Value>49,4989</Value>
  </Valute>
</ValCurs>`;

    const [, rates] = unwrapResult(() => parseCbrXml(xml));
    const jpyRate = rustExpect(rates.get('jpy') ?? null, 'JPY rate should exist');
    // 100 JPY = 49.4989 RUB -> 1 JPY = 0.494989 RUB
    expect(Math.abs(jpyRate - 0.494989) < 0.001, 'JPY rate should be ~0.494989').toBe(true);
  });

  test('test_parse_cbr_xml_empty', () => {
    const xml = `<ValCurs Date="25.02.2026" name="Foreign Currency Market">
</ValCurs>`;

    expect(isOk(() => parseCbrXml(xml)), 'Should fail with no rates').toBe(false);
  });

  test('test_extract_xml_text', () => {
    const block = '<CharCode>USD</CharCode><Nominal>1</Nominal>';
    expect(extractXmlText(block, 'CharCode')).toBe('USD');
    expect(extractXmlText(block, 'Nominal')).toBe('1');
    expect(extractXmlText(block, 'Missing')).toBe(null);
  });
});

// JS-only: the fetch path, with a stubbed global fetch (no network).
describe('currency_api fetch (stubbed)', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
  });

  test('parseFrankfurterJson lowercases keys and rejects bad shapes', () => {
    const [date, rates] = parseFrankfurterJson({ base: 'USD', date: '2026-01-02', rates: { EUR: 0.9 } });
    expect(date).toBe('2026-01-02');
    expect([...rates]).toEqual([['eur', 0.9]]);
    expect(() => parseFrankfurterJson({ base: 'USD', rates: {} })).toThrow(Err);
  });

  test('fetchHistoricalRates and fetchRate', async () => {
    /** @type {string[]} */
    const urls = [];
    vi.stubGlobal('fetch', async (/** @type {Request} */ req) => {
      urls.push(req.url);
      return new Response(JSON.stringify({ amount: 1, base: 'USD', date: '2026-01-02', rates: { EUR: 0.9 } }));
    });
    await fetchHistoricalRates('usd', '2026-01-02');
    const info = await fetchRate('usd', 'eur');
    expect(urls).toEqual(['https://api.frankfurter.app/2026-01-02?from=USD', 'https://api.frankfurter.app/latest?from=USD']);
    expect(info.rate).toBe(0.9);
    expect(info.source).toBe(API_SOURCE);
    await expect(fetchRate('usd', 'xyz')).rejects.toEqual(
      new Err(CurrencyApiError.RateNotFound({ from: 'USD', to: 'XYZ' })),
    );
  });

  test('fetchCbrRates', async () => {
    /** @type {Request[]} */
    const requests = [];
    vi.stubGlobal('fetch', async (/** @type {Request} */ req) => {
      requests.push(req);
      return new Response(
        '<ValCurs Date="25.02.2026"><Valute><CharCode>USD</CharCode><Nominal>1</Nominal><Value>76,5</Value></Valute></ValCurs>',
      );
    });
    const [date, rates] = await fetchCbrRates();
    expect(requests[0].url).toBe('https://www.cbr.ru/scripts/XML_daily.asp');
    expect(requests[0].headers.get('Accept')).toBe('application/xml, text/xml, */*');
    expect(date).toBe('2026-02-25');
    expect(rates.get('usd')).toBe(76.5);
    vi.stubGlobal('fetch', async () => new Response('', { status: 500 }));
    await expect(fetchCbrRates()).rejects.toEqual(new Err(CurrencyApiError.NetworkError('CBR HTTP error: 500')));
  });
});
