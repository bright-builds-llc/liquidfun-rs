export type BuildInfo = {
  readonly version: string;
  readonly commitLabel: string;
  readonly maybeCommitUrl: string | undefined;
  readonly buildLabel: string;
  readonly maybeBuildUrl: string | undefined;
};

export type BuildInfoEnv = {
  readonly VITE_APP_VERSION?: string;
  readonly VITE_GIT_SHA?: string;
  readonly VITE_BUILD_ID?: string;
  readonly VITE_BUILD_URL?: string;
};

const UNAVAILABLE = "Unavailable";
const REPO_ORIGIN = "https://github.com/bright-builds-llc/liquidfun-rs";
const FULL_SHA_PATTERN = /^[0-9a-f]{40}$/;
const BUILD_URL_PATTERN =
  /^https:\/\/github\.com\/bright-builds-llc\/liquidfun-rs\/actions\/runs\/\d+$/;
const COMMIT_LABEL_LENGTH = 12;

function maybeTrimmed(maybeValue: string | undefined): string | undefined {
  if (maybeValue === undefined) {
    return undefined;
  }

  const trimmed = maybeValue.trim();
  return trimmed.length === 0 ? undefined : trimmed;
}

function commitFromSha(maybeSha: string | undefined): {
  readonly commitLabel: string;
  readonly maybeCommitUrl: string | undefined;
} {
  const maybeFullSha = maybeTrimmed(maybeSha);
  if (maybeFullSha === undefined || !FULL_SHA_PATTERN.test(maybeFullSha)) {
    return { commitLabel: UNAVAILABLE, maybeCommitUrl: undefined };
  }

  return {
    commitLabel: maybeFullSha.slice(0, COMMIT_LABEL_LENGTH),
    maybeCommitUrl: `${REPO_ORIGIN}/commit/${maybeFullSha}`,
  };
}

function maybeAcceptedBuildUrl(
  maybeUrl: string | undefined,
): string | undefined {
  const maybeCandidate = maybeTrimmed(maybeUrl);
  if (maybeCandidate === undefined || !BUILD_URL_PATTERN.test(maybeCandidate)) {
    return undefined;
  }

  return maybeCandidate;
}

/** Read injected Vite provenance, showing Unavailable for missing fields. */
export function readBuildInfo(maybeEnv?: BuildInfoEnv): BuildInfo {
  const source = maybeEnv ?? import.meta.env;
  const commit = commitFromSha(maybeTrimmed(source.VITE_GIT_SHA));
  return {
    version: maybeTrimmed(source.VITE_APP_VERSION) ?? UNAVAILABLE,
    commitLabel: commit.commitLabel,
    maybeCommitUrl: commit.maybeCommitUrl,
    buildLabel: maybeTrimmed(source.VITE_BUILD_ID) ?? UNAVAILABLE,
    maybeBuildUrl: maybeAcceptedBuildUrl(source.VITE_BUILD_URL),
  };
}
