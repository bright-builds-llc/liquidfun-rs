# Phase 21: Playground leftover cleanup - Research

**Researched:** 2026-09-20
**Domain:** SolidJS leftover chrome deletion, opt-in Phase 16 forensic isolation, Bright Builds 628-line file-length cleanup
**Confidence:** HIGH

<user_constraints>
## User Constraints (from CONTEXT.md)

### Locked Decisions

### Unused proof helper and forensic spec

- **D-01:** Delete unused `loadProofSession` from `web/src/physics/loader.ts`. The live player already constructs worlds through `loadSceneSession(sceneId)`. Do not re-wire the Dam Break wrapper into `App.tsx`, tests, or the forensic spec.
- **D-02:** Keep `web/e2e/rust-wasm-proof.spec.ts` as an explicit opt-in behind `PHASE16_CLOSURE_ATTEMPT_DIR` / `just web-smoke`. Do not fold Phase 16 Dispose-session, PNG-hash, or closure-attempt forensics into ordinary `just web-player-smoke`.
- **D-03:** Do not rewrite the forensic spec onto current Play/Pause/Reset chrome in this phase. Product proof remains the existing Chromium player suite. Document that `just web-smoke` is historical Phase 16 chrome, not the v1.1 product gate.

### Dead FallbackPanel branches

- **D-04:** Remove `FallbackPanel` `empty` and `not-ready` kinds, copy, and `App.tsx` `fallbackProps` branches that cannot be reached from a real visitor path. All six catalog ids are `ready: true`, and empty hashes already normalize to `#/scene/dam-break`.
- **D-05:** Keep the unknown-hash fallback as the real useful path: `kind: "unknown"` still shows Scene not found, the locked explanation, and `Open Dam Break`. Do not leave a blank canvas for `#/scene/not-a-scene`.
- **D-06:** Do not undo `normalizeSceneRoute` empty-hash replacement to Dam Break. Parser `kind: "empty"` may remain as an internal parse result; it must not render empty FallbackPanel chrome.

### Scene file-length cleanup

- **D-07:** Named scene modules `color_mixer.rs`, `dam_break.rs`, and `water_wheel.rs` must satisfy Bright Builds `file-lengths` (628 physical lines). Live scout already shows them under the gate after `tests.rs` extraction (346 / 360 / 426). Confirm that, and split further only if a named file is over the gate again.
- **D-08:** Any split uses the existing `foo.rs` plus `foo/` layout. Extract tests or helpers; do not change public scene behavior, controls, particle recipes, gravity, joints, pointer magnitudes, or catalog/WASM ids.
- **D-09:** `tools/xtask/tests/upstream_cli.rs` (642 lines) is the current remaining `file-lengths` finding and is not one of the three named scene files. Bring it under the gate in this phase only if phase verification requires `bun scripts/bright-builds-check.ts file-lengths` or `all` to exit 0. Prefer a `foo.rs` plus `foo/` test split over a TSV exception. Do not change xtask CLI behavior.

### Verification boundary

- **D-10:** Prove leftover cleanup with existing local checks: loader/fallback unit tests, `cargo test -p liquidfun-wasm` when scene modules move, Bright Builds `file-lengths` on the named files (and `all` / `file-lengths` exit 0 if that is the phase gate), and `just web-player-smoke` after player/fallback edits. Do not add Firefox/Safari/WebKit, screenshot-hash oracles, Playwright on Pages CI, Linux qualification, or Dam Break C++ timing.
- **D-11:** Independent AI review remains eligible under the 2026-09-16 owner policy. The implementing agent must not approve its own work. No crate/npm publish, no release tag, and no new GitHub Pages URL.

### Claude's Discretion

- Exact helper filenames and test-module splits inside the locked `foo.rs` plus `foo/` shape, provided scene behavior and file-length gates hold.
- Whether unknown fallback props stay a dedicated union member or a single-kind component after empty/not-ready removal.
- Exact documentation wording that marks `just web-smoke` as historical opt-in forensic chrome.
- Exact Playwright assertions for unknown-hash fallback, provided D-05 remains true and empty/not-ready chrome is gone.

### Deferred Ideas (OUT OF SCOPE)

- Playground Dam Break headless speed versus pinned C++ — out of v1.1 definition of done.
- Rewriting Phase 16 forensic smoke onto current player chrome — new evidence tooling, not leftover deletion.
- MysticUI/Tailwind/design-system migration — revisit 2026-12-17 per `standards-overrides.md`.
- Crate/npm publication, release tags, and a new GitHub Pages URL — separately authorized.
- Linux qualification, sanitizers, coverage, and a broader browser matrix — optional hobby-scope checks.
</user_constraints>

<phase_requirements>
## Phase Requirements

This phase has **no milestone requirement IDs**. REQUIREMENTS.md maps 22/22 IDs to Phases 16–20. Phase 21 closes leftover audit items only. Do not reassign WEB-02, WEBTEST-01, or any DEMO-\* id here. [VERIFIED: `.planning/REQUIREMENTS.md` Coverage table; `.planning/ROADMAP.md` § Phase 21]

