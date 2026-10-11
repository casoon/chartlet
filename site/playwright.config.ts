import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './test',
  // The showcase holds every example, and axe checks all of it: more than 30 seconds on a slow runner.
  timeout: 90_000,
  use: {
    baseURL: 'http://127.0.0.1:4321/chartlet/',
  },
  webServer: {
    command: 'node test/server.mjs',
    url: 'http://127.0.0.1:4321/chartlet/',
    reuseExistingServer: !process.env.CI,
  },
});
