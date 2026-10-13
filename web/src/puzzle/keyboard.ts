/** Where a key event came from, as far as the puzzle key map cares (Pitfall 8). */
export type KeyTarget = "canvas" | "control" | "button" | "formControl" | "other";

/** The parts of a `KeyboardEvent` the map reads, so the map stays pure. */
export type PuzzleKeyInput = {
  readonly key: string;
  readonly shiftKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly altKey: boolean;
  readonly target: KeyTarget;
};

/** Closed set of editor and transport commands a key can produce (T-39-10). */
export type PuzzleKeyCommand =
  | { readonly kind: "nudge"; readonly dx: number; readonly dy: number }
  | { readonly kind: "rotate"; readonly direction: 1 | -1 }
  | { readonly kind: "remove" }
  | { readonly kind: "togglePlay" }
  | { readonly kind: "escape" }
  | { readonly kind: "cycleSelection"; readonly direction: 1 | -1 };

/** Grid cells a Shift+Arrow nudge moves (D-23). */
export const NUDGE_SHIFT_CELLS = 10;

type Direction = { readonly dx: number; readonly dy: number };

/** World y is up, so ArrowUp is +y. */
const ARROW_DIRECTIONS: ReadonlyMap<string, Direction> = new Map([
  ["ArrowLeft", { dx: -1, dy: 0 }],
  ["ArrowRight", { dx: 1, dy: 0 }],
  ["ArrowUp", { dx: 0, dy: 1 }],
  ["ArrowDown", { dx: 0, dy: -1 }],
]);

/** Space and Enter on a button already click it natively. */
function isNativeActivation(input: PuzzleKeyInput): boolean {
  return input.target === "button" && (input.key === " " || input.key === "Enter");
}

/** Pure D-23 map; phase gating happens in the controller (editing keys are no-ops outside building). */
export function puzzleKeyCommand(input: PuzzleKeyInput): PuzzleKeyCommand | undefined {
  if (input.ctrlKey || input.metaKey || input.altKey) {
    return undefined;
  }
  if (input.target === "formControl" || isNativeActivation(input)) {
    return undefined;
  }

  const maybeArrow = ARROW_DIRECTIONS.get(input.key);
  if (maybeArrow !== undefined) {
    const cells = input.shiftKey ? NUDGE_SHIFT_CELLS : 1;
    return { kind: "nudge", dx: maybeArrow.dx * cells, dy: maybeArrow.dy * cells };
  }

  switch (input.key) {
    case "r":
    case "R":
      return { kind: "rotate", direction: input.shiftKey ? -1 : 1 };
    case "Delete":
    case "Backspace":
      return { kind: "remove" };
    case " ":
      return { kind: "togglePlay" };
    case "Escape":
      return { kind: "escape" };
    case "Tab":
      return input.target === "canvas"
        ? { kind: "cycleSelection", direction: input.shiftKey ? -1 : 1 }
        : undefined;
    default:
      return undefined;
  }
}

/** Classifies an event target for the map: canvas, button/a, select/input/textarea, other HUD element. */
export function keyTargetOf(target: EventTarget | null): KeyTarget {
  if (typeof HTMLElement === "undefined" || !(target instanceof HTMLElement)) {
    return "other";
  }
  if (target instanceof HTMLCanvasElement) {
    return "canvas";
  }
  if (target instanceof HTMLButtonElement || target instanceof HTMLAnchorElement) {
    return "button";
  }
  if (
    target instanceof HTMLSelectElement ||
    target instanceof HTMLInputElement ||
    target instanceof HTMLTextAreaElement
  ) {
    return "formControl";
  }
  return "other";
}
