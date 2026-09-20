const UNKNOWN_HEADING = "Scene not found";
const UNKNOWN_BODY =
  "This playground link does not match a known scene. Open Dam Break, or choose a demo from the navigation list.";
const OPEN_DAM_BREAK = "Open Dam Break";

/** Unknown-hash copy with a hash-only Open Dam Break control. */
export function FallbackPanel() {
  return (
    <section class="fallback-panel" aria-labelledby="player-title">
      <h2 id="player-title">{UNKNOWN_HEADING}</h2>
      <p class="fallback-copy">{UNKNOWN_BODY}</p>
      <a class="product-control" href="#/scene/dam-break">
        {OPEN_DAM_BREAK}
      </a>
    </section>
  );
}