| Audit leftover | Success criterion | Research support |
|----------------|-------------------|------------------|
| Unused `loadProofSession` | Helper removed or used; forensic spec stays opt-in with no unused helper | Delete the wrapper. Keep `loadSceneSession`. Keep `rust-wasm-proof.spec.ts` behind `PHASE16_CLOSURE_ATTEMPT_DIR`. |
| Dead FallbackPanel empty/not-ready | Dead branches removed or reachable | Remove empty/not-ready. Keep unknown. Do not undo empty-hash Dam Break normalization. |
| Scene file-lengths | Named scene files satisfy 628-line gate without behavior change | Confirm 346/360/426. Split further only if a named file exceeds 628. Split `upstream_cli.rs` if the phase gate requires `file-lengths`/`all` exit 0. |
| Dam Break vs C++ timing | Remains out of v1.1 DoD | Do not run `just playground-dam-break-bench` or edit `docs/playground-dam-break-timing.md` as a gate. |
</phase_requirements>

## Summary

Phase 21 is leftover deletion and a file-length confirm, not a new player, scene, or evidence harness. The live player already constructs every ready scene through `loadSceneSession(sceneId)`. `loadProofSession` is a stale Dam Break wrapper with **zero** product, unit, or forensic call sites. Empty hashes already become `#/scene/dam-break` before render, and every catalog id is `ready: true`, so FallbackPanel `empty` / `not-ready` copy cannot be reached by a visitor. Unknown hashes still must show Scene not found plus Open Dam Break.

Named WASM scene files already sit under the Bright Builds 628-line trigger after the 2026-09-19 `foo.rs` plus `foo/tests.rs` extraction. The only current `file-lengths` finding is `tools/xtask/tests/upstream_cli.rs` at 642 lines. Bright Builds Checks on `main` is already failing on that finding. Because this phase is expected to push leftover cleanup under standing authorization, the planner should make `bun scripts/bright-builds-check.ts all` exit 0 the file-length gate, which then requires the xtask test split under D-09. Do not add a TSV exception. Do not change scene recipes or xtask CLI behavior.

**Primary recommendation:** Delete `loadProofSession`; keep the forensic spec skipped and out of `test:player`; collapse FallbackPanel to unknown-only chrome; confirm the three named scene files; split `upstream_cli.rs` with the existing `foo.rs` plus `foo/` integration-test pattern so `file-lengths`/`all` exit 0; prove with unit tests plus `just web-player-smoke`; obtain independent AI review.

## Project Constraints (from .cursor/rules/)

No `.cursor/rules/` directory exists in this repository. [VERIFIED: glob `.cursor/rules/**/*` returned 0 files]

Honor instead:

- `AGENTS.md` Repo-Local Guidance: hobby scope, standing autonomous iteration and ordinary non-force `main` pushes, independent AI review (implementer must not approve own work).
- `standards-overrides.md`: semantic HTML + scoped CSS + Kobalte Dialog 0.13.12 stay; no MysticUI/Tailwind migration; `skipLibCheck: true` stays.
- `standards/core/architecture.md`: keep hash/runtime decisions in the functional core; `App.tsx` remains the imperative shell.
- `standards/core/code-shape.md`: 628 physical-line trigger; `maybe_` naming; early returns.
- `standards/languages/rust.md`: `foo.rs` plus `foo/` for touched multi-file modules; no new `mod.rs`.
- `standards/languages/typescript-javascript.md`: SolidJS + Bun stay; do not add Python scripts to the web tree.
- `standards/core/frontend-ui.md`: keep dark default and existing source/provenance chrome; no visual system change.
- `standards/core/testing.md` and `standards/core/verification.md`: focused Arrange/Act/Assert tests; repo-native verification before commit; `just markdown-check` after non-GSD Markdown; never mdformat `.planning/**`.
- `PROJECT-SCOPE.md`: no Linux qualification, no package publication, no C++ timing as completion.

## Standard Stack

No new packages. Use the already-pinned playground and checker toolchain.

### Core

| Library / tool | Version | Purpose | Why Standard |
|----------------|---------|---------|--------------|
| TypeScript | 7.0.2 | Player typecheck | Existing `web/package.json` pin. [VERIFIED: `web/package.json`] |
| SolidJS | 1.9.15 | Player shell | Existing frontend. Do not migrate. [VERIFIED: `web/package.json`] |
| Vitest | 5.0.1 | Node unit tests (`tests/**/*.test.ts`) | Existing. Solid JSX is not in the include glob. [VERIFIED: `web/package.json`, `web/vitest.config.ts`] |
| Playwright | 1.63.0 | Chromium product smoke | Existing. One Chromium project. [VERIFIED: `web/package.json`, `web/playwright.config.ts`] |
| Bun | 1.4.2 local (`packageManager`); CI Bright Builds job uses setup-bun `1.3.9` | Script runner | Do not churn. [VERIFIED: `web/package.json`; `.github/workflows/bright-builds-checks.yml`] |
| `@kobalte/core` | 0.13.12 | Drawer only | `standards-overrides.md` exception through 2026-12-17. [VERIFIED: override table] |
| Rust | 1.97.0 | Scene modules and xtask tests | `rust-toolchain.toml` / local `rustc`. [VERIFIED: `rustc --version`] |
| Bright Builds checker | repo-managed `scripts/bright-builds-check.ts` | `file-lengths` fails at 629 physical lines | `standards/core/code-shape.md`; `--help` lists exact-file TSV exceptions. [VERIFIED: checker `--help` and live run 2026-09-20] |

### Supporting

