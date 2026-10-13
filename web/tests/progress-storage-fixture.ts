/** In-memory `Storage` fake with a write log and switchable failures (Phase 41 D-14 tests). */
export type FakeStorage = {
  readonly storage: Pick<Storage, "getItem" | "setItem" | "removeItem">;
  readonly values: Map<string, string>;
  /** Keys passed to `setItem`, in call order, including calls that then threw. */
  readonly writes: string[];
  failSetItem(on: boolean): void;
  failRemoveItem(on: boolean): void;
  failGetItem(on: boolean): void;
};

export function createFakeStorage(initial: Readonly<Record<string, string>> = {}): FakeStorage {
  const values = new Map<string, string>(Object.entries(initial));
  const writes: string[] = [];
  let setFails = false;
  let removeFails = false;
  let getFails = false;

  return {
    storage: {
      getItem(key) {
        if (getFails) {
          throw new DOMException("blocked", "SecurityError");
        }
        return values.get(key) ?? null;
      },
      setItem(key, value) {
        writes.push(key);
        if (setFails) {
          throw new DOMException("blocked", "QuotaExceededError");
        }
        values.set(key, value);
      },
      removeItem(key) {
        if (removeFails) {
          throw new DOMException("blocked", "SecurityError");
        }
        values.delete(key);
      },
    },
    values,
    writes,
    failSetItem(on) {
      setFails = on;
    },
    failRemoveItem(on) {
      removeFails = on;
    },
    failGetItem(on) {
      getFails = on;
    },
  };
}
