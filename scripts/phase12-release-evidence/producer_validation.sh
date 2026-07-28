#!/usr/bin/env bash
set -euo pipefail

conditional_artifact_name() {
	local candidate_sha=$1
	local platform_run_id=$2
	if jq -e '.conditional_targets[0].native_evidence != null' \
		reference/platform/support.json >/dev/null; then
		printf 'phase12-platform-x86_64-apple-darwin-%s-%s\n' \
			"$platform_run_id" "$candidate_sha"
	else
		printf 'phase12-platform-x86_64-apple-darwin-downgrade-%s-%s\n' \
			"$platform_run_id" "$candidate_sha"
	fi
}
write_expected_artifacts() {
	local path=$1
	local candidate_sha=$2
	local platform_run_id=$3
	local oracle_run_id=$4
	local safety_run_id=$5
	local fuzz_run_id=$6
	local regressions_run_id=$7
	local coverage_run_id=$8
	local performance_run_id=$9
	printf '%s\n' \
		"phase12-package-$platform_run_id-$candidate_sha" \
		"phase12-platform-msrv-$platform_run_id-$candidate_sha" \
		"phase12-platform-x86_64-unknown-linux-gnu-$platform_run_id-$candidate_sha" \
		"phase12-platform-aarch64-unknown-linux-gnu-$platform_run_id-$candidate_sha" \
		"phase12-platform-aarch64-apple-darwin-$platform_run_id-$candidate_sha" \
		"phase12-platform-x86_64-pc-windows-msvc-$platform_run_id-$candidate_sha" \
		"$(conditional_artifact_name "$candidate_sha" "$platform_run_id")" \
		"phase11-canonical-$oracle_run_id-$candidate_sha" \
		"phase11-sanitizer-$oracle_run_id-$candidate_sha" \
		"phase12-miri-$safety_run_id-$candidate_sha" \
		"phase12-rust-sanitizer-$safety_run_id-$candidate_sha" \
		"fuzz-protocol-$fuzz_run_id-$candidate_sha" \
		"fuzz-shapes_collision-$fuzz_run_id-$candidate_sha" \
		"fuzz-world_mutation-$fuzz_run_id-$candidate_sha" \
		"fuzz-particles-$fuzz_run_id-$candidate_sha" \
		"fuzz-groups_ownership-$fuzz_run_id-$candidate_sha" \
		"phase12-regressions-$candidate_sha" \
		"phase12-rust-coverage-$coverage_run_id-$candidate_sha" \
		"phase12-cpp-coverage-$coverage_run_id-$candidate_sha" \
		"phase12-differential-coverage-$coverage_run_id-$candidate_sha" \
		"phase12-performance-$performance_run_id-$candidate_sha" |
		LC_ALL=C sort >"$path"
}
validate_artifact_set() {
	local download_directory=$1
	local expected_path=$2
	[[ -d "$download_directory" && ! -L "$download_directory" ]] ||
		fail "download directory is unavailable"
	local actual_path
	actual_path=$(mktemp "${TMPDIR:-/tmp}/liquidfun-release-artifacts.XXXXXX")
	trap 'rm -f -- "$actual_path"' RETURN
	find "$download_directory" -mindepth 1 -maxdepth 1 -type d -exec basename {} \; |
		LC_ALL=C sort >"$actual_path"
	[[ "$(wc -l <"$actual_path")" -eq "$MAXIMUM_ARTIFACTS" ]] ||
		fail "producer artifact cardinality differs"
	cmp -s "$expected_path" "$actual_path" ||
		fail "producer artifact names are missing, duplicated, or substituted"
	rm -f -- "$actual_path"
	trap - RETURN
}
find_single_identity() {
	local artifact_directory=$1
	local filename=$2
	local -a matches
	mapfile -t matches < <(find "$artifact_directory" -type f -name "$filename" -print)
	[[ "${#matches[@]}" -eq 1 && ! -L "${matches[0]}" ]] ||
		fail "${artifact_directory##*/} has invalid $filename cardinality"
	printf '%s\n' "${matches[0]}"
}
validate_payload_hash() {
	local identity=$1
	local payload=$2
	[[ -f "$payload" && ! -L "$payload" ]] ||
		fail "producer payload is unavailable"
	[[ "$(jq -er '.payload_sha256' "$identity")" == "$(hash_file "$payload")" ]] ||
		fail "producer payload hash differs"
}

validate_platform_payload() {
	local identity=$1
	local verification=$2
	local candidate_sha=$3
	local target=$4
	local platform_run_id=$5
	local expected_job=$6
	local package_sha256=$7
	local expected_tier=$8
	jq -e \
		--arg candidate "$candidate_sha" \
		--arg target "$target" \
		--argjson run "$platform_run_id" \
		--arg job "$expected_job" \
		--arg package "$package_sha256" \
		--arg tier "$expected_tier" '
		.schema_version == 1 and
		.candidate_sha == $candidate and .target == $target and .run_id == $run and
		.job == $job and .workflow == "Platform release candidate" and
		(.runner | type == "string" and length > 0) and
		(.compiler | type == "string" and length > 0) and
		.archive_sha256 == $package and .scalar_mode == "strict_f32" and .tier == $tier and
		(.recorded_at_unix | type == "number" and . > 0)
	' "$identity" >/dev/null || fail "$target platform payload is malformed or substituted"
	jq -e '
		. == {
		  status: "verified",
		  package_isolation: true,
		  rustdoc: true,
		  platform_smoke: true
		}
	' "$verification" >/dev/null || fail "$target platform verification is incomplete"
}