| Tool | Version | Purpose | When to Use |
|------|---------|---------|-------------|
| `just` | local 1.48.0 (docs pin 1.55.1) | `web-player-smoke` / `web-smoke` aliases | Product gate is `just web-player-smoke`. Do not use `just web-smoke` as a phase gate. [VERIFIED: `justfile`, `scripts/web-build.ts`] |
| wasm-pack | 0.15.0 | Regenerates ignored WASM package | `just web-player-smoke` already rebuilds. [VERIFIED: README / `scripts/web-build.ts`] |
| mdformat | 1.0.0 | Non-GSD Markdown | After README/TESTING wording edits. Run `just markdown-check`. [VERIFIED: `mdformat --version`; `AGENTS.md`] |
| Python 3.13 | available at `/opt/homebrew/bin/python3.13` | mdformat dialect when required | Default `python3` is 3.14.6; `just markdown-check` invokes `mdformat` on PATH. [VERIFIED: env probe] |

### Alternatives Considered

| Instead of | Could Use | Tradeoff |
|------------|-----------|----------|
| Delete `loadProofSession` | Re-wire it into App or the forensic spec | Locked out by D-01. The forensic spec does not import the loader. |
| Keep empty/not-ready FallbackPanel | Undo empty-hash normalization or add a not-ready catalog id | Locked out by D-04/D-06. Would revive dead chrome or a stub scene. |
| TSV exception for `upstream_cli.rs` | `.bright-builds-rules-checks.tsv` exact-file row | D-09 prefers `foo.rs` plus `foo/`. No TSV file exists today. [VERIFIED: file missing] |
| Fold forensic spec into `test:player` | Add `e2e/rust-wasm-proof.spec.ts` to the player allowlist | Locked out by D-02. Spec still clicks `Dispose session`, which product chrome removed. |

**Installation:** none. Do not add dependencies.

**Version verification:** `web/package.json`, `rustc 1.97.0`, `bun 1.4.2`, Playwright 1.63.0, Bright Builds checker `--help` (628-line trigger, fail at 629). Training-data versions were not used.

## Architecture Patterns

### Recommended Project Structure

```
web/src/physics/loader.ts          # only loadSceneSession(sceneId)
web/src/components/FallbackPanel.tsx  # unknown-only chrome
web/src/App.tsx                    # Show fallback={<FallbackPanel />}
web/src/routing/hash.ts            # keep kind: "empty" parse + normalizeSceneRoute
web/e2e/rust-wasm-proof.spec.ts    # keep; skip unless PHASE16_CLOSURE_ATTEMPT_DIR
web/e2e/player.spec.ts             # existing unknown-hash coverage
crates/liquidfun-wasm/src/scene/
  color_mixer.rs + color_mixer/tests.rs
  dam_break.rs + dam_break/tests.rs
  water_wheel.rs + water_wheel/tests.rs
tools/xtask/tests/upstream_cli.rs  # fixtures + mod children
tools/xtask/tests/upstream_cli/    # extracted #[test] modules
```

### Pattern 1: Delete the unused wrapper; keep the live constructor

**What:** `App.tsx` already awaits `loadSceneSession(id)` then `createSceneSession` plus stale-generation dispose. `loadProofSession` only forwards `"dam-break"`.
**When to use:** This phase. Do not keep a compatibility alias.
**Example:**

```ts
// Source: web/src/physics/loader.ts (live, 2026-09-20)
export async function loadSceneSession(sceneId: string): Promise<ProofSession> {
  await init({ module_or_path: wasmUrl });
  return new ProofSession(sceneId);
}

// DELETE the loadProofSession wrapper. Do not rename generated ProofSession.
```

Keep the generated `ProofSession` class name. wasm-pack output stays ignored and is not a public API rename.

### Pattern 2: Empty stays a parse result; render only unknown fallback

**What:** `maybeParseSceneRoute` still returns `kind: "empty"` for `""`, `"#"`, `"#/"`, `"#/scene"`, `"#/scene/"`. `normalizeSceneRoute` replaces those with `{ kind: "scene", id: "dam-break" }` and `maybeReplacementHash: "#/scene/dam-break"`. `App` initializes and handles `hashchange` through `normalizeSceneRoute`, then `history.replaceState` when a replacement hash exists.
**When to use:** Always. D-06 forbids undoing this.
**Example:**

```ts
// Source: web/src/routing/hash.ts
if (route.kind === "empty") {
  return {
    route: { kind: "scene", id: "dam-break" },
    maybeReplacementHash: DEFAULT_SCENE_HASH,
  };
}
```

After D-04, `FallbackPanel` should not accept `empty` or `not-ready`. Recommend a no-prop unknown-only component (discretion). `fallbackProps` can disappear. `Show` fallback becomes `<FallbackPanel />`. If a not-ready catalog id ever returned, `maybeReadySceneId` would still mount this unknown chrome rather than a blank canvas — acceptable because all six ids are `ready: true` and D-04 removes not-ready copy.

Keep `SceneRoute` `kind: "empty"` for parser tests. Keep `routeIdentity`'s `"empty"` branch for union exhaustiveness; it is not FallbackPanel chrome.

### Pattern 3: Isolate forensic smoke with two existing locks

**What:** Product smoke is `just web-player-smoke` → `bun scripts/web-build.ts player-smoke` → `bun run test:player`, which lists only `player.spec.ts`, `demo-media-clock.spec.ts`, `shell.spec.ts`, and `reset-honesty.spec.ts`. Forensic smoke is `just web-smoke` → mode `smoke` → allocates `target/phase16/closure-attempt-N`, sets `PHASE16_CLOSURE_ATTEMPT_DIR`, and runs `bun run test:browser` (entire `e2e/`).
**When to use:** Keep both locks. Do not add the forensic spec to `test:player`.
**Example:**

