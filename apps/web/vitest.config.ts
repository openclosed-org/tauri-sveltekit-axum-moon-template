import { resolve } from 'node:path';
import { svelte } from '@sveltejs/vite-plugin-svelte';
import { defineConfig } from 'vitest/config';

export default defineConfig({
  plugins: [svelte({ hot: !process.env.VITEST })],
  test: {
    environment: 'happy-dom',
    include: ['tests/component/**/*.test.ts'],
    globals: true,
    setupFiles: [],
    css: false,
    testTimeout: 15000,
  },
  resolve: {
    conditions: ['browser'],
    alias: {
      $lib: resolve(__dirname, 'src/lib'),
      $app: resolve(__dirname, 'tests/__mocks__/app'),
    },
  },
});
