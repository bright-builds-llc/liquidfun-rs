import type { JSX } from "solid-js";

import { PAGE_SUMMARY } from "../player/runtime";

const SOURCE_HREF = "https://github.com/bright-builds-llc/liquidfun-rs";

export type SiteHeaderProps = {
  readonly mobileNavigationTrigger: JSX.Element;
};

/** Static project identity and source chrome with a composed mobile trigger. */
export function SiteHeader(props: SiteHeaderProps) {
  return (
    <header class="site-header">
      <div class="site-header-copy">
        <h1 id="site-title">liquidfun-rs</h1>
        <p>{PAGE_SUMMARY}</p>
      </div>
      <div class="site-header-actions">
        <a
          class="site-header-source"
          href={SOURCE_HREF}
          target="_blank"
          rel="noopener noreferrer"
        >
          GitHub source
        </a>
        {props.mobileNavigationTrigger}
      </div>
    </header>
  );
}
