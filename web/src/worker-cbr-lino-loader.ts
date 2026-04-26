export const LINO_RUB_PAIRS = [
  'usd-rub',
  'eur-rub',
  'gbp-rub',
  'jpy-rub',
  'chf-rub',
  'cny-rub',
  'inr-rub',
  'kzt-rub',
] as const;

export interface CbrLinoCalculator {
  update_cbr_rates_from_api(date: string, rates_json: string): number;
  load_rates_from_consolidated_lino(content: string): number;
}

export interface CbrLinoLoadResult {
  loadedPairCount: number;
  loadedHistoricalRateCount: number;
  latestDate?: string;
}

interface ConsolidatedRateEntry {
  date: string;
  value: number;
}

interface ConsolidatedRatesParseResult {
  success: boolean;
  from?: string;
  rates?: ConsolidatedRateEntry[];
  error?: string;
}

interface LoadCbrRatesFromLinoFilesOptions {
  calculator: CbrLinoCalculator | null;
  parseConsolidatedLinoRates: (content: string) => string;
  baseUrl?: string;
  pairs?: readonly string[];
  fetchImpl?: typeof fetch;
  applyLatestRates?: boolean;
  log?: Pick<Console, 'debug'>;
}

export async function loadCbrRatesFromLinoFiles({
  calculator,
  parseConsolidatedLinoRates,
  baseUrl = '/calculator/data/currency',
  pairs = LINO_RUB_PAIRS,
  fetchImpl = fetch,
  applyLatestRates = true,
  log = console,
}: LoadCbrRatesFromLinoFilesOptions): Promise<CbrLinoLoadResult> {
  const latestRatesByCurrency: Record<string, number> = {};
  let latestDate = '';
  let loadedPairCount = 0;
  let loadedHistoricalRateCount = 0;

  for (const pair of pairs) {
    const [fallbackFrom] = pair.split('-');

    try {
      const response = await fetchImpl(`${baseUrl}/${pair}.lino`);
      if (!response.ok) {
        log.debug(`[CBR fallback] ${pair}.lino not found (${response.status})`);
        continue;
      }

      const content = await response.text();
      if (calculator) {
        loadedHistoricalRateCount += calculator.load_rates_from_consolidated_lino(content);
      }

      const parsed = JSON.parse(parseConsolidatedLinoRates(content)) as ConsolidatedRatesParseResult;
      if (!parsed.success || !parsed.rates || parsed.rates.length === 0) {
        log.debug(`[CBR fallback] ${pair}.lino had no parseable rates (${parsed.error ?? 'unknown error'})`);
        continue;
      }

      const latestEntry = parsed.rates[parsed.rates.length - 1];
      const from = (parsed.from ?? fallbackFrom).toLowerCase();
      latestRatesByCurrency[from] = latestEntry.value;
      if (latestEntry.date > latestDate) {
        latestDate = latestEntry.date;
      }

      loadedPairCount += 1;
      log.debug(
        `[CBR fallback] Loaded ${pair}: ${parsed.rates.length} historical rates, latest 1 ${from.toUpperCase()} = ${latestEntry.value} RUB (${latestEntry.date})`
      );
    } catch (err) {
      log.debug(`[CBR fallback] Failed to load ${pair}.lino:`, err);
    }
  }

  if (applyLatestRates && loadedPairCount > 0 && calculator) {
    calculator.update_cbr_rates_from_api(latestDate, JSON.stringify(latestRatesByCurrency));
  }

  return {
    loadedPairCount,
    loadedHistoricalRateCount,
    latestDate: latestDate || undefined,
  };
}
