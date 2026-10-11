---
id: SEED-002
status: dormant
planted: 2026-10-11
planted_during: between milestones (v1.4 archived 2026-10-11)
trigger_when: The next milestone is about user-created content, scene authoring, or sharing scenes
scope: Large
---

# SEED-002: Sandbox editor with shareable links

## Why This Matters

Let people build their own scene: place walls, emitters, bodies, and joints, pick particle materials, then press play. Encode the scene in the URL so a link reproduces it. Because the engine steps deterministically, the recipient sees the same simulation the sender saw, which shows off the project's determinism priority in a way people can feel. The editor also becomes the authoring tool for puzzle levels (SEED-001) and new scenes.

## When to Surface

**Trigger:** The next milestone is about user-created content, scene authoring, or sharing scenes

This seed should be presented during `/gsd-new-milestone` when the milestone
scope matches any of these conditions:
- Building an in-browser scene or level editor
- Serializing scenes to a URL, file, or other shareable form
- Puzzle mode (SEED-001) needs a level-authoring workflow

## Scope Estimate

**Large** — a full milestone. Likely phases: a versioned scene description format and URL encoding; an editor tool palette and selection/move; material and emitter properties; share and load with a replay check.

## Breadcrumbs

- `web/src/routing/hash.ts` — current `#/scene/<id>` routing a share link would extend
- `web/src/catalog/scene-records.ts` — the shape a user-authored scene must match
- `web/src/export/` — existing record and animated-SVG export pipeline
- `crates/liquidfun-wasm/src/scene.rs` — native scene construction the description must drive
- Drawing Particles scene — closest existing "author while playing" interaction

## Notes

Keep the format versioned from day one so old links keep loading after the engine evolves; record the engine revision in the link if replay must stay bit-identical.