```ts
// Source: web/e2e/rust-wasm-proof.spec.ts
test.skip(
  process.env.PHASE16_CLOSURE_ATTEMPT_DIR === undefined,
  "Phase 16 forensic smoke is opt-in and requires PHASE16_CLOSURE_ATTEMPT_DIR",
);
```

The spec still looks for button `Dispose session`, status strings `Loading Rust/WASM session…` / `Running Rust/WASM session` / `Rust/WASM session disposed`, and PNG SHA-256 oracles. Product chrome removed Dispose session in Phase 17. Invoking `just web-smoke` against the current player is expected to fail. D-03: document that; do not rewrite the spec.

### Pattern 4: Confirm named scene files; split xtask tests like inventory_cli

**What:** Named scene modules already declare `#[cfg(test)] mod tests;` and keep private tests in `foo/tests.rs`. Do not move production physics.
**When to use:** Further scene splits only if a named file exceeds 628 again. For `upstream_cli.rs`, reuse the integration-test layout already used by `tools/xtask/tests/inventory_cli.rs`.
**Example:**

```rust
// Source: crates/liquidfun-wasm/src/scene/dam_break.rs (end of file)
#[cfg(test)]
mod tests;

// Source: tools/xtask/tests/inventory_cli.rs (existing split pattern)
#[path = "inventory_cli/attestation.rs"]
mod attestation;
```

Prefer implicit `mod verify;` plus `tools/xtask/tests/upstream_cli/verify.rs` over `mod.rs`. Keep `RepositoryFixture`, `FakeTools`, and helpers in the parent. Move `#[test]` functions into children grouped by command (`verify`, `configure`, `build` plus remaining LF/clang/cmake-failure cases). Keep `build_accepts_the_registered_playground_dam_break_bench` as a **registration** test; that is not Dam Break C++ timing.

### Anti-Patterns to Avoid

- **Re-wiring `loadProofSession` so it is "used":** D-01 forbids it. The live constructor is already generic.
- **Folding PNG hashes or Dispose-session into `test:player`:** D-02/D-03. Product smoke asserts DOM attributes, not canvas pixel SHA-256.
- **Rendering empty FallbackPanel for leaked `kind: "empty"`:** D-06. After normalize, that kind does not reach render. Do not restore "Choose a scene".
- **Undoing `normalizeSceneRoute` so empty fallback becomes reachable:** That would violate D-06 and Phase 20 empty-hash honesty (`web/e2e/shell.spec.ts`).
- **Tuning particle recipes, gravity, joints, or pointer magnitudes to shrink files:** D-08. File length is a layout problem, not a physics problem.
- **Adding `.bright-builds-rules-checks.tsv` for `upstream_cli.rs`:** D-09 prefers a split. No TSV exists. Do not edit the managed checker.
- **Self-approving `21-REVIEW.md`:** D-11. Passing `just web-player-smoke` is not the acknowledgment.
- **Running `just playground-dam-break-bench` as a phase gate:** Success criterion 4 / D-10.

## Don't Hand-Roll

| Problem | Don't Build | Use Instead | Why |
|---------|-------------|-------------|-----|
| Unused Dam Break constructor | New App special-case or forensic import | Delete `loadProofSession`; keep `loadSceneSession` | Wrapper has no callers. [VERIFIED: `rg loadProofSession` in `web/src` is only the definition] |
| Forensic vs product smoke | New Playwright project or Pages job | Existing `test:player` allowlist + `test.skip` + `PHASE16_CLOSURE_ATTEMPT_DIR` | Double isolation already works. [VERIFIED: `web/package.json`, `scripts/web-build.ts`, spec skip] |
| Unknown visitor hash | Blank canvas or 404 page | Existing FallbackPanel unknown copy + hash-only `#/scene/dam-break` | WEB-02 / Phase 17 D-06 useful fallback. [VERIFIED: `player.spec.ts` unknown-hash test] |
| File-length gate | Custom line counter or checker patch | `bun scripts/bright-builds-check.ts file-lengths` | Managed fail-at-629, physical lines, optional exact-file TSV. [VERIFIED: checker `--help`] |
| Oversized xtask integration test | `foo/mod.rs` or TSV exception | `upstream_cli.rs` + `upstream_cli/*.rs` like `inventory_cli` | rust.md `must` for touched multi-file modules. |
| Fallback unit tests that import Solid JSX | Vitest file importing `FallbackPanel.tsx` | Playwright unknown-hash test, or extract copy to `.ts` if a unit test is wanted | `vitest` include is `tests/**/*.test.ts` and fails Solid JSX analysis. [VERIFIED: Phase 18 control-helper extraction; `web/vitest.config.ts`] |
| Independent review digest | Ad-hoc "LGTM" | Separate `gsd-code-reviewer`, SHA-256 of listed file bytes, `implementing_or_fixing_executor: no` | Phase 20 review fields. [VERIFIED: `20-REVIEW.md` frontmatter] |

**Key insight:** The leftovers are already isolated by construction. Cleanup is deletion and confirmation, not a new architecture. The only extra structural task is the xtask test split needed to turn Bright Builds Checks green.

## Runtime State Inventory

This phase deletes identifiers and may split a test module. After every git-tracked file is updated:

