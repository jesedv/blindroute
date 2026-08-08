import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vite';

// `base: './'` so the built site works from GitHub Pages sub-path or any
// static host (relative asset URLs).
export default defineConfig({
  plugins: [svelte()],
  base: './',
  build: {
    outDir: 'dist',
    assetsDir: 'assets',
  },
  server: {
    port: 5173,
    fs: {
      // allow serving /public files (incl. the wasm-bindgen pkg) in dev
      allow: ['..'],
    },
  },
});
