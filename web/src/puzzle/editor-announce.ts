import type { PuzzleEditEvent } from "./edit-event";
import type { BuildingEditor, EditorTool } from "./editor";
import {
  CLEARED_ANNOUNCEMENT,
  PLACEMENT_CANCELED_ANNOUNCEMENT,
  armedAnnouncement,
  placedAnnouncement,
  removedAnnouncement,
  rotatedAnnouncement,
  selectedAnnouncement,
  toolKindLabel,
  validityChangeAnnouncement,
} from "./editor-format";
import { PLACEMENT_STATUS, placementReason, type PlacementStatusCode } from "./placement-status";

function toolById(editor: BuildingEditor, id: number | undefined): EditorTool | undefined {
  return id === undefined ? undefined : editor.tools.find((tool) => tool.id === id);
}

/** Status of a tool in the committed snapshot (UI-D14); undefined when the snapshot does not cover it. */
function committedStatus(editor: BuildingEditor, id: number): PlacementStatusCode | undefined {
  const maybeCommitted = editor.maybeCommittedInspection;
  if (maybeCommitted === undefined || maybeCommitted.statuses.length !== editor.tools.length) {
    return undefined;
  }

  const index = editor.tools.findIndex((tool) => tool.id === id);
  return index < 0 ? undefined : maybeCommitted.statuses[index];
}

function maybeReason(editor: BuildingEditor, id: number): string | undefined {
  const maybeStatus = committedStatus(editor, id);
  return maybeStatus === undefined || maybeStatus === PLACEMENT_STATUS.valid
    ? undefined
    : placementReason(maybeStatus);
}

/** A tool with no committed status counts as valid, so a newly invalid tool is announced. */
function isValid(editor: BuildingEditor, id: number): boolean {
  return (committedStatus(editor, id) ?? PLACEMENT_STATUS.valid) === PLACEMENT_STATUS.valid;
}

function validityChange(
  before: BuildingEditor,
  after: BuildingEditor,
  id: number | undefined,
): string | undefined {
  if (id === undefined) {
    return undefined;
  }

  return validityChangeAnnouncement(
    isValid(before, id),
    isValid(after, id),
    maybeReason(after, id),
  );
}

function armAnnouncement(before: BuildingEditor, after: BuildingEditor): string | undefined {
  const maybeArmed = after.maybeArmedKind;
  if (maybeArmed !== undefined && maybeArmed !== before.maybeArmedKind) {
    return armedAnnouncement(toolKindLabel(maybeArmed));
  }
  if (before.maybeArmedKind !== undefined && maybeArmed === undefined) {
    return PLACEMENT_CANCELED_ANNOUNCEMENT;
  }

  return undefined;
}

function placeAnnouncement(before: BuildingEditor, after: BuildingEditor): string | undefined {
  const maybePlaced = toolById(after, after.maybeSelectedId);
  if (maybePlaced === undefined || after.tools.length === before.tools.length) {
    return undefined;
  }

  return placedAnnouncement(
    toolKindLabel(maybePlaced.placement.kind),
    maybeReason(after, maybePlaced.id),
  );
}

function selectAnnouncement(
  after: BuildingEditor,
  maybeId: number | undefined,
): string | undefined {
  const maybeTool = toolById(after, maybeId);
  if (maybeTool === undefined) {
    return undefined;
  }

  const ordinal = after.tools.indexOf(maybeTool) + 1;
  return selectedAnnouncement(
    toolKindLabel(maybeTool.placement.kind),
    ordinal,
    after.tools.length,
    maybeReason(after, maybeTool.id),
  );
}

function rotateAnnouncement(before: BuildingEditor, after: BuildingEditor): string | undefined {
  const maybeTool = toolById(after, after.maybeSelectedId);
  if (maybeTool === undefined) {
    return undefined;
  }

  const rotated = rotatedAnnouncement(maybeTool.placement.angle);
  const maybeChange = validityChange(before, after, maybeTool.id);
  return maybeChange === undefined ? rotated : `${rotated} ${maybeChange}`;
}

function removeAnnouncement(before: BuildingEditor): string | undefined {
  const maybeRemoved = toolById(before, before.maybeSelectedId);
  return maybeRemoved === undefined
    ? undefined
    : removedAnnouncement(toolKindLabel(maybeRemoved.placement.kind));
}

/**
 * Polite announcement for one committed edit (UI-SPEC copy). Validity reads
 * only each editor's committed snapshot, so live inspections never change
 * what is announced (UI-D14). Undefined means nothing to announce.
 */
export function editAnnouncement(
  before: BuildingEditor,
  after: BuildingEditor,
  event: PuzzleEditEvent,
): string | undefined {
  switch (event.kind) {
    case "arm":
    case "disarm":
      return armAnnouncement(before, after);
    case "place":
      return placeAnnouncement(before, after);
    case "select":
      return selectAnnouncement(after, event.maybeId);
    case "move":
      return validityChange(before, after, event.id);
    case "nudge":
      return validityChange(before, after, after.maybeSelectedId);
    case "rotate":
      return rotateAnnouncement(before, after);
    case "remove":
      return removeAnnouncement(before);
    case "clear":
      return before.tools.length === 0 ? undefined : CLEARED_ANNOUNCEMENT;
    case "inspected":
      return undefined;
    default: {
      const unreachable: never = event;
      return unreachable;
    }
  }
}
