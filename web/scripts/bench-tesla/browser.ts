import { readFile, readdir } from "node:fs/promises";
import { createHash } from "node:crypto";
import { resolve, relative } from "node:path";
import { chromium, type Browser, type BrowserServer } from "@playwright/test";
import { build, preview } from "vite";
import type { BenchmarkWorkload } from "./model";
import { canonicalJson } from "./model";
import type { CaseReplicate } from "./contracts";

export type BrowserProbe = {
  readonly chromiumVersion: string;
  readonly userAgent: string;
  readonly hardwareConcurrency: number;
  readonly benchmarkBundleSha256: string;
  readonly builtWasmSha256: string;
  readonly chromiumExecutable: string;
  readonly chromiumExecutableSha256: string;
  runCase(workload: BenchmarkWorkload, caseId: string, repetition: number): Promise<CaseReplicate>;
  close(): Promise<void>;
};

async function bundleHash(directory: string): Promise<string> {
  const entries: { path: string; sha256: string }[] = [];
  async function walk(current: string): Promise<void> {
    for (const entry of await readdir(current, { withFileTypes: true })) {
      const full = resolve(current, entry.name);
      if (entry.isDirectory()) { await walk(full); continue; }
      if (!entry.isFile()) throw new Error(`Unexpected benchmark bundle entry: ${full}`);
      entries.push({ path: relative(directory, full), sha256: createHash("sha256").update(await readFile(full)).digest("hex") });
    }
  }
  await walk(directory);
  entries.sort((first, second) => first.path.localeCompare(second.path));
  return createHash("sha256").update(canonicalJson(entries)).digest("hex");
}

/** Builds and owns a separate real-browser bundle; normal web/dist is untouched. */
export async function openBenchmarkBrowser(webRoot: string, buildRoot: string, workload: BenchmarkWorkload, channel: "chromium" | "headless-shell"): Promise<BrowserProbe> {
  await build({ configFile: resolve(webRoot, "vite.config.ts"), base: "/",
    build: { outDir: buildRoot, emptyOutDir: true,
      rollupOptions: { input: resolve(webRoot, "benchmarks/tesla-valve/index.html") } } });
  const benchmarkBundleSha256 = await bundleHash(buildRoot);
  const wasmAssets = (await readdir(resolve(buildRoot, "assets"))).filter((name) => name.endsWith(".wasm"));
  if (wasmAssets.length !== 1) throw new Error("Benchmark bundle must contain one actual WASM producer");
  const builtWasmSha256 = createHash("sha256").update(await readFile(resolve(buildRoot, "assets", wasmAssets[0]!))).digest("hex");
  const server = await preview({ configFile: resolve(webRoot, "vite.config.ts"), base: "/",
    build: { outDir: buildRoot }, preview: { host: "127.0.0.1", port: 0, strictPort: true } });
  let maybeBrowser: Browser | undefined;
  let maybeBrowserServer: BrowserServer | undefined;
  const closeServer = () => new Promise<void>((resolveClose, reject) => {
    server.httpServer.close((error) => error === undefined ? resolveClose() : reject(error));
  });
  try {
    const address = server.httpServer.address();
    if (address === null || typeof address === "string") throw new Error("Benchmark preview did not expose a local port");
    const browserServer = await chromium.launchServer({ headless: true, host: "127.0.0.1", port: 0,
      ...(channel === "chromium" ? { channel: "chromium" } : {}) });
    maybeBrowserServer = browserServer;
    const browser = await chromium.connect(browserServer.wsEndpoint());
    maybeBrowser = browser;
    const context = await browser.newContext({ viewport: workload.viewport, deviceScaleFactor: workload.devicePixelRatio });
    const page = await context.newPage();
    await page.goto(`http://127.0.0.1:${address.port}/benchmarks/tesla-valve/index.html`);
    await page.waitForFunction(() => window.teslaBenchmark !== undefined);
    const navigatorIdentity = await page.evaluate(() => ({ userAgent: navigator.userAgent, hardwareConcurrency: navigator.hardwareConcurrency }));
    const chromiumExecutable = browserServer.process().spawnfile;
    const chromiumExecutableSha256 = createHash("sha256").update(await readFile(chromiumExecutable)).digest("hex");
    return {
      chromiumVersion: browser.version(), ...navigatorIdentity, benchmarkBundleSha256, builtWasmSha256,
      chromiumExecutable, chromiumExecutableSha256,
      runCase: async (settings, caseId, repetition) => await page.evaluate(
        ({ settings, caseId, repetition }) => window.teslaBenchmark.runCase(settings, caseId, repetition),
        { settings, caseId, repetition }),
      close: async () => { await browser.close(); await browserServer.close(); await closeServer(); },
    };
  } catch (error) {
    if (maybeBrowser !== undefined) await maybeBrowser.close();
    if (maybeBrowserServer !== undefined) await maybeBrowserServer.close();
    await closeServer();
    throw error;
  }
}
