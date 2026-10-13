// Type declarations for the native JS engine (`link-calculator/wasm`, js/src/wasm.js).
// It exposes the same API as the former wasm-pack build (snake_case names);
// `init`/`initSync` are no-ops because there is no WebAssembly module to load.

declare module 'link-calculator/wasm' {
  export class Calculator {
    constructor();
    static version(): string;
    plan(input: string): string;
    execute(input: string): string;
    calculate(input: string): string;
    update_rates_from_api(base: string, date: string, rates_json: string): number;
    update_crypto_rates_from_api(base: string, date: string, rates_json: string): number;
    update_cbr_rates_from_api(date: string, rates_json: string): number;
    load_rates_from_consolidated_lino(content: string): number;
    set_timezone_offset(offset_minutes: number): void;
    clear_timezone_offset(): void;
  }

  export default function init(moduleOrPath?: unknown): Promise<void>;
  export function initSync(module?: unknown): void;
  export function fetch_exchange_rates(base_currency: string): Promise<string>;
  export function fetch_historical_rates(base_currency: string, date: string): Promise<string>;
  export function fetch_crypto_rates(vs_currency: string): Promise<string>;
  export function fetch_cbr_rates(): Promise<string>;
  export function parse_consolidated_lino_rates(content: string): string;
  export function parse_lino_rate(content: string): string;
  export function get_version(): string;
  export function health_check(): boolean;
}
