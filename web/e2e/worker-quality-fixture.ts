import { test as base, expect } from "@playwright/test";
import { mkdir, mkdtemp } from "node:fs/promises";
import { resolve } from "node:path";
import { fileURLToPath } from "node:url";
import { build, preview } from "vite";

const webRoot = fileURLToPath(new URL("../", import.meta.url));

/** Owns a test-only production benchmark bundle without changing web/dist. */
export const test = base.extend<{}, { workerQualityUrl: string }>({
  workerQualityUrl: [
    async ({}, use) => {
      const targetRoot = resolve(webRoot, "../target/tesla-worker-browser");
      await mkdir(targetRoot, { recursive: true });
      const outputRoot = await mkdtemp(resolve(targetRoot, "fixture-"));
      const configFile = resolve(webRoot, "vite.config.ts");
      await build({
        configFile,
        base: "/",
        cacheDir: resolve(outputRoot, "cache"),
        build: {
          outDir: resolve(outputRoot, "bundle"),
          emptyOutDir: true,
          rollupOptions: {
            input: resolve(webRoot, "benchmarks/tesla-valve/index.html"),
          },
        },
      });
      const server = await preview({
        configFile,
        base: "/",
        build: { outDir: resolve(outputRoot, "bundle") },
        preview: { host: "127.0.0.1", port: 0, strictPort: true },
      });
      try {
        const address = server.httpServer.address();
        if (address === null || typeof address === "string")
          throw new Error("Worker quality fixture did not expose a local port");
        await use(
          `http://127.0.0.1:${address.port}/benchmarks/tesla-valve/index.html`,
        );
      } finally {
        await new Promise<void>((resolveClose, reject) => {
          server.httpServer.close((error) =>
            error === undefined ? resolveClose() : reject(error),
          );
          server.httpServer.closeAllConnections();
        });
      }
    },
    { scope: "worker", timeout: 120_000 },
  ],
});

export { expect };
