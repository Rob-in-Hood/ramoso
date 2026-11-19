import { defineConfig } from "@playwright/test";

const port = 4173;

export default defineConfig({
  testDir: "e2e",
  use: { baseURL: `http://localhost:${port}` },

  // The production build in e2e mode, where Rust calls are mocked.
  webServer: {
    command: `pnpm build:e2e && pnpm vite preview --outDir dist-e2e --port ${port} --strictPort`,
    port,
    reuseExistingServer: false,
  },
});
