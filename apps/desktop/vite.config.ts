import { defineConfig } from 'vitest/config';
import { svelte } from '@sveltejs/vite-plugin-svelte';

export default defineConfig({
  plugins: [svelte()],
  clearScreen: false,
  test: {
    environment: 'jsdom',
  },
  server: {
    strictPort: true,
    host: '127.0.0.1',
    port: 1420,
  },
});
