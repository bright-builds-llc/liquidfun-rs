#!/usr/bin/env bash
set -euo pipefail

append_independent_evidence() {
	local candidate_sha=$1
	local output_directory=$2
	local download_directory=$3
	local platform_run_id=$4
	local oracle_run_id=$5
	local safety_run_id=$6
	local fuzz_run_id=$7
	local regressions_run_id=$8
	local coverage_run_id=$9
	local performance_run_id=${10}
	local items_file="$output_directory/independent-items.jsonl"
	local package_identity
	package_identity=$(find_single_identity \
		"$download_directory/phase12-package-$platform_run_id-$candidate_sha" package-identity.json)
	local package_sha256
	package_sha256=$(jq -er '.archive_sha256' "$package_identity")
	[[ "$package_sha256" == \
		"$(jq -er '.archive_sha256' "$output_directory/package/package-identity.json")" ]] ||
		fail "independently produced package differs from the release package"
	local claims msrv_identity
	msrv_identity=$(find_single_identity \
		"$download_directory/phase12-platform-msrv-$platform_run_id-$candidate_sha" identity.json)
	local msrv_version
	msrv_version=$(jq -er '.compiler | capture("rustc (?<version>[0-9]+\\.[0-9]+)").version' \
		"$msrv_identity")
	claims=$(jq -cn --arg package_sha256 "$package_sha256" \
		--arg rust_version "$msrv_version" \
		'{package_sha256:$package_sha256,package_drift:false,rust_version:$rust_version}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" msrv \
		x86_64-unknown-linux-gnu platform.yml msrv "$platform_run_id" rust-1.92.0 "$claims"
	local target
	for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
		aarch64-apple-darwin x86_64-pc-windows-msvc; do
		local platform_identity
		platform_identity=$(find_single_identity \
			"$download_directory/phase12-platform-$target-$platform_run_id-$candidate_sha" \
			identity.json)
		claims=$(jq -cn --arg package_sha256 "$package_sha256" \
			--arg tier "$(jq -er '.tier' "$platform_identity")" \
			'{package_sha256:$package_sha256,package_drift:false,evidence_tier:$tier}')
		emit_evidence "$items_file" "$output_directory" "$candidate_sha" platform "$target" \
			platform.yml native "$platform_run_id" "$RUST_TOOLCHAIN" "$claims"
	done
	if jq -e '.conditional_targets[0].native_evidence != null' \
		reference/platform/support.json >/dev/null; then
		claims=$(jq -cn --arg package_sha256 "$package_sha256" \
			--argjson recorded "$(jq '.conditional_targets[0].native_evidence.recorded_at_unix' reference/platform/support.json)" \
			--argjson expires "$(jq '.conditional_targets[0].native_evidence.expires_at_unix' reference/platform/support.json)" \
			'{package_sha256:$package_sha256,package_drift:false,disposition:"supported",
			  recorded_at_unix:$recorded,expires_at_unix:$expires}')
	else
		claims=$(jq -cn --arg package_sha256 "$package_sha256" \
			'{package_sha256:$package_sha256,package_drift:false,disposition:"unsupported",
			  recorded_at_unix:null,expires_at_unix:null}')
	fi
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" conditional_platform \
		x86_64-apple-darwin platform.yml conditional-policy "$platform_run_id" "$RUST_TOOLCHAIN" "$claims"
	local canonical_result
	canonical_result=$(find_single_identity \
		"$download_directory/phase11-canonical-$oracle_run_id-$candidate_sha" \
		semantic-result.json)
	claims=$(jq -c \
		'{parity_tier,coverage_authority,performance_authority,gap_count}' \
		"$canonical_result")
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" canonical_differential \
		x86_64-unknown-linux-gnu oracle.yml phase11-canonical-linux "$oracle_run_id" \
		"$COMBINED_TOOLCHAIN" "$claims"
	local safety_summary
	safety_summary=$(find_single_identity \
		"$download_directory/phase12-miri-$safety_run_id-$candidate_sha" summary.json)
	claims=$(jq -c '.policy' "$safety_summary")
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" rust_safety \
		x86_64-unknown-linux-gnu safety.yml miri "$safety_run_id" "$NIGHTLY_TOOLCHAIN" \
		"$claims"
	local sanitizer_records sanitizer_findings
	sanitizer_records=$(find_single_identity \
		"$download_directory/phase11-sanitizer-$oracle_run_id-$candidate_sha" sanitizer.jsonl)
	sanitizer_findings=$(jq -s '[.[] | select(.outcome != "match")] | length' \
		"$sanitizer_records")
	claims=$(jq -cn --argjson findings "$sanitizer_findings" '{findings:$findings}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" cpp_sanitizer \
		x86_64-unknown-linux-gnu oracle.yml phase11-sanitizer-linux "$oracle_run_id" \
		"$CLANG_TOOLCHAIN" "$claims"
	local fuzz_findings=0 fuzz_target_count=0 name classification
	for name in protocol shapes_collision world_mutation particles groups_ownership; do
		classification=$(find_single_identity \
			"$download_directory/fuzz-$name-$fuzz_run_id-$candidate_sha" classification.json)
		fuzz_target_count=$((fuzz_target_count + 1))
		if ! jq -e '.outcome == "pass" and .exit_code == 0' "$classification" >/dev/null; then
			fuzz_findings=$((fuzz_findings + 1))
		fi
	done
	((fuzz_findings == 0)) || fail "fuzz producer payload reports findings"
	claims=$(jq -cn --argjson findings "$fuzz_findings" --argjson count "$fuzz_target_count" \
		'{findings:$findings,target_count:$count}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" fuzz \
		x86_64-unknown-linux-gnu fuzz.yml fuzz "$fuzz_run_id" "$NIGHTLY_TOOLCHAIN" \
		"$claims"
	local regression_directory
	regression_directory="$download_directory/phase12-regressions-$candidate_sha"
	local regression_completion
	regression_completion=$(find_single_identity "$regression_directory" completion.json)
	local regression_identity
	regression_identity=$(find_single_identity "$regression_directory" producer-identity.json)
	local expected_regressions actual_regressions unreviewed_regressions
	expected_regressions=$(jq -er '.named_test_count' "$regression_identity")
	actual_regressions=$(jq -er '.results | length' "$regression_completion")
	unreviewed_regressions=$(jq '[.results[] | select(.outcome != "passed")] | length' \
		"$regression_completion")
	claims=$(jq -cn --arg manifest_sha256 "$(jq -er '.regression_manifest_sha256' "$regression_identity")" \
		--argjson missing "$((expected_regressions - actual_regressions))" \
		--argjson unreviewed "$unreviewed_regressions" \
		'{manifest_sha256:$manifest_sha256,missing_results:$missing,
		  unreviewed_results:$unreviewed}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" regressions \
		x86_64-unknown-linux-gnu regressions.yml regressions "$regressions_run_id" \
		"$RUST_TOOLCHAIN" "$claims"
	local differential_summary differential_artifact differential_misses
	differential_summary=$(find_single_identity \
		"$download_directory/phase12-differential-coverage-$coverage_run_id-$candidate_sha" \
		summary.json)
	differential_artifact="$(dirname -- "$differential_summary")/$(jq -er '.artifact_path' "$differential_summary")"
	differential_misses=$(jq -er '.missed | length' "$differential_artifact")
	((differential_misses == 0)) || fail "differential producer payload reports coverage misses"
	claims=$(jq -cn --arg contract_sha256 "$(hash_file reference/coverage/contract.json)" \
		--argjson missing "$differential_misses" \
		'{contract_sha256:$contract_sha256,parity_authority:false,
		  missing_subsystems:$missing}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" rust_coverage \
		x86_64-unknown-linux-gnu coverage.yml rust-coverage "$coverage_run_id" \
		"$RUST_TOOLCHAIN" "$claims"
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" cpp_coverage \
		x86_64-unknown-linux-gnu coverage.yml cpp-coverage "$coverage_run_id" \
		"$CLANG_TOOLCHAIN" "$claims"
	local performance_entry
	performance_entry=$(find_single_identity \
		"$download_directory/phase12-performance-$performance_run_id-$candidate_sha" \
		manifest-entry.json)
	claims=$(jq -cn --arg policy_sha256 "$(jq -er '.policy_sha256' "$performance_entry")" \
		--argjson count "$(jq -er '.reviewed_report_count' "$performance_entry")" \
		'{policy_sha256:$policy_sha256,timing_authority:"unprofiled_wall_clock",
		  claim_scope:"workload_only",claim_status:"no_generalized_performance_claim",
		  profile_authority:false,reviewed_report_count:$count}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" performance \
		x86_64-unknown-linux-gnu performance.yml performance "$performance_run_id" \
		"$COMBINED_TOOLCHAIN" "$claims"
}

aggregate_evidence() {
	[[ $# -eq 11 ]] || usage
	local candidate_sha=$1
	local output_directory=$2
	local download_directory=$3
	local release_run_id=$4
	local platform_run_id=$5
	local oracle_run_id=$6
	local safety_run_id=$7
	local fuzz_run_id=$8
	local regressions_run_id=$9
	local coverage_run_id=${10}
	local performance_run_id=${11}
	require_candidate_checkout "$candidate_sha"
	validate_target_path "$output_directory"
	validate_target_path "$download_directory"
	local run_id
	for run_id in "$release_run_id" "$platform_run_id" "$oracle_run_id" "$safety_run_id" \
		"$fuzz_run_id" "$regressions_run_id" "$coverage_run_id" "$performance_run_id"; do
		validate_run_id "$run_id"
	done
	validate_cheap_evidence "$candidate_sha" "$output_directory" "$release_run_id"
	local expected_path="$output_directory/expected-artifacts.txt"
	write_expected_artifacts "$expected_path" "$candidate_sha" "$platform_run_id" \
		"$oracle_run_id" "$safety_run_id" "$fuzz_run_id" "$regressions_run_id" \
		"$coverage_run_id" "$performance_run_id"
	validate_artifact_set "$download_directory" "$expected_path"
	validate_producer_identities "$candidate_sha" "$download_directory" "$platform_run_id" \
		"$oracle_run_id" "$safety_run_id" "$fuzz_run_id" "$regressions_run_id" \
		"$coverage_run_id" "$performance_run_id"
	append_independent_evidence "$candidate_sha" "$output_directory" "$download_directory" \
		"$platform_run_id" "$oracle_run_id" "$safety_run_id" "$fuzz_run_id" \
		"$regressions_run_id" "$coverage_run_id" "$performance_run_id"
	jq -s --arg candidate "$candidate_sha" \
		'{schema_version:1,candidate_commit:$candidate,items:.}' \
		"$output_directory/cheap-items.jsonl" "$output_directory/independent-items.jsonl" \
		>"$output_directory/candidate-manifest.json"
	[[ "$(jq '.items | length' "$output_directory/candidate-manifest.json")" -eq 19 ]] ||
		fail "candidate manifest has the wrong evidence cardinality"
	printf 'phase12 release manifest constructed: %s\n' \
		"$output_directory/candidate-manifest.json"
}

publish_identity_last() {
	[[ $# -eq 3 ]] || usage
	local candidate_sha=$1
	local output_directory=$2
	local release_run_id=$3
	require_candidate_checkout "$candidate_sha"
	validate_run_id "$release_run_id"
	validate_target_path "$output_directory"
	local manifest="$output_directory/candidate-manifest.json"
	local report="$output_directory/audit-report.json"
	jq -e --arg candidate "$candidate_sha" \
		'.decision == "ready" and .candidate_commit == $candidate and .evidence_count == 19' \
		"$report" >/dev/null || fail "release audit did not authorize readiness"
	local identity_tmp
	identity_tmp=$(mktemp "$output_directory/.audit-identity.XXXXXX")
	jq -n \
		--arg candidate "$candidate_sha" \
		--arg run_id "$release_run_id" \
		--arg manifest_sha256 "$(hash_file "$manifest")" \
		--arg report_sha256 "$(hash_file "$report")" \
		--arg package_sha256 "$(hash_file "$output_directory/package/liquidfun.crate")" \
		'{schema_version:1,candidate_commit:$candidate,producer_workflow:"release.yml",
		  producer_job:"release-candidate",run_id:$run_id,ready:true,
		  manifest_sha256:$manifest_sha256,audit_report_sha256:$report_sha256,
		  package_sha256:$package_sha256}' >"$identity_tmp"
	mv -f -- "$identity_tmp" "$output_directory/audit-identity.json"
	printf 'phase12 release audit identity complete: %s\n' \
		"$output_directory/audit-identity.json"
}
