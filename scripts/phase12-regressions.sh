#!/usr/bin/env bash
set -euo pipefail

readonly REGISTRY_TIMEOUT_SECONDS=120
readonly PER_TEST_TIMEOUT_SECONDS=300
readonly VALIDATOR_TIMEOUT_SECONDS=120
readonly TOTAL_TIMEOUT_SECONDS=3600
readonly MAXIMUM_LOG_BYTES=$((16 * 1024 * 1024))

usage() {
	printf 'usage: %s <check|run> [candidate-sha]\n' "$0" >&2
	exit 64
}

fail() {
	printf 'phase12-regressions: %s\n' "$1" >&2
	exit 64
}

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
repository_root=$(cd -- "$script_directory/.." && pwd -P)
cd -- "$repository_root"

require_tools() {
	for tool in cargo git jq sha256sum timeout; do
		command -v "$tool" >/dev/null 2>&1 || fail "$tool is required"
	done
}

execution_list_is_valid() {
	local execution_list=$1
	jq -e '
	  . as $entries
	  | type == "array"
	  and ([.[].regression_id] | length == (unique | length))
	  and ([.[].named_test_path] | length == (unique | length))
	  and ([.[].minimized_input] | length == (unique | length))
	  and all(.[];
	    (.regression_id | type == "string" and test("^[a-z0-9-]{1,128}$"))
	    and (.named_test_path | type == "string" and contains("::") and (startswith("-") | not))
	    and (.minimized_input
	      | type == "string"
	      and test("^(scenarios/regressions|fuzz/corpus/regressions)/[A-Za-z0-9._/-]+$")
	      and (split("/") | all(. != "" and . != "." and . != "..")))
	    and (.minimized_sha256 | type == "string" and test("^[0-9a-f]{64}$"))
	    and (.provenance.target | type == "string" and length > 0)
	    and (.provenance.generator | type == "string" and length > 0)
	    and (.provenance.toolchain | type == "string" and length > 0)
	    and (.provenance.candidate_commit | type == "string" and test("^[0-9a-f]{40}$"))
	    and (.provenance.fix_commit | type == "string" and test("^[0-9a-f]{40}$"))
	    and (.provenance.candidate_commit != .provenance.fix_commit)
	    and (.provenance.first_divergence_signature | type == "string" and length > 0)
	    and (.provenance.failure_class
	      | type == "string"
	      and IN("Harness", "PhysicsMismatch", "Sanitizer", "Timeout", "Schema"))
	    and (
	      .provenance.failure_class != "PhysicsMismatch"
	      or (
	        (.provenance.oracle_identity | type == "string" and length > 0)
	        and (.provenance.tolerance_identity | type == "string" and length > 0)
	      )
	    )
	    and (
	      (.provenance.oracle_identity == null)
	      or (.provenance.oracle_identity | type == "string" and length > 0)
	    )
	    and (
	      (.provenance.tolerance_identity == null)
	      or (.provenance.tolerance_identity | type == "string" and length > 0)
	    )
	  )
	' "$execution_list" >/dev/null
}

check_contract() {
	require_tools
	local temporary_directory
	temporary_directory=$(mktemp -d "${TMPDIR:-/tmp}/phase12-regressions-check.XXXXXX")
	trap 'rm -rf -- "$temporary_directory"' EXIT
	local execution_list="$temporary_directory/execution-list.json"
	if ! timeout --signal=TERM "${REGISTRY_TIMEOUT_SECONDS}s" \
		cargo xtask safety-evidence validate-regressions --emit-execution-list \
		>"$execution_list"; then
		fail "typed regression registry validation failed"
	fi
	execution_list_is_valid "$execution_list" ||
		fail "typed execution list violates the closed projection"
	local registration_count
	registration_count=$(jq 'length' "$execution_list")
	rm -rf -- "$temporary_directory"
	trap - EXIT
	printf 'phase12-regressions check passed: %s reviewed registrations\n' "$registration_count"
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
	local output_root="$repository_root/target/phase12-regressions"
	local output_directory="$output_root/$candidate_sha"
	[[ ! -L "$repository_root/target" && ! -L "$output_root" && ! -L "$output_directory" ]] ||
		fail "regression output contains a symbolic link"
	mkdir -p -- "$output_root"
	if [[ -e "$output_directory" ]]; then
		[[ -d "$output_directory" ]] || fail "regression destination is not a directory"
		rm -rf -- "$output_directory"
	fi
	mkdir -p -- "$output_directory/.work/logs"
	printf '%s\n' "$output_directory"
}

hash_file() {
	sha256sum "$1" | awk '{print $1}'
}

bounded_log_or_fail() {
	local log_file=$1
	local case_name=$2
	local log_bytes
	log_bytes=$(wc -c <"$log_file")
	if ((log_bytes > MAXIMUM_LOG_BYTES)); then
		tail --bytes=65536 "$log_file" >&2
		fail "named regression log exceeds reviewed bound: $case_name"
	fi
}

