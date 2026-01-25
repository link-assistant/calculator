declare module 'links-notation' {
  export interface Link {
    source?: string | Link;
    target?: string | Link;
    [key: string]: unknown;
  }

  export class Parser {
    constructor();
    parse(input: string): Link[];
  }
}
