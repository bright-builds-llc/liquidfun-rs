#!/usr/bin/env bash
set -euo pipefail

readonly NIGHTLY_TOOLCHAIN=nightly-2026-07-15
readonly COMMAND_TIMEOUT_SECONDS=900
readonly MAXIMUM_LOG_BYTES=$((16 * 1024 * 1024))

usage() {
	printf 'usage: %s <check|run> [candidate-sha]\n' "$0" >&2
	exit 64
}

fail() {
	printf 'phase12-miri: %s\n' "$1" >&2
	exit 64
}

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_directory/.." && pwd -P)
cd -- "$repository_root"

check_contract() {
	grep -Fxq 'channel = "nightly-2026-07-15"' rust-toolchain-nightly.toml ||
		fail "shared nightly toolchain differs"
	for source in \
		crates/liquidfun/src/math.rs \
		crates/liquidfun/src/arena.rs \
		crates/liquidfun/src/identity.rs \
		crates/liquidfun/src/collision.rs \
		crates/liquidfun-test-protocol/src/codec.rs \
		crates/liquidfun/src/particle/storage/permutation.rs \
		crates/liquidfun/src/particle/storage/properties/group_model.rs; do
		[[ -f "$source" && ! -L "$source" ]] || fail "allowlisted source is unavailable: $source"
	done
	printf 'phase12-miri check passed: exact nightly and seven pure-Rust subsets\n'
}

validate_candidate() {
	local candidate_sha=$1
	[[ "$candidate_sha" =~ ^[0-9a-f]{40}$ ]] ||
		fail "candidate SHA must be canonical lowercase full hex"
	[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] ||
		fail "candidate SHA differs from the checked-out commit"
}

prepare_output() {
	local candidate_sha=$1
	local output_root="$repository_root/target/phase12-miri"
	local output_directory="$output_root/$candidate_sha"
	[[ ! -L "$repository_root/target" && ! -L "$output_root" && ! -L "$output_directory" ]] ||
		fail "evidence output contains a symbolic link"
	mkdir -p -- "$output_root"
	if [[ -e "$output_directory" ]]; then
		[[ -d "$output_directory" ]] || fail "evidence destination is not a directory"
		rm -rf -- "$output_directory"
	fi
	mkdir -p -- "$output_directory/logs"
	printf '%s\n' "$output_directory"
}

hash_file() {
	sha256sum "$1" | awk '{print $1}'
}

run_case() {
	local output_directory=$1
	local records_file=$2
	local case_name=$3
	shift 3
	local log_file="$output_directory/logs/$case_name.log"
	if ! timeout --signal=TERM "${COMMAND_TIMEOUT_SECONDS}s" "$@" >"$log_file" 2>&1; then
		tail -n 80 "$log_file" >&2
		fail "allowlisted case failed or timed out: $case_name"
	fi
	local log_bytes
	log_bytes=$(wc -c <"$log_file")
	((log_bytes <= MAXIMUM_LOG_BYTES)) || fail "allowlisted case log exceeds reviewed bound"
	jq -cn \
		--arg name "$case_name" \
		--arg path "logs/$case_name.log" \
		--arg sha256 "$(hash_file "$log_file")" \
		--argjson bytes "$log_bytes" \
		'{name: $name, path: $path, sha256: $sha256, bytes: $bytes}' >>"$records_file"
}

write_identity_last() {
	local output_directory=$1
	local candidate_sha=$2
	local summary_sha256
	summary_sha256=$(hash_file "$output_directory/summary.json")
	jq -n \
		--arg candidate_commit "$candidate_sha" \
		--arg toolchain "$NIGHTLY_TOOLCHAIN" \
		--arg producer_workflow "${GITHUB_WORKFLOW:-local}" \
		--arg producer_job "${GITHUB_JOB:-local}" \
		--argjson run_id "${GITHUB_RUN_ID:-0}" \
		--arg payload_path "summary.json" \
		--arg payload_sha256 "$summary_sha256" \
		'{
		  schema_version: 1,
		  evidence_kind: "miri",
		  candidate_commit: $candidate_commit,
		  toolchain_identity: $toolchain,
		  producer_workflow: $producer_workflow,
		  producer_job: $producer_job,
		  run_id: $run_id,
		  payload_path: $payload_path,
		  payload_sha256: $payload_sha256,
		  parity_authority: false
		}' >"$output_directory/identity.json"
}

run_miri() {
	local candidate_sha=$1
	validate_candidate "$candidate_sha"
	command -v timeout >/dev/null 2>&1 || fail "timeout is required"
	command -v sha256sum >/dev/null 2>&1 || fail "sha256sum is required"
	command -v jq >/dev/null 2>&1 || fail "jq is required"
	local output_directory
	output_directory=$(prepare_output "$candidate_sha")
	local records_file="$output_directory/cases.jsonl"
	: >"$records_file"

	run_case "$output_directory" "$records_file" math \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test -p liquidfun --lib math::
	run_case "$output_directory" "$records_file" arena_handles \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test -p liquidfun --lib arena::
	run_case "$output_directory" "$records_file" typed_identity \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test -p liquidfun --lib identity::
	run_case "$output_directory" "$records_file" collision \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test -p liquidfun --lib collision::
	run_case "$output_directory" "$records_file" protocol_codec \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test -p liquidfun-test-protocol --test fixtures
	run_case "$output_directory" "$records_file" particle_permutation \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test -p liquidfun --lib \
		particle::storage::permutation::tests::
	run_case "$output_directory" "$records_file" particle_group_model \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test -p liquidfun --lib \
		particle::storage::properties::group_model::

	jq -s \
		--arg candidate_commit "$candidate_sha" \
		--arg toolchain "$NIGHTLY_TOOLCHAIN" \
		'{
		  schema_version: 1,
		  candidate_commit: $candidate_commit,
		  toolchain_identity: $toolchain,
		  complete: true,
		  cases: .
		}' "$records_file" >"$output_directory/summary.json"
	rm -f -- "$records_file"
	cargo xtask safety-evidence validate-coverage
	write_identity_last "$output_directory" "$candidate_sha"
	printf 'phase12-miri evidence complete: %s\n' "$output_directory"
}

[[ $# -ge 1 ]] || usage
mode=$1
shift
case "$mode" in
check)
	[[ $# -eq 0 ]] || usage
	check_contract
	;;
run)
	[[ $# -eq 1 ]] || usage
	run_miri "$1"
	;;
*)
	usage
	;;
esac
