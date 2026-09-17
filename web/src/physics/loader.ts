import init, {
  ProofSession,
} from "../generated/liquidfun-wasm/liquidfun_wasm.js";
import wasmUrl from "../generated/liquidfun-wasm/liquidfun_wasm_bg.wasm?url";

/** Initializes the generated package before constructing its opaque session. */
export async function loadProofSession(): Promise<ProofSession> {
  await init({ module_or_path: wasmUrl });
  return new ProofSession();
}
