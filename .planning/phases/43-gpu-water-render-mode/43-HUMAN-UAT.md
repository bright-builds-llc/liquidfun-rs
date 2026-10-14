---
status: complete
phase: 43-gpu-water-render-mode
source: [43-VERIFICATION.md]
started: 2026-10-14T01:09:19Z
updated: 2026-10-14T01:09:57Z
---

## Current Test

[testing complete]

## Tests

### 1. iPhone Water mode (D-20)

expected: On a real iPhone, play Dam Break, Fountain and First Pour in Water mode. Water draws with foam, and the cup, meter and tools are drawn on top. If the device can't run Water, the view falls back to Shaded blob or the canvas and is never blank.
result: pass
verified_by: user (repository owner, chat approval "approved, please push it" at 2026-10-14T01:09:57Z)

### 2. Look review on a real GPU

expected: On a desktop GPU, compare Water with Shaded blob. Refraction, highlights and tint are visible, and foam reads as white spray on fast jets while slow, pooled water stays clear and tinted.
result: pass
verified_by: user (repository owner, chat approval "approved, please push it" at 2026-10-14T01:09:57Z)

## Summary

total: 2
passed: 2
issues: 0
pending: 0
skipped: 0
blocked: 0

## Gaps
