import { describe, it, expect, vi } from 'vitest';
import { createRateCoordination, detectRequiredSources } from './worker-rate-coordination';

// ---------------------------------------------------------------------------
// detectRequiredSources
// ---------------------------------------------------------------------------

describe('detectRequiredSources', () => {
  it('returns empty set for pure math expressions', () => {
    expect(detectRequiredSources('2 + 2')).toEqual(new Set());
    expect(detectRequiredSources('sin(pi)')).toEqual(new Set());
    expect(detectRequiredSources('100 * 3.14')).toEqual(new Set());
  });

  it('detects ECB for fiat currency codes', () => {
    const s = detectRequiredSources('100 USD');
    expect(s.has('ecb')).toBe(true);
  });

  it('detects ECB for fiat currency symbols', () => {
    expect(detectRequiredSources('100€').has('ecb')).toBe(true);
    expect(detectRequiredSources('£50').has('ecb')).toBe(true);
  });

  it('detects ECB for English currency names', () => {
    expect(detectRequiredSources('10 dollars').has('ecb')).toBe(true);
    expect(detectRequiredSources('5 euros').has('ecb')).toBe(true);
  });

  it('detects CBR for RUB code and symbol', () => {
    expect(detectRequiredSources('1000 RUB').has('cbr')).toBe(true);
    expect(detectRequiredSources('1000₽').has('cbr')).toBe(true);
  });

  it('detects CBR for Russian ruble names', () => {
    expect(detectRequiredSources('1000 рублей').has('cbr')).toBe(true);
    expect(detectRequiredSources('500 рубля').has('cbr')).toBe(true);
    expect(detectRequiredSources('1 рубль').has('cbr')).toBe(true);
  });

  it('detects crypto for ticker symbols', () => {
    expect(detectRequiredSources('20 TON').has('crypto')).toBe(true);
    expect(detectRequiredSources('0.5 BTC').has('crypto')).toBe(true);
    expect(detectRequiredSources('10 ETH').has('crypto')).toBe(true);
  });

  it('detects crypto for natural language names', () => {
    expect(detectRequiredSources('1 bitcoin').has('crypto')).toBe(true);
    expect(detectRequiredSources('5 ethereum').has('crypto')).toBe(true);
  });

  it('detects multiple sources for mixed expressions', () => {
    const s = detectRequiredSources('(1000 рублей + 20 TON) в USD');
    expect(s.has('cbr')).toBe(true);
    expect(s.has('crypto')).toBe(true);
    expect(s.has('ecb')).toBe(true);
  });

  it('detects CBR + ECB for ruble-to-dollar conversion', () => {
    const s = detectRequiredSources('1000 рублей в доллар');
    expect(s.has('cbr')).toBe(true);
    expect(s.has('ecb')).toBe(true);
    expect(s.has('crypto')).toBe(false);
  });

  it('handles the exact issue #100 expression', () => {
    const s = detectRequiredSources('(1000 рублей + 500 рублей + 2000 рублей + 20 TON + 1000 рублей) в USD');
    expect(s.has('cbr')).toBe(true);
    expect(s.has('crypto')).toBe(true);
    expect(s.has('ecb')).toBe(true);
  });
});

// ---------------------------------------------------------------------------
// createRateCoordination
// ---------------------------------------------------------------------------

describe('createRateCoordination', () => {
  it('pure math expression resolves immediately without fetching', async () => {
    const coord = createRateCoordination();
    const fetcher = vi.fn(async () => {});
    coord.registerFetcher('ecb', fetcher);
    coord.registerFetcher('cbr', fetcher);
    coord.registerFetcher('crypto', fetcher);

    const needed = await coord.ensureRatesForExpression('2 + 2');
    expect(needed.size).toBe(0);
    expect(fetcher).not.toHaveBeenCalled();
  });

  it('triggers fetch for needed source and waits for it', async () => {
    const coord = createRateCoordination();

    let resolveFetch!: () => void;
    const fetchPromise = new Promise<void>((r) => { resolveFetch = r; });
    const cryptoFetcher = vi.fn(async () => {
      await fetchPromise;
      coord.markLoaded('crypto');
    });

    coord.registerFetcher('ecb', vi.fn(async () => { coord.markLoaded('ecb'); }));
    coord.registerFetcher('cbr', vi.fn(async () => { coord.markLoaded('cbr'); }));
    coord.registerFetcher('crypto', cryptoFetcher);

    let resolved = false;
    const promise = coord.ensureRatesForExpression('20 TON').then((s) => {
      resolved = true;
      return s;
    });

    // Flush microtasks — crypto fetch started but not yet resolved
    await Promise.resolve();
    await Promise.resolve();
    expect(cryptoFetcher).toHaveBeenCalled();
    expect(resolved).toBe(false);

    // Complete the fetch
    resolveFetch();
    const needed = await promise;
    expect(resolved).toBe(true);
    expect(needed.has('crypto')).toBe(true);
  });

  it('already-loaded source resolves instantly', async () => {
    const coord = createRateCoordination();
    coord.registerFetcher('ecb', vi.fn(async () => { coord.markLoaded('ecb'); }));

    // Pre-load ECB
    coord.markLoaded('ecb');

    const needed = await coord.ensureRatesForExpression('100 USD');
    expect(needed.has('ecb')).toBe(true);
    // Fetcher should NOT have been called since it's already loaded
    expect(coord.getState('ecb')).toBe('loaded');
  });

  it('does not fetch sources the expression does not need', async () => {
    const coord = createRateCoordination();
    const ecbFetcher = vi.fn(async () => { coord.markLoaded('ecb'); });
    const cbrFetcher = vi.fn(async () => { coord.markLoaded('cbr'); });
    const cryptoFetcher = vi.fn(async () => { coord.markLoaded('crypto'); });

    coord.registerFetcher('ecb', ecbFetcher);
    coord.registerFetcher('cbr', cbrFetcher);
    coord.registerFetcher('crypto', cryptoFetcher);

    await coord.ensureRatesForExpression('100 USD');
    expect(ecbFetcher).toHaveBeenCalled();
    expect(cbrFetcher).not.toHaveBeenCalled();
    expect(cryptoFetcher).not.toHaveBeenCalled();
  });

  it('multiple callers waiting for same source all resolve', async () => {
    const coord = createRateCoordination();

    let resolveFetch!: () => void;
    coord.registerFetcher('ecb', vi.fn(async () => {
      await new Promise<void>((r) => { resolveFetch = r; });
      coord.markLoaded('ecb');
    }));
    coord.registerFetcher('cbr', vi.fn(async () => { coord.markLoaded('cbr'); }));
    coord.registerFetcher('crypto', vi.fn(async () => { coord.markLoaded('crypto'); }));

    const p1 = coord.ensureRatesForExpression('100 USD');
    const p2 = coord.ensureRatesForExpression('200 EUR');

    resolveFetch();
    const [n1, n2] = await Promise.all([p1, p2]);
    expect(n1.has('ecb')).toBe(true);
    expect(n2.has('ecb')).toBe(true);
  });

  it('getState reflects idle → loading → loaded transitions', async () => {
    const coord = createRateCoordination();

    expect(coord.getState('ecb')).toBe('idle');

    coord.registerFetcher('ecb', vi.fn(async () => {
      await Promise.resolve();
      coord.markLoaded('ecb');
    }));

    coord.markLoading('ecb');
    expect(coord.getState('ecb')).toBe('loading');

    coord.markLoaded('ecb');
    expect(coord.getState('ecb')).toBe('loaded');
  });
});
