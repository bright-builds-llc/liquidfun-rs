#!/usr/bin/env bash
set -euo pipefail

mode=$1
export PHASE12_MIRI_LIBRARY_ONLY=1
source scripts/phase12-miri.sh

case "$mode" in
scan-clean | scan-cmake | scan-oracle | scan-third_party | scan-submodule | scan-read-error)
	fixture=$(mktemp -d)
	trap 'rm -rf "$fixture"' EXIT
	script_directory=$fixture
	if [[ "$mode" != scan-read-error ]]; then
		printf '%s\n' "${mode#scan-}" | tr '[:lower:]' '[:upper:]' >"$fixture/phase12-miri.sh"
	fi
	scan_source
	;;
case-budgets)
	fixture=$(mktemp -d)
	trap 'rm -rf "$fixture"' EXIT
	mkdir -p "$fixture/logs"
	begin_attempt "$fixture" 1111111111111111111111111111111111111111
	compiler_identity='rustc fixture'
	for case_name in arena_handles particle_group_model; do
		run_case "$fixture" "$fixture/cases.jsonl" "$case_name" "" \
			printf '%s\n' 'test result: ok. 1 passed; 0 failed; 0 ignored; 0 measured; 0 filtered out; finished in 0.00s'
	done
	jq -se '.[0].timeout_seconds == 900 and .[1].timeout_seconds == 3600' "$fixture/cases.jsonl" >/dev/null
	;;
missing)
	PATH=/nonexistent scan_source
	;;
error | finding)
	grep() {
		case "$mode" in
		error) return 2 ;;
		finding) return 0 ;;
		esac
	}
	scan_source
	;;
lost-invocation)
	fixture=$(mktemp -d)
	trap 'rm -rf "$fixture"' EXIT
	validate_candidate() { return 0; }
	prepare_output() { printf '%s\n' "$fixture"; }
	rustc() { printf 'rustc test fixture\n'; }
	cargo() { return 0; }
	run_case() {
		[[ "$3" != math_endpoint ]] || return 0
		jq -cn --arg name "$3" --arg flags "$4" \
			'{name: $name, miriflags: $flags, passed: (if $name == "math" then 44 else 1 end), ignored: 0}' >>"$2"
	}
	status=0
	(run_miri 1111111111111111111111111111111111111111) || status=$?
	[[ "$status" == 64 && ! -e "$fixture/identity.json" ]]
	;;
*)
	fixture=$(mktemp)
	trap 'rm -f "$fixture"' EXIT
	jq -n --arg mode "$mode" '{cases: ([
	  {name: "math", passed: (if $mode == "zero-default" then 0 else 44 end), ignored: 0, miriflags: ""},
	  {name: "math_endpoint", passed: 1, ignored: 0, miriflags: (if $mode == "wrong-flags" then "" else "-Zmiri-no-extra-rounding-error" end)}
	] | if $mode == "missing-endpoint" then .[:1] else . end)}' >"$fixture"
	validate_math_modes "$fixture"
	;;
esac
