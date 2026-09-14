# Phase 14 CI formatter environment diagnosis

Recorded 2026-09-13. This is environment-repair evidence only; it does not establish Phase 14 physics correctness or platform success.

## Failed CI boundary

- Workflow: `.github/workflows/ci.yml`.
- Run: `34777296141`.
- Job: `103777632760`, `Linux quality and isolation`.
- Head: `5aadb6c105b98ae09443f74e44c57f8ce7eae19d`.
- Preserved log: `target/phase14-local-verification/attempt-20260913-execution-start/linux-quality.log`.
- Failure: `Check Markdown formatting`, exit 1, before Rust checks.
- Runner Python: 3.13.15. Its isolated environment installed only `mdformat==1.0.0`, resolving `markdown-it-py==4.2.0` and `mdurl==0.1.2`.

## Controlled reproduction

An isolated Python 3.13.12 virtual environment checked a temporary snapshot of tracked Markdown and `.mdformat.toml` read directly from the failed head. The snapshot contained 35 files; `.planning/` and `third_party/` content was not copied. Repository formatter exclusions were preserved. Every formatter invocation used `mdformat --check .`; no Markdown was rewritten.

Temporary experiment directory: `/var/folders/b6/j7bsvp3j6jzbl8r28p9wqtnh0000gn/T/liquidfun-mdformat-ci-fm7w9u0p`.

| Environment change | Result |
| --- | --- |
| Fresh venv with `mdformat==1.0.0`, `markdown-it-py==4.2.0`, `mdurl==0.1.2` | Exit 1; exact same 11 files as CI fail |
| Add only `mdformat-gfm==1.0.0` | Exit 0; unchanged snapshot passes |
| Then add `mdformat-frontmatter==2.1.2` | Exit 0; unchanged snapshot still passes |

The GFM installation resolved `mdit-py-plugins==0.6.1` and `wcwidth==0.8.3`. The frontmatter installation added `ruamel-yaml==0.19.1`. Final formatter version output was `mdformat 1.0.0 (mdformat-gfm 1.0.0, mdformat_frontmatter 2.1.2)`.

The matching 11 failing files were:

- `UPSTREAM.md`
- `standards-overrides.md`
- `THIRD_PARTY_NOTICES.md`
- `ARCHITECTURE.md`
- `TESTING.md`
- `UPSTREAM-CORPUS.md`
- `BENCHMARKING.md`
- `README.md`
- `docs/decisions/0001-oracle-selection.md`
- `crates/liquidfun-testbed/CAPABILITY.md`
- `fuzz/corpus/README.md`

## Root cause and proposed fix

The missing GFM plugin causes the observed formatting failures. Adding that plugin alone resolves all 11 without source changes. The existing `.github/workflows/bright-builds-auto-update.yml` already installs both GFM and frontmatter plugins at the versions below, matching the local formatter environment.

Proposed one-line replacement for the formatter installation in `.github/workflows/ci.yml`:

```bash
"${RUNNER_TEMP}/mdformat-venv/bin/python" -m pip install --disable-pip-version-check mdformat==1.0.0 mdformat-gfm==1.0.0 mdformat-frontmatter==2.1.2
```

Keep the formatter check and exclusions unchanged. Frontmatter is included for environment consistency; its absence was not required to reproduce this failure. No broad Markdown normalization is needed.

## Verification limits

The controlled experiment reproduces and resolves the formatter boundary on local macOS with Python 3.13.12. It does not claim a Linux rerun, successful later workflow stages, Windows particle-group repair, or phase completion. A CI rerun against the replacement candidate remains required. The diagnosis performed no repository source or CI edits, no commits, and no Cargo checks; this evidence file was added separately at the parent agent's request.

## Applied correction — 2026-09-14

The CI installation line now includes `mdformat-gfm==1.0.0` and `mdformat-frontmatter==2.1.2`. Actionlint, local Markdown checking, the managed checker and diff checking passed. The ordered Rust 1.97.0 format, Clippy, build and test gate passed in the local Linux ARM64 container; retained log: `target/phase14-local-verification/attempt-20260914-ci-environment/gate.log`. This is local verification of the environment correction; the updated GitHub quality job has not yet run, and no Windows physics success is claimed.
