# liquidfun-rs

<!-- bright-builds-rules-readme-badges:begin -->

<!-- Managed upstream by bright-builds-rules. If this badge block needs a fix, open an upstream PR or issue instead of editing the downstream managed block. Keep repo-local README content outside this managed badge block. -->

[![GitHub Stars](https://img.shields.io/github/stars/bright-builds-llc/liquidfun-rs)](https://github.com/bright-builds-llc/liquidfun-rs)
[![CI](https://img.shields.io/github/actions/workflow/status/bright-builds-llc/liquidfun-rs/ci.yml?style=flat-square&logo=github&label=CI)](https://github.com/bright-builds-llc/liquidfun-rs/actions/workflows/ci.yml)
[![License](https://img.shields.io/github/license/bright-builds-llc/liquidfun-rs?style=flat-square)](./LICENSE)
[![Rust 1.97.0](https://img.shields.io/badge/Rust-1.97.0-000000?logo=rust&logoColor=white)](https://www.rust-lang.org/)
[![Bright Builds: Rules](https://raw.githubusercontent.com/bright-builds-llc/bright-builds-rules/main/public/badges/bright-builds-rules-flat.svg)](https://github.com/bright-builds-llc/bright-builds-rules)

<!-- bright-builds-rules-readme-badges:end -->

An experimental, renderer-neutral Rust implementation of Google's LiquidFun
physics engine for learning and playful simulations, developed with a pinned
C++ reference available for optional comparisons.

## Web playground

The playground's native scenes are listed in the demo gallery. Visitors can
open the hosted playground at
<https://bright-builds-llc.github.io/liquidfun-rs/#/scene/dam-break>.
The recorded Phase 19 live revision is source
`d3d8688dabbacd54a6b0fa5fc6a055082f0bcf9e`. Production JavaScript and WASM
load under `/liquidfun-rs/`. GitHub Pages delivery is website hosting, not
Linux native qualification. This playground does not publish an npm or Rust
package, claim complete LiquidFun parity, or treat catalog cards as live
simulations. Color mixing is contact-driven particle color, not pigment
chemistry. Local Chromium `just web-player-smoke` remains the ordinary
WEBTEST-01 gate and does not claim Firefox or Safari coverage.

### Demo gallery

<!-- readme-svg-gallery:begin -->

Each preview is a 60 fps animated WebP rasterized from the committed 10 second
SVG. The frame is 1280 by 960, wireframe, with the identity camera. Scenes
that stay still until you act include one cue. Select a title or preview to
open the live scene. The SVG link under each preview is the vector source. A
post-merge workflow regenerates these files and commits them when the bytes
change.

#### [Dam Break](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/dam-break)

[![Dam Break simulation preview](docs/assets/readme/dam-break-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/dam-break)

[Animated SVG](docs/assets/readme/dam-break-10s.svg)

#### [Fountain](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/fountain)

[![Fountain simulation preview](docs/assets/readme/fountain-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/fountain)

[Animated SVG](docs/assets/readme/fountain-10s.svg)

#### [Float or Sink](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/float-or-sink)

[![Float or Sink simulation preview](docs/assets/readme/float-or-sink-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/float-or-sink)

[Animated SVG](docs/assets/readme/float-or-sink-10s.svg)

#### [Color Mixer](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/color-mixer)

[![Color Mixer simulation preview](docs/assets/readme/color-mixer-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/color-mixer)

[Animated SVG](docs/assets/readme/color-mixer-10s.svg)

#### [Jelly Drop](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/jelly-drop)

[![Jelly Drop simulation preview](docs/assets/readme/jelly-drop-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/jelly-drop)

[Animated SVG](docs/assets/readme/jelly-drop-10s.svg)

#### [Water Wheel](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/water-wheel)

[![Water Wheel simulation preview](docs/assets/readme/water-wheel-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/water-wheel)

[Animated SVG](docs/assets/readme/water-wheel-10s.svg)

#### [Particles](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/particles)

[![Particles simulation preview](docs/assets/readme/particles-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/particles)

[Animated SVG](docs/assets/readme/particles-10s.svg)

#### [Liquid Timer](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/liquid-timer)

[![Liquid Timer simulation preview](docs/assets/readme/liquid-timer-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/liquid-timer)

[Animated SVG](docs/assets/readme/liquid-timer-10s.svg)

#### [Surface Tension](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/surface-tension)

[![Surface Tension simulation preview](docs/assets/readme/surface-tension-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/surface-tension)

[Animated SVG](docs/assets/readme/surface-tension-10s.svg)

#### [Elastic Particles](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/elastic-particles)

[![Elastic Particles simulation preview](docs/assets/readme/elastic-particles-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/elastic-particles)

[Animated SVG](docs/assets/readme/elastic-particles-10s.svg)

#### [Rigid Particles](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/rigid-particles)

[![Rigid Particles simulation preview](docs/assets/readme/rigid-particles-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/rigid-particles)

[Animated SVG](docs/assets/readme/rigid-particles-10s.svg)

#### [Soup](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/soup)

[![Soup simulation preview](docs/assets/readme/soup-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/soup)

[Animated SVG](docs/assets/readme/soup-10s.svg)

#### [Soup Stirrer](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/soup-stirrer)

[![Soup Stirrer simulation preview](docs/assets/readme/soup-stirrer-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/soup-stirrer)

[Animated SVG](docs/assets/readme/soup-stirrer-10s.svg)

#### [Impulse](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/impulse)

[![Impulse simulation preview](docs/assets/readme/impulse-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/impulse)

[Animated SVG](docs/assets/readme/impulse-10s.svg)

#### [Wave Machine](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/wave-machine)

[![Wave Machine simulation preview](docs/assets/readme/wave-machine-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/wave-machine)

[Animated SVG](docs/assets/readme/wave-machine-10s.svg)

#### [Theo Jansen](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/theo-jansen)

[![Theo Jansen simulation preview](docs/assets/readme/theo-jansen-10s.webp)](https://bright-builds-llc.github.io/liquidfun-rs/#/scene/theo-jansen)

[Animated SVG](docs/assets/readme/theo-jansen-10s.svg)

<!-- readme-svg-gallery:end -->

The private, unpublished `liquidfun-wasm` wrapper and SolidJS player rebuild
from a clean checkout with the exact local tools:

```bash
rustup target add wasm32-unknown-unknown --toolchain 1.97.0
cargo install wasm-pack --version 0.15.0 --locked
curl -fsSL https://bun.com/install | bash -s "bun-v1.4.2"
cd web && bun install --frozen-lockfile
cd .. && just web-build
just web-player-smoke
```

`just web-build` regenerates the current checkout's WASM package and the
production site. `just web-player-smoke` is the local WEBTEST-01 gate: Chromium
against the production-base `/liquidfun-rs/` build exercises every catalog scene,
playback and reset, one representative pointer gesture plus a labeled control,
pointercancel, resize-then-drag, hidden-tab recovery, and a 375px keyboard and
page-scroll pass. It does not claim Firefox or Safari coverage. `just web-smoke`
is historical Phase 16 forensic chrome. It allocates
`target/phase16/closure-attempt-N`, sets `PHASE16_CLOSURE_ATTEMPT_DIR`, and
still looks for Dispose-session and PNG-hash selectors. It is
not the v1.1 product gate and is
not expected to pass against current Play/Pause/Reset chrome.
Ordinary playground proof is `just web-player-smoke`.

`just readme-svg` rebuilds the WASM package, writes one 10 second animated SVG
per catalog scene under `docs/assets/readme/`, rasterizes each SVG to a 60 fps
animated WebP, and upserts the demo gallery. The post-merge Readme scene
preview workflow runs the same command and commits when the export changes. A
new scene also needs a plan in `web/scripts/readme-svg/plans.ts`, including a
cue when the default scene does not move on its own.

`just demo-media` and `just demo-media-check` remain optional local MP4 and
WebP captures into `docs/assets/demos`. They are not the README gallery. They
require `ffmpeg`, `ffprobe`, and `webpmux` on `PATH`.

`web/src/generated/liquidfun-wasm`, `web/dist`, Playwright output, and
`target/web-build` are ignored and regenerated. Ordinary native builds and the
packaged `liquidfun` crate require none of Bun, Chromium, wasm-pack, C++, or
the upstream checkout.

## Hobby development

The goal is an enjoyable, useful experimental physics project. Linux x64
qualification and a dedicated benchmark machine are optional; they do not block
ordinary development or completion. Start with the Cargo-only commands below.
Current Cargo CI runs one macOS smoke job; cross-platform and expensive checks are manual options.
See [project scope](PROJECT-SCOPE.md) for accepted decisions and the
[experimental preparation checklist](RELEASE.md#experimental-package-preparation).
APIs may evolve; incompatible changes will be documented before release.
Safe handles, checked mutation and native Cargo-only consumption remain core boundaries.

## Maturity and evidence

The publishable crate is still version `0.0.0`; this repository has not declared
a parity-bearing v1 release candidate. The native scalar engine includes math,
collision, rigid bodies, contacts, CCD, all eleven joint kinds, standalone rope,
particles and groups, queries, semantic observations, debug primitives, and
safe owned particle-buffer transfer.

Capability is not the same as verified parity. The generated
[compatibility inventory](COMPATIBILITY.md) is authoritative for row-by-row
implementation, differential, platform, and documented-difference evidence.
Historical Phase 4 through Phase 8 corpora remain bounded evidence inputs, not
a generalized claim about the complete project. Performance claims likewise
apply only to immutable reports for named workloads.

The optional strict qualification profile for a parity-bearing release requires a frozen full candidate commit and a complete
reviewed manifest accepted by fail-closed `cargo xtask release audit`. This
checkout is **not release-ready**: it has no completed full-SHA
`release-candidate` workflow run, retained complete evidence bundle, or tracked
source/manifest/report records accepted by
`cargo xtask release attestation validate`. Local green checks do not substitute
for that run-bound attestation. See [RELEASE.md](RELEASE.md) for the
non-publication rule and the exact path to a future readiness claim.

## Cargo-only install and use

The crate declares Rust 1.92.0 as its minimum compiler; verify that minimum
before publication. A durable MSRV guarantee is deferred. Repository
development is reproducibly pinned to Rust 1.97.0 by `rust-toolchain.toml`.
Until a public release is published, build the reviewed repository checkout:

```bash
cargo build -p liquidfun
cargo test -p liquidfun --all-features
```

Ordinary use is Cargo-only. It does not initialize the upstream submodule,
discover CMake, compile C++, start an oracle process, or include the private
testbed:

```rust
use liquidfun::math::Vec2;
use liquidfun::{BodyDef, BodyType, World};

fn main() -> Result<(), Box<dyn std::error::Error>> {
    let mut world = World::new()?;
    let body = world.create_body(&BodyDef::new(
        BodyType::Dynamic,
        Vec2::ZERO,
        0.0,
        true,
    )?)?;
    assert!(world.contains_body(body));
    Ok(())
}
```

## Platform support

Ordinary CI exercises macOS; this is tested coverage, not a platform warranty.
Non-macOS platforms are best effort and can be tested manually as needed.
This table describes the existing optional strict platform profile. Linux x64
is not a requirement for hobby-project completion; the recorded results remain
scoped evidence, not a promise about every later revision.

Every supported lane verifies the same reviewed `.crate` bytes. Platform
results are D2 portability evidence and cannot create or promote canonical D1
physics fixtures.

| Target                      | Policy tier             | Current contract                                                                                                              |
| --------------------------- | ----------------------- | ----------------------------------------------------------------------------------------------------------------------------- |
| `x86_64-unknown-linux-gnu`  | durable supported       | Rust 1.97 native verification; canonical Linux also verifies Rust 1.92.0                                                      |
| `aarch64-unknown-linux-gnu` | durable supported       | Rust 1.97 native verification                                                                                                 |
| `aarch64-apple-darwin`      | durable supported       | Rust 1.97 native verification                                                                                                 |
| `x86_64-pc-windows-msvc`    | durable supported       | Rust 1.97 native verification                                                                                                 |
| `x86_64-apple-darwin`       | `conditional_supported` | Requires native evidence no older than 90 days; missing or expired evidence downgrades the current disposition to unsupported |

Targets outside this table are evidence-only unless a reviewed support decision
promotes them.

## Headless, catalog, and testbed workflows

The catalog is the shared renderer-independent scenario authority:

```bash
cargo xtask catalog list
cargo xtask catalog run --scenario rigid-stack-stability --timestep 0.016666668 --velocity-iterations 8 --position-iterations 3 --particle-iterations 1 --oracle-preset oracle-debug --session-profile one-shot --output human --commands auto
```

The private testbed consumes the same semantic catalog and cannot confer parity
or performance authority:

```bash
cargo run -p liquidfun-testbed -- --capability-check --fixture crates/liquidfun-differential/tests/fixtures/catalog/phase11-v1.json --output target/testbed-capability
cargo run -p liquidfun-testbed --bin interactive
```

See [TESTING.md](TESTING.md) for replay, differential, sanitizer, fuzz, Miri,
coverage, benchmark, and evidence-promotion workflows.

## Optional C++ oracle

Maintainer-only differential work requires the exact recursive upstream
checkout, CMake 3.25 or newer, Ninja 1.11 or newer, and a compatible C++
compiler:

```bash
git submodule update --init --recursive third_party/liquidfun
cargo xtask upstream verify
cargo xtask upstream configure --preset oracle-debug
cargo xtask upstream build --preset oracle-debug
```

Canonical Linux evidence records the stricter pinned identities documented in
[UPSTREAM.md](UPSTREAM.md). The C++ oracle is out of process and never enters
the `liquidfun` crate.

## Contributing and licensing

Read [CONTRIBUTING.md](CONTRIBUTING.md) before changing source, evidence, or
generated reports. Original project work is MIT-licensed under [LICENSE](LICENSE).
Pinned upstream and derived materials retain separate attribution, alteration,
and notice duties recorded in
[THIRD_PARTY_NOTICES.md](THIRD_PARTY_NOTICES.md).

## Architecture and evidence

- [UPSTREAM.md](UPSTREAM.md) — immutable oracle identity, ancestry, notices,
  and intentional update policy
- [COMPATIBILITY.md](COMPATIBILITY.md) — generated inventory and explicit
  evidence gaps
- [ARCHITECTURE.md](ARCHITECTURE.md) — native Rust dependency direction and
  oracle-isolation boundary
- [TESTING.md](TESTING.md) — local commands, CI lanes, package proof, and
  deterministic verification policy
- [SAFETY.md](SAFETY.md) — handle, callback, owned-buffer, panic, and zero-unsafe
  contracts
- [RELEASE.md](RELEASE.md) — candidate freeze, audit, package reuse, and
  non-publication policy
