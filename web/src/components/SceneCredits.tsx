import { For } from "solid-js";

import { noticesBlobUrl } from "../catalog/links";
import { readBuildInfo } from "../build-info";
import {
  IMPLEMENTATION_TERM,
  INSPIRATION_TERM,
  NOTICES_TERM,
  SCENE_SOURCE_HEADING,
  implementationHref,
  maybeCommitSha,
} from "./scene-credits";

export {
  IMPLEMENTATION_LINK_LABEL,
  IMPLEMENTATION_TERM,
  INSPIRATION_TERM,
  NOTICES_LINK_LABEL,
  NOTICES_TERM,
  SCENE_SOURCE_HEADING,
  implementationHref,
} from "./scene-credits";

export type SceneCreditsProps = {
  readonly implementationPath: string;
  readonly inspiration: readonly {
    readonly label: string;
    readonly href: string;
  }[];
};

/** Per-scene implementation, inspiration, and notices beside SiteFooter. */
export function SceneCredits(props: SceneCreditsProps) {
  const info = readBuildInfo();
  const maybeSha = maybeCommitSha(info.maybeCommitUrl);
  const implementationUrl = implementationHref(
    props.implementationPath,
    maybeSha,
  );
  const noticesUrl = noticesBlobUrl(maybeSha);

  return (
    <aside class="scene-credits" aria-labelledby="scene-credits-title">
      <p id="scene-credits-title" class="scene-credits-heading">
        {SCENE_SOURCE_HEADING}
      </p>
      <dl class="scene-credits-list">
        <div>
          <dt>{IMPLEMENTATION_TERM}</dt>
          <dd>
            <a
              href={implementationUrl}
              target="_blank"
              rel="noopener noreferrer"
            >
              View scene source
            </a>
          </dd>
        </div>
        <div>
          <dt>{INSPIRATION_TERM}</dt>
          <dd>
            <For each={props.inspiration}>
              {(item) => (
                <a href={item.href} target="_blank" rel="noopener noreferrer">
                  {item.label}
                </a>
              )}
            </For>
          </dd>
        </div>
        <div>
          <dt>{NOTICES_TERM}</dt>
          <dd>
            <a href={noticesUrl} target="_blank" rel="noopener noreferrer">
              Third-party notices
            </a>
          </dd>
        </div>
      </dl>
    </aside>
  );
}
