# Quick Task 260919-jco: Final Responsive Shell Review Fixes

## Goal

Close the complete final review findings for the responsive playground shell,
preserve the approved single CSS file, refresh generated demo metadata
truthfully, and commit the verified result without pushing.

## Tasks

1. Add permanent Playwright coverage for Kobalte modal dismissal, focus
   containment and restoration, scroll locking, outside hiding, and external
   hash-route closure using observable browser semantics and no timing sleeps.
2. Apply the source, CSS, policy, and license-verifier fixes: 44px source link,
   dead export removal, concise CSS cleanup, durable `skipLibCheck` rationale,
   and exact recursively derived Kobalte runtime closure validation.
3. Run focused red-green checks, the requested complete verification matrix,
   regenerate demo media metadata without unnecessary byte churn, write the
   final fix report, and commit the results while preserving `.vscode/`.

## Verification

- Focused Playwright shell regressions fail before the production changes where
  applicable, then pass after the fixes.
- Web unit, type, browser smoke, license, Markdown, managed checker, diff, and
  file-count checks pass.
- `just demo-media` and `just demo-media-check` establish and verify the exact
  media impact.
- The worktree contains only intended tracked changes plus the preserved
  untracked `.vscode/` directory.
