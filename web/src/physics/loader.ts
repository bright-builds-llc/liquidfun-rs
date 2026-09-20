import init, {
  ProofSession,
} from "../generated/liquidfun-wasm/liquidfun_wasm.js";
import wasmUrl from "../generated/liquidfun-wasm/liquidfun_wasm_bg.wasm?url";

/** Initializes the generated package and constructs an allowlisted scene. */
export async function loadSceneSession(sceneId: string): Promise<ProofSession> {
  await init({ module_or_path: wasmUrl });
  return new ProofSession(sceneId);
}
