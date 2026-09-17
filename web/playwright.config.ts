import { defineConfig, devices } from "@playwright/test";
import { resolve } from "node:path";

const maybeAttemptDirectory =
  process.env.PHASE16_CLOSURE_ATTEMPT_DIR;

if (maybeAttemptDirectory === undefined) {
  throw new Error("PHASE16_CLOSURE_ATTEMPT_DIR is required");
}

export default defineConfig({
  testDir: "./e2e",
  timeout: 30_000,
  retries: 0,
  outputDir: resolve(maybeAttemptDirectory, "playwright-output"),
  preserveOutput: "always",
  reporter: [
    ["line"],
    [
      "json",
      {
        outputFile: resolve(
          maybeAttemptDirectory,
          "playwright-report.json",
        ),
      },
    ],
  ],
  use: {
    baseURL: "http://127.0.0.1:4173",
    trace: "retain-on-failure",
    screenshot: "only-on-failure",
  },
  projects: [
    {
      name: "chromium",
      use: { ...devices["Desktop Chrome"] },
    },
  ],
  webServer: {
    command: "bun run preview -- --strictPort",
    url: "http://127.0.0.1:4173",
    reuseExistingServer: !process.env.CI,
    timeout: 30_000,
  },
});