| Category | Items Found | Action Required |
|----------|-------------|------------------|
| Stored data | None — verified no DB, Redis, or Mem0 keys named `loadProofSession`, FallbackPanel kinds, or scene module paths | None |
| Live service config | GitHub Pages already serves `https://bright-builds-llc.github.io/liquidfun-rs/`. Pages workflow does not store FallbackPanel kinds or `loadProofSession`. D-11 forbids a new Pages URL | Code edit only. Do not treat a Pages redeploy as a phase gate |
| OS-registered state | None — verified no launchd/systemd/pm2 names for these leftovers | None |
| Secrets/env vars | `PHASE16_CLOSURE_ATTEMPT_DIR` is a local opt-in path env, not a secret. Keep the name. Forensic `just web-smoke` still allocates `target/phase16/closure-attempt-N` | Keep env contract. Never overwrite failed attempt dirs |
| Build artifacts | Ignored `web/src/generated/liquidfun-wasm` regenerates on `just web-wasm` / player-smoke. Generated class remains `ProofSession`. Historical closure attempts under `target/phase16/` | Regenerated WASM is enough; do not commit generated output |

**Nothing found in category:** Stated explicitly above after live `rg` of `web/src`, `justfile`, Pages workflow, and loader/spec files.

## Common Pitfalls

### Pitfall 1: Reviving empty FallbackPanel chrome

**What goes wrong:** "Choose a scene" reappears for `""` / `"#"` / `"#/scene"`, or empty hashes stop replacing to Dam Break.
**Why it happens:** Phase 17 D-06 originally wanted distinct empty vs unknown copy. Phase 20 made empty a Dam Break `replaceState`. `fallbackProps` still has an `empty` branch that looks live in source.
**How to avoid:** Keep `normalizeSceneRoute`. Delete empty copy from FallbackPanel. Keep `hash.test.ts` empty-parse **and** empty-normalize tests. Keep `shell.spec.ts` empty-hash history test.
**Warning signs:** `EMPTY_HEADING` / "Choose a scene" still in `FallbackPanel.tsx`; Playwright sees FallbackPanel on `/liquidfun-rs/` instead of Dam Break.

### Pitfall 2: Changing scene recipes while "just splitting"

**What goes wrong:** Particle counts, gravity, joints, mix strength, or pointer magnitudes drift; catalog/WASM ids change.
**Why it happens:** Named files look large; shrinking constants is tempting. Water Wheel grew 422 → 426 after the particle-cap bump and is still under 628.
**How to avoid:** If a named file is under 628, do not touch it. If a split is required, move tests/helpers only. Re-run `cargo test -p liquidfun-wasm --lib` only when scene modules move.
**Warning signs:** Diff in `const` physics values, `ParticleFlags`, joint builders, or catalog ids.

### Pitfall 3: Folding forensic PNG hashes into product smoke

**What goes wrong:** `just web-player-smoke` starts requiring `PHASE16_CLOSURE_ATTEMPT_DIR`, Dispose-session, or canvas SHA-256 goldens.
**Why it happens:** ROADMAP success criterion 1 allows "fold or keep". D-02/D-03 already chose keep-as-opt-in. The spec lives in `e2e/` next to product tests.
**How to avoid:** Do not add `rust-wasm-proof.spec.ts` to `test:player`. Do not rewrite selectors onto Play/Pause/Reset. Do not run `just web-smoke` as a passing gate.
**Warning signs:** `test:player` script grows; player spec imports `createHash`; smoke mode required for ordinary proof.

### Pitfall 4: Leaving Bright Builds Checks red by scoping file-lengths to named scenes only

**What goes wrong:** Named scene files pass; `bun scripts/bright-builds-check.ts all` still fails on `upstream_cli.rs`; GitHub workflow `Bright Builds Checks` stays failed on push.
**Why it happens:** ROADMAP criterion 3 names only the three scene files. D-09 makes the xtask split conditional on the phase gate. Phase 20 scoped the checker to new CSS/App files and deferred this finding. Commit `a0f9355` added `build_accepts_the_registered_playground_dam_break_bench` and grew the file to 642. Latest two Bright Builds Checks runs on `main` concluded `failure`. [VERIFIED: live checker; `gh run list`; `git log -3 -- tools/xtask/tests/upstream_cli.rs`]
**How to avoid:** Set the phase file-length gate to `file-lengths`/`all` exit 0. Split `upstream_cli.rs`. Do not add a TSV exception. Do not delete the bench **registration** test; do not run the C++ timer.
**Warning signs:** Checker SUMMARY still `findings=1`; CI run for the phase SHA is `failure`.

### Pitfall 5: Self-approving independent review

**What goes wrong:** Implementer writes `21-REVIEW.md` APPROVED bound to a digest they produced and reviewed.
**Why it happens:** D-11 allows AI review; it does not allow the implementing agent to acknowledge its own work.
**How to avoid:** Launch a separate `gsd-code-reviewer`. Record `reviewer_disclosure: AI reviewer, not a human` and `implementing_or_fixing_executor: no`. Passing smoke is not the acknowledgment.
**Warning signs:** Same invocation id for implementer and reviewer; missing digest; missing `AI reviewer, not a human`.

### Pitfall 6: Importing FallbackPanel into Vitest

**What goes wrong:** `bun run test:unit` fails Solid JSX analysis.
**Why it happens:** D-10 says "loader/fallback unit tests" and FallbackPanel is a `.tsx` component.
**How to avoid:** Keep unknown coverage in `player.spec.ts`. Optionally extract copy strings to `web/src/components/fallback-copy.ts` if a node unit test is desired. Mechanical `rg` that empty/not-ready copy is gone.
**Warning signs:** New `tests/fallback.test.ts` importing `../src/components/FallbackPanel`.

