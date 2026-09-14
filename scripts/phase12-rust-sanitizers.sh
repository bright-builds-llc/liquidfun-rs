#!/usr/bin/env bash
set -euo pipefail

readonly NIGHTLY_TOOLCHAIN=nightly-2026-07-15
readonly SANITIZER_TARGET=x86_64-unknown-linux-gnu
readonly COMMAND_TIMEOUT_SECONDS=1200
readonly MAXIMUM_LOG_BYTES=$((16 * 1024 * 1024))

usage() {
	printf 'usage: %s <check|run> [candidate-sha]\n' "$0" >&2
	exit 64
}

fail() {
	printf 'phase12-rust-sanitizers: %s\n' "$1" >&2
	exit 64
}

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_directory/.." && pwd -P)
cd -- "$repository_root"
source "$script_directory/phase12-attempt.sh"

check_contract() {
	grep -Fxq 'channel = "nightly-2026-07-15"' rust-toolchain-nightly.toml ||
		fail "shared nightly toolchain differs"
	for test_file in \
		crates/liquidfun/tests/math_contract.rs \
		crates/liquidfun/tests/collision_distance.rs \
		crates/liquidfun/tests/object_model.rs \
		crates/liquidfun/tests/particle_permutation_coherence.rs \
		crates/liquidfun/tests/particle_group_properties.rs \
		crates/liquidfun-test-protocol/tests/fixtures.rs; do
		[[ -f "$test_file" && ! -L "$test_file" ]] ||
			fail "allowlisted sanitizer test is unavailable: $test_file"
	done
	printf 'phase12-rust-sanitizers check passed: exact nightly and six Linux subsets\n'
}

validate_candidate() {
	local candidate_sha=$1
	[[ "$candidate_sha" =~ ^[0-9a-f]{40}$ ]] ||
		fail "candidate SHA must be canonical lowercase full hex"
	[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] ||
		fail "candidate SHA differs from the checked-out commit"
	[[ "$(uname -s)" == Linux && "$(uname -m)" == x86_64 ]] ||
		fail "Rust ASan evidence requires Linux x86_64"
}

prepare_output() {
	local candidate_sha=$1
	local output_root="$repository_root/target/phase12-rust-sanitizers"
	local output_directory="$output_root/$candidate_sha"
	[[ ! -L "$repository_root/target" && ! -L "$output_root" && ! -L "$output_directory" ]] ||
		fail "evidence output contains a symbolic link"
	mkdir -p -- "$output_root"
	[[ ! -e "$output_directory" ]] || fail "evidence destination already exists; preserve the previous attempt"
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
	run_attempt_command "$case_name" "$COMMAND_TIMEOUT_SECONDS" \
		env \
		RUSTFLAGS="-Zsanitizer=address -Cforce-frame-pointers=yes" \
		ASAN_OPTIONS="abort_on_error=1:halt_on_error=1:detect_leaks=1" \
		"$@"
	cp "$output_directory/diagnostics/logs/$case_name.log" "$log_file"
	local log_bytes
	log_bytes=$(wc -c <"$log_file")
	((log_bytes <= MAXIMUM_LOG_BYTES)) || fail "sanitizer log exceeds reviewed bound"
	local counts
	counts=$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$log_file")
	[[ "$counts" =~ ^([1-9][0-9]*)[[:space:]]0$ ]] || fail "missing nonzero sanitizer test count: $case_name"
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
	seal_attempt 0
	jq -n \
		--arg candidate_commit "$candidate_sha" \
		--arg producer_workflow "${GITHUB_WORKFLOW:-local}" \
		--arg producer_job "${GITHUB_JOB:-local}" \
		--argjson run_id "${GITHUB_RUN_ID:-0}" \
		--argjson run_attempt "${GITHUB_RUN_ATTEMPT:-1}" \
		--arg diagnostics_sha256 "$(hash_file "$output_directory/diagnostics/checksums.sha256")" \
		--arg payload_sha256 "$(hash_file "$output_directory/summary.json")" \
		'{
		  schema_version: 1,
		  evidence_kind: "rust_sanitizer",
		  candidate_commit: $candidate_commit,
		  toolchain_identity: "nightly-2026-07-15",
		  target: "x86_64-unknown-linux-gnu",
		  producer_workflow: $producer_workflow,
		  producer_job: $producer_job,
		  run_id: $run_id,
		  run_attempt: $run_attempt,
		  diagnostics_sha256: $diagnostics_sha256,
		  payload_path: "summary.json",
		  payload_sha256: $payload_sha256,
		  parity_authority: false
		}' >"$output_directory/identity.json"
}

run_sanitizers() {
	local candidate_sha=$1
	validate_candidate "$candidate_sha"
	command -v timeout >/dev/null 2>&1 || fail "timeout is required"
	command -v sha256sum >/dev/null 2>&1 || fail "sha256sum is required"
	command -v jq >/dev/null 2>&1 || fail "jq is required"
	local output_directory
	output_directory=$(prepare_output "$candidate_sha")
	begin_attempt "$output_directory" "$candidate_sha"
	rustc "+$NIGHTLY_TOOLCHAIN" -vV >"$output_directory/diagnostics/compiler.txt"
	local records_file="$output_directory/cases.jsonl"
	: >"$records_file"
	local cargo_prefix=(cargo "+$NIGHTLY_TOOLCHAIN" test -Zbuild-std --target "$SANITIZER_TARGET")

	run_case "$output_directory" "$records_file" math_contract \
		"${cargo_prefix[@]}" -p liquidfun --test math_contract
	run_case "$output_directory" "$records_file" collision_distance \
		"${cargo_prefix[@]}" -p liquidfun --test collision_distance
	run_case "$output_directory" "$records_file" object_model \
		"${cargo_prefix[@]}" -p liquidfun --test object_model
	run_case "$output_directory" "$records_file" particle_permutation \
		"${cargo_prefix[@]}" -p liquidfun --test particle_permutation_coherence
	run_case "$output_directory" "$records_file" particle_group_model \
		"${cargo_prefix[@]}" -p liquidfun --test particle_group_properties
	run_case "$output_directory" "$records_file" protocol_codec \
		"${cargo_prefix[@]}" -p liquidfun-test-protocol --test fixtures

	jq -s \
		--arg candidate_commit "$candidate_sha" \
		'{
		  schema_version: 1,
		  evidence_kind: "rust_sanitizer",
		  candidate_commit: $candidate_commit,
		  toolchain_identity: "nightly-2026-07-15",
		  target: "x86_64-unknown-linux-gnu",
		  complete: true,
		  parity_authority: false,
		  policy: {
		    unsafe_code: "forbid",
		    unsafe_waivers: 0,
		    advisory_waivers: 0
		  },
		  cases: .
		}' "$records_file" >"$output_directory/summary.json"
	rm -f -- "$records_file"
	run_attempt_command validate-coverage 120 cargo xtask safety-evidence validate-coverage
	write_identity_last "$output_directory" "$candidate_sha"
	printf 'phase12-rust-sanitizers evidence complete: %s\n' "$output_directory"
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
	run_sanitizers "$1"
	;;
*)
	usage
	;;
esac
