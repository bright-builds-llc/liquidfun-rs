/**
 * Compiles one shader stage. Returns undefined when the driver refuses the
 * source, so callers can fall back instead of drawing with a broken program.
 */
export function compileShader(
  gl: WebGL2RenderingContext,
  type: number,
  source: string,
): WebGLShader | undefined {
  const shader = gl.createShader(type);
  if (shader === null) {
    return undefined;
  }
  gl.shaderSource(shader, source);
  gl.compileShader(shader);
  if (gl.getShaderParameter(shader, gl.COMPILE_STATUS) !== true) {
    gl.deleteShader(shader);
    return undefined;
  }
  return shader;
}

/**
 * Compiles and links a vertex/fragment pair. Returns undefined, after
 * deleting every shader and program it created, when either stage fails to
 * compile or the program fails to link.
 */
export function linkProgram(
  gl: WebGL2RenderingContext,
  vertexSource: string,
  fragmentSource: string,
): WebGLProgram | undefined {
  const vertex = compileShader(gl, gl.VERTEX_SHADER, vertexSource);
  const fragment = compileShader(gl, gl.FRAGMENT_SHADER, fragmentSource);
  const program =
    vertex === undefined || fragment === undefined
      ? null
      : gl.createProgram();
  if (vertex === undefined || fragment === undefined || program === null) {
    gl.deleteShader(vertex ?? null);
    gl.deleteShader(fragment ?? null);
    return undefined;
  }
  gl.attachShader(program, vertex);
  gl.attachShader(program, fragment);
  gl.linkProgram(program);
  gl.deleteShader(vertex);
  gl.deleteShader(fragment);
  if (gl.getProgramParameter(program, gl.LINK_STATUS) !== true) {
    gl.deleteProgram(program);
    return undefined;
  }
  return program;
}

/** Creates a GL buffer, mapping the lost-context `null` to undefined. */
export function requireBuffer(
  gl: WebGL2RenderingContext,
): WebGLBuffer | undefined {
  return gl.createBuffer() ?? undefined;
}
