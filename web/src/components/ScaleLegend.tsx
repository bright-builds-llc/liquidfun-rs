import { Show } from "solid-js";

import { scaleLegend } from "../render/scale-legend";

export type ScaleLegendProps = {
  readonly maybePixelsPerMeter: number | undefined;
};

/** One world-scale tick anchored to the current camera. */
export function ScaleLegend(props: ScaleLegendProps) {
  const maybeLegend = () => {
    const maybePixelsPerMeter = props.maybePixelsPerMeter;
    if (maybePixelsPerMeter === undefined) {
      return undefined;
    }

    return scaleLegend(maybePixelsPerMeter);
  };

  return (
    <Show when={maybeLegend()}>
      {(legend) => (
        <div
          class="scale-legend"
          role="img"
          aria-label={`Scale bar, ${legend().accessibleLabel}`}
          data-scale-label={legend().label}
        >
          <span class="scale-legend-label" aria-hidden="true">
            {legend().label}
          </span>
          <span
            class="scale-legend-bar"
            aria-hidden="true"
            style={{ width: `${legend().widthPx}px` }}
          />
        </div>
      )}
    </Show>
  );
}
