import { expect, it, vi } from "vitest";
import { forwardScenePointer } from "../src/input/canvas-pointer";
import { directInputSession, inputSession } from "./input-session-fixture";

it("counts an async pointer only after its operation succeeds", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  const session = inputSession({ pointer: () => operation.promise });
  const accepted = vi.fn();
  const count = vi.fn();
  const fail = vi.fn();

  // Act
  forwardScenePointer(session, "playing", "down", 1, 2, accepted, count, fail);

  // Assert
  expect(accepted).not.toHaveBeenCalled();
  operation.resolve();
  await operation.promise;
  expect(accepted).toHaveBeenCalledWith("down");
  expect(count.mock.calls[0]?.[0](4)).toBe(5);
  expect(fail).not.toHaveBeenCalled();
});

it("reports a rejected pointer without counting it", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  const owner = inputSession({ pointer: () => operation.promise });
  const accepted = vi.fn();
  const count = vi.fn();
  const fail = vi.fn();
  const error = new Error("pointer rejected");

  // Act
  forwardScenePointer(owner, "paused", "move", 0, 0, accepted, count, fail);
  operation.reject(error);
  await operation.promise.catch(() => undefined);
  await Promise.resolve();

  // Assert
  expect(fail).toHaveBeenCalledWith(error);
  expect(accepted).not.toHaveBeenCalled();
  expect(count).not.toHaveBeenCalled();
});

it.each(["resolve", "reject"] as const)(
  "ignores a stale pointer %s completion",
  async (completion) => {
    // Arrange
    const operation = Promise.withResolvers<void>();
    const owner = inputSession({ pointer: () => operation.promise });
    let current = true;
    const accepted = vi.fn();
    const count = vi.fn();
    const fail = vi.fn();

    // Act
    forwardScenePointer(
      owner,
      "playing",
      "up",
      0,
      0,
      accepted,
      count,
      fail,
      () => current,
    );
    current = false;
    if (completion === "resolve") operation.resolve();
    else operation.reject(new Error("stale"));
    await operation.promise.catch(() => undefined);
    await Promise.resolve();

    // Assert
    expect(accepted).not.toHaveBeenCalled();
    expect(count).not.toHaveBeenCalled();
    expect(fail).not.toHaveBeenCalled();
  },
);

it("keeps direct pointer acceptance synchronous", () => {
  // Arrange
  const owner = directInputSession();
  const accepted = vi.fn();
  const count = vi.fn();

  // Act
  forwardScenePointer(
    owner,
    "playing",
    "cancel",
    0,
    0,
    accepted,
    count,
    vi.fn(),
  );

  // Assert
  expect(accepted).toHaveBeenCalledWith("cancel");
  expect(count.mock.calls[0]?.[0](1)).toBe(2);
});
