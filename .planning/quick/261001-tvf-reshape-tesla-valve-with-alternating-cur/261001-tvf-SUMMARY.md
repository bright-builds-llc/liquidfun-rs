---
phase: quick-261001-tvf
plan: "01"
status: complete
generated_by: gsd-execute-plan
generated_at: "2026-10-01T21:55:55-05:00"
source_diff_sha256: b9c646fe9a40f438e69b2bce8d049bc875fcb1eea25cf42f2c4212a7e8a09f47
---

# Quick task 261001-tvf: Tesla Valve reference geometry

The valve now has four alternating rounded lobes, four closed solid teardrop splitters, and a winding central channel, following the supplied diagram in a downward gravity-driven orientation. Authored points feed both drawing and convex collision fixtures. The source stays centered, rate and direction controls remain live, and both orientations fit the existing world rectangle with 60 drawing segments. The catalog thumbnail, copy, centered phone frame and README SVG/WebP reflect the new geometry.

AGENTS.md standing authorization, AGENTS.bright-builds.md, standards-overrides.md, PROJECT-SCOPE.md, and the local architecture, code-shape, testing, verification and language standards informed this work. All active global/repository lessons were loaded. No release or complete real-world fluid-diode performance is claimed.

## Verification

- Required core checks passed in order: `cargo fmt --all`, `cargo clippy --all-targets --all-features -- -D warnings`, `cargo build --all-targets --all-features`, and `cargo test --all-features`. Clippy/build/tests used the known working isolated `CARGO_TARGET_DIR=target/main-pull-resolution/native-check`; 1032 core tests passed. Logs: `target/tesla-valve-validation/core-*.log`.
- Affected-package strict Clippy and all-target build passed. `cargo test -p liquidfun-wasm --all-features` passed all 287 tests in 71.05 seconds, including nine Tesla Valve tests. Logs: `target/tesla-valve-validation/wasm-*.log`.
- `just web-build` passed fresh WASM generation, TypeScript checking, all 402 web unit tests and the production build. `bun run test:player` passed all 44 Chromium tests in 39.2 seconds. Browser log: `target/tesla-valve-validation/browser-test.log`.
- Managed Bright Builds checks, `just markdown-check`, and `git diff --check` passed.
- The original two-head construction failed the new four-lobe assertion before implementation. New regressions check closed solid islands, both orientations, 60-segment frame capture, first drain arrival, per-four-step containment, unchanged 80-particle directional margin after six seconds, a ten-second run/stop drainage window, and preservation of zero/nonzero selected rates across flips.
- Actual browser inspection covered desktop and 390×844 phone framing, forward flow, reverse flow, a higher rate of 420 particles/s, zero emission, and a return to forward while retaining zero. Both layouts retain the meter scale legend and keep walls above transport/flow controls. Screenshots: `target/tesla-forward-attempt2.png`, `target/tesla-phone-attempt2.png`, `target/tesla-reverse-phone.png`, `target/tesla-high-rate-phone.png`, and `target/tesla-static-phone-final.png`.
- The filtered README capture recorded 600 frames at 60 fps for ten seconds and regenerated only Tesla Valve SVG/WebP. The WebP is 6,766,564 bytes and passes the recorder's byte limit.

## Bounded adjustments and retained diagnostics

Four stages require a longer transit window than the former two heads: the widened three-second candidate held 527 forward versus 540 reverse particles. The six-second regression preserves the original 80-particle margin and additionally verifies earlier drain arrival and containment after every four steps, so escapes cannot explain the result. A separate stop-source test proves continued drainage through ten seconds. Narrow passages and invalid thin ribbon polygons were diagnosed and corrected before final validation.

The first local gallery attempt failed because its font path was Linux-only. `LIQUIDFUN_README_FONT_FILE` now accepts an explicit installed font path while preserving the DejaVu CI default. Local media uses `/System/Library/Fonts/Supplemental/Arial.ttf`; glyph rasterization can differ from canonical DejaVu capture. The missing-font diagnostic is retained at `target/tesla-valve-validation/gallery-missing-font-attempt1.log`. The early failed browser construction screenshot is retained at `target/tesla-forward-desktop.png` and is not passing evidence.

## Independent review and finalization

Separate AI reviewer `/root/tesla_review` acknowledged exact source diff SHA256 `b9c646fe9a40f438e69b2bce8d049bc875fcb1eea25cf42f2c4212a7e8a09f47` at actual review time 2026-10-02 02:53:08 UTC, with no actionable findings. The complete relevant source/media diff and passing evidence were inspected; this is an AI acknowledgment. Record: `target/tesla-valve-validation/review.md`. The exact source binary diff is retained at `target/tesla-valve-validation/source-diff.patch`.

Source commit: `7d7ca9dac4fd2e8ec89f824c4987d17cefeac1ea` (`fix(playground): match Tesla Valve reference geometry`). Standing owner authorization is the authority for ordinary main-branch publication. The companion tracking commit records the completed checks and review; the final chat reports the actual main push result.
