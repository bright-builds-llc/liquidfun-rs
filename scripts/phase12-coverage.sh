#!/usr/bin/env bash
set -euo pipefail

readonly NIGHTLY_TOOLCHAIN=nightly-2026-07-15
readonly LLVM_COV_VERSION=0.8.7
readonly CLANG_VERSION=22.1.8
readonly COMMAND_TIMEOUT_SECONDS=1800
readonly MAXIMUM_ARTIFACT_BYTES=$((64 * 1024 * 1024))

usage() {
	printf 'usage: %s <check|rust|cpp|differential> [candidate-sha]\n' "$0" >&2
	exit 64
}

fail() {
	printf 'phase12-coverage: %s\n' "$1" >&2
	exit 64
}

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_directory/.." && pwd -P)
cd -- "$repository_root"

validate_contract() {
	cargo xtask safety-evidence validate-coverage
}

write_observed_leaves() {
	local observation_directory=$1
	local output_path=$2
	local line_path="$output_path.lines"
	local marker
	local leaf
	: >"$line_path"
	for marker in "$observation_directory"/*; do
		[[ -e "$marker" ]] || continue
		[[ -f "$marker" && ! -L "$marker" ]] ||
			fail "differential leaf observation is not a regular file"
		leaf=${marker##*/}
		[[ "$leaf" =~ ^[a-z0-9][a-z0-9.-]*$ ]] ||
			fail "differential leaf observation has an invalid ID"
		printf '%s\n' "$leaf" >>"$line_path"
	done
	jq -Rsc 'split("\n") | map(select(length > 0)) | unique | sort' \
		"$line_path" >"$output_path"
	rm -f -- "$line_path"
}

successful_target_without_observation() {
	return 0
}

check_observation_omission_guard() {
	local test_root
	test_root=$(mktemp -d "${TMPDIR:-/tmp}/liquidfun-phase12-coverage.XXXXXX")
	local observation_directory="$test_root/observations"
	local expected="$test_root/expected.json"
	local observed="$test_root/observed.json"
	local report="$test_root/report.json"
	mkdir -p -- "$observation_directory"
	printf '[]\n' >"$observation_directory/subsystem.observed"
	printf '["subsystem.missing","subsystem.observed"]\n' >"$expected"
	successful_target_without_observation
	write_observed_leaves "$observation_directory" "$observed"
	if cargo xtask safety-evidence validate-differential-leaves \
		--expected "$expected" \
		--observed "$observed" \
		--output "$report"; then
		fail "successful target omission was incorrectly accepted"
	fi
	jq -e \
		'.exercised == ["subsystem.observed"] and .missed == ["subsystem.missing"]' \
		"$report" >/dev/null ||
		fail "successful target omission did not produce the exact missing leaf"
	rm -rf -- "$test_root"
}

require_differential_oracles() {
	local reference_root=${1:-"$repository_root/target/reference"}
	local preset
	for preset in oracle-debug oracle-release; do
		local executable="$reference_root/$preset/liquidfun-reference"
		[[ -f "$executable" && -x "$executable" && ! -L "$executable" ]] ||
			fail "differential coverage requires the exact $preset oracle"
	done
}

check_contract() {
	grep -Fxq 'channel = "nightly-2026-07-15"' rust-toolchain-nightly.toml ||
		fail "shared nightly toolchain differs"
	grep -Fq '"parity_authority": false' reference/coverage/contract.json ||
		fail "coverage contract must remain non-authoritative for parity"
	for kind in rust_coverage cpp_coverage differential_coverage; do
		grep -Fq "\"$kind\"" reference/coverage/contract.json ||
			fail "coverage contract is missing $kind"
	done
	[[ -f tools/reference/CMakeLists.txt && ! -L tools/reference/CMakeLists.txt ]] ||
		fail "C++ coverage wrapper is unavailable"
	validate_contract
	check_observation_omission_guard
	printf 'phase12-coverage check passed: typed authority and separate evidence kinds\n'
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
	local coverage_kind=$2
	local output_root="$repository_root/target/phase12-coverage/$candidate_sha"
	local output_directory="$output_root/$coverage_kind"
	[[ ! -L "$repository_root/target" && ! -L "$repository_root/target/phase12-coverage" ]] ||
		fail "coverage output root contains a symbolic link"
	mkdir -p -- "$output_root"
	[[ ! -L "$output_root" && ! -L "$output_directory" ]] ||
		fail "coverage output contains a symbolic link"
	if [[ -e "$output_directory" ]]; then
		[[ -d "$output_directory" ]] || fail "coverage destination is not a directory"
		rm -rf -- "$output_directory"
	fi
	mkdir -p -- "$output_directory"
	printf '%s\n' "$output_directory"
}

