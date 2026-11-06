import { defineConfig } from 'vite';
import { svelte } from '@sveltejs/vite-plugin-svelte';

const host = process.env.TAURI_DEV_HOST;

// https://vite.dev/config/
export default defineConfig({
  plugins: [svelte()],

  // Tauri muestra sus propios errores: no limpiar la consola.
  clearScreen: false,

  server: {
    port: 1420,
    strictPort: true,
    host: host || false,
    hmr: host
      ? {
          protocol: 'ws',
          host,
          port: 1421,
        }
      : undefined,
    watch: {
      // Rust se recompila solo; no hace falta vigilarlo desde Vite.
      ignored: ['**/src-tauri/**'],
    },
  },

  envPrefix: ['VITE_', 'TAURI_ENV_*'],

  build: {
    // El webview de Tauri en Windows es Chromium (WebView2).
    target: process.env.TAURI_ENV_PLATFORM === 'windows' ? 'chrome105' : 'safari13',
    minify: !process.env.TAURI_ENV_DEBUG,
    sourcemap: !!process.env.TAURI_ENV_DEBUG,
  },
});
