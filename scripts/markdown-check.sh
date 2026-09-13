#!/usr/bin/env bash
set -euo pipefail

# Avoid walking ignored build caches; explicit files still honor .mdformat.toml.
git ls-files -z --cached --others --exclude-standard -- '*.md' |
	xargs -0 mdformat --check --
