import { afterEach, describe, expect, test, vi } from 'vitest';
import { now, setFixedNow } from '../src/clock.js';
import {
  CryptoApiError,
  coingeckoId,
  coingeckoPriceFromJson,
  fetchCryptoPrice,
  fetchCryptoPrices,
} from '../src/crypto_api.js';
import { Err } from '../src/rust/core.js';
import { Utc } from '../src/rust/chrono.js';

describe('crypto_api', () => {
  test('test_coingecko_id_ton', () => {
    expect(coingeckoId('TON')).toBe('the-open-network');
    expect(coingeckoId('ton')).toBe('the-open-network');
    expect(coingeckoId('TONCOIN')).toBe('the-open-network');
  });

  test('test_coingecko_id_bitcoin', () => {
    expect(coingeckoId('BTC')).toBe('bitcoin');
    expect(coingeckoId('BITCOIN')).toBe('bitcoin');
    expect(coingeckoId('btc')).toBe('bitcoin');
  });

  test('test_coingecko_id_ethereum', () => {
    expect(coingeckoId('ETH')).toBe('ethereum');
    expect(coingeckoId('ethereum')).toBe('ethereum');
  });

  test('test_coingecko_id_unknown', () => {
    expect(coingeckoId('UNKNOWN')).toBe(null);
    expect(coingeckoId('XYZ')).toBe(null);
  });

  test('test_crypto_api_error_display', () => {
    let e = CryptoApiError.NetworkError('timeout');
    expect(e.toString()).toContain('Network error');

    e = CryptoApiError.ParseError('invalid json');
    expect(e.toString()).toContain('Parse error');

    e = CryptoApiError.CoinNotFound({ ticker: 'XYZ' });
    expect(e.toString()).toContain('XYZ');
    expect(e.fmtDebug()).toBe('CoinNotFound { ticker: "XYZ" }');
  });
});

// JS-only: the fetch path, with a stubbed global fetch (no network).
describe('crypto_api fetch (stubbed)', () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    setFixedNow(null);
  });

  test('price lookup errors', () => {
    expect(coingeckoPriceFromJson({ bitcoin: { usd: 5 } }, 'bitcoin', 'usd')).toBe(5);
    try {
      coingeckoPriceFromJson({ bitcoin: { usd: '5' } }, 'bitcoin', 'usd');
      expect.unreachable();
    } catch (e) {
      expect(e).toBeInstanceOf(Err);
      expect(String(/** @type {Err} */ (e).error)).toBe('Parse error: Price not found for bitcoin in usd');
    }
  });

  test('fetchCryptoPrice sends the CoinGecko request', async () => {
    setFixedNow(Utc.withYmdAndHms(2026, 3, 15, 12, 0, 0).unwrap());
    /** @type {Request[]} */
    const requests = [];
    vi.stubGlobal('fetch', async (/** @type {Request} */ req) => {
      requests.push(req);
      return new Response(JSON.stringify({ 'the-open-network': { usd: 2.5 } }));
    });
    const info = await fetchCryptoPrice('ton', 'USD');
    expect(requests[0].url).toBe('https://api.coingecko.com/api/v3/simple/price?ids=the-open-network&vs_currencies=usd');
    expect(requests[0].headers.get('Accept')).toBe('application/json');
    expect(info.rate).toBe(2.5);
    expect(info.source).toBe('coingecko.com');
    expect(info.date).toBe('2026-03-15');
    expect(info.fetchedAt).toBe(now().toRfc3339());
  });

  test('fetchCryptoPrices maps tickers and skips missing prices', async () => {
    vi.stubGlobal('fetch', async () => new Response(JSON.stringify({ bitcoin: { usd: 1 }, ethereum: {} })));
    const result = await fetchCryptoPrices(['btc', 'ETH', 'nope'], 'usd');
    expect([...result.keys()]).toEqual(['BTC']);
    expect(await fetchCryptoPrices(['nope'], 'usd')).toEqual(new Map());
  });

  test('errors', async () => {
    await expect(fetchCryptoPrice('xyz', 'usd')).rejects.toEqual(new Err(CryptoApiError.CoinNotFound({ ticker: 'XYZ' })));
    vi.stubGlobal('fetch', async () => new Response('', { status: 503 }));
    await expect(fetchCryptoPrice('btc', 'usd')).rejects.toEqual(new Err(CryptoApiError.NetworkError('HTTP error: 503')));
    vi.stubGlobal('fetch', async () => {
      throw new TypeError('boom');
    });
    await expect(fetchCryptoPrice('btc', 'usd')).rejects.toEqual(
      new Err(CryptoApiError.NetworkError('Fetch failed: JsValue(TypeError: boom)')),
    );
  });
});
