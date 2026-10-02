import { loadSceneSession } from "../../src/physics/loader";
import { createSceneSession } from "../../src/physics/session";
import { createWorkerSession } from "../../src/physics/worker-session";
import type { RenderFrame } from "../../src/physics/frame";
import { fingerprint } from "../../scripts/bench-tesla/fingerprint";
import { capture } from "./benchmark";

function contains(x: number, y: number, points: readonly number[][]): boolean {
  let inside = false;
  for (let index = 0; index < points.length; index += 1) {
    const a = points[index]!,
      b = points[(index + 1) % points.length]!;
    if (
      a[1]! > y !== b[1]! > y &&
      x < a[0]! + ((y - a[1]!) * (b[0]! - a[0]!)) / (b[1]! - a[1]!)
    )
      inside = !inside;
  }
  return inside;
}

function boundaries(frame: RenderFrame): number[][][] {
  const paths: number[][][] = [];
  let current: number[][] = [];
  for (let index = 0; index < frame.rigidSegments.length; index += 4) {
    const start = [
      frame.rigidSegments[index]!,
      frame.rigidSegments[index + 1]!,
    ];
    const end = [
      frame.rigidSegments[index + 2]!,
      frame.rigidSegments[index + 3]!,
    ];
    const previous = current.at(-1);
    if (
      previous !== undefined &&
      (previous[0] !== start[0] || previous[1] !== start[1])
    ) {
      paths.push(current);
      current = [];
    }
    if (!current.length) current.push(start);
    current.push(end);
  }
  if (current.length) paths.push(current);
  if (paths.length !== 6)
    throw new Error(
      "Quality probe requires actual two outer paths/four islands",
    );
  return paths;
}

function assertContained(frame: RenderFrame, paths: number[][][]): void {
  const outer = [...paths[0]!, ...paths[1]!.toReversed()];
  const inletY = (paths[0]![0]![1]! + paths[1]![0]![1]!) * 0.5;
  for (let index = 0; index < frame.particlePositions.length; index += 2) {
    const x = frame.particlePositions[index]!,
      y = frame.particlePositions[index + 1]!;
    if (!Number.isFinite(x) || !Number.isFinite(y) || y < 0.08)
      throw new Error(`Invalid particle/drain state at ${x},${y}`);
    if (y > inletY) continue;
    if (
      !contains(x, y, outer) ||
      paths.slice(2).some((island) => contains(x, y, island))
    )
      throw new Error(`Conduit/island escape at ${x},${y}`);
  }
}

/** Actual worker/direct WASM replay; performance benchmarking is a separate probe. */
export async function compareWorkerDirect(
  rate: number,
  direction: string,
  steps: number,
  overflowGravity = false,
) {
  if (
    ![1440, 2880].includes(rate) ||
    !["1", "-1"].includes(direction) ||
    !Number.isInteger(steps) ||
    steps < 1 ||
    steps > 424
  )
    throw new Error("Invalid bounded worker quality case");
  const generated = await loadSceneSession("tesla-valve"),
    direct = createSceneSession(generated);
  let maybeWorker: Awaited<ReturnType<typeof createWorkerSession>> | undefined;
  try {
    const worker = await createWorkerSession(1);
    maybeWorker = worker;
    direct.applyControl("flow-rate", String(rate));
    direct.applyControl("flow-direction", direction);
    await worker.applyControl("flow-rate", String(rate));
    await worker.applyControl("flow-direction", direction);
    if (overflowGravity) {
      direct.setGravity(Number.MAX_VALUE, 0);
      await worker.setGravity(Number.MAX_VALUE, 0);
    }
    const paths = boundaries(capture(generated));
    let completed = 0;
    while (completed < steps) {
      const count = Math.min(4, steps - completed);
      generated.advance(count);
      await worker.advanceOnly(count);
      completed += count;
      const first = capture(generated),
        second = await worker.captureFrame();
      assertContained(first, paths);
      assertContained(second, paths);
    }
    const first = await fingerprint(capture(generated)),
      second = await fingerprint(await worker.captureFrame());
    return {
      direct: first,
      worker: second,
      steps,
      rate,
      direction,
      overflowGravity,
    };
  } finally {
    direct.dispose();
    maybeWorker?.dispose();
  }
}
