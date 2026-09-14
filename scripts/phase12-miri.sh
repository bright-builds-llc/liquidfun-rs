#!/usr/bin/env bash
set -euo pipefail

readonly NIGHTLY_TOOLCHAIN=nightly-2026-07-15
readonly COMMAND_TIMEOUT_SECONDS=900
readonly GROUP_MODEL_TIMEOUT_SECONDS=3600
readonly MAXIMUM_LOG_BYTES=$((16 * 1024 * 1024))
readonly MIRI_TARGET=x86_64-unknown-linux-gnu
readonly ENDPOINT_TEST=math::sweep::tests::transform_endpoints_preserve_exact_expected_bits

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
source "$script_directory/phase12-attempt.sh"

scan_source() {
	command -v grep >/dev/null 2>&1 || fail "Miri source scan requires grep"
	local status=0
	grep -Ei 'cm[a]ke|or[a]cle|third[_]party|submod[u]le' "$script_directory/phase12-miri.sh" || status=$?
	case "$status" in
	0) fail "Miri source scan found a forbidden dependency" ;;
	1) ;;
	*) fail "Miri source scan failed with status $status" ;;
	esac
}

validate_math_modes() {
	jq -e '([.cases[] | select(.name == "math")] | length == 1) and
	  ([.cases[] | select(.name == "math_endpoint")] | length == 1) and
	  all(.cases[]; .passed > 0 and .ignored == 0) and
	  all(.cases[] | select(.name == "math"); .passed == 44 and .miriflags == "") and
	  all(.cases[] | select(.name == "math_endpoint");
	    .passed == 1 and .miriflags == "-Zmiri-no-extra-rounding-error")' "$1" >/dev/null ||
		fail "Miri math modes are missing, empty, or misconfigured"
}

check_contract() {
	scan_source
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
	local miriflags=$4
	shift 4
	local log_file="$output_directory/logs/$case_name.log"
	local timeout_seconds=$COMMAND_TIMEOUT_SECONDS
	if [[ "$case_name" == particle_group_model ]]; then
		timeout_seconds=$GROUP_MODEL_TIMEOUT_SECONDS
	fi
	run_attempt_command "$case_name" "$timeout_seconds" env MIRIFLAGS="$miriflags" "$@"
	cp "$output_directory/diagnostics/logs/$case_name.log" "$log_file"
	local log_bytes
	log_bytes=$(wc -c <"$log_file")
	((log_bytes <= MAXIMUM_LOG_BYTES)) || fail "allowlisted case log exceeds reviewed bound"
	local counts passed ignored
	counts=$(sed -n 's/^test result: ok\. \([0-9]*\) passed; 0 failed; \([0-9]*\) ignored;.*/\1 \2/p' "$log_file")
	[[ "$counts" =~ ^([0-9]+)[[:space:]]([0-9]+)$ ]] || fail "missing unique successful test count: $case_name"
	passed=${BASH_REMATCH[1]}
	ignored=${BASH_REMATCH[2]}
	((passed > 0 && ignored == 0)) || fail "empty or ignored Miri case: $case_name"
	jq -cn \
		--arg miriflags "$miriflags" \
		--arg target "$MIRI_TARGET" \
		--arg compiler_identity "$compiler_identity" \
		--argjson passed "$passed" --argjson ignored "$ignored" \
		--argjson timeout_seconds "$timeout_seconds" \
		--arg name "$case_name" \
		--arg path "logs/$case_name.log" \
		--arg sha256 "$(hash_file "$log_file")" \
		--argjson bytes "$log_bytes" \
		'{name: $name, path: $path, sha256: $sha256, bytes: $bytes,
		  miriflags: $miriflags, target: $target, compiler_identity: $compiler_identity,
		  passed: $passed, ignored: $ignored, timeout_seconds: $timeout_seconds,
		  command: $ARGS.positional}' --args -- "$@" >>"$records_file"
}

write_identity_last() {
	local output_directory=$1
	local candidate_sha=$2
	local summary_sha256
	seal_attempt 0
	summary_sha256=$(hash_file "$output_directory/summary.json")
	jq -n \
		--arg candidate_commit "$candidate_sha" \
		--arg toolchain "$NIGHTLY_TOOLCHAIN" \
		--arg producer_workflow "${GITHUB_WORKFLOW:-local}" \
		--arg producer_job "${GITHUB_JOB:-local}" \
		--argjson run_id "${GITHUB_RUN_ID:-0}" \
		--argjson run_attempt "${GITHUB_RUN_ATTEMPT:-1}" \
		--arg diagnostics_sha256 "$(hash_file "$output_directory/diagnostics/checksums.sha256")" \
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
		  run_attempt: $run_attempt,
		  diagnostics_sha256: $diagnostics_sha256,
		  payload_path: $payload_path,
		  payload_sha256: $payload_sha256,
		  parity_authority: false
		}' >"$output_directory/identity.json"
}

run_miri() {
	local candidate_sha=$1
	validate_candidate "$candidate_sha"
	check_contract
	command -v timeout >/dev/null 2>&1 || fail "timeout is required"
	command -v sha256sum >/dev/null 2>&1 || fail "sha256sum is required"
	command -v jq >/dev/null 2>&1 || fail "jq is required"
	local compiler_identity
	compiler_identity=$(rustc "+$NIGHTLY_TOOLCHAIN" -vV)
	local output_directory
	output_directory=$(prepare_output "$candidate_sha")
	begin_attempt "$output_directory" "$candidate_sha"
	printf '%s\n' "$compiler_identity" >"$output_directory/diagnostics/compiler.txt"
	local records_file="$output_directory/cases.jsonl"
	: >"$records_file"

	run_case "$output_directory" "$records_file" math "" \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun --lib math:: -- --skip "$ENDPOINT_TEST"
	run_case "$output_directory" "$records_file" math_endpoint -Zmiri-no-extra-rounding-error \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun --lib "$ENDPOINT_TEST" -- --exact
	run_case "$output_directory" "$records_file" arena_handles "" \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun --lib arena::
	run_case "$output_directory" "$records_file" typed_identity "" \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun --lib identity::
	run_case "$output_directory" "$records_file" collision "" \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun --lib collision::
	run_case "$output_directory" "$records_file" protocol_codec "" \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun-test-protocol --test fixtures
	run_case "$output_directory" "$records_file" particle_permutation "" \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun --lib \
		particle::storage::permutation::tests::
	run_case "$output_directory" "$records_file" particle_group_model "" \
		cargo "+$NIGHTLY_TOOLCHAIN" miri test --target "$MIRI_TARGET" -p liquidfun --lib \
		particle::storage::properties::group_model::

	jq -s \
		--arg candidate_commit "$candidate_sha" \
		--arg toolchain "$NIGHTLY_TOOLCHAIN" \
		'{
		  schema_version: 1,
		  evidence_kind: "miri",
		  candidate_commit: $candidate_commit,
		  toolchain_identity: $toolchain,
		  complete: true,
		  parity_authority: false,
		  policy: {
		    unsafe_code: "forbid",
		    unsafe_waivers: 0,
		    advisory_waivers: 0
		  },
		  cases: .
		}' "$records_file" >"$output_directory/summary.json"
	validate_math_modes "$output_directory/summary.json"
	rm -f -- "$records_file"
	run_attempt_command validate-coverage 120 cargo xtask safety-evidence validate-coverage
	write_identity_last "$output_directory" "$candidate_sha"
	printf 'phase12-miri evidence complete: %s\n' "$output_directory"
}

if [[ "${PHASE12_MIRI_LIBRARY_ONLY:-0}" == 1 ]]; then
	return 0
fi

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
