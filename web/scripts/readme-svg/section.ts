import { maybeSceneById } from "../../src/catalog/scenes";
import { readmeSvgRepoPath, type ReadmeSvgPlan } from "./plans";

export const README_SVG_BEGIN = "<!-- readme-svg-gallery:begin -->";
export const README_SVG_END = "<!-- readme-svg-gallery:end -->";

const RETIRED_DAM_BREAK_BEGIN = "<!-- dam-break-animated-svg:begin -->";
const RETIRED_DAM_BREAK_END = "<!-- dam-break-animated-svg:end -->";
const GALLERY_HEADING = "### Demo gallery\n";
const GALLERY_FOLLOWING = "\nThe private, unpublished";
const LIVE_SCENE_ORIGIN = "https://bright-builds-llc.github.io/liquidfun-rs/#/scene/";

/**
 * Marked demo-gallery body for every README scene plan.
 *
 * The heading stays outside the markers. Image links open the live scene.
 */
export function readmeSvgGalleryBody(plans: readonly ReadmeSvgPlan[]): string {
  const entries = plans.map((plan) => sceneEntry(plan));
  return [
    README_SVG_BEGIN,
    "",
    "Each preview is a 10 second looping animated SVG. The frame is 1280 by 960,",
    "wireframe, with the identity camera. Scenes that stay still until you act",
    "include one cue. Select a title or preview to open the live scene. A",
    "post-merge workflow regenerates these files and commits them when the",
    "bytes change.",
    "",
    entries.join("\n\n"),
    "",
    README_SVG_END,
  ].join("\n");
}

/**
 * Replaces the demo gallery with the scene SVGs.
 *
 * A retired Dam Break-only block is removed so the gallery is the one README
 * embed. An existing marked gallery is replaced in place. Otherwise the
 * legacy gallery under `### Demo gallery` is replaced through the line before
 * the WASM install paragraph.
 */
export function upsertReadmeSvgGallery(
  readme: string,
  plans: readonly ReadmeSvgPlan[],
): string {
  const body = readmeSvgGalleryBody(plans);
  const withoutRetired = removeMarkedSection(
    readme,
    RETIRED_DAM_BREAK_BEGIN,
    RETIRED_DAM_BREAK_END,
    "Dam Break animated SVG markers are incomplete.",
  );
  if (withoutRetired.includes(README_SVG_BEGIN) || withoutRetired.includes(README_SVG_END)) {
    return replaceMarkedSection(withoutRetired, body);
  }

  return replaceLegacyGallery(withoutRetired, body);
}

function sceneEntry(plan: ReadmeSvgPlan): string {
  const maybeScene = maybeSceneById(plan.id);
  if (maybeScene === undefined) {
    throw new Error(`README SVG plan ${plan.id} is missing from the scene catalog.`);
  }

  const liveUrl = `${LIVE_SCENE_ORIGIN}${plan.id}`;
  const svgPath = readmeSvgRepoPath(plan.id);
  const heading = `#### [${maybeScene.title}](${liveUrl})`;
  const image = `[![${maybeScene.title} simulation preview](${svgPath})](${liveUrl})`;
  return `${heading}\n\n${image}`;
}

function replaceMarkedSection(readme: string, body: string): string {
  const withoutBody = removeMarkedSection(
    readme,
    README_SVG_BEGIN,
    README_SVG_END,
    "README SVG gallery markers are incomplete.",
  );
  const headingAt = withoutBody.indexOf(GALLERY_HEADING);
  if (headingAt === -1) {
    throw new Error("README is missing the demo gallery heading.");
  }

  const insertAt = headingAt + GALLERY_HEADING.length;
  const prefix = withoutBody.slice(0, insertAt).replace(/\n*$/, "\n\n");
  const suffix = withoutBody.slice(insertAt).replace(/^\n*/, "");
  return `${prefix}${body}\n\n${suffix}`;
}

function replaceLegacyGallery(readme: string, body: string): string {
  const headingAt = readme.indexOf(GALLERY_HEADING);
  const followingAt = readme.indexOf(GALLERY_FOLLOWING);
  if (headingAt === -1 || followingAt === -1 || followingAt < headingAt) {
    throw new Error(
      "README is missing the demo gallery heading or the install paragraph that follows it.",
    );
  }

  const prefix = readme.slice(0, headingAt + GALLERY_HEADING.length).replace(/\n*$/, "\n\n");
  const suffix = readme.slice(followingAt + 1);
  return `${prefix}${body}\n\n${suffix}`;
}

function removeMarkedSection(
  readme: string,
  begin: string,
  end: string,
  incompleteMessage: string,
): string {
  const beginAt = readme.indexOf(begin);
  const endAt = readme.indexOf(end);
  if (beginAt === -1 && endAt === -1) {
    return readme;
  }

  if (beginAt === -1 || endAt === -1 || endAt < beginAt) {
    throw new Error(incompleteMessage);
  }

  const close = endAt + end.length;
  const before = readme.slice(0, beginAt).replace(/\n+$/, "\n");
  const after = readme.slice(close).replace(/^\n+/, "\n");
  return `${before}${after}`;
}
