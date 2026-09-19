import { describe, expect, it } from "vitest";

import {
  cssPointFromClient,
  emptyGestureState,
  parsePointerKind,
  reducePointerEvent,
} from "../src/input/pointer";

describe("parsePointerKind", () => {
  it("maps the allowlisted pointer kinds", () => {
    // Arrange
    const rawKinds = ["down", "move", "up", "cancel"] as const;

    // Act
    const parsed = rawKinds.map((raw) => parsePointerKind(raw));

    // Assert
    expect(parsed).toEqual(["down", "move", "up", "cancel"]);
  });

  it("rejects unknown or differently cased pointer kinds", () => {
    // Arrange
    const rawKinds = ["Down", "click", ""];

    // Act
    const parsed = rawKinds.map((raw) => parsePointerKind(raw));

    // Assert
    expect(parsed).toEqual([undefined, undefined, undefined]);
  });
});

describe("cssPointFromClient", () => {
  it("converts client coordinates into CSS pixels relative to the rect", () => {
    // Arrange
    const rect = { left: 10, top: 20, width: 320, height: 180 };

    // Act
    const maybePoint = cssPointFromClient(100, 40, rect);

    // Assert
    expect(maybePoint).toEqual({ x: 90, y: 20 });
  });

  it("rejects a non-finite client x", () => {
    // Arrange
    const rect = { left: 10, top: 20, width: 320, height: 180 };

    // Act
    const maybePoint = cssPointFromClient(Number.NaN, 40, rect);

    // Assert
    expect(maybePoint).toBeUndefined();
  });

  it("rejects a non-finite rect top", () => {
    // Arrange
    const rect = {
      left: 10,
      top: Number.POSITIVE_INFINITY,
      width: 320,
      height: 180,
    };

    // Act
    const maybePoint = cssPointFromClient(100, 40, rect);

    // Assert
    expect(maybePoint).toBeUndefined();
  });

  it("rejects an empty rect", () => {
    // Arrange
    const rect = { left: 10, top: 20, width: 0, height: 180 };

    // Act
    const maybePoint = cssPointFromClient(100, 40, rect);

    // Assert
    expect(maybePoint).toBeUndefined();
  });
});

describe("reducePointerEvent", () => {
  it("captures on down and emits move for the stored pointer id", () => {
    // Arrange
    const afterDown = reducePointerEvent(emptyGestureState(), {
      name: "pointerdown",
      pointerId: 1,
    });

    // Act
    const afterMove = reducePointerEvent(afterDown.next, {
      name: "pointermove",
      pointerId: 1,
    });

    // Assert
    expect(afterDown.maybeKind).toBe("down");
    expect(afterDown.shouldCapture).toBe(true);
    expect(afterDown.next.maybePointerId).toBe(1);
    expect(afterMove.maybeKind).toBe("move");
    expect(afterMove.next.maybePointerId).toBe(1);
  });

  it("ignores move for a different pointer id", () => {
    // Arrange
    const afterDown = reducePointerEvent(emptyGestureState(), {
      name: "pointerdown",
      pointerId: 1,
    });

    // Act
    const afterMove = reducePointerEvent(afterDown.next, {
      name: "pointermove",
      pointerId: 2,
    });

    // Assert
    expect(afterMove.maybeKind).toBeUndefined();
    expect(afterMove.next).toEqual(afterDown.next);
    expect(afterMove.shouldCapture).toBe(false);
    expect(afterMove.shouldRelease).toBe(false);
  });

  it("clears the stored id and requests release on up", () => {
    // Arrange
    const afterDown = reducePointerEvent(emptyGestureState(), {
      name: "pointerdown",
      pointerId: 1,
    });

    // Act
    const afterUp = reducePointerEvent(afterDown.next, {
      name: "pointerup",
      pointerId: 1,
    });

    // Assert
    expect(afterUp.maybeKind).toBe("up");
    expect(afterUp.shouldRelease).toBe(true);
    expect(afterUp.next.maybePointerId).toBeUndefined();
  });

  it("clears the stored id and requests release on cancel", () => {
    // Arrange
    const afterDown = reducePointerEvent(emptyGestureState(), {
      name: "pointerdown",
      pointerId: 1,
    });

    // Act
    const afterCancel = reducePointerEvent(afterDown.next, {
      name: "pointercancel",
      pointerId: 1,
    });

    // Assert
    expect(afterCancel.maybeKind).toBe("cancel");
    expect(afterCancel.shouldRelease).toBe(true);
    expect(afterCancel.next.maybePointerId).toBeUndefined();
  });

  it("clears the stored id and requests release on lostpointercapture", () => {
    // Arrange
    const afterDown = reducePointerEvent(emptyGestureState(), {
      name: "pointerdown",
      pointerId: 1,
    });

    // Act
    const afterLost = reducePointerEvent(afterDown.next, {
      name: "lostpointercapture",
      pointerId: 1,
    });

    // Assert
    expect(afterLost.maybeKind).toBe("cancel");
    expect(afterLost.shouldRelease).toBe(true);
    expect(afterLost.next.maybePointerId).toBeUndefined();
  });

  it("does not emit cancel when lostpointercapture arrives with empty state", () => {
    // Arrange
    const empty = emptyGestureState();

    // Act
    const afterLost = reducePointerEvent(empty, {
      name: "lostpointercapture",
      pointerId: 1,
    });

    // Assert
    expect(afterLost.maybeKind).toBeUndefined();
    expect(afterLost.shouldRelease).toBe(false);
    expect(afterLost.next).toEqual(empty);
  });
});
