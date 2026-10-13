import type { LevelStars, RunStars } from "./stars";

/** One star criterion, in the fixed row order Win, Brim, Thrifty. */
export type StarCriterion = "win" | "brim" | "thrifty";

/** Star glyph state (UI-D12): filled, amber outline or muted outline. */
export type StarGlyphState = "earned" | "best" | "none";

/** Level thresholds shown in the rows; whole percents and par come from `levelInfo()` (D-13). */
export type StarThresholds = {
  readonly targetPct: number;
  readonly brimPct: number;
  readonly parTools: number;
};

/** One star row: visible label, visually hidden full text and glyph state (D-16). */
export type StarRowModel = {
  readonly criterion: StarCriterion;
  readonly label: string;
  readonly hiddenText: string;
  readonly glyph: StarGlyphState;
};

const STAR_CRITERIA: readonly StarCriterion[] = ["win", "brim", "thrifty"];

function toolCount(parTools: number): string {
  return `${parTools} ${parTools === 1 ? "tool" : "tools"}`;
}

function rowLabel(criterion: StarCriterion, thresholds: StarThresholds): string {
  switch (criterion) {
    case "win":
      return `Win ${thresholds.targetPct}%`;
    case "brim":
      return `Brim ${thresholds.brimPct}%`;
    case "thrifty":
      return `Thrifty ≤ ${toolCount(thresholds.parTools)}`;
    default: {
      const unreachable: never = criterion;
      return unreachable;
    }
  }
}

function rowDescription(criterion: StarCriterion, thresholds: StarThresholds): string {
  switch (criterion) {
    case "win":
      return `Win: fill the cup to ${thresholds.targetPct}%.`;
    case "brim":
      return `Brim: fill the cup to ${thresholds.brimPct}%.`;
    case "thrifty":
      return `Thrifty: win using at most ${toolCount(thresholds.parTools)}.`;
    default: {
      const unreachable: never = criterion;
      return unreachable;
    }
  }
}

type RowState = { readonly glyph: StarGlyphState; readonly phrase: string };

function rowState(
  criterion: StarCriterion,
  maybeBest: LevelStars | undefined,
  maybeRun: RunStars | undefined,
): RowState {
  const inBest = maybeBest?.[criterion] === true;
  if (maybeRun === undefined) {
    return inBest
      ? { glyph: "earned", phrase: "Earned." }
      : { glyph: "none", phrase: "Not earned yet." };
  }

  if (maybeRun[criterion]) {
    return { glyph: "earned", phrase: "Earned this run." };
  }

  return inBest
    ? { glyph: "best", phrase: "Earned before." }
    : { glyph: "none", phrase: "Not earned." };
}

/** `maybeRun` undefined = building (glyph from best-of); defined = running/ended (UI-D12). */
export function starRows(
  thresholds: StarThresholds,
  maybeBest: LevelStars | undefined,
  maybeRun: RunStars | undefined,
): readonly StarRowModel[] {
  return STAR_CRITERIA.map((criterion) => {
    const state = rowState(criterion, maybeBest, maybeRun);
    return {
      criterion,
      label: rowLabel(criterion, thresholds),
      hiddenText: `${rowDescription(criterion, thresholds)} ${state.phrase}`,
      glyph: state.glyph,
    };
  });
}

/** Accessible name of the star list: the checklist before the end, this run's rows after. */
export function starListLabel(ended: boolean): "Stars" | "Stars this run" {
  return ended ? "Stars this run" : "Stars";
}
