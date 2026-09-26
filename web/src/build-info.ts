export type BuildInfo = {
  readonly version: string;
  readonly commitLabel: string;
  readonly maybeCommitUrl: string | undefined;
  readonly buildLabel: string;
  readonly maybeBuildUrl: string | undefined;
  readonly builtAtLabel: string;
  readonly maybeBuiltAtIso: string | undefined;
};

export type BuildInfoEnv = {
  readonly VITE_APP_VERSION?: string;
  readonly VITE_GIT_SHA?: string;
  readonly VITE_BUILD_ID?: string;
  readonly VITE_BUILD_URL?: string;
  readonly VITE_BUILT_AT?: string;
};

const UNAVAILABLE = "Unavailable";
const REPO_ORIGIN = "https://github.com/bright-builds-llc/liquidfun-rs";
const FULL_SHA_PATTERN = /^[0-9a-f]{40}$/;
const BUILD_URL_PATTERN =
  /^https:\/\/github\.com\/bright-builds-llc\/liquidfun-rs\/actions\/runs\/\d+$/;
const COMMIT_LABEL_LENGTH = 12;
const BUILT_AT_PATTERN =
  /^(\d{4})-(\d{2})-(\d{2})T(\d{2}):(\d{2}):(\d{2})(?:\.\d{1,3})?Z$/;
const UNAVAILABLE_BUILT_AT = {
  builtAtLabel: UNAVAILABLE,
  maybeBuiltAtIso: undefined,
} as const;

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

function readBuiltAt(maybeValue: string | undefined): {
  readonly builtAtLabel: string;
  readonly maybeBuiltAtIso: string | undefined;
} {
  const maybeCandidate = maybeTrimmed(maybeValue);
  const match =
    maybeCandidate === undefined ? null : BUILT_AT_PATTERN.exec(maybeCandidate);
  const yearText = match?.[1];
  const monthText = match?.[2];
  const dayText = match?.[3];
  const hourText = match?.[4];
  const minuteText = match?.[5];
  const secondText = match?.[6];
  if (
    maybeCandidate === undefined ||
    yearText === undefined ||
    monthText === undefined ||
    dayText === undefined ||
    hourText === undefined ||
    minuteText === undefined ||
    secondText === undefined
  ) {
    return UNAVAILABLE_BUILT_AT;
  }

  const year = Number(yearText);
  const month = Number(monthText);
  const day = Number(dayText);
  const hour = Number(hourText);
  const minute = Number(minuteText);
  const second = Number(secondText);
  const parsed = new Date(Date.UTC(year, month - 1, day, hour, minute, second));
  if (
    parsed.getUTCFullYear() !== year ||
    parsed.getUTCMonth() !== month - 1 ||
    parsed.getUTCDate() !== day ||
    parsed.getUTCHours() !== hour ||
    parsed.getUTCMinutes() !== minute ||
    parsed.getUTCSeconds() !== second
  ) {
    return UNAVAILABLE_BUILT_AT;
  }

  return {
    builtAtLabel: `${yearText}-${monthText}-${dayText} ${hourText}:${minuteText}:${secondText} UTC`,
    maybeBuiltAtIso: maybeCandidate,
  };
}

function maybeBuiltAtPart(
  parts: readonly Intl.DateTimeFormatPart[],
  type: Intl.DateTimeFormatPartTypes,
): string | undefined {
  return parts.find((part) => part.type === type)?.value;
}

/**
 * Format a validated build instant for display.
 * Omit `maybeTimeZone` to use the runtime local zone.
 */
export function formatLocalBuiltAtLabel(
  iso: string,
  maybeTimeZone?: string,
): string {
  const parsed = new Date(iso);
  if (Number.isNaN(parsed.getTime())) {
    return UNAVAILABLE;
  }

  const options: Intl.DateTimeFormatOptions = {
    year: "numeric",
    month: "2-digit",
    day: "2-digit",
    hour: "2-digit",
    minute: "2-digit",
    second: "2-digit",
    hourCycle: "h23",
    timeZoneName: "short",
  };
  if (maybeTimeZone !== undefined) {
    options.timeZone = maybeTimeZone;
  }

  let parts: Intl.DateTimeFormatPart[];
  try {
    parts = new Intl.DateTimeFormat("en-US", options).formatToParts(parsed);
  } catch (error) {
    if (error instanceof RangeError) {
      return UNAVAILABLE;
    }
    throw error;
  }

  const year = maybeBuiltAtPart(parts, "year");
  const month = maybeBuiltAtPart(parts, "month");
  const day = maybeBuiltAtPart(parts, "day");
  const hour = maybeBuiltAtPart(parts, "hour");
  const minute = maybeBuiltAtPart(parts, "minute");
  const second = maybeBuiltAtPart(parts, "second");
  const zone = maybeBuiltAtPart(parts, "timeZoneName");
  if (
    year === undefined ||
    month === undefined ||
    day === undefined ||
    hour === undefined ||
    minute === undefined ||
    second === undefined ||
    zone === undefined
  ) {
    return UNAVAILABLE;
  }

  return `${year}-${month}-${day} ${hour}:${minute}:${second} ${zone}`;
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
  const builtAt = readBuiltAt(source.VITE_BUILT_AT);
  return {
    version: maybeTrimmed(source.VITE_APP_VERSION) ?? UNAVAILABLE,
    commitLabel: commit.commitLabel,
    maybeCommitUrl: commit.maybeCommitUrl,
    buildLabel: maybeTrimmed(source.VITE_BUILD_ID) ?? UNAVAILABLE,
    maybeBuildUrl: maybeAcceptedBuildUrl(source.VITE_BUILD_URL),
    builtAtLabel: builtAt.builtAtLabel,
    maybeBuiltAtIso: builtAt.maybeBuiltAtIso,
  };
}
