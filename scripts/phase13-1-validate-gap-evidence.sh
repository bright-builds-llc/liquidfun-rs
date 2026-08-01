#!/usr/bin/env bash
set -euo pipefail

usage() {
	printf 'usage: %s <manifest> <evidence-json> <repository-root> <retained-root>\n' "$0" >&2
	exit 64
}

fail() {
	printf 'phase13-1-gap-validator: %s\n' "$1" >&2
	exit 1
}

[[ $# -eq 4 ]] || usage
manifest_path=$1
evidence_path=$2
repository_root=$3
retained_root=$4

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
# Reuse the repository's reviewed platform-safe SHA-256 implementation.
# The library path is resolved relative to this checked script at runtime.
# shellcheck disable=SC1091
source "$script_directory/phase12-release-evidence/common.sh"

for tool in jq git; do
	command -v "$tool" >/dev/null 2>&1 || fail "$tool is required"
done

[[ -f "$manifest_path" && ! -L "$manifest_path" ]] || fail "manifest must be a regular nonsymlink file"
[[ -f "$evidence_path" && ! -L "$evidence_path" ]] || fail "evidence must be a regular nonsymlink file"
repository_root=$(cd -- "$repository_root" && pwd -P)
retained_root=$(cd -- "$retained_root" && pwd -P)
cd -- "$repository_root"

require_relative_regular_file() {
	local relative_path=$1
	local label=$2
	[[ -n "$relative_path" && "$relative_path" != /* ]] || fail "$label path must be relative"
	case "/$relative_path/" in
	*/../* | */./*) fail "$label path contains a traversal component" ;;
	esac
	local candidate_path="$retained_root/$relative_path"
	local current_path=$retained_root
	local component
	IFS='/' read -r -a components <<<"$relative_path"
	for component in "${components[@]}"; do
		current_path="$current_path/$component"
		[[ ! -L "$current_path" ]] || fail "$label path contains a symbolic link"
	done
	[[ -f "$candidate_path" ]] || fail "$label file is missing"
	local resolved_path
	resolved_path=$(cd -- "$(dirname -- "$candidate_path")" && printf '%s/%s\n' "$PWD" "$(basename -- "$candidate_path")")
	case "$resolved_path" in
	"$retained_root"/*) ;;
	*) fail "$label path escapes retained root" ;;
	esac
	printf '%s\n' "$resolved_path"
}

jq -e '.schema == "phase13-1-gap-verification-manifest-v1"' "$manifest_path" >/dev/null ||
	fail "manifest schema differs"
jq -e '.schema == "phase13-1-gap-verification-evidence-v1" and .complete == true' "$evidence_path" >/dev/null ||
	fail "evidence is not terminal and complete"

candidate_sha=$(jq -er '.candidate_sha' "$evidence_path")
candidate_tree=$(jq -er '.candidate_tree' "$evidence_path")
output_root=$(jq -er '.output_root' "$evidence_path")
remote_ref=$(jq -er '.remote_ref' "$evidence_path")
canonical_run_id=$(jq -er '.canonical_run_id' "$evidence_path")
validate_sha "$candidate_sha"
[[ "$candidate_tree" =~ ^[0-9a-f]{40}$ ]] || fail "candidate tree must be full lowercase hex"
[[ "$output_root" == "$retained_root" ]] || fail "evidence output root differs from retained root"
[[ "$canonical_run_id" =~ ^[1-9][0-9]*$ ]] || fail "canonical run ID is invalid"

manifest_sha256=$(hash_file "$manifest_path")
[[ "$(jq -er '.manifest_sha256' "$evidence_path")" == "$manifest_sha256" ]] ||
	fail "manifest digest mismatch"

unknown_placeholders=$(jq -r '.allowed_placeholders as $declared
	| ([.. | strings | scan("\\$\\{[^}]+\\}")] | unique - $declared | length)' "$manifest_path")
[[ "$unknown_placeholders" -eq 0 ]] || fail "manifest contains an undeclared placeholder"
expected_commands=$(jq -cS \
	--arg candidate "$candidate_sha" \
	--arg tree "$candidate_tree" \
	--arg output "$output_root" \
	--arg remote "$remote_ref" \
	--arg run "$canonical_run_id" \
	'def expand:
	  gsub("\\$\\{CANDIDATE\\}"; $candidate)
	  | gsub("\\$\\{CANDIDATE_TREE\\}"; $tree)
	  | gsub("\\$\\{OUTPUT_ROOT\\}"; $output)
	  | gsub("\\$\\{REMOTE_REF\\}"; $remote)
	  | gsub("\\$\\{CANONICAL_RUN_ID\\}"; $run);
	.commands | walk(if type == "string" then expand else . end)
	| map({id, argv, environment, stdout_log, stderr_log})' "$manifest_path")
actual_commands=$(jq -cS '.commands | map({id, argv, environment, stdout_log, stderr_log})' "$evidence_path")
[[ "$actual_commands" == "$expected_commands" ]] || fail "command order, argv, environment, or log paths differ"
jq -e '.commands as $commands
	| ($commands | length) > 0
	and (($commands | map(.id) | unique | length) == ($commands | length))
	and ($commands | all(.exit_code == 0))' "$evidence_path" >/dev/null ||
	fail "command IDs must be unique and every exit must be zero"

while IFS= read -r encoded_command; do
	command_json=$(printf '%s' "$encoded_command" | base64 --decode)
	command_id=$(jq -er '.id' <<<"$command_json")
	for stream in stdout stderr; do
		log_path=$(jq -er ".${stream}_log" <<<"$command_json")
		expected_digest=$(jq -er ".${stream}_sha256" <<<"$command_json")
		log_file=$(require_relative_regular_file "$log_path" "$command_id $stream")
		[[ "$(hash_file "$log_file")" == "$expected_digest" ]] || fail "$command_id $stream digest mismatch"
	done
done < <(jq -r '.commands[] | @base64' "$evidence_path")

for checker_id in managed-checker managed-checker-final; do
	if jq -e --arg id "$checker_id" '.commands[] | select(.id == $id)' "$evidence_path" >/dev/null; then
		checker_log=$(jq -er --arg id "$checker_id" '.commands[] | select(.id == $id) | .stdout_log' "$evidence_path")
		checker_file=$(require_relative_regular_file "$checker_log" "$checker_id")
		grep -Eq '^SUMMARY file-lengths scanned=[0-9]+ exceptions=0 findings=0$' "$checker_file" ||
			fail "$checker_id did not prove zero structural findings and exceptions"
		grep -Fxq 'SUMMARY all findings=0' "$checker_file" || fail "$checker_id did not prove zero total findings"
	fi
done
jq -e '.checker == {findings: 0, exceptions: 0}' "$evidence_path" >/dev/null || fail "checker summary differs"

git cat-file -e "$candidate_sha^{commit}" || fail "candidate commit object is missing"
[[ "$(git rev-parse "$candidate_sha^{tree}")" == "$candidate_tree" ]] || fail "candidate tree differs"
[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] || fail "candidate differs from HEAD"
[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || fail "repository is dirty after verification"

structural_commit=$(jq -er '.structural_source.commit' "$manifest_path")
structural_parent=$(jq -er '.structural_source.parent' "$manifest_path")
validate_sha "$structural_commit"
validate_sha "$structural_parent"
[[ "$(git rev-parse "$structural_commit^")" == "$structural_parent" ]] || fail "structural parent differs"
git merge-base --is-ancestor "$structural_commit" "$candidate_sha" || fail "structural commit is not an ancestor"

if [[ "$(jq -r '.test_fixture // false' "$manifest_path")" != true ]]; then
	[[ "$structural_commit" == 981908ea87b6789b6b6e9aa136e65a369c5c736d ]] || fail "production structural commit differs"
	[[ "$structural_parent" == 5394fd92ed036fe4f90358501848a61fa81cfc6d ]] || fail "production structural parent differs"
	merge_commit=$(jq -er '.merge_facts.merge_commit' "$evidence_path")
	[[ "$merge_commit" == f4f5ca661cfb8549b115faa054489f24ba888ae9 ]] || fail "integration merge differs"
	git merge-base --is-ancestor "$merge_commit" "$candidate_sha" || fail "integration merge is not an ancestor"
	[[ "$(git show -s --format='%P' "$merge_commit")" == "be444a01e8649a4540effc147307a1041c4649ca 96b24974eee054fd0afec71e6e14b7998ed7df01" ]] || fail "integration parents differ"
	jq -e '.merge_facts.side_paths_disjoint == true
	  and .merge_facts.override.accepted_by == "pRizz"
	  and (.merge_facts.override.accepted_at | test("^[0-9]{4}-[0-9]{2}-[0-9]{2}T[0-9]{2}:[0-9]{2}:[0-9]{2}Z$"))' "$evidence_path" >/dev/null ||
		fail "accepted merge facts differ"
fi

canonical_relative=$(jq -er '.canonical_identity.path' "$evidence_path")
expected_canonical_relative=$(jq -er '.artifacts[] | select(.id == "canonical-native") | .identity' "$manifest_path")
[[ "$canonical_relative" == "$expected_canonical_relative" ]] || fail "canonical identity path differs"
canonical_file=$(require_relative_regular_file "$canonical_relative" "canonical identity")
[[ "$(hash_file "$canonical_file")" == "$(jq -er '.canonical_identity.sha256' "$evidence_path")" ]] ||
	fail "canonical identity digest mismatch"
jq -e \
	--arg candidate "$candidate_sha" \
	--arg tree "$candidate_tree" \
	--arg run "$canonical_run_id" \
	'.candidate_sha == $candidate
	and .candidate_tree == $tree
	and .workflow_run_id == $run
	and .runner == {os: "ubuntu-24.04", architecture: "x86_64"}
	and .tools == {rust: "1.97.0", clang: "22.1.8", cmake: "4.3.3", ninja: "1.13.2"}
	and .evidence_tier == "D1"
	and (.command_exits | length > 0 and all(.exit_code == 0))' "$canonical_file" >/dev/null ||
	fail "canonical D1 identity differs"

canonical_directory=$(dirname -- "$canonical_relative")
canonical_digests=$(jq -er '.log_digests' "$canonical_file")
digest_list=$(require_relative_regular_file "$canonical_directory/$canonical_digests" "canonical log digest list")
while read -r expected_digest relative_log; do
	[[ "$expected_digest" =~ ^[0-9a-f]{64}$ && -n "$relative_log" ]] || fail "canonical log digest record is malformed"
	canonical_log=$(require_relative_regular_file "$canonical_directory/$relative_log" "canonical log")
	[[ "$(hash_file "$canonical_log")" == "$expected_digest" ]] || fail "canonical retained-log digest mismatch"
done <"$digest_list"

printf 'phase13-1 gap evidence valid: %s\n' "$candidate_sha"
