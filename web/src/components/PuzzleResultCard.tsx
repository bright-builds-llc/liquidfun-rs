import { Show, createMemo } from "solid-js";

import type { PuzzleId } from "../catalog/puzzles";
import { resultActionsModel, resultCardModel } from "../puzzle/format";
import type { PuzzleState } from "../puzzle/reduce-puzzle";

export type PuzzleResultCardProps = {
  readonly state: PuzzleState;
  readonly maybeNextId: PuzzleId | undefined;
  readonly onRetry: () => void;
  readonly onEdit: () => void;
  readonly primaryRef: (element: HTMLElement) => void;
};

/**
 * Fixed result slot (D-14, D-15): goal text before the latch, then the outcome
 * with Next, Retry and Edit. Copy comes only from the pure view models.
 */
export function PuzzleResultCard(props: PuzzleResultCardProps) {
  const card = createMemo(() => resultCardModel(props.state));
  const actions = createMemo(() => resultActionsModel(props.maybeNextId));
  const maybeOutcomeCard = () => {
    const current = card();
    return current.kind === "outcome" ? current : undefined;
  };
  const message = () => {
    const current = card();
    return current.kind === "outcome" ? current.text : current.message;
  };
  const assignPrimary = (primary: "next" | "retry") => (element: HTMLElement) => {
    if (actions().primary === primary) {
      props.primaryRef(element);
    }
  };

  return (
    <section
      class="puzzle-result puzzle-chip"
      data-hud-obstacle
      data-outcome={maybeOutcomeCard()?.outcome}
      aria-label="Result"
    >
      <p class="puzzle-result-message" aria-live="polite" aria-atomic="true">
        <Show when={maybeOutcomeCard()} fallback={message()}>
          {(outcome) => (
            <>
              <span class="puzzle-result-outcome">{outcome().outcomeWord}</span>{" "}
              <span class="puzzle-result-score">{outcome().scoreText}</span>
            </>
          )}
        </Show>
      </p>
      <Show when={maybeOutcomeCard()}>
        <div class="puzzle-result-actions">
          <Show
            when={actions().next.maybeHref}
            fallback={
              <button
                type="button"
                class="product-control"
                disabled
                aria-label={actions().next.ariaLabel}
                title={actions().next.maybeTitle}
              >
                {actions().next.label}
              </button>
            }
          >
            {(href) => (
              <a
                ref={assignPrimary("next")}
                class="product-control"
                classList={{ "product-control--accent": actions().primary === "next" }}
                href={href()}
                aria-label={actions().next.ariaLabel}
              >
                {actions().next.label}
              </a>
            )}
          </Show>
          <button
            ref={assignPrimary("retry")}
            type="button"
            class="product-control"
            classList={{ "product-control--accent": actions().primary === "retry" }}
            onClick={() => props.onRetry()}
          >
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
