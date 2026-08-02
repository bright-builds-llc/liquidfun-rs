#!/usr/bin/env bash
set -euo pipefail

readonly MAXIMUM_LOG_BYTES=$((1024 * 1024))
readonly WORKFLOW_FILE=phase13-1-canonical-native.yml
readonly INTENT_RELATIVE=dispatch-intent.json
readonly RESULT_RELATIVE=dispatch-result.json

usage() {
	printf 'usage: %s <full-candidate-sha> <output-root> <remote-default-branch>\n' "$0" >&2
	exit 64
}

fail() {
	printf 'phase13-1-gap-verification: %s\n' "$1" >&2
	exit 1
}

[[ $# -eq 3 ]] || usage
candidate_sha=$1
output_argument=$2
remote_default_branch=$3

script_directory=$(cd -- "$(dirname -- "${BASH_SOURCE[0]}")" && pwd -P)
default_repository_root=$(cd -- "$script_directory/.." && pwd -P)
manifest_path=${PHASE13_1_GAP_MANIFEST:-"$script_directory/phase13-1-gap-verification-manifest.json"}
validator_path="$script_directory/phase13-1-validate-gap-evidence.sh"
[[ -f "$manifest_path" && ! -L "$manifest_path" ]] || fail "manifest is unavailable"
[[ -x "$validator_path" && ! -L "$validator_path" ]] || fail "independent validator is unavailable"
jq -e '.schema == "phase13-1-gap-verification-manifest-v1"' "$manifest_path" >/dev/null || fail "manifest schema differs"
test_fixture=$(jq -r '.test_fixture // false' "$manifest_path")

if [[ "$test_fixture" == true ]]; then
	[[ -n "${PHASE13_1_GAP_REPOSITORY_ROOT:-}" ]] || fail "fixture repository root is required"
	repository_root=$(cd -- "$PHASE13_1_GAP_REPOSITORY_ROOT" && pwd -P)
	canonical_tmp=$(cd -- "${TMPDIR:-/tmp}" && pwd -P)
	case "$repository_root" in
	"$canonical_tmp"/*) ;;
	*) fail "fixture repository must be below the system temporary directory" ;;
	esac
else
	[[ "$manifest_path" == "$script_directory/phase13-1-gap-verification-manifest.json" ]] || fail "production manifest path differs"
	[[ -z "${PHASE13_1_GAP_REPOSITORY_ROOT:-}" ]] || fail "production repository root cannot be overridden"
	repository_root=$default_repository_root
fi

cd -- "$repository_root"
# shellcheck disable=SC1091
source "$script_directory/phase12-release-evidence/common.sh"
for tool in jq git gh; do
	command -v "$tool" >/dev/null 2>&1 || fail "$tool is required"
done

validate_sha "$candidate_sha"
[[ "$remote_default_branch" =~ ^[A-Za-z0-9._/-]+$ ]] || fail "remote default branch is invalid"
[[ "$remote_default_branch" != -* && "$remote_default_branch" != */../* ]] || fail "remote default branch is unsafe"
[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] || fail "candidate differs from HEAD"
[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || fail "candidate worktree must be clean"
candidate_tree=$(git rev-parse "$candidate_sha^{tree}")

reported_default=$(gh repo view --json defaultBranchRef --jq '.defaultBranchRef.name')
[[ "$reported_default" == "$remote_default_branch" ]] || fail "supplied branch is not the repository default"
remote_tracking_sha=$(git rev-parse "refs/remotes/origin/$remote_default_branch")
[[ "$remote_tracking_sha" == "$candidate_sha" ]] || fail "remote-tracking default branch differs from candidate"
remote_live_sha=$(git ls-remote origin "refs/heads/$remote_default_branch" | awk 'NR == 1 {print $1}')
[[ "$remote_live_sha" == "$candidate_sha" ]] || fail "live remote default branch differs from candidate"
local_workflow_blob=$(git rev-parse "$candidate_sha:.github/workflows/$WORKFLOW_FILE")
remote_workflow_blob=$(git rev-parse "$remote_live_sha:.github/workflows/$WORKFLOW_FILE")
[[ "$local_workflow_blob" == "$remote_workflow_blob" ]] || fail "remote workflow blob differs from candidate"
repository_slug=$(gh repo view --json nameWithOwner --jq '.nameWithOwner')
[[ "$repository_slug" =~ ^[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+$ ]] || fail "repository slug is invalid"
gh api "repos/$repository_slug/actions/workflows/$WORKFLOW_FILE" --jq '.path' |
	grep -Fxq ".github/workflows/$WORKFLOW_FILE" || fail "workflow API does not recognize the candidate workflow"

structural_commit=$(jq -er '.structural_source.commit' "$manifest_path")
git merge-base --is-ancestor "$structural_commit" "$candidate_sha" || fail "structural commit is not an ancestor"
manifest_sha256=$(hash_file "$manifest_path")

case "$output_argument" in
/*) output_root=$output_argument ;;
*) output_root="$repository_root/$output_argument" ;;
esac
validate_target_path "$output_root"
mkdir -p -- "$output_root"
output_root=$(cd -- "$output_root" && pwd -P)
output_directory="$output_root/$candidate_sha"
validate_target_path "$output_directory"
if [[ -e "$output_directory" ]]; then
	[[ -d "$output_directory" && ! -L "$output_directory" ]] || fail "candidate output is not a regular directory"
else
	mkdir -- "$output_directory"
fi
mkdir -p -- "$output_directory/logs" "$output_directory/spool"
output_directory=$(cd -- "$output_directory" && pwd -P)

pending_path="$output_directory/final-verification.json.pending"
terminal_path="$output_directory/final-verification.json"
records_path="$output_directory/command-records.jsonl"
intent_path="$output_directory/$INTENT_RELATIVE"
result_path="$output_directory/$RESULT_RELATIVE"
[[ ! -L "$pending_path" && ! -L "$terminal_path" && ! -L "$records_path" ]] || fail "producer state contains a symbolic link"

if [[ -e "$terminal_path" ]]; then
	[[ -f "$terminal_path" ]] || fail "terminal evidence is not a regular file"
	"$validator_path" "$manifest_path" "$terminal_path" "$repository_root" "$output_directory"
	printf 'phase13-1 gap verification: terminal evidence is already valid: %s\n' "$terminal_path"
	exit 0
fi
[[ ! -e "$pending_path" ]] || fail "partial terminal evidence requires reconciliation"

require_relative_regular_file() {
	local relative_path=$1
	local label=$2
	[[ -n "$relative_path" && "$relative_path" != /* ]] || fail "$label path must be relative"
	case "/$relative_path/" in
	*/../* | */./*) fail "$label path contains a traversal component" ;;
	esac
	local candidate_path="$output_directory/$relative_path"
	local current_path=$output_directory
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
	"$output_directory"/*) ;;
	*) fail "$label path escapes candidate output" ;;
	esac
	printf '%s\n' "$resolved_path"
}

atomic_json_write() {
	local destination=$1
	local value=$2
	local temporary
	temporary=$(mktemp "$destination.tmp.XXXXXX")
	jq -S . <<<"$value" >"$temporary"
	mv -- "$temporary" "$destination"
}

atomic_append_record() {
	local record=$1
	local temporary
	temporary=$(mktemp "$records_path.tmp.XXXXXX")
	if [[ -e "$records_path" ]]; then
		[[ -f "$records_path" && ! -L "$records_path" ]] || fail "command records are not a regular file"
		cp -- "$records_path" "$temporary"
	fi
	printf '%s\n' "$record" >>"$temporary"
	mv -- "$temporary" "$records_path"
}

canonical_run_id=0
expand_json() {
	jq -c \
		--arg candidate "$candidate_sha" \
		--arg tree "$candidate_tree" \
		--arg output "$output_directory" \
		--arg remote "$remote_default_branch" \
		--arg run "$canonical_run_id" \
		'def expand:
		  gsub("\\$\\{CANDIDATE\\}"; $candidate)
		  | gsub("\\$\\{CANDIDATE_TREE\\}"; $tree)
		  | gsub("\\$\\{OUTPUT_ROOT\\}"; $output)
		  | gsub("\\$\\{REMOTE_REF\\}"; $remote)
		  | gsub("\\$\\{CANONICAL_RUN_ID\\}"; $run);
		walk(if type == "string" then expand else . end)'
}

parsed_dispatch_url=
parsed_run_id=
parse_exact_dispatch_url() {
	local stdout_path=$1
	jq -Rse 'test("^https://github.com/[A-Za-z0-9_.-]+/[A-Za-z0-9_.-]+/actions/runs/[1-9][0-9]*\\n?$")' "$stdout_path" >/dev/null ||
		fail "dispatch URL run ID is invalid"
	parsed_dispatch_url=$(jq -Rrs 'rtrimstr("\n")' "$stdout_path")
	local expected_prefix="https://github.com/$repository_slug/actions/runs/"
	[[ "$parsed_dispatch_url" == "$expected_prefix"* ]] || fail "dispatch URL repository differs"
	parsed_run_id=${parsed_dispatch_url#"$expected_prefix"}
	[[ "$parsed_run_id" =~ ^[1-9][0-9]*$ ]] || fail "dispatch URL run ID is invalid"
}

dispatch_command_json() {
	jq -cer '.commands[] | select(.id == "canonical-dispatch")' "$manifest_path" | expand_json
}

expected_intent_json() {
	local dispatch_command
	dispatch_command=$(dispatch_command_json)
	jq -cnS \
		--arg candidate "$candidate_sha" --arg tree "$candidate_tree" \
		--arg repository "$repository_slug" --arg workflow "$WORKFLOW_FILE" \
		--arg ref "$remote_default_branch" --arg manifest "$manifest_sha256" \
		--argjson argv "$(jq -c '.argv' <<<"$dispatch_command")" \
		'{schema:"phase13-1-dispatch-intent-v1",candidate_sha:$candidate,candidate_tree:$tree,
		repository_slug:$repository,workflow_file:$workflow,remote_ref:$ref,
		dispatch_argv:$argv,manifest_sha256:$manifest}'
}

validate_dispatch_intent() {
	local intent_file
	intent_file=$(require_relative_regular_file "$INTENT_RELATIVE" "dispatch intent")
	[[ "$(jq -cS . "$intent_file")" == "$(expected_intent_json)" ]] || fail "dispatch intent journal differs"
}

record_digest() {
	jq -cS . <<<"$1" | hash_stream
}

expected_result_json() {
	local dispatch_record=$1
	local stdout_file=$2
	local intent_digest
	intent_digest=$(hash_file "$intent_path")
	jq -cnS \
		--arg candidate "$candidate_sha" --arg tree "$candidate_tree" \
		--arg repository "$repository_slug" --arg workflow "$WORKFLOW_FILE" \
		--arg ref "$remote_default_branch" --arg intent "$intent_digest" \
		--arg url "$parsed_dispatch_url" --arg run "$parsed_run_id" \
		--arg record "$(record_digest "$dispatch_record")" --arg log "$(hash_file "$stdout_file")" \
		'{schema:"phase13-1-dispatch-result-v1",candidate_sha:$candidate,candidate_tree:$tree,
		repository_slug:$repository,workflow_file:$workflow,remote_ref:$ref,
		intent_sha256:$intent,dispatch_url:$url,canonical_run_id:$run,
		command_id:"canonical-dispatch",command_record_sha256:$record,command_stdout_sha256:$log}'
}

validate_dispatch_result() {
	local dispatch_record=$1
	local stdout_file=$2
	local result_file
	result_file=$(require_relative_regular_file "$RESULT_RELATIVE" "dispatch result")
	[[ "$(jq -cS . "$result_file")" == "$(expected_result_json "$dispatch_record" "$stdout_file")" ]] ||
		fail "dispatch result journal differs"
}

recover_dispatch_result() {
	local dispatch_record=$1
	local stdout_file=$2
	validate_dispatch_intent
	parse_exact_dispatch_url "$stdout_file"
	if [[ -e "$result_path" ]]; then
		validate_dispatch_result "$dispatch_record" "$stdout_file"
	else
		atomic_json_write "$result_path" "$(expected_result_json "$dispatch_record" "$stdout_file")"
	fi
	canonical_run_id=$parsed_run_id
}

validate_run_view() {
	local stdout_file=$1
	local stage=$2
	local expected_url="https://github.com/$repository_slug/actions/runs/$canonical_run_id"
	if [[ "$stage" == initial ]]; then
		jq -e --arg candidate "$candidate_sha" --arg run "$canonical_run_id" --arg url "$expected_url" \
			'(.databaseId | tostring) == $run and .url == $url and .event == "workflow_dispatch"
			and .headSha == $candidate and (.status == "queued" or .status == "in_progress" or .status == "completed")
			and (if .status == "completed" then .conclusion == "success" else .conclusion == null end)' \
			"$stdout_file" >/dev/null || fail "canonical initial run view differs"
	else
		jq -e --arg candidate "$candidate_sha" --arg run "$canonical_run_id" --arg url "$expected_url" \
			'(.databaseId | tostring) == $run and .url == $url and .event == "workflow_dispatch"
			and .headSha == $candidate and .status == "completed" and .conclusion == "success"' \
			"$stdout_file" >/dev/null || fail "canonical terminal run view differs"
	fi
}

validate_existing_prefix() {
	existing_count=0
	[[ -e "$records_path" ]] || return 0
	[[ -f "$records_path" && ! -L "$records_path" ]] || fail "command records are not a regular file"
	jq -e -s 'all(type == "object")' "$records_path" >/dev/null || fail "retained command records contain partial JSON"
	existing_count=$(jq -s 'length' "$records_path")
	manifest_count=$(jq '.commands | length' "$manifest_path")
	[[ "$existing_count" -le "$manifest_count" ]] || fail "retained command records exceed the manifest"
	local index=0
	while [[ "$index" -lt "$existing_count" ]]; do
		local actual_record command_template expected_command actual_command command_id stdout_relative stderr_relative
		actual_record=$(jq -c -s ".[$index]" "$records_path")
		command_template=$(jq -c ".commands[$index]" "$manifest_path")
		expected_command=$(expand_json <<<"$command_template")
		actual_command=$(jq -cS '{id,argv,environment,stdout_log,stderr_log,evidence_class}' <<<"$actual_record")
		[[ "$actual_command" == "$(jq -cS '{id,argv,environment,stdout_log,stderr_log,evidence_class}' <<<"$expected_command")" ]] ||
			fail "retained command order, argv, environment, or paths differ"
		[[ "$(jq -er '.exit_code' <<<"$actual_record")" -eq 0 ]] || fail "retained command exit is nonzero"
		command_id=$(jq -er '.id' <<<"$actual_record")
		stdout_relative=$(jq -er '.stdout_log' <<<"$actual_record")
		stderr_relative=$(jq -er '.stderr_log' <<<"$actual_record")
		stdout_file=$(require_relative_regular_file "$stdout_relative" "$command_id stdout")
		stderr_file=$(require_relative_regular_file "$stderr_relative" "$command_id stderr")
		[[ "$(hash_file "$stdout_file")" == "$(jq -er '.stdout_sha256' <<<"$actual_record")" ]] || fail "retained stdout digest differs"
		[[ "$(hash_file "$stderr_file")" == "$(jq -er '.stderr_sha256' <<<"$actual_record")" ]] || fail "retained stderr digest differs"
		if [[ "$command_id" == canonical-dispatch ]]; then
			recover_dispatch_result "$actual_record" "$stdout_file"
		elif [[ "$command_id" == canonical-initial-view ]]; then
			validate_run_view "$stdout_file" initial
		elif [[ "$command_id" == canonical-inspect ]]; then
			validate_run_view "$stdout_file" terminal
		fi
		index=$((index + 1))
	done
}

validate_existing_prefix
dispatch_index=$(jq -er '[.commands[].id] | index("canonical-dispatch")' "$manifest_path")
if [[ -e "$intent_path" && "$existing_count" -le "$dispatch_index" ]]; then
	validate_dispatch_intent
	fail "dispatch reconciliation is required before another dispatch"
fi
if [[ -e "$result_path" && "$existing_count" -le "$dispatch_index" ]]; then
	fail "dispatch result exists without its command record"
fi

run_manifest_command() {
	local command_template=$1
	local command_json command_id stdout_relative stderr_relative stdout_path stderr_path
	command_json=$(expand_json <<<"$command_template")
	command_id=$(jq -er '.id' <<<"$command_json")
	if jq -e '.. | strings | test("\\$\\{[^}]+\\}")' <<<"$command_json" >/dev/null; then
		fail "$command_id retained an unexpanded placeholder"
	fi
	local -a command_argv=()
	while IFS= read -r argument; do
		command_argv+=("$argument")
	done < <(jq -r '.argv[]' <<<"$command_json")
	local -a environment_overrides=()
	while IFS= read -r override; do
		environment_overrides+=("$override")
	done < <(jq -r '.environment | to_entries[] | "\(.key)=\(.value)"' <<<"$command_json")
	stdout_relative=$(jq -er '.stdout_log' <<<"$command_json")
	stderr_relative=$(jq -er '.stderr_log' <<<"$command_json")
	stdout_path="$output_directory/$stdout_relative"
	stderr_path="$output_directory/$stderr_relative"
	[[ -d "$(dirname -- "$stdout_path")" && -d "$(dirname -- "$stderr_path")" ]] || fail "$command_id log destination is missing"
	local spool_stdout="$output_directory/spool/$command_id.stdout"
	local spool_stderr="$output_directory/spool/$command_id.stderr"
	printf 'phase13-1 gap verification: %s\n' "$command_id"
	set +e
	if ((${#environment_overrides[@]} > 0)); then
		env "${environment_overrides[@]}" "${command_argv[@]}" >"$spool_stdout" 2>"$spool_stderr"
	else
		env "${command_argv[@]}" >"$spool_stdout" 2>"$spool_stderr"
	fi
	local exit_code=$?
	set -e
	tail -c "$MAXIMUM_LOG_BYTES" "$spool_stdout" >"$stdout_path"
	tail -c "$MAXIMUM_LOG_BYTES" "$spool_stderr" >"$stderr_path"
	rm -f -- "$spool_stdout" "$spool_stderr"
	local record
	record=$(jq -cn \
		--argjson command "$command_json" --argjson exit_code "$exit_code" \
		--arg stdout_sha256 "$(hash_file "$stdout_path")" --arg stderr_sha256 "$(hash_file "$stderr_path")" \
		'$command + {exit_code: $exit_code, stdout_sha256: $stdout_sha256, stderr_sha256: $stderr_sha256}')
	atomic_append_record "$record"
	[[ "$exit_code" -eq 0 ]] || fail "$command_id failed"

	if [[ "$command_id" == canonical-dispatch ]]; then
		recover_dispatch_result "$record" "$stdout_path"
	elif [[ "$command_id" == canonical-initial-view ]]; then
		validate_run_view "$stdout_path" initial
	elif [[ "$command_id" == canonical-inspect ]]; then
		validate_run_view "$stdout_path" terminal
	fi
	[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] || fail "candidate changed during verification"
	[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || fail "worktree changed during verification"
}

manifest_count=$(jq '.commands | length' "$manifest_path")
command_index=$existing_count
while [[ "$command_index" -lt "$manifest_count" ]]; do
	command_template=$(jq -c ".commands[$command_index]" "$manifest_path")
	command_id=$(jq -er '.id' <<<"$command_template")
	if [[ "$command_id" == canonical-dispatch ]]; then
		[[ ! -e "$intent_path" && ! -e "$result_path" ]] || fail "dispatch reconciliation is required before another dispatch"
		atomic_json_write "$intent_path" "$(expected_intent_json)"
	fi
	run_manifest_command "$command_template"
	command_index=$((command_index + 1))
done

[[ "$canonical_run_id" =~ ^[1-9][0-9]*$ ]] || fail "canonical run ID was not established"
validate_dispatch_intent
dispatch_record=$(jq -c -s '.[] | select(.id == "canonical-dispatch")' "$records_path")
dispatch_stdout_relative=$(jq -er '.stdout_log' <<<"$dispatch_record")
dispatch_stdout_file=$(require_relative_regular_file "$dispatch_stdout_relative" "canonical dispatch stdout")
parse_exact_dispatch_url "$dispatch_stdout_file"
validate_dispatch_result "$dispatch_record" "$dispatch_stdout_file"
canonical_identity="$output_directory/canonical/identity.json"
[[ -f "$canonical_identity" && ! -L "$canonical_identity" ]] || fail "canonical identity is missing"

commands_json=$(jq -s '.' "$records_path")
rust_version=$(rustc --version)
cargo_version=$(cargo --version)
cmake_version=$(cmake --version | sed -n '1p')
ninja_version=$(ninja --version)
compiler_version=$(${CXX:-c++} --version | sed -n '1p')
host_os=$(uname -s)
host_architecture=$(uname -m)

terminal_json=$(jq -n \
	--arg candidate_sha "$candidate_sha" --arg candidate_tree "$candidate_tree" \
	--arg output_root "$output_directory" --arg remote_ref "$remote_default_branch" \
	--arg repository_slug "$repository_slug" --arg canonical_run_id "$canonical_run_id" \
	--arg manifest_sha256 "$manifest_sha256" --arg canonical_sha256 "$(hash_file "$canonical_identity")" \
	--arg intent_sha256 "$(hash_file "$intent_path")" --arg result_sha256 "$(hash_file "$result_path")" \
	--arg rust "$rust_version" --arg cargo "$cargo_version" --arg cmake "$cmake_version" \
	--arg ninja "$ninja_version" --arg cxx "$compiler_version" --arg os "$host_os" \
	--arg architecture "$host_architecture" --argjson commands "$commands_json" \
	'{
	  schema: "phase13-1-gap-verification-evidence-v1",
	  candidate_sha: $candidate_sha,
	  candidate_tree: $candidate_tree,
	  output_root: $output_root,
	  remote_ref: $remote_ref,
	  repository_slug: $repository_slug,
	  canonical_run_id: $canonical_run_id,
	  manifest_sha256: $manifest_sha256,
	  dispatch_intent: {path: "dispatch-intent.json", sha256: $intent_sha256},
	  dispatch_result: {path: "dispatch-result.json", sha256: $result_sha256},
	  environment: {os: $os, architecture: $architecture, tools: {rust: $rust, cargo: $cargo, cmake: $cmake, ninja: $ninja, cxx: $cxx}, presets: ["oracle-debug", "oracle-release"]},
	  structural_source: {commit: "981908ea87b6789b6b6e9aa136e65a369c5c736d", parent: "5394fd92ed036fe4f90358501848a61fa81cfc6d"},
	  merge_facts: {
	    merge_commit: "f4f5ca661cfb8549b115faa054489f24ba888ae9",
	    parents: ["be444a01e8649a4540effc147307a1041c4649ca", "96b24974eee054fd0afec71e6e14b7998ed7df01"],
	    merge_base: "9bf7abb0540f7cf42ed786be20c0d94a608f474e",
	    side_paths_disjoint: true,
	    override: {accepted_by: "pRizz", accepted_at: "2026-08-01T15:56:27Z"}
	  },
	  commands: $commands,
	  checker: {findings: 0, exceptions: 0},
	  canonical_identity: {path: "canonical/identity.json", sha256: $canonical_sha256},
	  complete: true
	}')
atomic_json_write "$pending_path" "$terminal_json"
"$validator_path" "$manifest_path" "$pending_path" "$repository_root" "$output_directory"
mv -- "$pending_path" "$terminal_path"
printf 'phase13-1 gap verification complete: %s\n' "$terminal_path"
