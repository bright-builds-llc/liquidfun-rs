import { defineConfig, devices } from "@playwright/test";
import { resolve } from "node:path";

const maybeAttemptDirectory = process.env.PHASE16_CLOSURE_ATTEMPT_DIR;
const forensicConfig =
  maybeAttemptDirectory === undefined
    ? {
        reporter: [["line" as const]],
      }
    : {
        outputDir: resolve(maybeAttemptDirectory, "playwright-output"),
        reporter: [
          ["line" as const],
          [
            "json" as const,
            {
              outputFile: resolve(
                maybeAttemptDirectory,
                "playwright-report.json",
              ),
            },
          ],
        ],
      };

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  retries: 0,
  preserveOutput: "always",
  ...forensicConfig,
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      testIgnore: /puzzle-touch\.spec\.ts$/,
      use: { ...devices["Desktop Chrome"] },
    },
    {
      // Chromium only: `browser:install` installs chromium, so the phone
      // project uses a Chromium device descriptor with an iPhone-sized viewport.
      name: "touch",
      testMatch: /puzzle-touch\.spec\.ts$/,
      use: {
        ...devices["Pixel 7"],
        viewport: { width: 390, height: 844 },
        hasTouch: true,
        isMobile: true,
      },
    },
  ],
  webServer: {
    command: "bun run preview -- --strictPort",
    url: "http://127.0.0.1:4173/liquidfun-rs/",
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
