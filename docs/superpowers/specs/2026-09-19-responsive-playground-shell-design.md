# Responsive Playground Shell Design

**Date:** 2026-09-19
**Status:** Approved for implementation

## Goal

Refactor the web playground into an application shell where demo navigation is
separate from the active simulation. A static top header identifies the
project, a persistent desktop sidebar lists the six demos, and the main column
contains the selected demo. Narrow screens use an accessible mobile drawer.

## Library decision

Add exact production dependency `@kobalte/core` 0.13.12. Kobalte is an MIT
licensed, unstyled SolidJS accessibility-primitives library compatible with
the repository's SolidJS version.

Use Kobalte only where it supplies behavior that semantic HTML and CSS do not:
the mobile demo drawer uses `Dialog` from `@kobalte/core/dialog` for focus
trapping, screen-reader announcements, Escape handling, inert background,
scroll locking, overlay dismissal, and trigger-focus restoration.

Do not use Kobalte NavigationMenu for the scene list. The scene choices are
direct page-navigation links rather than flyout-menu commands. Both desktop and
mobile lists remain semantic `<nav>`, `<ul>`, and `<a>` elements.

Update `standards-overrides.md` to replace the Phase 17 no-library thin-slice
decision with the owner-approved Kobalte Dialog plus semantic/CSS shell
decision. No Tailwind, MysticUI, community drawer package, or second UI
dependency is introduced.

## Desktop shell

At viewport widths above 768 CSS pixels:

1. A top header spans the application width and remains sticky at the viewport
   top.
1. The content region is a two-column grid with a 280-pixel navigation sidebar
   and a flexible main column.
1. The sidebar remains sticky below the header, has bounded viewport height,
   and scrolls internally when needed.
1. The main column contains only the selected player or route fallback, not the
   demo catalog.
1. The existing footer spans the full shell below the content region.

The shell may grow beyond the old 960-pixel page maximum while the player keeps
its renderer-responsive sizing and fixed world aspect ratio.

## Header

The header contains:

- `liquidfun-rs` as the single page `h1`
- the concise existing playground tagline
- a stable GitHub source link opening safely in a new tab
- a `Demos` trigger shown only at mobile widths

The header content is static across scenes. Build provenance and maintainer
identity remain in the footer.

## Demo navigation

Replace the six preview cards with compact route items. Each item contains:

- scene title
- one-line existing scene description
- a current-scene accent and `aria-current="page"` when selected

The entire item is one direct hash link. Static SVG thumbnails, `Static preview` copy, and separate `Open` buttons are removed. The current scene is
never inferred from focus or drawer state; routing remains authoritative.

The same `DemoNavigation` component renders inside the desktop sidebar and
mobile drawer. It accepts an optional navigation callback so mobile selection
can close the drawer after the hash link activates.

## Mobile drawer

At 768 CSS pixels and below:

- hide the desktop sidebar
- show the header's `Demos` Kobalte Dialog trigger
- render the shared demo navigation in a left-side Dialog panel
- cap the panel width at the smaller of 320 pixels or 88 viewport-width units
- include an announced `Demos` title and a visible close button
- close on Escape, overlay click, close button, or scene selection
- restore focus to the Demos trigger after close
- prevent background scrolling while open

The drawer state is controlled by the shell and is not persisted. Hash changes
from outside the drawer also close it so stale navigation cannot cover a newly
selected scene.

## Routing behavior

Opening the playground without a hash automatically selects Dam Break. Use
history replacement rather than adding an extra browser-history entry. Update
the application route state in the same operation so no empty fallback flashes.

Known scene hashes open their selected scene as today. Unknown and not-ready
hashes retain the current useful fallback and keep navigation available.
Back/forward hash navigation remains authoritative and must not recreate the
same already-active scene unnecessarily.

## Component architecture

Create focused UI modules:

- `PlaygroundShell` owns controlled mobile Dialog state and overall layout
- `SiteHeader` owns static identity/source chrome and receives the Dialog
  trigger as composition
- `DemoNavigation` owns compact scene links and active-state semantics

`App` continues to own routing, one-session lifecycle, playback, rendering
mode, controls, and pointer forwarding. It supplies the selected scene ID and
main player/fallback content to the shell. The shell does not call WASM or own
simulation state.

Remove `CatalogNav` and the now-unused static preview module after all call
sites and tests migrate. Keep `App.tsx` below the managed 628-line trigger by
moving shell markup and empty-route normalization into focused modules or pure
helpers.

## Styling

Continue using the repository's single scoped `app.css` file and current dark
tokens. Add shell, header, sidebar, compact navigation-item, Dialog overlay,
Dialog positioner, and drawer styles. Preserve:

- visible 2-pixel focus rings
- 44-pixel minimum interactive targets
- sufficient active/current contrast
- canvas-only touch-action suppression
- page scrolling outside the open modal drawer
- no horizontal overflow at 375 pixels

Animations are optional and must honor reduced-motion preferences. Functional
drawer behavior must not depend on animation completion.

## Dependency and notice integrity

Pin `@kobalte/core` exactly in `web/package.json` and update `web/bun.lock`.
Record the dependency decision and MIT license in repository documentation
where the existing dependency/notice policy requires it. Ordinary Rust crate
consumers remain unaffected because the playground package is private and
unpublished.

## Verification

Unit and component-focused coverage must prove:

1. scene navigation renders all six canonical routes in catalog order
1. only the selected ready scene receives `aria-current="page"`
1. navigation callbacks run after link activation
1. empty-route normalization produces Dam Break and uses replacement semantics
1. unknown hashes remain unknown

Chromium coverage must prove:

1. desktop header, sidebar, active demo, and player are visible together
1. no static preview cards or separate Open buttons remain
1. selecting a desktop sidebar scene changes the active player and current link
1. an empty hash opens Dam Break without an extra back-history entry
1. at 375 pixels the desktop sidebar is hidden and Demos trigger is visible
1. opening the drawer moves focus inside and makes its title available
1. Escape closes the drawer and restores trigger focus
1. selecting a drawer scene closes it and opens the selected demo
1. the page has no horizontal overflow
1. existing playback, controls, pointer interactions, rendering mode, hidden
   tab, resize, and keyboard tests remain green

Run typecheck, all web unit/browser tests, Markdown validation, managed Bright
Builds checks, relevant WASM tests, and the exact-SHA CI workflows.

## Deterministic media

The player panel remains the media capture region, so the sticky header,
desktop sidebar, and closed mobile drawer are excluded from recordings.
Nevertheless, shell CSS, dependency, and source changes alter the capture-input
profile and may alter panel dimensions. Regenerate all six MP4s, animated WebPs,
and the manifest; prove byte-stable second generation; run check mode; and
inspect representative frames before publication.

## Out of scope

- Reskinning simulation controls or the Canvas palette
- Adding nested navigation, search, favorites, or user accounts
- Replacing hash routing with a router dependency
- Persisting drawer-open state
- Community drawer packages
- Tailwind, MysticUI, or a broad design-system migration
- Changes to Rust physics, WASM frame lanes, or public crate APIs
