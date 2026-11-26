import { defineConfig } from 'vite';
import react from '@vitejs/plugin-react';

export default defineConfig({
  base: '/territories/',
  plugins: [react()],
  server: {
    host: true,
  },
  preview: {
    host: true,
    port: 4173,
  },
  worker: {
    format: 'es',
  },
  build: {
    target: 'esnext',
  },
  optimizeDeps: {
    exclude: ['onnxruntime-web'],
  },
});
