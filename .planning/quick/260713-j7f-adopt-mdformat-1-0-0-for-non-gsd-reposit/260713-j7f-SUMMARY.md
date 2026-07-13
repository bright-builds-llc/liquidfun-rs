# Quick Task 260713-j7f Summary

## Outcome

- Added a root mdformat 1.0.0 contract for repository-owned non-GSD Markdown, with parser-owned, vendored, generated, Git, and backup paths excluded.
- Added repo-local contributor guidance and a discoverable `just markdown-check` command without changing the Cargo-only `just check` workflow.
- Preserved all three existing lesson blocks and normalized only their ordered-list markers to mdformat's repeated `1.` style.
- Added Python 3.13 and an isolated, exact `mdformat==1.0.0` check to the Linux Cargo quality job before Rust quality work.

## Commits

- `30a4f72` — `docs: standardize Markdown formatting`
- `716e371` — `ci: enforce Markdown formatting`

## Verification

- `mdformat .` — passed; existing Markdown rewrites were limited to `.codex/tasks/lessons.md` marker normalization.
- `mdformat --check .` — passed.
- `just markdown-check` — passed.
- `git diff --exit-code -- .planning third_party` — passed before each implementation commit.
- `actionlint .github/workflows/ci.yml` — passed.
- `cargo test -p xtask --test docs_contract` — passed, 29 tests.
- `cargo xtask docs check` — passed, including all Phase 4–7 document contracts.
- `git diff --check` — passed.
- Mandatory Rust sequence passed in order before each commit: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`.
- The global Codex lesson file retained SHA-256 `fa63d40defa088f9743f2732ea83c72625f77c8496d597204f8d8ef645b1ab53` throughout execution.

## Residual Risks

- mdformat 1.0.0 discovers Markdown paths before applying configuration exclusions, so `mdformat --check .` can be slow when an excluded `target/` tree is large. Correctness and isolation are unaffected, but local check duration depends on checkout size.
- `.planning/**` remains intentionally outside mdformat until GSD's parser and generators support a compatible formatting contract.
