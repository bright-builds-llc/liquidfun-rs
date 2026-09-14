#!/usr/bin/env bash
set -euo pipefail

mode=$1
export PHASE12_MIRI_LIBRARY_ONLY=1
source scripts/phase12-miri.sh

case "$mode" in
missing | error | finding)
	rg() {
		case "$mode" in
		missing) return 127 ;;
		error) return 2 ;;
		finding) return 0 ;;
		esac
	}
	scan_source
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
