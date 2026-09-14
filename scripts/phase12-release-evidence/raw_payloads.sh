#!/usr/bin/env bash
set -euo pipefail

validate_relative_path() {
	local relative=$1
	[[ "$relative" =~ ^[A-Za-z0-9._/-]+$ && "$relative" != /* && "$relative" != */ && "$relative" != *//* ]] ||
		fail "payload path must be normalized"
	local component
	local -a parts
	IFS=/ read -r -a parts <<<"$relative"
	for component in "${parts[@]}"; do
		[[ "$component" != . && "$component" != .. ]] || fail "payload path must be normalized"
	done
}

require_bounded_payload() {
	local payload=$1
	local maximum=${2:-16777216}
	validate_target_path "$payload"
	[[ -f "$payload" ]] || fail "producer payload is unavailable"
	local bytes
	bytes=$(wc -c <"$payload")
	((bytes <= maximum)) || fail "producer payload exceeds byte bound"
}

validate_performance_inventory() {
	local entry=$1
	local directory=${entry%/*}
	validate_target_path "$directory"
	[[ -z "$(find "$directory" -type l -print -quit)" ]] || fail "performance inventory contains a symbolic link"
	local index="$directory/payload-files.sha256"
	require_bounded_payload "$entry"
	require_bounded_payload "$index" 1048576
	[[ "$(hash_file "$index")" == "$(jq -er '.payload_files_sha256' "$entry")" ]] ||
		fail "performance inventory hash differs"
	local line relative digest
	local -a names=()
	while IFS= read -r line || [[ -n "$line" ]]; do
		[[ "$line" =~ ^([0-9a-f]{64})\ \ ([A-Za-z0-9._/-]+)$ ]] || fail "performance inventory is malformed"
		digest=${BASH_REMATCH[1]}
		relative=${BASH_REMATCH[2]}
		validate_relative_path "$relative"
		case "$relative" in
		payload-files.sha256 | manifest-entry.json | producer-identity.json) fail "performance inventory references its envelope" ;;
		esac
		local existing
		for existing in "${names[@]}"; do
			[[ "$existing" != "$relative" ]] || fail "performance inventory contains duplicate paths"
		done
		names+=("$relative")
		((${#names[@]} <= 256)) || fail "performance inventory exceeds cardinality bound"
		require_bounded_payload "$directory/$relative"
		[[ "$(hash_file "$directory/$relative")" == "$digest" ]] || fail "performance raw payload hash differs"
	done <"$index"
	((${#names[@]} > 0)) || fail "performance inventory is empty"
	local actual expected item
	local -a actual_names=()
	while IFS= read -r -d '' item; do
		relative=${item#"$directory/"}
		validate_relative_path "$relative"
		case "$relative" in
		producer-identity.json | manifest-entry.json | payload-files.sha256) continue ;;
		esac
		actual_names+=("$relative")
	done < <(find "$directory" -type f -print0)
	actual=$(printf '%s\n' "${actual_names[@]}" | LC_ALL=C sort)
	expected=$(printf '%s\n' "${names[@]}" | LC_ALL=C sort)
	[[ "$actual" == "$expected" ]] || fail "performance raw inventory differs"
	local required
	required=$({
		jq -r '.cases[] | "raw/" + .case_id + ".json"' protocol/benchmarks/phase12-v1.json
		printf '%s\n' logs/calibrate.log logs/paired.log logs/validate.log calibration.json paired-summary.json summary.json validation-identity.json
	} | LC_ALL=C sort)
	[[ "$expected" == "$required" ]] || fail "performance required inventory differs"
	jq -e --argjson cases "$(jq '[.cases[].case_id] | sort' protocol/benchmarks/phase12-v1.json)" \
		'(.completed_cases | sort) == $cases' "$directory/paired-summary.json" >/dev/null ||
		fail "performance paired summary is incomplete"
}
