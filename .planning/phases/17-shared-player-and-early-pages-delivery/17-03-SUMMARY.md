---
phase: 17-shared-player-and-early-pages-delivery
plan: "03"
subsystem: infra
tags: [vite, github-pages, provenance, host-03, wasm]

requires:
  - phase: 16-rust-wasm-browser-bridge
    provides: Vite Solid app with wasm-pack ?url loader and just web-build
provides:
  - Production and preview Vite base /liquidfun-rs/
  - readBuildInfo Unavailable fallback and allowlisted GitHub URLs
  - just web-build VITE_* injection and dist path gate
affects: [17-04, 17-05, 17-07, pages-deploy, site-footer]

tech-stack:
  added: []
  patterns: [command-or-preview project base, injected VITE provenance, fail-closed dist gate]

key-files:
  created:
    - web/src/build-info.ts
    - web/tests/build-info.test.ts
  modified:
    - web/vite.config.ts
    - scripts/web-build.ts

key-decisions:
  - "Production and preview Vite base is /liquidfun-rs/; local serve stays /."
  - "Short commit labels are the first 12 lowercase hex characters of a full SHA."
  - "Commit and build URLs are accepted only on https://github.com/bright-builds-llc/liquidfun-rs/."
  - "just web-build fails unless dist/index.html contains /liquidfun-rs/assets/ and a .wasm file exists."

patterns-established:
  - "Vite base is command === build || isPreview ? /liquidfun-rs/ : / so local serve stays root."
  - "Provenance is injected VITE_* env; missing or blank fields render Unavailable."
  - "Same-checkout just web-build is the fail-closed HOST-03 asset-path gate."

requirements-completed: [HOST-03]
generated_by: gsd-execute-plan
lifecycle_mode: yolo
phase_lifecycle_id: 17-2026-09-17T11-24-10
generated_at: 2026-09-17T11:54:06Z

duration: 2min
completed: 2026-09-17
---

# Phase 17 Plan 03: Production Base And Provenance Summary

**Production Vite `base: "/liquidfun-rs/"`, `readBuildInfo()` Unavailable fallback, and a `just web-build` dist gate that rejects a root-base or wasm-less artifact.**

## Performance

- **Duration:** 2 min
- **Started:** 2026-09-17T11:51:40Z
- **Completed:** 2026-09-17T11:54:06Z
- **Tasks:** 2
- **Files modified:** 4

## Accomplishments

- Production and preview-of-production Vite builds prefix JS, CSS, and hashed WASM with `/liquidfun-rs/`; local `vite` serve stays `/`.
- `readBuildInfo` maps missing or blank version, commit, and build fields to `Unavailable`, shortens a 40-character lowercase hex SHA to 12 characters, and constructs commit/build URLs only on `https://github.com/bright-builds-llc/liquidfun-rs/`.
- `just web-build` injects `VITE_APP_VERSION`, `VITE_GIT_SHA`, `VITE_BUILD_ID`, and optional `VITE_BUILD_URL`, then fails unless `web/dist/index.html` contains `/liquidfun-rs/assets/` and `web/dist` contains a `.wasm` file.

## Task Commits

Each task was committed atomically:

1. **Task 1: Set the production Vite project base**
   - `bba2b2b` (`feat`) — `command === "build" || isPreview` uses `/liquidfun-rs/`
1. **Task 2: Inject provenance and fail a root-base dist**
   - `0bd5ecb` (`test`) — failing empty-env, SHA, and Actions URL cases
   - `43e1bd7` (`feat`) — `readBuildInfo` plus `just web-build` VITE_* injection and dist gate

**Plan metadata:** recorded in the plan-completion docs commit

## Files Created/Modified

- `web/vite.config.ts` — Production/preview project base `/liquidfun-rs/`
- `web/src/build-info.ts` — Safe provenance reader with Unavailable fallback
- `web/tests/build-info.test.ts` — Empty, version, SHA, and URL allowlist cases
- `scripts/web-build.ts` — VITE_* injection and post-Vite dist path gate

## Decisions Made

- Production and preview Vite base is `/liquidfun-rs/`; local serve stays `/` so hobby `just web-dev` is unchanged.
- Short commit labels are the first 12 lowercase hex characters of a full SHA, inside the 7–12 character chrome range.
- Commit and build URLs are accepted only on `https://github.com/bright-builds-llc/liquidfun-rs/`; `javascript:` and off-host values stay undefined.
- `just web-build` fails closed unless `dist/index.html` contains `/liquidfun-rs/assets/` and a hashed `.wasm` file exists.

## Deviations from Plan

None - plan executed exactly as written.

---

**Total deviations:** 0 auto-fixed
**Impact on plan:** None.

## Issues Encountered

None

## User Setup Required

None - no external service configuration required.

## Next Phase Readiness

Ready for 17-04. Footer chrome can consume `readBuildInfo()`. Pages workflow (17-07) can rely on the same-checkout dist gate; this plan did not create `pages.yml` or deploy.

---
*Phase: 17-shared-player-and-early-pages-delivery*
*Completed: 2026-09-17*

## Self-Check: PASSED
