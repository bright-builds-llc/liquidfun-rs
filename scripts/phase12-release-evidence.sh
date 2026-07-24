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