## Code Examples

Verified from the live tree on 2026-09-20.

### Delete loadProofSession (D-01)

```ts
// Source: web/src/physics/loader.ts — keep this, delete the wrapper below it
export async function loadSceneSession(sceneId: string): Promise<ProofSession> {
  await init({ module_or_path: wasmUrl });
  return new ProofSession(sceneId);
}
```

Live App call site (do not change to `loadProofSession`):

```ts
// Source: web/src/App.tsx
const generatedSession = await loadSceneSession(id);
if (isStaleGeneration(started, generation)) {
  createSceneSession(generatedSession).dispose();
  return;
}
```

Mechanical check after deletion:

```bash
test -z "$(rg -n 'loadProofSession' web/src web/tests web/e2e README.md TESTING.md || true)"
```

Planning docs under `.planning/` may still mention the historical helper; do not rewrite archived phase records.

### Unknown-only FallbackPanel (D-04/D-05)

Keep these strings; delete `EMPTY_*`, `NOT_READY_BODY`, and the `not-ready` heading interpolating `sceneTitle`:

```tsx
// Source: web/src/components/FallbackPanel.tsx
const UNKNOWN_HEADING = "Scene not found";
const UNKNOWN_BODY =
  "This playground link does not match a known scene. Open Dam Break, or choose a demo from the navigation list.";
const OPEN_DAM_BREAK = "Open Dam Break";
// href must remain hash-only: "#/scene/dam-break"
```

Recommend collapsing to a no-prop component:

```tsx
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
```

Existing product assertion (keep; do not add PNG hashes):

```ts
// Source: web/e2e/player.spec.ts
await page.goto(UNKNOWN_SCENE_PATH, { waitUntil: "domcontentloaded" });
await expect(
  page.getByRole("heading", { name: "Scene not found" }),
).toBeVisible();
await page.getByRole("link", { name: "Open Dam Break" }).click();
await expect(page).toHaveURL(/#\/scene\/dam-break$/);
```

`UNKNOWN_SCENE_PATH` is `/liquidfun-rs/#/scene/not-a-scene`. [VERIFIED: `web/e2e/player-helpers.ts`]

Do not interpolate `maybeRaw` into FallbackPanel copy (unknown tokens stay out of HTML). `routeIdentity` may still key on `unknown:${maybeRaw}`.

### Product smoke vs forensic smoke (D-02/D-03)

```jsonc
// Source: web/package.json
"test:browser": "playwright test",
"test:player": "playwright test e2e/player.spec.ts e2e/demo-media-clock.spec.ts e2e/shell.spec.ts e2e/reset-honesty.spec.ts"
```

```ts
// Source: scripts/web-build.ts
// player-smoke: bun run test:player  (no PHASE16_CLOSURE_ATTEMPT_DIR)
// smoke: allocate closure attempt, set PHASE16_CLOSURE_ATTEMPT_DIR, bun run test:browser
```

Document in README/TESTING that `just web-smoke` is historical Phase 16 forensic chrome (Dispose-session / PNG-hash / closure-attempt directories) and is **not** the v1.1 product gate and is **not** expected to pass against current Play/Pause/Reset chrome. Keep `just web-player-smoke` as the ordinary WEBTEST-01 gate. After README/TESTING edits: `just markdown-check`.

### Named scene file-length confirm (D-07)

Live physical lines 2026-09-20 (`wc -l` matches the checker):

| File | Lines | Gate |
|------|-------|------|
| `crates/liquidfun-wasm/src/scene/color_mixer.rs` | 346 | pass |
| `crates/liquidfun-wasm/src/scene/color_mixer/tests.rs` | 309 | pass |
| `crates/liquidfun-wasm/src/scene/dam_break.rs` | 360 | pass |
| `crates/liquidfun-wasm/src/scene/dam_break/tests.rs` | 347 | pass |
| `crates/liquidfun-wasm/src/scene/water_wheel.rs` | 426 | pass |
| `crates/liquidfun-wasm/src/scene/water_wheel/tests.rs` | 387 | pass |

Do not split these further unless a named parent exceeds 628. Nearby watch-only files (out of D-07 named set): `jelly_drop.rs` 605, `fountain.rs` 503, `float_or_sink.rs` 519, `web/src/App.tsx` 617, `web/src/app.css` 628 exactly. Do not grow `app.css`.

### xtask integration-test split (D-09, recommended because the gate should require `all` exit 0)

Live checker 2026-09-20:

```
FAIL file-lengths tools/xtask/tests/upstream_cli.rs: 642 physical lines exceeds 628
SUMMARY file-lengths scanned=1103 exceptions=0 findings=1
```

Recommended seam (parent keeps fixture; children keep `#[test]` names/assertions):

```rust
// tools/xtask/tests/upstream_cli.rs
mod verify;
mod configure;
mod build;
mod failures; // LF forcing, clang warning, cmake stdout-only failure

// tools/xtask/tests/upstream_cli/verify.rs
use super::{stderr, stdout, RepositoryFixture, TestResult};
```

Natural groups already in the file:

