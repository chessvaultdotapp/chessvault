import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./tests",
  forbidOnly: !!process.env.CI,
  use: {
    baseURL: "http://127.0.0.1:4321/chessvault/",
    trace: "retain-on-failure",
  },
  projects: [
    { name: "desktop", use: { browserName: "chromium", viewport: { width: 1280, height: 800 } } },
    {
      name: "mobile",
      use: { browserName: "chromium", viewport: { width: 390, height: 844 }, isMobile: true, hasTouch: true },
    },
  ],
  webServer: {
    // Keep Astro in the foreground even in agent environments; Playwright owns its lifecycle.
    command:
      "pnpm build --base /chessvault/ && pnpm preview --base /chessvault/ --host 127.0.0.1 --port 4321 --ignore-lock",
    url: "http://127.0.0.1:4321/chessvault/",
    reuseExistingServer: false,
    timeout: 120_000,
  },
});
