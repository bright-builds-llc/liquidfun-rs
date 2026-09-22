// Adapted from https://github.com/hngngn/shadcn-solid (MIT).

import type { JSX } from "solid-js";

const extractCSSregex = /((?:--)?(?:\w+-?)+)\s*:\s*([^;]*)/g;

export function stringStyleToObject(style: string): JSX.CSSProperties {
  const object: Record<string, string> = {};
  let match: RegExpExecArray | null;
  const pattern = new RegExp(extractCSSregex.source, "g");
  while ((match = pattern.exec(style))) {
    const name = match[1];
    const value = match[2];
    if (name !== undefined && value !== undefined) {
      object[name] = value;
    }
  }
  return object;
}

export function combineStyle(
  a: JSX.CSSProperties | string | undefined,
  b: JSX.CSSProperties | string | undefined,
): JSX.CSSProperties {
  const left = typeof a === "string" ? stringStyleToObject(a) : a;
  const right = typeof b === "string" ? stringStyleToObject(b) : b;
  return { ...left, ...right };
}
