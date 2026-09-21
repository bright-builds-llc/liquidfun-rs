import { For, Show } from "solid-js";

import type { RenderFrame } from "../physics/frame";
import { debugRows } from "./debug-readout";

export type DebugReadoutProps = {
  readonly enabled: boolean;
  readonly maybeFrame: RenderFrame | undefined;
  readonly stepsThisFrame: number;
};

/** Overlay of live simulation counters. Hidden while the toggle is off. */
export function DebugReadout(props: DebugReadoutProps) {
  return (
    <Show when={props.enabled ? props.maybeFrame : undefined}>
      {(frame) => (
        <dl class="debug-readout">
          <For each={debugRows(frame(), props.stepsThisFrame)}>
            {(row) => (
              <>
                <dt>{row.label}</dt>
                <dd>{row.value}</dd>
              </>
            )}
          </For>
        </dl>
      )}
    </Show>
  );
}
