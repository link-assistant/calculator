import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';
import path from 'node:path';

export default defineConfig({
  plugins: [react()],
  base: '/calculator/',
  server: {
    fs: {
      // Allow importing shared data from the repository root
      // (e.g. `../../../data/examples.lino?raw` in src/examples/index.ts).
      allow: [path.resolve(import.meta.dirname, '..')],
    },
  },
  build: {
    outDir: 'dist',
    target: 'esnext',
    chunkSizeWarningLimit: 700,
  },
  worker: {
    format: 'es',
  },
});
