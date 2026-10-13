# Phase 41: Level Select and Progress - Discussion Log

> **Audit trail only.** Do not use as input to planning, research, or execution agents.
> Decisions are captured in CONTEXT.md — this log preserves the alternatives considered.

**Date:** 2026-10-13
**Phase:** 41-level-select-and-progress
**Mode:** Yolo. Recommended answers were auto-selected with no user prompts, and advisor research agents were skipped to keep the process light.
**Areas discussed:** Route and navigation, Grid content and locks, Progress model and unlocks, Storage schema and degraded mode, Saved placements and level change detection

---

## Route and navigation

| Option | Description | Selected |
|--------|-------------|----------|
| Text-only sidebar link above Demos | Keeps the shell spec's preview and icon counts valid | ✓ |
| Header action link | Visible on every page, but crowds the header next to GitHub | |
| Sidebar section listing every level | More links, duplicates the grid, and breaks the sidebar counts | |

**User's choice:** auto: text-only sidebar link, plus All levels links in the Puzzle options drawer, the unknown-puzzle fallback and the last level's Next slot.

## Grid content and locks

| Option | Description | Selected |
|--------|-------------|----------|
| Locked cards not clickable; deep links still open | The lock means something in the grid while staying soft | ✓ |
| Locked cards clickable with a warning | Weak lock that adds a confirm step | |
| Show par and tool budget | Needs a TS mirror of Rust data | |

**User's choice:** auto: show the title, the idea and three labelled star slots. Locked cards say "Solve {previous} to unlock".

## Progress model and unlocks

| Option | Description | Selected |
|--------|-------------|----------|
| Unlock level i when level i−1 has Win | Linear and simple, matching the owner's decision | ✓ |
| Unlock by total stars | Not what the owner chose | |

**User's choice:** auto: StarStore is hydrated from storage, records write through with a read-merge-write, Next always links to the next level, and on the last level Next becomes All levels.

## Storage schema and degraded mode

| Option | Description | Selected |
|--------|-------------|----------|
| One versioned JSON key; any failure turns on read-only degraded mode | Never destroys corrupt or future-version data | ✓ |
| Overwrite corrupt data with a fresh record | Contradicts the success criterion and loses data | |
| Per-level keys | More keys and harder versioning | |

**User's choice:** auto: `liquidfun.puzzle-progress.v1`. A probe write runs at startup, and a mid-session write failure also switches to degraded mode. The note shows in the grid and in the Puzzle options drawer, never in the canvas HUD.

## Saved placements and level change detection

| Option | Description | Selected |
|--------|-------------|----------|
| Rust FNV-1a 64 level hash over the `LevelSpec` Debug encoding, pinned by a test | Automatic and authoritative; the pin forces a conscious update | ✓ |
| Hand-bumped revision number in TS | Easy to forget | |
| TS hash of the `levelInfo()` getters | Misses fields that aren't exposed | |

**User's choice:** auto: save after each committed edit and at Play. Restore precedence is test seam, then matching saved placements, then empty. A mismatch or malformed data is discarded silently, and stars are kept.

## Claude's Discretion

- Names, file locations, copy, an optional best-fill line, the exact hash encoding, and card visuals.

## Deferred Ideas

- A progress reset or export control, par or the tool mix on cards, a README puzzle gallery, migration of the hint key, and the Phase 40 tray-truncation item.
