---
phase: quick-260919-eut
plan: "01"
type: execute
wave: 1
depends_on: []
files_modified:
  - crates/liquidfun-wasm/src/scene/color_mixer.rs
  - crates/liquidfun-wasm/src/scene/color_mixer/tests.rs
  - crates/liquidfun-wasm/src/scene/dam_break.rs
  - crates/liquidfun-wasm/src/scene/dam_break/tests.rs
  - crates/liquidfun-wasm/src/scene/water_wheel.rs
  - crates/liquidfun-wasm/src/scene/water_wheel/tests.rs
autonomous: true
requirements: []
generated_by: gsd-plan-phase
lifecycle_mode: direct-fallback
phase_lifecycle_id: quick-260919-eut-wasm-scene-test-split
generated_at: 2026-09-19T15:42:54Z
must_haves:
  truths:
    - The managed Bright Builds checker reports no file-length finding for Color Mixer, Dam Break, or Water Wheel.
    - Every existing scene test still compiles and passes with unchanged physics assertions and runtime behavior.
    - Each scene keeps its implementation in `foo.rs` and its private tests in `foo/tests.rs`.
    - The pushed commit is verified by SHA in Bright Builds Checks, Cargo CI, and Pages.
    - The unrelated untracked `.vscode/` directory remains untouched and uncommitted.
  artifacts:
    - path: crates/liquidfun-wasm/src/scene/color_mixer/tests.rs
      provides: Color Mixer unit tests extracted from the implementation file
    - path: crates/liquidfun-wasm/src/scene/dam_break/tests.rs
      provides: Dam Break unit tests extracted from the implementation file
    - path: crates/liquidfun-wasm/src/scene/water_wheel/tests.rs
      provides: Water Wheel unit tests extracted from the implementation file
  key_links:
    - from: crates/liquidfun-wasm/src/scene/color_mixer.rs
      to: crates/liquidfun-wasm/src/scene/color_mixer/tests.rs
      via: `#[cfg(test)] mod tests;`
    - from: crates/liquidfun-wasm/src/scene/dam_break.rs
      to: crates/liquidfun-wasm/src/scene/dam_break/tests.rs
      via: `#[cfg(test)] mod tests;`
    - from: crates/liquidfun-wasm/src/scene/water_wheel.rs
      to: crates/liquidfun-wasm/src/scene/water_wheel/tests.rs
      via: `#[cfg(test)] mod tests;`
---

# Quick Plan: Split Oversized WASM Scene Tests

<objective>
Resolve the only failure in GitHub Actions run 35425567476 by extracting the three oversized scene files' inline test modules into conventional child modules.

Purpose: Satisfy the Bright Builds 628-line source-file trigger without checker exceptions or physics changes.
Output: Three shortened scene implementation files, three private child test modules, passing local verification, and SHA-bound passing push CI.
</objective>

<execution_context>
@$HOME/.cursor/get-shit-done/workflows/execute-plan.md
@$HOME/.cursor/get-shit-done/templates/summary.md
</execution_context>

<context>
@AGENTS.md
@AGENTS.bright-builds.md
@standards-overrides.md
@standards/core/code-shape.md
@standards/core/testing.md
@standards/core/verification.md
@standards/languages/rust.md
@crates/liquidfun-wasm/src/scene/color_mixer.rs
@crates/liquidfun-wasm/src/scene/dam_break.rs
@crates/liquidfun-wasm/src/scene/water_wheel.rs

Discovery is Level 0: this is a structural extraction using Rust's established `foo.rs` plus `foo/` module layout, with no new dependency or behavior.

