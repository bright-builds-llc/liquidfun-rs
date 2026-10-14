# Phase 44: v1.5 Polish and Test Gaps - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md. This log keeps the alternatives that were considered.

**Date:** 2026-10-14
**Phase:** 44-v1-5-polish-and-test-gaps
**Mode:** Yolo. Recommended answers were chosen automatically. The owner had already selected the scope from the audit: phone tray and accessibility, test gaps, and the typecheck include.
**Areas discussed:** Phone tray, Valve accessibility, E2E coverage, E2E typecheck

---

## Phone tray

| Option | Description | Selected |
| --- | --- | --- |
| Change the chip's inner layout and keep the tray box | Keeps the HUD obstacle contract and UI-SPEC fit checks | ✓ |
| Widen the tray box | Breaks the portrait fit table and clearance to neighbouring leaves | |
| Hide names in portrait (icon and count only) | Removes truncation but drops visible names on phones | |

## Valve accessibility

| Option | Description | Selected |
| --- | --- | --- |
| Use the readout as the group's `aria-describedby` and add no live region | Current value is read with the group, and changes are announced once by the existing announcer | ✓ |
| Make the readout an `aria-live` region | Changes would be announced twice | |

**Also:** cap the TS tick at 255 (matches Rust `u8`), and fold in IN-06 (the seam default for `openTick`).

## E2E coverage

| Option | Description | Selected |
| --- | --- | --- |
| Win with the seam, delete the seam global, click Next, assert the building state on level 2 | Fast, and avoids the cross-level seam throw | ✓ |
| Build level 1 through the UI before clicking Next | Slower, and adds no coverage of the Next path | |
| Timed-gate at 1×, 2× and 4×, comparing run fingerprints | Covers audit finding F3 | ✓ |

## E2E typecheck

| Option | Description | Selected |
| --- | --- | --- |
| Separate `tsconfig.e2e.json`, with `typecheck` running both projects | Keeps app types (`vite/client`) apart from Node and Playwright types | ✓ |
| Add e2e to the main include | Mixes runtime type environments | |
| Loosen flags or use `ts-ignore` | Rejected: fixes must address the root cause | |

## Claude's Discretion

- The chip layout technique, the e2e file placement, and how the valve fast-forward test shares its build.

## Deferred Ideas

- The broader review-note sweep, re-authoring the first-pour frame, and a Water probe retry.
