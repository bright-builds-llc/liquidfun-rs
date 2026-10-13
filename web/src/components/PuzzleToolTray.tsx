import { Index, createMemo } from "solid-js";

import { trayModel } from "../puzzle/hud-format";
import type { PuzzleToolKind } from "../puzzle/placements";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import { PlankIcon, ValveIcon } from "./PuzzleIcons";

export type PuzzleToolTrayProps = {
  readonly state: PuzzleState;
  readonly onToggleArm: (kind: PuzzleToolKind) => void;
  readonly onPlaceAtCenter: (kind: PuzzleToolKind) => void;
};

function ToolKindIcon(props: { readonly kind: PuzzleToolKind }) {
  switch (props.kind) {
    case "plank":
      return <PlankIcon />;
    case "valve":
      return <ValveIcon />;
    default: {
      const unreachable: never = props.kind;
      return unreachable;
    }
  }
}

/**
 * Tool tray leaf (D-17, D-23): one button per budget kind with its remaining
 * count. A click or Space arms the kind (again to disarm); Enter places one
 * tool at the build-area centre. Unavailable items stay focusable with
 * `aria-disabled` so focus survives the last Enter placement.
 */
export function PuzzleToolTray(props: PuzzleToolTrayProps) {
  const items = createMemo(() => trayModel(props.state));
  return (
    <div
      class="puzzle-tray"
      data-hud-obstacle
      data-kind-count={items().length}
      role="group"
      aria-label="Tools"
    >
      <Index each={items()}>
        {(item) => (
          <button
            type="button"
            class="product-control puzzle-tray-item"
            aria-pressed={item().armed ? "true" : "false"}
            aria-disabled={item().disabled ? "true" : undefined}
            aria-label={item().accessibleName}
            title={item().title}
            data-tool-kind={item().kind}
            data-remaining={item().remaining}
            onClick={() => {
              if (!item().disabled) {
                props.onToggleArm(item().kind);
              }
            }}
            onKeyDown={(event) => {
              if (event.key !== "Enter") {
                return;
              }
              event.preventDefault();
              if (!item().disabled) {
                props.onPlaceAtCenter(item().kind);
              }
            }}
          >
            <ToolKindIcon kind={item().kind} />
            <span class="puzzle-tray-name">{item().label}</span>
            <span class="puzzle-tray-count" data-empty={item().remaining === 0 ? "true" : "false"}>
              {item().countText}
            </span>
          </button>
        )}
      </Index>
    </div>
  );
}
