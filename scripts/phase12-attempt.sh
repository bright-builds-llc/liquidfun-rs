#!/usr/bin/env bash
set -euo pipefail

begin_attempt() {
	ATTEMPT_OUTPUT=$1
	ATTEMPT_CANDIDATE=$2
	ATTEMPT_STARTED=$SECONDS
	local run=${GITHUB_RUN_ID:-0} attempt=${GITHUB_RUN_ATTEMPT:-1}
	[[ "$run" =~ ^[0-9]+$ && "$attempt" =~ ^[1-9][0-9]*$ ]] || fail "invalid run/attempt identity"
	mkdir -- "$ATTEMPT_OUTPUT/diagnostics"
	mkdir -- "$ATTEMPT_OUTPUT/diagnostics/logs"
	: >"$ATTEMPT_OUTPUT/diagnostics/commands.jsonl"
	trap 'attempt_exit "$?"' EXIT
}

seal_attempt() {
	local exit_code=$1
	local status=failed
	if ((exit_code == 0)); then status=passed; fi
	local diagnostics="$ATTEMPT_OUTPUT/diagnostics"
	jq -n --arg candidate "$ATTEMPT_CANDIDATE" --arg status "$status" \
		--arg workflow "${GITHUB_WORKFLOW:-local}" --arg job "${GITHUB_JOB:-local}" \
		--argjson run "${GITHUB_RUN_ID:-0}" --argjson attempt "${GITHUB_RUN_ATTEMPT:-1}" \
		--argjson exit_code "$exit_code" --argjson elapsed "$((SECONDS - ATTEMPT_STARTED))" \
		'{schema_version:1,candidate_commit:$candidate,status:$status,producer_workflow:$workflow,
		  producer_job:$job,run_id:$run,run_attempt:$attempt,exit_code:$exit_code,elapsed_seconds:$elapsed}' \
		>"$diagnostics/terminal.json"
	(
		cd "$diagnostics"
		find . -type f ! -path ./checksums.sha256 -print0 | LC_ALL=C sort -z | xargs -0 sha256sum
	) >"$diagnostics/checksums.sha256"
}

attempt_exit() {
	local exit_code=$1
	trap - EXIT
	if ((exit_code != 0)); then
		if [[ -e "$ATTEMPT_OUTPUT/identity.json" ]]; then
			mv "$ATTEMPT_OUTPUT/identity.json" "$ATTEMPT_OUTPUT/diagnostics/failed-identity.json"
		fi
		seal_attempt "$exit_code"
	elif [[ ! -f "$ATTEMPT_OUTPUT/diagnostics/terminal.json" ]]; then
		seal_attempt 0
	fi
	exit "$exit_code"
}

run_attempt_command() {
	local name=$1 seconds=$2
	shift 2
	local log="$ATTEMPT_OUTPUT/diagnostics/logs/$name.log"
	[[ ! -e "$log" ]] || fail "diagnostic command already exists"
	local maximum=16777216 started=$SECONDS
	local -a statuses
	if {
		if [[ -n "${ATTEMPT_STDOUT:-}" ]]; then
			timeout --signal=TERM --kill-after=10s "${seconds}s" "$@" >"$ATTEMPT_STDOUT"
		else
			timeout --signal=TERM --kill-after=10s "${seconds}s" "$@"
		fi
	} 2>&1 | head -c "$maximum" >"$log"; then
		statuses=("${PIPESTATUS[@]}")
	else
		statuses=("${PIPESTATUS[@]}")
	fi
	local bytes classification=passed elapsed=$((SECONDS - started))
	bytes=$(wc -c <"$log")
	if ((bytes >= maximum)); then
		classification=log_limit
	elif (((statuses[0] == 124 || statuses[0] == 137) && elapsed >= seconds)); then
		classification=timeout
	elif ((statuses[0] != 0 || statuses[1] != 0)); then
		classification=command_failed
	fi
	jq -cn --arg name "$name" --arg classification "$classification" \
		--arg path "logs/$name.log" --arg sha256 "$(hash_file "$log")" \
		--argjson command_exit_code "${statuses[0]}" --argjson capture_exit_code "${statuses[1]}" \
		--argjson retained_bytes "$bytes" --argjson elapsed "$elapsed" \
		--argjson timeout "$seconds" \
		'{name:$name,classification:$classification,path:$path,sha256:$sha256,
		  command_exit_code:$command_exit_code,capture_exit_code:$capture_exit_code,
		  retained_bytes:$retained_bytes,elapsed_seconds:$elapsed,timeout_seconds:$timeout,command:$ARGS.positional}' \
		--args -- "$@" >>"$ATTEMPT_OUTPUT/diagnostics/commands.jsonl"
	if [[ "$classification" != passed ]]; then
		tail -c 65536 "$log" >&2
		fail "command $name failed: $classification (exit ${statuses[0]})"
	fi
}
