import type { compareWorkerDirect } from "../benchmarks/tesla-valve/worker-quality";

declare global {
  interface Window {
    /** Installed by benchmarks/tesla-valve/benchmark.ts on the worker-quality fixture page only. */
    teslaBenchmark: { readonly compareWorkerDirect: typeof compareWorkerDirect };
  }
}

export {};
