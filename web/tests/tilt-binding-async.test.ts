import { afterEach, beforeEach, expect, it, vi } from "vitest";
import {
  changeTiltGravity,
  createTiltBinding,
  reapplyStoredTiltGravity,
} from "../src/input/tilt-binding";
import type { TiltDebug } from "../src/input/tilt-gravity";
import { directInputSession, inputSession } from "./input-session-fixture";

let host: EventTarget;
beforeEach(() => {
  host = new EventTarget();
  vi.stubGlobal("window", host);
  vi.stubGlobal("isSecureContext", true);
  vi.stubGlobal("DeviceMotionEvent", {});
  vi.stubGlobal("navigator", { userAgent: "test" });
  vi.stubGlobal("screen", { orientation: { angle: 0 } });
});

it("shows a gravity rejection truthfully when no failure handler is supplied", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  const owner = inputSession({ gravity: () => operation.promise });
  const binding = createTiltBinding();
  const debug: TiltDebug[] = [];
  await changeTiltGravity(
    binding,
    true,
    () => owner,
    () => undefined,
    vi.fn(),
    (value) => debug.push(value),
  );

  // Act
  motion();
  operation.reject(new Error("gravity rejected"));
  await operation.promise.catch(() => undefined);
  await Promise.resolve();

  // Assert
  expect(debug.at(-1)).toEqual({
    kind: "problem",
    detail: "Error: gravity rejected",
  });
  binding.stop?.();
});

it("does not publish a replaced owner's late sample", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  let owner = inputSession({ gravity: () => operation.promise });
  const binding = createTiltBinding();
  const debug: TiltDebug[] = [];
  await changeTiltGravity(
    binding,
    true,
    () => owner,
    () => undefined,
    vi.fn(),
    (value) => debug.push(value),
  );

  // Act
  motion();
  owner = inputSession();
  operation.resolve();
  await operation.promise;

  // Assert
  expect(debug.at(-1)?.kind).toBe("waiting");
  binding.stop?.();
});

it("disabling queues authored gravity after pending tilt and suppresses its late debug", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  let queue = Promise.resolve();
  let applied = "initial";
  const owner = inputSession({
    gravity: () =>
      (queue = queue
        .then(() => operation.promise)
        .then(() => {
          applied = "tilt";
        })),
    restore: () =>
      (queue = queue.then(() => {
        applied = "authored";
      })),
  });
  const binding = createTiltBinding();
  const debug: TiltDebug[] = [];
  const setDebug = (value: TiltDebug) => debug.push(value);
  await changeTiltGravity(
    binding,
    true,
    () => owner,
    () => undefined,
    vi.fn(),
    setDebug,
  );

  // Act
  motion();
  await changeTiltGravity(
    binding,
    false,
    () => owner,
    () => undefined,
    vi.fn(),
    setDebug,
  );
  operation.resolve();
  await queue;

  // Assert
  expect(applied).toBe("authored");
  expect(debug.at(-1)?.kind).toBe("idle");
  expect(binding.stop).toBeUndefined();
});

it("reports restoration rejection through the supplied failure handler", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  const owner = inputSession({ restore: () => operation.promise });
  const binding = createTiltBinding();
  const fail = vi.fn();
  const error = new Error("restore rejected");

  // Act
  await changeTiltGravity(
    binding,
    false,
    () => owner,
    () => undefined,
    vi.fn(),
    vi.fn(),
    fail,
  );
  operation.reject(error);
  await operation.promise.catch(() => undefined);
  await Promise.resolve();

  // Assert
  expect(fail).toHaveBeenCalledWith(error);
});

it("reapplies the stored sample synchronously to a direct recreated scene", async () => {
  // Arrange
  const gravity = vi.fn();
  let owner = directInputSession({ gravity });
  const binding = createTiltBinding();
  const debug = vi.fn();
  await changeTiltGravity(
    binding,
    true,
    () => owner,
    () => undefined,
    vi.fn(),
    debug,
  );
  motion();
  const replacementGravity = vi.fn();
  owner = directInputSession({ gravity: replacementGravity });

  // Act
  reapplyStoredTiltGravity(
    binding,
    () => owner,
    () => undefined,
    debug,
  );

  // Assert
  expect(replacementGravity).toHaveBeenCalledWith(-1, -2);
  expect(debug.mock.calls.at(-1)?.[0].kind).toBe("live");
  binding.stop?.();
});

it("does not let an older sample overwrite the latest pending sample", async () => {
  // Arrange
  const older = Promise.withResolvers<void>();
  const newer = Promise.withResolvers<void>();
  let requested = 0;
  const owner = inputSession({
    gravity: () => (requested++ === 0 ? older.promise : newer.promise),
  });
  const binding = createTiltBinding();
  const debug: TiltDebug[] = [];
  await changeTiltGravity(
    binding,
    true,
    () => owner,
    () => undefined,
    vi.fn(),
    (value) => debug.push(value),
  );

  // Act
  motion(1);
  motion(3);
  older.resolve();
  await older.promise;

  // Assert
  expect(debug.at(-1)?.kind).toBe("waiting");
  newer.resolve();
  await newer.promise;
  expect(debug.at(-1)).toMatchObject({ kind: "live", sample: { x: 3 } });
  binding.stop?.();
});

it("ignores a stored-sample completion after its recreated owner is replaced", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  let owner: import("../src/physics/live-session").PlayerSession =
    directInputSession();
  const binding = createTiltBinding();
  const debug: TiltDebug[] = [];
  const setDebug = (value: TiltDebug) => debug.push(value);
  await changeTiltGravity(
    binding,
    true,
    () => owner,
    () => undefined,
    vi.fn(),
    setDebug,
  );
  motion();
  owner = inputSession({ gravity: () => operation.promise });

  // Act
  reapplyStoredTiltGravity(
    binding,
    () => owner,
    () => undefined,
    setDebug,
  );
  owner = directInputSession();
  setDebug({ kind: "waiting" });
  operation.resolve();
  await operation.promise;

  // Assert
  expect(debug.at(-1)?.kind).toBe("waiting");
  binding.stop?.();
});
afterEach(() => vi.unstubAllGlobals());

it("reports motion setup failure instead of rejecting the UI toggle", async () => {
  // Arrange
  vi.stubGlobal("DeviceMotionEvent", undefined);
  const binding = createTiltBinding();
  const enabled = vi.fn();
  const fail = vi.fn();

  // Act
  const toggle = changeTiltGravity(
    binding,
    true,
    () => inputSession(),
    () => undefined,
    enabled,
    vi.fn(),
    fail,
  );

  // Assert
  await expect(toggle).resolves.toBeUndefined();
  expect(enabled).toHaveBeenLastCalledWith(false);
  expect(fail).toHaveBeenCalledOnce();
});

function motion(x = 1): void {
  const event = new Event("devicemotion");
  Object.defineProperty(event, "accelerationIncludingGravity", {
    value: { x, y: 2, z: 0 },
  });
  host.dispatchEvent(event);
}

it("publishes live tilt only after gravity is accepted", async () => {
  // Arrange
  const operation = Promise.withResolvers<void>();
  const owner = inputSession({ gravity: () => operation.promise });
  const binding = createTiltBinding();
  const debug: TiltDebug[] = [];
  await changeTiltGravity(
    binding,
    true,
    () => owner,
    () => undefined,
    vi.fn(),
    (value) => debug.push(value),
  );

  // Act
  motion();

  // Assert
  expect(debug.at(-1)?.kind).toBe("waiting");
  operation.resolve();
  await operation.promise;
  expect(debug.at(-1)?.kind).toBe("live");
  binding.stop?.();
});
