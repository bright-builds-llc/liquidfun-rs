#!/usr/bin/env bash
set -euo pipefail

validate_producer_identities() {
	local candidate_sha=$1
	local download_directory=$2
	local platform_run_id=$3
	local oracle_run_id=$4
	local safety_run_id=$5
	local fuzz_run_id=$6
	local regressions_run_id=$7
	local coverage_run_id=$8
	local performance_run_id=$9
	local name identity
	for name in \
		"phase11-canonical-$oracle_run_id-$candidate_sha" \
		"phase11-sanitizer-$oracle_run_id-$candidate_sha"; do
		identity=$(find_single_identity "$download_directory/$name" identity.json)
		jq -e --arg candidate "$candidate_sha" --argjson run "$oracle_run_id" \
			'.head_sha == $candidate and .run_id == $run' "$identity" >/dev/null ||
			fail "$name carries the wrong candidate or run"
		validate_oracle_inventory "$identity" "$download_directory/$name"
	done
	for name in "phase12-miri-$safety_run_id-$candidate_sha" \
		"phase12-rust-sanitizer-$safety_run_id-$candidate_sha"; do
		identity=$(find_single_identity "$download_directory/$name" identity.json)
		local safety_kind=${name#phase12-}
		safety_kind=${safety_kind%%-"$safety_run_id"-*}
		jq -e --arg candidate "$candidate_sha" --argjson run "$safety_run_id" \
			--arg kind "${safety_kind//-/_}" \
			'.candidate_commit == $candidate and .run_id == $run and
			 .evidence_kind == $kind and .producer_workflow == "Phase 12 safety evidence"' \
			"$identity" >/dev/null ||
			fail "$name carries the wrong candidate or run"
		local payload_path
		payload_path=$(jq -er '.payload_path' "$identity")
		local safety_payload
		safety_payload="$(dirname -- "$identity")/$payload_path"
		validate_payload_hash "$identity" "$safety_payload"
		validate_safety_payload "$safety_payload" "$candidate_sha" "${safety_kind//-/_}"
	done
	for name in rust cpp differential; do
		identity=$(find_single_identity \
			"$download_directory/phase12-$name-coverage-$coverage_run_id-$candidate_sha" identity.json)
		jq -e --arg candidate "$candidate_sha" --argjson run "$coverage_run_id" \
			--arg kind "${name}_coverage" \
			'.candidate_commit == $candidate and .run_id == $run and
			 .evidence_kind == $kind and .producer_workflow == "Phase 12 coverage evidence"' \
			"$identity" >/dev/null ||
			fail "$name coverage carries the wrong candidate or run"
		local coverage_payload_path
		coverage_payload_path=$(jq -er '.payload_path' "$identity")
		local coverage_payload
		coverage_payload="$(dirname -- "$identity")/$coverage_payload_path"
		validate_payload_hash "$identity" "$coverage_payload"
		validate_coverage_payload "$coverage_payload" "$candidate_sha" "${name}_coverage"
	done
	for name in protocol shapes_collision world_mutation particles groups_ownership; do
		identity=$(find_single_identity \
			"$download_directory/fuzz-$name-$fuzz_run_id-$candidate_sha" identity.json)
		jq -e --arg candidate "$candidate_sha" --arg target "$name" \
			'.candidate_sha == $candidate and .target == $target' "$identity" >/dev/null ||
			fail "$name fuzz evidence carries the wrong candidate"
		local classification
		classification=$(find_single_identity \
			"$download_directory/fuzz-$name-$fuzz_run_id-$candidate_sha" classification.json)
		jq -e '.outcome == "pass" and .exit_code == 0' "$classification" >/dev/null ||
			fail "$name fuzz evidence contains a finding"
	done
	identity=$(find_single_identity \
		"$download_directory/phase12-regressions-$candidate_sha" producer-identity.json)
	jq -e --arg candidate "$candidate_sha" --argjson run "$regressions_run_id" \
		'.candidate_sha == $candidate and .run_id == $run and
		 .producer_job == "regressions"' "$identity" >/dev/null ||
		fail "regression evidence carries the wrong producer identity"
	local regression_payload_path
	regression_payload_path=$(jq -er '.payload_path' "$identity")
	validate_payload_hash "$identity" "$(dirname -- "$identity")/$regression_payload_path"
	validate_regression_payload "$(dirname -- "$identity")" "$candidate_sha" "$identity"
	identity=$(find_single_identity \
		"$download_directory/phase12-performance-$performance_run_id-$candidate_sha" \
		producer-identity.json)
	jq -e --arg candidate "$candidate_sha" --arg run "$performance_run_id" \
		'.candidate_sha == $candidate and .run_id == $run and
		 .producer_job == "performance" and .release_reviewed == true' "$identity" >/dev/null ||
		fail "performance evidence carries the wrong producer identity"
	local performance_entry
	performance_entry="$(dirname -- "$identity")/manifest-entry.json"
	validate_payload_hash "$identity" "$performance_entry"
	jq -e --arg candidate "$candidate_sha" \
		--arg policy "$(sed -n 's/^policy_sha256 = \"\\([0-9a-f]*\\)\"/\\1/p' \
			reference/performance/manifest.toml)" '
		.schema_version == 1 and .evidence_kind == "paired_performance" and
		.candidate_sha == $candidate and .release_reviewed == true and
		.disposition == "reviewed_controlled" and .workload_count == 14 and .case_count == 32 and
		.reviewed_report_count >= 0 and .policy_sha256 == $policy and
		(.matrix_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		(.payload_files_sha256 | type == "string" and test("^[0-9a-f]{64}$")) and
		(.controlled_host_label | type == "string" and length > 0) and
		(.controlled_host_identity | type == "string" and test("^[0-9a-f]{64}$"))
	' "$performance_entry" >/dev/null || fail "performance manifest payload is incomplete"
	validate_platform_identities "$candidate_sha" "$download_directory" "$platform_run_id"
}

validate_platform_identities() {
	local candidate_sha=$1
	local download_directory=$2
	local platform_run_id=$3
	local package_directory="$download_directory/phase12-package-$platform_run_id-$candidate_sha"
	local package_identity
	package_identity=$(find_single_identity "$package_directory" package-identity.json)
	jq -e --arg candidate "$candidate_sha" '.candidate_commit == $candidate' \
		"$package_identity" >/dev/null || fail "platform package carries the wrong candidate"
	local package_archive
	package_archive=$(find_single_identity "$package_directory" liquidfun.crate)
	[[ "$(hash_file "$package_archive")" == "$(jq -er '.archive_sha256' "$package_identity")" ]] ||
		fail "platform package archive hash differs"
	local package_sha256
	package_sha256=$(jq -er '.archive_sha256' "$package_identity")
	local msrv_artifact="phase12-platform-msrv-$platform_run_id-$candidate_sha"
	local msrv_identity
	msrv_identity=$(find_single_identity "$download_directory/$msrv_artifact" identity.json)
	jq -e --arg candidate "$candidate_sha" --argjson run "$platform_run_id" \
		'.candidate_sha == $candidate and .target == "x86_64-unknown-linux-gnu" and
		 .run_id == $run and .job == "msrv" and
		 (.compiler | startswith("rustc 1.92.0"))' "$msrv_identity" >/dev/null ||
		fail "$msrv_artifact carries the wrong producer identity"
	local msrv_verification
	msrv_verification=$(find_single_identity \
		"$download_directory/$msrv_artifact" verification.json)
	validate_platform_payload "$msrv_identity" "$msrv_verification" "$candidate_sha" \
		x86_64-unknown-linux-gnu "$platform_run_id" msrv "$package_sha256" d2_supported
	local target artifact identity
	for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
		aarch64-apple-darwin x86_64-pc-windows-msvc; do
		artifact="phase12-platform-$target-$platform_run_id-$candidate_sha"
		identity=$(find_single_identity "$download_directory/$artifact" identity.json)
		jq -e --arg candidate "$candidate_sha" --arg target "$target" \
			--argjson run "$platform_run_id" \
			'.candidate_sha == $candidate and .target == $target and
			 .run_id == $run and .job == "native"' "$identity" >/dev/null ||
			fail "$artifact carries the wrong producer identity"
		local verification
		verification=$(find_single_identity "$download_directory/$artifact" verification.json)
		validate_platform_payload "$identity" "$verification" "$candidate_sha" "$target" \
			"$platform_run_id" native "$package_sha256" d2_supported
	done
	artifact=$(conditional_artifact_name "$candidate_sha" "$platform_run_id")
	identity=$(find_single_identity "$download_directory/$artifact" identity.json)
	if jq -e '.conditional_targets[0].native_evidence != null' \
		reference/platform/support.json >/dev/null; then
		local conditional_verification
		conditional_verification=$(find_single_identity \
			"$download_directory/$artifact" verification.json)
		validate_platform_payload "$identity" "$conditional_verification" "$candidate_sha" \
			x86_64-apple-darwin "$platform_run_id" conditional-macos-intel \
			"$package_sha256" d2_supported
	else
		jq -e --arg candidate "$candidate_sha" \
			--arg support "$(hash_file reference/platform/support.json)" '
			.schema_version == 1 and .candidate_sha == $candidate and
			.target == "x86_64-apple-darwin" and .runner == "macos-15-intel" and
			.tier == "unsupported" and .reason == "missing_or_expired_native_evidence" and
			.max_age_days == 90 and .support_sha256 == $support and
			(.recorded_at_unix | type == "number" and . > 0)
		' "$identity" >/dev/null || fail "$artifact downgrade payload is malformed"
	fi
}
