import { describe, expect, it, vi } from 'vitest';
import { loadCbrRatesFromLinoFiles, type CbrLinoCalculator } from './worker-cbr-lino-loader';

const inrRubLino = `rates:
  from INR
  to RUB
  source 'cbr.ru (Central Bank of Russia)'
  data:
    2026-04-11 0.830794
    2026-04-24 0.7954370000000001`;

function okTextResponse(content: string): Response {
  return {
    ok: true,
    status: 200,
    text: vi.fn().mockResolvedValue(content),
  } as unknown as Response;
}

describe('loadCbrRatesFromLinoFiles', () => {
  it('loads historical CBR rates before applying the latest fallback rate', async () => {
    const calculator: CbrLinoCalculator = {
      load_rates_from_consolidated_lino: vi.fn().mockReturnValue(2),
      update_cbr_rates_from_api: vi.fn().mockReturnValue(1),
    };
    const parseConsolidatedLinoRates = vi.fn().mockReturnValue(JSON.stringify({
      success: true,
      from: 'INR',
      to: 'RUB',
      source: 'cbr.ru (Central Bank of Russia)',
      rates: [
        { date: '2026-04-11', value: 0.830794 },
        { date: '2026-04-24', value: 0.7954370000000001 },
      ],
    }));
    const fetchImpl = vi.fn().mockResolvedValue(okTextResponse(inrRubLino));

    const result = await loadCbrRatesFromLinoFiles({
      calculator,
      parseConsolidatedLinoRates,
      pairs: ['inr-rub'],
      fetchImpl,
      log: { debug: vi.fn() },
    });

    expect(calculator.load_rates_from_consolidated_lino).toHaveBeenCalledWith(inrRubLino);
    expect(calculator.update_cbr_rates_from_api).toHaveBeenCalledWith(
      '2026-04-24',
      JSON.stringify({ inr: 0.7954370000000001 })
    );
    expect(result.loadedPairCount).toBe(1);
    expect(result.loadedHistoricalRateCount).toBe(2);
    expect(result.latestDate).toBe('2026-04-24');
  });

  it('can load historical rates without overwriting current API rates', async () => {
    const calculator: CbrLinoCalculator = {
      load_rates_from_consolidated_lino: vi.fn().mockReturnValue(2),
      update_cbr_rates_from_api: vi.fn().mockReturnValue(1),
    };
    const parseConsolidatedLinoRates = vi.fn().mockReturnValue(JSON.stringify({
      success: true,
      from: 'INR',
      rates: [
        { date: '2026-04-11', value: 0.830794 },
        { date: '2026-04-24', value: 0.7954370000000001 },
      ],
    }));

    const result = await loadCbrRatesFromLinoFiles({
      calculator,
      parseConsolidatedLinoRates,
      pairs: ['inr-rub'],
      fetchImpl: vi.fn().mockResolvedValue(okTextResponse(inrRubLino)),
      applyLatestRates: false,
      log: { debug: vi.fn() },
    });

    expect(calculator.load_rates_from_consolidated_lino).toHaveBeenCalledWith(inrRubLino);
    expect(calculator.update_cbr_rates_from_api).not.toHaveBeenCalled();
    expect(result.loadedHistoricalRateCount).toBe(2);
  });
});
