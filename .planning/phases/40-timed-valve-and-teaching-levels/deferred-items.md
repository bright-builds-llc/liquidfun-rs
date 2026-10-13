# Phase 40 Deferred Items

Out-of-scope discoveries logged during execution (not fixed in the plan that found them).

## From 40-10

- **Tool tray truncates names when a level allows two tool kinds (390x844).** grand-finale is the first level whose budget has two kinds (Plank 2, Valve 1). In portrait, the tray chips read "Pl… ×2" and "Va… ×1" (see `target/phase40/screens/level-grand-finale-390x844.png`). The icons and counts stay visible, the HUD overlap checks pass, and the chips keep their accessible names. A wider chip layout or icon-only portrait chips would fix it; this is tray UI work outside 40-10's scope.
