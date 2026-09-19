const EMPTY_HEADING = "Choose a scene";
const EMPTY_BODY =
  "Choose a demo from the navigation list to open it in the shared player. Each scene is a live Rust simulation.";
const UNKNOWN_HEADING = "Scene not found";
const UNKNOWN_BODY =
  "This playground link does not match a known scene. Open Dam Break, or choose a demo from the navigation list.";
const NOT_READY_BODY =
  "This demo is listed in the navigation and has no physics yet. Open Dam Break to play the first working scene.";
const OPEN_DAM_BREAK = "Open Dam Break";

export type FallbackKind = "empty" | "unknown" | "not-ready";

export type FallbackPanelProps =
  | { readonly kind: "empty" }
  | { readonly kind: "unknown" }
  | { readonly kind: "not-ready"; readonly sceneTitle: string };

function fallbackCopy(props: FallbackPanelProps): {
  readonly heading: string;
  readonly body: string;
} {
  if (props.kind === "empty") {
    return { heading: EMPTY_HEADING, body: EMPTY_BODY };
  }

  if (props.kind === "unknown") {
    return { heading: UNKNOWN_HEADING, body: UNKNOWN_BODY };
  }

  return {
    heading: `${props.sceneTitle} is not ready yet`,
    body: NOT_READY_BODY,
  };
}

/** Empty, unknown, and not-ready copy with a hash-only Open Dam Break control. */
export function FallbackPanel(props: FallbackPanelProps) {
  const copy = () => fallbackCopy(props);

  return (
    <section class="fallback-panel" aria-labelledby="player-title">
      <h2 id="player-title">{copy().heading}</h2>
      <p class="fallback-copy">{copy().body}</p>
      <a class="product-control" href="#/scene/dam-break">
        {OPEN_DAM_BREAK}
      </a>
    </section>
  );
}
