#!/usr/bin/env bash
set -euo pipefail

readonly MAXIMUM_LOG_BYTES=$((1024 * 1024))
readonly WORKFLOW_FILE=phase13-1-canonical-native.yml

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
# The common library supplies the same platform-safe SHA helper used by release evidence.
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
gh api "repos/$repository_slug/actions/workflows/$WORKFLOW_FILE" --jq '.path' |
	grep -Fxq ".github/workflows/$WORKFLOW_FILE" || fail "workflow API does not recognize the candidate workflow"

structural_commit=$(jq -er '.structural_source.commit' "$manifest_path")
git merge-base --is-ancestor "$structural_commit" "$candidate_sha" || fail "structural commit is not an ancestor"

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
	rm -rf -- "$output_directory"
fi
mkdir -p -- "$output_directory/logs" "$output_directory/spool"
output_directory=$(cd -- "$output_directory" && pwd -P)

pending_path="$output_directory/final-verification.json.pending"
terminal_path="$output_directory/final-verification.json"
records_path="$output_directory/command-records.jsonl"
published=0
cleanup_terminal_on_failure() {
	if [[ "$published" -ne 1 ]]; then
		rm -f -- "$pending_path" "$terminal_path"
	fi
}
trap cleanup_terminal_on_failure EXIT

canonical_run_id=${PHASE13_1_GAP_TEST_RUN_ID:-0}
if [[ "$test_fixture" != true && -n "${PHASE13_1_GAP_TEST_RUN_ID:-}" ]]; then
	fail "test run ID is forbidden for the production manifest"
fi

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

run_manifest_command() {
	local command_template=$1
	local command_json
	command_json=$(expand_json <<<"$command_template")
	local command_id
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
	local stdout_relative stderr_relative stdout_path stderr_path
	stdout_relative=$(jq -er '.stdout_log' <<<"$command_json")
	stderr_relative=$(jq -er '.stderr_log' <<<"$command_json")
	stdout_path="$output_directory/$stdout_relative"
	stderr_path="$output_directory/$stderr_relative"
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
	jq -cn \
		--argjson command "$command_json" \
		--argjson exit_code "$exit_code" \
		--arg stdout_sha256 "$(hash_file "$stdout_path")" \
		--arg stderr_sha256 "$(hash_file "$stderr_path")" \
		'$command + {exit_code: $exit_code, stdout_sha256: $stdout_sha256, stderr_sha256: $stderr_sha256}' >>"$records_path"
	[[ "$exit_code" -eq 0 ]] || fail "$command_id failed"

	if [[ "$command_id" == canonical-discover ]]; then
		canonical_run_id=$(jq -er --arg candidate "$candidate_sha" \
			'[.[] | select(.headSha == $candidate)] | first | .databaseId' "$stdout_path")
		[[ "$canonical_run_id" =~ ^[1-9][0-9]*$ ]] || fail "canonical run discovery failed"
	fi
	if [[ "$command_id" == canonical-inspect ]]; then
		jq -e --arg candidate "$candidate_sha" \
			'.headSha == $candidate and .status == "completed" and .conclusion == "success"' "$stdout_path" >/dev/null ||
			fail "canonical run identity or conclusion differs"
	fi
	[[ "$(git rev-parse HEAD)" == "$candidate_sha" ]] || fail "candidate changed during verification"
	[[ -z "$(git status --porcelain=v1 --untracked-files=all)" ]] || fail "worktree changed during verification"
}

: >"$records_path"
while IFS= read -r encoded_command; do
	command_template=$(printf '%s' "$encoded_command" | base64 --decode)
	run_manifest_command "$command_template"
done < <(jq -r '.commands[] | @base64' "$manifest_path")

[[ "$canonical_run_id" =~ ^[1-9][0-9]*$ ]] || fail "canonical run ID was not established"
canonical_identity="$output_directory/canonical/identity.json"
[[ -f "$canonical_identity" && ! -L "$canonical_identity" ]] || fail "canonical identity is missing"

commands_json=$(jq -s '.' "$records_path")
manifest_sha256=$(hash_file "$manifest_path")
rust_version=$(rustc --version)
cargo_version=$(cargo --version)
cmake_version=$(cmake --version | sed -n '1p')
ninja_version=$(ninja --version)
compiler_version=$(${CXX:-c++} --version | sed -n '1p')
host_os=$(uname -s)
host_architecture=$(uname -m)

jq -n \
	--arg candidate_sha "$candidate_sha" \
	--arg candidate_tree "$candidate_tree" \
	--arg output_root "$output_directory" \
	--arg remote_ref "$remote_default_branch" \
	--arg canonical_run_id "$canonical_run_id" \
	--arg manifest_sha256 "$manifest_sha256" \
	--arg canonical_sha256 "$(hash_file "$canonical_identity")" \
	--arg rust "$rust_version" --arg cargo "$cargo_version" \
	--arg cmake "$cmake_version" --arg ninja "$ninja_version" --arg cxx "$compiler_version" \
	--arg os "$host_os" --arg architecture "$host_architecture" \
	--argjson commands "$commands_json" \
	'{
	  schema: "phase13-1-gap-verification-evidence-v1",
	  candidate_sha: $candidate_sha,
	  candidate_tree: $candidate_tree,
	  output_root: $output_root,
	  remote_ref: $remote_ref,
	  canonical_run_id: $canonical_run_id,
	  manifest_sha256: $manifest_sha256,
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
	}' >"$pending_path"

"$validator_path" "$manifest_path" "$pending_path" "$repository_root" "$output_directory"
mv -- "$pending_path" "$terminal_path"
published=1
rm -f -- "$records_path"
printf 'phase13-1 gap verification complete: %s\n' "$terminal_path"
