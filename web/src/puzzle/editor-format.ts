import { angleDegrees, type BuildingEditor } from "./editor";
import { PLACEMENT_STATUS, placementReason } from "./placement-status";

/** One tray button's view model (D-06, D-17). */
export type TrayItemModel<K extends string> = {
  readonly kind: K;
  readonly label: string;
  readonly remaining: number;
  readonly countText: string;
  readonly accessibleName: string;
  readonly title: string;
  readonly armed: boolean;
  readonly disabled: boolean;
};

/** Result-slot message while building; `status` is the invalid status line. */
export type BuildingMessage = { readonly tone: "status" | "hint" | "goal"; readonly text: string };

/** Announced when an armed kind is disarmed without placing. */
export const PLACEMENT_CANCELED_ANNOUNCEMENT = "Placement canceled.";

/** Announced after Clear. */
export const CLEARED_ANNOUNCEMENT = "All tools cleared.";

const MINUS_SIGN = "−";

/** Player label for a tool kind ("plank" → "Plank"). */
export function toolKindLabel(kind: string): string {
  return kind.charAt(0).toUpperCase() + kind.slice(1);
}

function lowerFirst(text: string): string {
  return text.charAt(0).toLowerCase() + text.slice(1);
}

/** Tray items in budget order for any set of kinds (D-06). */
export function trayItems<K extends string>(
  budget: readonly { readonly kind: K; readonly count: number }[],
  placedKinds: readonly K[],
  maybeArmed: K | undefined,
  enabled: boolean,
): readonly TrayItemModel<K>[] {
  return budget.map(({ kind, count }) => {
    const placed = placedKinds.filter((placedKind) => placedKind === kind).length;
    const remaining = Math.max(0, count - placed);
    const label = toolKindLabel(kind);
    return {
      kind,
      label,
      remaining,
      countText: `×${remaining}`,
      accessibleName: remaining === 0 ? `${label}, none left` : `${label}, ${remaining} left`,
      title: `Place a ${label.toLowerCase()} (${remaining} left)`,
      armed: maybeArmed === kind,
      disabled: !enabled || remaining === 0,
    };
  });
}

/** Invalid-placement status line from the committed snapshot only (UI-D14); undefined when nothing blocks Play. */
export function statusLine(editor: BuildingEditor): string | undefined {
  const maybeCommitted = editor.maybeCommittedInspection;
  if (maybeCommitted === undefined || maybeCommitted.statuses.length !== editor.tools.length) {
    return undefined;
  }

  const invalid = maybeCommitted.statuses.flatMap((status, index) =>
    status === PLACEMENT_STATUS.valid ? [] : [{ status, id: editor.tools[index]?.id }],
  );
  const [maybeFirst] = invalid;
  if (maybeFirst === undefined) {
    return undefined;
  }
  if (invalid.length === 1) {
    return `${placementReason(maybeFirst.status)}. Move or remove that tool to play.`;
  }

  const count = `${invalid.length} tools need moving or removing before you can play.`;
  const maybeSelected = invalid.find((entry) => entry.id === editor.maybeSelectedId);
  return maybeSelected === undefined
    ? `${count} Select one to see why.`
    : `${placementReason(maybeSelected.status)}. ${count}`;
}

/** Building message priority: status line, armed hint, empty hint, goal (UI-SPEC). */
export function buildingMessage(editor: BuildingEditor, goal: string): BuildingMessage {
  const maybeStatus = statusLine(editor);
  if (maybeStatus !== undefined) {
    return { tone: "status", text: maybeStatus };
  }
  if (editor.maybeArmedKind !== undefined) {
    const label = toolKindLabel(editor.maybeArmedKind).toLowerCase();
    return { tone: "hint", text: `Click or tap where the ${label} should go.` };
  }
  if (editor.tools.length === 0) {
    return { tone: "hint", text: `${goal} Pick a tool from the tray to start.` };
  }

  return { tone: "goal", text: goal };
}

function withReason(sentence: string, maybeReason: string | undefined): string {
  return maybeReason === undefined ? sentence : `${sentence} ${maybeReason}.`;
}

/** "Plank ready. Click or tap the build area." */
export function armedAnnouncement(label: string): string {
  return `${label} ready. Click or tap the build area.`;
}

/** "Plank placed." plus the reason when invalid. */
export function placedAnnouncement(label: string, maybeReason: string | undefined): string {
  return withReason(`${label} placed.`, maybeReason);
}

/** "Plank 1 of 2 selected." plus the reason when invalid. */
export function selectedAnnouncement(
  label: string,
  ordinal: number,
  total: number,
  maybeReason: string | undefined,
): string {
  return withReason(`${label} ${ordinal} of ${total} selected.`, maybeReason);
}

/** "Rotated to −15°." with a U+2212 minus sign. */
export function rotatedAnnouncement(angleIndex: number): string {
  const degrees = angleDegrees(angleIndex);
  const text = degrees < 0 ? `${MINUS_SIGN}${-degrees}` : `${degrees}`;
  return `Rotated to ${text}°.`;
}

/** Announces a validity flip of one tool; undefined when validity is unchanged. */
export function validityChangeAnnouncement(
  wasValid: boolean,
  isValid: boolean,
  maybeReason: string | undefined,
): string | undefined {
  if (wasValid === isValid) {
    return undefined;
  }
  if (isValid) {
    return "This tool fits again.";
  }
  if (maybeReason === undefined) {
    throw new Error("A newly invalid tool needs a reason");
  }

  return `Now ${lowerFirst(maybeReason)}.`;
}

/** "Plank removed." */
export function removedAnnouncement(label: string): string {
  return `${label} removed.`;
}

/** Stop announcement listing stars earned so far, or that tools are kept. */
export function stoppedAnnouncement(starNames: readonly string[]): string {
  return starNames.length === 0
    ? "Stopped. Your tools are kept."
    : `Stopped. Stars kept: ${starNames.join(", ")}.`;
}
