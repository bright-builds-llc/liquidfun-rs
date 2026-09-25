// Adapted from https://github.com/hngngn/shadcn-solid (MIT).

import { createSignal, onCleanup } from "solid-js";
import { isServer } from "solid-js/web";

import { readCanvasStage } from "../player/fullscreen-capability";

const MOBILE_QUERY = "(max-width: 767px)";

/** True when the viewport matches the sidebar's phone drawer breakpoint. */
export const useIsMobile = () => {
  if (isServer) {
    return () => false;
  }

  const mql = window.matchMedia(MOBILE_QUERY);
  const [state, setState] = createSignal(mql.matches);
  const update = () => setState(mql.matches);
  mql.addEventListener("change", update);
  onCleanup(() => {
    mql.removeEventListener("change", update);
  });
  return () => state() || readCanvasStage(document.fullscreenEnabled, navigator);
};
