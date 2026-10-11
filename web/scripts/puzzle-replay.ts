/**
 * Phase 37 puzzle replay entry point (run from `web/`).
 *
 * - `spike`: prints per-step wasm frame hashes for the spike scenes.
 * - `compare <left> <right>`: compares two spike hash files per scene.
 */

import { readFile } from "node:fs/promises";

import { fnv1a64Lanes } from "./puzzle-replay/hash";
import {
  compareSpikes,
  parseSpikeLines,
  SPIKE_SCENES,
  SPIKE_STEPS,
  spikeLine,
} from "./puzzle-replay/spike";
import { loadGeneratedWasm } from "./puzzle-replay/wasm";

const USAGE =
  "usage: bun scripts/puzzle-replay.ts spike | compare <leftPath> <rightPath>";

async function runSpike(): Promise<void> {
  const generated = await loadGeneratedWasm();
  for (const scene of SPIKE_SCENES) {
    const session = new generated.ProofSession(scene);
    try {
      for (let step = 1; step <= SPIKE_STEPS; step += 1) {
        session.advance(1);
        const frame = session.captureFrame();
        try {
          const hash = fnv1a64Lanes([
            frame.particlePositions(),
            frame.rigidSegments(),
            frame.rigidCircles(),
          ]);
          console.log(spikeLine(scene, step, hash));
        } finally {
          frame.free();
        }
      }
    } finally {
      session.free();
    }
  }
}

async function runCompare(leftPath: string, rightPath: string): Promise<void> {
  const [left, right] = await Promise.all([
    readFile(leftPath, "utf8"),
    readFile(rightPath, "utf8"),
  ]);
  const rows = compareSpikes(parseSpikeLines(left), parseSpikeLines(right));
  for (const row of rows) {
    console.log(
      row.maybeFirstDivergingStep === undefined
        ? `COMPARE ${row.scene} identical ${row.steps}`
        : `COMPARE ${row.scene} diverges-at ${row.maybeFirstDivergingStep} of ${row.steps}`,
    );
  }
}

async function main(args: readonly string[]): Promise<void> {
  const [mode, leftPath, rightPath] = args;
  if (mode === "spike" && args.length === 1) {
    await runSpike();
    return;
  }
  if (
    mode === "compare" &&
    args.length === 3 &&
    leftPath !== undefined &&
    rightPath !== undefined
  ) {
    await runCompare(leftPath, rightPath);
    return;
  }
  console.error(USAGE);
  process.exit(2);
}

main(process.argv.slice(2)).catch((error: unknown) => {
  console.error(error instanceof Error ? error.message : String(error));
  process.exit(1);
});