- `verify_*` (matching identity, wrong SHA, dirty, missing submodule, origin URL)
- `configure_*` (unknown preset, digest arguments, extra path, cmake failure)
- `build_*` (complete suite, reference executable, phase9/phase10 witnesses, **playground-dam-break-bench registration**, unregistered target)
- remaining: LF text, clang compatibility

Verify with:

```bash
cargo test -p xtask --test upstream_cli -- --test-threads=1
bun scripts/bright-builds-check.ts file-lengths
bun scripts/bright-builds-check.ts all
```

Do not change `CARGO_BIN_EXE_xtask` argument vectors, fake-tool env vars, or registered target names. That would be an xtask CLI behavior change (D-09).

## State of the Art

| Old Approach | Current Approach | When Changed | Impact |
|--------------|------------------|--------------|--------|
| `loadProofSession()` Dam Break-only constructor | `loadSceneSession(sceneId)` for all six ready ids | Phase 18-01 / 18-09; wrapper left behind | Delete the wrapper in this phase |
| Distinct empty FallbackPanel ("Choose a scene") | `normalizeSceneRoute` replaces empty hashes with Dam Break | Phase 20 | Empty parse kind stays; empty chrome goes |
| Not-ready FallbackPanel for stub catalog ids | All six `SCENES[].ready === true` | Phase 18-09 | not-ready copy is dead |
| Phase 16 Dispose-session / PNG-hash `just web-smoke` | `just web-player-smoke` / `test:player` DOM assertions | Phase 17 D-17 onward | Keep forensic spec opt-in and historical |
| Inline `#[cfg(test)] mod tests { ... }` in oversized scene files | `foo.rs` + `foo/tests.rs` | Quick 260919-eut (`888b5e8`) | Named files already under 628 |
| `upstream_cli.rs` under 628 | 642 after playground dam-break-bench registration test | `a0f9355` (2026-09-20) | Split in this phase if `all` must exit 0 |

**Deprecated/outdated:**

- FallbackPanel `empty` / `not-ready` kinds: dead visitor paths.
- `loadProofSession`: unused. Comment "so the Phase 17 player keeps compiling" is stale (`App.tsx` imports `loadSceneSession` only).
- Invoking `just web-smoke` as a passing product proof: expects retired Dispose-session chrome.
- Phase 19 deferred-items line counts 656/652/811 for the named scene files: superseded by the eut split; do not re-split from those numbers.

## Assumptions Log

| # | Claim | Section | Risk if Wrong |
|---|-------|---------|---------------|
| *(none)* | All planning-critical claims were verified against the live tree, checker, or GitHub runs | — | — |

Discretion recommendations below are labeled as recommendations, not `[ASSUMED]` facts.

## Open Questions

1. **Is `bun scripts/bright-builds-check.ts all` the phase file-length gate?**
   - What we know: D-09 splits `upstream_cli.rs` only if that gate is required. ROADMAP criterion 3 names only the three scene files. Those three already pass. CI workflow `Bright Builds Checks` runs `all` on every push and the latest two `main` runs failed (`4d008dc`, `a0f9355`). [VERIFIED: `gh run list`]
   - What's unclear: Whether branch protection treats that workflow as required. Failure is still a red check on `main`.
   - Recommendation: **Yes — make `file-lengths`/`all` exit 0 the phase gate.** That fires D-09. Split `upstream_cli.rs`. Do not TSV-except. This unblocks Bright Builds Checks without touching C++ timing.

2. **FallbackPanel shape after kind removal?**
   - What we know: D-05 needs unknown copy and Open Dam Break. Discretion allows a one-member union or a single-kind component.
   - Recommendation: **No-prop component.** A `{ kind: "unknown" }` union is noise. Do not pass `maybeRaw` into the panel.

3. **Dedicated FallbackPanel unit test file?**
   - What we know: D-10 mentions loader/fallback unit tests. There is no `loader` or FallbackPanel Vitest file today. Unknown hash is already covered in Playwright. Vitest cannot import Solid JSX.
   - Recommendation: Mechanical `rg` + existing `hash.test.ts` / `runtime.test.ts` / `player.spec.ts`. Extract `fallback-copy.ts` only if the planner wants a node test of the unknown strings.

## Environment Availability

| Dependency | Required By | Available | Version | Fallback |
|------------|------------|-----------|---------|----------|
| bun | unit tests, checker, player-smoke | ✓ | 1.4.2 | — |
| cargo / rustc | xtask test split; wasm tests only if scenes move | ✓ | 1.97.0 | — |
| just | `web-player-smoke`, `markdown-check` | ✓ | 1.48.0 | invoke `bun scripts/web-build.ts player-smoke` directly |
| Playwright Chromium | `just web-player-smoke` | ✓ (install via `bun run browser:install`) | 1.63.0 pin | player-smoke already installs |
| mdformat | README/TESTING wording | ✓ | 1.0.0 | — |
| python3.13 | AGENTS.md mdformat dialect | ✓ | 3.13 on PATH | `just markdown-check` uses `mdformat` binary |
| CMake / Ninja / C++ oracle | Dam Break vs C++ timing | present in repo tooling | — | **Do not use.** Out of phase |
| Firefox / WebKit | extra browser matrix | n/a | — | **Do not add.** D-10 |

**Missing dependencies with no fallback:** none for this phase.

**Missing dependencies with fallback:** none blocking. C++ timing tools are present but forbidden as a gate.

Step 2.6: probed. No blocking gaps.

## Security Domain

`security_enforcement` is not `false` in `.planning/config.json`. Cleanup still touches hash routing and visitor-facing fallback HTML.

