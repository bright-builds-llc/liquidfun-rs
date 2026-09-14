#!/usr/bin/env bash
set -euo pipefail

candidate_retention() {
	# The entrypoint binds script_directory before sourcing this adapter.
	# shellcheck disable=SC2154
	exec python3 -B "$script_directory/phase15-candidate-evidence/main.py" "$@"
}
