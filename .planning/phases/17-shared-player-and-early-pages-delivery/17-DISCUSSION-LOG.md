# Phase 17: Shared Player and Early Pages Delivery - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-09-17
**Phase:** 17-shared-player-and-early-pages-delivery
**Mode:** Yolo
**Areas discussed:** First hosted scene and honest catalog, Stable URLs and unknown-scene fallback, Shared player chrome, Session teardown and hidden-tab bounding, GitHub Pages delivery, Verification boundary

---

## First hosted scene and honest catalog

### Which scene ships first?

| Option | Description | Selected |
| --- | --- | --- |
| Dam Break from the Phase 16 basin | Matches FEATURES research ("put Dam Break into the shared player") and keeps one real native scene. | ✓ |
| Keep the unnamed proof scene | Faster, but fails WEB-02/WEB-03 product naming and shareable first-scene identity. | |
| Implement all six scenes now | Crosses into Phase 18 DEMO requirements. | |

**Choice:** Dam Break as the first hosted working scene.
**Notes:** `[auto]` recommended default. Other approved names appear as honest not-ready entries only.

### How complete is the catalog?

| Option | Description | Selected |
| --- | --- | --- |
| Thin named list with honest not-ready labels | Satisfies the roadmap note without claiming WEB-01 card/preview completeness. | ✓ |
| Full six-card gallery with previews | Owned by Phase 18 WEB-01. | |
| Dam Break only, hide other names | Under-communicates the approved catalog and makes later URLs surprising. | |

**Choice:** Thin catalog/navigation listing all six names; only Dam Break is playable.
**Notes:** `[auto]` recommended default.

---

## Stable URLs and unknown-scene fallback

### Navigation scheme

| Option | Description | Selected |
| --- | --- | --- |
| Hash routes (`#/scene/dam-break`) | Works on GitHub Pages without a rewrite rule. | ✓ |
| Query string (`?scene=dam-break`) | Also static-host safe, but easier to lose behind index redirects. | |
| Path routes (`/scene/dam-break`) | Needs a server fallback GitHub Pages does not provide. | |

**Choice:** Hash-based scene navigation with production Vite base `/liquidfun-rs/`.
**Notes:** `[auto]` recommended default from ARCHITECTURE.md and PITFALLS.md.

### Unknown scene identifier

| Option | Description | Selected |
| --- | --- | --- |
| Useful catalog/fallback plus path back to Dam Break | Matches WEB-02. | ✓ |
| Silent redirect to Dam Break | Hides the bad identifier and makes shared typos confusing. | |
| Blank/error-only canvas | Fails WEB-02 and WEB-06. | |

**Choice:** Explain the unknown id in a catalog/fallback view with a working Dam Break path.
**Notes:** `[auto]` recommended default.

---

## Shared player chrome

### Playback controls

| Option | Description | Selected |
| --- | --- | --- |
| Play, pause, and reset-to-initial-state | Matches WEB-03; reset disposes and recreates. | ✓ |
| Keep Phase 16 Dispose-only proof button | Does not give visitors playback control. | |
| Add per-scene sliders now | WEB-04 belongs to Phase 18. | |

**Choice:** Shared play/pause/reset with visible loading and retryable failure.
**Notes:** `[auto]` recommended default.

### Visual stack

| Option | Description | Selected |
| --- | --- | --- |
| Evolve Phase 16 semantic HTML + scoped CSS | Ships the player without a design-system adoption. Record as a thin-slice exception. | ✓ |
| Adopt MysticUI + Tailwind 3 now | Matches the managed default but delays first Pages delivery. | |
| Introduce a second CSS framework | Extra churn with no product need. | |

**Choice:** Keep semantic HTML and scoped CSS; defer MysticUI to a later catalog-card decision.
**Notes:** `[auto]` recommended default. STACK.md asked for an explicit no-library exception.

### Source and build chrome

| Option | Description | Selected |
| --- | --- | --- |
| Site-level repo, FOSS, version, commit, build, Peter/OpenLinks | Required by frontend-ui.md for public OSS apps; WEB-08 per-scene credits wait. | ✓ |
| Defer all chrome to Phase 18 | Would ship a public Pages site without source/provenance disclosure. | |
| Full per-scene inspiration/notice pages now | WEB-08 is Phase 18. | |

**Choice:** Site-level source/build chrome now; per-scene credits later.
**Notes:** `[auto]` recommended default. MIT license supports "free and open source".

---

## Session teardown and hidden-tab bounding

### Ownership

| Option | Description | Selected |
| --- | --- | --- |
| One session, generation token, dispose on reset/leave/change | Prevents leaked worlds and duplicate loops (WASM-04). | ✓ |
| Allow overlapping loads until the latest wins visually | Can leave orphaned WASM worlds stepping. | |
| Rely on page unload only | Fails in-app reset and route changes. | |

**Choice:** Exactly one world and animation loop; stale async results discarded.
**Notes:** `[auto]` recommended default.

### Hidden tabs and catch-up

| Option | Description | Selected |
| --- | --- | --- |
| Pause while hidden, clear catch-up, cap at four steps/frame | Matches ARCHITECTURE.md and WASM-04. | ✓ |
| Keep Phase 16 unconstrained rAF stepping while hidden | Background tabs can accumulate catch-up debt. | |
| Multi-second catch-up to "stay honest" with wall clocks | Causes particle bursts and jank. | |

**Choice:** Pause hidden documents, clear accumulated time on resume, cap steps per frame.
**Notes:** `[auto]` recommended default.

---

## GitHub Pages delivery

### Workflow shape

| Option | Description | Selected |
| --- | --- | --- |
| Every main push, same-checkout WASM+site, deploy after checks, no PAT | Matches HOST-01/HOST-02/HOST-03. | ✓ |
| Path-filtered Pages job | Can skip a main push that still needs a site rebuild. | |
| Personal token deploy | Forbidden by HOST-02. | |

**Choice:** SHA-pinned Actions Pages workflow, `github-pages` environment, latest-main protection, recorded URL/revision.
**Notes:** `[auto]` recommended default. Website runner is not Linux native qualification.

---

## Verification boundary

| Option | Description | Selected |
| --- | --- | --- |
| Player + production-base + first live Pages evidence | Closes this phase's success criteria. | ✓ |
| Wait for all six scenes before deploying | Leaves HOST-* unverified and contradicts the early-slice roadmap. | |
| Full WEBTEST-01 six-scene pointer matrix now | Phase 19. | |

**Choice:** Prove Dam Break player, fallback, cleanup, hidden-tab bounding, and one recorded Pages deployment.
**Notes:** `[auto]` recommended default. Independent AI review remains required; self-approval is not allowed.

## Claude's Discretion

Exact copy, Dam Break tuning within documented reset, workflow job names, and local production-base preview recipe.

## Deferred Ideas

Phase 18 catalog/demos/credits; Phase 19 pointer/accessibility/six-scene smoke; MysticUI adoption; workers/threads/WebGPU/publication.
