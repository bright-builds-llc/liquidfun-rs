import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import {
  cancelPendingFrame,
  presentOwnedFrame,
  scheduleFrame,
} from "../src/player/frame-loop";
import { applyWorkerControl } from "../src/player/worker-controls";
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

function start(fixture: ReturnType<typeof workerLoopFixture>) {
  scheduleFrame(fixture.clock, fixture.context, fixture.deps);
  fixture.raf(0);
  vi.advanceTimersByTime(17);
  fixture.raf(17);
}

describe("production live worker clock", () => {
  it("admits existing debt immediately on completion without another full RAF and never repaints held frames", async () => {
    // Arrange
    const fixture = workerLoopFixture();
    start(fixture);

    // Act
    vi.advanceTimersByTime(34);
    fixture.raf(51);
    expect(fixture.deps.drawSceneFrame).not.toHaveBeenCalled();
    fixture.finish(0, 1);
    await flush();

    // Assert
    expect(fixture.requests).toHaveLength(2);
    expect(fixture.requests[1]?.count).toBe(2);
    expect(fixture.deps.drawSceneFrame).toHaveBeenCalledTimes(1);
    expect(fixture.deps.setStepsThisFrame).toHaveBeenLastCalledWith(1);
    cancelPendingFrame(fixture.clock);
  });

  it("settles an already accepted advance into paused state and then performs exactly one manual step", async () => {
    // Arrange
    const fixture = workerLoopFixture();
    start(fixture);

    // Act
    fixture.pause();
    cancelPendingFrame(fixture.clock);
    fixture.finish(0, 1);
    await flush();
    const settled = fixture.view();
    await presentOwnedFrame(
      fixture.clock,
      fixture.owner,
      fixture.context,
      false,
      fixture.deps,
    );

    // Assert
    expect(settled.kind).toBe("paused");
    expect(settled.kind === "paused" ? settled.frame.stepIndex : -1).toBe(1);
    const after = fixture.view();
    expect(after.kind).toBe("paused");
    expect(after.kind === "paused" ? after.frame.stepIndex : -1).toBe(2);
    expect(fixture.owner.nextFrame).toHaveBeenCalledExactlyOnceWith(1);
    expect(fixture.requests).toHaveLength(1);
  });

  it("does not let a pre-flip deferred snapshot overwrite a refreshed controlled picture", async () => {
    // Arrange
    const fixture = workerLoopFixture();
    start(fixture);
    const old = testFrame(1, 1),
      control = deferred<boolean>();
    vi.mocked(fixture.owner.applyControl).mockReturnValueOnce(control.promise);

    // Act
    const mutation = applyWorkerControl(
      fixture.runtime,
      "flow-direction",
      "-1",
    );
    fixture.setSide(-1);
    control.resolve(false);
    await mutation;
    fixture.finish(0, 1, old);
    await flush();

    // Assert
    const painted = vi
      .mocked(fixture.deps.drawSceneFrame)
      .mock.calls.map((call) => call[1].rigidSegments[0]);
    expect(painted).toEqual([-1]);
    expect(fixture.runtime.constructionValues["flow-direction"]).toBe("-1");
    cancelPendingFrame(fixture.clock);
  });

  it("suppresses late frames and failures after route abandonment", async () => {
    // Arrange
    const fixture = workerLoopFixture();
    start(fixture);

    // Act
    fixture.abandon();
    cancelPendingFrame(fixture.clock);
    fixture.finish(0, 1);
    await flush();

    // Assert
    expect(fixture.deps.drawSceneFrame).not.toHaveBeenCalled();
    expect(fixture.deps.fail).not.toHaveBeenCalled();
    expect(fixture.requests).toHaveLength(1);
  });

  it("does not publish or continue stepping when a tab becomes hidden during accepted work", async () => {
    // Arrange
    const fixture = workerLoopFixture();
    start(fixture);

    // Act
    vi.stubGlobal("document", { hidden: true });
    fixture.finish(0, 1);
    await flush();

    // Assert
    expect(fixture.deps.drawSceneFrame).not.toHaveBeenCalled();
    expect(fixture.requests).toHaveLength(1);
    cancelPendingFrame(fixture.clock);
  });
});