validate_safety_payload() {
	local summary=$1
	local candidate_sha=$2
	local expected_kind=$3
	jq -e --arg candidate "$candidate_sha" --arg kind "$expected_kind" '
		.schema_version == 1 and .candidate_commit == $candidate and .complete == true and
		.evidence_kind == $kind and .parity_authority == false and
		(.toolchain_identity | type == "string" and length > 0) and
		.policy == {
		  unsafe_code: "forbid",
		  unsafe_waivers: 0,
		  advisory_waivers: 0
		} and
		(.cases | type == "array" and length > 0) and
		([.cases[].name] | length == (unique | length)) and
		([.cases[].path] | length == (unique | length)) and
		all(.cases[];
		  (.name | type == "string" and length > 0) and
		  (.path | type == "string" and
		    test("^logs/[a-z0-9][a-z0-9_-]*[.]log$")) and
		  (.sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		  (.bytes | type == "number" and . >= 0)) and
		($kind != "rust_sanitizer" or
		  .target == "x86_64-unknown-linux-gnu")
	' "$summary" >/dev/null || fail "$expected_kind safety payload is incomplete"
	local artifact_directory
	artifact_directory=$(dirname -- "$summary")
	while IFS=$'\t' read -r relative expected_sha256 expected_bytes; do
		local log_path="$artifact_directory/$relative"
		[[ -f "$log_path" && ! -L "$log_path" ]] ||
			fail "$expected_kind safety log is unavailable"
		[[ "$(hash_file "$log_path")" == "$expected_sha256" ]] ||
			fail "$expected_kind safety log hash differs"
		[[ "$(wc -c <"$log_path")" -eq "$expected_bytes" ]] ||
			fail "$expected_kind safety log byte count differs"
	done < <(jq -r '.cases[] | [.path, .sha256, .bytes] | @tsv' "$summary")
}

validate_canonical_payload() {
	local result=$1
	local identity=$2
	local candidate_sha=$3
	jq -e --arg candidate "$candidate_sha" \
		--arg semantic "$(jq -er '.semantic_sha256' "$identity")" '
		. == {
		  schema_version: 1,
		  evidence_kind: "canonical_differential",
		  candidate_commit: $candidate,
		  complete: true,
		  parity_tier: "d1_canonical",
		  coverage_authority: false,
		  performance_authority: false,
		  gap_count: 0,
		  semantic_sha256: $semantic
		}
	' "$result" >/dev/null || fail "canonical differential result is incomplete"
}

validate_coverage_payload() {
	local summary=$1
	local candidate_sha=$2
	local expected_kind=$3
	local artifact_directory
	artifact_directory=$(dirname -- "$summary")
	jq -e --arg candidate "$candidate_sha" --arg kind "$expected_kind" '
		.schema_version == 1 and .candidate_commit == $candidate and
		.evidence_kind == $kind and .parity_authority == false and
		(.toolchain_identity | type == "string" and length > 0) and
		(.artifact_path | type == "string" and
		  test("^[A-Za-z0-9][A-Za-z0-9._-]*$")) and
		(.artifact_sha256 | type == "string" and test("^[0-9a-f]{64}$"))
	' "$summary" >/dev/null || fail "$expected_kind coverage summary is malformed"
	local artifact_path
	artifact_path=$(jq -er '.artifact_path' "$summary")
	local artifact="$artifact_directory/$artifact_path"
	[[ -f "$artifact" && ! -L "$artifact" ]] ||
		fail "$expected_kind coverage artifact is unavailable"
	[[ "$(hash_file "$artifact")" == "$(jq -er '.artifact_sha256' "$summary")" ]] ||
		fail "$expected_kind coverage artifact hash differs"
	if [[ "$expected_kind" == "differential_coverage" ]]; then
		jq -e '
			.schema_version == 1 and .parity_authority == false and
			([.exercised[], .missed[]] | all(.; type == "string" and length > 0)) and
			([.exercised[], .missed[]] | length == (unique | length)) and
			(.missed | length) == 0
		' "$artifact" >/dev/null || fail "differential coverage payload is malformed"
	fi
}

validate_regression_payload() {
	local artifact_directory=$1
	local candidate_sha=$2
	local producer_identity=$3
	local validation_identity="$artifact_directory/identity.json"
	local completion="$artifact_directory/completion.json"
	[[ -f "$validation_identity" && -f "$completion" ]] ||
		fail "regression payload is incomplete"
	[[ "$(hash_file "$completion")" == "$(jq -er '.completion_sha256' "$validation_identity")" ]] ||
		fail "regression completion hash differs"
	jq -e --arg candidate "$candidate_sha" \
		--arg manifest "$(hash_file reference/regressions/manifest.toml)" \
		--argjson expected "$(jq -er '.named_test_count' "$producer_identity")" '
		.schema_version == 1 and .candidate_sha == $candidate and .complete == true and
		(.results | length) == $expected and
		([.results[].regression_id] | length == (unique | length)) and
		all(.results[];
		  .candidate_sha == $candidate and .outcome == "passed" and
		  (.regression_id | type == "string" and length > 0) and
		  (.named_test_path | type == "string" and length > 0) and
		  (.minimized_sha256 | type == "string" and test("^[0-9a-f]{64}$")))
	' "$completion" >/dev/null || fail "regression completion contains missing or unreviewed results"
	jq -e --arg manifest "$(hash_file reference/regressions/manifest.toml)" \
		'.regression_manifest_sha256 == $manifest' "$producer_identity" >/dev/null ||
		fail "regression manifest hash differs"
}

validate_sanitizer_records() {
	local sanitizer_records=$1
	[[ -s "$sanitizer_records" ]] || fail "sanitizer payload is unavailable"
	jq -se 'length > 0 and all(.[]; .outcome == "match")' "$sanitizer_records" \
		>/dev/null || fail "sanitizer payload reports a finding"
}

validate_cheap_evidence() {
	local candidate_sha=$1
	local output_directory=$2
	local release_run_id=$3
	local items="$output_directory/cheap-items.json"
	local identity="$output_directory/cheap-identity.json"
	jq -e --arg candidate "$candidate_sha" --arg run "$release_run_id" \
		'.candidate_commit == $candidate and .producer_run_id == $run and .item_count == 5' \
		"$identity" >/dev/null || fail "inexpensive typed evidence is absent or incomplete"
	[[ "$(jq -er '.payload_sha256' "$identity")" == "$(hash_file "$items")" ]] ||
		fail "inexpensive evidence set hash differs"
	jq -e --arg candidate "$candidate_sha" --arg run "$release_run_id" '
	  length == 5 and
	  ([.[].kind] | sort) ==
	    ["compatibility_closure","corpus_closure","docs","notices","package"] and
	  all(.[];
	    .candidate_commit == $candidate and
	    .producer.workflow == "release.yml" and
	    .producer.job == "release-candidate" and
	    .producer.run_id == $run and
	    .review_status == "reviewed" and .status == "passed")
	' "$items" >/dev/null || fail "inexpensive evidence records are invalid"
	while IFS=$'\t' read -r artifact_path expected_sha256 expected_payload; do
		local artifact="$repository_root/$artifact_path"
		[[ "$artifact" == "$output_directory"/artifacts/* && -f "$artifact" && ! -L "$artifact" ]] ||
			fail "inexpensive evidence artifact is unconfined"
		[[ "$(hash_file "$artifact")" == "$expected_sha256" ]] ||
			fail "inexpensive evidence artifact hash differs"
		jq -e --arg candidate "$candidate_sha" --arg payload "$expected_payload" \
			'.schema_version == 1 and .candidate_commit == $candidate and
			 .status == "passed" and .payload_sha256 == $payload' "$artifact" >/dev/null ||
			fail "inexpensive evidence artifact envelope is invalid"
		[[ "$(jq -cjS '.claims' "$artifact" | hash_stream)" == "$expected_payload" ]] ||
			fail "inexpensive evidence payload hash differs"
	done < <(jq -r '.[] | [.artifact_path, .artifact_sha256, .payload_sha256] | @tsv' "$items")
}
validate_oracle_inventory() {
	local identity=$1
	local artifact_directory=$2
	local count
	count=$(jq '.files | length' "$identity")
	((count > 0 && count <= 256)) || fail "oracle file inventory cardinality is invalid"
	while IFS=$'\t' read -r relative expected_sha256; do
		[[ "$relative" != /* && "$relative" != *".."* ]] ||
			fail "oracle inventory path is unsafe"
		local payload="$artifact_directory/$relative"
		[[ -f "$payload" && ! -L "$payload" ]] ||
			fail "oracle inventory payload is unavailable"
		[[ "$(hash_file "$payload")" == "$expected_sha256" ]] ||
			fail "oracle inventory payload hash differs"
	done < <(jq -r '.files[] | [.path, .sha256] | @tsv' "$identity")
	if [[ "${artifact_directory##*/}" == phase11-sanitizer-* ]]; then
		local sanitizer_records="$artifact_directory/sanitizer.jsonl"
		validate_sanitizer_records "$sanitizer_records"
	elif [[ "${artifact_directory##*/}" == phase11-canonical-* ]]; then
		local canonical_result="$artifact_directory/semantic-result.json"
		[[ -f "$canonical_result" && ! -L "$canonical_result" ]] ||
			fail "canonical differential result is unavailable"
		validate_canonical_payload \
			"$canonical_result" "$identity" "$(jq -er '.head_sha' "$identity")"
	fi
}
