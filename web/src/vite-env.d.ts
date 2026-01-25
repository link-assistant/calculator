/// <reference types="vite/client" />

// Type declarations for Links Notation files imported as raw strings
declare module '*.lino?raw' {
  const content: string;
  export default content;
}

// Type declarations for the WASM calculator module
// The actual module is built by wasm-pack to web/pkg/
declare module '*.wasm' {
  const content: WebAssembly.Module;
  export default content;
}

declare module '@wasm/link_calculator' {
  export interface CalculatorInstance {
    calculate(input: string): string;
    free(): void;
  }

  export interface CalculatorClass {
    new(): CalculatorInstance;
    version(): string;
  }

  export const Calculator: CalculatorClass;

  export default function init(input?: RequestInfo | URL | Response | WebAssembly.Module): Promise<void>;
}
