import { sceneBlobUrl } from "../catalog/links";

export const SCENE_SOURCE_HEADING = "Scene source";
export const IMPLEMENTATION_TERM = "Implementation";
export const IMPLEMENTATION_LINK_LABEL = "View scene source";
export const INSPIRATION_TERM = "Inspiration";
export const NOTICES_TERM = "Notices";
export const NOTICES_LINK_LABEL = "Third-party notices";

const COMMIT_SHA_PATTERN = /\/commit\/([0-9a-f]{40})$/;

/** Extract a host-locked 40-hex SHA from a GitHub commit URL. */
export function maybeCommitSha(
  maybeCommitUrl: string | undefined,
): string | undefined {
  const maybeMatch = maybeCommitUrl?.match(COMMIT_SHA_PATTERN);
  return maybeMatch?.[1];
}

/** Host-locked implementation blob URL; never uses google/liquidfun. */
export function implementationHref(
  path: string,
  maybeSha: string | undefined,
): string {
  return sceneBlobUrl(path, maybeSha);
}
