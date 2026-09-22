const REVEAL_FRAME_LIMIT = 12;

export type SidebarScrollTarget = {
  readonly containerClientHeight: number;
  readonly containerScrollHeight: number;
  readonly itemOffset: number;
  readonly itemHeight: number;
};

/**
 * Scroll offset that centers `item` in the sidebar viewport.
 *
 * An item taller than the viewport aligns to its start. The result stays
 * inside the container's scroll range.
 */
export function scrollTopToCenterItem(target: SidebarScrollTarget): number {
  const maxScroll = Math.max(
    0,
    target.containerScrollHeight - target.containerClientHeight,
  );
  if (target.containerClientHeight <= 0) {
    return 0;
  }

  const unclamped =
    target.itemHeight >= target.containerClientHeight
      ? target.itemOffset
      : target.itemOffset -
        (target.containerClientHeight - target.itemHeight) / 2;
  return clamp(unclamped, 0, maxScroll);
}

/**
 * Scroll the sidebar so its current link is centered once laid out.
 *
 * The mobile drawer focuses its first link, and that focus scrolls the list
 * back to the top. Reapply across a few frames so the current link wins.
 */
export function scheduleSidebarReveal(navigation: HTMLElement): () => void {
  let frame = 0;
  let cancelled = false;
  let applying = false;
  let maybeRequest: number | undefined;
  let maybeContainer: HTMLElement | undefined;

  const stopListening = () => {
    maybeContainer?.removeEventListener("scroll", onScroll);
    maybeContainer = undefined;
  };

  const onScroll = () => {
    if (applying || cancelled) {
      return;
    }
    applying = true;
    revealCurrentSidebarLink(navigation);
    applying = false;
  };

  const attempt = () => {
    maybeRequest = undefined;
    if (cancelled) {
      return;
    }
    if (navigation.isConnected) {
      applying = true;
      const container = revealCurrentSidebarLink(navigation);
      applying = false;
      if (container !== undefined && container !== maybeContainer) {
        stopListening();
        maybeContainer = container;
        container.addEventListener("scroll", onScroll);
      }
    }
    frame += 1;
    if (frame >= REVEAL_FRAME_LIMIT) {
      stopListening();
      return;
    }
    maybeRequest = requestAnimationFrame(attempt);
  };

  attempt();

  return () => {
    cancelled = true;
    stopListening();
    if (maybeRequest !== undefined) {
      cancelAnimationFrame(maybeRequest);
    }
  };
}

function revealCurrentSidebarLink(
  navigation: HTMLElement,
): HTMLElement | undefined {
  const container = navigation.closest("[data-slot='sidebar-content']");
  if (!(container instanceof HTMLElement) || container.clientHeight <= 0) {
    return undefined;
  }

  const current = navigation.querySelector("[aria-current='page']");
  if (!(current instanceof HTMLElement) || current.offsetHeight <= 0) {
    return container;
  }

  const itemOffset =
    current.getBoundingClientRect().top -
    container.getBoundingClientRect().top +
    container.scrollTop;
  container.scrollTop = scrollTopToCenterItem({
    containerClientHeight: container.clientHeight,
    containerScrollHeight: container.scrollHeight,
    itemOffset,
    itemHeight: current.offsetHeight,
  });
  return container;
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}