### Applicable ASVS Categories

| ASVS Category | Applies | Standard Control |
|---------------|---------|------------------|
| V2 Authentication | no | No accounts |
| V3 Session Management | no | WASM `ProofSession` is a physics session, not an auth session; keep existing one-session dispose |
| V4 Access Control | no | Public Pages site |
| V5 Input Validation | yes | Keep `maybeParseSceneRoute` allowlist; do not coerce case; do not interpolate `maybeRaw` into HTML |
| V6 Cryptography | no new use | Do not add screenshot-hash oracles to product smoke. Forensic SHA-256 stays in the opt-in spec only |

### Known Threat Patterns for this leftover cleanup

| Pattern | STRIDE | Standard Mitigation |
|---------|--------|---------------------|
| Unknown hash token reflected into HTML | XSS / Tampering | Keep locked unknown copy; hash-only Open Dam Break `href="#/scene/dam-break"`; do not put `maybeRaw` in the panel |
| Traversal scene implementation links | Information disclosure | Unchanged Phase 18 fail-closed scene paths; do not widen credits |
| Treating canvas PNG SHA as a product oracle | Repudiation / brittle integrity | Keep forensic hashes out of `test:player` (D-02) |
| Self-review of cleanup that claims Pages/product proof | Repudiation | Separate AI reviewer + digest (D-11) |
| Checker exception hiding an oversized file | Tampering with process | Split instead of TSV; do not edit managed `scripts/bright-builds-check.ts` |

## Sources

### Primary (HIGH confidence)

- Live files 2026-09-20: `web/src/physics/loader.ts`, `web/src/App.tsx`, `web/src/components/FallbackPanel.tsx`, `web/src/routing/hash.ts`, `web/src/player/runtime.ts`, `web/src/catalog/scenes.ts` (six `ready: true`), `web/e2e/rust-wasm-proof.spec.ts`, `web/e2e/player.spec.ts`, `web/e2e/shell.spec.ts`, `web/package.json`, `web/playwright.config.ts`, `web/vitest.config.ts`, `scripts/web-build.ts`, `justfile`, `crates/liquidfun-wasm/src/scene/{color_mixer,dam_break,water_wheel}.rs` and `*/tests.rs`, `tools/xtask/tests/upstream_cli.rs`, `tools/xtask/tests/inventory_cli.rs`
- `bun scripts/bright-builds-check.ts --help` and live `file-lengths` / `all` runs (1 finding: `upstream_cli.rs` 642)
- `wc -l` on named scene files (346 / 360 / 426)
- `gh run list --workflow 'Bright Builds Checks'` (latest two conclusions `failure`)
- `standards/core/code-shape.md` (628-line trigger, TSV exception rules)
- `standards/languages/rust.md` (`foo.rs` plus `foo/`)
- `.planning/phases/21-playground-leftover-cleanup/21-CONTEXT.md` locked D-01..D-11
- `.planning/ROADMAP.md` § Phase 21
- `.planning/v1.1-MILESTONE-AUDIT.md` tech-debt leftovers
- `.planning/quick/260919-eut-split-oversized-wasm-scene-tests-to-sati/260919-eut-SUMMARY.md`

### Secondary (MEDIUM confidence)

- Phase 17 CONTEXT D-06 useful unknown/empty fallback vs later empty-hash Dam Break normalization (Phase 20 `shell.spec.ts`)
- Phase 20 deferred-items.md recording `upstream_cli.rs` 642 as pre-existing during catalog/Reset work
- `.github/workflows/bright-builds-checks.yml` runs `all` on every push (not verified as a GitHub *required* check)

### Tertiary (LOW confidence)

- Whether GitHub branch protection blocks merge on Bright Builds Checks — not inspected; still recommend making `all` exit 0 because the workflow is red on `main`

## Metadata

**Confidence breakdown:**

- Standard stack: HIGH — pins read from lockfiles/manifests and local `--version`
- Architecture: HIGH — live call sites, hash normalization, smoke scripts, and scene module layout inspected
- Pitfalls: HIGH — dead-path, recipe-drift, forensic-fold, CI-red, and self-review risks are evidenced in this tree

**Research date:** 2026-09-20
**Valid until:** 2026-10-20 (repo-internal leftover cleanup; re-scout line counts if scene files move again)

## Planner task sketch (not a plan)

Suggested waves, for the planner to turn into PLAN.md:

1. **Loader leftover:** Delete `loadProofSession`. Mechanical `rg`. No App re-wire. Strengthen README/TESTING `just web-smoke` wording. `just markdown-check`. `cd web && bun run test:unit && bun run typecheck`.
2. **Fallback leftover:** Remove empty/not-ready from FallbackPanel and `fallbackProps`. Keep unknown copy and Open Dam Break. Keep parser `empty` + `normalizeSceneRoute`. `bun run test:unit` plus `just web-player-smoke` (unknown-hash + empty-hash tests).
3. **File-lengths:** Confirm named scene `wc -l` < 628 and absent from checker findings. Split `upstream_cli.rs` into `foo.rs` plus `foo/` so `bun scripts/bright-builds-check.ts all` exits 0. `cargo test -p xtask --test upstream_cli`. Do not move scene modules unless a named file is over again (`cargo test -p liquidfun-wasm --lib` only then).
4. **Independent AI review:** Manifest + digest + separate reviewer. No self-approval, no publish, no new Pages URL, no C++ timing.