Root cause is confirmed from [GitHub run 35425567476](https://github.com/bright-builds-llc/liquidfun-rs/actions/runs/35425567476): `bun scripts/bright-builds-check.ts all` reported exactly three findings—`color_mixer.rs` at 656 physical lines, `dam_break.rs` at 652, and `water_wheel.rs` at 811. The lessons check had zero findings and the aggregate summary had exactly three findings. Local `wc -l` reproduces those counts.

The current worktree is `main...origin/main` with only unrelated untracked `.vscode/`. Preserve it exactly. The materially applicable guidance is the repo's autonomous iteration and hobby-project policy, the managed 628-line refactor trigger, focused Arrange/Act/Assert tests, pre-commit verification, and Rust's required `foo.rs` plus `foo/` layout.

Constraint coverage: Q-01 structural extraction to child `tests.rs` modules → Task 1 (full); Q-02 preserve privacy, tests, and physics behavior → Task 1 (full); Q-03 every resulting source file under 628 lines with no exception → Tasks 1–2 (full); Q-04 format, native tests, full browser smoke, managed checker, and push CI → Task 2 (full); Q-05 preserve `.vscode/` → Tasks 1–2 (full).

<interfaces>
Each parent scene module must declare its external unit-test child only for test builds:

```rust
#[cfg(test)]
mod tests;
```

The child remains nested under its scene module, so existing `super::` access to private scene items remains valid. `include_str!` paths become relative to the new child file: use `include_str!("../color_mixer.rs")` and `include_str!("../water_wheel.rs")`.
</interfaces>
</context>

<tasks>

<task type="auto">
  <name>Task 1: Extract the three inline scene test modules</name>
  <files>crates/liquidfun-wasm/src/scene/color_mixer.rs, crates/liquidfun-wasm/src/scene/color_mixer/tests.rs, crates/liquidfun-wasm/src/scene/dam_break.rs, crates/liquidfun-wasm/src/scene/dam_break/tests.rs, crates/liquidfun-wasm/src/scene/water_wheel.rs, crates/liquidfun-wasm/src/scene/water_wheel/tests.rs</files>
  <action>Fetch remote state before editing and confirm the source commit is still compatible, without rebasing through or modifying the unrelated `.vscode/` directory. For each scene, replace the complete inline `#[cfg(test)] mod tests { ... }` block with `#[cfg(test)] mod tests;`, then move the block contents verbatim into the corresponding `foo/tests.rs` child file. Preserve test names, imports, helpers, Arrange/Act/Assert structure, assertion text, order, visibility, and all numeric expectations. Remove only the wrapper indentation and braces required by extraction. In the two source-inspection tests, change only the relative include paths to `include_str!("../color_mixer.rs")` and `include_str!("../water_wheel.rs")`; keep splitting at `#[cfg(test)]` so those tests still inspect implementation-only text. Run `cargo fmt --all`. Do not move production code, alter constants or scene hooks, change public/private visibility, add a checker exception, edit managed tooling, or touch `.vscode/`. Require all six resulting Rust files to contain fewer than 628 physical lines.</action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo test -p liquidfun-wasm --lib -- --test-threads=1 &amp;&amp; for file in crates/liquidfun-wasm/src/scene/color_mixer.rs crates/liquidfun-wasm/src/scene/color_mixer/tests.rs crates/liquidfun-wasm/src/scene/dam_break.rs crates/liquidfun-wasm/src/scene/dam_break/tests.rs crates/liquidfun-wasm/src/scene/water_wheel.rs crates/liquidfun-wasm/src/scene/water_wheel/tests.rs; do test "$(wc -l &lt; "$file")" -lt 628 || exit 1; done</automated>
  </verify>
  <done>All three parent files load private external test children, all existing `liquidfun-wasm` library tests pass unchanged, source-inspection tests still read their parent implementations, and each parent and child file is below 628 physical lines.</done>
</task>

<task type="auto">
  <name>Task 2: Verify locally, push safely, and close CI</name>
  <files>No additional source files; verification and ordinary non-force publication of Task 1 only</files>
  <action>Run the complete required gate against the final diff: format check, full single-threaded `liquidfun-wasm` library tests, `just web-player-smoke` because these Rust scene module paths are rebuilt into the six-scene browser artifact, the managed Bright Builds checker, and `git diff --check`. Review the diff to prove production code changed only by the three `mod tests;` declarations and that test bodies changed only for extraction/indentation plus the two necessary `include_str!` paths. Confirm no `.bright-builds-rules-checks.tsv` exception was added and `git status --short -- .vscode` still reports only the pre-existing untracked directory. Stage only the six owned Rust paths and GSD completion artifact; never use broad staging that captures `.vscode/`. Commit descriptively, fetch once more, integrate ordinary compatible drift if needed, and push `main` to `origin` without force. Bind remote verification to the exact pushed SHA: wait for and run `gh run watch --exit-status` on that SHA's `Bright Builds Checks`, `Cargo CI`, and `Pages` runs. If any run fails, inspect its logs, preserve the failed run, correct within this scope, rerun all local gates, create a new commit/push, and verify the replacement SHA rather than treating stale evidence as passing. Do not publish a crate, create a release tag, or claim success from run 35425567476.</action>
  <verify>
    <automated>cargo fmt --all --check &amp;&amp; cargo test -p liquidfun-wasm --lib -- --test-threads=1 &amp;&amp; just web-player-smoke &amp;&amp; bun scripts/bright-builds-check.ts all &amp;&amp; git diff --check</automated>
  </verify>
  <done>The final local gate passes, only the intended scene/test split is committed, `.vscode/` remains untracked, the ordinary push succeeds, and SHA-matched Bright Builds Checks, Cargo CI, and Pages runs all conclude successfully.</done>
</task>

</tasks>

<threat_model>
## Trust Boundaries

| Boundary | Description |
| --- | --- |
| Scene implementation to child test module | Tests retain privileged access to private scene internals and source-inspection fixtures. |
| Local checkout to GitHub Actions | Remote success must correspond to the exact pushed commit, not the historical failed run or a newer SHA. |

## STRIDE Threat Register

| Threat ID | Category | Component | Disposition | Mitigation Plan |
| --- | --- | --- | --- | --- |
| T-Q-EUT-01 | T | Scene source extraction | mitigate | Restrict implementation edits to `mod tests;`, preserve test bodies, run native and browser smoke suites, and inspect the complete diff. |
| T-Q-EUT-02 | R | CI evidence | mitigate | Capture the pushed SHA and watch all three relevant workflows filtered to that commit with exit-status enforcement. |
| T-Q-EUT-03 | T | Unrelated local files | mitigate | Use path-scoped staging and verify `.vscode/` remains untracked and untouched. |
</threat_model>

<verification>
Run, in order:

1. `cargo fmt --all --check`
1. `cargo test -p liquidfun-wasm --lib -- --test-threads=1`
1. A strict `< 628` physical-line assertion over all three parents and all three child test files
1. `just web-player-smoke`
1. `bun scripts/bright-builds-check.ts all`
1. `git diff --check`
1. Complete diff and `git status --short` review, including preservation of `.vscode/`
1. Ordinary non-force push followed by exact-SHA passing `Bright Builds Checks`, `Cargo CI`, and `Pages` runs

Do not suppress the checker, alter physics behavior, broaden the source refactor, or accept stale/failed CI evidence.
</verification>

<success_criteria>
Run 35425567476's three file-length findings are structurally resolved; every resulting Rust file is below 628 physical lines; all scene tests and the full web-player smoke pass; the managed checker has zero findings; `.vscode/` is untouched; and the exact pushed SHA passes all relevant automatic workflows.
</success_criteria>

<output>
After completion, create `.planning/quick/260919-eut-split-oversized-wasm-scene-tests-to-sati/260919-eut-SUMMARY.md`.
</output>
