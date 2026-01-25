declare module 'lino-objects-codec' {
  export interface EncodeOptions {
    obj: unknown;
  }

  export interface DecodeOptions {
    notation: string;
  }

  export interface JsonToLinoOptions {
    json: unknown;
  }

  export interface LinoToJsonOptions {
    lino: string;
  }

  export interface EscapeReferenceOptions {
    value: string;
  }

  export function encode(options: EncodeOptions): string;
  export function decode(options: DecodeOptions): unknown;
  export function jsonToLino(options: JsonToLinoOptions): string;
  export function linoToJson(options: LinoToJsonOptions): unknown;
  export function escapeReference(options: EscapeReferenceOptions): string;
}