hash_file() {
	sha256sum "$1" | awk '{print $1}'
}

require_bounded_artifact() {
	local artifact=$1
	[[ -f "$artifact" && ! -L "$artifact" ]] || fail "coverage artifact is unavailable"
	local artifact_bytes
	artifact_bytes=$(wc -c <"$artifact")
	((artifact_bytes > 0 && artifact_bytes <= MAXIMUM_ARTIFACT_BYTES)) ||
		fail "coverage artifact violates the reviewed byte bound"
}

write_summary() {
	local output_directory=$1
	local candidate_sha=$2
	local evidence_kind=$3
	local toolchain_identity=$4
	local artifact_name=$5
	local artifact_path="$output_directory/$artifact_name"
	require_bounded_artifact "$artifact_path"
	jq -n \
		--arg candidate_commit "$candidate_sha" \
		--arg evidence_kind "$evidence_kind" \
		--arg toolchain_identity "$toolchain_identity" \
		--arg artifact_path "$artifact_name" \
		--arg artifact_sha256 "$(hash_file "$artifact_path")" \
		'{
		  schema_version: 1,
		  evidence_kind: $evidence_kind,
		  candidate_commit: $candidate_commit,
		  toolchain_identity: $toolchain_identity,
		  artifact_path: $artifact_path,
		  artifact_sha256: $artifact_sha256,
		  parity_authority: false
		}' >"$output_directory/summary.json"
}

write_identity_last() {
	local output_directory=$1
	local candidate_sha=$2
	local evidence_kind=$3
	local toolchain_identity=$4
	jq -n \
		--arg candidate_commit "$candidate_sha" \
		--arg evidence_kind "$evidence_kind" \
		--arg toolchain_identity "$toolchain_identity" \
		--arg producer_workflow "${GITHUB_WORKFLOW:-local}" \
		--arg producer_job "${GITHUB_JOB:-local}" \
		--argjson run_id "${GITHUB_RUN_ID:-0}" \
		--arg payload_sha256 "$(hash_file "$output_directory/summary.json")" \
		'{
		  schema_version: 1,
		  evidence_kind: $evidence_kind,
		  candidate_commit: $candidate_commit,
		  toolchain_identity: $toolchain_identity,
		  producer_workflow: $producer_workflow,
		  producer_job: $producer_job,
		  run_id: $run_id,
		  payload_path: "summary.json",
		  payload_sha256: $payload_sha256,
		  parity_authority: false
		}' >"$output_directory/identity.json"
}

finish_coverage() {
	local output_directory=$1
	local candidate_sha=$2
	local evidence_kind=$3
	local toolchain_identity=$4
	local artifact_name=$5
	write_summary \
		"$output_directory" \
		"$candidate_sha" \
		"$evidence_kind" \
		"$toolchain_identity" \
		"$artifact_name"
	validate_contract
	write_identity_last "$output_directory" "$candidate_sha" "$evidence_kind" "$toolchain_identity"
	printf 'phase12-coverage evidence complete: %s\n' "$output_directory"
}

run_rust_coverage() {
	local candidate_sha=$1
	local version
	version=$(cargo llvm-cov --version)
	[[ "$version" == "cargo-llvm-cov $LLVM_COV_VERSION" ]] ||
		fail "cargo-llvm-cov must be exactly $LLVM_COV_VERSION"
	local output_directory
	output_directory=$(prepare_output "$candidate_sha" rust)
	timeout --signal=TERM "${COMMAND_TIMEOUT_SECONDS}s" \
		cargo "+$NIGHTLY_TOOLCHAIN" llvm-cov \
		--workspace --all-features --lcov \
		--output-path "$output_directory/rust.lcov"
	finish_coverage \
		"$output_directory" \
		"$candidate_sha" \
		rust_coverage \
		"$NIGHTLY_TOOLCHAIN" \
		rust.lcov
}

