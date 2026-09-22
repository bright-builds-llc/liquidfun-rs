import { Show } from "solid-js";

import { readBuildInfo } from "../build-info";
import { PAGE_SUMMARY } from "../player/runtime";

const SOURCE_HREF = "https://github.com/bright-builds-llc/liquidfun-rs";
const MAINTAINER_HREF = "https://openlinks.us/";
const SOURCE_LABEL = "View source on GitHub";
const LICENSE_LABEL = "Free and open source";
const MAINTAINER_LABEL = "By Peter Ryszkiewicz";

type ProvenanceLinkProps = {
  readonly label: string;
  readonly maybeUrl: string | undefined;
};

type BuiltAtValueProps = {
  readonly label: string;
  readonly maybeIso: string | undefined;
};

function BuiltAtValue(props: BuiltAtValueProps) {
  return (
    <Show when={props.maybeIso} fallback={props.label}>
      {(iso) => <time datetime={iso()}>{props.label}</time>}
    </Show>
  );
}

function ProvenanceValue(props: ProvenanceLinkProps) {
  return (
    <Show
      when={props.maybeUrl}
      fallback={props.label}
    >
      {(url) => (
        <a href={url()} target="_blank" rel="noopener noreferrer">
          {props.label}
        </a>
      )}
    </Show>
  );
}

/** Site-level source, FOSS, maintainer, and provenance chrome. */
export function SiteFooter() {
  const info = readBuildInfo();

  return (
    <footer class="site-footer">
      <p class="site-footer-summary">{PAGE_SUMMARY}</p>
      <div class="site-footer-identity">
        <a href={SOURCE_HREF} target="_blank" rel="noopener noreferrer">
          {SOURCE_LABEL}
        </a>
        <span>{LICENSE_LABEL}</span>
        <a href={MAINTAINER_HREF} target="_blank" rel="noopener noreferrer">
          {MAINTAINER_LABEL}
        </a>
      </div>
      <dl class="site-footer-provenance">
        <div>
          <dt>Version</dt>
          <dd>{info.version}</dd>
        </div>
        <div>
          <dt>Commit</dt>
          <dd>
            <ProvenanceValue
              label={info.commitLabel}
              maybeUrl={info.maybeCommitUrl}
            />
          </dd>
        </div>
        <div>
          <dt>Build</dt>
          <dd>
            <ProvenanceValue
              label={info.buildLabel}
              maybeUrl={info.maybeBuildUrl}
            />
          </dd>
        </div>
        <div>
          <dt>Built at</dt>
          <dd>
            <BuiltAtValue
              label={info.builtAtLabel}
              maybeIso={info.maybeBuiltAtIso}
            />
          </dd>
        </div>
      </dl>
    </footer>
  );
}
