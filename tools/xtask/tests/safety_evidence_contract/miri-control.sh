#!/usr/bin/env bash
set -euo pipefail

mode=$1
export PHASE12_MIRI_LIBRARY_ONLY=1
source scripts/phase12-miri.sh

case "$mode" in
missing)
	PATH=/nonexistent scan_source
	;;
error | finding)
	rg() {
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