run_cpp_coverage() {
	local candidate_sha=$1
	clang++-22 --version | grep -Eq 'clang version 22\.1\.8' ||
		fail "clang++-22 must report exactly $CLANG_VERSION"
	command -v llvm-profdata-22 >/dev/null 2>&1 || fail "llvm-profdata-22 is required"
	command -v llvm-cov-22 >/dev/null 2>&1 || fail "llvm-cov-22 is required"
	local output_directory
	output_directory=$(prepare_output "$candidate_sha" cpp)
	local build_directory="$repository_root/target/reference/oracle-debug"
	[[ ! -L "$build_directory" ]] || fail "C++ coverage build directory is a symbolic link"
	rm -rf -- "$build_directory"
	timeout --signal=TERM "${COMMAND_TIMEOUT_SECONDS}s" \
		env \
		CC=clang-22 \
		CXX=clang++-22 \
		CFLAGS="-fprofile-instr-generate -fcoverage-mapping" \
		CXXFLAGS="-fprofile-instr-generate -fcoverage-mapping" \
		LDFLAGS="-fprofile-instr-generate" \
		LIQUIDFUN_XTASK_CXX=clang++-22 \
		cargo xtask upstream configure --preset oracle-debug
	timeout --signal=TERM "${COMMAND_TIMEOUT_SECONDS}s" \
		cmake --build "$build_directory" --target liquidfun-reference-protocol-tests --parallel 1
	LLVM_PROFILE_FILE="$output_directory/cpp.profraw" \
		timeout --signal=TERM "${COMMAND_TIMEOUT_SECONDS}s" \
		ctest --test-dir "$build_directory" \
		--output-on-failure --no-tests=error -R '^liquidfun-reference-protocol$'
	llvm-profdata-22 merge -sparse \
		"$output_directory/cpp.profraw" \
		-o "$output_directory/cpp.profdata"
	llvm-cov-22 export \
		-format=lcov \
		-instr-profile="$output_directory/cpp.profdata" \
		"$build_directory/liquidfun-reference-protocol-tests" \
		>"$output_directory/cpp.lcov"
	finish_coverage \
		"$output_directory" \
		"$candidate_sha" \
		cpp_coverage \
		clang-22.1.8 \
		cpp.lcov
}

run_differential_coverage() {
	local candidate_sha=$1
	require_differential_oracles
	local output_directory
	output_directory=$(prepare_output "$candidate_sha" differential)
	local expected="$output_directory/expected-leaves.json"
	local observed="$output_directory/observed-leaves.json"
	local observation_directory="$output_directory/observations"
	local targets=(
		collision_probe
		round_trip
		phase8_comparator
		phase9_corpus
		phase10_corpus
		phase11_corpus
	)
	mkdir -p -- "$observation_directory"
	jq '[
	  .entries[]
	  | select(.evidence.differentially_validated.status == "evidenced")
	  | .id
	] | sort' reference/compatibility.json >"$expected"
	for target in "${targets[@]}"; do
		timeout --signal=TERM "${COMMAND_TIMEOUT_SECONDS}s" \
			env LIQUIDFUN_DIFFERENTIAL_LEAF_DIRECTORY="$observation_directory" \
			cargo test -p liquidfun-differential --all-features --test "$target" -- \
			--test-threads=1
	done
	write_observed_leaves "$observation_directory" "$observed"
	rm -rf -- "$observation_directory"
	cargo xtask inventory check
	cargo xtask safety-evidence validate-differential-leaves \
		--expected "$expected" \
		--observed "$observed" \
		--output "$output_directory/differential-leaves.json"
	finish_coverage \
		"$output_directory" \
		"$candidate_sha" \
		differential_coverage \
		semantic-leaf-v1 \
		differential-leaves.json
}

if [[ "${PHASE12_COVERAGE_LIBRARY_ONLY:-0}" == 1 ]]; then
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
rust | cpp | differential)
	[[ $# -eq 1 ]] || usage
	validate_candidate "$1"
	command -v timeout >/dev/null 2>&1 || fail "timeout is required"
	command -v sha256sum >/dev/null 2>&1 || fail "sha256sum is required"
	command -v jq >/dev/null 2>&1 || fail "jq is required"
	"run_${mode}_coverage" "$1"
	;;
*)
	usage
	;;
esac
