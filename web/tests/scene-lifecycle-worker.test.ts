import { afterEach, describe, expect, it, vi } from "vitest";
import { createWorkerSession } from "../src/physics/worker-session";
import { bindLifecycle } from "../src/player/scene-lifecycle";
import { deferred, workerLoopFixture } from "./worker-loop-fixture";
import type { LiveSceneSession } from "../src/physics/live-session";

vi.mock("../src/physics/worker-session", () => ({
  createWorkerSession: vi.fn(),
}));
vi.mock("../src/physics/loader", () => ({ loadSceneSession: vi.fn() }));
afterEach(() => {
  vi.unstubAllGlobals();
  vi.clearAllMocks();
});

function lifecycleFixture() {
  const fixture = workerLoopFixture(),
    session = fixture.runtime;
  session.maybeSession = undefined;
  session.maybeCanvas = { dataset: {} } as HTMLCanvasElement;
  session.setRenderedParticleDraft = vi.fn();
  session.setMaxRenderedParticles = vi.fn();
  session.refreshCamera = vi.fn();
  session.drawSceneFrame = vi.fn();
  session.setMaybePixelsPerMeter = vi.fn();
  bindLifecycle(session);
  return { ...fixture, session };
}

describe("Tesla worker lifecycle completion ownership", () => {
  it("aborts initializing ownership and disposes a late ready owner without reopening an abandoned route", async () => {
    // Arrange
    const fixture = lifecycleFixture(),
      initialized = deferred<LiveSceneSession>();
    vi.mocked(createWorkerSession).mockReturnValueOnce(initialized.promise);
    const started = fixture.session.beginScene("tesla-valve");
    const signal = vi.mocked(createWorkerSession).mock.calls[0]?.[2];

    // Act
    fixture.session.abandonScene();
    initialized.resolve(fixture.owner);
    await started;

    // Assert
    expect(signal?.aborted).toBe(true);
    expect(fixture.session.maybeSession).toBeUndefined();
    expect(fixture.view().kind).toBe("fallback");
    expect(fixture.owner.dispose).toHaveBeenCalled();
    expect(fixture.owner.nextFrame).not.toHaveBeenCalled();
  });

  it("does not publish a late preset result over a newer lifecycle generation", async () => {
    // Arrange
    const fixture = lifecycleFixture(),
      preset = deferred<boolean>();
    fixture.session.constructionValues = { gravity: "5" };
    vi.mocked(createWorkerSession).mockResolvedValueOnce(fixture.owner);
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(preset.promise);
    const started = fixture.session.beginScene("tesla-valve");
    await Promise.resolve();
    await Promise.resolve();

    // Act
    fixture.session.abandonScene();
    preset.resolve(true);
    await started;

    // Assert
    expect(fixture.session.maybeSession).toBeUndefined();
    expect(fixture.view().kind).toBe("fallback");
    expect(fixture.owner.nextFrame).not.toHaveBeenCalled();
  });

  it("shows an actual initialization failure instead of presenting a ready or paused stale snapshot", async () => {
    // Arrange
    const fixture = lifecycleFixture();
    vi.mocked(createWorkerSession).mockRejectedValueOnce(
      new Error("Worker loader failed"),
    );

    // Act
    await fixture.session.beginScene("tesla-valve");

    // Assert
    expect(fixture.view().kind).toBe("failure");
    expect(fixture.session.maybeSession).toBeUndefined();
    expect(fixture.session.drawSceneFrame).not.toHaveBeenCalled();
  });
});
