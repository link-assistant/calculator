/// <reference types="vite/client" />

// Type declarations for Links Notation files imported as raw strings
declare module '*.lino?raw' {
  const content: string;
  export default content;
}
