import { loadSceneSession } from "../../src/physics/loader";
import { createSceneSession } from "../../src/physics/session";
import { cancelPendingFrame, createFrameClock, scheduleFrame, type FrameLoopDeps } from "../../src/player/frame-loop";
import { observeFrame } from "../../src/player/observe";
import type { PlayerView } from "../../src/player/view";
import type { FpsTick } from "../../src/components/fps-meter";
import type { BenchmarkCase } from "../../scripts/bench-tesla/model";
import type { LiveCadence, LivePaint } from "../../scripts/bench-tesla/contracts";
import { fingerprint } from "../../scripts/bench-tesla/fingerprint";
import { capture, present, rendererIdentity, warm, type BenchmarkSurface } from "./benchmark";

/** Uses the production RAF/clock/session/painter, with product chrome excluded explicitly. */
export async function measureLiveCadence(surface: BenchmarkSurface, scenario: BenchmarkCase): Promise<LiveCadence> {
  const generated = await loadSceneSession("tesla-valve");
  const owner = createSceneSession(generated);
  const clock = createFrameClock();
  let maybeTimer: ReturnType<typeof setInterval> | undefined;
  let maybeTimeout: ReturnType<typeof setTimeout> | undefined;
  try {
  generated.applyControl("flow-rate", String(surface.workload.rate));
  generated.applyControl("flow-direction", scenario.direction);
  warm(generated, scenario.warmupSteps);
  let finalFrame = capture(generated);
  const start = await fingerprint(finalFrame);
  clock.maybeCamera = surface.camera;
  clock.viewportWidth = surface.workload.viewport.width;
  clock.viewportHeight = surface.workload.viewport.height;
  clock.worldBounds = surface.camera.bounds;
  clock.maybePreviousFrame = finalFrame;
  let view: PlayerView = { kind: "playing", frame: observeFrame(finalFrame, undefined, 0) };
  let ticks: readonly FpsTick[] = [];
  const paints: LivePaint[] = [];
  const timerTaskLatenessMs: number[] = [];
  const counts = [finalFrame.particleCount];
  let elapsedMs = 0;
  let startedAt = 0;
    await new Promise<void>((resolve, reject) => {
      const deps: FrameLoopDeps = {
        view: () => view, fail: reject, maybeSession: () => owner,
        route: () => ({ kind: "scene", id: "tesla-valve" }), startScene: () => reject(new Error("Benchmark cannot change scenes")),
        drawSceneFrame: (_context, frame) => { finalFrame = frame; present(surface, frame); },
        setMaybeDebugFrame: () => undefined, setStepsThisFrame: () => undefined,
        setMaybePixelsPerMeter: () => undefined, setView: (next) => { view = next; },
        setFpsTicks: (update) => {
          ticks = update(ticks);
          const tick = ticks[ticks.length - 1];
          if (tick === undefined) { reject(new Error("Production frame loop did not publish cadence")); return; }
          const now = performance.now();
          paints.push({ presentedAtMs: now, rafTimestampMs: tick.timeMs,
            simSteps: tick.simSteps, stepIndex: finalFrame.stepIndex, particleCount: finalFrame.particleCount,
            bodyContactCount: finalFrame.bodyContactCount, workerComputeMs: null, workerCaptureMs: null, workerRoundTripMs: null });
          counts.push(finalFrame.particleCount);
          if (now - startedAt >= surface.workload.liveDurationMs) {
            elapsedMs = now - startedAt;
            resolve();
          }
        },
      };
      startedAt = performance.now();
      let expectedTimerAt = startedAt + 16;
      maybeTimer = setInterval(() => {
        const now = performance.now();
        timerTaskLatenessMs.push(Math.max(0, now - expectedTimerAt));
        expectedTimerAt = now + 16;
      }, 16);
      maybeTimeout = setTimeout(() => reject(new Error("Live benchmark did not finish within its bounded timeout")), surface.workload.liveDurationMs + 10_000);
      scheduleFrame(clock, surface.context, deps);
    });
    cancelPendingFrame(clock);
    if (maybeTimer !== undefined) clearInterval(maybeTimer);
    if (maybeTimeout !== undefined) clearTimeout(maybeTimeout);
    const end = await fingerprint(finalFrame);
    const renderer = rendererIdentity(surface, finalFrame);
    return { driver: "production-frame-loop", executionBackend: "direct", renderer, start, end, elapsedMs, startedAtMs: startedAt,
      presentedFps: paints.length * 1000 / elapsedMs,
      simulationStepsPerSecond: paints.reduce((total, paint) => total + paint.simSteps, 0) * 1000 / elapsedMs,
      paints, timerTaskLatenessMs, responsivenessProxy: "16ms-main-thread-timer-lateness",
      includesProductChrome: false, minimumParticleCount: Math.min(...counts), maximumParticleCount: Math.max(...counts) };
  } finally {
    cancelPendingFrame(clock);
    if (maybeTimer !== undefined) clearInterval(maybeTimer);
    if (maybeTimeout !== undefined) clearTimeout(maybeTimeout);
    owner.dispose();
  }
}
