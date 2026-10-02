import { spawnSync } from "node:child_process";
import { readFile, writeFile } from "node:fs/promises";
import { arch, cpus, platform, release, totalmem } from "node:os";
import { createHash } from "node:crypto";
import { resolve } from "node:path";
import type { ProducerIdentity } from "./contracts";
import { canonicalJson } from "./model";

export function command(command: string, args: readonly string[], cwd: string): string {
  const result = spawnSync(command, [...args], { cwd, encoding: "utf8" });
  if (result.error !== undefined) throw result.error;
  if (result.status !== 0) throw new Error(`${command} ${args.join(" ")} failed: ${result.stderr}`);
  return result.stdout.trim();
}

export function sha256(value: string | Uint8Array): string {
  return createHash("sha256").update(value).digest("hex");
}

/** Actual bytes/manifest must stay fixed throughout one timed run. */
export function assertSameProducer(
  before: Pick<ProducerIdentity, "revision" | "sourceFilesSha256" | "wasmSha256" | "generatedBindingsSha256">,
  after: Pick<ProducerIdentity, "revision" | "sourceFilesSha256" | "wasmSha256" | "generatedBindingsSha256">,
): void {
  for (const key of ["revision", "sourceFilesSha256", "wasmSha256", "generatedBindingsSha256"] as const) {
    if (before[key] !== after[key]) throw new Error(`Producer changed during benchmark: ${key}; partial evidence is retained`);
  }
}

/** Tracks new/untracked source too, not just git's tracked diff. */
export async function producerIdentity(repoRoot: string): Promise<Omit<ProducerIdentity, "benchmarkBundleSha256"> & { readonly sourceManifest: readonly { readonly path: string; readonly sha256: string }[] }> {
  const sourcePaths = ["Cargo.toml", "Cargo.lock", ".cargo", "rust-toolchain.toml", "crates/liquidfun", "crates/liquidfun-wasm",
    "web", "scripts"];
  const listed = command("git", ["ls-files", "--cached", "--others", "--exclude-standard", "--", ...sourcePaths], repoRoot);
  const files = [...new Set(listed.split("\n").filter(Boolean))].sort();
  const entries = await Promise.all(files.map(async (path) => ({ path, sha256: sha256(await readFile(resolve(repoRoot, path))) })));
  const diff = command("git", ["diff", "--binary", "HEAD", "--", ...sourcePaths], repoRoot);
  const untracked = command("git", ["ls-files", "--others", "--exclude-standard", "--", ...sourcePaths], repoRoot).split("\n").filter(Boolean);
  return {
    revision: command("git", ["rev-parse", "HEAD"], repoRoot),
    sourceDiffSha256: sha256(canonicalJson({ trackedDiff: diff, untracked: entries.filter((entry) => untracked.includes(entry.path)) })),
    sourceFilesSha256: sha256(canonicalJson(entries)), sourcePaths, sourceManifest: entries,
    wasmSha256: sha256(await readFile(resolve(repoRoot, "web/src/generated/liquidfun-wasm/liquidfun_wasm_bg.wasm"))),
    generatedBindingsSha256: sha256(await readFile(resolve(repoRoot, "web/src/generated/liquidfun-wasm/liquidfun_wasm.js"))),
    rustToolchain: command("rustc", ["--version"], repoRoot), executionBackend: "direct",
    buildProfile: "release", wasmTarget: "wasm32-unknown-unknown",
    wasmPackVersion: command("wasm-pack", ["--version"], repoRoot),
    rustflags: process.env.RUSTFLAGS ?? "unset", cargoEncodedRustflags: process.env.CARGO_ENCODED_RUSTFLAGS ?? "unset",
  };
}

export function hostIdentity(repoRoot: string) {
  const processors = cpus();
  return { platform: platform(), architecture: arch(), osRelease: release(), cpuModel: processors[0]?.model ?? "unavailable",
    logicalCpuCount: processors.length, totalMemoryBytes: totalmem(), bun: command("bun", ["--version"], repoRoot),
    nodeCompatibility: process.version };
}

export async function rebuildWasm(repoRoot: string, logDestination: string): Promise<void> {
  const result = spawnSync("bun", ["scripts/web-build.ts", "wasm"], { cwd: repoRoot, stdio: "inherit",
    env: { ...process.env, LIQUIDFUN_REUSE_WASM_PACKAGE: "false" } });
  await writeFile(logDestination, await readFile(resolve(repoRoot, "target/web-build/web-build.log")), { flag: "wx" });
  if (result.error !== undefined) throw result.error;
  if (result.status !== 0) throw new Error(`Production WASM rebuild failed (${result.status})`);
}
