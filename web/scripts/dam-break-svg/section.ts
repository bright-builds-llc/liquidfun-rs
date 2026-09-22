import { DAM_BREAK_SVG_REPO_PATH } from "./request";

export const DAM_BREAK_SVG_BEGIN = "<!-- dam-break-animated-svg:begin -->";
export const DAM_BREAK_SVG_END = "<!-- dam-break-animated-svg:end -->";

const README_INSERT_ANCHOR = "## Hobby development";

/** README block that embeds the committed Dam Break loop. */
export function damBreakAnimatedSvgSection(): string {
  return [
    DAM_BREAK_SVG_BEGIN,
    "",
    "### Dam Break animated SVG",
    "",
    "A 10 second loop of the default Dam Break scene from the playground's animated",
    "SVG export. The clip uses medium water, normal gravity, the identity camera,",
    "wireframe rendering, and the demo gallery's 1280 by 960 frame. A post-merge",
    "workflow regenerates the file and does not commit when the export matches",
    "this copy.",
    "",
    `[![Dam Break 10 second animated SVG](${DAM_BREAK_SVG_REPO_PATH})](${DAM_BREAK_SVG_REPO_PATH})`,
    "",
    DAM_BREAK_SVG_END,
  ].join("\n");
}

/**
 * Inserts or replaces the Dam Break SVG section.
 *
 * The block stays at the end of the web playground section, immediately
 * above Hobby development. An existing marked block is removed first so a
 * later run can correct an older location without duplicating the section.
 */
export function upsertDamBreakAnimatedSvgSection(readme: string): string {
  const section = damBreakAnimatedSvgSection();
  const withoutSection = removeMarkedSection(readme);
  return insertSection(withoutSection, section);
}

function removeMarkedSection(readme: string): string {
  const begin = readme.indexOf(DAM_BREAK_SVG_BEGIN);
  const end = readme.indexOf(DAM_BREAK_SVG_END);
  if (begin === -1 && end === -1) {
    return readme;
  }

  if (begin === -1 || end === -1 || end < begin) {
    throw new Error("Dam Break animated SVG markers are incomplete.");
  }

  const close = end + DAM_BREAK_SVG_END.length;
  const before = readme.slice(0, begin).replace(/\n+$/, "\n");
  const after = readme.slice(close).replace(/^\n+/, "\n");
  return `${before}${after}`;
}

function insertSection(readme: string, section: string): string {
  const anchorAt = readme.indexOf(README_INSERT_ANCHOR);
  if (anchorAt === -1) {
    throw new Error(
      "README is missing the Hobby development heading used to place the Dam Break SVG section.",
    );
  }

  const prefix = readme.slice(0, anchorAt).replace(/\s*$/, "\n\n");
  return `${prefix}${section}\n\n${readme.slice(anchorAt)}`;
}
