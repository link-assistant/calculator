// Additional WASM type declarations for functions not automatically recognized

declare module '@wasm/link_calculator' {
  // Re-export all the generated types
  export * from '../../public/pkg/link_calculator';

  // Explicitly declare the functions that TypeScript might not recognize
  export function fetch_exchange_rates(base_currency: string): Promise<string>;
  export function fetch_historical_rates(base_currency: string, date: string): Promise<string>;
}
