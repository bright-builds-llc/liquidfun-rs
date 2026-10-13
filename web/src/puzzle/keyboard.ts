/** Where a key event came from, as far as the puzzle key map cares (Pitfall 8). */
export type KeyTarget = "canvas" | "control" | "button" | "formControl" | "other";

/** The parts of a `KeyboardEvent` the map reads, so the map stays pure. */
export type PuzzleKeyInput = {
  readonly key: string;
  readonly shiftKey: boolean;
  readonly ctrlKey: boolean;
  readonly metaKey: boolean;
  readonly altKey: boolean;
  /** `getModifierState("AltGraph")`; Windows reports AltGr as Ctrl+Alt plus this state. */
  readonly altGraph: boolean;
  readonly target: KeyTarget;
};

/** Closed set of editor and transport commands a key can produce (T-39-10). */
export type PuzzleKeyCommand =
  | { readonly kind: "nudge"; readonly dx: number; readonly dy: number }
  | { readonly kind: "rotate"; readonly direction: 1 | -1 }
  | { readonly kind: "remove" }
  | { readonly kind: "valveTime"; readonly direction: 1 | -1 }
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

/** Space and Enter on a button or other activatable element already activate it natively. */
function isNativeActivation(input: PuzzleKeyInput): boolean {
  return input.target === "button" && (input.key === " " || input.key === "Enter");
}

/**
 * True when a modifier makes the key a shortcut for something else. Some
 * layouts type `[` and `]` with Option (macOS German) or AltGr (Windows German
 * and French), and `key` already names the produced bracket, so only Meta or a
 * Ctrl that is not part of AltGr blocks a bracket.
 */
function isModifiedShortcut(input: PuzzleKeyInput): boolean {
  if (input.key === "[" || input.key === "]") {
    return input.metaKey || (input.ctrlKey && !input.altGraph);
  }
  return input.ctrlKey || input.metaKey || input.altKey;
}

/** Pure D-23 map; phase gating happens in the controller (editing keys are no-ops outside building). */
export function puzzleKeyCommand(input: PuzzleKeyInput): PuzzleKeyCommand | undefined {
  if (isModifiedShortcut(input)) {
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
    case "[":
      return { kind: "valveTime", direction: -1 };
    case "]":
      return { kind: "valveTime", direction: 1 };
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

/**
 * True for an auto-repeated Play/Stop toggle. Only the first keydown of a held
 * Space toggles; the caller swallows repeats so a long press never stops the
 * run it just started. Arrow-nudge repeats stay useful and pass through.
 */
export function isRepeatedPlayToggle(command: PuzzleKeyCommand, repeat: boolean): boolean {
  return repeat && command.kind === "togglePlay";
}

/** The element facts the classifier reads, so it is testable without a DOM. */
export type KeyTargetElement = {
  readonly tagName: string;
  readonly isContentEditable: boolean;
  getAttribute(name: string): string | null;
};

/** Elements whose native Space/Enter activation the map must not take, such as a `<summary>`. */
const ACTIVATABLE_TAGS: ReadonlySet<string> = new Set(["BUTTON", "A", "SUMMARY"]);
const FORM_CONTROL_TAGS: ReadonlySet<string> = new Set(["SELECT", "INPUT", "TEXTAREA"]);
/** ARIA widget roles that activate or toggle on Space or Enter. */
const ACTIVATABLE_ROLES: ReadonlySet<string> = new Set([
  "button",
  "checkbox",
  "link",
  "menuitem",
  "menuitemcheckbox",
  "menuitemradio",
  "option",
  "radio",
  "switch",
  "tab",
]);

/**
 * Classifies one element: canvas; a button, link, `<summary>` or ARIA
 * activatable role as a button; select/input/textarea or editable text as a
 * form control; anything else as other.
 */
export function classifyKeyTarget(element: KeyTargetElement): KeyTarget {
  const tag = element.tagName.toUpperCase();
  if (tag === "CANVAS") {
    return "canvas";
  }
  if (FORM_CONTROL_TAGS.has(tag) || element.isContentEditable) {
    return "formControl";
  }
  const maybeRole = element.getAttribute("role");
  if (ACTIVATABLE_TAGS.has(tag) || (maybeRole !== null && ACTIVATABLE_ROLES.has(maybeRole))) {
    return "button";
  }
  return "other";
}

/** Classifies an event target for the map; non-HTML targets are other. */
export function keyTargetOf(target: EventTarget | null): KeyTarget {
  if (typeof HTMLElement === "undefined" || !(target instanceof HTMLElement)) {
    return "other";
  }
  return classifyKeyTarget(target);
}
