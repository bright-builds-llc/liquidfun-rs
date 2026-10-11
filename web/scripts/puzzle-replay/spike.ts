/**
 * Phase 37 native-vs-wasm determinism spike: per-step frame hash lines and
 * their fail-closed comparison.
 */

import { hex16 } from "./hash";

/** Scenes stepped by the spike, matching the native test. */
export const SPIKE_SCENES = [
  "dam-break",
  "water-wheel",
  "theo-jansen",
] as const;
/** Fixed steps per spike scene. */
export const SPIKE_STEPS = 300;

const HASH_PATTERN = /^[0-9a-f]{16}$/;
const STEP_PATTERN = /^[1-9][0-9]*$/;

/** One comparison row per scene. */
export type SpikeComparison = {
  readonly scene: string;
  readonly identical: boolean;
  readonly steps: number;
  readonly maybeFirstDivergingStep?: number;
};

/** Formats one `SPIKE <scene> <step> <hash16>` line. */
export function spikeLine(scene: string, step: number, hash: bigint): string {
  return `SPIKE ${scene} ${step} ${hex16(hash)}`;
}

/**
 * Parses spike lines into per-scene hash lists in first-seen scene order.
 * Fails closed on any malformed field or a step that is not the next one.
 */
export function parseSpikeLines(text: string): Map<string, string[]> {
  const scenes = new Map<string, string[]>();
  text.split("\n").forEach((line, index) => {
    if (line === "") {
      return;
    }
    const malformed = (reason: string): Error =>
      new Error(`Malformed spike line ${index + 1}: ${reason}: ${line}`);
    const fields = line.split(" ");
    const [tag, scene, step, hash] = fields;
    if (
      fields.length !== 4 ||
      tag !== "SPIKE" ||
      scene === undefined ||
      scene === "" ||
      step === undefined ||
      hash === undefined
    ) {
      throw malformed("expected SPIKE <scene> <step> <hash16>");
    }
    if (!STEP_PATTERN.test(step) || !HASH_PATTERN.test(hash)) {
      throw malformed("invalid step or hash");
    }
    const hashes = scenes.get(scene) ?? [];
    if (Number(step) !== hashes.length + 1) {
      throw malformed(`expected step ${hashes.length + 1}`);
    }
    hashes.push(hash);
    scenes.set(scene, hashes);
  });
  return scenes;
}

/**
 * Compares two parsed spike runs scene by scene in `left` order. A scene
 * missing from `right` or with a different step count is an error.
 */
export function compareSpikes(
  left: Map<string, string[]>,
  right: Map<string, string[]>,
): SpikeComparison[] {
  return [...left].map(([scene, leftHashes]) => {
    const rightHashes = right.get(scene);
    if (rightHashes === undefined) {
      throw new Error(`Spike scene ${scene} is missing from the right run`);
    }
    if (rightHashes.length !== leftHashes.length) {
      throw new Error(
        `Spike scene ${scene} has ${leftHashes.length} vs ${rightHashes.length} steps`,
      );
    }
    const index = leftHashes.findIndex((hash, at) => hash !== rightHashes[at]);
    const steps = leftHashes.length;
    return index === -1
      ? { scene, identical: true, steps }
      : { scene, identical: false, steps, maybeFirstDivergingStep: index + 1 };
  });
}
