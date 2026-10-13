import { Match, Show, Switch, createMemo } from "solid-js";

import type { PuzzleId } from "../catalog/puzzles";
import { resultActionsModel, resultCardModel } from "../puzzle/format";
import { runStarsForState, starThresholdsFor } from "../puzzle/hud-format";
import type { PuzzleState } from "../puzzle/reduce-puzzle";
import { starListLabel, starRows } from "../puzzle/star-format";
import type { LevelStars } from "../puzzle/stars";
import { PuzzleStarList } from "./PuzzleStarList";

export type PuzzleResultCardProps = {
  readonly state: PuzzleState;
  readonly maybeNextId: PuzzleId | undefined;
  /** Par from `levelInfo()`; undefined until the level loads (D-13). */
  readonly maybeParTools: number | undefined;
  readonly maybeBestStars: LevelStars | undefined;
  readonly onRetry: () => void;
  readonly onEdit: () => void;
  readonly primaryRef: (element: HTMLElement) => void;
};

const STATUS_LINE_ID = "puzzle-status-line";

/**
 * Fixed result slot (D-10, D-14, D-16): building hints, the status line or
 * goal text with the star checklist; "Filled! Going for Brim" while a won run
 * is deciding; then the outcome, this run's star rows and Next, Retry, Edit.
 * Copy comes only from the pure view models over the committed snapshot
 * (UI-D14); thresholds come from the frame and `levelInfo()`.
 */
export function PuzzleResultCard(props: PuzzleResultCardProps) {
  const card = createMemo(() => resultCardModel(props.state));
  const actions = createMemo(() => resultActionsModel(props.maybeNextId));
  const maybeOutcomeCard = () => {
    const current = card();
    return current.kind === "outcome" ? current : undefined;
  };
  const maybeDecidingCard = () => {
    const current = card();
    return current.kind === "deciding" ? current : undefined;
  };
  const statusTone = () => {
    const current = card();
    return current.kind === "goal" && current.tone === "status";
  };
  const plainMessage = () => {
    const current = card();
    return current.kind === "loading" || current.kind === "goal" ? current.message : "";
  };
  const maybeStarRows = createMemo(() => {
    const maybeParTools = props.maybeParTools;
    if (maybeParTools === undefined) {
      return undefined;
    }
    const maybeThresholds = starThresholdsFor(props.state, maybeParTools);
    if (maybeThresholds === undefined) {
      return undefined;
    }
    const maybeRun = runStarsForState(props.state, maybeParTools);
    return starRows(maybeThresholds, props.maybeBestStars, maybeRun);
  });
  return (
    <section
      class="puzzle-result puzzle-chip"
      data-hud-obstacle
      data-outcome={maybeOutcomeCard()?.outcome}
      data-deciding={maybeDecidingCard() === undefined ? undefined : "true"}
      aria-label="Result"
    >
      <p
        id={statusTone() ? STATUS_LINE_ID : undefined}
        class="puzzle-result-message"
        classList={{ "puzzle-result-message--status": statusTone() }}
        aria-live="polite"
        aria-atomic="true"
      >
        <Switch fallback={plainMessage()}>
          <Match when={maybeOutcomeCard()}>
            {(outcome) => (
              <>
                <span class="puzzle-result-outcome">{outcome().outcomeWord}</span>{" "}
                <span class="puzzle-result-score">{outcome().scoreText}</span>
              </>
            )}
          </Match>
          <Match when={maybeDecidingCard()}>
            {(deciding) => (
              <>
                <span class="puzzle-result-outcome">{deciding().outcomeWord}</span>{" "}
                <span class="puzzle-result-pending">{deciding().pendingText}</span>
              </>
            )}
          </Match>
        </Switch>
      </p>
      <Show when={maybeStarRows()}>
        {(rows) => (
          <PuzzleStarList rows={rows()} label={starListLabel(props.state.kind === "ended")} />
        )}
      </Show>
      <Show when={maybeOutcomeCard()}>
        <div class="puzzle-result-actions">
          <a
            ref={(element) => props.primaryRef(element)}
            class="product-control product-control--accent"
            href={actions().next.href}
            aria-label={actions().next.ariaLabel}
          >
            {actions().next.label}
          </a>
          <button type="button" class="product-control" onClick={() => props.onRetry()}>
            Retry
          </button>
          <button type="button" class="product-control" onClick={() => props.onEdit()}>
            Edit
          </button>
        </div>
      </Show>
    </section>
  );
}