run_registered_test() {
	local entry=$1
	local output_directory=$2
	local result_records=$3
	local started_seconds=$4
	local candidate_sha=$5

	local regression_id named_test_path minimized_input minimized_sha256
	local target generator toolchain original_candidate fix_commit
	local oracle_identity tolerance_identity first_divergence failure_class
	regression_id=$(jq -er '.regression_id' <<<"$entry")
	named_test_path=$(jq -er '.named_test_path' <<<"$entry")
	minimized_input=$(jq -er '.minimized_input' <<<"$entry")
	minimized_sha256=$(jq -er '.minimized_sha256' <<<"$entry")
	target=$(jq -er '.provenance.target' <<<"$entry")
	generator=$(jq -er '.provenance.generator' <<<"$entry")
	toolchain=$(jq -er '.provenance.toolchain' <<<"$entry")
	original_candidate=$(jq -er '.provenance.candidate_commit' <<<"$entry")
	fix_commit=$(jq -er '.provenance.fix_commit' <<<"$entry")
	oracle_identity=$(jq -er '.provenance.oracle_identity // ""' <<<"$entry")
	tolerance_identity=$(jq -er '.provenance.tolerance_identity // ""' <<<"$entry")
	first_divergence=$(jq -er '.provenance.first_divergence_signature' <<<"$entry")
	failure_class=$(jq -er '.provenance.failure_class' <<<"$entry")

	local minimized_path="$repository_root/$minimized_input"
	[[ -f "$minimized_path" && ! -L "$minimized_path" ]] ||
		fail "reviewed minimized input is unavailable: $regression_id"
	[[ "$(hash_file "$minimized_path")" == "$minimized_sha256" ]] ||
		fail "reviewed minimized input hash differs: $regression_id"

	local elapsed_seconds=$((SECONDS - started_seconds))
	local remaining_seconds=$((TOTAL_TIMEOUT_SECONDS - elapsed_seconds))
	((remaining_seconds > 0)) || fail "total named-regression budget exhausted"
	local case_timeout=$PER_TEST_TIMEOUT_SECONDS
	if ((remaining_seconds < case_timeout)); then
		case_timeout=$remaining_seconds
	fi

	local log_file="$output_directory/.work/logs/$regression_id.log"
	if ! timeout --signal=TERM "${case_timeout}s" \
		env \
		LIQUIDFUN_REGRESSION_ID="$regression_id" \
		LIQUIDFUN_REGRESSION_INPUT="$minimized_input" \
		LIQUIDFUN_REGRESSION_INPUT_SHA256="$minimized_sha256" \
		LIQUIDFUN_REGRESSION_TARGET="$target" \
		LIQUIDFUN_REGRESSION_GENERATOR="$generator" \
		LIQUIDFUN_REGRESSION_TOOLCHAIN="$toolchain" \
		LIQUIDFUN_REGRESSION_ORIGINAL_CANDIDATE="$original_candidate" \
		LIQUIDFUN_REGRESSION_FIX_COMMIT="$fix_commit" \
		LIQUIDFUN_REGRESSION_ORACLE_IDENTITY="$oracle_identity" \
		LIQUIDFUN_REGRESSION_TOLERANCE_IDENTITY="$tolerance_identity" \
		LIQUIDFUN_REGRESSION_FIRST_DIVERGENCE="$first_divergence" \
		LIQUIDFUN_REGRESSION_FAILURE_CLASS="$failure_class" \
		cargo test --all-features -- "$named_test_path" --exact \
		>"$log_file" 2>&1; then
		bounded_log_or_fail "$log_file" "$regression_id"
		tail -n 80 "$log_file" >&2
		fail "named regression failed or timed out: $regression_id"
	fi
	bounded_log_or_fail "$log_file" "$regression_id"

	jq -cn \
		--arg regression_id "$regression_id" \
		--arg candidate_sha "$candidate_sha" \
		--arg named_test_path "$named_test_path" \
		--arg minimized_sha256 "$minimized_sha256" \
		'{
		  regression_id: $regression_id,
		  candidate_sha: $candidate_sha,
		  named_test_path: $named_test_path,
		  minimized_sha256: $minimized_sha256,
		  outcome: "passed"
		}' >>"$result_records"
}

write_completion() {
	local output_directory=$1
	local candidate_sha=$2
	local result_records=$3
	local expected_count=$4
	local completion_staging="$output_directory/.work/completion.json"
	jq -s \
		--arg candidate_sha "$candidate_sha" \
		'{
		  schema_version: 1,
		  candidate_sha: $candidate_sha,
		  complete: true,
		  results: .
		}' "$result_records" >"$completion_staging"
	[[ "$(jq '.results | length' "$completion_staging")" == "$expected_count" ]] ||
		fail "result cardinality differs from reviewed registrations"
	[[ "$(jq '[.results[].regression_id] | unique | length' "$completion_staging")" == "$expected_count" ]] ||
		fail "result set contains duplicate registrations"
	mv -- "$completion_staging" "$output_directory/completion.json"
	rm -rf -- "$output_directory/.work"
}

