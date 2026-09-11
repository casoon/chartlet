import { defineConfig } from '@playwright/test';

export default defineConfig({
  testDir: './test',
  use: {
    baseURL: 'http://127.0.0.1:4321/chartlet/',
  },
  webServer: {
    command: 'node test/server.mjs',
    url: 'http://127.0.0.1:4321/chartlet/',
    reuseExistingServer: !process.env.CI,
  },
});
