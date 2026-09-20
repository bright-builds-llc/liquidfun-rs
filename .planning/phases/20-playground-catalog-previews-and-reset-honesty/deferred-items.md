# Phase 20 deferred items

Discovered during 20-02 and left out of scope (pre-existing, not caused by
this plan's files).

- Bright Builds `file-lengths` still fails for
  `tools/xtask/tests/upstream_cli.rs` (642 physical lines). Plan 20-02
  required the checker to exit 0 as a cap on `web/src/app.css` and new
  catalog files; those files are 628, 120, and 40 lines. Do not split the
  xtask test in this catalog-preview plan. Scene-module file-length debt
  remains Phase 21.
