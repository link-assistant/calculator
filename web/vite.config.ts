import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import wasm from 'vite-plugin-wasm';
import path from 'node:path';

export default defineConfig({
  plugins: [react(), wasm()],
  base: '/calculator/',
  resolve: {
    alias: {
      '@wasm': path.resolve(import.meta.dirname, 'public/pkg'),
    },
  },
  server: {
    fs: {
      // Allow importing shared data from the repository root
      // (e.g. `../../../data/examples.lino?raw` in src/examples/index.ts).
      allow: [path.resolve(import.meta.dirname, '..')],
    },
  },
  build: {
    outDir: 'dist',
    // `esnext` supports top-level await natively, so the WASM glue code
    // works without vite-plugin-top-level-await.
    target: 'esnext',
    chunkSizeWarningLimit: 700,
  },
  worker: {
    format: 'es',
    plugins: () => [wasm()],
  },
});
