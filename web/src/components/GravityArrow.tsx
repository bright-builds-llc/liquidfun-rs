import { Show } from "solid-js";

import type { TiltDebug } from "../input/tilt-gravity";
import { formatArrowPoints, maybeGravityArrow } from "./gravity-arrow";

export type GravityArrowProps = {
  readonly tiltGravityEnabled: boolean;
  readonly tiltDebug: TiltDebug;
};

/** Canvas-corner arrow for the live phone-tilt gravity vector. */
export function GravityArrow(props: GravityArrowProps) {
  const maybeArrow = () =>
    maybeGravityArrow(props.tiltGravityEnabled, props.tiltDebug);

  return (
    <Show when={maybeArrow()}>
      {(arrow) => (
        <div class="gravity-arrow" role="img" aria-label={arrowLabel(props.tiltDebug)}>
          <svg viewBox="0 0 48 48" aria-hidden="true">
            <line
              x1={arrow().x1}
              y1={arrow().y1}
              x2={arrow().x2}
              y2={arrow().y2}
              stroke-width={arrow().strokeWidth}
            />
            <polygon points={formatArrowPoints(arrow().head)} />
          </svg>
        </div>
      )}
    </Show>
  );
}

function arrowLabel(tiltDebug: TiltDebug): string {
  if (tiltDebug.kind !== "live") {
    return "Gravity direction";
  }
  const x = tiltDebug.gravity.x.toFixed(2);
  const y = tiltDebug.gravity.y.toFixed(2);
  return `Gravity direction ${x}, ${y}`;
}
