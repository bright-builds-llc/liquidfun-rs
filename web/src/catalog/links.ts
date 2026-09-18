const FULL_SHA_PATTERN = /^[0-9a-f]{40}$/;
const SCENE_PATH_PREFIX = "crates/liquidfun-wasm/src/scene/";
const NOTICES_PATH = "THIRD_PARTY_NOTICES.md";
const ALLOWLIST_MESSAGE = "Scene source path is not allowlisted";

export const REPO_BLOB_ORIGIN =
  "https://github.com/bright-builds-llc/liquidfun-rs/blob";

function blobRef(maybeSha: string | undefined): string {
  const maybeTrimmed = maybeSha?.trim();
  if (
    maybeTrimmed === undefined ||
    maybeTrimmed.length === 0 ||
    !FULL_SHA_PATTERN.test(maybeTrimmed)
  ) {
    return "main";
  }

  return maybeTrimmed;
}

function isAllowlistedScenePath(path: string): boolean {
  if (path.includes("..") || path.startsWith("/")) {
    return false;
  }

  if (!path.startsWith(SCENE_PATH_PREFIX) || !path.endsWith(".rs")) {
    return false;
  }

  const fileName = path.slice(SCENE_PATH_PREFIX.length);
  return fileName.length > 0 && !fileName.includes("/") && !fileName.includes("\\");
}

/** Host-locked GitHub blob URL for an allowlisted WASM scene module. */
export function sceneBlobUrl(
  path: string,
  maybeSha: string | undefined,
): string {
  if (!isAllowlistedScenePath(path)) {
    throw new Error(ALLOWLIST_MESSAGE);
  }

  return `${REPO_BLOB_ORIGIN}/${blobRef(maybeSha)}/${path}`;
}

/** Host-locked GitHub blob URL for repository third-party notices. */
export function noticesBlobUrl(maybeSha: string | undefined): string {
  return `${REPO_BLOB_ORIGIN}/${blobRef(maybeSha)}/${NOTICES_PATH}`;
}
