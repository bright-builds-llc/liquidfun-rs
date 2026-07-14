<!-- bright-builds-rules-contributing:begin -->

# Bright Builds Contribution Defaults

This managed block is owned upstream by `bright-builds-rules`. If this block needs a fix, open an upstream PR or issue instead of editing the managed text in a downstream repo. Keep repo-local contribution guidance outside this managed block.

## Default contribution expectations

- Treat `AGENTS.md` as the entrypoint for repo-local instructions, not the complete Bright Builds Rules spec.
- Before plan, review, implementation, or audit work, read local `AGENTS.md`, `AGENTS.bright-builds.md`, `standards-overrides.md` when present, and the local managed standards pages relevant to the task; if that has not happened yet, stop and load them before continuing.
- Follow the local `AGENTS.md`.
- Use the local managed standards pages as the canonical reference, with source provenance recorded in `AGENTS.bright-builds.md` and `bright-builds-rules.audit.md`.
- Prefer simple, root-cause fixes over broad rewrites.
- Document repo-specific exceptions in `standards-overrides.md`.

## Code expectations

- Keep business logic in a functional core when practical.
- Prefer early returns and shallow control flow.
- Prefix internal nullable or optional names with `maybe`, including functions, bindings, and internal fields, and use `MaybeX` aliases only when they materially clarify a repeated nullable surface.
- Split oversized functions and files into sensible units.
- Do not hide substantial foreign-language logic inside strings; keep workflow and automation config thin, move scripts, queries, and similar artifacts into repo-owned or language-aware files, make checked-in scripts rerunnable when sensible, and have them leave breadcrumb-heavy logs and summaries in a repo-defined gitignored location.
- Parse boundary input into domain types when that removes repeated validation.
- Apply any relevant language-specific guidance from the local managed standards.

## Verification expectations

- Before substantive implementation work, sync first: fetch remote state before editing; if the current branch tracks an upstream and the worktree is clean, prefer rebasing onto the latest upstream or the repo's equivalent sync path, such as `git pull --rebase` when local guidance uses it; if a worktree starts detached, assume the repo default branch, often `main`; resolve any sync conflicts before proceeding, then run the repo's normal bootstrap or dependency-sync step when dependencies or tools may be stale.
- Before formatting Markdown, inspect repo-local guidance and formatter configuration, require the configured syntax extensions, preserve existing configuration, use repo-owned setup or migration commands when provided, and run check mode before an authorized, scoped write. Never fall back to bare `mdformat` when required plugins are unavailable.
- Before committing, run the relevant repo-native verification steps for the changed paths, including repository-compatible Markdown or shell formatter checks when supported tools are already available and local guidance does not define a clearer workflow, and do not commit if they fail.
- Prefer a repo-owned verify/check/validate/ci command when it exists over reconstructing tool commands by hand.
- Heavy integration, end-to-end, or external-service suites may stay pre-push or CI-only when local guidance or `standards-overrides.md` documents that choice.
- If hooks appear to own verification here and the local workflow is unclear, clarify whether the repo expects hooks, manual checks, or both.

## Test expectations

- Unit test pure code and business logic.
- Keep each unit test focused on one concept.
- Use explicit Arrange, Act, Assert sections unless the structure is truly obvious.

## Pull request expectations

- Explain the behavior change, not just the code movement.
- Call out any new exceptions to the standards.
- When plan, review, or audit work relied on these standards, briefly name the local guidance, sidecar, overrides, or standards pages that materially informed the work.
- Include verification evidence for the changed paths.
- Note any residual risks or follow-up work.

<!-- bright-builds-rules-contributing:end -->

## Repository-specific workflow

The crate is still a foundation scaffold. Contributions must not describe it
as a working physics engine unless the corresponding implementation and
validation evidence exists in the compatibility ledger.

### Prerequisites

Cargo-only work uses the Rust 1.97.0 toolchain selected by
`rust-toolchain.toml`; the publishable crate also has a provisional Rust 1.92.0
MSRV check. Ordinary Rust users do not need CMake, C++, the submodule, or
reference data.

Oracle work additionally requires:

- the exact recursive submodule checkout;
- CMake 3.25 or newer, with 4.3.3 used by canonical CI;
- Ninja 1.11 or newer, with 1.13.2 used by canonical CI; and
- a C++ compiler, with Clang 22.1.8 used by canonical Linux CI.

Initialize and verify the committed oracle identity:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
```

Never edit, format, or regenerate files inside `third_party/liquidfun`.

### Discover commands

The `justfile` is a thin command menu:

```bash
just --list
cargo xtask --help
```

Recipes print and invoke their underlying Cargo or `cargo xtask` commands.
Validation logic belongs in Rust tooling or the CMake wrapper, not in recipes.

### Required Rust checks

Run these commands in order before every commit:

```bash
cargo fmt --all
cargo clippy --all-targets --all-features -- -D warnings
cargo build --all-targets --all-features
cargo test --all-features
```

When private tooling changes, also run the workspace variants and denied-warning
documentation build listed in [TESTING.md](TESTING.md). `cargo xtask check`
adds the applicable inventory, package, upstream, and provenance gates; in a
Cargo-only checkout it labels the skipped oracle-dependent checks explicitly.

### Oracle workflow

After initialization, run the evidence gates before CMake:

```bash
cargo xtask upstream verify
cargo xtask provenance check
cargo xtask inventory check
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
```

`just oracle-debug` is the visible configure-and-build alias. Build outputs
belong under `target/reference/`, never in the upstream tree or consumer crate.

### Generated files and evidence ownership

- `reference/compatibility.json` is the authoritative curated ledger. Every row
  keeps independent investigated, planned, implemented, unit-test,
  differential, platform, documented-difference, and unsupported evidence.
- `reference/discovery.json` is refreshed only by
  `cargo xtask inventory discover` against the verified pinned tree.
- `COMPATIBILITY.md` is generated only by
  `cargo xtask inventory generate`; do not edit it by hand.
- `cargo xtask inventory check` is read-only and must leave all three surfaces
  unchanged.
- `reference/artifacts/manifest.toml` records reviewed artifact hashes, oracle
  and generator revisions, compiler/preset/target/flags, and notice references.
- `reference/upstream-lock.toml` and the submodule gitlink change only through
  the intentional update review described in [UPSTREAM.md](UPSTREAM.md).

### Provenance and licensing duties

Before translating or deriving source, tests, scenarios, or reference data,
add or update its `reference/source-map.toml` entry with the local path,
upstream revision and path, derivation kind, alteration summary, and notice
class. Preserve the applicable LiquidFun/Box2D notices, mark altered source
representations where required, and update
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md) when a new notice class enters
the repository.

The root MIT license covers original project work; it does not replace
upstream attribution, provenance, alteration, or notice obligations. An
unmapped derivation, dirty submodule, stale generated report, or unexplained
compatibility claim blocks the contribution.

### Pull request evidence

Describe the changed behavior or repository contract, list the exact commands
that passed, and call out platform checks that remain CI-only. Physics work
must update the applicable compatibility rows and cite its unit, differential,
platform, documentation, and provenance evidence independently.
