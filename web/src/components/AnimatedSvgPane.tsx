import { Show, createEffect, createSignal, on, onCleanup } from "solid-js";

import type { SceneId } from "../catalog/scenes";
import {
  DEFAULT_SVG_EXPORT_SECONDS,
  exportProgressPercent,
  formatGenerationSeconds,
  maybeParseSvgExportSeconds,
  sampleCountForDuration,
  svgExportFileName,
} from "../export/duration";
import { createSvgExportClient } from "../export/client";
import type { SvgExportRequest } from "../export/messages";

export type AnimatedSvgPaneProps = {
  readonly disabled: boolean;
  readonly sceneId: SceneId;
  readonly onCreateRequest: (durationSeconds: number) => SvgExportRequest | undefined;
};

type ExportPhase =
  | { readonly kind: "idle" }
  | { readonly kind: "running"; readonly completed: number; readonly total: number }
  | { readonly kind: "assembling"; readonly total: number }
  | { readonly kind: "done"; readonly elapsedMs: number }
  | { readonly kind: "failed"; readonly message: string };

const HINT =
  "Generates a looping SVG from the current scene settings. Pointer actions and phone tilt are not included. Particles that appear or disappear can jump between slots.";

/** Seconds field, worker progress, and the finished generation time. */
export function AnimatedSvgPane(props: AnimatedSvgPaneProps) {
  const [secondsDraft, setSecondsDraft] = createSignal(String(DEFAULT_SVG_EXPORT_SECONDS));
  const [phase, setPhase] = createSignal<ExportPhase>({ kind: "idle" });
  const client = createSvgExportClient();
  const maybeSeconds = () => maybeParseSvgExportSeconds(secondsDraft());
  const busy = () => {
    const current = phase();
    return current.kind === "running" || current.kind === "assembling";
  };

  onCleanup(() => client.cancel());

  createEffect(
    on(
      () => props.sceneId,
      () => {
        client.cancel();
        setPhase({ kind: "idle" });
      },
      { defer: true },
    ),
  );

  createEffect(
    on(
      () => props.disabled,
      (disabled) => {
        if (!disabled) {
          return;
        }

        client.cancel();
        setPhase((current) => {
          if (current.kind === "running" || current.kind === "assembling") {
            return { kind: "idle" };
          }
          return current;
        });
      },
      { defer: true },
    ),
  );

  function generate(): void {
    const durationSeconds = maybeSeconds();
    if (durationSeconds === undefined || props.disabled) {
      return;
    }

    const total = sampleCountForDuration(durationSeconds);
    setPhase({ kind: "running", completed: 0, total });
    const maybeRequest = props.onCreateRequest(durationSeconds);
    if (maybeRequest === undefined) {
      setPhase({ kind: "failed", message: "The scene view is not ready to export." });
      return;
    }

    client.start(maybeRequest, {
      onSampling(completed, sampleTotal) {
        setPhase({ kind: "running", completed, total: sampleTotal });
      },
      onAssembling(sampleTotal) {
        setPhase({ kind: "assembling", total: sampleTotal });
      },
      onComplete(result) {
        setPhase({ kind: "done", elapsedMs: result.elapsedMs });
        downloadSvg(svgExportFileName(props.sceneId, durationSeconds), result.svg);
      },
      onError(message) {
        setPhase({ kind: "failed", message });
      },
    });
  }

  function cancel(): void {
    client.cancel();
    setPhase({ kind: "idle" });
  }

  return (
    <section class="svg-export-pane" aria-labelledby="svg-export-title">
      <h3 id="svg-export-title">Animated SVG</h3>
      <div class="svg-export-controls">
        <label class="svg-export-seconds">
          Seconds
          <input
            type="text"
            inputMode="numeric"
            spellcheck={false}
            value={secondsDraft()}
            disabled={busy()}
            onInput={(event) => setSecondsDraft(event.currentTarget.value)}
          />
        </label>
        <button
          class="product-control product-control--accent"
          type="button"
          disabled={props.disabled || busy() || maybeSeconds() === undefined}
          onClick={generate}
        >
          Generate animated SVG
        </button>
        <Show when={busy()}>
          <button class="product-control" type="button" onClick={cancel}>
            Cancel SVG export
          </button>
        </Show>
      </div>
      <Show when={secondsDraft().trim().length > 0 && maybeSeconds() === undefined}>
        <p class="svg-export-problem">Enter a whole number from 1 to 30.</p>
      </Show>
      <Progress phase={phase()} />
      <Show when={phase().kind === "idle"}>
        <p class="svg-export-hint">{HINT}</p>
      </Show>
    </section>
  );
}

function Progress(props: { readonly phase: ExportPhase }) {
  const current = () => props.phase;
  return (
    <Show
      when={
        current().kind === "running" ||
        current().kind === "assembling" ||
        current().kind === "done" ||
        current().kind === "failed"
      }
    >
      <Show when={current().kind === "running" || current().kind === "assembling"}>
        <progress
          class="svg-export-progress"
          max={progressTotal(current())}
          value={progressValue(current())}
          aria-label="Animated SVG progress"
        />
      </Show>
      <p
        class="svg-export-status"
        classList={{ "svg-export-problem": current().kind === "failed" }}
        role="status"
      >
        {progressText(current())}
      </p>
    </Show>
  );
}

function progressTotal(phase: ExportPhase): number {
  if (phase.kind === "running" || phase.kind === "assembling") {
    return phase.total;
  }

  return 1;
}

function progressValue(phase: ExportPhase): number {
  if (phase.kind === "running") {
    return phase.completed;
  }

  if (phase.kind === "assembling") {
    return phase.total;
  }

  return 0;
}

function progressText(phase: ExportPhase): string {
  switch (phase.kind) {
    case "running":
      return `Generating… ${exportProgressPercent(phase.completed, phase.total)}% (${phase.completed} of ${phase.total} samples)`;
    case "assembling":
      return "Assembling SVG…";
    case "done":
      return `Generated in ${formatGenerationSeconds(phase.elapsedMs)}.`;
    case "failed":
      return phase.message;
    case "idle":
      return "";
  }
}

function downloadSvg(fileName: string, svg: string): void {
  const url = URL.createObjectURL(new Blob([svg], { type: "image/svg+xml" }));
  const anchor = document.createElement("a");
  anchor.href = url;
  anchor.download = fileName;
  document.body.append(anchor);
  anchor.click();
  anchor.remove();
  setTimeout(() => URL.revokeObjectURL(url), 60_000);
}
