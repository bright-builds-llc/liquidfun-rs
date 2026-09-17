import init, {
  ProofSession,
} from "../generated/liquidfun-wasm/liquidfun_wasm.js";
import wasmUrl from "../generated/liquidfun-wasm/liquidfun_wasm_bg.wasm?url";

/** Initializes the generated package and constructs the named Dam Break scene. */
export async function loadProofSession(): Promise<ProofSession> {
  await init({ module_or_path: wasmUrl });
  return new ProofSession();
}
