import { vi } from "vitest";
import type { RenderFrame } from "../src/physics/frame";

export function particleFrame(): RenderFrame {
  return {
    stepIndex: 1,
    particleCount: 2,
    rigidShapeCount: 0,
    maxSpeed: 1,
    stuckCandidateCount: 0,
    bodyContactCount: 0,
    particlePositions: new Float32Array([0, 1, 1, 2]),
    particleRadii: new Float32Array([0.1, 0.2]),
    particleColors: new Uint8Array([10, 20, 30, 255, 40, 50, 60, 128]),
    rigidSegments: new Float32Array(),
    rigidCircles: new Float32Array(),
    circleLabels: [],
  };
}

export function glFixture() {
  let nextId = 0,
    lost = false;
  let bound: object | undefined;
  const uploads: { buffer: object | undefined; values: number[] }[] = [];
  const constants = Object.fromEntries(
    [
      "VERTEX_SHADER",
      "FRAGMENT_SHADER",
      "COMPILE_STATUS",
      "LINK_STATUS",
      "ARRAY_BUFFER",
      "FLOAT",
      "STATIC_DRAW",
      "DYNAMIC_DRAW",
      "TEXTURE_2D",
      "TEXTURE_MIN_FILTER",
      "TEXTURE_MAG_FILTER",
      "LINEAR",
      "TEXTURE_WRAP_S",
      "TEXTURE_WRAP_T",
      "CLAMP_TO_EDGE",
      "RGBA16F",
      "RGBA",
      "HALF_FLOAT",
      "FRAMEBUFFER",
      "COLOR_ATTACHMENT0",
      "FRAMEBUFFER_COMPLETE",
      "ALIASED_POINT_SIZE_RANGE",
      "COLOR_BUFFER_BIT",
      "BLEND",
      "ONE",
      "POINTS",
      "TEXTURE0",
      "TRIANGLE_STRIP",
    ].map((name, index) => [name, index + 1]),
  );
  const methods: Record<string, unknown> = {
    ...constants,
    createBuffer: vi.fn(() => ({ id: ++nextId })),
    createShader: vi.fn(() => ({ id: ++nextId })),
    createProgram: vi.fn(() => ({ id: ++nextId })),
    createVertexArray: vi.fn(() => ({ id: ++nextId })),
    createTexture: vi.fn(() => ({ id: ++nextId })),
    createFramebuffer: vi.fn(() => ({ id: ++nextId })),
    getShaderParameter: vi.fn(() => true),
    getProgramParameter: vi.fn(() => true),
    getExtension: vi.fn(() => ({})),
    getAttribLocation: vi.fn(() => 0),
    getUniformLocation: vi.fn((_program, name) => ({ name })),
    checkFramebufferStatus: vi.fn(() => constants.FRAMEBUFFER_COMPLETE),
    getParameter: vi.fn(() => new Float32Array([1, 1024])),
    isContextLost: vi.fn(() => lost),
    bindBuffer: vi.fn((_target, buffer) => {
      bound = buffer;
    }),
    bufferData: vi.fn((_target, values: Float32Array) => {
      uploads.push({ buffer: bound, values: Array.from(values) });
    }),
  };
  const gl = new Proxy(methods, {
    get(target, name: string) {
      if (!(name in target)) target[name] = vi.fn();
      return target[name];
    },
  }) as unknown as WebGL2RenderingContext;
  const events = new EventTarget();
  const canvas = {
    width: 640,
    height: 480,
    getContext: vi.fn(() => gl),
    addEventListener: events.addEventListener.bind(events),
  } as unknown as HTMLCanvasElement;
  return {
    gl,
    canvas,
    uploads,
    lose: () => {
      lost = true;
      const event = new Event("webglcontextlost", { cancelable: true });
      events.dispatchEvent(event);
      return event;
    },
    restore: () => {
      lost = false;
      events.dispatchEvent(new Event("webglcontextrestored"));
    },
  };
}
