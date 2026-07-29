#!/usr/bin/env bash
set -euo pipefail

readonly RELEASE_WORKFLOW="release.yml"
readonly RELEASE_JOB="release-candidate"
readonly RUST_TOOLCHAIN="rust-1.97.0"
readonly NIGHTLY_TOOLCHAIN="nightly-2026-07-15"
readonly CLANG_TOOLCHAIN="clang-22.1.8"
readonly COMBINED_TOOLCHAIN="clang-22.1.8+rust-1.97.0"
readonly MAXIMUM_ARTIFACTS=21
usage() {
	printf 'usage: %s <check|prepare|aggregate|publish-identity-last> ...\n' "$0" >&2
	exit 64
}
fail() {
	printf 'phase12-release-evidence: %s\n' "$1" >&2
	exit 64
}

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_directory/.." && pwd -P)
cd -- "$repository_root"
source "$script_directory/phase12-release-evidence/common.sh"
source "$script_directory/phase12-release-evidence/producer_validation.sh"
source "$script_directory/phase12-release-evidence/identity_validation.sh"
source "$script_directory/phase12-release-evidence/aggregation.sh"

check_contract() {
	jq -e '.properties.schema_version.const == 1' reference/release/schema.json >/dev/null
	[[ "$(awk '/^\[\[evidence\]\]/{count++} END{print count+0}' \
		reference/release/required-evidence.toml)" -eq 19 ]] ||
		fail "required release evidence registry cardinality differs"
	printf 'phase12 release evidence constructor check passed\n'
}

if [[ "${PHASE12_RELEASE_EVIDENCE_LIBRARY_ONLY:-0}" == 1 ]]; then
	# shellcheck disable=SC2317
	return 0 2>/dev/null || exit 0
fi

[[ $# -ge 1 ]] || usage
mode=$1
shift
case "$mode" in
check)
	[[ $# -eq 0 ]] || usage
	check_contract
	;;
prepare)
	[[ $# -eq 3 ]] || usage
	prepare_output "$@"
	;;
aggregate)
	aggregate_evidence "$@"
	;;
publish-identity-last)
	publish_identity_last "$@"
	;;
*)
	usage
	;;
esac
