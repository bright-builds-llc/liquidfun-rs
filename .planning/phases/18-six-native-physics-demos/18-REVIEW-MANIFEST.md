---
phase: 18-six-native-physics-demos
plan: "10"
generated_by: gsd-code-reviewer
lifecycle_mode: yolo
phase_lifecycle_id: 18-2026-09-18T03-36-16
generated_at: 2026-09-18T05:45:27Z
---

# Phase 18 review manifest

Fixed ordered manifest for independent AI review after WR-01 wood-default
alignment (commit `504d720`). The implementing agent does not acknowledge or
approve this digest.

## Digest method

For each path below in the listed order, concatenate UTF-8 path bytes, one NUL
byte, the lowercase SHA-256 of the exact file bytes as ASCII hex, and one LF
byte. The review digest is the lowercase SHA-256 of that complete
concatenation.

review_digest: 1f6349eb4a080179aad2dbf9af670e4cf3e95eecf3a0c725449eb3da21acaa94

## Files

- `crates/liquidfun-wasm/src/scene/dam_break.rs` `e3c17af104221bbf025a98683f07a51ddc62fb3269c85a28fcf052cdba7d2c17`
- `crates/liquidfun-wasm/src/scene/fountain.rs` `15f819a54635ecadb696b8495b7c49c668b7e0315dc0cd6a56ef5aaeef67c804`
- `crates/liquidfun-wasm/src/scene/float_or_sink.rs` `670bb4b707f886fdafbd9fa5c7d3c7749121bb36fd62305b1ebeecff5c89d2cf`
- `crates/liquidfun-wasm/src/scene/color_mixer.rs` `e08748d9621a5a3f005fd66ac7d1ec8178ca8d1e2c5ab9de6f1ce7cf8b1cbcf2`
- `crates/liquidfun-wasm/src/scene/jelly_drop.rs` `a2a118fc12eb18b0ab426001017b35751affde51406b764276aef99b0246bb3b`
- `crates/liquidfun-wasm/src/scene/water_wheel.rs` `795e79f2b0f6c0a96e76ef12df126d06ede7d71c2b6cee010ffd874152547394`
- `crates/liquidfun-wasm/src/scene.rs` `ab832635482ba42c28c559d3c79e59e68dd4e86335825dddfc7ca4987861fb59`
- `web/src/catalog/scenes.ts` `3f5aa493ecd19dbb983af83def89e9b4858d03e6cb974be478b5847c034d2e2d`
- `web/src/components/CatalogNav.tsx` `6fa36d6c51b7cb3dece71960c7bd2e73b45c28d1f63fbe8a08d17e39ca2b25e1`
- `web/src/components/SceneControls.tsx` `a6db74ec02ef11ce60fc946b2e4fe04c46fccba09e1438e7a66f22aabd65468a`
- `web/src/components/scene-controls.ts` `ee28e631fa1c4aafad283adb28b49436211400a7b7a69639addfdd9df9630241`
- `web/src/components/SceneCredits.tsx` `831de3931faa4874a3b51d58bc204a875b78dc67a4b697bc155ed86101dc7bb4`
- `web/src/components/scene-credits.ts` `99be6d199f7fde16362c54050fff81aa2d357e27adde803ef5a8732a64a47ed9`
- `web/src/App.tsx` `15928510bead08587ed45ba1050a4621c0f31a7761ba913de4376f889d5bcb7a`
- `web/e2e/player.spec.ts` `583766ceccbf1b051873400fc987cc13e0dee5c86128f9089554ba6f1de93bc9`
- `target/web-build/web-build.log` `a1b87af09711dd072c041973caf29a19030ec3f3032ddf23acbdb0b396415610`

Manifest entry count: `16`.

## Evidence notes

- Implementing task-1 commit: `1a69ff3bc1b03039818bd621983cfbe394fdd244`
- WR-01 fix commit: `504d720f9e59ea8331e698c23de043fee60b6590`
- Only `web/src/components/scene-controls.ts` changed among the 16 hashed
  files versus digest
  `22635277d29513bb6ffa83a1b51c13f0207ea63b0f402a852d30a660413a618c`.
- `just web-player-smoke` exited 0 and wrote ignored
  `target/web-build/web-build.log` ending in `complete player-smoke`.
- This is local Chromium proof, not a Pages deploy, crate publication, or
  WEBTEST-01 pointer matrix.
