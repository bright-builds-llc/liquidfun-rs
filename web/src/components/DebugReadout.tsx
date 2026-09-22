import { For, Show, createSignal, onCleanup, onMount } from "solid-js";

import type { RenderFrame } from "../physics/frame";
import { debugRows } from "./debug-readout";
import {
  FPS_BASELINE,
  FPS_COUNTER_WINDOW_MS,
  formatFps,
  fpsHistory,
  fpsOverWindow,
  fpsTone,
  graphScaleMax,
  type FpsTick,
  type FpsTone,
} from "./fps-meter";

export type DebugReadoutProps = {
  readonly enabled: boolean;
  readonly maybeFrame: RenderFrame | undefined;
  readonly stepsThisFrame: number;
  readonly fpsTicks: readonly FpsTick[];
};

const GRAPH_HEIGHT = 32;

/** Overlay of live simulation counters. Hidden while the toggle is off. */
export function DebugReadout(props: DebugReadoutProps) {
  const [nowMs, setNowMs] = createSignal(performance.now());

  onMount(() => {
    const id = window.setInterval(() => setNowMs(performance.now()), 100);
    onCleanup(() => window.clearInterval(id));
  });

  return (
    <Show when={props.enabled ? props.maybeFrame : undefined}>
      {(frame) => {
        const rates = () => fpsOverWindow(props.fpsTicks, nowMs(), FPS_COUNTER_WINDOW_MS);
        const history = () => fpsHistory(props.fpsTicks, nowMs());
        return (
          <div class="debug-readout">
            <dl>
              <For each={debugRows(frame(), props.stepsThisFrame)}>
                {(row) => (
                  <>
                    <dt>{row.label}</dt>
                    <dd>{row.value}</dd>
                  </>
                )}
              </For>
            </dl>
            <FpsMeter
              label="Simulation"
              fps={rates().simFps}
              samples={history().sim}
            />
            <FpsMeter
              label="Render"
              fps={rates().renderFps}
              samples={history().render}
            />
          </div>
        );
      }}
    </Show>
  );
}

function FpsMeter(props: {
  readonly label: string;
  readonly fps: number;
  readonly samples: readonly number[];
}) {
  const tone = () => fpsTone(props.fps);
  const scaleMax = () => graphScaleMax(props.samples);
  const baselineStyle = () => {
    const percent = (1 - FPS_BASELINE / scaleMax()) * 100;
    let transform = "translateY(-50%)";
    if (percent < 12) {
      transform = "translateY(0)";
    } else if (percent > 88) {
      transform = "translateY(-100%)";
    }
    return { top: `${percent}%`, transform };
  };

  return (
    <div class="fps-meter" data-fps-tone={tone()}>
      <div class="fps-meter-label">
        <span>{props.label}</span>
        <span class={`fps-meter-value fps-meter-value--${tone()}`}>
          {`${formatFps(props.fps)} fps`}
        </span>
      </div>
      <div class="fps-graph-frame">
        <svg
          class="fps-graph"
          viewBox={`0 0 ${props.samples.length} ${GRAPH_HEIGHT}`}
          preserveAspectRatio="none"
          aria-hidden="true"
        >
          <line
            x1="0"
            x2={props.samples.length}
            y1={GRAPH_HEIGHT - (FPS_BASELINE / scaleMax()) * GRAPH_HEIGHT}
            y2={GRAPH_HEIGHT - (FPS_BASELINE / scaleMax()) * GRAPH_HEIGHT}
            stroke="#8AA0B4"
            stroke-dasharray="3 2"
            stroke-width="1"
            vector-effect="non-scaling-stroke"
          />
          <For each={props.samples}>
            {(sample, index) => {
              const barHeight = (Math.max(sample, 0) / scaleMax()) * GRAPH_HEIGHT;
              return (
                <rect
                  x={index()}
                  y={GRAPH_HEIGHT - barHeight}
                  width="0.82"
                  height={barHeight}
                  fill={toneColor(fpsTone(sample))}
                />
              );
            }}
          </For>
        </svg>
        <span class="fps-baseline" style={baselineStyle()}>
          {String(FPS_BASELINE)}
        </span>
      </div>
    </div>
  );
}

function toneColor(tone: FpsTone): string {
  if (tone === "good") {
    return "#3DDC97";
  }
  if (tone === "fair") {
    return "#E6C35C";
  }
  return "#F07178";
}
