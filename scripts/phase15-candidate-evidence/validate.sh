#!/usr/bin/env bash
set -euo pipefail

((BASH_VERSINFO[0] >= 4)) || {
	printf 'retained payload validator requires Bash >=4 and GNU find (Linux or prepared macOS)\n' >&2
	exit 64
}
[[ $# -eq 9 ]]
script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")/.." && pwd -P)
export PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY=1
# shellcheck source=scripts/phase12-release-evidence.sh
source "$script_directory/phase12-release-evidence.sh"
require_candidate_checkout "$1"
validate_target_path "$2"
expected_path=$(mktemp "${TMPDIR:-/tmp}/liquidfun-retained.XXXXXX")
trap 'rm -f -- "$expected_path"' EXIT
write_expected_artifacts "$expected_path" "$1" "${@:3}"
validate_artifact_set "$2" "$expected_path"
validate_producer_identities "$@"
