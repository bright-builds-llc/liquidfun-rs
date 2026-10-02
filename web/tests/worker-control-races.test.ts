import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { cancelPendingFrame, scheduleFrame } from "../src/player/frame-loop";
import { applyWorkerControl } from "../src/player/worker-controls";
import { handleVisibilityChange } from "../src/player/scene-lifecycle";
import { deferred, testFrame, workerLoopFixture } from "./worker-loop-fixture";

beforeEach(() =>
  vi.useFakeTimers({ toFake: ["setTimeout", "clearTimeout", "performance"] }),
);
afterEach(() => {
  vi.useRealTimers();
  vi.unstubAllGlobals();
});

async function flush() {
  await Promise.resolve();
  await Promise.resolve();
  await Promise.resolve();
}

function painted(fixture: ReturnType<typeof workerLoopFixture>) {
  return vi
    .mocked(fixture.deps.drawSceneFrame)
    .mock.calls.map((call) => call[1].rigidSegments[0]);
}

describe("accepted worker control publication", () => {
  it("settles a recreated paused control while hidden and redraws without an extra step on visibility", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      control = deferred<boolean>();
    fixture.pause();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);
    const mutation = applyWorkerControl(fixture.runtime, "gravity", "5");
    expect(fixture.view().kind).toBe("loading");
    vi.stubGlobal("document", { hidden: true });
    handleVisibilityChange(fixture.runtime);

    // Act
    control.resolve(true);
    await mutation;
    expect(painted(fixture)).toEqual([]);
    expect(fixture.view().kind).toBe("paused");
    vi.stubGlobal("document", { hidden: false });
    handleVisibilityChange(fixture.runtime);

    // Assert
    expect(painted(fixture)).toEqual([1]);
    expect(fixture.view().kind).toBe("paused");
    expect(fixture.owner.nextFrame).toHaveBeenCalledExactlyOnceWith(1);
    expect(fixture.requests).toHaveLength(0);
  });

  it("holds a paused control completed while hidden and paints it on visibility without stepping", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      control = deferred<boolean>();
    fixture.pause();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);
    const mutation = applyWorkerControl(
      fixture.runtime,
      "flow-direction",
      "-1",
    );
    vi.stubGlobal("document", { hidden: true });
    handleVisibilityChange(fixture.runtime);

    // Act
    fixture.setSide(-1);
    control.resolve(false);
    await mutation;
    expect(painted(fixture)).toEqual([]);
    vi.stubGlobal("document", { hidden: false });
    handleVisibilityChange(fixture.runtime);

    // Assert
    expect(painted(fixture)).toEqual([-1]);
    expect(fixture.view().kind).toBe("paused");
    expect(fixture.owner.nextFrame).not.toHaveBeenCalled();
    expect(fixture.requests).toHaveLength(0);
  });

  it("does not repaint a hidden completed control after its owner is abandoned", async () => {
    // Arrange
    const fixture = workerLoopFixture();
    fixture.pause();
    vi.stubGlobal("document", { hidden: true });
    fixture.setSide(-1);
    await applyWorkerControl(fixture.runtime, "flow-direction", "-1");

    // Act
    fixture.runtime.generation += 1;
    fixture.runtime.maybeSession = undefined;
    fixture.abandon();
    vi.stubGlobal("document", { hidden: false });
    handleVisibilityChange(fixture.runtime);

    // Assert
    expect(painted(fixture)).toEqual([]);
    expect(fixture.requests).toHaveLength(0);
  });

  it("paints only the latest mutation completed while hidden", async () => {
    // Arrange
    const fixture = workerLoopFixture();
    fixture.pause();
    vi.stubGlobal("document", { hidden: true });
    fixture.setSide(1);
    await applyWorkerControl(fixture.runtime, "flow-direction", "1");
    fixture.setSide(-1);
    await applyWorkerControl(fixture.runtime, "flow-direction", "-1");

    // Act
    vi.stubGlobal("document", { hidden: false });
    handleVisibilityChange(fixture.runtime);

    // Assert
    expect(painted(fixture)).toEqual([-1]);
    expect(fixture.view().kind).toBe("paused");
    expect(fixture.requests).toHaveLength(0);
  });

  it("honors an explicit play while a paused control is pending", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      control = deferred<boolean>();
    fixture.pause();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);
    const mutation = applyWorkerControl(
      fixture.runtime,
      "flow-direction",
      "-1",
    );

    // Act
    const paused = fixture.view();
    if (paused.kind !== "paused") throw new Error("Expected paused test state");
    fixture.deps.setView({ kind: "playing", frame: paused.frame });
    fixture.setSide(-1);
    control.resolve(false);
    await mutation;

    // Assert
    expect(painted(fixture)).toEqual([-1]);
    expect(fixture.view().kind).toBe("playing");
    cancelPendingFrame(fixture.clock);
  });

  it("publishes the current controlled geometry into paused state after an accepted advance settles", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      control = deferred<boolean>();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);
    scheduleFrame(fixture.clock, fixture.context, fixture.deps);
    fixture.raf(0);
    vi.advanceTimersByTime(17);
    fixture.raf(17);
    const mutation = applyWorkerControl(
      fixture.runtime,
      "flow-direction",
      "-1",
    );

    // Act
    fixture.pause();
    cancelPendingFrame(fixture.clock);
    fixture.finish(0, 1, testFrame(1, 1));
    await flush();
    fixture.setSide(-1);
    control.resolve(false);
    await mutation;

    // Assert
    expect(painted(fixture)).toEqual([-1]);
    expect(fixture.runtime.constructionValues["flow-direction"]).toBe("-1");
    const settled = fixture.view();
    expect(settled.kind).toBe("paused");
    expect(settled.kind === "paused" ? settled.frame.stepIndex : -1).toBe(1);
    expect(fixture.requests).toHaveLength(1);
  });

  it("suppresses an older mutation snapshot after a newer control publishes", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      snapshot = deferred<ReturnType<typeof testFrame>>();
    vi.mocked(fixture.owner.captureFrame).mockReturnValueOnce(snapshot.promise);
    const older = applyWorkerControl(fixture.runtime, "flow-direction", "1");
    await flush();

    // Act
    fixture.setSide(-1);
    await applyWorkerControl(fixture.runtime, "flow-direction", "-1");
    snapshot.resolve(testFrame(0, 1));
    await older;

    // Assert
    expect(painted(fixture)).toEqual([-1]);
    expect(fixture.runtime.constructionValues["flow-direction"]).toBe("-1");
    cancelPendingFrame(fixture.clock);
  });

  it("keeps current mutation authority across a hidden and visible transition without resuming pause", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      control = deferred<boolean>();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);
    const mutation = applyWorkerControl(
      fixture.runtime,
      "flow-direction",
      "-1",
    );

    // Act
    vi.stubGlobal("document", { hidden: true });
    cancelPendingFrame(fixture.clock);
    fixture.pause();
    vi.stubGlobal("document", { hidden: false });
    fixture.setSide(-1);
    control.resolve(false);
    await mutation;

    // Assert
    expect(painted(fixture)).toEqual([-1]);
    expect(fixture.view().kind).toBe("paused");
    expect(fixture.requests).toHaveLength(0);
  });

  it("does not paint a controlled snapshot while the tab remains hidden", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      control = deferred<boolean>();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);
    const mutation = applyWorkerControl(
      fixture.runtime,
      "flow-direction",
      "-1",
    );

    // Act
    vi.stubGlobal("document", { hidden: true });
    cancelPendingFrame(fixture.clock);
    fixture.setSide(-1);
    control.resolve(false);
    await mutation;

    // Assert
    expect(painted(fixture)).toEqual([]);
    expect(fixture.requests).toHaveLength(0);
  });

  it("does not repaint or change stored controls after generation abandonment", async () => {
    // Arrange
    const fixture = workerLoopFixture(),
      control = deferred<boolean>();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);
    const mutation = applyWorkerControl(
      fixture.runtime,
      "flow-direction",
      "-1",
    );

    // Act
    fixture.runtime.generation += 1;
    fixture.abandon();
    cancelPendingFrame(fixture.clock);
    control.resolve(false);
    await mutation;

    // Assert
    expect(painted(fixture)).toEqual([]);
    expect(
      fixture.runtime.constructionValues["flow-direction"],
    ).toBeUndefined();
    expect(fixture.owner.captureFrame).not.toHaveBeenCalled();
  });
});
