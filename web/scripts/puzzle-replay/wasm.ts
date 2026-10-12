/** Loads the locally generated (gitignored) wasm package for Bun scripts. */

import { access, readFile } from "node:fs/promises";
import { dirname, resolve } from "node:path";
import { fileURLToPath, pathToFileURL } from "node:url";

import type { GeneratedProofSession } from "../../src/physics/session";

/** Generated `PuzzleLevelInfo` surface (D-24). */
export interface RawPuzzleLevelInfo {
  levelId(): string;
  title(): string;
  targetCount(): number;
  fullCount(): number;
  settleSteps(): number;
  stepLimit(): number;
  cupInterior(): Float32Array;
  buildArea(): Int32Array;
  gridMeters(): number;
  toolKinds(): Uint8Array;
  toolCounts(): Uint32Array;
  free(): void;
}

/** A generated session built by `ProofSession.newPuzzle`. */
export interface GeneratedPuzzleSession extends GeneratedProofSession {
  levelInfo(): RawPuzzleLevelInfo;
}

/** Generated module surface used by the puzzle replay scripts. */
export type GeneratedPuzzleWasm = {
  default: (input?: { module_or_path?: BufferSource }) => Promise<unknown>;
  ProofSession: {
    new (sceneId: string): GeneratedProofSession;
    newPuzzle(levelId: string, kinds: Uint8Array, poses: Int32Array): GeneratedPuzzleSession;
  };
};

/** Repository root, resolved from this file. */
export const repoRoot = resolve(
  dirname(fileURLToPath(import.meta.url)),
  "../../..",
);

/** Imports and initializes `web/src/generated/liquidfun-wasm`. */
export async function loadGeneratedWasm(): Promise<GeneratedPuzzleWasm> {
  const directory = resolve(repoRoot, "web/src/generated/liquidfun-wasm");
  const jsPath = resolve(directory, "liquidfun_wasm.js");
  const wasmPath = resolve(directory, "liquidfun_wasm_bg.wasm");
  try {
    await access(jsPath);
    const wasmBytes = await readFile(wasmPath);
    const generated = (await import(
      pathToFileURL(jsPath).href
    )) as GeneratedPuzzleWasm;
    await generated.default({ module_or_path: wasmBytes });
    return generated;
  } catch (error) {
    const detail = error instanceof Error ? error.message : "unknown error";
    throw new Error(
      `Could not load the generated WASM package (${detail}). Run \`bun scripts/web-build.ts wasm\` from the repository root first.`,
      { cause: error },
    );
  }
}
