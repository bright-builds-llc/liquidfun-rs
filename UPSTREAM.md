# Upstream LiquidFun Oracle

This repository uses the official Google LiquidFun source only as a
development-time oracle for research, tests, comparison, reference generation,
and benchmark comparison. The published Rust library must remain independent
of this checkout and of a C++ toolchain.

The rationale for the selected post-release commit is recorded in
[ADR 0001](docs/decisions/0001-oracle-selection.md). The machine-readable
identity authority is [`reference/upstream-lock.toml`](reference/upstream-lock.toml).

## Identity

| Field | Canonical value |
| --- | --- |
| Repository | `https://github.com/google/liquidfun.git` |
| Selected revision | `7f20402173fd143a3988c921bc384459c6a858f2` |
| Release tag | `v1.1.0` |
| Annotated tag object | `d15bcf1879144bf2a4c8ebcc73f6418186756fb2` |
| Peeled release commit | `f38db7c627c3dc5ec879d726e16fa5a12ad6e478` |
| Submodule path | `third_party/liquidfun` |
| Patch set | `none` |

The selected revision is an immutable official commit. The tag records release
context; it is not a substitute for the selected revision.

## Ancestry

The pinned `liquidfun/ReleaseNotes.md` states that LiquidFun 1.1.0 is based on
Box2D 2.3.0, revision 280. That statement and the exact pinned tree are the
ancestry evidence for this project. Modern Box2D releases and unrelated forks
are not compatibility references unless a later decision record explicitly
changes the oracle.

The selected revision descends from the peeled `v1.1.0` release commit. ADR
0001 audits the material changes between the release and the selected official
commit.

## Tree Scope

The gitlink preserves the complete official repository tree:

- `liquidfun/` contains the native engine, documentation, examples, testbed,
  unit tests, bindings, and contributions.
- `googletest/` and `freeglut/` are upstream-vendored developer dependencies
  used by tests and the testbed.

All content under `third_party/liquidfun` is read-only. Do not edit, format,
regenerate, or commit changes inside the submodule. Repository-owned wrappers,
provenance records, fixtures, and derived work live outside it.

## Initialize

Initialize exactly the gitlink recorded by the parent repository:

```bash
git submodule update --init --recursive third_party/liquidfun
```

This command consumes the committed gitlink. It does not select a branch.

## Verify

Inspect the gitlink, checked-out revision, and upstream worktree:

```bash
git submodule status third_party/liquidfun
git -C third_party/liquidfun rev-parse HEAD
git -C third_party/liquidfun status --short
```

The revision must be
`7f20402173fd143a3988c921bc384459c6a858f2`, the status line must begin with a
space rather than `+` or `-`, and the upstream worktree status must be empty.
`cargo xtask upstream verify` automates agreement among the gitlink, checkout,
lock, origin URL, clean worktree, and local tool identities. Run
`cargo xtask provenance check` for the related repository evidence records.

## Intentional Update

Oracle updates are review events, not routine dependency refreshes. Start only
with a reviewed, full 40-hex official commit and validate the input before any
checkout:

```bash
revision=0123456789abcdef0123456789abcdef01234567
test "${#revision}" -eq 40
test -z "$(printf '%s' "$revision" | tr -d '0-9a-f')"
git -C third_party/liquidfun fetch origin "$revision"
git -C third_party/liquidfun checkout --detach "$revision"
```

Then, in the same review:

1. prove the commit belongs to the official repository and record its ancestry;
1. audit the delta from both the current oracle and the release baseline;
1. update `reference/upstream-lock.toml`, the gitlink, ADR 0001 or a superseding
   ADR, source mappings, notices, and affected artifact manifests;
1. re-run upstream, provenance, inventory, differential, package-isolation, and
   license checks applicable at that stage; and
1. verify `git -C third_party/liquidfun status --short` remains empty.

Never enable branch tracking for the submodule or use a branch name to choose
the oracle.

## Build

The repository-owned wrapper requires CMake 3.25 or newer and Ninja 1.11 or
newer. Canonical Linux CI uses CMake 4.3.3, Ninja 1.13.2, and Clang 22.1.8.
Verify identity and provenance before configuring or building:

```bash
cargo xtask upstream verify
cargo xtask provenance check
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
```

`just oracle-debug` is the thin configure-and-build alias. The wrapper owns
legacy policy compatibility outside the submodule and writes only under
`target/reference/`. Ordinary Cargo build, test, documentation, and package
paths remain free of C++ requirements.

## Patches

The canonical patch identity is currently `patch_set = "none"`. Prefer wrapper
configuration, compiler options, and adapter code outside the submodule over
source changes.

If a source patch becomes unavoidable, stop and create a reviewable external
patch register before applying it. Each entry must identify the upstream
revision and path, record preimage and patch SHA-256 hashes, summarize the
alteration, classify it as build-only or behavior-affecting, link its license
and notice class, and replace `none` with a stable patch-set identity in the
lock. An unregistered or in-place upstream modification blocks reference use.

## Licenses and Notices

- The root [`LICENSE`](LICENSE) covers original `liquidfun-rs` work.
- `third_party/liquidfun/liquidfun/Box2D/License.txt` contains the upstream
  zlib-style Box2D/LiquidFun source license and altered-source conditions.
- `third_party/liquidfun/liquidfun/NOTICE` contains the LiquidFun notice.
- [`THIRD_PARTY_NOTICES.md`](THIRD_PARTY_NOTICES.md) records the repository's
  notice and developer-only dependency policy.
- `reference/source-map.toml` records provenance and alterations for local
  derived material.

The root license does not replace upstream notice, provenance, attribution, or
alteration duties. Any unresolved licensing ambiguity blocks a release claim
until reviewed.
