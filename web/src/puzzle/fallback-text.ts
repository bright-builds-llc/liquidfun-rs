/** Longest unknown level id shown verbatim in the fallback panel. */
export const MAX_RAW_DISPLAY_LENGTH = 80;

/** Display form of an unknown level id: text only, at most 80 characters plus an ellipsis. */
export function truncateRaw(raw: string): string {
  if (raw.length <= MAX_RAW_DISPLAY_LENGTH) {
    return raw;
  }

  return `${raw.slice(0, MAX_RAW_DISPLAY_LENGTH)}…`;
}
