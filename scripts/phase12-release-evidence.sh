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
hash_file() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum "$1" | awk '{print $1}'
	else
		shasum -a 256 "$1" | awk '{print $1}'
	fi
}
validate_sha() {
	local candidate_sha=$1
	[[ "$candidate_sha" =~ ^[0-9a-f]{40}$ ]] ||
		fail "candidate SHA must be canonical lowercase full hex"
}
validate_run_id() {
	local run_id=$1
	[[ "$run_id" =~ ^[1-9][0-9]*$ ]] || fail "producer run ID is invalid"
}
validate_target_path() {
	local path=$1
	case "$path" in
	"$repository_root"/target/*) ;;
	*) fail "output must be confined beneath repository target/" ;;
	esac
	[[ ! -L "$repository_root/target" ]] || fail "target must not be a symbolic link"
	local current=$repository_root
	local relative=${path#"$repository_root/"}
	local component
	IFS='/' read -r -a components <<<"$relative"
	for component in "${components[@]}"; do
		current="$current/$component"
		[[ ! -L "$current" ]] || fail "output path contains a symbolic link"
	done
}
require_candidate_checkout() {
	local candidate_sha=$1
	validate_sha "$candidate_sha"
	[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] ||
		fail "checked-out commit differs from the frozen candidate"
}
emit_evidence() {
	local items_file=$1
	local output_directory=$2
	local candidate_sha=$3
	local kind=$4
	local target=$5
	local workflow=$6
	local job=$7
	local run_id=$8
	local toolchain=$9
	local claims=${10}
	local artifact_directory="$output_directory/artifacts"
	local artifact_path="$artifact_directory/$kind-${target//\//_}.json"
	local payload_sha256
	payload_sha256=$(jq -cjS '.' <<<"$claims" | hash_stream)
	jq -n \
		--arg kind "$kind" \
		--arg target "$target" \
		--arg candidate "$candidate_sha" \
		--arg payload_sha256 "$payload_sha256" \
		--argjson claims "$claims" \
		'{
		  schema_version: 1,
		  kind: $kind,
		  target: $target,
		  candidate_commit: $candidate,
		  status: "passed",
		  payload_sha256: $payload_sha256,
		  claims: $claims
		}' >"$artifact_path"
	local relative_artifact=${artifact_path#"$repository_root/"}
	jq -cn \
		--arg kind "$kind" \
		--arg target "$target" \
		--arg candidate "$candidate_sha" \
		--arg workflow "$workflow" \
		--arg job "$job" \
		--arg run_id "$run_id" \
		--arg artifact_path "$relative_artifact" \
		--arg artifact_sha256 "$(hash_file "$artifact_path")" \
		--arg payload_sha256 "$payload_sha256" \
		--arg toolchain "$toolchain" \
		'{
		  kind: $kind,
		  target: $target,
		  candidate_commit: $candidate,
		  producer: {workflow: $workflow, job: $job, run_id: $run_id},
		  artifact_path: $artifact_path,
		  artifact_sha256: $artifact_sha256,
		  payload_sha256: $payload_sha256,
		  toolchain: $toolchain,
		  review_status: "reviewed",
		  status: "passed"
		}' >>"$items_file"
}
hash_stream() {
	if command -v sha256sum >/dev/null 2>&1; then
		sha256sum | awk '{print $1}'
	else
		shasum -a 256 | awk '{print $1}'
	fi
}
prepare_output() {
	local candidate_sha=$1
	local output_directory=$2
	local release_run_id=$3
	require_candidate_checkout "$candidate_sha"
	validate_run_id "$release_run_id"
	validate_target_path "$output_directory"
	[[ ! -e "$output_directory" ]] || fail "release output already exists"
	mkdir -p -- "$output_directory/artifacts" "$output_directory/package"
	local items_file="$output_directory/cheap-items.jsonl"

	cargo xtask package create-artifact \
		--archive "${output_directory#"$repository_root/"}"/package/liquidfun.crate \
		--identity "${output_directory#"$repository_root/"}"/package/package-identity.json \
		--candidate-commit "$candidate_sha"
	cargo xtask package verify-artifact \
		--archive "${output_directory#"$repository_root/"}"/package/liquidfun.crate \
		--identity "${output_directory#"$repository_root/"}"/package/package-identity.json \
		--toolchain 1.97.0 \
		--target x86_64-unknown-linux-gnu
	cargo publish -p liquidfun --dry-run
	cargo deny check --locked
	RUSTDOCFLAGS="-D warnings" cargo doc -p liquidfun --all-features --no-deps
	cargo test -p liquidfun --all-features --doc
	cargo xtask docs check
	cargo xtask inventory check
	cargo xtask inventory corpus check-snapshot
	cargo xtask inventory corpus check-closure
	cargo xtask provenance check
	git diff --exit-code -- \
		protocol scenarios reference COMPATIBILITY.md UPSTREAM-CORPUS.md

	local package_identity="$output_directory/package/package-identity.json"
	local package_sha256
	package_sha256=$(jq -er '.archive_sha256' "$package_identity")
	[[ "$(hash_file "$output_directory/package/liquidfun.crate")" == "$package_sha256" ]] ||
		fail "package archive differs from its identity"
	local package_claims
	package_claims=$(jq -cn \
		--arg package_sha256 "$package_sha256" \
		--arg archive_path "${output_directory#"$repository_root/"}/package/liquidfun.crate" \
		'{
		  package_name: "liquidfun",
		  package_sha256: $package_sha256,
		  archive_path: $archive_path,
		  archive_sha256: $package_sha256,
		  rust_version: "1.92",
		  scalar_mode: "strict_f32",
		  package_drift: false
		}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" package all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" "$package_claims"
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" docs all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" \
		'{"docs_complete":true,"rustdoc_warnings":0}'
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" notices all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" \
		'{"notices_complete":true,"license":"MIT","advisory_waivers":0}'

	local corpus_claims
	corpus_claims=$(jq -cn \
		--arg authority_sha256 "$(hash_file reference/upstream-corpus.json)" \
		--argjson item_count "$(jq '.items | length' reference/upstream-corpus.json)" \
		'{
		  authority_sha256: $authority_sha256,
		  item_count: $item_count,
		  unresolved_count: 0,
		  nonterminal_count: 0
		}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" corpus_closure all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" "$corpus_claims"
	local compatibility_claims
	compatibility_claims=$(jq -cn \
		--arg authority_sha256 "$(hash_file reference/compatibility.json)" \
		'{
		  authority_sha256: $authority_sha256,
		  gap_count: 0,
		  unexplained_count: 0,
		  mixed_commit_count: 0,
		  coverage_promoted_to_parity: false,
		  platform_promoted_to_parity: false
		}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" compatibility_closure all \
		"$RELEASE_WORKFLOW" "$RELEASE_JOB" "$release_run_id" "$RUST_TOOLCHAIN" \
		"$compatibility_claims"
	jq -s '.' "$items_file" >"$output_directory/cheap-items.json"
	local cheap_sha256
	cheap_sha256=$(hash_file "$output_directory/cheap-items.json")
	jq -n \
		--arg candidate "$candidate_sha" \
		--arg run_id "$release_run_id" \
		--arg cheap_sha256 "$cheap_sha256" \
		'{schema_version: 1, candidate_commit: $candidate, producer_run_id: $run_id,
		  item_count: 5, payload_sha256: $cheap_sha256}' \
		>"$output_directory/cheap-identity.json"
	printf 'phase12 release inexpensive evidence complete: %s\n' "$output_directory"
}
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
}

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
		validate_payload_hash "$identity" "$(dirname -- "$identity")/$payload_path"
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
		validate_payload_hash "$identity" "$(dirname -- "$identity")/$coverage_payload_path"
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
	identity=$(find_single_identity \
		"$download_directory/phase12-performance-$performance_run_id-$candidate_sha" \
		producer-identity.json)
	jq -e --arg candidate "$candidate_sha" --arg run "$performance_run_id" \
		'.candidate_sha == $candidate and .run_id == $run and
		 .producer_job == "performance" and .release_reviewed == true' "$identity" >/dev/null ||
		fail "performance evidence carries the wrong producer identity"
	validate_payload_hash "$identity" "$(dirname -- "$identity")/manifest-entry.json"
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
	local msrv_artifact="phase12-platform-msrv-$platform_run_id-$candidate_sha"
	local msrv_identity
	msrv_identity=$(find_single_identity "$download_directory/$msrv_artifact" identity.json)
	jq -e --arg candidate "$candidate_sha" --argjson run "$platform_run_id" \
		'.candidate_sha == $candidate and .target == "x86_64-unknown-linux-gnu" and
		 .run_id == $run and .job == "msrv" and
		 (.compiler | startswith("rustc 1.92.0"))' "$msrv_identity" >/dev/null ||
		fail "$msrv_artifact carries the wrong producer identity"
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
	done
	artifact=$(conditional_artifact_name "$candidate_sha" "$platform_run_id")
	identity=$(find_single_identity "$download_directory/$artifact" identity.json)
	jq -e --arg candidate "$candidate_sha" \
		'.candidate_sha == $candidate and .target == "x86_64-apple-darwin"' \
		"$identity" >/dev/null || fail "$artifact carries the wrong candidate"
}

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
	local claims
	claims=$(jq -cn --arg package_sha256 "$package_sha256" \
		'{package_sha256:$package_sha256,package_drift:false,rust_version:"1.92"}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" msrv \
		x86_64-unknown-linux-gnu platform.yml msrv "$platform_run_id" rust-1.92.0 "$claims"
	local target
	for target in x86_64-unknown-linux-gnu aarch64-unknown-linux-gnu \
		aarch64-apple-darwin x86_64-pc-windows-msvc; do
		claims=$(jq -cn --arg package_sha256 "$package_sha256" \
			'{package_sha256:$package_sha256,package_drift:false,evidence_tier:"d2_supported"}')
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
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" canonical_differential \
		x86_64-unknown-linux-gnu oracle.yml phase11-canonical-linux "$oracle_run_id" \
		"$COMBINED_TOOLCHAIN" \
		'{"parity_tier":"d1_canonical","coverage_authority":false,"performance_authority":false,"gap_count":0}'
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" rust_safety \
		x86_64-unknown-linux-gnu safety.yml miri "$safety_run_id" "$NIGHTLY_TOOLCHAIN" \
		'{"unsafe_waivers":0,"advisory_waivers":0,"unsafe_code":"forbid"}'
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" cpp_sanitizer \
		x86_64-unknown-linux-gnu oracle.yml phase11-sanitizer-linux "$oracle_run_id" \
		"$CLANG_TOOLCHAIN" '{"findings":0}'
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" fuzz \
		x86_64-unknown-linux-gnu fuzz.yml fuzz "$fuzz_run_id" "$NIGHTLY_TOOLCHAIN" \
		'{"findings":0,"target_count":5}'
	claims=$(jq -cn --arg manifest_sha256 "$(hash_file reference/regressions/manifest.toml)" \
		'{manifest_sha256:$manifest_sha256,missing_results:0,unreviewed_results:0}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" regressions \
		x86_64-unknown-linux-gnu regressions.yml regressions "$regressions_run_id" \
		"$RUST_TOOLCHAIN" "$claims"
	claims=$(jq -cn --arg contract_sha256 "$(hash_file reference/coverage/contract.json)" \
		'{contract_sha256:$contract_sha256,parity_authority:false,missing_subsystems:0}')
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" rust_coverage \
		x86_64-unknown-linux-gnu coverage.yml rust-coverage "$coverage_run_id" \
		"$RUST_TOOLCHAIN" "$claims"
	emit_evidence "$items_file" "$output_directory" "$candidate_sha" cpp_coverage \
		x86_64-unknown-linux-gnu coverage.yml cpp-coverage "$coverage_run_id" \
		"$CLANG_TOOLCHAIN" "$claims"
	local policy_sha256 reviewed_count
	policy_sha256=$(sed -n 's/^policy_sha256 = "\([0-9a-f]*\)"/\1/p' \
		reference/performance/manifest.toml)
	reviewed_count=$(awk '/^\[\[reviewed_reports\]\]/{count++} END{print count+0}' \
		reference/performance/manifest.toml)
	claims=$(jq -cn --arg policy_sha256 "$policy_sha256" --argjson count "$reviewed_count" \
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

check_contract() {
	jq -e '.properties.schema_version.const == 1' reference/release/schema.json >/dev/null
	[[ "$(awk '/^\[\[evidence\]\]/{count++} END{print count+0}' \
		reference/release/required-evidence.toml)" -eq 19 ]] ||
		fail "required release evidence registry cardinality differs"
	printf 'phase12 release evidence constructor check passed\n'
}

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
