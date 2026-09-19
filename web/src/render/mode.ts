/** Supported browser presentation modes. */
export type RenderMode = "wireframe" | "solid";

export const DEFAULT_RENDER_MODE: RenderMode = "wireframe";
export const RENDER_MODE_STORAGE_KEY = "liquidfun.render-mode.v1";

type RenderModeStorage = Pick<Storage, "getItem" | "setItem">;
export type RenderModeStorageProvider = () => RenderModeStorage;

/** Parses one persisted or control-provided rendering token. */
export function maybeParseRenderMode(
  value: string | null,
): RenderMode | undefined {
  if (value === "wireframe" || value === "solid") {
    return value;
  }

  return undefined;
}

/** Loads a render preference while containing unavailable browser storage. */
export function loadRenderMode(
  storageProvider: RenderModeStorageProvider,
): RenderMode {
  try {
    return (
      maybeParseRenderMode(
        storageProvider().getItem(RENDER_MODE_STORAGE_KEY),
      ) ?? DEFAULT_RENDER_MODE
    );
  } catch {
    return DEFAULT_RENDER_MODE;
  }
}

/** Persists a render preference without turning storage failure into app failure. */
export function persistRenderMode(
  storageProvider: RenderModeStorageProvider,
  mode: RenderMode,
): void {
  try {
    storageProvider().setItem(RENDER_MODE_STORAGE_KEY, mode);
  } catch {
    // The in-memory preference remains valid when storage is unavailable.
  }
}
