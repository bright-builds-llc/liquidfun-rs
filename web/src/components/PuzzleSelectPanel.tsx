import { For, Show } from "solid-js";

import { PROGRESS_NOTE, PUZZLES_INTRO } from "../catalog/puzzle-levels";
import { PUZZLE_IDS } from "../catalog/puzzles";
import type { ProgressDocument } from "../puzzle/progress";
import { levelCardModels, solvedCount, type LevelCardModel } from "../puzzle/progress-view";
import type { RunStars } from "../puzzle/stars";
import { LockIcon, StarIcon } from "./PuzzleIcons";
import { SiteFooter } from "./SiteFooter";
import "../styles/puzzle-select.css";

export type PuzzleSelectPanelProps = {
  readonly canvasStage: boolean;
  readonly progress: ProgressDocument;
  readonly degraded: boolean;
};

/** Star slots in their fixed display order (D-05). */
const STAR_SLOTS: readonly { readonly key: keyof RunStars; readonly name: string }[] = [
  { key: "win", name: "Win" },
  { key: "brim", name: "Brim" },
  { key: "thrifty", name: "Thrifty" },
];

function flag(value: boolean): "true" | "false" {
  return value ? "true" : "false";
}

/** One star slot; `role="img"` makes the glyph and label presentational (UI-SPEC). */
function LevelStarSlot(props: { readonly name: string; readonly starKey: string; readonly earned: boolean }) {
  return (
    <span
      class="puzzle-level-star"
      role="img"
      aria-label={`${props.name}: ${props.earned ? "earned" : "not yet"}`}
      data-star={props.starKey}
      data-earned={flag(props.earned)}
    >
      <StarIcon filled={props.earned} />
      <span class="puzzle-level-star-label">{props.name}</span>
    </span>
  );
}

/** Header, title, caption, stars and footer rows shared by unlocked and locked cards. */
function LevelCardBody(props: { readonly card: LevelCardModel }) {
  return (
    <>
      <div class="puzzle-level-card-header">
        <span class="puzzle-level-eyebrow">{`Level ${props.card.number}`}</span>
        <Show when={props.card.isNextUp}>
          <span class="puzzle-level-tag puzzle-level-tag--next">Up next</span>
        </Show>
        <Show when={props.card.locked}>
          <span class="puzzle-level-tag puzzle-level-tag--locked">
            <LockIcon />
            Locked
          </span>
        </Show>
      </div>
      <h3 class="puzzle-level-title">{props.card.title}</h3>
      <p class="puzzle-level-caption">{props.card.caption}</p>
      <div class="puzzle-level-stars">
        <For each={STAR_SLOTS}>
          {(slot) => (
            <LevelStarSlot name={slot.name} starKey={slot.key} earned={props.card.stars[slot.key]} />
          )}
        </For>
      </div>
      <p class="puzzle-level-footer">{props.card.footer}</p>
    </>
  );
}

/** Whole-card link when unlocked; a non-focusable named group when locked (D-06). */
function LevelCard(props: { readonly card: LevelCardModel }) {
  const lockedCard = () => (
    <div
      class="puzzle-level-card puzzle-level-card--locked"
      role="group"
      aria-label={props.card.lockedLabel}
      data-level-id={props.card.id}
      data-locked="true"
      data-solved={flag(props.card.solved)}
      data-next-up={flag(props.card.isNextUp)}
    >
      <LevelCardBody card={props.card} />
    </div>
  );

  return (
    <Show when={!props.card.locked} fallback={lockedCard()}>
      <a
        class="puzzle-level-card"
        href={props.card.href}
        data-level-id={props.card.id}
        data-locked="false"
        data-solved={flag(props.card.solved)}
        data-next-up={flag(props.card.isNextUp)}
      >
        <LevelCardBody card={props.card} />
      </a>
    </Show>
  );
}

/** `#/puzzles` level grid (D-05..D-07). Renders only TS metadata and persisted progress; never touches wasm. */
export function PuzzleSelectPanel(props: PuzzleSelectPanelProps) {
  const cards = () => levelCardModels(props.progress, props.degraded);

  return (
    <section
      class="puzzle-select"
      classList={{ "puzzle-select--canvas-stage": props.canvasStage }}
      aria-labelledby="puzzle-select-title"
      data-progress-degraded={flag(props.degraded)}
    >
      <div class="puzzle-select-content">
        <header class="puzzle-select-header">
          <h2 id="puzzle-select-title">Puzzles</h2>
          <p class="puzzle-select-intro">{PUZZLES_INTRO}</p>
          <p class="puzzle-select-progress">
            {`${solvedCount(props.progress)} of ${PUZZLE_IDS.length} solved`}
          </p>
          <Show when={props.degraded}>
            <p id="puzzle-progress-note" class="puzzle-progress-note" data-progress-note>
              {PROGRESS_NOTE}
            </p>
          </Show>
        </header>
        <ol class="puzzle-level-grid" aria-label="Levels">
          <For each={cards()}>
            {(card) => (
              <li>
                <LevelCard card={card} />
              </li>
            )}
          </For>
        </ol>
      </div>
      <Show when={props.canvasStage}>
        <SiteFooter />
      </Show>
    </section>
  );
}
