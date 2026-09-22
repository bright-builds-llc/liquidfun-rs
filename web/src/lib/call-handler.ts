// Adapted from https://github.com/hngngn/shadcn-solid (MIT).

import type { JSX } from "solid-js";

type BoundHandler = readonly [
  callback: (data: unknown, event: Event) => void,
  data: unknown,
];

const isFunction = (
  value: unknown,
): value is (event: never) => void => typeof value === "function";

/** Call a JSX.EventHandlerUnion with the event. */
export const callHandler = <T, E extends Event>(
  event: E & { currentTarget: T; target: Element },
  handler: JSX.EventHandlerUnion<T, E> | undefined,
) => {
  if (handler) {
    if (isFunction(handler)) {
      handler(event);
    } else {
      const [callback, data] = handler as BoundHandler;
      callback(data, event);
    }
  }

  return event.defaultPrevented;
};