write_producer_identity_last() {
	local output_directory=$1
	local candidate_sha=$2
	local validation_identity="$output_directory/identity.json"
	[[ -f "$validation_identity" && ! -L "$validation_identity" ]] ||
		fail "typed validator did not publish its identity"
	[[ "$(jq -er '.candidate_sha' "$validation_identity")" == "$candidate_sha" ]] ||
		fail "typed validation identity carries the wrong candidate"
	local completion_sha256
	completion_sha256=$(hash_file "$output_directory/completion.json")
	[[ "$(jq -er '.completion_sha256' "$validation_identity")" == "$completion_sha256" ]] ||
		fail "typed validation identity carries the wrong payload hash"
	local manifest_sha256
	manifest_sha256=$(jq -er '.regression_manifest_sha256' "$validation_identity")
	[[ "$manifest_sha256" =~ ^[0-9a-f]{64}$ ]] ||
		fail "typed validation identity carries an invalid manifest hash"
	local named_test_count
	named_test_count=$(jq -er '.results | length' "$output_directory/completion.json")
	local run_id=${GITHUB_RUN_ID:-0}
	[[ "$run_id" =~ ^[0-9]+$ ]] || fail "producer run ID must be an unsigned integer"

	jq -n \
		--arg candidate_sha "$candidate_sha" \
		--arg producer_workflow "${GITHUB_WORKFLOW:-local}" \
		--arg producer_job "${GITHUB_JOB:-local}" \
		--argjson run_id "$run_id" \
		--arg regression_manifest_sha256 "$manifest_sha256" \
		--argjson named_test_count "$named_test_count" \
		--arg payload_sha256 "$(hash_file "$validation_identity")" \
		'{
		  schema_version: 1,
		  evidence_kind: "named_regressions",
		  candidate_sha: $candidate_sha,
		  producer_workflow: $producer_workflow,
		  producer_job: $producer_job,
		  run_id: $run_id,
		  regression_manifest_sha256: $regression_manifest_sha256,
		  named_test_count: $named_test_count,
		  payload_path: "identity.json",
		  payload_sha256: $payload_sha256
		}' >"$output_directory/producer-identity.json"
}

run_regressions() {
	local candidate_sha=$1
	require_tools
	validate_candidate "$candidate_sha"
	local started_seconds=$SECONDS
	local output_directory
	output_directory=$(prepare_output "$candidate_sha")
	local execution_list="$output_directory/.work/execution-list.json"
	if ! timeout --signal=TERM "${REGISTRY_TIMEOUT_SECONDS}s" \
		cargo xtask safety-evidence validate-regressions --emit-execution-list \
		>"$execution_list"; then
		fail "typed regression registry validation failed"
	fi
	execution_list_is_valid "$execution_list" ||
		fail "typed execution list violates the closed projection"
	local registration_count
	registration_count=$(jq 'length' "$execution_list")
	((registration_count > 0)) || fail "reviewed regression registry is empty"

	local result_records="$output_directory/.work/results.jsonl"
	: >"$result_records"
	local entries=()
	while IFS= read -r entry; do
		entries+=("$entry")
	done < <(jq -c '.[]' "$execution_list")
	[[ "${#entries[@]}" == "$registration_count" ]] ||
		fail "execution-list cardinality changed during projection"
	local entry
	for entry in "${entries[@]}"; do
		run_registered_test \
			"$entry" \
			"$output_directory" \
			"$result_records" \
			"$started_seconds" \
			"$candidate_sha"
	done
	[[ "$(jq -s 'length' "$result_records")" == "$registration_count" ]] ||
		fail "named regression result set is incomplete"

	write_completion "$output_directory" "$candidate_sha" "$result_records" "$registration_count"
	local elapsed_seconds=$((SECONDS - started_seconds))
	local remaining_seconds=$((TOTAL_TIMEOUT_SECONDS - elapsed_seconds))
	((remaining_seconds > 0)) || fail "total named-regression budget exhausted before validation"
	local validation_timeout=$VALIDATOR_TIMEOUT_SECONDS
	if ((remaining_seconds < validation_timeout)); then
		validation_timeout=$remaining_seconds
	fi
	if ! timeout --signal=TERM "${validation_timeout}s" \
		cargo xtask safety-evidence validate-regression-results --candidate "$candidate_sha" --results "target/phase12-regressions/$candidate_sha"; then
		fail "typed regression result validation failed"
	fi
	write_producer_identity_last "$output_directory" "$candidate_sha"
	printf 'phase12-regressions evidence complete: %s\n' "$output_directory"
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
	run_regressions "$1"
	;;
*)
	usage
	;;
esac
