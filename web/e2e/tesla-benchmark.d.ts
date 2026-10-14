import type { TeslaBenchmarkApi } from "../benchmarks/tesla-valve/benchmark";

declare global {
  interface Window {
    /** Installed by benchmarks/tesla-valve/benchmark.ts on the worker-quality fixture page only. */
    teslaBenchmark?: TeslaBenchmarkApi;
  }
}

export {};
